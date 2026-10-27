#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod animations;
mod runtime;

#[tauri::command]
fn switchboard_candidates() -> Result<Vec<runtime::ProcessCandidate>, String> {
    runtime::list_candidates()
}

#[tauri::command]
fn switchboard_attach(
    pid: u32,
    monitor: tauri::State<runtime::RuntimeMonitor>,
    app: tauri::AppHandle,
) -> Result<runtime::MonitorStatus, String> {
    monitor.attach(pid, app)
}

#[tauri::command]
fn switchboard_detach(monitor: tauri::State<runtime::RuntimeMonitor>) -> runtime::MonitorStatus {
    monitor.detach()
}

#[tauri::command]
fn switchboard_set_write_enabled(
    enabled: bool,
    monitor: tauri::State<runtime::RuntimeMonitor>,
) -> Result<runtime::MonitorStatus, String> {
    monitor.set_write_enabled(enabled)
}

#[tauri::command]
fn switchboard_set_valve(
    name: String,
    value: i32,
    expected_value: i32,
    monitor: tauri::State<runtime::RuntimeMonitor>,
) -> Result<runtime::ValveState, String> {
    monitor.set_valve(name, value, expected_value)
}

#[tauri::command]
fn switchboard_snapshot(monitor: tauri::State<runtime::RuntimeMonitor>) -> runtime::SwitchboardSnapshot {
    runtime::snapshot(&monitor)
}

#[tauri::command]
fn switchboard_status(monitor: tauri::State<runtime::RuntimeMonitor>) -> runtime::MonitorStatus {
    monitor.status()
}

fn main() {
    tauri::Builder::default()
        .manage(runtime::RuntimeMonitor::default())
        .invoke_handler(tauri::generate_handler![
            switchboard_candidates,
            switchboard_attach,
            switchboard_detach,
            switchboard_set_write_enabled,
            switchboard_set_valve,
            switchboard_snapshot,
            switchboard_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the valve switchboard");
}
