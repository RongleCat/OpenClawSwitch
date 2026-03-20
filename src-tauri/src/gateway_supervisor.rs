use crate::bundled_runtime::{self, GatewayStatusSnapshot};
use std::process::Child;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

#[derive(Default)]
pub struct GatewaySupervisor {
    child: Mutex<Option<Child>>,
    last_message: Mutex<Option<String>>,
}

const GATEWAY_READY_TIMEOUT: Duration = Duration::from_secs(10);
const GATEWAY_READY_POLL_INTERVAL: Duration = Duration::from_millis(250);
const GATEWAY_HEALTH_TIMEOUT: Duration = Duration::from_millis(200);

fn resolve_gateway_state(process_alive: bool, http_ready: bool) -> &'static str {
    if !process_alive {
        "stopped"
    } else if http_ready {
        "running"
    } else {
        "starting"
    }
}

fn snapshot_from_child(
    child: Option<&mut Child>,
    message: Option<String>,
) -> GatewayStatusSnapshot {
    match child {
        Some(process) => {
            let http_ready = bundled_runtime::is_default_web_ui_reachable(GATEWAY_HEALTH_TIMEOUT);

            GatewayStatusSnapshot {
                state: resolve_gateway_state(true, http_ready).to_string(),
                pid: Some(process.id()),
                url: Some(bundled_runtime::default_web_ui_url().to_string()),
                message,
            }
        }
        None => GatewayStatusSnapshot {
            state: "stopped".to_string(),
            pid: None,
            url: Some(bundled_runtime::default_web_ui_url().to_string()),
            message,
        },
    }
}

fn gateway_launch_args() -> Vec<String> {
    vec!["gateway".to_string()]
}

fn wait_for_gateway_ready_with(
    mut check_ready: impl FnMut() -> bool,
    timeout: Duration,
    interval: Duration,
) -> bool {
    let mut waited = Duration::ZERO;
    loop {
        if check_ready() {
            return true;
        }

        if waited >= timeout {
            return false;
        }

        thread::sleep(interval);
        waited += interval;
    }
}

pub fn wait_for_gateway_ready() -> bool {
    wait_for_gateway_ready_with(
        || bundled_runtime::is_default_web_ui_reachable(GATEWAY_HEALTH_TIMEOUT),
        GATEWAY_READY_TIMEOUT,
        GATEWAY_READY_POLL_INTERVAL,
    )
}

pub fn start_gateway_process(app: &AppHandle) -> Result<String, String> {
    crate::startup_trace::append("gateway_supervisor.start.begin", "requested");
    let supervisor = app.state::<GatewaySupervisor>();
    let mut guard = supervisor
        .child
        .lock()
        .map_err(|_| "Failed to lock the gateway process state.".to_string())?;
    if let Some(child) = guard.as_mut() {
        if child
            .try_wait()
            .map_err(|error| format!("Failed to inspect the gateway process state: {}", error))?
            .is_none()
        {
            *supervisor
                .last_message
                .lock()
                .map_err(|_| "Failed to update the gateway status message.".to_string())? =
                Some("Gateway is already running.".to_string());
            crate::startup_trace::append("gateway_supervisor.start.skip", "already running");
            return Ok("Gateway is already running.".to_string());
        }
    }

    let args = gateway_launch_args();
    crate::startup_trace::append(
        "gateway_supervisor.start.spawning",
        format!("args={args:?}"),
    );
    let child = bundled_runtime::build_openclaw_command(app, &args)?
        .spawn()
        .map_err(|error| format!("Failed to start the gateway: {}", error))?;
    let pid = child.id();
    crate::startup_trace::append("gateway_supervisor.start.spawned", format!("pid={pid}"));
    *guard = Some(child);
    *supervisor
        .last_message
        .lock()
        .map_err(|_| "Failed to update the gateway status message.".to_string())? =
        Some(format!("Gateway started with PID {}.", pid));
    Ok("Gateway start command was launched.".to_string())
}

pub fn stop_gateway_process(app: &AppHandle) -> Result<String, String> {
    let supervisor = app.state::<GatewaySupervisor>();
    let mut guard = supervisor
        .child
        .lock()
        .map_err(|_| "Failed to lock the gateway process state.".to_string())?;
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    *supervisor
        .last_message
        .lock()
        .map_err(|_| "Failed to update the gateway status message.".to_string())? =
        Some("Gateway stopped.".to_string());
    Ok("Gateway stop command was launched.".to_string())
}

pub fn restart_gateway_process(app: &AppHandle) -> Result<String, String> {
    let _ = stop_gateway_process(app);
    start_gateway_process(app)?;
    Ok("Gateway restart command was sent.".to_string())
}

pub fn current_gateway_status(app: &AppHandle) -> Result<GatewayStatusSnapshot, String> {
    let supervisor = app.state::<GatewaySupervisor>();
    let mut guard = supervisor
        .child
        .lock()
        .map_err(|_| "Failed to lock the gateway process state.".to_string())?;
    if let Some(child) = guard.as_mut() {
        if child
            .try_wait()
            .map_err(|error| format!("Failed to inspect the gateway process state: {}", error))?
            .is_some()
        {
            *guard = None;
        }
    }
    let message = supervisor
        .last_message
        .lock()
        .map_err(|_| "Failed to read the gateway status message.".to_string())?
        .clone();
    Ok(snapshot_from_child(guard.as_mut(), message))
}

pub fn shutdown_gateway_processes(app: &AppHandle) {
    let _ = stop_gateway_process(app);
}

#[cfg(test)]
mod tests {
    use super::{gateway_launch_args, resolve_gateway_state, wait_for_gateway_ready_with};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use std::time::Duration;

    #[test]
    fn gateway_launch_args_use_foreground_gateway_process() {
        assert_eq!(gateway_launch_args(), vec!["gateway".to_string()]);
    }

    #[test]
    fn gateway_state_is_starting_while_process_is_alive_but_http_is_not_ready() {
        assert_eq!(resolve_gateway_state(true, false), "starting");
        assert_eq!(resolve_gateway_state(true, true), "running");
        assert_eq!(resolve_gateway_state(false, false), "stopped");
    }

    #[test]
    fn wait_for_gateway_ready_stops_polling_once_gateway_becomes_ready() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_for_check = Arc::clone(&attempts);

        let ready = wait_for_gateway_ready_with(
            move || attempts_for_check.fetch_add(1, Ordering::SeqCst) >= 2,
            Duration::from_millis(5),
            Duration::ZERO,
        );

        assert!(ready);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn wait_for_gateway_ready_times_out_when_gateway_never_becomes_ready() {
        let ready = wait_for_gateway_ready_with(|| false, Duration::ZERO, Duration::ZERO);

        assert!(!ready);
    }
}
