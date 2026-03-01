use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
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

const FNM_GITHUB_RELEASE: &str = "https://github.com/Schniz/fnm/releases/latest/download";
const FNM_MIRROR_PREFIX: &str = "https://ghproxy.com/";

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
        Command::new("cmd")
            .args(["/c", cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
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
        Command::new("cmd")
            .args(["/c", cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
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

/// 构建包含 fnm 环境的 shell 命令
fn with_fnm_env(cmd: &str) -> String {
    if cfg!(target_os = "windows") {
        // Windows: fnm env --use-on-cd 输出 PowerShell 格式，用 cmd 需要特殊处理
        format!(
            "set \"FNM_DIR=%USERPROFILE%\\.fnm\" && set \"PATH=%USERPROFILE%\\.fnm;%PATH%\" && {}",
            cmd
        )
    } else {
        format!(
            "export FNM_DIR=\"$HOME/.fnm\" && export PATH=\"$HOME/.fnm:$PATH\" && eval \"$(fnm env)\" && {}",
            cmd
        )
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

/// 获取 fnm 下载 URL
fn get_fnm_download_url(use_mirror: bool) -> String {
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

    let base_url = format!("{}/{}.zip", FNM_GITHUB_RELEASE, platform);
    if use_mirror {
        format!("{}{}", FNM_MIRROR_PREFIX, base_url)
    } else {
        base_url
    }
}

/// 安装 fnm
#[tauri::command]
pub async fn install_fnm(app: AppHandle, use_mirror: bool) -> Result<String, String> {
    let step = "install_fnm";
    emit_log(&app, step, "开始安装 fnm (Fast Node Manager)...", "info");

    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let fnm_dir = home.join(".fnm");

    // 创建 fnm 目录
    std::fs::create_dir_all(&fnm_dir)
        .map_err(|e| format!("创建 fnm 目录失败: {}", e))?;

    // 下载 fnm
    let url = get_fnm_download_url(use_mirror);
    emit_log(&app, step, &format!("下载 fnm: {}", url), "info");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            let msg = format!("下载 fnm 失败: {}", e);
            emit_log(&app, step, &msg, "error");
            msg
        })?;

    if !response.status().is_success() {
        // 如果镜像失败，尝试直连
        if use_mirror {
            emit_log(&app, step, "镜像下载失败，尝试直连 GitHub...", "warn");
            let direct_url = get_fnm_download_url(false);
            let response = client
                .get(&direct_url)
                .send()
                .await
                .map_err(|e| format!("直连下载也失败: {}", e))?;

            if !response.status().is_success() {
                return Err("fnm 下载失败，请检查网络连接".to_string());
            }

            let bytes = response.bytes().await.map_err(|e| format!("读取数据失败: {}", e))?;
            extract_fnm_zip(&app, step, &bytes, &fnm_dir)?;
        } else {
            return Err(format!("下载失败: HTTP {}", response.status()));
        }
    } else {
        let bytes = response.bytes().await.map_err(|e| format!("读取数据失败: {}", e))?;
        extract_fnm_zip(&app, step, &bytes, &fnm_dir)?;
    }

    // 配置 PATH（添加到 shell 配置文件）
    configure_fnm_path(&app, step)?;

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
            "npm install -g openclaw --registry={}",
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
    let total_steps: u8 = 6;

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

    // 步骤 3: 安装 fnm（如需）
    emit_progress(&app, 3, total_steps, "安装 fnm", "running");
    if !env.fnm.installed {
        install_fnm(app.clone(), use_china).await.map_err(|e| {
            emit_progress(&app, 3, total_steps, "安装 fnm", "error");
            e
        })?;
    } else {
        emit_log(&app, "install_fnm", "fnm 已安装，跳过", "success");
    }
    emit_progress(&app, 3, total_steps, "安装 fnm", "success");

    // 步骤 4: 安装 Node.js（如需）
    emit_progress(&app, 4, total_steps, "安装 Node.js", "running");
    if !env.node.meets_requirement {
        install_node_via_fnm(app.clone(), "22".to_string(), use_china)
            .await
            .map_err(|e| {
                emit_progress(&app, 4, total_steps, "安装 Node.js", "error");
                e
            })?;
    } else {
        emit_log(&app, "install_node", "Node.js >= 22 已安装，跳过", "success");
    }
    emit_progress(&app, 4, total_steps, "安装 Node.js", "success");

    // 步骤 5: 安装 OpenClaw
    emit_progress(&app, 5, total_steps, "安装 OpenClaw", "running");
    if !env.openclaw.installed {
        install_openclaw(app.clone(), use_china).await.map_err(|e| {
            emit_progress(&app, 5, total_steps, "安装 OpenClaw", "error");
            e
        })?;
    } else {
        emit_log(&app, "install_openclaw", "OpenClaw 已安装，跳过", "success");
    }
    emit_progress(&app, 5, total_steps, "安装 OpenClaw", "success");

    // 步骤 6: 验证安装
    emit_progress(&app, 6, total_steps, "验证安装", "running");
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
        emit_progress(&app, 6, total_steps, "验证安装", "success");
        Ok("安装完成".to_string())
    } else {
        emit_log(&app, "verify", "验证失败: openclaw 命令不可用", "error");
        emit_log(
            &app,
            "verify",
            "请尝试重启终端后再次检测，或手动执行: openclaw --version",
            "warn",
        );
        emit_progress(&app, 6, total_steps, "验证安装", "error");
        Err("安装验证失败，请重启终端后重试".to_string())
    }
}
