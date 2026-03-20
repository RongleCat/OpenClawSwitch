// SSH 连接配置管理模块
// 负责连接配置的持久化存储（保存/加载/删除）

use crate::ssh::SshProfile;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 配置文件存储结构
#[derive(Debug, Serialize, Deserialize)]
struct ProfileStore {
    profiles: Vec<SshProfile>,
}

/// 获取配置文件存储路径
fn get_profiles_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let dir = home.join(".openclaw");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    Ok(dir.join("ssh_profiles.json"))
}

/// 从文件加载配置列表
fn load_store() -> Result<ProfileStore, String> {
    let path = get_profiles_path()?;
    if !path.exists() {
        return Ok(ProfileStore {
            profiles: Vec::new(),
        });
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("读取配置失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析配置失败: {}", e))
}

/// 保存配置列表到文件
fn save_store(store: &ProfileStore) -> Result<(), String> {
    let path = get_profiles_path()?;
    let json = serde_json::to_string_pretty(store).map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}

// ============================================================================
// Tauri 命令
// ============================================================================

/// 保存 SSH 连接配置
#[tauri::command]
pub fn ssh_save_profile(profile: SshProfile) -> Result<(), String> {
    let mut store = load_store()?;

    // 如果已存在同 id 的配置，更新；否则新增
    if let Some(existing) = store.profiles.iter_mut().find(|p| p.id == profile.id) {
        *existing = profile;
    } else {
        store.profiles.push(profile);
    }

    save_store(&store)
}

/// 加载所有 SSH 连接配置
#[tauri::command]
pub fn ssh_load_profiles() -> Result<Vec<SshProfile>, String> {
    let store = load_store()?;
    Ok(store.profiles)
}

/// 删除 SSH 连接配置
#[tauri::command]
pub fn ssh_delete_profile(id: String) -> Result<(), String> {
    let mut store = load_store()?;
    let original_len = store.profiles.len();
    store.profiles.retain(|p| p.id != id);

    if store.profiles.len() == original_len {
        return Err(format!("配置 '{}' 不存在", id));
    }

    save_store(&store)
}
