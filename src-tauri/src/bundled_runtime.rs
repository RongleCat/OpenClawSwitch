use serde::Serialize;
use serde_json::json;
use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

const DEFAULT_WEB_UI_URL: &str = "http://127.0.0.1:18789";
const DEFAULT_WEB_UI_SOCKET_ADDR: &str = "127.0.0.1:18789";

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHealth {
    pub runtime_not_ready: bool,
    pub node_ready: bool,
    pub openclaw_ready: bool,
    pub config_ready: bool,
    pub local_openclaw_available: bool,
    pub should_skip_setup: bool,
    pub config_path: String,
    pub data_dir: String,
    pub node_path: String,
    pub openclaw_root: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GatewayStatusSnapshot {
    pub state: String,
    pub pid: Option<u32>,
    pub url: Option<String>,
    pub message: Option<String>,
}

pub fn default_web_ui_url() -> &'static str {
    DEFAULT_WEB_UI_URL
}

pub fn is_default_web_ui_reachable(timeout: Duration) -> bool {
    let mut addrs = match DEFAULT_WEB_UI_SOCKET_ADDR.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(_) => return false,
    };
    let Some(addr) = addrs.next() else {
        return false;
    };

    TcpStream::connect_timeout(&addr, timeout).is_ok()
}

fn openclaw_data_dir_from_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".openclaw")
}

pub fn openclaw_data_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Unable to resolve the home directory.".to_string())?;
    Ok(openclaw_data_dir_from_home(&home_dir))
}

fn ensure_openclaw_data_dir_with(home_dir: &Path) -> Result<PathBuf, String> {
    let data_dir = openclaw_data_dir_from_home(home_dir);
    fs::create_dir_all(data_dir.join("logs"))
        .map_err(|error| format!("Failed to create the logs directory: {}", error))?;
    fs::create_dir_all(data_dir.join("extensions"))
        .map_err(|error| format!("Failed to create the extensions directory: {}", error))?;
    Ok(data_dir)
}

pub fn ensure_openclaw_data_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Unable to resolve the home directory.".to_string())?;
    ensure_openclaw_data_dir_with(&home_dir)
}

fn openclaw_config_path_for_home(home_dir: &Path) -> PathBuf {
    openclaw_data_dir_from_home(home_dir).join("openclaw.json")
}

#[derive(Debug, Clone)]
struct RuntimeConfigProbe {
    config_path: PathBuf,
    config_ready: bool,
    should_skip_setup: bool,
}

fn detect_system_openclaw_bin_path() -> Option<PathBuf> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "where openclaw"])
            .output()
            .ok()?
    } else {
        Command::new("sh")
            .args(["-lc", "command -v openclaw || which openclaw"])
            .output()
            .ok()?
    };

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let candidate = PathBuf::from(trimmed);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn probe_runtime_config_with(
    home_dir: &Path,
    local_openclaw_available: bool,
) -> RuntimeConfigProbe {
    let config_path = openclaw_config_path_for_home(home_dir);
    let config_ready = config_path.is_file();

    RuntimeConfigProbe {
        config_path,
        config_ready,
        should_skip_setup: local_openclaw_available && config_ready,
    }
}

fn dev_resource_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("vendor")
}

fn dev_compiled_resource_roots() -> Vec<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    vec![
        manifest_dir
            .join("target")
            .join("debug")
            .join("resources")
            .join("vendor"),
        manifest_dir
            .join("target")
            .join("release")
            .join("resources")
            .join("vendor"),
        dev_resource_root(),
    ]
}

fn select_resource_dir_with_fallbacks(
    resource_dir: Option<PathBuf>,
    fallback_roots: &[PathBuf],
) -> PathBuf {
    match resource_dir {
        Some(path) => {
            let vendor_dir = path.join("vendor");
            if vendor_dir.exists() {
                vendor_dir
            } else {
                fallback_roots
                    .iter()
                    .find(|path| path.exists())
                    .cloned()
                    .unwrap_or_else(dev_resource_root)
            }
        }
        None => fallback_roots
            .iter()
            .find(|path| path.exists())
            .cloned()
            .unwrap_or_else(dev_resource_root),
    }
}

fn select_resource_dir(resource_dir: Option<PathBuf>) -> PathBuf {
    select_resource_dir_with_fallbacks(resource_dir, &dev_compiled_resource_roots())
}

fn resolve_resource_dir(app: &AppHandle) -> PathBuf {
    select_resource_dir(app.path().resource_dir().ok())
}

fn platform_target_id() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "win32-x64"
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        "win32-arm64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "darwin-x64"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "darwin-arm64"
    }
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64")
    )))]
    {
        "unsupported"
    }
}

pub fn resolve_bundled_openclaw_root(app: &AppHandle) -> Result<PathBuf, String> {
    let root = resolve_resource_dir(app).join("openclaw");
    if root.exists() {
        return Ok(root);
    }
    Err(format!(
        "Bundled OpenClaw directory was not found: {}",
        root.display()
    ))
}

pub fn resolve_bundled_openclaw_entry(app: &AppHandle) -> Result<PathBuf, String> {
    let entry = resolve_bundled_openclaw_root(app)?.join("openclaw.mjs");
    if entry.exists() {
        return Ok(entry);
    }
    Err(format!(
        "Bundled OpenClaw entry was not found: {}",
        entry.display()
    ))
}

pub fn resolve_bundled_node_bin(app: &AppHandle) -> Result<PathBuf, String> {
    let target = platform_target_id();
    let node_root = resolve_resource_dir(app).join("node").join(target);
    let binary = if cfg!(target_os = "windows") {
        node_root.join("node.exe")
    } else {
        node_root.join("bin").join("node")
    };

    if binary.exists() {
        return Ok(binary);
    }

    Err(format!(
        "Bundled Node runtime for this platform was not found: {}",
        binary.display()
    ))
}

pub fn generate_default_gateway_token() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("openclaw-{:x}", timestamp)
}

pub fn build_default_openclaw_config(token: &str) -> serde_json::Value {
    json!({
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

fn merge_missing_config(target: &mut serde_json::Value, defaults: &serde_json::Value) {
    if let Some(default_map) = defaults.as_object() {
        if let Some(target_map) = target.as_object_mut() {
            for (key, default_value) in default_map {
                match target_map.get_mut(key) {
                    Some(existing_value) => merge_missing_config(existing_value, default_value),
                    None => {
                        target_map.insert(key.clone(), default_value.clone());
                    }
                }
            }
            return;
        }
    }

    if target.is_null() {
        *target = defaults.clone();
    }
}

fn backfill_legacy_model_names(config: &mut serde_json::Value) {
    let Some(providers) = config
        .get_mut("models")
        .and_then(|models| models.get_mut("providers"))
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };

    for provider in providers.values_mut() {
        let Some(models) = provider
            .get_mut("models")
            .and_then(serde_json::Value::as_array_mut)
        else {
            continue;
        };

        for model in models {
            let Some(model_map) = model.as_object_mut() else {
                continue;
            };

            let legacy_id = model_map
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
            let name_missing = model_map
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::is_empty)
                .unwrap_or(true);

            if name_missing {
                if let Some(id) = legacy_id {
                    model_map.insert("name".to_string(), serde_json::Value::String(id));
                }
            }
        }
    }
}

pub fn ensure_gateway_ready_config(config: &mut serde_json::Value, token: &str) {
    let defaults = build_default_openclaw_config(token);
    merge_missing_config(config, &defaults);
    backfill_legacy_model_names(config);
}

fn ensure_default_config_file_with(home_dir: &Path) -> Result<PathBuf, String> {
    let data_dir = ensure_openclaw_data_dir_with(home_dir)?;
    let config_path = data_dir.join("openclaw.json");
    let token = generate_default_gateway_token();
    let mut config = if config_path.exists() {
        let raw = fs::read_to_string(&config_path)
            .map_err(|error| format!("Failed to read the config file: {}", error))?;
        serde_json::from_str(&raw)
            .map_err(|error| format!("Failed to parse the config file: {}", error))?
    } else {
        json!({})
    };

    ensure_gateway_ready_config(&mut config, &token);

    let serialized = serde_json::to_string_pretty(&config)
        .map_err(|error| format!("Failed to serialize the default config: {}", error))?;
    fs::write(&config_path, serialized)
        .map_err(|error| format!("Failed to write the default config file: {}", error))?;
    Ok(config_path)
}

pub fn ensure_default_config_file() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Unable to resolve the home directory.".to_string())?;
    ensure_default_config_file_with(&home_dir)
}

fn resolve_runtime_config_path_for_launch_with(
    home_dir: &Path,
    local_openclaw_available: bool,
) -> Result<PathBuf, String> {
    let probe = probe_runtime_config_with(home_dir, local_openclaw_available);
    if probe.should_skip_setup {
        return Ok(probe.config_path);
    }

    ensure_default_config_file_with(home_dir)
}

pub fn resolve_runtime_config_path_for_launch() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Unable to resolve the home directory.".to_string())?;
    let local_openclaw_available = detect_system_openclaw_bin_path().is_some();
    resolve_runtime_config_path_for_launch_with(&home_dir, local_openclaw_available)
}

pub fn get_runtime_health(app: &AppHandle) -> RuntimeHealth {
    let node_path = resolve_bundled_node_bin(app).ok();
    let openclaw_root = resolve_bundled_openclaw_root(app).ok();
    let local_openclaw_available = detect_system_openclaw_bin_path().is_some();
    let config_probe = dirs::home_dir()
        .map(|home_dir| probe_runtime_config_with(&home_dir, local_openclaw_available));
    let data_dir = openclaw_data_dir().ok();

    RuntimeHealth {
        runtime_not_ready: node_path.is_none() || openclaw_root.is_none(),
        node_ready: node_path.is_some(),
        openclaw_ready: openclaw_root.is_some(),
        config_ready: config_probe
            .as_ref()
            .map(|probe| probe.config_ready)
            .unwrap_or(false),
        local_openclaw_available,
        should_skip_setup: config_probe
            .as_ref()
            .map(|probe| probe.should_skip_setup)
            .unwrap_or(false),
        config_path: config_probe
            .as_ref()
            .map(|probe| probe.config_path.clone())
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        data_dir: data_dir.unwrap_or_default().to_string_lossy().to_string(),
        node_path: node_path.unwrap_or_default().to_string_lossy().to_string(),
        openclaw_root: openclaw_root
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    }
}

pub fn build_openclaw_command(app: &AppHandle, args: &[String]) -> Result<Command, String> {
    let node_bin = resolve_bundled_node_bin(app)?;
    let openclaw_entry = resolve_bundled_openclaw_entry(app)?;
    let working_dir = openclaw_entry
        .parent()
        .map(Path::to_path_buf)
        .ok_or("Unable to resolve the OpenClaw working directory.".to_string())?;
    let data_dir = ensure_openclaw_data_dir()?;
    let config_path = resolve_runtime_config_path_for_launch()?;
    crate::startup_trace::append(
        "bundled_runtime.command",
        format!(
            "node={} entry={} args={:?}",
            node_bin.display(),
            openclaw_entry.display(),
            args
        ),
    );

    let mut command = Command::new(&node_bin);
    command.arg(&openclaw_entry);
    command.args(args);
    command.current_dir(working_dir);
    command.env("OPENCLAW_HOME", &data_dir);
    command.env("OPENCLAW_CONFIG_PATH", &config_path);
    command.env("CLAWDBOT_CONFIG_PATH", &config_path);
    command.env("HOME", dirs::home_dir().unwrap_or_default());
    Ok(command)
}

pub fn run_openclaw_capture(app: &AppHandle, args: &[String]) -> Result<Output, String> {
    build_openclaw_command(app, args)?
        .output()
        .map_err(|error| format!("Failed to run the bundled OpenClaw command: {}", error))
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn default_openclaw_config_is_gateway_ready() {
        let token = "openclaw-test-token";
        let config = build_default_openclaw_config(token);

        assert_eq!(config["gateway"]["mode"].as_str(), Some("local"));
        assert_eq!(config["gateway"]["port"].as_u64(), Some(18789));
        assert_eq!(config["gateway"]["auth"]["mode"].as_str(), Some("token"));
        assert_eq!(config["gateway"]["auth"]["token"].as_str(), Some(token));
        assert_eq!(
            config["agents"]["defaults"]["workspace"].as_str(),
            Some("~/.openclaw/workspace")
        );
        assert_eq!(
            config["agents"]["defaults"]["model"]["primary"].as_str(),
            Some("placeholder/complete-quick-setup")
        );
    }

    #[test]
    fn ensure_gateway_ready_config_backfills_missing_gateway_block() {
        let mut config = json!({
            "models": {
                "providers": {
                    "demo": {
                        "baseUrl": "https://example.com"
                    }
                }
            }
        });

        ensure_gateway_ready_config(&mut config, "openclaw-test-token");

        assert_eq!(config["gateway"]["mode"].as_str(), Some("local"));
        assert_eq!(config["gateway"]["auth"]["mode"].as_str(), Some("token"));
        assert_eq!(
            config["models"]["providers"]["demo"]["baseUrl"].as_str(),
            Some("https://example.com")
        );
    }

    #[test]
    fn ensure_gateway_ready_config_migrates_legacy_model_id_to_name() {
        let mut config = json!({
            "models": {
                "providers": {
                    "openai": {
                        "baseUrl": "https://api.openai.com/v1",
                        "models": [
                            {
                                "id": "gpt-4o-mini"
                            }
                        ]
                    }
                }
            }
        });

        ensure_gateway_ready_config(&mut config, "openclaw-test-token");

        assert_eq!(
            config["models"]["providers"]["openai"]["models"][0]["name"].as_str(),
            Some("gpt-4o-mini")
        );
        assert_eq!(
            config["models"]["providers"]["openai"]["models"][0]["id"].as_str(),
            Some("gpt-4o-mini")
        );
    }

    #[test]
    fn select_resource_dir_falls_back_when_runtime_vendor_dir_is_missing() {
        let fake_runtime_dir = std::env::temp_dir().join("openclawswitch-missing-runtime-dir");
        let fallback = std::env::temp_dir().join("openclawswitch-source-vendor");
        let _ = fs::create_dir_all(&fallback);
        let selected =
            select_resource_dir_with_fallbacks(Some(fake_runtime_dir), &[fallback.clone()]);

        assert_eq!(selected, fallback);
        let _ = fs::remove_dir_all(fallback);
    }

    #[test]
    fn select_resource_dir_prefers_compiled_vendor_root_before_source_vendor() {
        let base = std::env::temp_dir().join("openclawswitch-resource-dir-test");
        let compiled = base
            .join("target")
            .join("debug")
            .join("resources")
            .join("vendor");
        let source = base.join("resources").join("vendor");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&compiled).expect("create compiled vendor dir");
        fs::create_dir_all(&source).expect("create source vendor dir");

        let selected =
            select_resource_dir_with_fallbacks(None, &[compiled.clone(), source.clone()]);

        assert_eq!(selected, compiled);
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn runtime_probe_skips_setup_when_local_openclaw_and_config_both_exist() {
        let home_dir = make_temp_dir("runtime-probe");
        let config_path = home_dir.join(".openclaw").join("openclaw.json");
        fs::create_dir_all(config_path.parent().expect("config parent"))
            .expect("create config dir");
        fs::write(&config_path, "{\"models\":{\"providers\":{}}}").expect("write config");

        let probe = probe_runtime_config_with(&home_dir, true);

        assert!(probe.config_ready);
        assert!(probe.should_skip_setup);
        assert_eq!(probe.config_path, config_path);
        let _ = fs::remove_dir_all(home_dir);
    }

    #[test]
    fn launch_path_keeps_existing_local_config_file_unchanged() {
        let home_dir = make_temp_dir("launch-config");
        let config_path = home_dir.join(".openclaw").join("openclaw.json");
        fs::create_dir_all(config_path.parent().expect("config parent"))
            .expect("create config dir");
        let original =
            "{\"models\":{\"providers\":{\"demo\":{\"baseUrl\":\"https://example.com\"}}}}";
        fs::write(&config_path, original).expect("write config");

        let selected = resolve_runtime_config_path_for_launch_with(&home_dir, true)
            .expect("select runtime config path");

        assert_eq!(selected, config_path);
        assert_eq!(
            fs::read_to_string(&config_path).expect("read config"),
            original
        );
        let _ = fs::remove_dir_all(home_dir);
    }
}
