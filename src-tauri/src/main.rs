// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ============================================================================
// 类型定义说明
// ============================================================================
//
// 本项目使用 serde_json::Value 进行动态 JSON 操作以获得最大灵活性。
// 完整的 TypeScript 类型定义请参考 src/types/config.ts
//
// 主要类型：
// - OpenClawConfig: 完整的配置结构
// - ProviderConfig: 提供商配置
// - ModelConfig: 模型配置
// - ModelSelection: 模型选择配置
// ============================================================================

/// 返回给前端的提供商信息
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProviderInfo {
    name: String,
    base_url: String,
    has_api_key: bool,
    api: Option<String>,
    model_count: usize,
    models: Vec<ModelInfo>,
}

/// 返回给前端的模型信息
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ModelInfo {
    id: String,
    name: Option<String>,
    reasoning: bool,
    context_window: Option<u64>,
}

/// 返回给前端的模型选择信息
#[derive(Debug, Serialize, Clone)]
struct ModelSelectionInfo {
    primary: Option<String>,
    fallbacks: Vec<String>,
}

/// 配置文件信息
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ConfigFileInfo {
    path: String,
    mode: String, // "local" | "remote"
    file_name: String,
    dir_path: String,
}

/// OpenAI 格式 /models 响应中的模型项
#[derive(Debug, Serialize, Deserialize)]
struct ModelItem {
    id: String,
}

/// OpenAI 格式 /models 响应
#[derive(Debug, Serialize, Deserialize)]
struct ModelsListResponse {
    data: Vec<ModelItem>,
}

// ============================================================================
// 文件操作函数
// ============================================================================

/// 获取默认的 OpenClaw 配置目录
fn get_default_config_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home_dir.join(".openclaw"))
}

/// 在指定目录中检测配置文件
fn detect_config_file(dir_path: &PathBuf) -> Result<PathBuf, String> {
    // 优先级：openclaw.json > clawdbot.json
    let openclaw_path = dir_path.join("openclaw.json");
    if openclaw_path.exists() {
        return Ok(openclaw_path);
    }

    let clawdbot_path = dir_path.join("clawdbot.json");
    if clawdbot_path.exists() {
        return Ok(clawdbot_path);
    }

    Err(format!(
        "在目录 {} 中未找到 openclaw.json 或 clawdbot.json",
        dir_path.display()
    ))
}

/// 从指定路径加载配置文件
fn load_config_from_path(path: &PathBuf) -> Result<Value, String> {
    if !path.exists() {
        return Err(format!("配置文件不存在: {}", path.display()));
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config: Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    Ok(config)
}

/// 保存配置到指定路径
fn save_config_to_path(config: &Value, path: &PathBuf) -> Result<(), String> {
    // 确保目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    fs::write(path, json).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

// ============================================================================
// Tauri 命令
// ============================================================================

/// 获取默认配置目录路径
#[tauri::command]
fn get_default_config_path() -> Result<String, String> {
    let dir = get_default_config_dir()?;
    Ok(dir.to_string_lossy().to_string())
}

/// 从默认目录加载配置（本地模式）
#[tauri::command]
fn load_default_config() -> Result<(Value, ConfigFileInfo), String> {
    let dir = get_default_config_dir()?;
    let path = detect_config_file(&dir)?;
    let config = load_config_from_path(&path)?;

    let info = ConfigFileInfo {
        path: path.to_string_lossy().to_string(),
        mode: "local".to_string(),
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        dir_path: dir.to_string_lossy().to_string(),
    };

    Ok((config, info))
}

/// 加载本地配置文件（~/.openclaw）
#[tauri::command]
fn load_local_config() -> Result<(Value, ConfigFileInfo), String> {
    load_default_config()
}

/// 从指定目录加载配置（本地模式）
#[tauri::command]
fn load_config_from_directory(dir_path: String) -> Result<(Value, ConfigFileInfo), String> {
    let dir = PathBuf::from(&dir_path);
    if !dir.exists() || !dir.is_dir() {
        return Err(format!("目录不存在: {}", dir_path));
    }

    let path = detect_config_file(&dir)?;
    let config = load_config_from_path(&path)?;

    let info = ConfigFileInfo {
        path: path.to_string_lossy().to_string(),
        mode: "local".to_string(),
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        dir_path,
    };

    Ok((config, info))
}

/// 从指定文件加载配置（远程模式）
#[tauri::command]
fn load_config_from_file(file_path: String) -> Result<(Value, ConfigFileInfo), String> {
    let path = PathBuf::from(&file_path);
    let config = load_config_from_path(&path)?;

    let dir_path = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let info = ConfigFileInfo {
        path: file_path,
        mode: "remote".to_string(),
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        dir_path,
    };

    Ok((config, info))
}

/// 保存配置到原文件
#[tauri::command]
fn save_config(config: Value, path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    save_config_to_path(&config, &path)
}

/// 另存为（返回新路径，前端需要先调用对话框获取路径）
#[tauri::command]
fn save_config_as(config: Value, new_path: String) -> Result<String, String> {
    let path = PathBuf::from(&new_path);
    save_config_to_path(&config, &path)?;
    Ok(new_path)
}

// ============================================================================
// 配置操作命令
// ============================================================================

/// 获取提供商列表
#[tauri::command]
fn get_providers(config: Value) -> Result<Vec<ProviderInfo>, String> {
    let mut providers = Vec::new();

    if let Some(models) = config.get("models") {
        if let Some(providers_obj) = models.get("providers") {
            if let Some(obj) = providers_obj.as_object() {
                for (name, provider_value) in obj {
                    let base_url = provider_value
                        .get("baseUrl")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let has_api_key = provider_value
                        .get("apiKey")
                        .map(|v| !v.is_null() && v.as_str().map(|s| !s.is_empty()).unwrap_or(true))
                        .unwrap_or(false);

                    let api = provider_value
                        .get("api")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let models: Vec<ModelInfo> = provider_value
                        .get("models")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|m| {
                                    let id = m.get("id")?.as_str()?.to_string();
                                    let name = m.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                                    let reasoning = m.get("reasoning").and_then(|v| v.as_bool()).unwrap_or(false);
                                    let context_window = m.get("contextWindow").and_then(|v| v.as_u64());
                                    Some(ModelInfo {
                                        id,
                                        name,
                                        reasoning,
                                        context_window,
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    providers.push(ProviderInfo {
                        name: name.clone(),
                        base_url,
                        has_api_key,
                        api,
                        model_count: models.len(),
                        models,
                    });
                }
            }
        }
    }

    // 按名称排序
    providers.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(providers)
}

/// 获取当前模型选择
#[tauri::command]
fn get_model_selection(config: Value) -> Result<ModelSelectionInfo, String> {
    let primary = config
        .get("agents")
        .and_then(|a| a.get("defaults"))
        .and_then(|d| d.get("model"))
        .and_then(|m| m.get("primary"))
        .and_then(|p| p.as_str())
        .map(|s| s.to_string());

    let fallbacks = config
        .get("agents")
        .and_then(|a| a.get("defaults"))
        .and_then(|d| d.get("model"))
        .and_then(|m| m.get("fallbacks"))
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(ModelSelectionInfo { primary, fallbacks })
}

/// 设置主要模型
#[tauri::command]
fn set_primary_model(mut config: Value, model_path: String) -> Result<Value, String> {
    // 确保 agents.defaults.model 结构存在
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }
    if config["agents"]["defaults"].get("model").is_none() {
        config["agents"]["defaults"]["model"] = json!({});
    }

    config["agents"]["defaults"]["model"]["primary"] = json!(model_path);
    Ok(config)
}

/// 设置备用模型
#[tauri::command]
fn set_fallback_models(mut config: Value, fallbacks: Vec<String>) -> Result<Value, String> {
    // 确保 agents.defaults.model 结构存在
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }
    if config["agents"]["defaults"].get("model").is_none() {
        config["agents"]["defaults"]["model"] = json!({});
    }

    if fallbacks.is_empty() {
        // 移除 fallbacks 字段
        if let Some(model) = config["agents"]["defaults"]["model"].as_object_mut() {
            model.remove("fallbacks");
        }
    } else {
        config["agents"]["defaults"]["model"]["fallbacks"] = json!(fallbacks);
    }

    Ok(config)
}

/// 添加或更新提供商
#[tauri::command]
fn upsert_provider(
    mut config: Value,
    name: String,
    base_url: String,
    api_key: Option<String>,
    api: Option<String>,
) -> Result<Value, String> {
    // 确保 models.providers 结构存在
    if config.get("models").is_none() {
        config["models"] = json!({});
    }
    if config["models"].get("providers").is_none() {
        config["models"]["providers"] = json!({});
    }

    // 如果已存在，基于原有配置更新；否则创建新的
    let mut provider = if let Some(existing) = config["models"]["providers"].get(&name) {
        existing.clone()
    } else {
        json!({})
    };

    // 更新必填字段
    provider["baseUrl"] = json!(base_url);

    // 更新可选字段（仅当提供时）
    if let Some(key) = api_key {
        if !key.is_empty() {
            provider["apiKey"] = json!(key);
        }
    }

    if let Some(api_type) = api {
        if !api_type.is_empty() {
            provider["api"] = json!(api_type);
        }
    }

    // 如果没有设置 api 类型，默认设置为 openai-completions
    if provider.get("api").is_none() {
        provider["api"] = json!("openai-completions");
    }

    config["models"]["providers"][&name] = provider;
    Ok(config)
}

/// 删除提供商
#[tauri::command]
fn delete_provider(mut config: Value, name: String) -> Result<Value, String> {
    // 检查提供商是否存在
    let provider_exists = config
        .get("models")
        .and_then(|m| m.get("providers"))
        .and_then(|p| p.as_object())
        .map(|obj| obj.contains_key(&name))
        .unwrap_or(false);

    if !provider_exists {
        return Err(format!("提供商 '{}' 不存在", name));
    }

    // 在删除前，先收集所有剩余提供商的信息（用于查找新模型）
    let all_providers: Vec<(String, String)> = config
        .get("models")
        .and_then(|m| m.get("providers"))
        .and_then(|p| p.as_object())
        .map(|obj| {
            obj.iter()
                .filter(|(k, _)| *k != &name) // 排除要删除的提供商
                .filter_map(|(k, v)| {
                    v.get("models")
                        .and_then(|models| models.as_array())
                        .and_then(|models_list| {
                            if !models_list.is_empty() {
                                models_list.first()
                                    .and_then(|first_model| first_model.get("id"))
                                    .and_then(|id| id.as_str())
                                    .map(|id_str| (k.clone(), id_str.to_string()))
                            } else {
                                None
                            }
                        })
                })
                .collect()
        })
        .unwrap_or_default();

    // 按名称排序
    let mut sorted_providers = all_providers.clone();
    sorted_providers.sort_by(|a, b| a.0.cmp(&b.0));

    // 找到第一个可用模型
    let first_available_model = sorted_providers.first().map(|(provider_name, model_id)| {
        format!("{}/{}", provider_name, model_id)
    });

    // 现在删除提供商
    if let Some(models) = config.get_mut("models") {
        if let Some(providers) = models.get_mut("providers") {
            if let Some(obj) = providers.as_object_mut() {
                obj.remove(&name);
            }
        }
    }

    // 检查是否需要清除模型选择
    let provider_prefix = format!("{}/", name);

    if let Some(agents) = config.get_mut("agents") {
        if let Some(defaults) = agents.get_mut("defaults") {
            if let Some(model) = defaults.get_mut("model") {
                // 清除 fallbacks 中该提供商的所有模型
                if let Some(fallbacks) = model.get_mut("fallbacks") {
                    if let Some(arr) = fallbacks.as_array_mut() {
                        arr.retain(|v| {
                            v.as_str()
                                .map(|s| !s.starts_with(&provider_prefix))
                                .unwrap_or(true)
                        });
                        if arr.is_empty() {
                            if let Some(obj) = model.as_object_mut() {
                                obj.remove("fallbacks");
                            }
                        }
                    }
                }

                // 检查 primary 是否需要切换
                let should_clear_primary = model
                    .get("primary")
                    .and_then(|p| p.as_str())
                    .map(|s| s.starts_with(&provider_prefix))
                    .unwrap_or(false);

                if should_clear_primary {
                    // 尝试从剩余的 fallback 中找第一个
                    let new_primary_from_fallback = model
                        .get("fallbacks")
                        .and_then(|f| f.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    if let Some(new_p) = new_primary_from_fallback {
                        model["primary"] = json!(new_p);
                        // 移除已使用的 fallback
                        if let Some(fallbacks) = model.get_mut("fallbacks") {
                            if let Some(arr) = fallbacks.as_array_mut() {
                                arr.remove(0);
                                if arr.is_empty() {
                                    if let Some(obj) = model.as_object_mut() {
                                        obj.remove("fallbacks");
                                    }
                                }
                            }
                        }
                    } else {
                        // 没有可用的 fallback，使用第一个可用模型
                        if let Some(new_model) = first_available_model {
                            model["primary"] = json!(new_model);
                        } else {
                            // 没有找到任何可用模型，清除 primary
                            if let Some(obj) = model.as_object_mut() {
                                obj.remove("primary");
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(config)
}

/// 添加模型到提供商
#[tauri::command]
fn add_model_to_provider(
    mut config: Value,
    provider_name: String,
    model_id: String,
    model_name: Option<String>,
) -> Result<Value, String> {
    // 检查提供商是否存在
    let provider_exists = config
        .get("models")
        .and_then(|m| m.get("providers"))
        .and_then(|p| p.get(&provider_name))
        .is_some();

    if !provider_exists {
        return Err(format!("提供商 '{}' 不存在", provider_name));
    }

    // 确保 models 数组存在
    if config["models"]["providers"][&provider_name]
        .get("models")
        .is_none()
    {
        config["models"]["providers"][&provider_name]["models"] = json!([]);
    }

    // 创建新模型，name 默认与 id 相同
    let name = model_name.unwrap_or_else(|| model_id.clone());
    let new_model = json!({
        "id": model_id,
        "name": name,
    });

    // 添加到数组
    if let Some(models) = config["models"]["providers"][&provider_name]["models"].as_array_mut() {
        // 检查是否已存在
        let exists = models.iter().any(|m| {
            m.get("id")
                .and_then(|id| id.as_str())
                .map(|s| s == model_id)
                .unwrap_or(false)
        });
        if exists {
            return Err(format!("模型 '{}' 已存在", model_id));
        }
        models.push(new_model);
    }

    Ok(config)
}

/// 从提供商删除模型
#[tauri::command]
fn remove_model_from_provider(
    mut config: Value,
    provider_name: String,
    model_id: String,
) -> Result<Value, String> {
    if let Some(models) = config
        .get_mut("models")
        .and_then(|m| m.get_mut("providers"))
        .and_then(|p| p.get_mut(&provider_name))
        .and_then(|provider| provider.get_mut("models"))
        .and_then(|m| m.as_array_mut())
    {
        let original_len = models.len();
        models.retain(|m| {
            m.get("id")
                .and_then(|id| id.as_str())
                .map(|s| s != model_id)
                .unwrap_or(true)
        });
        if models.len() == original_len {
            return Err(format!("模型 '{}' 不存在", model_id));
        }
    } else {
        return Err(format!("提供商 '{}' 不存在或没有模型", provider_name));
    }

    Ok(config)
}

/// 从提供商 API 获取模型列表
#[tauri::command]
async fn fetch_provider_models(
    base_url: String,
    api_key: String,
) -> Result<Vec<String>, String> {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建客户端失败: {}", e))?;

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API 返回错误: {}", response.status()));
    }

    let result: ModelsListResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    Ok(result.data.into_iter().map(|m| m.id).collect())
}

// ============================================================================
// OpenClaw 工具命令
// ============================================================================

/// 重启 OpenClaw 网关
#[tauri::command]
fn restart_gateway() -> Result<String, String> {
    // 使用 spawn 非阻塞方式执行，避免 GUI 卡死
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/c", "openclaw", "gateway", "restart"])
            .spawn()
            .map_err(|e| format!("执行命令失败: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("sh")
            .args(["-c", "openclaw gateway restart"])
            .spawn()
            .map_err(|e| format!("执行命令失败: {}", e))?;
    }

    Ok("网关重启命令已发送".to_string())
}

/// 打开终端并进入 TUI 模式
#[tauri::command]
fn open_tui() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 start 命令打开新的 cmd 窗口并执行 openclaw tui
        Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", "openclaw tui"])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 osascript 打开 Terminal 并执行命令
        // 需要先加载 shell 配置以确保 openclaw 命令可用
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let shell_name = shell
            .rsplit('/')
            .next()
            .unwrap_or("zsh")
            .to_string();

        // 根据不同 shell 加载相应的配置文件
        let config_file = match shell_name.as_str() {
            "zsh" => "$HOME/.zshrc",
            "bash" => "$HOME/.bash_profile",
            _ => "$HOME/.zshrc",
        };

        // 使用 printf 避免 shell 转义问题
        Command::new("osascript")
            .args([
                "-e",
                &format!(
                    "tell application \"Terminal\" to do script \"source {} && openclaw tui\"",
                    config_file
                ),
                "-e",
                "tell application \"Terminal\" to activate",
            ])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // 尝试多种终端模拟器
        let terminals = [
            ("gnome-terminal", vec!["--", "openclaw", "tui"]),
            ("konsole", vec!["-e", "openclaw", "tui"]),
            ("xfce4-terminal", vec!["-e", "openclaw tui"]),
            ("xterm", vec!["-e", "openclaw", "tui"]),
        ];

        let mut success = false;
        for (terminal, args) in terminals {
            if Command::new(terminal)
                .args(&args)
                .spawn()
                .is_ok()
            {
                success = true;
                break;
            }
        }

        if !success {
            return Err("未找到可用的终端模拟器".to_string());
        }
    }

    Ok(())
}

// ============================================================================
// 主函数
// ============================================================================

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // 文件操作
            get_default_config_path,
            load_default_config,
            load_local_config,
            load_config_from_directory,
            load_config_from_file,
            save_config,
            save_config_as,
            // 配置操作
            get_providers,
            get_model_selection,
            set_primary_model,
            set_fallback_models,
            upsert_provider,
            delete_provider,
            add_model_to_provider,
            remove_model_from_provider,
            fetch_provider_models,
            // OpenClaw 工具
            restart_gateway,
            open_tui,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
