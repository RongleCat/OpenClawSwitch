use flate2::read::GzDecoder;
use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
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
            return Some(candidate);
        }
    }
    None
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

fn get_extension_meta(channel_id: &str) -> Result<(&'static str, &'static str), String> {
    match channel_id {
        "feishu" => Ok(("@m1heng-clawd/feishu", "feishu")),
        "dingtalk" => Ok(("@dingtalk-real-ai/dingtalk-connector", "dingtalk")),
        _ => Err(format!("不支持的渠道扩展: {}", channel_id)),
    }
}

fn is_channel_extension_installed(target_dir_name: &str) -> bool {
    let extensions_root = match get_extensions_root() {
        Ok(path) => path,
        Err(_) => return false,
    };

    let target_dir = extensions_root.join(target_dir_name);
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

/// 执行命令并返回 stdout（跨平台）
fn run_cmd(program: &str, args: &[&str]) -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.arg("/c").arg(program);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    } else {
        Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
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

/// 执行 shell 命令（用于复杂命令）
fn run_shell(cmd: &str) -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        // Windows: 优先使用 PowerShell（更好的 UTF-8 支持和环境变量处理）
        if cmd.starts_with("powershell") {
            // 已经是 PowerShell 命令，直接执行
            Command::new("cmd")
                .args(["/c", cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        } else {
            // 普通命令，用 cmd 执行
            Command::new("cmd")
                .args(["/c", cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        }
    } else {
        Command::new("sh")
            .args(["-c", cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
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

/// 执行命令并实时推送日志到前端
fn run_shell_with_log(app: &AppHandle, step: &str, cmd: &str) -> Result<String, String> {
    emit_log(app, step, &format!("$ {}", cmd), "info");

    let mut child = if cfg!(target_os = "windows") {
        if cmd.starts_with("powershell") {
            // PowerShell 命令
            Command::new("cmd")
                .args(["/c", cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        } else {
            Command::new("cmd")
                .args(["/c", cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        }
    } else {
        Command::new("sh")
            .args(["-c", cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
    .map_err(|e| format!("启动命令失败: {}", e))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // 并发读取 stdout 和 stderr 防止死锁
    let stderr_app = app.clone();
    let stderr_step = step.to_string();
    let stderr_handle = stderr.map(|err| {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                emit_log(&stderr_app, &stderr_step, &line, "warn");
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

    // 等待 stderr 线程完成
    if let Some(handle) = stderr_handle {
        let _ = handle.join();
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
        Command::new(candidate)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
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

    Command::new(fnm_path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn shell_quote(value: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

/// 构建包含 fnm 环境的 shell 命令
fn with_fnm_env(cmd: &str) -> String {
    if has_system_node() {
        return cmd.to_string();
    }

    if !can_execute_fnm_binary_directly() {
        return cmd.to_string();
    }

    if cfg!(target_os = "windows") {
        // Windows: 使用 PowerShell 执行 fnm 相关命令
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
fn configure_fnm_path(app: &AppHandle, step: &str) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        // Windows: 使用 PowerShell 安全地添加到用户 PATH
        emit_log(app, step, "配置 Windows PATH...", "info");
        let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let fnm_path = home.join(".fnm").to_string_lossy().to_string();
        let cmd = format!(
            "powershell -Command \"$p = [Environment]::GetEnvironmentVariable('Path','User'); if ($p -notlike '*\\.fnm*') {{ [Environment]::SetEnvironmentVariable('Path', \\\"$p;{}\\\", 'User') }}\"",
            fnm_path
        );
        let _ = run_shell(&cmd);
    } else {
        // Unix: 添加到 shell 配置文件
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

        // 检查是否已配置
        if let Ok(content) = std::fs::read_to_string(&rc_file) {
            if content.contains("FNM_DIR") {
                emit_log(app, step, "fnm PATH 已配置，跳过", "info");
                return Ok(());
            }
        }

        emit_log(
            app,
            step,
            &format!("添加 fnm 配置到 {}", rc_file.display()),
            "info",
        );

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&rc_file)
            .map_err(|e| format!("打开配置文件失败: {}", e))?;

        use std::io::Write;
        file.write_all(fnm_init.as_bytes())
            .map_err(|e| format!("写入配置失败: {}", e))?;
    }

    Ok(())
}

/// 通过 fnm 安装 Node.js
#[tauri::command]
pub async fn install_node_via_fnm(
    app: AppHandle,
    version: String,
    use_china_mirror: bool,
) -> Result<String, String> {
    let step = "install_node";
    emit_log(
        &app,
        step,
        &format!("开始安装 Node.js v{}...", version),
        "info",
    );

    // 选择镜像源
    let mirror = if use_china_mirror {
        NODE_MIRRORS[0]
    } else {
        ""
    };

    // 构建安装命令
    let cmd = if use_china_mirror {
        format!(
            "export FNM_NODE_DIST_MIRROR={} && {}",
            mirror,
            with_fnm_env(&format!("fnm install {}", version))
        )
    } else {
        with_fnm_env(&format!("fnm install {}", version))
    };

    // 执行安装
    match run_shell_with_log(&app, step, &cmd) {
        Ok(_) => {
            emit_log(&app, step, &format!("Node.js v{} 安装成功!", version), "success");

            // 设置默认版本
            let default_cmd = with_fnm_env(&format!("fnm default {}", version));
            let _ = run_shell_with_log(&app, step, &default_cmd);

            Ok(format!("Node.js v{} 安装成功", version))
        }
        Err(e) => {
            // 如果使用镜像失败，尝试其他镜像
            if use_china_mirror {
                for mirror in &NODE_MIRRORS[1..] {
                    emit_log(
                        &app,
                        step,
                        &format!("切换镜像源: {}...", mirror),
                        "warn",
                    );
                    let cmd = format!(
                        "export FNM_NODE_DIST_MIRROR={} && {}",
                        mirror,
                        with_fnm_env(&format!("fnm install {}", version))
                    );
                    if run_shell_with_log(&app, step, &cmd).is_ok() {
                        let default_cmd = with_fnm_env(&format!("fnm default {}", version));
                        let _ = run_shell_with_log(&app, step, &default_cmd);
                        emit_log(
                            &app,
                            step,
                            &format!("Node.js v{} 安装成功!", version),
                            "success",
                        );
                        return Ok(format!("Node.js v{} 安装成功", version));
                    }
                }
            }
            Err(format!("Node.js 安装失败: {}", e))
        }
    }
}

/// 安装 OpenClaw
#[tauri::command]
pub async fn install_openclaw(
    app: AppHandle,
    use_china_mirror: bool,
) -> Result<String, String> {
    let step = "install_openclaw";
    emit_log(&app, step, "开始安装 OpenClaw...", "info");

    let registries = if use_china_mirror {
        NPM_REGISTRIES.to_vec()
    } else {
        vec![NPM_REGISTRIES[2]] // 仅官方源
    };

    for registry in &registries {
        emit_log(
            &app,
            step,
            &format!("使用 registry: {}", registry),
            "info",
        );

        let cmd = with_fnm_env(&format!(
            "npm install -g openclaw@latest --registry={}",
            registry
        ));

        match run_shell_with_log(&app, step, &cmd) {
            Ok(_) => {
                emit_log(&app, step, "OpenClaw 安装成功!", "success");
                return Ok("OpenClaw 安装成功".to_string());
            }
            Err(e) => {
                emit_log(
                    &app,
                    step,
                    &format!("使用 {} 安装失败: {}", registry, e),
                    "warn",
                );
            }
        }
    }

    Err("OpenClaw 安装失败，所有镜像源均不可用".to_string())
}

/// 安装 Git（平台相关）
#[tauri::command]
pub async fn install_git(app: AppHandle) -> Result<String, String> {
    let step = "install_git";
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

    #[cfg(target_os = "windows")]
    {
        // Windows: 尝试 winget
        emit_log(&app, step, "Windows: 尝试通过 winget 安装 Git...", "info");
        match run_shell_with_log(&app, step, "winget install Git.Git --silent --accept-package-agreements --accept-source-agreements") {
            Ok(_) => {
                emit_log(&app, step, "Git 安装成功!", "success");
                return Ok("Git 安装成功".to_string());
            }
            Err(_) => {
                emit_log(&app, step, "winget 安装失败，请手动下载安装", "warn");
                return Err("Git 安装失败，请手动下载: https://git-scm.com/download/win".to_string());
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

    // 步骤 1: 环境检测
    emit_progress(&app, 1, total_steps, "环境检测", "running");
    emit_log(&app, "check", "开始环境检测...", "info");

    let env = check_environment().await;
    emit_log(
        &app,
        "check",
        &format!("系统: {} {}", env.system.os, env.system.arch),
        "info",
    );
    emit_log(
        &app,
        "check",
        &format!("网络环境: {}", if env.network_region == "china" { "中国大陆" } else { "国际" }),
        "info",
    );
    emit_log(
        &app,
        "check",
        &format!("Git: {}", if env.git.installed { format!("✓ {}", env.git.version.as_deref().unwrap_or("")) } else { "✗ 未安装".to_string() }),
        if env.git.installed { "success" } else { "warn" },
    );
    emit_log(
        &app,
        "check",
        &format!("fnm: {}", if env.fnm.installed { format!("✓ {}", env.fnm.version.as_deref().unwrap_or("")) } else { "✗ 未安装".to_string() }),
        if env.fnm.installed { "success" } else { "warn" },
    );
    emit_log(
        &app,
        "check",
        &format!("Node.js: {}", if env.node.installed { format!("✓ v{}", env.node.version.as_deref().unwrap_or("")) } else { "✗ 未安装".to_string() }),
        if env.node.meets_requirement { "success" } else { "warn" },
    );
    emit_log(
        &app,
        "check",
        &format!("OpenClaw: {}", if env.openclaw.installed { format!("✓ {}", env.openclaw.version.as_deref().unwrap_or("")) } else { "✗ 未安装".to_string() }),
        if env.openclaw.installed { "success" } else { "warn" },
    );
    emit_progress(&app, 1, total_steps, "环境检测", "success");

    let use_china = env.network_region == "china";

    // 步骤 2: 安装 Git（如需）
    emit_progress(&app, 2, total_steps, "安装 Git", "running");
    if !env.git.installed {
        match install_git(app.clone()).await {
            Ok(msg) => {
                emit_log(&app, "install_git", &msg, "success");
                emit_progress(&app, 2, total_steps, "安装 Git", "success");
            }
            Err(e) => {
                emit_log(&app, "install_git", &format!("Git 安装失败: {}", e), "error");
                emit_log(&app, "install_git", "Git 不是必须的，继续安装...", "warn");
                emit_progress(&app, 2, total_steps, "安装 Git", "success");
            }
        }
    } else {
        emit_log(&app, "install_git", "Git 已安装，跳过", "success");
        emit_progress(&app, 2, total_steps, "安装 Git", "success");
    }

    // 步骤 3: 安装 Node.js（包含 fnm）
    emit_progress(&app, 3, total_steps, "安装 Node.js", "running");

    // 如果 Node.js >= 22 已安装，完全跳过此步骤
    if env.node.meets_requirement {
        emit_log(&app, "install_node", "Node.js >= 22 已安装，跳过 fnm 和 Node.js 安装", "success");
        emit_progress(&app, 3, total_steps, "安装 Node.js", "success");
    } else {
        // 先安装 fnm（如需）
        if !env.fnm.installed {
            install_fnm(app.clone(), use_china).await.map_err(|e| {
                emit_progress(&app, 3, total_steps, "安装 Node.js", "error");
                e
            })?;
        } else {
            emit_log(&app, "install_fnm", "fnm 已安装，跳过", "success");
        }

        // 再安装 Node.js
        install_node_via_fnm(app.clone(), "22".to_string(), use_china)
            .await
            .map_err(|e| {
                emit_progress(&app, 3, total_steps, "安装 Node.js", "error");
                e
            })?;
        emit_progress(&app, 3, total_steps, "安装 Node.js", "success");
    }

    // 步骤 4: 安装 OpenClaw
    emit_progress(&app, 4, total_steps, "安装 OpenClaw", "running");
    if !env.openclaw.installed {
        install_openclaw(app.clone(), use_china).await.map_err(|e| {
            emit_progress(&app, 4, total_steps, "安装 OpenClaw", "error");
            e
        })?;
    } else {
        emit_log(&app, "install_openclaw", "OpenClaw 已安装，跳过", "success");
    }
    emit_progress(&app, 4, total_steps, "安装 OpenClaw", "success");

    // 步骤 5: 验证安装
    emit_progress(&app, 5, total_steps, "验证安装", "running");
    emit_log(&app, "verify", "验证安装结果...", "info");

    let final_status = check_openclaw_installed();
    if final_status.installed {
        emit_log(
            &app,
            "verify",
            &format!(
                "OpenClaw {} 安装成功!",
                final_status.version.as_deref().unwrap_or("")
            ),
            "success",
        );
        emit_progress(&app, 5, total_steps, "验证安装", "success");
        Ok("安装完成".to_string())
    } else {
        emit_log(&app, "verify", "验证失败: openclaw 命令不可用", "error");
        emit_log(
            &app,
            "verify",
            "请尝试重启终端后再次检测，或手动执行: openclaw --version",
            "warn",
        );
        emit_progress(&app, 5, total_steps, "验证安装", "error");
        Err("安装验证失败，请重启终端后重试".to_string())
    }
}

// ============================================================================
// Tauri 命令 - 安装后配置
// ============================================================================

/// 打开终端并执行命令（跨平台）
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
    let default_config = serde_json::json!({
        "gateway": {
            "mode": "local",
            "port": 18789,
            "bind": "loopback",
            "auth": {
                "mode": "token",
                "token": token
            }
        },
        "agents": {
            "defaults": {
                "workspace": "~/.openclaw/workspace"
            }
        }
    });

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

/// 安装网关服务（后台自动启动）
#[tauri::command]
pub async fn install_gateway_service(app: AppHandle) -> Result<String, String> {
    let step = "install_service";
    emit_log(&app, step, "安装网关服务...", "info");

    // 执行 openclaw gateway install
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

/// 启动本地网关服务
#[tauri::command]
pub async fn start_gateway() -> Result<String, String> {
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
pub async fn stop_gateway() -> Result<String, String> {
    let cmd = with_fnm_env("openclaw gateway stop");
    let output = run_shell(&cmd)?;
    if output.is_empty() {
        Ok("网关停止命令已执行".to_string())
    } else {
        Ok(output)
    }
}

/// 获取消息渠道扩展安装状态
#[tauri::command]
pub async fn get_channel_extension_status() -> Result<ChannelExtensionStatus, String> {
    Ok(ChannelExtensionStatus {
        feishu_installed: is_channel_extension_installed("feishu"),
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
        let (npm_package, target_dir_name) = get_extension_meta(&channel_id)?;
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

        let pack_output = Command::new(npm)
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

        let install_output = Command::new(npm)
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

        if !is_channel_extension_installed(target_dir_name) {
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
            Command::new("cmd")
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
            Command::new("cmd")
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
