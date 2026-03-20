use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DesktopPreferences {
    pub launch_at_startup: bool,
    pub setup_complete: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DesktopPreferencesPatch {
    pub launch_at_startup: Option<bool>,
    pub setup_complete: Option<bool>,
}

fn prefs_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录".to_string())?;
    Ok(home_dir.join(".openclaw").join("desktop-preferences.json"))
}

pub(crate) fn read_prefs() -> Result<DesktopPreferences, String> {
    let path = prefs_path()?;
    if !path.exists() {
        return Ok(DesktopPreferences::default());
    }
    let content =
        fs::read_to_string(&path).map_err(|error| format!("读取桌面偏好失败: {}", error))?;
    serde_json::from_str(&content).map_err(|error| format!("解析桌面偏好失败: {}", error))
}

pub(crate) fn write_prefs(prefs: &DesktopPreferences) -> Result<DesktopPreferences, String> {
    let path = prefs_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("创建桌面偏好目录失败: {}", error))?;
    }
    let content = serde_json::to_string_pretty(prefs)
        .map_err(|error| format!("序列化桌面偏好失败: {}", error))?;
    fs::write(path, content).map_err(|error| format!("写入桌面偏好失败: {}", error))?;
    Ok(prefs.clone())
}

#[tauri::command]
pub fn get_desktop_preferences() -> Result<DesktopPreferences, String> {
    read_prefs()
}

#[tauri::command]
pub fn set_desktop_preferences(
    app: AppHandle,
    preferences: DesktopPreferencesPatch,
) -> Result<DesktopPreferences, String> {
    let mut current = read_prefs()?;
    if let Some(value) = preferences.launch_at_startup {
        current.launch_at_startup = set_launch_at_startup_enabled_inner(&app, value)?;
    }
    if let Some(value) = preferences.setup_complete {
        current.setup_complete = value;
    }
    let prefs = write_prefs(&current)?;
    let _ = crate::desktop_shell::refresh_tray_menu(&app);
    Ok(prefs)
}

#[tauri::command]
pub fn get_launch_at_startup_enabled(app: AppHandle) -> Result<bool, String> {
    get_launch_at_startup_enabled_inner(&app)
}

#[tauri::command]
pub fn set_launch_at_startup_enabled(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let value = set_launch_at_startup_enabled_inner(&app, enabled)?;
    let _ = crate::desktop_shell::refresh_tray_menu(&app);
    Ok(value)
}

pub(crate) fn get_launch_at_startup_enabled_inner(app: &AppHandle) -> Result<bool, String> {
    crate::startup_trace::append("desktop_prefs.autolaunch.begin", "reading autolaunch state");
    let enabled = app
        .autolaunch()
        .is_enabled()
        .map_err(|error| format!("读取开机自启状态失败: {}", error))?;
    crate::startup_trace::append(
        "desktop_prefs.autolaunch.end",
        format!("enabled={enabled}"),
    );

    let mut prefs = read_prefs()?;
    if prefs.launch_at_startup != enabled {
        prefs.launch_at_startup = enabled;
        let _ = write_prefs(&prefs);
    }

    Ok(enabled)
}

pub(crate) fn set_launch_at_startup_enabled_inner(
    app: &AppHandle,
    enabled: bool,
) -> Result<bool, String> {
    if enabled {
        app.autolaunch()
            .enable()
            .map_err(|error| format!("开启开机自启失败: {}", error))?;
    } else {
        app.autolaunch()
            .disable()
            .map_err(|error| format!("关闭开机自启失败: {}", error))?;
    }

    let mut prefs = read_prefs()?;
    prefs.launch_at_startup = enabled;
    let _ = write_prefs(&prefs)?;
    Ok(enabled)
}
