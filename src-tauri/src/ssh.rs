// SSH 远程连接核心模块
// 使用 ssh2 crate 实现 SSH 连接、认证，通过 channel 命令执行实现文件操作

use serde::{Deserialize, Serialize};
use ssh2::Session;
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;

// ============================================================================
// Known Hosts 管理
// ============================================================================

fn get_known_hosts_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openclaw")
        .join("known_hosts")
}

fn load_known_hosts() -> HashSet<String> {
    let path = get_known_hosts_path();
    if !path.exists() {
        return HashSet::new();
    }

    fs::read_to_string(&path)
        .ok()
        .map(|content| {
            content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn save_known_host(fingerprint: &str) -> Result<(), String> {
    let path = get_known_hosts_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let mut hosts = load_known_hosts();
    hosts.insert(fingerprint.to_string());

    let content = hosts.into_iter().collect::<Vec<_>>().join("\n");
    fs::write(&path, content).map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(())
}

fn is_host_known(fingerprint: &str) -> bool {
    load_known_hosts().contains(fingerprint)
}

// ============================================================================
// 类型定义
// ============================================================================

/// SSH 认证方式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SshAuthMode {
    Password,
    PrivateKey,
}

/// SSH 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_mode: SshAuthMode,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

/// 指纹信息
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FingerprintInfo {
    pub sha256: String,
    pub md5: String,
    pub host: String,
    pub is_known: bool,
}

/// 远程文件条目
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

/// 配置文件搜索结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSearchResult {
    pub path: String,
    pub file_name: String,
    pub dir_path: String,
}

// ============================================================================
// SSH 连接管理器
// ============================================================================

/// SSH 连接状态，持有 Session 和连接信息
struct SshConnection {
    session: Session,
    #[allow(dead_code)]
    host: String,
    username: String,
}

/// SSH 管理器，线程安全的连接状态容器
pub struct SshManager {
    connection: Mutex<Option<SshConnection>>,
}

impl SshManager {
    pub fn new() -> Self {
        SshManager {
            connection: Mutex::new(None),
        }
    }
}

/// 将指纹字节数组格式化为十六进制字符串
fn format_fingerprint_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(":")
}

/// 将指纹字节数组格式化为 Base64（SHA-256 常用格式）
fn format_fingerprint_base64(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = Vec::new();
    for chunk in bytes.chunks(3) {
        match chunk.len() {
            3 => {
                result.push(CHARS[(chunk[0] >> 2) as usize]);
                result.push(CHARS[(((chunk[0] & 0x03) << 4) | (chunk[1] >> 4)) as usize]);
                result.push(CHARS[(((chunk[1] & 0x0f) << 2) | (chunk[2] >> 6)) as usize]);
                result.push(CHARS[(chunk[2] & 0x3f) as usize]);
            }
            2 => {
                result.push(CHARS[(chunk[0] >> 2) as usize]);
                result.push(CHARS[(((chunk[0] & 0x03) << 4) | (chunk[1] >> 4)) as usize]);
                result.push(CHARS[((chunk[1] & 0x0f) << 2) as usize]);
                result.push(b'=');
            }
            1 => {
                result.push(CHARS[(chunk[0] >> 2) as usize]);
                result.push(CHARS[((chunk[0] & 0x03) << 4) as usize]);
                result.push(b'=');
                result.push(b'=');
            }
            _ => {}
        }
    }
    String::from_utf8(result).unwrap_or_default()
}

// ============================================================================
// Tauri 命令
// ============================================================================

/// 建立 SSH 连接并返回指纹信息（尚未认证）
#[tauri::command]
pub fn ssh_connect(
    manager: State<SshManager>,
    host: String,
    port: u16,
    username: String,
) -> Result<FingerprintInfo, String> {
    // 先断开已有连接
    {
        let mut conn = manager.connection.lock().map_err(|e| format!("锁错误: {}", e))?;
        *conn = None;
    }

    let addr = format!("{}:{}", host, port);
    let tcp = TcpStream::connect(&addr).map_err(|e| format!("连接失败: {}", e))?;
    tcp.set_nodelay(true).ok();

    let mut session = Session::new().map_err(|e| format!("创建会话失败: {}", e))?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|e| format!("握手失败: {}", e))?;

    // 握手完成后再设置超时和 keepalive，避免干扰握手和 SFTP 初始化
    session.set_timeout(0); // 不设置 session 超时，由各操作自行控制
    session.set_keepalive(true, 30); // 每 30 秒发送 keepalive 防止断连

    // 获取主机指纹
    let md5 = format_fingerprint_hex(
        &session
            .host_key_hash(ssh2::HashType::Md5)
            .ok_or("无法获取 MD5 指纹")?,
    );

    let sha256_bytes = session
        .host_key_hash(ssh2::HashType::Sha256)
        .ok_or("无法获取 SHA-256 指纹")?;
    let sha256 = format!("SHA256:{}", format_fingerprint_base64(sha256_bytes));

    // 检查 known_hosts
    let is_known = is_host_known(&sha256);

    let fingerprint = FingerprintInfo {
        sha256,
        md5,
        host: host.clone(),
        is_known,
    };

    // 保存会话（尚未认证）
    let mut conn = manager.connection.lock().map_err(|e| format!("锁错误: {}", e))?;
    *conn = Some(SshConnection {
        session,
        host,
        username,
    });

    Ok(fingerprint)
}

/// 保存主机指纹到 known_hosts
#[tauri::command]
pub fn ssh_save_fingerprint(fingerprint: String) -> Result<(), String> {
    save_known_host(&fingerprint)
}

/// 使用密码认证
#[tauri::command]
pub fn ssh_auth_password(
    manager: State<SshManager>,
    password: String,
) -> Result<(), String> {
    let conn = manager.connection.lock().map_err(|e| format!("锁错误: {}", e))?;
    let conn = conn.as_ref().ok_or("未建立连接")?;

    conn.session
        .userauth_password(&conn.username, &password)
        .map_err(|e| format!("密码认证失败: {}", e))?;

    if !conn.session.authenticated() {
        return Err("认证失败：用户名或密码错误".to_string());
    }

    Ok(())
}

/// 使用私钥认证
#[tauri::command]
pub fn ssh_auth_key(
    manager: State<SshManager>,
    key_path: String,
    passphrase: Option<String>,
) -> Result<(), String> {
    let conn = manager.connection.lock().map_err(|e| format!("锁错误: {}", e))?;
    let conn = conn.as_ref().ok_or("未建立连接")?;

    let key = Path::new(&key_path);
    if !key.exists() {
        return Err(format!("私钥文件不存在: {}", key_path));
    }

    conn.session
        .userauth_pubkey_file(
            &conn.username,
            None,
            key,
            passphrase.as_deref(),
        )
        .map_err(|e| format!("私钥认证失败: {}", e))?;

    if !conn.session.authenticated() {
        return Err("认证失败：私钥无效".to_string());
    }

    Ok(())
}

/// 断开 SSH 连接
#[tauri::command]
pub fn ssh_disconnect(manager: State<SshManager>) -> Result<(), String> {
    let mut conn = manager.connection.lock().map_err(|e| format!("锁错误: {}", e))?;
    if let Some(c) = conn.as_ref() {
        let _ = c.session.disconnect(None, "用户断开连接", None);
    }
    *conn = None;
    Ok(())
}

/// 获取连接状态
#[tauri::command]
pub fn ssh_get_status(manager: State<SshManager>) -> Result<bool, String> {
    let conn = manager.connection.lock().map_err(|e| format!("锁错误: {}", e))?;
    Ok(conn
        .as_ref()
        .map(|c| c.session.authenticated())
        .unwrap_or(false))
}

/// 列出远程目录（通过 ls 命令）
#[tauri::command]
pub fn ssh_list_dir(
    manager: State<SshManager>,
    path: String,
) -> Result<Vec<RemoteFileEntry>, String> {
    let conn = manager.connection.lock().map_err(|e| format!("锁错误: {}", e))?;
    let conn = conn.as_ref().ok_or("未建立连接")?;

    if !conn.session.authenticated() {
        return Err("未认证".to_string());
    }

    // 使用 ls -la 获取目录列表，--time-style 确保输出格式一致
    let cmd = format!(
        "ls -la --time-style=+%s '{}' 2>/dev/null || ls -la '{}' 2>/dev/null",
        path, path
    );
    let output = exec_remote_command(&conn.session, &cmd)?;

    let mut entries = Vec::new();
    for line in output.lines().skip(1) {
        // 跳过 "total" 行
        let line = line.trim();
        if line.is_empty() || line.starts_with("total") {
            continue;
        }
        if let Some(entry) = parse_ls_line(line, &path) {
            if entry.name != "." && entry.name != ".." {
                entries.push(entry);
            }
        }
    }

    // 目录在前，文件在后，各自按名称排序
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(entries)
}

/// 解析 ls -la 输出的单行
fn parse_ls_line(line: &str, parent_path: &str) -> Option<RemoteFileEntry> {
    // ls -la 格式: drwxr-xr-x 2 root root 4096 1234567890 dirname
    // 或: -rw-r--r-- 1 root root 1234 Jan 01 12:00 filename
    let parts: Vec<&str> = line.splitn(9, char::is_whitespace)
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() < 7 {
        return None;
    }

    let is_dir = parts[0].starts_with('d');
    let size: u64 = parts[4].parse().unwrap_or(0);

    // 文件名是最后一个字段（可能包含空格）
    // 对于 --time-style=+%s 格式，字段数为 7+
    // 对于标准格式，字段数为 8+
    let name = if parts.len() >= 9 {
        parts[8..].join(" ")
    } else if parts.len() >= 8 {
        parts[7..].join(" ")
    } else {
        parts[parts.len() - 1].to_string()
    };

    // 跳过符号链接的 -> 部分
    let name = name.split(" -> ").next().unwrap_or(&name).to_string();

    if name.is_empty() {
        return None;
    }

    Some(RemoteFileEntry {
        path: format!("{}/{}", parent_path.trim_end_matches('/'), name),
        name,
        is_dir,
        size,
    })
}

/// 读取远程文件内容（通过 cat 命令）
#[tauri::command]
pub fn ssh_read_file(
    manager: State<SshManager>,
    path: String,
) -> Result<String, String> {
    let conn = manager.connection.lock().map_err(|e| format!("锁错误: {}", e))?;
    let conn = conn.as_ref().ok_or("未建立连接")?;

    if !conn.session.authenticated() {
        return Err("未认证".to_string());
    }

    let cmd = format!("cat '{}'", path.replace('\'', "'\\''"));
    exec_remote_command(&conn.session, &cmd)
}

/// 写入远程文件（通过 channel stdin）
#[tauri::command]
pub fn ssh_write_file(
    manager: State<SshManager>,
    path: String,
    content: String,
) -> Result<(), String> {
    let conn = manager.connection.lock().map_err(|e| format!("锁错误: {}", e))?;
    let conn = conn.as_ref().ok_or("未建立连接")?;

    if !conn.session.authenticated() {
        return Err("未认证".to_string());
    }

    let escaped_path = path.replace('\'', "'\\''");
    let cmd = format!("cat > '{}'", escaped_path);

    let mut channel = conn.session
        .channel_session()
        .map_err(|e| format!("创建通道失败: {}", e))?;
    channel
        .exec(&cmd)
        .map_err(|e| format!("执行命令失败: {}", e))?;

    use std::io::Write;
    channel
        .write_all(content.as_bytes())
        .map_err(|e| format!("写入失败: {}", e))?;

    // 关闭 stdin 通知远程 cat 命令结束
    channel.send_eof().map_err(|e| format!("发送 EOF 失败: {}", e))?;
    channel.wait_eof().ok();
    channel.wait_close().ok();

    let exit = channel.exit_status().unwrap_or(-1);
    if exit != 0 {
        return Err(format!("写入失败，退出码: {}", exit));
    }

    Ok(())
}

/// 自动搜寻远程服务器上的 OpenClaw 配置文件（通过 test -f 命令）
#[tauri::command]
pub fn ssh_search_config(
    manager: State<SshManager>,
) -> Result<Vec<ConfigSearchResult>, String> {
    let conn = manager.connection.lock().map_err(|e| format!("锁错误: {}", e))?;
    let conn = conn.as_ref().ok_or("未建立连接")?;

    if !conn.session.authenticated() {
        return Err("未认证".to_string());
    }

    // 获取远程 home 目录
    let home = get_remote_home(&conn.session).unwrap_or_else(|| "/root".to_string());

    let search_dirs = [
        format!("{}/.openclaw", home),
        home.clone(),
        format!("{}/.config/openclaw", home),
        "/etc/openclaw".to_string(),
        "/opt/openclaw".to_string(),
    ];

    let config_names = [
        "openclaw.json",
        "clawdbot.json",
        "openclaw.yaml",
        "openclaw.yml",
    ];

    // 构建批量检查命令
    let mut checks = Vec::new();
    for dir in &search_dirs {
        for name in &config_names {
            let p = format!("{}/{}", dir.trim_end_matches('/'), name);
            checks.push(format!("test -f '{}' && echo '{}'", p, p));
        }
    }
    let cmd = checks.join("; ");

    let mut results = Vec::new();
    if let Ok(output) = exec_remote_command(&conn.session, &cmd) {
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || !line.starts_with('/') {
                continue;
            }
            let path = Path::new(line);
            if let Some(file_name) = path.file_name() {
                let dir_path = path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                results.push(ConfigSearchResult {
                    path: line.to_string(),
                    file_name: file_name.to_string_lossy().to_string(),
                    dir_path,
                });
            }
        }
    }

    Ok(results)
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 通过执行远程命令获取用户 home 目录
fn get_remote_home(session: &Session) -> Option<String> {
    let output = exec_remote_command(session, "echo $HOME").ok()?;
    let home = output.trim().to_string();
    if home.is_empty() { None } else { Some(home) }
}

/// 执行远程命令并返回清理后的输出
fn exec_remote_command(session: &Session, cmd: &str) -> Result<String, String> {
    let mut channel = session
        .channel_session()
        .map_err(|e| format!("创建通道失败: {}", e))?;
    channel
        .exec(cmd)
        .map_err(|e| format!("执行命令失败: {}", e))?;

    let mut output = String::new();
    channel
        .read_to_string(&mut output)
        .map_err(|e| format!("读取输出失败: {}", e))?;

    // 排空 stderr
    let mut stderr_buf = String::new();
    let _ = channel.stderr().read_to_string(&mut stderr_buf);

    let _ = channel.send_eof();
    let _ = channel.wait_eof();
    let _ = channel.wait_close();

    Ok(strip_ansi_escapes(&output))
}

/// 清理 ANSI/OSC/CSI 终端转义序列
fn strip_ansi_escapes(input: &str) -> String {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut result = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        if bytes[i] == 0x1b {
            if i + 1 < len {
                match bytes[i + 1] {
                    b'[' => {
                        // CSI: ESC [ ... (终止于 @-~)
                        i += 2;
                        while i < len && !(bytes[i] >= b'@' && bytes[i] <= b'~') {
                            i += 1;
                        }
                        if i < len { i += 1; }
                    }
                    b']' => {
                        // OSC: ESC ] ... (终止于 BEL 或 ESC \)
                        i += 2;
                        while i < len {
                            if bytes[i] == 0x07 { i += 1; break; }
                            if bytes[i] == 0x1b && i + 1 < len && bytes[i + 1] == b'\\' {
                                i += 2; break;
                            }
                            i += 1;
                        }
                    }
                    _ => { i += 2; }
                }
            } else {
                i += 1;
            }
        } else if bytes[i] == 0x07 || bytes[i] == 0x0e || bytes[i] == 0x0f {
            i += 1;
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }

    String::from_utf8(result).unwrap_or_else(|_| input.to_string())
}

// ============================================================================
// SSH 远程环境检测
// ============================================================================

/// 通过 SSH 检测远程服务器的 OpenClaw 环境状态
#[tauri::command]
pub fn ssh_check_environment(
    manager: State<SshManager>,
) -> Result<crate::installer::EnvironmentStatus, String> {
    let conn = manager
        .connection
        .lock()
        .map_err(|e| format!("锁错误: {}", e))?;
    let conn = conn.as_ref().ok_or("未建立 SSH 连接")?;
    if !conn.session.authenticated() {
        return Err("SSH 未认证".to_string());
    }

    // 一次性执行多条检测命令，减少 round-trip
    let detect_script = r#"
echo "===OPENCLAW_VERSION==="
openclaw --version 2>/dev/null || echo "__NOT_INSTALLED__"
echo "===OPENCLAW_PATH==="
which openclaw 2>/dev/null || echo "__NOT_FOUND__"
echo "===NODE_VERSION==="
node --version 2>/dev/null || echo "__NOT_INSTALLED__"
echo "===GIT_VERSION==="
git --version 2>/dev/null || echo "__NOT_INSTALLED__"
echo "===FNM_VERSION==="
fnm --version 2>/dev/null || echo "__NOT_INSTALLED__"
echo "===SYSTEM_INFO==="
uname -s && uname -m && basename "$SHELL" 2>/dev/null || echo "unknown"
echo "===END==="
"#;

    let output = exec_remote_command(&conn.session, detect_script)?;

    // 解析输出
    let get_section = |key: &str| -> String {
        let start_marker = format!("==={}===", key);
        let lines: Vec<&str> = output.lines().collect();
        let mut capture = false;
        let mut result = Vec::new();
        for line in &lines {
            if line.contains(&start_marker) {
                capture = true;
                continue;
            }
            if capture && line.contains("===") {
                break;
            }
            if capture {
                result.push(line.trim());
            }
        }
        result.join("\n").trim().to_string()
    };

    // OpenClaw
    let oc_version_raw = get_section("OPENCLAW_VERSION");
    let oc_path_raw = get_section("OPENCLAW_PATH");
    let oc_installed = !oc_version_raw.contains("__NOT_INSTALLED__");
    let openclaw = crate::installer::OpenClawStatus {
        installed: oc_installed,
        version: if oc_installed {
            Some(oc_version_raw)
        } else {
            None
        },
        path: if oc_path_raw.contains("__NOT_FOUND__") {
            None
        } else {
            Some(oc_path_raw)
        },
    };

    // Node.js
    let node_raw = get_section("NODE_VERSION");
    let node_installed = !node_raw.contains("__NOT_INSTALLED__");
    let node_version = if node_installed {
        Some(node_raw.trim_start_matches('v').to_string())
    } else {
        None
    };
    let node_major: u32 = node_version
        .as_deref()
        .and_then(|v| v.split('.').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let node = crate::installer::NodeStatus {
        installed: node_installed,
        version: node_version,
        meets_requirement: node_major >= 22,
    };

    // Git
    let git_raw = get_section("GIT_VERSION");
    let git_installed = !git_raw.contains("__NOT_INSTALLED__");
    let git = crate::installer::GitStatus {
        installed: git_installed,
        version: if git_installed {
            Some(git_raw.replace("git version ", "").trim().to_string())
        } else {
            None
        },
    };

    // fnm
    let fnm_raw = get_section("FNM_VERSION");
    let fnm_installed = !fnm_raw.contains("__NOT_INSTALLED__");
    let fnm = crate::installer::FnmStatus {
        installed: fnm_installed,
        version: if fnm_installed {
            Some(fnm_raw.replace("fnm ", "").trim().to_string())
        } else {
            None
        },
    };

    // 系统信息
    let sys_raw = get_section("SYSTEM_INFO");
    let sys_lines: Vec<&str> = sys_raw.lines().collect();
    let os_name = sys_lines.first().unwrap_or(&"linux").to_lowercase();
    let os = if os_name.contains("darwin") {
        "macos"
    } else if os_name.contains("windows") || os_name.contains("mingw") {
        "windows"
    } else {
        "linux"
    }
    .to_string();
    let arch_raw = sys_lines.get(1).unwrap_or(&"x86_64").to_lowercase();
    let arch = if arch_raw.contains("aarch64") || arch_raw.contains("arm64") {
        "aarch64"
    } else {
        "x86_64"
    }
    .to_string();
    let shell = sys_lines
        .get(2)
        .unwrap_or(&"sh")
        .to_string();

    let system = crate::installer::SystemInfo { os, arch, shell };

    Ok(crate::installer::EnvironmentStatus {
        openclaw,
        node,
        git,
        fnm,
        system,
        network_region: "unknown".to_string(),
    })
}
