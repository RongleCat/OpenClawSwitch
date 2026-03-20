use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewWindow, Window, WindowEvent,
};

const TRAY_ID: &str = "gateway-tray";
const MENU_TOGGLE_WINDOW_ID: &str = "toggle-window";
const MENU_OPEN_WEBUI_ID: &str = "open-webui";
const MENU_LAUNCH_AT_STARTUP_ID: &str = "launch-at-startup";
const MENU_OPEN_SETTINGS_ID: &str = "open-settings";
const MENU_EXIT_ID: &str = "exit-app";
const MENU_GATEWAY_STATUS_ID: &str = "gateway-status";
const MAIN_WINDOW_LABEL: &str = "main";

#[derive(Default)]
pub struct DesktopShellState {
    quitting: AtomicBool,
}

impl DesktopShellState {
    pub fn is_quitting(&self) -> bool {
        self.quitting.load(Ordering::SeqCst)
    }

    pub fn set_quitting(&self, quitting: bool) {
        self.quitting.store(quitting, Ordering::SeqCst);
    }
}

fn main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| "无法找到主窗口".to_string())
}

fn gateway_status_label(app: &AppHandle) -> String {
    let snapshot = crate::gateway_supervisor::current_gateway_status(app).unwrap_or(
        crate::bundled_runtime::GatewayStatusSnapshot {
            state: "stopped".to_string(),
            pid: None,
            url: Some(crate::bundled_runtime::default_web_ui_url().to_string()),
            message: None,
        },
    );

    let state_text = match snapshot.state.as_str() {
        "running" => "运行中",
        "starting" => "启动中",
        "error" => "异常",
        _ => "未运行",
    };

    format!("网关状态：{}", state_text)
}

fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
    let window = main_window(app)?;
    let window_visible = window.is_visible().unwrap_or(true);
    let launch_at_startup = crate::desktop_prefs::get_launch_at_startup_enabled_inner(app)?;

    let toggle_window = MenuItem::with_id(
        app,
        MENU_TOGGLE_WINDOW_ID,
        if window_visible {
            "隐藏主窗口"
        } else {
            "显示主窗口"
        },
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let gateway_status = MenuItem::with_id(
        app,
        MENU_GATEWAY_STATUS_ID,
        gateway_status_label(app),
        false,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let open_webui = MenuItem::with_id(
        app,
        MENU_OPEN_WEBUI_ID,
        "打开 OpenClaw WebUI",
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let launch_at_startup_item = CheckMenuItem::with_id(
        app,
        MENU_LAUNCH_AT_STARTUP_ID,
        "开机自启网关",
        true,
        launch_at_startup,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let open_settings =
        MenuItem::with_id(app, MENU_OPEN_SETTINGS_ID, "打开设置", true, None::<&str>)
            .map_err(|error| error.to_string())?;
    let exit = MenuItem::with_id(app, MENU_EXIT_ID, "退出", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let separator = PredefinedMenuItem::separator(app).map_err(|error| error.to_string())?;

    Menu::with_items(
        app,
        &[
            &toggle_window,
            &gateway_status,
            &open_webui,
            &launch_at_startup_item,
            &open_settings,
            &separator,
            &exit,
        ],
    )
    .map_err(|error| error.to_string())
}

pub fn refresh_tray_menu(app: &AppHandle) -> Result<(), String> {
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| "无法找到托盘图标".to_string())?;
    let menu = build_tray_menu(app)?;
    tray.set_menu(Some(menu)).map_err(|error| error.to_string())
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    let _ = window.unminimize();
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    refresh_tray_menu(app)?;
    Ok(())
}

pub fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    window.hide().map_err(|error| error.to_string())?;
    refresh_tray_menu(app)?;
    Ok(())
}

pub fn toggle_main_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    if window.is_visible().unwrap_or(true) {
        hide_main_window(app)
    } else {
        show_main_window(app)
    }
}

pub fn initialize(app: &AppHandle) -> Result<(), String> {
    let menu = build_tray_menu(app)?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("OpenClaw 控制台");

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app).map_err(|error| error.to_string())?;
    refresh_tray_menu(app)?;
    Ok(())
}

pub fn apply_startup_preferences(app: &AppHandle) -> Result<(), String> {
    crate::startup_trace::append("startup_preferences.begin", "checking launch-at-startup");
    let launch_at_startup = crate::desktop_prefs::get_launch_at_startup_enabled_inner(app)?;
    crate::startup_trace::append(
        "startup_preferences.flag",
        format!("launchAtStartup={launch_at_startup}"),
    );
    if launch_at_startup {
        crate::startup_trace::append(
            "startup_preferences.start_gateway",
            "launch-at-startup enabled; starting gateway",
        );
        let _ = crate::gateway_supervisor::start_gateway_process(app);
        let _ = refresh_tray_menu(app);
    }
    crate::startup_trace::append("startup_preferences.end", "preferences applied");
    Ok(())
}

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }

    if let WindowEvent::CloseRequested { api, .. } = event {
        let app = window.app_handle();
        if !app.state::<DesktopShellState>().is_quitting() {
            api.prevent_close();
            let _ = hide_main_window(&app);
        }
    }
}

pub fn handle_tray_icon_event(app: &AppHandle, event: &TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        let _ = toggle_main_window(app);
    }
}

pub fn handle_menu_event(app: &AppHandle, event: &MenuEvent) {
    match event.id().as_ref() {
        MENU_TOGGLE_WINDOW_ID => {
            let _ = toggle_main_window(app);
        }
        MENU_OPEN_WEBUI_ID => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = crate::installer::open_web_ui(app.clone()).await;
                let _ = refresh_tray_menu(&app);
            });
        }
        MENU_LAUNCH_AT_STARTUP_ID => {
            let current =
                crate::desktop_prefs::get_launch_at_startup_enabled_inner(app).unwrap_or(false);
            let _ = crate::desktop_prefs::set_launch_at_startup_enabled_inner(app, !current);
            let _ = refresh_tray_menu(app);
        }
        MENU_OPEN_SETTINGS_ID => {
            let _ = show_main_window(app);
            let _ = app.emit("desktop:navigate", "/settings");
        }
        MENU_EXIT_ID => {
            app.state::<DesktopShellState>().set_quitting(true);
            crate::gateway_supervisor::shutdown_gateway_processes(app);
            app.exit(0);
        }
        _ => {}
    }
}

pub fn handle_second_instance(app: &AppHandle) {
    let _ = show_main_window(app);
}
