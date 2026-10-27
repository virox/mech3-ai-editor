#![cfg_attr(not(debug_assertions),windows_subsystem="windows")]

#[path="../../../switchboard/src-tauri/src/runtime.rs"]
mod runtime;
mod trace;

#[tauri::command]
fn dev_scan_game_root(root_path:String)->Result<mech3_ai_editor_lib::LibrarySnapshot,String>{mech3_ai_editor_lib::scan_game_root_data(root_path)}

#[tauri::command]
fn dev_load_mission(root_path:String,mission_path:String,tracer:tauri::State<trace::TraceManager>)->Result<mech3_ai_editor_lib::ReadonlyMissionIndex,String>{tracer.stop(true);mech3_ai_editor_lib::load_readonly_mission_index(root_path,mission_path)}

#[tauri::command]
fn switchboard_candidates()->Result<Vec<runtime::ProcessCandidate>,String>{runtime::list_candidates()}

#[tauri::command]
fn switchboard_attach(pid:u32,monitor:tauri::State<runtime::RuntimeMonitor>,app:tauri::AppHandle)->Result<runtime::MonitorStatus,String>{monitor.attach(pid,app)}

#[tauri::command]
fn switchboard_detach(monitor:tauri::State<runtime::RuntimeMonitor>,tracer:tauri::State<trace::TraceManager>)->runtime::MonitorStatus{tracer.stop(true);monitor.detach()}

#[tauri::command]
fn switchboard_snapshot(monitor:tauri::State<runtime::RuntimeMonitor>)->runtime::SwitchboardSnapshot{runtime::snapshot(&monitor)}

#[tauri::command]
fn switchboard_set_write_enabled(enabled:bool,monitor:tauri::State<runtime::RuntimeMonitor>)->Result<runtime::MonitorStatus,String>{monitor.set_write_enabled(enabled)}

#[tauri::command]
fn switchboard_set_valve(name:String,value:i32,expected_value:i32,monitor:tauri::State<runtime::RuntimeMonitor>)->Result<runtime::ValveState,String>{monitor.set_valve(name,value,expected_value)}

#[tauri::command]
fn trace_start(monitor:tauri::State<runtime::RuntimeMonitor>,tracer:tauri::State<trace::TraceManager>)->Result<trace::TraceStatus,String>{let status=monitor.status();let pid=status.pid.ok_or("Attach the verified Mech3.exe mission before tracing.")?;tracer.start(pid)}

#[tauri::command]
fn trace_stop(tracer:tauri::State<trace::TraceManager>)->trace::TraceStatus{tracer.stop(true)}

#[tauri::command]
fn trace_snapshot(tracer:tauri::State<trace::TraceManager>)->trace::TraceSnapshot{tracer.snapshot()}

fn main(){
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(runtime::RuntimeMonitor::default())
        .manage(trace::TraceManager::default())
        .invoke_handler(tauri::generate_handler![dev_scan_game_root,dev_load_mission,switchboard_candidates,switchboard_attach,switchboard_detach,switchboard_snapshot,switchboard_set_write_enabled,switchboard_set_valve,trace_start,trace_stop,trace_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running the developer valve switchboard");
}
