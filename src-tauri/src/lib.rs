mod storage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, path::Path, sync::{atomic::{AtomicU64, Ordering}, Mutex}};

#[derive(Clone)] struct Session { root: Value, container: storage::Container }
#[derive(Default)] struct AppState { next: AtomicU64, sessions: Mutex<HashMap<u64, Session>> }

#[derive(Debug, Clone, Serialize, Deserialize)] #[serde(rename_all="camelCase")]
struct Vehicle { id:String, name:String, record:Vec<Value> }
#[derive(Debug, Clone, Serialize, Deserialize)] #[serde(rename_all="camelCase")]
struct DocumentSnapshot { session_id:u64, source_path:String, source_kind:String, #[serde(skip_serializing_if="Option::is_none")] archive_entry:Option<String>, vehicles:Vec<Vehicle>, linked_valves:Vec<String>, linked_nets:Vec<String>, warnings:Vec<String> }

fn vehicle_list(root:&Value)->anyhow::Result<&Vec<Value>> { let outer=root.as_array().ok_or_else(||anyhow::anyhow!("aiv root is not an array"))?; if outer.len()==1 { if let Some(inner)=outer[0].as_array(){return Ok(inner)} } Ok(outer) }
fn decode_vehicles(root:&Value)->anyhow::Result<Vec<Vehicle>> { let a=vehicle_list(root)?; let mut out=Vec::new(); for (i,p) in a.chunks_exact(2).enumerate(){ let name=p[0].as_str().ok_or_else(||anyhow::anyhow!("vehicle name at root index {} is not text (vehicle-list length {})",i*2,a.len()))?; let record=p[1].as_array().ok_or_else(||anyhow::anyhow!("record for {name} at root index {} is not an array (vehicle-list length {})",i*2+1,a.len()))?; out.push(Vehicle{id:format!("v{i}"),name:name.into(),record:record.clone()}); } Ok(out) }
fn encode_vehicles(v:&[Vehicle])->Value { Value::Array(v.iter().flat_map(|x|[Value::String(x.name.clone()),Value::Array(x.record.clone())]).collect()) }
fn encode_vehicles_like(v:&[Vehicle],original:&Value)->Value { let mut encoded=encode_vehicles(v); if let (Ok(old),Some(new))=(vehicle_list(original),encoded.as_array_mut()) { if old.len()%2==1 {if let Some(trailer)=old.last(){new.push(trailer.clone())}} } if original.as_array().is_some_and(|a|a.len()==1&&a[0].is_array()){Value::Array(vec![encoded])}else{encoded} }

#[tauri::command] fn open_document(path:String,state:tauri::State<AppState>)->Result<DocumentSnapshot,String>{
 let loaded=storage::load(Path::new(&path)).map_err(|e|format!("Could not open file: {e:#}"))?; let vehicles=decode_vehicles(&loaded.root).map_err(|e|e.to_string())?;
 let id=state.next.fetch_add(1,Ordering::Relaxed)+1; let mut warnings=loaded.warnings; if vehicle_list(&loaded.root).is_ok_and(|a|a.len()%2==1){warnings.push("The aiv vehicle list contains one trailing non-vehicle value. It will be preserved unchanged.".into())} let snap=DocumentSnapshot{session_id:id,source_path:path,source_kind:loaded.container.kind().into(),archive_entry:loaded.container.entry_name(),vehicles,linked_valves:loaded.linked_valves,linked_nets:loaded.linked_nets,warnings};
 state.sessions.lock().unwrap().insert(id,Session{root:loaded.root,container:loaded.container}); Ok(snap)
}
#[tauri::command] fn save_document(path:String,snapshot:DocumentSnapshot,state:tauri::State<AppState>)->Result<(),String>{
 let mut sessions=state.sessions.lock().unwrap(); let session=sessions.get_mut(&snapshot.session_id).ok_or("Editor session expired")?;session.root=encode_vehicles_like(&snapshot.vehicles,&session.root);
 storage::save(Path::new(&path),&session.root,&session.container).map_err(|e|format!("Could not save file: {e:#}"))
}
#[cfg_attr(mobile, tauri::mobile_entry_point)] pub fn run(){ tauri::Builder::default().plugin(tauri_plugin_dialog::init()).manage(AppState::default()).invoke_handler(tauri::generate_handler![open_document,save_document]).run(tauri::generate_context!()).expect("error while running application"); }

#[cfg(test)] mod tests { use super::*; #[test] fn duplicate_names_roundtrip(){let root=serde_json::json!(["A",[1,[2,3,4],5,6,"mystery",[7]],"A",[8,[9,10,11],12,13]]);let v=decode_vehicles(&root).unwrap();assert_eq!(encode_vehicles_like(&v,&root),root);assert_ne!(v[0].id,v[1].id);} #[test] fn wrapped_root_and_trailer_are_preserved(){let root=serde_json::json!([["A",[1,[2,3,4],5,6],"trailer"]]);let v=decode_vehicles(&root).unwrap();assert_eq!(v.len(),1);assert_eq!(encode_vehicles_like(&v,&root),root);} }
