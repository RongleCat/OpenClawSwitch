use flate2::read::GzDecoder;
use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use tauri::{AppHandle, Manager};

// ============================================================================
// 类型定义
// ============================================================================

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub meets_requirement: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FnmStatus {
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub shell: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallLogEvent {
    pub step: String,
    pub message: String,
    pub level: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgressEvent {
    pub current_step: u8,
    pub total_steps: u8,
    pub step_name: String,
    pub status: String,
}

/// 环境检测综合结果
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentStatus {
    pub openclaw: OpenClawStatus,
    pub node: NodeStatus,
    pub git: GitStatus,
    pub fnm: FnmStatus,
    pub system: SystemInfo,
    pub network_region: String,
}

/// 下载进度事件
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallDownloadEvent {
    pub step: String,
    pub percent: u8,
    pub speed: String,
    pub downloaded: u64,
    pub total: u64,
}

/// 步骤耗时事件
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallStepTimingEvent {
    pub step: String,
    pub start_time: u64,
    pub end_time: u64,
    pub duration: u64,
}

/// 工作台实时日志事件
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeLogEvent {
    pub message: String,
    pub level: String,
    pub timestamp: u64,
}

/// 工作台日志跟踪状态事件
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeLogStatusEvent {
    pub running: bool,
    pub reason: Option<String>,
}

/// 服务诊断任务状态事件
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DoctorStatusEvent {
    pub running: bool,
    pub mode: String,
    pub success: Option<bool>,
    pub exit_code: Option<i32>,
    pub reason: Option<String>,
}

/// 消息渠道扩展安装状态
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChannelExtensionStatus {
    pub feishu_installed: bool,
    pub wecom_installed: bool,
    pub qq_installed: bool,
    pub dingtalk_installed: bool,
}

/// 消息渠道扩展安装流程状态事件
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChannelExtensionInstallStateEvent {
    pub channel_id: String,
    pub status: String,
}

// ============================================================================
// 镜像源配置
// ============================================================================

const NODE_MIRRORS: &[&str] = &[
    "https://npmmirror.com/mirrors/node",
    "https://mirrors.cloud.tencent.com/nodejs-release/",
    "https://repo.huaweicloud.com/nodejs/",
];

const NPM_REGISTRIES: &[&str] = &[
    "https://registry.npmmirror.com",
    "https://mirrors.cloud.tencent.com/npm/",
    "https://registry.npmjs.org",
];

// GitHub 镜像源（多源容错）
const FNM_MIRRORS: &[&str] = &[
    "https://kkgithub.com/Schniz/fnm/releases/latest/download",
    "https://ghfast.top/https://github.com/Schniz/fnm/releases/latest/download",
    "https://ghproxy.com/https://github.com/Schniz/fnm/releases/latest/download",
    "https://github.com/Schniz/fnm/releases/latest/download", // 直连作为最后备选
];

const MANAGED_NODE_VERSION: &str = "22.22.0";
const OPENCLAW_MANAGED_PATH_MARKER_START: &str = "# >>> openclaw managed runtime >>>";
const OPENCLAW_MANAGED_PATH_MARKER_END: &str = "# <<< openclaw managed runtime <<<";
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// 下载超时（秒）
const DOWNLOAD_TIMEOUT_SECS: u64 = 30;
static LOG_FOLLOW_RUNNING: AtomicBool = AtomicBool::new(false);
static DOCTOR_RUNNING: AtomicBool = AtomicBool::new(false);
static CHANNEL_EXTENSION_INSTALLING: AtomicBool = AtomicBool::new(false);

// ============================================================================
// 辅助函数
// ============================================================================

/// 获取当前时间戳（毫秒）
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 发送安装日志事件
fn emit_log(app: &AppHandle, step: &str, message: &str, level: &str) {
    let _ = app.emit_all(
        "install-log",
        InstallLogEvent {
            step: step.to_string(),
            message: message.to_string(),
            level: level.to_string(),
            timestamp: now_ms(),
        },
    );
}

/// 发送安装进度事件
fn emit_progress(app: &AppHandle, current: u8, total: u8, name: &str, status: &str) {
    let _ = app.emit_all(
        "install-progress",
        InstallProgressEvent {
            current_step: current,
            total_steps: total,
            step_name: name.to_string(),
            status: status.to_string(),
        },
    );
}

/// 发送下载进度事件
fn emit_download(app: &AppHandle, step: &str, percent: u8, speed: &str, downloaded: u64, total: u64) {
    let _ = app.emit_all(
        "install-download",
        InstallDownloadEvent {
            step: step.to_string(),
            percent,
            speed: speed.to_string(),
            downloaded,
            total,
        },
    );
}

/// 发送步骤耗时事件
fn emit_timing(app: &AppHandle, step: &str, start: u64, end: u64) {
    let _ = app.emit_all(
        "install-step-timing",
        InstallStepTimingEvent {
            step: step.to_string(),
            start_time: start,
            end_time: end,
            duration: end - start,
        },
    );
}

/// 发送工作台实时日志行事件
fn emit_runtime_log(app: &AppHandle, message: &str, level: &str) {
    let _ = app.emit_all(
        "openclaw-log-line",
        RealtimeLogEvent {
            message: message.to_string(),
            level: level.to_string(),
            timestamp: now_ms(),
        },
    );
}

/// 发送工作台日志跟踪状态事件
fn emit_runtime_log_status(app: &AppHandle, running: bool, reason: Option<String>) {
    let _ = app.emit_all(
        "openclaw-log-status",
        RealtimeLogStatusEvent {
            running,
            reason,
        },
    );
}

/// 发送服务诊断日志行事件
fn emit_doctor_log(app: &AppHandle, message: &str, level: &str) {
    let _ = app.emit_all(
        "openclaw-doctor-line",
        RealtimeLogEvent {
            message: message.to_string(),
            level: level.to_string(),
            timestamp: now_ms(),
        },
    );
}

/// 发送服务诊断状态事件
fn emit_doctor_status(
    app: &AppHandle,
    running: bool,
    mode: &str,
    success: Option<bool>,
    exit_code: Option<i32>,
    reason: Option<String>,
) {
    let _ = app.emit_all(
        "openclaw-doctor-status",
        DoctorStatusEvent {
            running,
            mode: mode.to_string(),
            success,
            exit_code,
            reason,
        },
    );
}

fn detect_doctor_log_level(line: &str, from_stderr: bool) -> &'static str {
    let lower = line.to_lowercase();
    let error_signal = lower.contains("error")
        || lower.contains("fatal")
        || lower.contains("exception")
        || lower.contains("panic")
        || lower.contains("failed")
        || lower.contains("fail")
        || lower.contains("denied")
        || lower.contains("unauthorized")
        || lower.contains("forbidden")
        || lower.contains("refused")
        || lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("invalid")
        || lower.contains("cannot")
        || lower.contains("unable")
        || lower.contains("enoent")
        || lower.contains("econn")
        || lower.contains("错误")
        || lower.contains("失败")
        || lower.contains("异常")
        || lower.contains("✗")
        || lower.contains("×");

    if error_signal {
        return "error";
    }

    let warn_signal = lower.contains("warn")
        || lower.contains("warning")
        || lower.contains("deprecated")
        || lower.contains("建议")
        || lower.contains("警告");
    if warn_signal {
        return "warn";
    }

    let success_signal = lower.contains("success")
        || lower.contains("healthy")
        || lower.contains("passed")
        || lower.contains("完成")
        || lower.contains("✓");
    if success_signal {
        return "success";
    }

    if from_stderr {
        "warn"
    } else {
        "info"
    }
}

/// 发送渠道扩展安装日志事件（沿用 install-log 结构，前端可复用日志面板）
fn emit_channel_extension_log(app: &AppHandle, channel_id: &str, message: &str, level: &str) {
    let _ = app.emit_all(
        "channel-extension-install-log",
        InstallLogEvent {
            step: channel_id.to_string(),
            message: message.to_string(),
            level: level.to_string(),
            timestamp: now_ms(),
        },
    );
}

/// 发送渠道扩展安装状态事件
fn emit_channel_extension_state(app: &AppHandle, channel_id: &str, status: &str) {
    let _ = app.emit_all(
        "channel-extension-install-state",
        ChannelExtensionInstallStateEvent {
            channel_id: channel_id.to_string(),
            status: status.to_string(),
        },
    );
}

fn is_openclaw_package_root(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    if !name.eq_ignore_ascii_case("openclaw") {
        return false;
    }

    let parent_is_node_modules = path
        .parent()
        .and_then(|value| value.file_name())
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("node_modules"))
        .unwrap_or(false);

    parent_is_node_modules && path.join("package.json").is_file()
}

fn parse_path_line(line: &str) -> Option<PathBuf> {
    let value = line.trim().trim_matches('"').trim_matches('\'');
    if value.is_empty() {
        return None;
    }
    Some(PathBuf::from(value))
}

fn detect_openclaw_root_from_npm_ls() -> Option<PathBuf> {
    let npm = npm_executable();
    let cmd = format!("{} ls -g openclaw --parseable --depth=0", npm);
    let output = run_shell(&with_fnm_env(&cmd)).ok()?;
    for line in output.lines().rev() {
        let Some(candidate) = parse_path_line(line) else {
            continue;
        };
        if is_openclaw_package_root(&candidate) {
            return Some(candidate);
        }
        if let Ok(resolved) = std::fs::canonicalize(&candidate) {
            if is_openclaw_package_root(&resolved) {
                return Some(resolved);
            }
        }
    }
    None
}

fn detect_openclaw_root_from_npm_root() -> Option<PathBuf> {
    let npm = npm_executable();
    let cmd = format!("{} root -g", npm);
    let output = run_shell(&with_fnm_env(&cmd)).ok()?;
    for line in output.lines().rev() {
        let Some(root) = parse_path_line(line) else {
            continue;
        };
        let candidate = root.join("openclaw");
        if is_openclaw_package_root(&candidate) {
            return Some(candidate);
        }
        if let Ok(resolved) = std::fs::canonicalize(&candidate) {
            if is_openclaw_package_root(&resolved) {
                return Some(resolved);
            }
        }
    }
    None
}

fn find_openclaw_root_in_ancestors(path: &Path) -> Option<PathBuf> {
    for ancestor in path.ancestors() {
        if is_openclaw_package_root(ancestor) {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn prefer_windows_command_wrapper(candidate: &Path) -> PathBuf {
    let extension = candidate
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_default();

    if matches!(extension.as_str(), "cmd" | "exe" | "bat") {
        return candidate.to_path_buf();
    }

    for preferred_extension in ["cmd", "exe", "bat"] {
        let preferred = candidate.with_extension(preferred_extension);
        if preferred.exists() {
            return preferred;
        }
    }

    candidate.to_path_buf()
}

fn detect_openclaw_bin_path() -> Option<PathBuf> {
    let locate_cmd = if cfg!(target_os = "windows") {
        "where openclaw"
    } else {
        "command -v openclaw || which openclaw"
    };
    let output = run_shell(&with_fnm_env(locate_cmd)).ok()?;
    for line in output.lines() {
        let Some(candidate) = parse_path_line(line) else {
            continue;
        };
        if candidate.exists() {
            if cfg!(target_os = "windows") {
                return Some(prefer_windows_command_wrapper(&candidate));
            }
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod installer_tests {
    use super::{
        append_windows_user_path_entry,
        build_default_openclaw_config,
        build_windows_elevated_powershell_command,
        build_windows_gateway_service_install_script,
        build_windows_nssm_service_install_args,
        build_windows_relaunch_as_admin_command,
        git_install_is_blocking_in_full_install,
        prefer_windows_command_wrapper,
        windows_git_path_entries,
    };
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("openclawswitch-{name}-{unique}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn prefer_windows_command_wrapper_uses_cmd_for_extensionless_entry() {
        let dir = make_temp_dir("cmd-wrapper");
        let extensionless = dir.join("openclaw");
        let cmd_wrapper = dir.join("openclaw.cmd");
        std::fs::write(&extensionless, "shim").expect("write extensionless shim");
        std::fs::write(&cmd_wrapper, "@echo off").expect("write cmd shim");

        let selected = prefer_windows_command_wrapper(&extensionless);
        assert_eq!(selected, cmd_wrapper);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn prefer_windows_command_wrapper_rewrites_ps1_to_cmd_when_available() {
        let dir = make_temp_dir("ps1-wrapper");
        let powershell_wrapper = dir.join("openclaw.ps1");
        let cmd_wrapper = dir.join("openclaw.cmd");
        std::fs::write(&powershell_wrapper, "Write-Host test").expect("write powershell shim");
        std::fs::write(&cmd_wrapper, "@echo off").expect("write cmd shim");

        let selected = prefer_windows_command_wrapper(&powershell_wrapper);
        assert_eq!(selected, cmd_wrapper);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn nssm_command_keeps_executable_and_script_paths_as_single_args() {
        let service_name = "OpenClaw Gateway";
        let cmd_exe = r"C:\Windows\System32\cmd.exe";
        let script_path = PathBuf::from(r"C:\Program Files\OpenClaw Switch\gateway-service.cmd");

        let args = build_windows_nssm_service_install_args(service_name, cmd_exe, &script_path);

        assert_eq!(args[0], "install");
        assert_eq!(args[1], service_name);
        assert_eq!(args[2], cmd_exe);
        assert_eq!(args[3], "/d");
        assert_eq!(args[4], "/s");
        assert_eq!(args[5], "/c");
        assert_eq!(args[6], format!("\"{}\"", script_path.display()));
    }

    #[test]
    fn decode_command_output_reads_utf16le_console_output() {
        let bytes = "Administrator access is needed to install a service."
            .encode_utf16()
            .flat_map(|unit| unit.to_le_bytes())
            .collect::<Vec<_>>();

        assert_eq!(super::decode_command_output(&bytes), "Administrator access is needed to install a service.");
    }

    #[test]
    fn elevated_powershell_command_uses_runas_and_file_script() {
        let script_path = PathBuf::from(r"C:\Users\Ronglecat\AppData\Local\Temp\install-gateway-service.ps1");
        let args = build_windows_elevated_powershell_command(&script_path);

        assert!(args.iter().any(|arg| arg.contains("-Verb RunAs")));
        assert!(args.iter().any(|arg| arg.contains("install-gateway-service.ps1")));
    }

    #[test]
    fn elevated_gateway_script_contains_result_file_and_nssm_commands() {
        let script = build_windows_gateway_service_install_script(
            Path::new(r"C:\tools\nssm.exe"),
            "OpenClaw Gateway",
            r"C:\Windows\System32\cmd.exe",
            Path::new(r"C:\Users\Ronglecat\.openclaw\service\gateway-service.cmd"),
            Path::new(r"C:\Users\Ronglecat"),
            Path::new(r"C:\Users\Ronglecat\.openclaw\logs\gateway-service.stdout.log"),
            Path::new(r"C:\Users\Ronglecat\.openclaw\logs\gateway-service.stderr.log"),
            Path::new(r"C:\Users\Ronglecat\.openclaw\service\install-gateway-service.result.txt"),
        );

        assert!(script.contains("Set-Content -Path $resultPath -Value 'ok'"));
        assert!(script.contains("'install', 'OpenClaw Gateway'"));
        assert!(script.contains("'start', 'OpenClaw Gateway'"));
    }

    #[test]
    fn relaunch_as_admin_command_uses_current_exe_and_runas() {
        let exe_path = Path::new(r"C:\Program Files\OpenClaw Switch\openclawswitch.exe");
        let args = build_windows_relaunch_as_admin_command(exe_path);

        assert_eq!(args[0], "-NoProfile");
        assert_eq!(args[1], "-ExecutionPolicy");
        assert_eq!(args[2], "Bypass");
        assert_eq!(args[3], "-Command");
        assert!(args[4].contains("Start-Process"));
        assert!(args[4].contains("-Verb RunAs"));
        assert!(args[4].contains("openclawswitch.exe"));
    }

    #[test]
    fn default_config_keeps_channels_empty_until_user_selects_one() {
        let config = build_default_openclaw_config("openclaw-test-token");

        assert!(config.get("channels").is_none());
        assert_eq!(config["gateway"]["auth"]["token"], "openclaw-test-token");
        assert_eq!(config["models"]["mode"], "merge");
    }

    #[test]
    fn append_windows_user_path_entry_ignores_case_whitespace_and_trailing_slash() {
        let target = Path::new(r"C:\Users\Ronglecat\.openclaw\npm-global");
        let current = format!(
            " C:\\tools ; {}\\ ; C:\\Windows\\System32 ",
            target.display()
        );

        let updated = append_windows_user_path_entry(&current, target);

        assert!(updated.is_none());
    }

    #[test]
    fn append_windows_user_path_entry_appends_missing_entry_once() {
        let target = Path::new(r"C:\Users\Ronglecat\.openclaw\npm-global");
        let current = r"C:\tools;C:\Windows\System32";

        let updated = append_windows_user_path_entry(current, target)
            .expect("missing entry should be appended");

        assert_eq!(
            updated,
            format!(r"C:\tools;C:\Windows\System32;{}", target.display())
        );
    }

    #[test]
    fn windows_git_path_entries_include_root_and_executable_dirs() {
        let dir = make_temp_dir("bundled-git");
        let cmd_dir = dir.join("cmd");
        let mingw_bin_dir = dir.join("mingw64").join("bin");
        let usr_bin_dir = dir.join("usr").join("bin");
        std::fs::create_dir_all(&cmd_dir).expect("create cmd dir");
        std::fs::create_dir_all(&mingw_bin_dir).expect("create mingw64 bin dir");
        std::fs::create_dir_all(&usr_bin_dir).expect("create usr bin dir");
        std::fs::write(cmd_dir.join("git.exe"), "git").expect("write cmd git");
        std::fs::write(mingw_bin_dir.join("git.exe"), "git").expect("write mingw64 git");

        let entries = windows_git_path_entries(&dir);

        assert_eq!(
            entries,
            vec![
                dir.clone(),
                cmd_dir.clone(),
                mingw_bin_dir.clone(),
                usr_bin_dir.clone(),
            ]
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn windows_full_install_treats_git_as_required() {
        assert!(git_install_is_blocking_in_full_install());
    }
}

fn detect_openclaw_root_from_bin_path() -> Option<PathBuf> {
    let bin_path = detect_openclaw_bin_path()?;
    let resolved_bin_path = std::fs::canonicalize(&bin_path).unwrap_or_else(|_| bin_path.clone());

    if let Some(root) = find_openclaw_root_in_ancestors(&resolved_bin_path) {
        return Some(root);
    }
    if let Some(root) = find_openclaw_root_in_ancestors(&bin_path) {
        return Some(root);
    }

    let mut candidates = Vec::new();
    for path in [&bin_path, &resolved_bin_path] {
        if let Some(parent) = path.parent() {
            candidates.push(parent.join("node_modules").join("openclaw"));
            candidates.push(parent.join("..").join("node_modules").join("openclaw"));
            candidates.push(parent.join("..").join("lib").join("node_modules").join("openclaw"));
        }
    }

    for candidate in candidates {
        if is_openclaw_package_root(&candidate) {
            return Some(candidate);
        }
        if let Ok(resolved) = std::fs::canonicalize(&candidate) {
            if is_openclaw_package_root(&resolved) {
                return Some(resolved);
            }
        }
    }

    None
}

fn detect_openclaw_package_root() -> Result<PathBuf, String> {
    if let Some(path) = detect_openclaw_root_from_npm_ls() {
        return Ok(path);
    }
    if let Some(path) = detect_openclaw_root_from_npm_root() {
        return Ok(path);
    }
    if let Some(path) = detect_openclaw_root_from_bin_path() {
        return Ok(path);
    }

    Err("未能定位 openclaw 的 npm 安装目录，请确认 openclaw 可通过 npm 全局访问".to_string())
}

fn get_extensions_root() -> Result<PathBuf, String> {
    let openclaw_root = detect_openclaw_package_root()?;
    Ok(openclaw_root.join("extensions"))
}

fn get_extension_meta(channel_id: &str) -> Result<(&'static str, &'static str, &'static str), String> {
    match channel_id {
        "feishu" => Ok(("@larksuiteoapi/feishu-openclaw-plugin", "feishu", "@larksuiteoapi/feishu-openclaw-plugin")),
        "wecom" => Ok(("@wecom/wecom-openclaw-plugin", "wecom-openclaw-plugin", "@wecom/wecom-openclaw-plugin")),
        "qq" => Ok(("@sliverp/qqbot", "qqbot", "@sliverp/qqbot")),
        "dingtalk" => Ok(("@dingtalk-real-ai/dingtalk-connector", "dingtalk", "@dingtalk-real-ai/dingtalk-connector")),
        _ => Err(format!("不支持的渠道扩展: {}", channel_id)),
    }
}

fn read_extension_package_name(package_json_path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(package_json_path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&raw).ok()?;
    parsed.get("name")?.as_str().map(|value| value.to_string())
}

fn find_extension_dir_by_package_name(package_name: &str) -> Option<PathBuf> {
    let extensions_root = match get_extensions_root() {
        Ok(path) => path,
        Err(_) => return None,
    };

    let entries = std::fs::read_dir(extensions_root).ok()?;
    for entry in entries {
        let entry = entry.ok()?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let package_json_path = path.join("package.json");
        if !package_json_path.is_file() {
            continue;
        }

        if read_extension_package_name(&package_json_path)
            .map(|value| value == package_name)
            .unwrap_or(false)
        {
            return Some(path);
        }
    }

    None
}

fn is_channel_extension_installed(channel_id: &str) -> bool {
    let (_, _, package_name) = match get_extension_meta(channel_id) {
        Ok(meta) => meta,
        Err(_) => return false,
    };

    let target_dir = match find_extension_dir_by_package_name(package_name) {
        Some(path) => path,
        None => return false,
    };

    target_dir.exists()
        && target_dir.join("package.json").exists()
        && target_dir.join("node_modules").exists()
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_all(&path, &target)?;
        } else {
            std::fs::copy(&path, &target).map_err(|e| format!("复制文件失败: {}", e))?;
        }
    }

    Ok(())
}

fn npm_executable() -> &'static str {
    if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    }
}

fn node_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "node.exe"
    } else {
        "node"
    }
}

#[cfg(target_os = "windows")]
fn apply_no_window(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn apply_no_window(_command: &mut Command) {}

fn openclaw_managed_root() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home.join(".openclaw"))
}

fn managed_runtime_root() -> Result<PathBuf, String> {
    Ok(openclaw_managed_root()?.join("runtime"))
}

fn managed_node_root() -> Result<PathBuf, String> {
    Ok(managed_runtime_root()?.join("node"))
}

fn managed_git_root() -> Result<PathBuf, String> {
    Ok(managed_runtime_root()?.join("git"))
}

fn managed_git_install_dir() -> Result<PathBuf, String> {
    Ok(managed_git_root()?.join("mingit"))
}

fn managed_npm_prefix() -> Result<PathBuf, String> {
    Ok(openclaw_managed_root()?.join("npm-global"))
}

fn managed_npm_bin_dir() -> Result<PathBuf, String> {
    let prefix = managed_npm_prefix()?;
    if cfg!(target_os = "windows") {
        Ok(prefix)
    } else {
        Ok(prefix.join("bin"))
    }
}

fn managed_node_platform_target() -> &'static str {
    if cfg!(target_os = "windows") {
        if cfg!(target_arch = "aarch64") {
            "win-arm64"
        } else {
            "win-x64"
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "darwin-arm64"
        } else {
            "darwin-x64"
        }
    } else if cfg!(target_arch = "aarch64") {
        "linux-arm64"
    } else {
        "linux-x64"
    }
}

fn resolve_requested_node_version(version: &str) -> String {
    let normalized = version.trim().trim_start_matches('v');
    if normalized.contains('.') {
        normalized.to_string()
    } else {
        MANAGED_NODE_VERSION.to_string()
    }
}

fn managed_node_install_dir(version: &str) -> Result<PathBuf, String> {
    let version = resolve_requested_node_version(version);
    Ok(managed_node_root()?.join(format!("node-v{}-{}", version, managed_node_platform_target())))
}

fn managed_node_archive_name(version: &str) -> String {
    let extension = if cfg!(target_os = "windows") { "zip" } else { "tar.gz" };
    format!(
        "node-v{}-{}.{}",
        resolve_requested_node_version(version),
        managed_node_platform_target(),
        extension
    )
}

fn managed_node_home_has_binary(path: &Path) -> bool {
    if cfg!(target_os = "windows") {
        path.join(node_executable_name()).is_file()
    } else {
        path.join("bin").join(node_executable_name()).is_file()
    }
}

fn resolve_managed_node_home() -> Option<PathBuf> {
    let root = managed_node_root().ok()?;
    let exact = managed_node_install_dir(MANAGED_NODE_VERSION).ok();
    if let Some(path) = exact {
        if managed_node_home_has_binary(&path) {
            return Some(path);
        }
    }

    let mut candidates: Vec<PathBuf> = std::fs::read_dir(root)
        .ok()?
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| path.is_dir() && managed_node_home_has_binary(path))
        .collect();
    candidates.sort();
    candidates.pop()
}

fn resolve_managed_node_bin_dir() -> Option<PathBuf> {
    let home = resolve_managed_node_home()?;
    if cfg!(target_os = "windows") {
        Some(home)
    } else {
        Some(home.join("bin"))
    }
}

fn ensure_process_path_entries(entries: &[PathBuf]) -> Result<(), String> {
    let normalized: Vec<PathBuf> = entries
        .iter()
        .filter(|path| path.exists())
        .cloned()
        .collect();
    if normalized.is_empty() {
        return Ok(());
    }

    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut merged = normalized.clone();
    merged.extend(
        std::env::split_paths(&current)
            .filter(|existing| !normalized.iter().any(|candidate| candidate == existing)),
    );
    let updated = std::env::join_paths(merged)
        .map_err(|error| format!("更新 PATH 失败: {}", error))?;
    std::env::set_var("PATH", updated);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn unix_shell_rc_files() -> Result<Vec<PathBuf>, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let mut files = vec![home.join(".profile"), home.join(".bashrc"), home.join(".zshrc")];
    files.dedup();
    Ok(files)
}

#[cfg(not(target_os = "windows"))]
fn build_managed_path_snippet(entries: &[PathBuf]) -> String {
    let joined = entries
        .iter()
        .map(|path| path.to_string_lossy().replace('\\', "\\\\").replace('"', "\\\""))
        .collect::<Vec<_>>()
        .join(":");
    format!(
        "{start}\nexport PATH=\"{joined}:$PATH\"\n{end}\n",
        start = OPENCLAW_MANAGED_PATH_MARKER_START,
        joined = joined,
        end = OPENCLAW_MANAGED_PATH_MARKER_END
    )
}

#[cfg(not(target_os = "windows"))]
fn persist_unix_user_path_entries(entries: &[PathBuf]) -> Result<(), String> {
    let normalized: Vec<PathBuf> = entries
        .iter()
        .filter(|path| path.exists())
        .cloned()
        .collect();
    if normalized.is_empty() {
        return Ok(());
    }

    let snippet = build_managed_path_snippet(&normalized);
    for rc_file in unix_shell_rc_files()? {
        let current = std::fs::read_to_string(&rc_file).unwrap_or_default();
        let updated = if let (Some(start), Some(end)) = (
            current.find(OPENCLAW_MANAGED_PATH_MARKER_START),
            current.find(OPENCLAW_MANAGED_PATH_MARKER_END),
        ) {
            let end_index = end + OPENCLAW_MANAGED_PATH_MARKER_END.len();
            format!("{}{}{}", &current[..start], snippet, &current[end_index..])
        } else if current.trim().is_empty() {
            snippet.clone()
        } else {
            format!("{}\n{}", current.trim_end(), snippet)
        };

        if updated != current {
            std::fs::write(&rc_file, updated)
                .map_err(|error| format!("写入 shell 配置失败 {}: {}", rc_file.display(), error))?;
        }
    }
    Ok(())
}

fn persist_path_entries_to_user(entries: &[PathBuf]) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        for entry in entries {
            if entry.exists() {
                persist_windows_user_path_entry(entry)?;
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        persist_unix_user_path_entries(entries)
    }
}

fn ensure_managed_npm_prefix_config() -> Result<PathBuf, String> {
    let prefix = managed_npm_prefix()?;
    std::fs::create_dir_all(&prefix)
        .map_err(|error| format!("创建 npm prefix 目录失败: {}", error))?;
    if !cfg!(target_os = "windows") {
        std::fs::create_dir_all(prefix.join("bin"))
            .map_err(|error| format!("创建 npm bin 目录失败: {}", error))?;
    }

    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let npmrc = home.join(".npmrc");
    let desired = format!("prefix={}", prefix.to_string_lossy().replace('\\', "/"));
    let current = std::fs::read_to_string(&npmrc).unwrap_or_default();
    let mut replaced = false;

    let mut lines = Vec::new();
    for line in current.lines() {
        if line.trim_start().starts_with("prefix=") {
            lines.push(desired.clone());
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !replaced {
        lines.push(desired);
    }
    let updated = format!("{}\n", lines.join("\n"));
    if updated != current {
        std::fs::write(&npmrc, updated)
            .map_err(|error| format!("写入 .npmrc 失败: {}", error))?;
    }
    Ok(prefix)
}

fn managed_runtime_path_entries() -> Result<Vec<PathBuf>, String> {
    let npm_bin = managed_npm_bin_dir()?;
    if !npm_bin.exists() {
        std::fs::create_dir_all(&npm_bin)
            .map_err(|error| format!("创建运行时目录失败: {}", error))?;
    }

    let mut entries = Vec::new();
    if let Some(node_bin_dir) = resolve_managed_node_bin_dir() {
        entries.push(node_bin_dir);
    }
    entries.push(npm_bin);
    entries.dedup();
    Ok(entries)
}

fn prepare_managed_runtime_process_env() -> Result<Vec<PathBuf>, String> {
    let prefix = ensure_managed_npm_prefix_config()?;
    let entries = managed_runtime_path_entries()?;
    ensure_process_path_entries(&entries)?;
    std::env::set_var("npm_config_prefix", &prefix);
    Ok(entries)
}

fn expose_managed_runtime_to_user_path_silently() -> Result<(), String> {
    let entries = prepare_managed_runtime_process_env()?;
    persist_path_entries_to_user(&entries)?;
    Ok(())
}

fn run_cmd(program: &str, args: &[&str]) -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        apply_no_window(&mut cmd);
        cmd.arg("/c").arg(program);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).output()
    } else {
        let mut cmd = Command::new(program);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
        apply_no_window(&mut cmd);
        cmd.output()
    };

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
            }
        }
        Err(e) => Err(format!("执行命令失败: {}", e)),
    }
}

fn run_shell(cmd: &str) -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        apply_no_window(&mut command);
        command.args(["/c", cmd]).stdout(Stdio::piped()).stderr(Stdio::piped()).output()
    } else {
        let mut command = Command::new("sh");
        command.args(["-c", cmd]).stdout(Stdio::piped()).stderr(Stdio::piped());
        apply_no_window(&mut command);
        command.output()
    };

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            if out.status.success() {
                Ok(stdout)
            } else {
                Err(if stderr.is_empty() { stdout } else { stderr })
            }
        }
        Err(e) => Err(format!("执行命令失败: {}", e)),
    }
}

fn format_command_failure(status_code: Option<i32>, stdout: &str, stderr: &str) -> String {
    let stderr = stderr.trim();
    let stdout = stdout.trim();
    let detail = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        ""
    };

    if detail.is_empty() {
        format!("命令执行失败，退出码: {:?}", status_code)
    } else {
        format!("命令执行失败，退出码: {:?}\n{}", status_code, detail)
    }
}

fn run_shell_with_log(app: &AppHandle, step: &str, cmd: &str) -> Result<String, String> {
    emit_log(app, step, &format!("$ {}", cmd), "info");

    let mut child = if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        apply_no_window(&mut command);
        command.args(["/c", cmd]).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
    } else {
        let mut command = Command::new("sh");
        command.args(["-c", cmd]).stdout(Stdio::piped()).stderr(Stdio::piped());
        apply_no_window(&mut command);
        command.spawn()
    }
    .map_err(|e| format!("启动命令失败: {}", e))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let stderr_app = app.clone();
    let stderr_step = step.to_string();
    let stderr_lines = Arc::new(Mutex::new(Vec::new()));
    let stderr_lines_handle = Arc::clone(&stderr_lines);
    let stderr_handle = stderr.map(|err| {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                emit_log(&stderr_app, &stderr_step, &line, "warn");
                if let Ok(mut collected) = stderr_lines_handle.lock() {
                    collected.push(line);
                }
            }
        })
    });

    let mut output_lines = Vec::new();
    if let Some(out) = stdout {
        let reader = BufReader::new(out);
        for line in reader.lines().flatten() {
            emit_log(app, step, &line, "info");
            output_lines.push(line);
        }
    }

    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }

    let stderr_output = stderr_lines
        .lock()
        .map(|collected| collected.join("\n"))
        .unwrap_or_default();

    let status = child.wait().map_err(|e| format!("等待命令完成失败: {}", e))?;
    if !status.success() {
        return Err(format_command_failure(status.code(), &output_lines.join("\n"), &stderr_output));
    }

    let status = child.wait().map_err(|e| format!("等待命令完成失败: {}", e))?;
    if status.success() {
        Ok(output_lines.join("\n"))
    } else {
        Err(format!("命令执行失败，退出码: {:?}", status.code()))
    }
}

fn has_system_node() -> bool {
    if run_cmd("node", &["--version"]).is_ok() {
        return true;
    }

    let candidate_paths: &[&str] = if cfg!(target_os = "windows") {
        &[
            r"C:\Program Files\nodejs\node.exe",
            r"C:\Program Files (x86)\nodejs\node.exe",
        ]
    } else {
        &[
            "/opt/homebrew/bin/node",
            "/usr/local/bin/node",
            "/usr/bin/node",
        ]
    };

    candidate_paths.iter().any(|path| {
        let candidate = Path::new(path);
        if !candidate.exists() {
            return false;
        }
        let mut command = Command::new(candidate);
        apply_no_window(&mut command);
        command.arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().map(|status| status.success()).unwrap_or(false)
    })
}

fn can_execute_fnm_binary_directly() -> bool {
    if run_cmd("fnm", &["--version"]).is_ok() {
        return true;
    }

    let Some(home) = dirs::home_dir() else {
        return false;
    };

    let fnm_path = if cfg!(target_os = "windows") {
        home.join(".fnm").join("fnm.exe")
    } else {
        home.join(".fnm").join("fnm")
    };

    if !fnm_path.exists() {
        return false;
    }

    let mut command = Command::new(fnm_path);
    apply_no_window(&mut command);
    command.arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().map(|status| status.success()).unwrap_or(false)
}

fn shell_quote(value: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

fn decode_command_output(bytes: &[u8]) -> String {
    let decoded = if bytes.len() >= 2 && bytes.len() % 2 == 0 {
        let odd_nulls = bytes.iter().skip(1).step_by(2).filter(|byte| **byte == 0).count();
        if odd_nulls * 2 >= bytes.len() / 2 {
            let utf16 = bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect::<Vec<_>>();
            String::from_utf16_lossy(&utf16)
        } else {
            String::from_utf8_lossy(bytes).to_string()
        }
    } else {
        String::from_utf8_lossy(bytes).to_string()
    };
    decoded.trim().to_string()
}

#[cfg(target_os = "windows")]
fn powershell_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

fn with_fnm_env(cmd: &str) -> String {
    let _ = prepare_managed_runtime_process_env();

    if has_system_node() || resolve_managed_node_bin_dir().is_some() {
        return cmd.to_string();
    }

    if !can_execute_fnm_binary_directly() {
        return cmd.to_string();
    }

    if cfg!(target_os = "windows") {
        let fnm_dir = "$env:USERPROFILE\\.fnm";
        format!(
            "powershell -NoProfile -Command \"$env:FNM_DIR='{}'; $env:PATH='{}' + ';' + $env:PATH; & fnm env --use-on-cd | Invoke-Expression; {}\"",
            fnm_dir, fnm_dir, cmd
        )
    } else {
        format!(
            "export FNM_DIR=\"$HOME/.fnm\" && export PATH=\"$HOME/.fnm:$PATH\" && eval \"$(fnm env)\" && {}",
            cmd
        )
    }
}

fn parse_openclaw_config_get_value(raw: &str) -> String {
    let line = raw
        .lines()
        .rev()
        .find(|item| !item.trim().is_empty())
        .map(|item| item.trim())
        .unwrap_or("");

    let value = if let Some((_, right)) = line.split_once('=') {
        right.trim()
    } else {
        line
    };

    value
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn is_redacted_or_empty_secret(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return true;
    }
    let lowered = trimmed.to_ascii_lowercase();
    lowered.contains("redacted")
}

fn get_local_openclaw_config_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let config_dir = home.join(".openclaw");
    let openclaw_path = config_dir.join("openclaw.json");
    if openclaw_path.exists() {
        return Ok(openclaw_path);
    }

    let clawdbot_path = config_dir.join("clawdbot.json");
    if clawdbot_path.exists() {
        return Ok(clawdbot_path);
    }

    Err(format!(
        "未找到配置文件: {}",
        config_dir.display()
    ))
}

fn read_gateway_auth_token_from_local_config() -> Result<String, String> {
    let config_path = get_local_openclaw_config_path()?;
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败({}): {}", config_path.display(), e))?;
    let config: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    let token = config
        .get("gateway")
        .and_then(|gateway| gateway.get("auth"))
        .and_then(|auth| auth.get("token"))
        .and_then(|token| token.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    Ok(token)
}

/// 使用 openclaw config set 写入飞书渠道配置
#[tauri::command]
pub async fn set_feishu_channel_config(
    app_id: String,
    app_secret: String,
    enabled: bool,
) -> Result<String, String> {
    if app_id.trim().is_empty() {
        return Err("飞书 App ID 不能为空".to_string());
    }
    if app_secret.trim().is_empty() {
        return Err("飞书 App Secret 不能为空".to_string());
    }

    let commands = vec![
        format!(
            "openclaw config set channels.feishu.appId {}",
            shell_quote(app_id.trim())
        ),
        format!(
            "openclaw config set channels.feishu.appSecret {}",
            shell_quote(app_secret.trim())
        ),
        format!(
            "openclaw config set channels.feishu.enabled {}",
            if enabled { "true" } else { "false" }
        ),
    ];

    for command in commands {
        run_shell(&with_fnm_env(&command))
            .map_err(|error| format!("执行 `{}` 失败: {}", command, error))?;
    }

    Ok("飞书渠道配置写入成功".to_string())
}

/// 使用 openclaw config set 写入钉钉渠道配置
#[tauri::command]
pub async fn set_dingtalk_channel_config(
    client_id: String,
    client_secret: String,
    enabled: bool,
) -> Result<String, String> {
    if client_id.trim().is_empty() {
        return Err("钉钉 Client ID 不能为空".to_string());
    }
    if client_secret.trim().is_empty() {
        return Err("钉钉 Client Secret 不能为空".to_string());
    }

    let commands = vec![
        format!(
            "openclaw config set channels.dingtalk-connector.clientId {}",
            shell_quote(client_id.trim())
        ),
        format!(
            "openclaw config set channels.dingtalk-connector.clientSecret {}",
            shell_quote(client_secret.trim())
        ),
        format!(
            "openclaw config set channels.dingtalk-connector.enabled {}",
            if enabled { "true" } else { "false" }
        ),
    ];

    for command in commands {
        run_shell(&with_fnm_env(&command))
            .map_err(|error| format!("执行 `{}` 失败: {}", command, error))?;
    }

    if enabled {
        // 启用钉钉时补齐 gateway.http.chatCompletions（合并到现有 gateway）
        let enable_chat_completions_cmd =
            "openclaw config set gateway.http.endpoints.chatCompletions.enabled true";
        run_shell(&with_fnm_env(enable_chat_completions_cmd))
            .map_err(|error| format!("写入 gateway.http.endpoints.chatCompletions.enabled 失败: {}", error))?;

        // 先显式启用 token 鉴权，触发 token 生成流程
        let ensure_gateway_auth_mode_cmd = "openclaw config set gateway.auth.mode token";
        run_shell(&with_fnm_env(ensure_gateway_auth_mode_cmd))
            .map_err(|error| format!("设置 gateway.auth.mode 失败: {}", error))?;

        // 优先从本地配置文件读取真实 token（避免 config get 脱敏返回）
        let mut gateway_token = read_gateway_auth_token_from_local_config().unwrap_or_default();

        // 兜底：尝试用 CLI 读取
        if is_redacted_or_empty_secret(&gateway_token) {
            let get_gateway_token_cmd = "openclaw config get gateway.auth.token";
            if let Ok(gateway_token_raw) = run_shell(&with_fnm_env(get_gateway_token_cmd)) {
                gateway_token = parse_openclaw_config_get_value(&gateway_token_raw);
            }
        }

        // 仍拿不到就主动生成并写入
        if is_redacted_or_empty_secret(&gateway_token) {
            let generated_gateway_token = format!("openclaw-{:x}", now_ms());
            let set_gateway_auth_token_cmd = format!(
                "openclaw config set gateway.auth.token {}",
                shell_quote(&generated_gateway_token)
            );
            run_shell(&with_fnm_env(&set_gateway_auth_token_cmd))
                .map_err(|error| format!("生成 gateway.auth.token 失败: {}", error))?;
            gateway_token = generated_gateway_token;
        }

        if is_redacted_or_empty_secret(&gateway_token) {
            return Err("读取 gateway.auth.token 失败: token 未生成或被脱敏".to_string());
        }

        let set_gateway_token_cmd = format!(
            "openclaw config set channels.dingtalk-connector.gatewayToken {}",
            shell_quote(&gateway_token)
        );
        run_shell(&with_fnm_env(&set_gateway_token_cmd))
            .map_err(|error| format!("写入 channels.dingtalk-connector.gatewayToken 失败: {}", error))?;

        let enable_media_upload_cmd =
            "openclaw config set channels.dingtalk-connector.enableMediaUpload true";
        run_shell(&with_fnm_env(enable_media_upload_cmd))
            .map_err(|error| format!("写入 channels.dingtalk-connector.enableMediaUpload 失败: {}", error))?;
    }

    Ok("钉钉渠道配置写入成功".to_string())
}

/// 使用 openclaw pairing approve feishu <code> 执行飞书配对
#[tauri::command]
pub async fn approve_feishu_pairing(pairing_code: String) -> Result<String, String> {
    let code = pairing_code.trim();
    if code.is_empty() {
        return Err("配对码不能为空".to_string());
    }

    let command = format!(
        "openclaw pairing approve feishu {}",
        shell_quote(code)
    );

    let output = run_shell(&with_fnm_env(&command))
        .map_err(|error| format!("飞书配对失败: {}", error))?;

    if output.is_empty() {
        Ok("飞书配对成功".to_string())
    } else {
        Ok(output)
    }
}

/// 兼容保留：钉钉渠道不再需要手动配对（防止旧注册残留导致编译失败）
#[allow(dead_code)]
#[tauri::command]
pub async fn approve_dingtalk_pairing(_pairing_code: String) -> Result<String, String> {
    Err("钉钉渠道无需填写配对码，请直接配置 Client ID / Client Secret 并启用。".to_string())
}

/// 带进度监控的下载函数（支持多镜像源自动切换）
async fn download_with_progress(
    app: &AppHandle,
    step: &str,
    urls: &[&str],
    description: &str,
) -> Result<bytes::Bytes, String> {
    emit_log(app, step, &format!("下载 {}", description), "info");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let mut last_error = String::new();

    for (idx, url) in urls.iter().enumerate() {
        emit_log(
            app,
            step,
            &format!("尝试镜像源 {}/{}: {}", idx + 1, urls.len(), url),
            "info",
        );

        match download_from_url(&client, app, step, url).await {
            Ok(data) => {
                emit_log(app, step, &format!("下载完成: {}", description), "success");
                return Ok(data);
            }
            Err(e) => {
                last_error = e.clone();
                emit_log(
                    app,
                    step,
                    &format!("镜像源 {} 下载失败: {}", idx + 1, e),
                    "warn",
                );
            }
        }
    }

    Err(format!("所有镜像源均下载失败，最后错误: {}", last_error))
}

/// 从单个 URL 下载（带进度）
async fn download_from_url(
    client: &reqwest::Client,
    app: &AppHandle,
    step: &str,
    url: &str,
) -> Result<bytes::Bytes, String> {
    use std::time::Instant;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut buffer = Vec::new();
    let start_time = Instant::now();
    let mut last_emit_time = Instant::now();

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("读取数据失败: {}", e))?;
        downloaded += chunk.len() as u64;
        buffer.extend_from_slice(&chunk);

        // 每 200ms 更新一次进度
        if last_emit_time.elapsed().as_millis() >= 200 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                downloaded as f64 / elapsed
            } else {
                0.0
            };

            let speed_str = format_speed(speed);
            let percent = if total_size > 0 {
                ((downloaded as f64 / total_size as f64) * 100.0) as u8
            } else {
                0
            };

            emit_download(app, step, percent, &speed_str, downloaded, total_size);
            last_emit_time = Instant::now();
        }
    }

    // 最后发送 100% 进度
    if total_size > 0 {
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            downloaded as f64 / elapsed
        } else {
            0.0
        };
        emit_download(app, step, 100, &format_speed(speed), downloaded, total_size);
    }

    Ok(bytes::Bytes::from(buffer))
}

/// 格式化速度（字节/秒 → 人类可读）
fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_048_576.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_048_576.0)
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

// ============================================================================
// Tauri 命令 - 环境检测
// ============================================================================

/// 检测 OpenClaw 安装状态
#[tauri::command]
pub fn check_openclaw_installed() -> OpenClawStatus {
    // 尝试直接执行
    if let Ok(version) = run_shell(&with_fnm_env("openclaw --version")) {
        let path = run_shell(&with_fnm_env(if cfg!(target_os = "windows") {
            "where openclaw"
        } else {
            "which openclaw"
        }))
        .ok();
        return OpenClawStatus {
            installed: true,
            version: Some(version),
            path,
        };
    }

    // 不带 fnm 环境再试一次
    if let Ok(version) = run_cmd("openclaw", &["--version"]) {
        let path = if cfg!(target_os = "windows") {
            run_shell("where openclaw").ok()
        } else {
            run_shell("which openclaw").ok()
        };
        return OpenClawStatus {
            installed: true,
            version: Some(version),
            path,
        };
    }

    OpenClawStatus {
        installed: false,
        version: None,
        path: None,
    }
}

/// 检测 Node.js 安装状态
#[tauri::command]
pub fn check_node_installed() -> NodeStatus {
    // 先尝试 fnm 环境
    let version_str = run_shell(&with_fnm_env("node --version"))
        .or_else(|_| run_cmd("node", &["--version"]));

    match version_str {
        Ok(v) => {
            let version = v.trim_start_matches('v').to_string();
            let major: u32 = version
                .split('.')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            NodeStatus {
                installed: true,
                version: Some(version),
                meets_requirement: major >= 22,
            }
        }
        Err(_) => NodeStatus {
            installed: false,
            version: None,
            meets_requirement: false,
        },
    }
}

/// 检测 Git 安装状态
#[tauri::command]
pub fn check_git_installed() -> GitStatus {
    match run_cmd("git", &["--version"]) {
        Ok(v) => {
            let version = v.replace("git version ", "").trim().to_string();
            GitStatus {
                installed: true,
                version: Some(version),
            }
        }
        Err(_) => GitStatus {
            installed: false,
            version: None,
        },
    }
}

/// 检测 fnm 安装状态
#[tauri::command]
pub fn check_fnm_installed() -> FnmStatus {
    match run_cmd("fnm", &["--version"]) {
        Ok(v) => FnmStatus {
            installed: true,
            version: Some(v.replace("fnm ", "").trim().to_string()),
        },
        Err(_) => {
            // 检查 ~/.fnm/fnm 是否存在
            if let Some(home) = dirs::home_dir() {
                let fnm_path = if cfg!(target_os = "windows") {
                    home.join(".fnm").join("fnm.exe")
                } else {
                    home.join(".fnm").join("fnm")
                };
                if fnm_path.exists() {
                    if let Ok(v) = run_shell(&format!("{} --version", fnm_path.display())) {
                        return FnmStatus {
                            installed: true,
                            version: Some(v.replace("fnm ", "").trim().to_string()),
                        };
                    }
                }
            }
            FnmStatus {
                installed: false,
                version: None,
            }
        }
    }
}

/// 获取系统信息
#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
    .to_string();

    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    }
    .to_string();

    let shell = if cfg!(target_os = "windows") {
        "cmd".to_string()
    } else {
        std::env::var("SHELL")
            .unwrap_or_else(|_| "/bin/sh".to_string())
            .rsplit('/')
            .next()
            .unwrap_or("sh")
            .to_string()
    };

    SystemInfo { os, arch, shell }
}

/// 检测网络环境（中国/全球）
#[tauri::command]
pub async fn detect_network_region() -> String {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    // 尝试访问淘宝镜像，如果快速响应则判定为中国网络
    match client.head("https://registry.npmmirror.com").send().await {
        Ok(resp) if resp.status().is_success() => "china".to_string(),
        _ => "global".to_string(),
    }
}

/// 综合环境检测
#[tauri::command]
pub async fn check_environment() -> EnvironmentStatus {
    repair_managed_node_path_silently();
    let openclaw = check_openclaw_installed();
    let node = check_node_installed();
    let git = check_git_installed();
    let fnm = check_fnm_installed();
    let system = get_system_info();
    let network_region = detect_network_region().await;

    EnvironmentStatus {
        openclaw,
        node,
        git,
        fnm,
        system,
        network_region,
    }
}

// ============================================================================
// Tauri 命令 - 安装操作
// ============================================================================

/// 获取 fnm 下载 URL 列表（多镜像源）
fn get_fnm_download_urls() -> Vec<String> {
    let platform = if cfg!(target_os = "windows") {
        "fnm-windows"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "fnm-arm64"
        } else {
            "fnm-macos"
        }
    } else {
        // Linux
        if cfg!(target_arch = "aarch64") {
            "fnm-arm64"
        } else {
            "fnm-linux"
        }
    };

    FNM_MIRRORS
        .iter()
        .map(|mirror| format!("{}/{}.zip", mirror, platform))
        .collect()
}

/// 安装 fnm
#[tauri::command]
pub async fn install_fnm(app: AppHandle, _use_mirror: bool) -> Result<String, String> {
    let step = "install_fnm";
    let start_time = now_ms();
    emit_log(&app, step, "开始安装 fnm (Fast Node Manager)...", "info");

    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let fnm_dir = home.join(".fnm");

    // 创建 fnm 目录
    std::fs::create_dir_all(&fnm_dir)
        .map_err(|e| format!("创建 fnm 目录失败: {}", e))?;

    // 获取下载 URL 列表
    let urls = get_fnm_download_urls();
    let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();

    // 使用新的下载函数（带进度）
    let bytes = download_with_progress(&app, step, &url_refs, "fnm").await?;

    // 解压
    extract_fnm_zip(&app, step, &bytes, &fnm_dir)?;

    // 配置 PATH（添加到 shell 配置文件）
    configure_fnm_path(&app, step)?;

    let end_time = now_ms();
    emit_timing(&app, step, start_time, end_time);
    emit_log(&app, step, "fnm 安装完成!", "success");
    Ok("fnm 安装成功".to_string())
}

/// 解压 fnm zip 文件
fn extract_fnm_zip(
    app: &AppHandle,
    step: &str,
    data: &[u8],
    target_dir: &std::path::Path,
) -> Result<(), String> {
    emit_log(app, step, "解压 fnm...", "info");

    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("解压失败: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("读取压缩文件失败: {}", e))?;
        let name = file.name().to_string();

        if name.ends_with('/') {
            continue;
        }

        let outpath = target_dir.join(&name);
        emit_log(app, step, &format!("解压: {}", name), "info");

        let mut outfile = std::fs::File::create(&outpath)
            .map_err(|e| format!("创建文件失败: {}", e))?;
        std::io::copy(&mut file, &mut outfile)
            .map_err(|e| format!("写入文件失败: {}", e))?;

        // Unix: 设置可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if name == "fnm" {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| format!("设置权限失败: {}", e))?;
            }
        }
    }

    Ok(())
}

/// 配置 fnm PATH
fn extract_zip_archive_to_dir<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    target_dir: &Path,
) -> Result<(), String> {
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| format!("读取压缩文件失败: {}", error))?;
        let outpath = target_dir.join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)
                .map_err(|error| format!("创建目录失败: {}", error))?;
            continue;
        }

        if let Some(parent) = outpath.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("创建目录失败: {}", error))?;
        }

        let mut outfile = std::fs::File::create(&outpath)
            .map_err(|error| format!("创建文件失败: {}", error))?;
        std::io::copy(&mut file, &mut outfile)
            .map_err(|error| format!("写入文件失败: {}", error))?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn ensure_windows_process_path(dir: &Path) -> Result<(), String> {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut entries = vec![dir.to_path_buf()];
    entries.extend(std::env::split_paths(&current).filter(|existing| existing != dir));
    let updated = std::env::join_paths(entries).map_err(|e| format!("更新进程 PATH 失败: {}", e))?;
    std::env::set_var("PATH", updated);
    Ok(())
}

#[cfg(target_os = "windows")]
fn sanitize_windows_path_entry(value: &str) -> String {
    let mut sanitized = value.trim().trim_matches('"').replace('/', "\\");
    while sanitized.len() > 3 && sanitized.ends_with('\\') {
        sanitized.pop();
    }
    sanitized
}

#[cfg(target_os = "windows")]
fn normalize_windows_path_entry(value: &str) -> String {
    sanitize_windows_path_entry(value).to_ascii_lowercase()
}

#[cfg(target_os = "windows")]
fn append_windows_user_path_entry(current: &str, dir: &Path) -> Option<String> {
    let target = sanitize_windows_path_entry(&dir.to_string_lossy());
    if target.is_empty() {
        return None;
    }

    let target_normalized = normalize_windows_path_entry(&target);
    let mut segments = Vec::new();
    let mut exists = false;

    for segment in current.split(';') {
        let sanitized = sanitize_windows_path_entry(segment);
        if sanitized.is_empty() {
            continue;
        }
        if normalize_windows_path_entry(&sanitized) == target_normalized {
            exists = true;
        }
        segments.push(sanitized);
    }

    if exists {
        None
    } else {
        segments.push(target);
        Some(segments.join(";"))
    }
}

fn windows_git_path_entries(install_dir: &Path) -> Vec<PathBuf> {
    let candidates = [
        install_dir.to_path_buf(),
        install_dir.join("cmd"),
        install_dir.join("mingw64").join("bin"),
        install_dir.join("usr").join("bin"),
    ];

    let mut entries = Vec::new();
    for candidate in candidates {
        if candidate.exists() && !entries.iter().any(|existing| existing == &candidate) {
            entries.push(candidate);
        }
    }

    entries
}

fn windows_git_install_dir_has_binary(path: &Path) -> bool {
    windows_git_path_entries(path)
        .into_iter()
        .any(|entry| entry.join("git.exe").is_file())
}

fn find_directory_containing_windows_git_binary(root: &Path) -> Option<PathBuf> {
    if windows_git_install_dir_has_binary(root) {
        return Some(root.to_path_buf());
    }

    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_directory_containing_windows_git_binary(&path) {
                return Some(found);
            }
        }
    }

    None
}

fn git_install_is_blocking_in_full_install() -> bool {
    cfg!(target_os = "windows")
}

#[cfg(target_os = "windows")]
fn run_powershell_no_profile(script: &str) -> Result<String, String> {
    let mut command = Command::new("powershell");
    apply_no_window(&mut command);
    command
        .args(["-NoProfile", "-Command", script])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = command
        .output()
        .map_err(|error| format!("执行 PowerShell 失败: {}", error))?;
    let stdout = decode_command_output(&output.stdout);
    let stderr = decode_command_output(&output.stderr);

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format_command_failure(output.status.code(), &stdout, &stderr))
    }
}

#[cfg(target_os = "windows")]
fn persist_windows_user_path_entry(dir: &Path) -> Result<(), String> {
    let current = run_powershell_no_profile("[Environment]::GetEnvironmentVariable('Path','User')")?;
    let Some(updated) = append_windows_user_path_entry(&current, dir) else {
        return Ok(());
    };
    let escaped = updated.replace('\'', "''");
    run_powershell_no_profile(&format!(
        "[Environment]::SetEnvironmentVariable('Path', '{}', 'User')",
        escaped
    ))
    .map(|_| ())
}

#[cfg(target_os = "windows")]
fn resolve_active_node_bin_dir() -> Option<PathBuf> {
    if let Some(path) = resolve_managed_node_bin_dir() {
        return Some(path);
    }

    let output = run_shell(&with_fnm_env("where node")).ok()?;
    output.lines().find_map(|line| {
        let candidate = PathBuf::from(line.trim().trim_matches('"'));
        candidate.parent().map(|path| path.to_path_buf())
    })
}

#[cfg(target_os = "windows")]
fn expose_active_node_to_user_path_silently() -> Result<Option<PathBuf>, String> {
    let Some(node_bin_dir) = resolve_active_node_bin_dir() else {
        return Ok(None);
    };
    ensure_windows_process_path(&node_bin_dir)?;
    persist_windows_user_path_entry(&node_bin_dir)?;
    if let Ok(npm_bin_dir) = managed_npm_bin_dir() {
        let _ = ensure_windows_process_path(&npm_bin_dir);
        let _ = persist_windows_user_path_entry(&npm_bin_dir);
    }
    Ok(Some(node_bin_dir))
}

#[cfg(target_os = "windows")]
fn bundled_git_resource_candidates() -> &'static [&'static str] {
    &[
        "windows/git/mingit.zip",
        "resources/windows/git/mingit.zip",
    ]
}

#[cfg(target_os = "windows")]
fn resolve_bundled_git_archive(app: &AppHandle) -> Option<PathBuf> {
    for relative in bundled_git_resource_candidates() {
        if let Some(path) = app.path_resolver().resolve_resource(relative) {
            if path.exists() {
                return Some(path);
            }
        }
    }

    let local_candidates = [
        PathBuf::from("src-tauri")
            .join("resources")
            .join("windows")
            .join("git")
            .join("mingit.zip"),
        PathBuf::from("resources")
            .join("windows")
            .join("git")
            .join("mingit.zip"),
    ];

    local_candidates.into_iter().find(|path| path.exists())
}

#[cfg(target_os = "windows")]
fn extract_managed_git_archive(data: &[u8]) -> Result<PathBuf, String> {
    let root = managed_git_root()?;
    std::fs::create_dir_all(&root)
        .map_err(|error| format!("创建 Git 运行时目录失败: {}", error))?;

    let temp_dir = root.join(format!(".tmp-{}", now_ms()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|error| format!("创建 Git 临时目录失败: {}", error))?;

    let cursor = std::io::Cursor::new(data);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|error| format!("解压 Git 压缩包失败: {}", error))?;
    extract_zip_archive_to_dir(&mut archive, &temp_dir)?;

    let extracted_home = find_directory_containing_windows_git_binary(&temp_dir)
        .ok_or_else(|| "未找到解压后的 Git 目录".to_string())?;
    let final_dir = managed_git_install_dir()?;
    if final_dir.exists() {
        std::fs::remove_dir_all(&final_dir)
            .map_err(|error| format!("清理旧 Git 目录失败: {}", error))?;
    }
    if let Some(parent) = final_dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("创建 Git 目录失败: {}", error))?;
    }

    match std::fs::rename(&extracted_home, &final_dir) {
        Ok(_) => {}
        Err(_) => copy_dir_all(&extracted_home, &final_dir)?,
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(final_dir)
}

#[cfg(target_os = "windows")]
fn expose_windows_git_to_user_path_silently(install_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = windows_git_path_entries(install_dir);
    if entries.is_empty() {
        return Err("Git 安装目录缺少可用 PATH 条目".to_string());
    }

    ensure_process_path_entries(&entries)?;
    persist_path_entries_to_user(&entries)?;
    Ok(entries)
}

#[cfg(target_os = "windows")]
fn install_bundled_windows_git(app: &AppHandle, step: &str) -> Result<PathBuf, String> {
    let existing_install_dir = managed_git_install_dir()?;
    if windows_git_install_dir_has_binary(&existing_install_dir) {
        let entries = expose_windows_git_to_user_path_silently(&existing_install_dir)?;
        emit_log(
            app,
            step,
            &format!("复用已解压的内置 Git: {}", existing_install_dir.display()),
            "info",
        );
        emit_log(
            app,
            step,
            &format!(
                "Git PATH 已更新: {}",
                entries
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(";")
            ),
            "info",
        );
        if check_git_installed().installed {
            return Ok(existing_install_dir);
        }
        return Err("已找到内置 Git，但当前进程仍无法调用 git 命令".to_string());
    }

    let archive_path = resolve_bundled_git_archive(app)
        .ok_or_else(|| "未找到安装包内置的 Git 离线安装包".to_string())?;
    emit_log(
        app,
        step,
        &format!("Windows: 使用内置 Git 离线包安装: {}", archive_path.display()),
        "info",
    );

    let archive_data = std::fs::read(&archive_path)
        .map_err(|error| format!("读取 Git 离线安装包失败: {}", error))?;
    let install_dir = extract_managed_git_archive(&archive_data)?;
    let entries = expose_windows_git_to_user_path_silently(&install_dir)?;
    emit_log(
        app,
        step,
        &format!(
            "Git PATH 已更新: {}",
            entries
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(";")
        ),
        "info",
    );

    if !check_git_installed().installed {
        return Err("Git 离线包已解压，但 git 命令仍不可用".to_string());
    }

    Ok(install_dir)
}

fn has_managed_runtime_artifacts() -> bool {
    let has_managed_node = resolve_managed_node_bin_dir().is_some();
    let has_managed_npm = managed_npm_bin_dir().map(|path| path.exists()).unwrap_or(false);
    has_managed_node || has_managed_npm
}

fn repair_managed_node_path_silently() {
    if !has_managed_runtime_artifacts() {
        return;
    }

    let _ = expose_managed_runtime_to_user_path_silently();
    #[cfg(target_os = "windows")]
    {
        let _ = expose_active_node_to_user_path_silently();
    }
}fn configure_fnm_path(app: &AppHandle, step: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        emit_log(app, step, "Configuring Windows PATH...", "info");
        let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let fnm_path = home.join(".fnm");
        persist_windows_user_path_entry(&fnm_path)?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

        let rc_file = if shell.contains("zsh") {
            home.join(".zshrc")
        } else {
            home.join(".bashrc")
        };

        let fnm_init = r#"
# fnm (Fast Node Manager)
export FNM_DIR="$HOME/.fnm"
export PATH="$HOME/.fnm:$PATH"
eval "$(fnm env --use-on-cd)"
"#;

        if let Ok(content) = std::fs::read_to_string(&rc_file) {
            if content.contains("FNM_DIR") {
                emit_log(app, step, "fnm PATH 已配置，跳过", "info");
                return Ok(());
            }
        }

        emit_log(app, step, &format!("追加 fnm 初始化到 {}", rc_file.display()), "info");

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&rc_file)
            .map_err(|e| format!("打开 shell 配置失败: {}", e))?;

        use std::io::Write;
        file.write_all(fnm_init.as_bytes())
            .map_err(|e| format!("写入 fnm 配置失败: {}", e))?;
    }

    Ok(())
}

/// 通过 fnm 安装 Node.js
fn node_download_urls(version: &str, use_china_mirror: bool) -> Vec<String> {
    let version = resolve_requested_node_version(version);
    let release = format!("v{}", version);
    let archive = managed_node_archive_name(&version);
    let mut bases = Vec::new();
    if use_china_mirror {
        bases.extend(NODE_MIRRORS.iter().copied());
    }
    bases.push("https://nodejs.org/download/release");
    bases.into_iter().map(|base| format!("{}/{}/{}", base.trim_end_matches('/'), release, archive)).collect()
}

fn find_directory_containing_node_binary(root: &Path) -> Option<PathBuf> {
    if managed_node_home_has_binary(root) {
        return Some(root.to_path_buf());
    }
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_directory_containing_node_binary(&path) {
                return Some(found);
            }
        }
    }
    None
}

fn extract_managed_node_archive(data: &[u8], version: &str) -> Result<PathBuf, String> {
    let version = resolve_requested_node_version(version);
    let root = managed_node_root()?;
    std::fs::create_dir_all(&root).map_err(|error| format!("创建 Node 运行时目录失败: {}", error))?;

    let temp_dir = root.join(format!(".tmp-{}-{}", version, now_ms()));
    std::fs::create_dir_all(&temp_dir).map_err(|error| format!("创建临时目录失败: {}", error))?;

    if cfg!(target_os = "windows") {
        let cursor = std::io::Cursor::new(data);
        let mut archive = zip::ZipArchive::new(cursor).map_err(|error| format!("解压 Node zip 失败: {}", error))?;
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).map_err(|error| format!("读取 Node 压缩包失败: {}", error))?;
            let outpath = temp_dir.join(file.name());
            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath).map_err(|error| format!("创建目录失败: {}", error))?;
                continue;
            }
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent).map_err(|error| format!("创建目录失败: {}", error))?;
            }
            let mut outfile = std::fs::File::create(&outpath).map_err(|error| format!("创建文件失败: {}", error))?;
            std::io::copy(&mut file, &mut outfile).map_err(|error| format!("写入文件失败: {}", error))?;
        }
    } else {
        let gzip = GzDecoder::new(std::io::Cursor::new(data));
        let mut archive = tar::Archive::new(gzip);
        archive.unpack(&temp_dir).map_err(|error| format!("解压 Node 压缩包失败: {}", error))?;
    }

    let extracted_home = find_directory_containing_node_binary(&temp_dir).ok_or_else(|| "未找到解压后的 Node 目录".to_string())?;
    let final_dir = managed_node_install_dir(&version)?;
    if final_dir.exists() {
        std::fs::remove_dir_all(&final_dir).map_err(|error| format!("清理旧 Node 目录失败: {}", error))?;
    }
    if let Some(parent) = final_dir.parent() {
        std::fs::create_dir_all(parent).map_err(|error| format!("创建 Node 目录失败: {}", error))?;
    }

    match std::fs::rename(&extracted_home, &final_dir) {
        Ok(_) => {}
        Err(_) => copy_dir_all(&extracted_home, &final_dir)?,
    }
    let _ = std::fs::remove_dir_all(&temp_dir);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for binary in [final_dir.join("bin").join("node"), final_dir.join("bin").join("npm"), final_dir.join("bin").join("npx")] {
            if binary.exists() {
                let _ = std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755));
            }
        }
    }

    Ok(final_dir)
}

fn verify_openclaw_available_now() -> Result<String, String> {
    run_shell(&with_fnm_env("openclaw --version")).or_else(|_| run_cmd("openclaw", &["--version"]))
}

fn install_openclaw_with_source(app: &AppHandle, step: &str, source: &str, registry: Option<&str>) -> Result<String, String> {
    prepare_managed_runtime_process_env()?;
    let npm = npm_executable();
    let mut cmd = format!("{} install -g {} --no-fund --no-audit", npm, source);
    if let Some(registry) = registry {
        cmd.push_str(&format!(" --registry={}", registry));
    }
    run_shell_with_log(app, step, &cmd)?;
    expose_managed_runtime_to_user_path_silently()?;
    repair_managed_node_path_silently();
    verify_openclaw_available_now()
}

#[tauri::command]
pub async fn install_node_via_fnm(app: AppHandle, version: String, use_china_mirror: bool) -> Result<String, String> {
    let step = "install_node";
    let version = resolve_requested_node_version(&version);
    emit_log(&app, step, &format!("开始安装 Node.js {} ...", version), "info");

    let target_dir = managed_node_install_dir(&version)?;
    if managed_node_home_has_binary(&target_dir) {
        expose_managed_runtime_to_user_path_silently()?;
        emit_log(&app, step, "检测到已安装的托管 Node.js，跳过下载", "success");
        return Ok(format!("Node.js {} 已可用", version));
    }

    let urls = node_download_urls(&version, use_china_mirror);
    let url_refs: Vec<&str> = urls.iter().map(|item| item.as_str()).collect();
    let bytes = download_with_progress(&app, step, &url_refs, "node").await?;
    let install_dir = extract_managed_node_archive(&bytes, &version)?;
    expose_managed_runtime_to_user_path_silently()?;
    emit_log(&app, step, &format!("Node.js {} 安装完成: {}", version, install_dir.display()), "success");
    Ok(format!("Node.js {} 安装成功", version))
}

#[tauri::command]
pub async fn install_openclaw(app: AppHandle, use_china_mirror: bool) -> Result<String, String> {
    let step = "install_openclaw";
    emit_log(&app, step, "开始安装 OpenClaw...", "info");
    expose_managed_runtime_to_user_path_silently()?;

    let registries = if use_china_mirror { NPM_REGISTRIES.to_vec() } else { vec![NPM_REGISTRIES[2]] };

    for registry in &registries {
        emit_log(&app, step, &format!("使用 registry: {}", registry), "info");
        match install_openclaw_with_source(&app, step, "openclaw@latest", Some(registry)) {
            Ok(version) => {
                emit_log(&app, step, &format!("OpenClaw 安装成功: {}", version), "success");
                return Ok("OpenClaw 安装成功".to_string());
            }
            Err(error) => emit_log(&app, step, &format!("使用 {} 安装失败: {}", registry, error), "warn"),
        }
    }

    Err("OpenClaw 安装失败，所有安装源均不可用".to_string())
}


#[tauri::command]
pub async fn install_git(app: AppHandle) -> Result<String, String> {
    let step = "install_git";
    #[cfg(target_os = "windows")]
    {
        emit_log(&app, step, "Windows: 使用安装包内置 Git 离线包安装...", "info");
        let install_dir = install_bundled_windows_git(&app, step)?;
        emit_log(&app, step, &format!("Git 安装成功: {}", install_dir.display()), "success");
        return Ok("Git 安装成功".to_string());
    }
    #[cfg(not(target_os = "windows"))]
    emit_log(&app, step, "开始安装 Git...", "info");

    #[cfg(target_os = "macos")]
    {
        // macOS: 尝试 xcode-select --install
        emit_log(&app, step, "macOS: 尝试通过 Xcode Command Line Tools 安装 Git...", "info");
        match run_shell_with_log(&app, step, "xcode-select --install") {
            Ok(_) => {
                emit_log(&app, step, "已触发 Xcode CLT 安装，请在弹出的对话框中确认安装", "success");
                return Ok("请在系统弹窗中确认安装 Xcode Command Line Tools".to_string());
            }
            Err(_) => {
                // 可能已经安装了
                if check_git_installed().installed {
                    emit_log(&app, step, "Git 已安装", "success");
                    return Ok("Git 已安装".to_string());
                }
                return Err("Git 安装失败，请手动安装: https://git-scm.com/download/mac".to_string());
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 尝试多种包管理器
        let pkg_managers = [
            ("apt", "sudo apt install -y git"),
            ("yum", "sudo yum install -y git"),
            ("dnf", "sudo dnf install -y git"),
            ("pacman", "sudo pacman -S --noconfirm git"),
        ];

        for (name, cmd) in &pkg_managers {
            if run_shell(&format!("which {}", name)).is_ok() {
                emit_log(&app, step, &format!("使用 {} 安装 Git...", name), "info");
                match run_shell_with_log(&app, step, cmd) {
                    Ok(_) => {
                        emit_log(&app, step, "Git 安装成功!", "success");
                        return Ok("Git 安装成功".to_string());
                    }
                    Err(e) => {
                        emit_log(&app, step, &format!("{} 安装失败: {}", name, e), "warn");
                    }
                }
            }
        }

        return Err("Git 安装失败，请手动安装: https://git-scm.com/download/linux".to_string());
    }

    #[allow(unreachable_code)]
    Err("不支持的操作系统".to_string())
}

/// 执行完整安装流程
#[tauri::command]
pub async fn run_full_install(app: AppHandle) -> Result<String, String> {
    let total_steps: u8 = 5;

    emit_progress(&app, 1, total_steps, "环境检测", "running");
    emit_log(&app, "check", "开始环境检测...", "info");

    let env = check_environment().await;
    emit_log(&app, "check", &format!("系统: {} {}", env.system.os, env.system.arch), "info");
    emit_log(&app, "check", &format!("网络环境: {}", if env.network_region == "china" { "中国大陆" } else { "国际" }), "info");
    emit_log(&app, "check", &format!("Git: {}", if env.git.installed { format!("✓ {}", env.git.version.as_deref().unwrap_or("")) } else { "✗ 未安装".to_string() }), if env.git.installed { "success" } else { "warn" });
    emit_log(&app, "check", &format!("Node.js: {}", if env.node.installed { format!("✓ v{}", env.node.version.as_deref().unwrap_or("")) } else { "✗ 未安装".to_string() }), if env.node.meets_requirement { "success" } else { "warn" });
    emit_log(&app, "check", &format!("OpenClaw: {}", if env.openclaw.installed { format!("✓ {}", env.openclaw.version.as_deref().unwrap_or("")) } else { "✗ 未安装".to_string() }), if env.openclaw.installed { "success" } else { "warn" });
    emit_progress(&app, 1, total_steps, "环境检测", "success");

    let use_china = env.network_region == "china";

    emit_progress(&app, 2, total_steps, "安装 Git", "running");
    if !env.git.installed {
        match install_git(app.clone()).await {
            Ok(message) => {
                emit_log(&app, "install_git", &message, "success");
                emit_progress(&app, 2, total_steps, "安装 Git", "success");
            }
            Err(error) => {
                emit_log(&app, "install_git", &format!("Git 安装失败: {}", error), "error");
                emit_log(&app, "install_git", "Git 不是阻塞项，继续安装后续组件", "warn");
                emit_progress(&app, 2, total_steps, "安装 Git", "success");
            }
        }
    } else {
        emit_log(&app, "install_git", "Git 已安装，跳过", "success");
        emit_progress(&app, 2, total_steps, "安装 Git", "success");
    }

    emit_progress(&app, 3, total_steps, "安装 Node.js", "running");
    if git_install_is_blocking_in_full_install() && !check_git_installed().installed {
        emit_log(&app, "install_git", "Git is required before continuing.", "error");
        emit_progress(&app, 2, total_steps, "安装 Git", "error");
        return Err("Git 安装失败，已停止后续安装".to_string());
    }

    if env.node.meets_requirement {
        expose_managed_runtime_to_user_path_silently()?;
        emit_log(&app, "install_node", "Node.js >= 22 已满足要求，跳过安装", "success");
        emit_progress(&app, 3, total_steps, "安装 Node.js", "success");
    } else {
        install_node_via_fnm(app.clone(), MANAGED_NODE_VERSION.to_string(), use_china)
            .await
            .map_err(|error| {
                emit_progress(&app, 3, total_steps, "安装 Node.js", "error");
                error
            })?;
        emit_progress(&app, 3, total_steps, "安装 Node.js", "success");
    }

    emit_progress(&app, 4, total_steps, "安装 OpenClaw", "running");
    if !env.openclaw.installed {
        install_openclaw(app.clone(), use_china).await.map_err(|error| {
            emit_progress(&app, 4, total_steps, "安装 OpenClaw", "error");
            error
        })?;
    } else {
        expose_managed_runtime_to_user_path_silently()?;
        emit_log(&app, "install_openclaw", "OpenClaw 已安装，跳过", "success");
    }
    emit_progress(&app, 4, total_steps, "安装 OpenClaw", "success");

    emit_progress(&app, 5, total_steps, "验证安装", "running");
    emit_log(&app, "verify", "验证安装结果...", "info");

    let final_status = check_openclaw_installed();
    if final_status.installed {
        emit_log(&app, "verify", &format!("OpenClaw {} 安装成功", final_status.version.as_deref().unwrap_or("")), "success");
        emit_progress(&app, 5, total_steps, "验证安装", "success");
        Ok("安装完成".to_string())
    } else {
        emit_log(&app, "verify", "验证失败: openclaw 命令不可用", "error");
        emit_log(&app, "verify", "已写入用户 PATH，可重新打开终端后执行 openclaw --version 复检", "warn");
        emit_progress(&app, 5, total_steps, "验证安装", "error");
        Err("安装验证失败，请重新打开终端后重试".to_string())
    }
}


#[tauri::command]
pub async fn open_terminal_with_command(command: String) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let wrapped_command = format!("{}; exec $SHELL -l", command);
        // macOS: 使用 osascript 打开 Terminal.app 并执行命令
        let script = format!(
            "tell application \"Terminal\"\n\
             activate\n\
             do script \"{}\"\n\
             end tell",
            wrapped_command.replace("\"", "\\\"")
        );

        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;

        Ok("已打开终端".to_string())
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 start 打开 cmd 并执行命令
        Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", &command])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;

        Ok("已打开终端".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 尝试多种终端模拟器，执行后保持窗口不自动关闭
        let wrapped_command = format!("{}; exec ${SHELL:-bash} -l", command);
        let terminals = ["gnome-terminal", "konsole", "xterm", "x-terminal-emulator"];

        for term in &terminals {
            let exists = Command::new("which")
                .arg(term)
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false);
            if !exists {
                continue;
            }

            let mut cmd = Command::new(term);
            match *term {
                "gnome-terminal" => {
                    cmd.args(["--", "bash", "-lc", &wrapped_command]);
                }
                _ => {
                    cmd.args(["-e", "bash", "-lc", &wrapped_command]);
                }
            }

            if cmd.spawn().is_ok() {
                return Ok("已打开终端".to_string());
            }
        }

        Err("未找到可用的终端模拟器".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("不支持的操作系统".to_string())
    }
}

/// 生成默认配置文件
#[tauri::command]
pub async fn generate_default_config(app: AppHandle) -> Result<String, String> {
    let step = "generate_config";
    emit_log(&app, step, "生成默认配置文件...", "info");

    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let config_dir = home.join(".openclaw");
    let config_path = config_dir.join("openclaw.json");

    // 创建配置目录
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    // 生成随机 token
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let token = format!("openclaw-{:x}", timestamp);

    // 默认配置（最精简可启动）
    // 参考官方配置文档：最小可用配置 + gateway.mode=local，确保 gateway 可启动
    let default_config = build_default_openclaw_config(&token);

    // 写入配置文件
    let config_str = serde_json::to_string_pretty(&default_config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    std::fs::write(&config_path, config_str)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    emit_log(
        &app,
        step,
        &format!("配置文件已生成: {}", config_path.display()),
        "success",
    );

    Ok(format!("配置文件已生成: {}", config_path.display()))
}

fn build_default_openclaw_config(token: &str) -> serde_json::Value {
    serde_json::json!({
        "gateway": {
            "mode": "local",
            "port": 18789,
            "bind": "loopback",
            "auth": {
                "mode": "token",
                "token": token
            }
        },
        "models": {
            "mode": "merge",
            "providers": {}
        },
        "agents": {
            "defaults": {
                "workspace": "~/.openclaw/workspace",
                "model": {
                    "primary": "placeholder/complete-quick-setup"
                }
            }
        }
    })
}

#[cfg(target_os = "windows")]
fn windows_gateway_service_name() -> &'static str {
    "OpenClaw Gateway"
}

#[cfg(target_os = "windows")]
fn bundled_nssm_resource_candidates() -> &'static [&'static str] {
    &[
        "windows/nssm/nssm.exe",
        "resources/windows/nssm/nssm.exe",
    ]
}

#[cfg(target_os = "windows")]
fn resolve_bundled_nssm_executable(app: &AppHandle) -> Option<PathBuf> {
    for relative in bundled_nssm_resource_candidates() {
        if let Some(path) = app.path_resolver().resolve_resource(relative) {
            if path.exists() {
                return Some(path);
            }
        }
    }

    let local_candidates = [
        PathBuf::from("src-tauri").join("resources").join("windows").join("nssm").join("nssm.exe"),
        PathBuf::from("resources").join("windows").join("nssm").join("nssm.exe"),
    ];
    local_candidates.into_iter().find(|path| path.exists())
}

#[cfg(target_os = "windows")]
fn resolve_nssm_from_system_path() -> Option<PathBuf> {
    let output = run_shell("where nssm").ok()?;
    output.lines().find_map(|line| {
        let candidate = PathBuf::from(line.trim().trim_matches('"'));
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    })
}

#[cfg(target_os = "windows")]
fn resolve_windows_nssm_executable(app: &AppHandle) -> Option<PathBuf> {
    resolve_bundled_nssm_executable(app).or_else(resolve_nssm_from_system_path)
}

#[cfg(target_os = "windows")]
fn build_windows_gateway_service_path(node_bin_dir: &Path, openclaw_bin_path: &Path) -> Result<String, String> {
    let mut entries = Vec::new();
    entries.push(node_bin_dir.to_path_buf());
    if let Some(parent) = openclaw_bin_path.parent() {
        entries.push(parent.to_path_buf());
    }
    if let Some(current) = std::env::var_os("PATH") {
        entries.extend(std::env::split_paths(&current));
    }
    std::env::join_paths(entries)
        .map(|value| value.to_string_lossy().to_string())
        .map_err(|e| format!("构建网关服务 PATH 失败: {}", e))
}

#[cfg(target_os = "windows")]
fn write_windows_gateway_service_script(home_dir: &Path, openclaw_bin_path: &Path, node_bin_dir: &Path) -> Result<PathBuf, String> {
    let service_dir = home_dir.join(".openclaw").join("service");
    std::fs::create_dir_all(&service_dir).map_err(|e| format!("创建服务目录失败: {}", e))?;

    let script_path = service_dir.join("gateway-service.cmd");
    let path_value = build_windows_gateway_service_path(node_bin_dir, openclaw_bin_path)?;
    let script = format!(
        "@echo off\r\nsetlocal\r\nset \"USERPROFILE={}\"\r\nset \"HOME={}\"\r\nset \"PATH={}\"\r\ncall {} gateway run\r\n",
        home_dir.display(),
        home_dir.display(),
        path_value.replace('"', ""),
        shell_quote(&openclaw_bin_path.to_string_lossy()),
    );

    std::fs::write(&script_path, script).map_err(|e| format!("写入网关服务脚本失败: {}", e))?;
    Ok(script_path)
}

#[cfg(target_os = "windows")]
fn format_command_for_log(program: &Path, args: &[&str]) -> String {
    let mut command = shell_quote(&program.to_string_lossy());
    for arg in args {
        command.push(' ');
        command.push_str(&shell_quote(arg));
    }
    command
}

#[cfg(target_os = "windows")]
fn run_windows_command_direct(program: &Path, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new(program);
    apply_no_window(&mut command);
    command.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = command.output().map_err(|error| format!("执行命令失败: {}", error))?;
    let stdout = decode_command_output(&output.stdout);
    let stderr = decode_command_output(&output.stderr);

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format_command_failure(output.status.code(), &stdout, &stderr))
    }
}

#[cfg(target_os = "windows")]
fn run_nssm_command_with_log(app: &AppHandle, step: &str, nssm_path: &Path, args: &[&str]) -> Result<String, String> {
    emit_log(app, step, &format!("$ {}", format_command_for_log(nssm_path, args)), "info");
    match run_windows_command_direct(nssm_path, args) {
        Ok(stdout) => {
            for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
                emit_log(app, step, line, "info");
            }
            Ok(stdout)
        }
        Err(error) => {
            for line in error.lines().filter(|line| !line.trim().is_empty()) {
                emit_log(app, step, line, "warn");
            }
            Err(error)
        }
    }
}

#[cfg(target_os = "windows")]
fn run_nssm_command(nssm_path: &Path, args: &[&str]) -> Result<String, String> {
    run_windows_command_direct(nssm_path, args)
}

#[cfg(target_os = "windows")]
fn build_windows_nssm_service_install_args(service_name: &str, cmd_exe: &str, script_path: &Path) -> Vec<String> {
    vec![
        "install".to_string(),
        service_name.to_string(),
        cmd_exe.to_string(),
        "/d".to_string(),
        "/s".to_string(),
        "/c".to_string(),
        format!("\"{}\"", script_path.display()),
    ]
}

#[cfg(target_os = "windows")]
fn build_windows_gateway_service_install_script(
    nssm_path: &Path,
    service_name: &str,
    cmd_exe: &str,
    script_path: &Path,
    home_dir: &Path,
    stdout_log: &Path,
    stderr_log: &Path,
    result_path: &Path,
) -> String {
    let install_args = build_windows_nssm_service_install_args(service_name, cmd_exe, script_path);
    let install_args_ps = install_args
        .iter()
        .map(|arg| format!("'{}'", powershell_single_quote(arg)))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        concat!(
            "$ErrorActionPreference = 'Stop'\n",
            "$nssm = '{}'\n",
            "$resultPath = '{}'\n",
            "function Invoke-Nssm {{\n",
            "  param([string[]]$Args, [switch]$IgnoreErrors)\n",
            "  & $nssm @Args\n",
            "  if ($LASTEXITCODE -ne 0 -and -not $IgnoreErrors) {{\n",
            "    throw \"nssm failed ($LASTEXITCODE): $($Args -join ' ')\"\n",
            "  }}\n",
            "}}\n",
            "try {{\n",
            "  Remove-Item -Path $resultPath -Force -ErrorAction SilentlyContinue\n",
            "  Invoke-Nssm -Args @('stop', '{}') -IgnoreErrors\n",
            "  Invoke-Nssm -Args @('remove', '{}', 'confirm') -IgnoreErrors\n",
            "  Invoke-Nssm -Args @({})\n",
            "  Invoke-Nssm -Args @('set', '{}', 'AppDirectory', '{}')\n",
            "  Invoke-Nssm -Args @('set', '{}', 'Description', 'OpenClaw Gateway')\n",
            "  Invoke-Nssm -Args @('set', '{}', 'Start', 'SERVICE_AUTO_START')\n",
            "  Invoke-Nssm -Args @('set', '{}', 'AppExit', 'Default', 'Restart')\n",
            "  Invoke-Nssm -Args @('set', '{}', 'AppStdout', '{}')\n",
            "  Invoke-Nssm -Args @('set', '{}', 'AppStderr', '{}')\n",
            "  Invoke-Nssm -Args @('start', '{}')\n",
            "  Set-Content -Path $resultPath -Value 'ok' -Encoding UTF8\n",
            "  exit 0\n",
            "}} catch {{\n",
            "  Set-Content -Path $resultPath -Value $_.Exception.Message -Encoding UTF8\n",
            "  exit 1\n",
            "}}\n"
        ),
        powershell_single_quote(&nssm_path.to_string_lossy()),
        powershell_single_quote(&result_path.to_string_lossy()),
        powershell_single_quote(service_name),
        powershell_single_quote(service_name),
        install_args_ps,
        powershell_single_quote(service_name),
        powershell_single_quote(&home_dir.to_string_lossy()),
        powershell_single_quote(service_name),
        powershell_single_quote(service_name),
        powershell_single_quote(service_name),
        powershell_single_quote(service_name),
        powershell_single_quote(&stdout_log.to_string_lossy()),
        powershell_single_quote(service_name),
        powershell_single_quote(&stderr_log.to_string_lossy()),
        powershell_single_quote(service_name),
    )
}

#[cfg(target_os = "windows")]
fn build_windows_elevated_powershell_command(script_path: &Path) -> Vec<String> {
    vec![
        "-NoProfile".to_string(),
        "-ExecutionPolicy".to_string(),
        "Bypass".to_string(),
        "-Command".to_string(),
        format!(
            "$p = Start-Process -FilePath 'powershell.exe' -Verb RunAs -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File','{}') -Wait -PassThru; exit $p.ExitCode",
            powershell_single_quote(&script_path.to_string_lossy())
        ),
    ]
}

#[cfg(target_os = "windows")]
fn build_windows_relaunch_as_admin_command(exe_path: &Path) -> Vec<String> {
    vec![
        "-NoProfile".to_string(),
        "-ExecutionPolicy".to_string(),
        "Bypass".to_string(),
        "-Command".to_string(),
        format!(
            "Start-Process -FilePath '{}' -Verb RunAs",
            powershell_single_quote(&exe_path.to_string_lossy())
        ),
    ]
}

#[cfg(target_os = "windows")]
fn run_elevated_powershell_file(script_path: &Path) -> Result<String, String> {
    let args = build_windows_elevated_powershell_command(script_path);
    let arg_refs = args.iter().map(|arg| arg.as_str()).collect::<Vec<_>>();
    run_windows_command_direct(Path::new("powershell.exe"), &arg_refs)
}

#[cfg(target_os = "windows")]
fn is_windows_admin_required_error(error: &str) -> bool {
    let normalized = error.to_lowercase();
    normalized.contains("access is denied")
        || normalized.contains("administrator access is needed")
        || error.contains("拒绝访问")
}

#[cfg(target_os = "windows")]
fn install_gateway_service_via_bundled_nssm_elevated(app: &AppHandle, step: &str) -> Result<String, String> {
    let nssm_path = resolve_windows_nssm_executable(app)
        .ok_or_else(|| "未找到可用的 nssm.exe（内置和系统 PATH 均未命中）".to_string())?;
    let openclaw_bin = detect_openclaw_bin_path()
        .ok_or_else(|| "未找到 openclaw 可执行文件，无法安装网关服务".to_string())?;
    let node_bin_dir = resolve_active_node_bin_dir()
        .ok_or_else(|| "未找到 node 运行目录，无法安装网关服务".to_string())?;
    let home_dir = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let service_script_path = write_windows_gateway_service_script(&home_dir, &openclaw_bin, &node_bin_dir)?;
    let service_dir = home_dir.join(".openclaw").join("service");
    std::fs::create_dir_all(&service_dir).map_err(|e| format!("创建服务目录失败: {}", e))?;
    let result_path = service_dir.join("install-gateway-service.result.txt");
    let elevated_script_path = service_dir.join("install-gateway-service.ps1");
    let service_name = windows_gateway_service_name();
    let cmd_exe = std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".to_string());
    let log_dir = home_dir.join(".openclaw").join("logs");
    std::fs::create_dir_all(&log_dir).map_err(|e| format!("创建服务日志目录失败: {}", e))?;
    let stdout_log = log_dir.join("gateway-service.stdout.log");
    let stderr_log = log_dir.join("gateway-service.stderr.log");
    let script = build_windows_gateway_service_install_script(
        &nssm_path,
        service_name,
        &cmd_exe,
        &service_script_path,
        &home_dir,
        &stdout_log,
        &stderr_log,
        &result_path,
    );
    std::fs::write(&elevated_script_path, script).map_err(|e| format!("写入提权安装脚本失败: {}", e))?;

    emit_log(app, step, "检测到需要管理员权限，正在请求 UAC 提权...", "warn");
    match run_elevated_powershell_file(&elevated_script_path) {
        Ok(_) => Ok(format!("网关服务已通过内置 nssm 安装: {}", service_name)),
        Err(error) => {
            let detail = std::fs::read_to_string(&result_path)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty() && value != "ok")
                .unwrap_or(error);
            Err(detail)
        }
    }
}

#[cfg(target_os = "windows")]
fn is_windows_gateway_service_installed(app: &AppHandle) -> bool {
    let Some(nssm_path) = resolve_windows_nssm_executable(app) else {
        return false;
    };
    run_nssm_command(&nssm_path, &["status", windows_gateway_service_name()]).is_ok()
}

#[cfg(target_os = "windows")]
fn control_gateway_service_via_nssm(app: &AppHandle, action: &str) -> Result<String, String> {
    let nssm_path = resolve_windows_nssm_executable(app)
        .ok_or_else(|| "未找到可用的 nssm.exe（内置和系统 PATH 均未命中）".to_string())?;
    let service_name = windows_gateway_service_name();
    run_nssm_command(&nssm_path, &[action, service_name])?;
    Ok(format!("网关服务已执行 {}: {}", action, service_name))
}
#[cfg(target_os = "windows")]
fn install_gateway_service_via_bundled_nssm(app: &AppHandle, step: &str) -> Result<String, String> {
    let nssm_path = resolve_windows_nssm_executable(app)
        .ok_or_else(|| "未找到可用的 nssm.exe（内置和系统 PATH 均未命中）".to_string())?;
    let openclaw_bin = detect_openclaw_bin_path()
        .ok_or_else(|| "未找到 openclaw 可执行文件，无法安装网关服务".to_string())?;
    let node_bin_dir = resolve_active_node_bin_dir()
        .ok_or_else(|| "未找到 node 运行目录，无法安装网关服务".to_string())?;
    let home_dir = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let script_path = write_windows_gateway_service_script(&home_dir, &openclaw_bin, &node_bin_dir)?;
    let service_name = windows_gateway_service_name();
    let cmd_exe = std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".to_string());
    let home_dir_str = home_dir.to_string_lossy().to_string();
    let log_dir = home_dir.join(".openclaw").join("logs");
    std::fs::create_dir_all(&log_dir).map_err(|e| format!("创建服务日志目录失败: {}", e))?;
    let stdout_log = log_dir.join("gateway-service.stdout.log");
    let stderr_log = log_dir.join("gateway-service.stderr.log");
    let stdout_log_str = stdout_log.to_string_lossy().to_string();
    let stderr_log_str = stderr_log.to_string_lossy().to_string();

    emit_log(app, step, &format!("使用 nssm 安装网关服务: {}", nssm_path.display()), "info");
    emit_log(app, step, &format!("网关服务脚本: {}", script_path.display()), "info");

    let _ = run_nssm_command_with_log(app, step, &nssm_path, &["stop", service_name]);
    let _ = run_nssm_command_with_log(app, step, &nssm_path, &["remove", service_name, "confirm"]);

    let install_args = build_windows_nssm_service_install_args(service_name, &cmd_exe, &script_path);
    let install_arg_refs = install_args.iter().map(|arg| arg.as_str()).collect::<Vec<_>>();
    run_nssm_command_with_log(
        app,
        step,
        &nssm_path,
        &install_arg_refs,
    )?;
    run_nssm_command_with_log(app, step, &nssm_path, &["set", service_name, "AppDirectory", home_dir_str.as_str()])?;
    run_nssm_command_with_log(app, step, &nssm_path, &["set", service_name, "Description", "OpenClaw Gateway"])?;
    run_nssm_command_with_log(app, step, &nssm_path, &["set", service_name, "Start", "SERVICE_AUTO_START"])?;
    run_nssm_command_with_log(app, step, &nssm_path, &["set", service_name, "AppExit", "Default", "Restart"])?;
    run_nssm_command_with_log(app, step, &nssm_path, &["set", service_name, "AppStdout", stdout_log_str.as_str()])?;
    run_nssm_command_with_log(app, step, &nssm_path, &["set", service_name, "AppStderr", stderr_log_str.as_str()])?;
    run_nssm_command_with_log(app, step, &nssm_path, &["start", service_name])?;

    Ok(format!("网关服务已通过内置 nssm 安装: {}", service_name))
}
/// 安装网关服务（后台自动启动）
#[tauri::command]
pub async fn install_gateway_service(app: AppHandle) -> Result<String, String> {
    let step = "install_service";
    emit_log(&app, step, "安装网关服务...", "info");

    #[cfg(target_os = "windows")]
    {
        match install_gateway_service_via_bundled_nssm(&app, step) {
            Ok(message) => {
                emit_log(&app, step, &message, "success");
                return Ok(message);
            }
            Err(error) => {
                if is_windows_admin_required_error(&error) {
                    match install_gateway_service_via_bundled_nssm_elevated(&app, step) {
                        Ok(message) => {
                            emit_log(&app, step, &message, "success");
                            return Ok(message);
                        }
                        Err(elevated_error) => {
                            let message = if is_windows_admin_required_error(&elevated_error) {
                                format!("网关服务安装失败（需要管理员权限）: {}", elevated_error)
                            } else {
                                format!("网关服务安装失败: {}", elevated_error)
                            };
                            emit_log(&app, step, &message, "error");
                            return Err(message);
                        }
                    }
                }

                let message = format!("网关服务安装失败: {}", error);
                emit_log(&app, step, &message, "error");
                return Err(message);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let cmd = with_fnm_env("openclaw gateway install");
        match run_shell_with_log(&app, step, &cmd) {
            Ok(_) => {
                emit_log(&app, step, "网关服务安装成功!", "success");
                Ok("网关服务已安装".to_string())
            }
            Err(e) => {
                emit_log(&app, step, &format!("网关服务安装失败: {}", e), "error");
                Err(format!("网关服务安装失败: {}", e))
            }
        }
    }
}
/// 启动本地网关服务
#[tauri::command]
pub async fn start_gateway(_app: AppHandle) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        if is_windows_gateway_service_installed(&_app) {
            return control_gateway_service_via_nssm(&_app, "start");
        }
    }

    let cmd = with_fnm_env("openclaw gateway start");
    let output = run_shell(&cmd)?;
    if output.is_empty() {
        Ok("网关启动命令已执行".to_string())
    } else {
        Ok(output)
    }
}

/// 停止本地网关服务
#[tauri::command]
pub async fn stop_gateway(_app: AppHandle) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        if is_windows_gateway_service_installed(&_app) {
            return control_gateway_service_via_nssm(&_app, "stop");
        }
    }

    let cmd = with_fnm_env("openclaw gateway stop");
    let output = run_shell(&cmd)?;
    if output.is_empty() {
        Ok("网关停止命令已执行".to_string())
    } else {
        Ok(output)
    }
}

#[cfg(target_os = "windows")]
fn remove_windows_user_path_entry(dir: &Path) -> Result<(), String> {
    let target = dir.to_string_lossy().replace("'", "''");
    let cmd = format!(
        r#"powershell -NoProfile -Command "$target='{}'; $current=[Environment]::GetEnvironmentVariable('Path','User'); if ([string]::IsNullOrWhiteSpace($current)) {{ exit 0 }}; $segments=@($current -split ';' | Where-Object {{ $_ -and $_.Trim() -ne '' -and $_.Trim() -ne $target }}); $updated = if ($segments.Count -gt 0) {{ $segments -join ';' }} else {{ '' }}; [Environment]::SetEnvironmentVariable('Path', $updated, 'User')""#,
        target
    );
    run_shell(&cmd).map(|_| ())
}

fn remove_process_path_entries(entries: &[PathBuf]) -> Result<(), String> {
    if entries.is_empty() {
        return Ok(());
    }

    let current = std::env::var_os("PATH").unwrap_or_default();
    let updated_entries: Vec<PathBuf> = std::env::split_paths(&current)
        .filter(|existing| !entries.iter().any(|candidate| candidate == existing))
        .collect();
    let updated = std::env::join_paths(updated_entries)
        .map_err(|error| format!("更新进程 PATH 失败: {}", error))?;
    std::env::set_var("PATH", updated);
    Ok(())
}

fn managed_runtime_user_path_candidates() -> Vec<PathBuf> {
    let mut entries = Vec::new();
    if let Some(node_bin_dir) = resolve_managed_node_bin_dir() {
        entries.push(node_bin_dir);
    }
    if let Ok(npm_bin_dir) = managed_npm_bin_dir() {
        entries.push(npm_bin_dir);
    }
    entries.dedup();
    entries
}

#[cfg(not(target_os = "windows"))]
fn remove_unix_managed_path_snippet() -> Result<(), String> {
    for rc_file in unix_shell_rc_files()? {
        let current = std::fs::read_to_string(&rc_file).unwrap_or_default();
        let updated = if let (Some(start), Some(end)) = (
            current.find(OPENCLAW_MANAGED_PATH_MARKER_START),
            current.find(OPENCLAW_MANAGED_PATH_MARKER_END),
        ) {
            let mut suffix = current[end + OPENCLAW_MANAGED_PATH_MARKER_END.len()..].to_string();
            suffix = suffix.trim_start_matches(['\r', '\n']).to_string();
            let prefix = current[..start].trim_end_matches(['\r', '\n']).to_string();
            if prefix.is_empty() {
                suffix
            } else if suffix.is_empty() {
                prefix
            } else {
                format!("{}\n{}", prefix, suffix)
            }
        } else {
            current.clone()
        };

        if updated != current {
            if updated.trim().is_empty() {
                let _ = std::fs::remove_file(&rc_file);
            } else {
                std::fs::write(&rc_file, updated)
                    .map_err(|error| format!("移除 shell PATH 配置失败 {}: {}", rc_file.display(), error))?;
            }
        }
    }
    Ok(())
}

fn remove_managed_runtime_from_user_path() -> Result<(), String> {
    let entries = managed_runtime_user_path_candidates();
    remove_process_path_entries(&entries)?;

    #[cfg(target_os = "windows")]
    {
        for entry in entries {
            let _ = remove_windows_user_path_entry(&entry);
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        remove_unix_managed_path_snippet()
    }
}

fn remove_managed_npm_prefix_config() -> Result<(), String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let npmrc = home.join(".npmrc");
    if !npmrc.exists() {
        return Ok(());
    }

    let prefix = managed_npm_prefix()?;
    let desired = format!("prefix={}", prefix.to_string_lossy().replace('\\', "/"));
    let current = std::fs::read_to_string(&npmrc).unwrap_or_default();
    let lines: Vec<String> = current
        .lines()
        .filter(|line| line.trim() != desired)
        .map(|line| line.to_string())
        .collect();

    if lines.len() == current.lines().count() {
        return Ok(());
    }

    if lines.is_empty() {
        let _ = std::fs::remove_file(&npmrc);
        return Ok(());
    }

    std::fs::write(&npmrc, format!("{}\n", lines.join("\n")))
        .map_err(|error| format!("清理 .npmrc 失败: {}", error))?;
    Ok(())
}

fn remove_path_if_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|error| format!("删除目录失败 {}: {}", path.display(), error))?;
    } else {
        std::fs::remove_file(path).map_err(|error| format!("删除文件失败 {}: {}", path.display(), error))?;
    }
    Ok(())
}

fn remove_lingering_openclaw_binaries() -> Result<(), String> {
    if let Some(bin_path) = detect_openclaw_bin_path() {
        let _ = remove_path_if_exists(&bin_path);
    }

    if let Ok(npm_bin_dir) = managed_npm_bin_dir() {
        let names: &[&str] = if cfg!(target_os = "windows") {
            &["openclaw.cmd", "openclaw.ps1", "openclaw"]
        } else {
            &["openclaw"]
        };
        for name in names {
            let _ = remove_path_if_exists(&npm_bin_dir.join(name));
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn uninstall_windows_gateway_service(app: &AppHandle, step: &str) -> Result<(), String> {
    let service_name = windows_gateway_service_name();

    if let Some(nssm_path) = resolve_windows_nssm_executable(app) {
        let _ = run_nssm_command_with_log(app, step, &nssm_path, &["stop", service_name]);
        let _ = run_nssm_command_with_log(app, step, &nssm_path, &["remove", service_name, "confirm"]);
        return Ok(());
    }

    let _ = run_cmd("sc.exe", &["stop", service_name]);
    let _ = run_cmd("sc.exe", &["delete", service_name]);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn uninstall_unix_gateway_service(app: &AppHandle, step: &str) -> Result<(), String> {
    let _ = run_shell_with_log(app, step, &with_fnm_env("openclaw gateway stop"));
    run_shell_with_log(
        app,
        step,
        &with_fnm_env("openclaw gateway uninstall || openclaw gateway remove"),
    )
    .map(|_| ())
}

#[tauri::command]
pub async fn uninstall_openclaw(app: AppHandle, remove_config_dir: bool) -> Result<String, String> {
    let step = "uninstall_openclaw";
    emit_log(&app, step, "开始卸载 OpenClaw...", "info");

    let package_root_before = detect_openclaw_package_root().ok();
    let managed_root = openclaw_managed_root()?;
    let managed_entries = managed_runtime_user_path_candidates();

    #[cfg(target_os = "windows")]
    uninstall_windows_gateway_service(&app, step)?;

    #[cfg(not(target_os = "windows"))]
    {
        if let Err(error) = uninstall_unix_gateway_service(&app, step) {
            emit_log(&app, step, &format!("卸载网关服务失败: {}", error), "warn");
        }
    }

    let _ = prepare_managed_runtime_process_env();
    let npm = npm_executable();
    let uninstall_cmd = format!("{} uninstall -g openclaw --no-fund --no-audit", npm);
    match run_shell_with_log(&app, step, &with_fnm_env(&uninstall_cmd)) {
        Ok(_) => emit_log(&app, step, "openclaw npm 包已卸载", "success"),
        Err(error) => emit_log(&app, step, &format!("npm 卸载返回异常，继续清理残留: {}", error), "warn"),
    }

    if let Some(package_root) = package_root_before {
        let _ = remove_path_if_exists(&package_root);
    }
    let _ = remove_lingering_openclaw_binaries();
    let _ = remove_process_path_entries(&managed_entries);

    if remove_config_dir {
        let _ = remove_managed_runtime_from_user_path();
        let _ = remove_managed_npm_prefix_config();
        let _ = remove_path_if_exists(&managed_root);
    }

    let remaining_status = check_openclaw_installed();
    if remaining_status.installed {
        return Err("卸载后仍检测到 OpenClaw，请检查是否存在其他全局安装来源".to_string());
    }

    let message = if remove_config_dir {
        "OpenClaw 已卸载，~/.openclaw 与相关环境配置已清理".to_string()
    } else {
        "OpenClaw 已卸载，~/.openclaw 已保留".to_string()
    };
    emit_log(&app, step, &message, "success");
    Ok(message)
}
/// 获取消息渠道扩展安装状态
#[tauri::command]
pub async fn get_channel_extension_status() -> Result<ChannelExtensionStatus, String> {
    Ok(ChannelExtensionStatus {
        feishu_installed: is_channel_extension_installed("feishu"),
        wecom_installed: is_channel_extension_installed("wecom"),
        qq_installed: is_channel_extension_installed("qq"),
        dingtalk_installed: is_channel_extension_installed("dingtalk"),
    })
}

/// 安装消息渠道扩展（feishu / dingtalk）
#[tauri::command]
pub async fn install_channel_extension(app: AppHandle, channel_id: String) -> Result<String, String> {
    if CHANNEL_EXTENSION_INSTALLING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("已有扩展安装任务正在进行，请稍后再试".to_string());
    }

    let install_result = (|| -> Result<String, String> {
        let (npm_package, target_dir_name, _) = get_extension_meta(&channel_id)?;
        let npm = npm_executable();
        let extensions_root = get_extensions_root()?;
        std::fs::create_dir_all(&extensions_root).map_err(|e| format!("创建扩展目录失败: {}", e))?;

        emit_channel_extension_state(&app, &channel_id, "running");
        emit_channel_extension_log(
            &app,
            &channel_id,
            &format!("开始安装 {} 扩展", channel_id),
            "info",
        );
        emit_channel_extension_log(
            &app,
            &channel_id,
            &format!("目标包: {}", npm_package),
            "info",
        );

        let temp_dir = std::env::temp_dir()
            .join(format!("openclawswitch-extension-{}-{}", target_dir_name, now_ms()));
        std::fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;
        emit_channel_extension_log(
            &app,
            &channel_id,
            &format!("临时目录: {}", temp_dir.display()),
            "info",
        );

        let mut pack_command = Command::new(npm);
        apply_no_window(&mut pack_command);
        let pack_output = pack_command
            .arg("pack")
            .arg(npm_package)
            .current_dir(&temp_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("执行 npm pack 失败: {}", e))?;

        if !pack_output.status.success() {
            let stderr = String::from_utf8_lossy(&pack_output.stderr).trim().to_string();
            let msg = if stderr.is_empty() {
                "npm pack 失败".to_string()
            } else {
                format!("npm pack 失败: {}", stderr)
            };
            emit_channel_extension_log(&app, &channel_id, &msg, "error");
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(msg);
        }

        let pack_stdout = String::from_utf8_lossy(&pack_output.stdout);
        let tar_name = pack_stdout
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .ok_or("无法解析 npm pack 输出文件名")?;
        let tarball_path = temp_dir.join(&tar_name);
        if !tarball_path.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(format!("未找到下载包: {}", tarball_path.display()));
        }
        emit_channel_extension_log(
            &app,
            &channel_id,
            &format!("包下载完成: {}", tar_name),
            "success",
        );

        let tar_file = File::open(&tarball_path).map_err(|e| format!("打开压缩包失败: {}", e))?;
        let gzip = GzDecoder::new(tar_file);
        let mut archive = tar::Archive::new(gzip);
        archive
            .unpack(&temp_dir)
            .map_err(|e| format!("解压压缩包失败: {}", e))?;
        emit_channel_extension_log(&app, &channel_id, "解压完成", "success");

        let unpacked_dir = temp_dir.join("package");
        if !unpacked_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err("解压结果异常: 未找到 package 目录".to_string());
        }

        let target_dir = extensions_root.join(target_dir_name);
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir).map_err(|e| format!("清理旧扩展失败: {}", e))?;
        }

        match std::fs::rename(&unpacked_dir, &target_dir) {
            Ok(_) => {}
            Err(_) => {
                copy_dir_all(&unpacked_dir, &target_dir)?;
                let _ = std::fs::remove_dir_all(&unpacked_dir);
            }
        }
        emit_channel_extension_log(
            &app,
            &channel_id,
            &format!("扩展目录已就绪: {}", target_dir.display()),
            "success",
        );

        let mut install_command = Command::new(npm);
        apply_no_window(&mut install_command);
        let install_output = install_command
            .arg("install")
            .arg("--registry=https://registry.npmmirror.com")
            .current_dir(&target_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("执行 npm install 失败: {}", e))?;

        if !install_output.status.success() {
            let stderr = String::from_utf8_lossy(&install_output.stderr).trim().to_string();
            let msg = if stderr.is_empty() {
                "npm install 失败".to_string()
            } else {
                format!("npm install 失败: {}", stderr)
            };
            emit_channel_extension_log(&app, &channel_id, &msg, "error");
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(msg);
        }

        emit_channel_extension_log(
            &app,
            &channel_id,
            "依赖安装完成（registry: npmmirror）",
            "success",
        );

        if !is_channel_extension_installed(&channel_id) {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err("安装完成校验失败，请重试".to_string());
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
        let ok_msg = format!("{} 扩展安装完成", channel_id);
        emit_channel_extension_log(&app, &channel_id, &ok_msg, "success");
        Ok(ok_msg)
    })();

    CHANNEL_EXTENSION_INSTALLING.store(false, Ordering::SeqCst);

    match install_result {
        Ok(message) => {
            emit_channel_extension_state(&app, &channel_id, "success");
            Ok(message)
        }
        Err(error) => {
            emit_channel_extension_state(&app, &channel_id, "error");
            Err(error)
        }
    }
}

/// 启动 openclaw logs --follow 实时日志跟踪（运行中时拒绝重复启动）
#[tauri::command]
pub fn start_openclaw_logs_follow(app: AppHandle) -> Result<bool, String> {
    if LOG_FOLLOW_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        emit_runtime_log_status(&app, true, Some("日志跟踪已在运行".to_string()));
        return Ok(false);
    }

    emit_runtime_log_status(&app, true, Some("开始跟踪 openclaw logs --follow".to_string()));

    thread::spawn(move || {
        let follow_cmd = with_fnm_env("openclaw logs --follow");
        let spawn_result = if cfg!(target_os = "windows") {
            let mut command = Command::new("cmd");
            apply_no_window(&mut command);
            command
                .args(["/c", &follow_cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        } else {
            Command::new("sh")
                .args(["-c", &follow_cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        };

        let mut child = match spawn_result {
            Ok(child) => child,
            Err(error) => {
                let reason = format!("启动日志跟踪失败: {}", error);
                emit_runtime_log(&app, &reason, "error");
                LOG_FOLLOW_RUNNING.store(false, Ordering::SeqCst);
                emit_runtime_log_status(&app, false, Some(reason));
                return;
            }
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let stderr_app = app.clone();
        let stderr_handle = stderr.map(move |err| {
            thread::spawn(move || {
                let reader = BufReader::new(err);
                for line in reader.lines().flatten() {
                    if !line.trim().is_empty() {
                        emit_runtime_log(&stderr_app, &line, "warn");
                    }
                }
            })
        });

        if let Some(out) = stdout {
            let reader = BufReader::new(out);
            for line in reader.lines().flatten() {
                if !line.trim().is_empty() {
                    emit_runtime_log(&app, &line, "info");
                }
            }
        }

        if let Some(handle) = stderr_handle {
            let _ = handle.join();
        }

        let reason = match child.wait() {
            Ok(status) if status.success() => "日志跟踪已结束".to_string(),
            Ok(status) => format!("日志跟踪已退出，退出码: {:?}", status.code()),
            Err(error) => format!("日志跟踪进程异常: {}", error),
        };
        let level = if reason.contains("异常") || reason.contains("退出码") {
            "warn"
        } else {
            "info"
        };

        emit_runtime_log(&app, &reason, level);
        LOG_FOLLOW_RUNNING.store(false, Ordering::SeqCst);
        emit_runtime_log_status(&app, false, Some(reason));
    });

    Ok(true)
}

/// 查询服务诊断任务是否正在执行
#[tauri::command]
pub fn is_openclaw_doctor_running() -> bool {
    DOCTOR_RUNNING.load(Ordering::SeqCst)
}

/// 启动服务诊断（fix=true 时执行 openclaw doctor --fix）
#[tauri::command]
pub fn start_openclaw_doctor(app: AppHandle, fix: bool) -> Result<bool, String> {
    let mode = if fix { "fix" } else { "check" };
    if DOCTOR_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        emit_doctor_status(
            &app,
            true,
            mode,
            None,
            None,
            Some("诊断任务已在运行".to_string()),
        );
        return Ok(false);
    }

    let doctor_cmd = if fix {
        "openclaw doctor --fix"
    } else {
        "openclaw doctor"
    };
    let wrapped_cmd = with_fnm_env(doctor_cmd);
    let mode_name = mode.to_string();

    emit_doctor_status(
        &app,
        true,
        &mode_name,
        None,
        None,
        Some(format!("开始执行 {}", doctor_cmd)),
    );
    emit_doctor_log(&app, &format!("$ {}", doctor_cmd), "info");

    thread::spawn(move || {
        let spawn_result = if cfg!(target_os = "windows") {
            let mut command = Command::new("cmd");
            apply_no_window(&mut command);
            command
                .args(["/c", &wrapped_cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        } else {
            Command::new("sh")
                .args(["-c", &wrapped_cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        };

        let mut child = match spawn_result {
            Ok(child) => child,
            Err(error) => {
                let reason = format!("启动服务诊断失败: {}", error);
                emit_doctor_log(&app, &reason, "error");
                DOCTOR_RUNNING.store(false, Ordering::SeqCst);
                emit_doctor_status(&app, false, &mode_name, Some(false), None, Some(reason));
                return;
            }
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let stderr_app = app.clone();
        let stderr_handle = stderr.map(move |err| {
            thread::spawn(move || {
                let reader = BufReader::new(err);
                for line in reader.lines().flatten() {
                    let text = line.trim();
                    if text.is_empty() {
                        continue;
                    }
                    let level = detect_doctor_log_level(text, true);
                    emit_doctor_log(&stderr_app, text, level);
                }
            })
        });

        if let Some(out) = stdout {
            let reader = BufReader::new(out);
            for line in reader.lines().flatten() {
                let text = line.trim();
                if text.is_empty() {
                    continue;
                }
                let level = detect_doctor_log_level(text, false);
                emit_doctor_log(&app, text, level);
            }
        }

        if let Some(handle) = stderr_handle {
            let _ = handle.join();
        }

        match child.wait() {
            Ok(status) => {
                let success = status.success();
                let exit_code = status.code();
                let reason = if success {
                    if mode_name == "fix" {
                        "自动修复执行完成".to_string()
                    } else {
                        "服务诊断执行完成".to_string()
                    }
                } else {
                    format!("服务诊断执行失败，退出码: {:?}", exit_code)
                };
                let level = if success { "success" } else { "error" };

                emit_doctor_log(&app, &reason, level);
                DOCTOR_RUNNING.store(false, Ordering::SeqCst);
                emit_doctor_status(
                    &app,
                    false,
                    &mode_name,
                    Some(success),
                    exit_code,
                    Some(reason),
                );
            }
            Err(error) => {
                let reason = format!("服务诊断进程异常: {}", error);
                emit_doctor_log(&app, &reason, "error");
                DOCTOR_RUNNING.store(false, Ordering::SeqCst);
                emit_doctor_status(&app, false, &mode_name, Some(false), None, Some(reason));
            }
        }
    });

    Ok(true)
}

/// 打开 Web UI
#[tauri::command]
pub async fn open_web_ui() -> Result<String, String> {
    let output = run_shell(&with_fnm_env("openclaw dashboard --no-open"))?;
    let url = extract_dashboard_url(&output)
        .ok_or_else(|| format!("无法解析 Dashboard URL，命令输出: {}", output))?;

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/c", "start", "", &url])
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }

    Ok(format!("已打开 Dashboard: {}", url))
}

#[tauri::command]
pub async fn relaunch_as_admin(app: AppHandle) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let exe_path = std::env::current_exe().map_err(|e| format!("无法获取当前程序路径: {}", e))?;
        let args = build_windows_relaunch_as_admin_command(&exe_path);

        Command::new("powershell.exe")
            .args(args)
            .spawn()
            .map_err(|e| format!("请求管理员权限失败: {}", e))?;

        let app_handle = app.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(400));
            app_handle.exit(0);
        });

        return Ok("正在以管理员身份重新启动应用...".to_string());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        Err("仅支持 Windows 管理员重启".to_string())
    }
}

fn extract_dashboard_url(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some((_, raw)) = line.split_once("Dashboard URL:") {
            let candidate = raw.trim();
            if candidate.starts_with("http://") || candidate.starts_with("https://") {
                return Some(candidate.to_string());
            }
        }
    }

    for token in output.split_whitespace() {
        let normalized = token
            .trim_matches(|c: char| matches!(c, '"' | '\'' | ',' | ';' | ')' | '('))
            .trim();
        if normalized.starts_with("http://") || normalized.starts_with("https://") {
            return Some(normalized.to_string());
        }
    }

    None
}

/// 执行 openclaw doctor --fix
#[tauri::command]
pub async fn run_doctor_fix(app: AppHandle) -> Result<String, String> {
    let step = "doctor_fix";
    emit_log(&app, step, "开始诊断并修复...", "info");

    let cmd = with_fnm_env("openclaw doctor --fix");

    match run_shell_with_log(&app, step, &cmd) {
        Ok(output) => {
            emit_log(&app, step, "诊断修复完成!", "success");
            Ok(output)
        }
        Err(e) => {
            emit_log(&app, step, &format!("诊断修复失败: {}", e), "error");
            Err(format!("诊断修复失败: {}", e))
        }
    }
}
