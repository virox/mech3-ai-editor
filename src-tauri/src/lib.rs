mod animation;
mod storage;
mod valve_graph;

use serde::{Deserialize,Serialize};
use serde_json::Value;
use std::{collections::{HashMap,HashSet},fs,path::{Path,PathBuf},sync::{atomic::{AtomicU64,Ordering},Mutex}};

#[derive(Clone)] struct Session{root:Value,container:storage::Container}
#[derive(Clone)] struct WorkspaceArchiveState{id:String,relative_path:PathBuf,layer:String,archive:storage::StoredArchive}
#[derive(Clone)] struct WorkspaceSession{root_path:PathBuf,mission_path:PathBuf,archives:Vec<WorkspaceArchiveState>}
#[derive(Default)] struct AppState{next:AtomicU64,sessions:Mutex<HashMap<u64,Session>>,workspaces:Mutex<HashMap<u64,WorkspaceSession>>,animations:Mutex<HashMap<String,animation::LoadedAnimations>>}

#[derive(Debug,Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]struct Vehicle{id:String,name:String,record:Vec<Value>}
#[derive(Debug,Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]struct DocumentSnapshot{session_id:u64,source_path:String,source_kind:String,#[serde(skip_serializing_if="Option::is_none")]archive_entry:Option<String>,vehicles:Vec<Vehicle>,linked_valves:Vec<String>,linked_nets:Vec<String>,warnings:Vec<String>}

#[derive(Debug,Clone,Serialize)]#[serde(rename_all="camelCase")]pub struct MissionSummary{pub path:String,pub relative_path:String,pub campaign:String,pub name:String,pub kind:String,pub vehicle_count:usize,pub has_valves:bool,pub network_count:usize,pub region_count:usize}
#[derive(Debug,Clone,Serialize)]#[serde(rename_all="camelCase")]pub struct LibrarySnapshot{pub root_path:String,pub missions:Vec<MissionSummary>,pub warnings:Vec<String>}
#[derive(Debug,Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]struct WorkspaceArchive{id:String,path:String,relative_path:String,layer:String,locked:bool}
#[derive(Debug,Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]struct WorkspaceDocument{id:String,archive_id:String,entry_name:String,kind:String,layer:String,root:Value}
#[derive(Debug,Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]struct ReferenceOccurrence{name:String,role:String,document_id:String,location:String,layer:String,target_kind:String}
#[derive(Debug,Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]struct Diagnostic{severity:String,code:String,message:String,#[serde(skip_serializing_if="Option::is_none")]document_id:Option<String>,#[serde(skip_serializing_if="Option::is_none")]location:Option<String>}
#[derive(Debug,Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]struct WorkspaceSnapshot{session_id:u64,root_path:String,selected_mission:String,archives:Vec<WorkspaceArchive>,documents:Vec<WorkspaceDocument>,references:Vec<ReferenceOccurrence>,diagnostics:Vec<Diagnostic>,#[serde(default)]valve_graph:valve_graph::ValveGraphSnapshot}
#[derive(Debug,Clone,Serialize)]#[serde(rename_all="camelCase")]struct ExportResult{files:Vec<String>,changed_documents:usize}

fn ai_layout_source_path()->Result<PathBuf,String>{Path::new(env!("CARGO_MANIFEST_DIR")).parent().map(|root|root.join("src").join("ai-layout.json")).ok_or("Could not locate the repository root".into())}
#[tauri::command]fn load_ai_layout()->Result<Value,String>{let path=ai_layout_source_path()?;let source=fs::read_to_string(&path).map_err(|e|format!("Could not read {}: {e}",path.display()))?;serde_json::from_str(&source).map_err(|e|format!("Invalid layout JSON in {}: {e}",path.display()))}
#[tauri::command]fn save_ai_layout(layout:Value)->Result<String,String>{if layout.get("version").and_then(Value::as_u64)!=Some(1)||!layout.get("tabs").is_some_and(Value::is_array){return Err("Refusing to save an invalid AI layout document".into())}let path=ai_layout_source_path()?;let source=serde_json::to_string_pretty(&layout).map_err(|e|e.to_string())?+"\n";fs::write(&path,source).map_err(|e|format!("Could not write {}: {e}",path.display()))?;Ok(path.to_string_lossy().into())}

fn pair_list(root:&Value)->Option<&Vec<Value>>{let outer=root.as_array()?;if outer.len()==1{outer[0].as_array().or(Some(outer))}else{Some(outer)}}
fn vehicle_list(root:&Value)->anyhow::Result<&Vec<Value>>{pair_list(root).ok_or_else(||anyhow::anyhow!("aiv root is not an array"))}
fn decode_vehicles(root:&Value)->anyhow::Result<Vec<Vehicle>>{let a=vehicle_list(root)?;let mut out=Vec::new();for(i,p)in a.chunks_exact(2).enumerate(){let name=p[0].as_str().ok_or_else(||anyhow::anyhow!("vehicle name at root index {} is not text (vehicle-list length {})",i*2,a.len()))?;let record=p[1].as_array().ok_or_else(||anyhow::anyhow!("record for {name} at root index {} is not an array (vehicle-list length {})",i*2+1,a.len()))?;out.push(Vehicle{id:format!("v{i}"),name:name.into(),record:record.clone()});}Ok(out)}
fn encode_vehicles(v:&[Vehicle])->Value{Value::Array(v.iter().flat_map(|x|[Value::String(x.name.clone()),Value::Array(x.record.clone())]).collect())}
fn encode_vehicles_like(v:&[Vehicle],original:&Value)->Value{let mut encoded=encode_vehicles(v);if let(Ok(old),Some(new))=(vehicle_list(original),encoded.as_array_mut()){if old.len()%2==1{if let Some(trailer)=old.last(){new.push(trailer.clone())}}}if original.as_array().is_some_and(|a|a.len()==1&&a[0].is_array()){Value::Array(vec![encoded])}else{encoded}}

fn classify_entry(name:&str)->String{let n=name.to_ascii_lowercase();if n=="aiv.zrd"{"aiv"}else if n=="valves.zrd"{"valves"}else if n.starts_with("net_")&&n.ends_with(".zrd"){"network"}else if n.starts_with("reg_")&&n.ends_with(".zrd"){"region"}else if n=="targets.zrd"{"targets"}else if n=="objectives.zrd"||n=="mpinfo.zrd"{"objectives"}else{"raw"}.into()}
fn entry_stem(name:&str)->String{name.strip_suffix(".zrd").or_else(||name.strip_suffix(".ZRD")).unwrap_or(name).to_owned()}
fn rel_string(path:&Path)->String{path.to_string_lossy().replace('\\',"/")}

fn collect_reader_files(dir:&Path,out:&mut Vec<PathBuf>)->std::io::Result<()>{for item in fs::read_dir(dir)?{let path=item?.path();if path.is_dir(){collect_reader_files(&path,out)?}else if path.extension().and_then(|x|x.to_str()).is_some_and(|x|x.eq_ignore_ascii_case("zbd")){let stem=path.file_stem().and_then(|x|x.to_str()).unwrap_or("").to_ascii_lowercase();if stem.starts_with("reader")&&stem!="reader"{out.push(path)}}}Ok(())}
fn first_pair_string(root:&Value,key:&str)->Option<String>{let list=pair_list(root)?;for pair in list.chunks_exact(2){if pair[0].as_str().is_some_and(|x|x.eq_ignore_ascii_case(key)){return pair[1].as_array()?.first()?.as_str().map(str::to_owned)}}None}

pub fn scan_game_root_data(root_path:String)->Result<LibrarySnapshot,String>{let root=PathBuf::from(&root_path);let zbd=root.join("zbd");if !zbd.is_dir(){return Err(format!("{} has no zbd directory",root.display()))}let mut files=Vec::new();collect_reader_files(&zbd,&mut files).map_err(|e|e.to_string())?;files.sort();let mut missions=Vec::new();let mut warnings=Vec::new();for path in files{let relative=path.strip_prefix(&root).unwrap_or(&path).to_owned();let campaign=path.parent().and_then(|p|p.strip_prefix(&zbd).ok()).map(rel_string).filter(|x|!x.is_empty()).unwrap_or_else(||"global".into());let stem=path.file_stem().and_then(|x|x.to_str()).unwrap_or("reader").to_owned();let lower=stem.to_ascii_lowercase();let kind=if lower.starts_with("readermp"){"Multiplayer"}else if lower.starts_with("readeria"){"Instant Action"}else{"Mission"};let mut name=format!("{} / {}",campaign,stem);let mut vehicle_count=0;let mut has_valves=false;let mut network_count=0;let mut region_count=0;match storage::load_archive(&path){Ok(archive)=>{for entry in &archive.entries{let n=entry.name.to_ascii_lowercase();if n=="mpinfo.zrd"{if let Some(root)=&entry.root{if let Some(db)=first_pair_string(root,"db_name"){name=format!("{} / {}",campaign,db)}}}else if n=="aiv.zrd"{vehicle_count=entry.root.as_ref().and_then(pair_list).map_or(0,|x|x.len()/2)}else if n=="valves.zrd"{has_valves=true}else if n.starts_with("net_")&&n.ends_with(".zrd"){network_count+=1}else if n.starts_with("reg_")&&n.ends_with(".zrd"){region_count+=1}}},Err(e)=>warnings.push(format!("Could not inspect {}: {e:#}",relative.display()))}missions.push(MissionSummary{path:path.to_string_lossy().into(),relative_path:rel_string(&relative),campaign,name,kind:kind.into(),vehicle_count,has_valves,network_count,region_count})}Ok(LibrarySnapshot{root_path,warnings,missions})}
#[tauri::command]fn scan_game_root(root_path:String)->Result<LibrarySnapshot,String>{scan_game_root_data(root_path)}

fn add_reference(out:&mut Vec<ReferenceOccurrence>,doc:&WorkspaceDocument,name:&str,role:&str,location:String,target_kind:&str){if !name.is_empty(){out.push(ReferenceOccurrence{name:name.into(),role:role.into(),document_id:doc.id.clone(),location,layer:doc.layer.clone(),target_kind:target_kind.into()})}}
fn collect_aiv_refs(doc:&WorkspaceDocument,out:&mut Vec<ReferenceOccurrence>){let Some(list)=pair_list(&doc.root)else{return};for(i,pair)in list.chunks_exact(2).enumerate(){let Some(vehicle)=pair[0].as_str()else{continue};add_reference(out,doc,vehicle,"defines-vehicle",format!("vehicle[{i}]"),"vehicle");let Some(record)=pair[1].as_array()else{continue};for j in(4..record.len().saturating_sub(1)).step_by(2){if record[j].as_str()!=Some("vlist"){continue}let Some(rows)=record[j+1].as_array()else{continue};for(k,row)in rows.iter().enumerate(){let Some(values)=row.as_array()else{continue};if let Some(name)=values.get(15).and_then(Value::as_str){add_reference(out,doc,name,"writes-valve",format!("{vehicle}.vlist[{k}][15]"),"valve")}if let Some(name)=values.get(9).and_then(Value::as_str){add_reference(out,doc,name,"uses-region",format!("{vehicle}.vlist[{k}][9]"),"region")}}}}}
fn collect_valve_refs(doc:&WorkspaceDocument,out:&mut Vec<ReferenceOccurrence>){let Some(list)=pair_list(&doc.root)else{return};for(i,pair)in list.chunks_exact(2).enumerate(){let Some(event)=pair[0].as_str()else{continue};add_reference(out,doc,event,"handles-valve",format!("action[{i}].event"),"valve");let Some(record)=pair[1].as_array()else{continue};let Some(action)=record.first().and_then(Value::as_str)else{continue};if matches!(action,"any_nonzero"|"all_nonzero"|"all_zero"){if record.get(1).and_then(Value::as_str)==Some("namelist"){if let Some(names)=record.get(2).and_then(Value::as_array){for(k,name)in names.iter().filter_map(Value::as_str).enumerate(){add_reference(out,doc,name,"reads-valve",format!("action[{i}].namelist[{k}]"),"valve")}}}continue}let Some(args)=record.get(1).and_then(Value::as_array)else{continue};match action{"delayupdate"=>if let Some(name)=args.first().and_then(Value::as_str){add_reference(out,doc,name,"writes-valve-delayed",format!("action[{i}].args[0]"),"valve")},"shutdown"|"destroy"|"teleport"|"attack_strategy"=>if let Some(name)=args.first().and_then(Value::as_str){add_reference(out,doc,name,"uses-vehicle",format!("action[{i}].args[0]"),"vehicle")},"sound"|"sound_once"=>if let Some(name)=args.first().and_then(Value::as_str){add_reference(out,doc,name,"uses-sound",format!("action[{i}].args[0]"),"sound")},_=>{}}}}
fn tagged_arrays<'a>(value:&'a Value,out:&mut Vec<(&'a str,&'a Vec<Value>)>){let Some(a)=value.as_array()else{return};for pair in a.windows(2){if let(Some(tag),Some(v))=(pair[0].as_str(),pair[1].as_array()){out.push((tag,v))}}for child in a{tagged_arrays(child,out)}}
fn collect_net_refs(doc:&WorkspaceDocument,out:&mut Vec<ReferenceOccurrence>){add_reference(out,doc,&entry_stem(&doc.entry_name),if doc.kind=="region"{"defines-region"}else{"defines-network"},"document".into(),&doc.kind);let mut tags=Vec::new();tagged_arrays(&doc.root,&mut tags);for(i,(tag,v))in tags.into_iter().enumerate(){match tag{"valve"=>if let Some(name)=v.get(1).and_then(Value::as_str){add_reference(out,doc,name,"reads-valve",format!("{tag}[{i}]"),"valve")},"valveunion"=>for(k,row)in v.iter().filter_map(Value::as_array).enumerate(){if let Some(name)=row.get(1).and_then(Value::as_str){add_reference(out,doc,name,"reads-valve",format!("{tag}[{i}][{k}]"),"valve")}},"valve_assign"=>if let Some(name)=v.first().and_then(Value::as_str){add_reference(out,doc,name,"writes-valve",format!("{tag}[{i}]"),"valve")},"gate"=>if let Some(name)=v.first().and_then(Value::as_str){add_reference(out,doc,name,"uses-vehicle",format!("{tag}[{i}]"),"vehicle")},_=>{}}}}
fn collect_target_refs(doc:&WorkspaceDocument,out:&mut Vec<ReferenceOccurrence>){let mut tags=Vec::new();tagged_arrays(&doc.root,&mut tags);for(i,(tag,v))in tags.into_iter().enumerate(){if tag=="health_below"{for(k,chunk)in v.chunks(3).enumerate(){if let Some(name)=chunk.first().and_then(Value::as_str){add_reference(out,doc,name,"writes-valve",format!("health_below[{i}][{k}]"),"valve")}}}}}
fn collect_objective_refs(doc:&WorkspaceDocument,out:&mut Vec<ReferenceOccurrence>){let mut tags=Vec::new();tagged_arrays(&doc.root,&mut tags);for(i,(tag,v))in tags.into_iter().enumerate(){if tag=="valve"{if let Some(name)=v.first().and_then(Value::as_str){add_reference(out,doc,name,"observes-valve",format!("valve[{i}]"),"valve")}}else if tag=="set_valve"{if let Some(name)=v.first().and_then(Value::as_str){add_reference(out,doc,name,"writes-valve",format!("set_valve[{i}]"),"valve")}}}}
fn collect_animation_refs(doc:&WorkspaceDocument,out:&mut Vec<ReferenceOccurrence>){for callback in valve_graph::animation_state_callbacks(&doc.root).into_iter().filter(|callback|callback.depth==0){add_reference(out,doc,&callback.channel,"writes-valve-animation",format!("{}/{}.{}.CALLBACK({})",callback.channel,callback.animation_name,callback.sequence,callback.callback),"valve")}}
fn validate_links(doc:&WorkspaceDocument,out:&mut Vec<Diagnostic>){if doc.kind!="network"&&doc.kind!="region"{return}let Some(list)=pair_list(&doc.root)else{return};let mut node_ids=HashSet::new();for pair in list.chunks_exact(2){if let Some(name)=pair[0].as_str(){if let Some(id)=name.strip_prefix("node_").and_then(|x|x.parse::<i64>().ok()){node_ids.insert(id);}}}for pair in list.chunks_exact(2){let Some(name)=pair[0].as_str()else{continue};if !name.starts_with("node_"){continue}let Some(record)=pair[1].as_array()else{continue};if record.get(1).and_then(Value::as_array).is_some_and(|p|p.len()>=3){if let Some(links)=record.get(2).and_then(Value::as_array){for link in links.iter().filter_map(Value::as_i64){if !node_ids.contains(&link){out.push(Diagnostic{severity:"warning".into(),code:"invalid-node-link".into(),message:format!("{name} links to missing node_{link:02}"),document_id:Some(doc.id.clone()),location:Some(format!("{name}.links"))})}}}}}}
fn build_index(documents:&[WorkspaceDocument])->(Vec<ReferenceOccurrence>,Vec<Diagnostic>){let mut refs=Vec::new();for doc in documents{match doc.kind.as_str(){"aiv"=>collect_aiv_refs(doc,&mut refs),"valves"=>collect_valve_refs(doc,&mut refs),"network"|"region"=>collect_net_refs(doc,&mut refs),"targets"=>collect_target_refs(doc,&mut refs),"objectives"=>collect_objective_refs(doc,&mut refs),_ if valve_graph::is_animation_document(&doc.root)=>collect_animation_refs(doc,&mut refs),_=>{}}}let regions:HashSet<_>=refs.iter().filter(|r|r.role=="defines-region").map(|r|r.name.to_ascii_lowercase()).collect();let vehicles:HashSet<_>=refs.iter().filter(|r|r.role=="defines-vehicle").map(|r|r.name.to_ascii_lowercase()).collect();let mut diagnostics=Vec::new();for r in &refs{if r.role=="uses-region"&&!regions.contains(&r.name.to_ascii_lowercase()){diagnostics.push(Diagnostic{severity:"warning".into(),code:"missing-region".into(),message:format!("Region '{}' is referenced but not loaded",r.name),document_id:Some(r.document_id.clone()),location:Some(r.location.clone())})}else if r.role=="uses-vehicle"&&!vehicles.contains(&r.name.to_ascii_lowercase()){diagnostics.push(Diagnostic{severity:"warning".into(),code:"missing-vehicle".into(),message:format!("Vehicle '{}' is referenced but not loaded",r.name),document_id:Some(r.document_id.clone()),location:Some(r.location.clone())})}}for doc in documents{validate_links(doc,&mut diagnostics)}(refs,diagnostics)}

fn workspace_snapshot(id:u64,session:&WorkspaceSession)->WorkspaceSnapshot{let mut archives=Vec::new();let mut documents=Vec::new();for state in &session.archives{let locked=state.layer!="mission";archives.push(WorkspaceArchive{id:state.id.clone(),path:state.archive.path.to_string_lossy().into(),relative_path:rel_string(&state.relative_path),layer:state.layer.clone(),locked});for(i,entry)in state.archive.entries.iter().enumerate(){if let Some(root)=&entry.root{documents.push(WorkspaceDocument{id:format!("{}:e{i}",state.id),archive_id:state.id.clone(),entry_name:entry.name.clone(),kind:classify_entry(&entry.name),layer:state.layer.clone(),root:root.clone()})}}}let(references,diagnostics)=build_index(&documents);let selected_mission=session.mission_path.to_string_lossy().into_owned();let valve_graph=valve_graph::build_valve_graph(&documents,&archives,&selected_mission);WorkspaceSnapshot{session_id:id,root_path:session.root_path.to_string_lossy().into(),selected_mission,archives,documents,references,diagnostics,valve_graph}}

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct ReadonlyChannelEndpoint {
    pub id:String,
    pub kind:String,
    pub label:String,
    pub detail:String,
    pub layer:String,
    pub archive_relative_path:String,
    pub entry_name:String,
    pub locator:String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub value:Option<i64>,
}

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct ReadonlyChannelIndex {
    pub name:String,
    pub consumers:Vec<ReadonlyChannelEndpoint>,
    pub producers:Vec<ReadonlyChannelEndpoint>,
}

#[derive(Debug,Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct ReadonlyMissionIndex {
    pub root_path:String,
    pub selected_mission:String,
    pub channels:Vec<ReadonlyChannelIndex>,
    pub warnings:Vec<String>,
}

/// Read-only mission index shared by Reader Studio and developer companions.
/// It deliberately uses the same archive loader and behavior-graph bindings as
/// the editor, but does not allocate an editable workspace session.
pub fn load_readonly_mission_index(root_path:String,mission_path:String)->Result<ReadonlyMissionIndex,String>{
    let root=PathBuf::from(&root_path);
    let mission=PathBuf::from(&mission_path);
    let zbd=root.join("zbd");
    if !zbd.is_dir(){return Err(format!("{} has no zbd directory",root.display()))}
    if !mission.is_file(){return Err(format!("{} is not a Reader archive",mission.display()))}
    let mut sources=Vec::<(PathBuf,String)>::new();
    let global=zbd.join("reader.zbd");
    if global.exists(){sources.push((global,"global-base".into()))}
    if let Some(parent)=mission.parent(){let campaign=parent.join("reader.zbd");if campaign.exists(){sources.push((campaign,"campaign-base".into()))}}
    sources.push((mission.clone(),"mission".into()));
    let mut seen=HashSet::new();
    sources.retain(|(path,_)|seen.insert(path.to_string_lossy().to_ascii_lowercase()));
    let mut archives=Vec::new();
    for(path,layer)in sources{
        let archive=storage::load_archive(&path).map_err(|error|format!("Could not load {}: {error:#}",path.display()))?;
        let relative_path=path.strip_prefix(&root).unwrap_or(&path).to_owned();
        archives.push(WorkspaceArchiveState{id:format!("a{}",archives.len()),relative_path,layer,archive});
    }
    let session=WorkspaceSession{root_path:root.clone(),mission_path:mission.clone(),archives};
    let snapshot=workspace_snapshot(0,&session);
    let mut channels:HashMap<String,(Vec<ReadonlyChannelEndpoint>,Vec<ReadonlyChannelEndpoint>)>=HashMap::new();
    let mut endpoint_ids=HashSet::new();
    for node in &snapshot.valve_graph.nodes{
        for port in &node.ports{
            if port.channel.is_empty()||!endpoint_ids.insert(port.id.clone()){continue}
            let endpoint=ReadonlyChannelEndpoint{
                id:port.id.clone(),kind:node.kind.clone(),label:node.label.clone(),
                detail:if node.subtitle.is_empty(){port.label.clone()}else{format!("{} · {}",port.label,node.subtitle)},
                layer:node.layer.clone(),archive_relative_path:node.binding.archive_relative_path.clone(),
                entry_name:node.binding.entry_name.clone(),locator:node.binding.locator.clone(),value:port.value,
            };
            let entry=channels.entry(port.channel.clone()).or_default();
            if port.direction=="read"{entry.0.push(endpoint)}else if port.direction=="write"{entry.1.push(endpoint)}
        }
    }
    let mut channels:Vec<_>=channels.into_iter().map(|(name,(mut consumers,mut producers))|{
        consumers.sort_by(|a,b|a.label.to_ascii_lowercase().cmp(&b.label.to_ascii_lowercase()).then(a.id.cmp(&b.id)));
        producers.sort_by(|a,b|a.label.to_ascii_lowercase().cmp(&b.label.to_ascii_lowercase()).then(a.id.cmp(&b.id)));
        ReadonlyChannelIndex{name,consumers,producers}
    }).collect();
    channels.sort_by(|a,b|a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()).then(a.name.cmp(&b.name)));
    let warnings=snapshot.valve_graph.diagnostics.iter().filter(|item|item.severity!="info").map(|item|item.message.clone()).collect();
    Ok(ReadonlyMissionIndex{root_path,selected_mission:mission.to_string_lossy().into(),channels,warnings})
}

#[tauri::command]fn open_workspace(root_path:String,mission_path:String,additional_paths:Option<Vec<String>>,state:tauri::State<AppState>)->Result<WorkspaceSnapshot,String>{let root=PathBuf::from(&root_path);let mission=PathBuf::from(&mission_path);let zbd=root.join("zbd");let mut sources=Vec::<(PathBuf,String)>::new();let global=zbd.join("reader.zbd");if global.exists(){sources.push((global,"global-base".into()))}if let Some(parent)=mission.parent(){let campaign=parent.join("reader.zbd");if campaign.exists(){sources.push((campaign,"campaign-base".into()))}}sources.push((mission.clone(),"mission".into()));for path in additional_paths.unwrap_or_default(){sources.push((PathBuf::from(path),"attached".into()))}let mut seen=HashSet::new();sources.retain(|(p,_)|seen.insert(p.to_string_lossy().to_ascii_lowercase()));let mut archives=Vec::new();for(path,layer)in sources{let archive=storage::load_archive(&path).map_err(|e|format!("Could not load {}: {e:#}",path.display()))?;let relative=path.strip_prefix(&root).unwrap_or(&path).to_owned();let id=format!("a{}",archives.len());archives.push(WorkspaceArchiveState{id,relative_path:relative,layer,archive})}let id=state.next.fetch_add(1,Ordering::Relaxed)+1;let session=WorkspaceSession{root_path:root,mission_path:mission,archives};let snapshot=workspace_snapshot(id,&session);state.workspaces.lock().unwrap().insert(id,session);Ok(snapshot)}

#[tauri::command]fn refresh_workspace(snapshot:WorkspaceSnapshot,state:tauri::State<AppState>)->Result<WorkspaceSnapshot,String>{let sessions=state.workspaces.lock().unwrap();let session=sessions.get(&snapshot.session_id).ok_or("Workspace session expired")?;let(mut refs,mut diagnostics)=build_index(&snapshot.documents);for a in &snapshot.archives{if a.locked{for d in snapshot.documents.iter().filter(|d|d.archive_id==a.id){if let Some(original)=session.archives.iter().find(|x|x.id==a.id).and_then(|x|x.archive.entries.iter().find(|e|e.name==d.entry_name)).and_then(|e|e.root.as_ref()){if original!=&d.root{diagnostics.push(Diagnostic{severity:"warning".into(),code:"locked-document-changed".into(),message:format!("{} is changed but its archive is locked",d.entry_name),document_id:Some(d.id.clone()),location:None})}}}}}refs.sort_by(|a,b|a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()).then(a.role.cmp(&b.role)));let valve_graph=valve_graph::build_valve_graph(&snapshot.documents,&snapshot.archives,&snapshot.selected_mission);Ok(WorkspaceSnapshot{references:refs,diagnostics,valve_graph,..snapshot})}

#[tauri::command]fn apply_valve_graph_edits(mut snapshot:WorkspaceSnapshot,edits:Vec<valve_graph::ValveGraphEdit>,state:tauri::State<AppState>)->Result<WorkspaceSnapshot,String>{if !state.workspaces.lock().unwrap().contains_key(&snapshot.session_id){return Err("Workspace session expired".into())}valve_graph::apply_edits(&mut snapshot,&edits)?;let(references,diagnostics)=build_index(&snapshot.documents);snapshot.references=references;snapshot.diagnostics=diagnostics;Ok(snapshot)}

#[tauri::command]fn export_workspace(export_root:String,snapshot:WorkspaceSnapshot,state:tauri::State<AppState>)->Result<ExportResult,String>{valve_graph::validate_for_export(&snapshot)?;let sessions=state.workspaces.lock().unwrap();let session=sessions.get(&snapshot.session_id).ok_or("Workspace session expired")?;let lock_map:HashMap<_,_>=snapshot.archives.iter().map(|a|(a.id.as_str(),a.locked)).collect();let mut files=Vec::new();let mut changed_documents=0;for stored in &session.archives{let mut roots=HashMap::new();for doc in snapshot.documents.iter().filter(|d|d.archive_id==stored.id){let Some(entry)=stored.archive.entries.iter().find(|e|e.name==doc.entry_name)else{continue};if entry.root.as_ref()!=Some(&doc.root){if *lock_map.get(stored.id.as_str()).unwrap_or(&true){return Err(format!("{} is locked; unlock it before export",stored.relative_path.display()))}roots.insert(doc.entry_name.clone(),doc.root.clone());changed_documents+=1}}if roots.is_empty(){continue}let output=PathBuf::from(&export_root).join(&stored.relative_path);storage::write_archive_with_roots(&output,&stored.archive,&roots).map_err(|e|format!("Could not export {}: {e:#}",output.display()))?;let verify=storage::load_archive(&output).map_err(|e|format!("Could not verify {}: {e:#}",output.display()))?;for(name,root)in &roots{let actual=verify.entries.iter().find(|e|e.name==*name).and_then(|e|e.root.as_ref()).ok_or_else(||format!("Verification did not find {name}"))?;if actual!=root{return Err(format!("Verification mismatch in {} / {name}",output.display()))}}files.push(output.to_string_lossy().into())}Ok(ExportResult{files,changed_documents})}

#[tauri::command]fn open_document(path:String,state:tauri::State<AppState>)->Result<DocumentSnapshot,String>{let loaded=storage::load(Path::new(&path)).map_err(|e|format!("Could not open file: {e:#}"))?;let vehicles=decode_vehicles(&loaded.root).map_err(|e|e.to_string())?;let id=state.next.fetch_add(1,Ordering::Relaxed)+1;let mut warnings=loaded.warnings;if vehicle_list(&loaded.root).is_ok_and(|a|a.len()%2==1){warnings.push("The aiv vehicle list contains one trailing non-vehicle value. It will be preserved unchanged.".into())}let snap=DocumentSnapshot{session_id:id,source_path:path,source_kind:loaded.container.kind().into(),archive_entry:loaded.container.entry_name(),vehicles,linked_valves:loaded.linked_valves,linked_nets:loaded.linked_nets,warnings};state.sessions.lock().unwrap().insert(id,Session{root:loaded.root,container:loaded.container});Ok(snap)}
#[tauri::command]fn save_document(path:String,snapshot:DocumentSnapshot,state:tauri::State<AppState>)->Result<(),String>{let mut sessions=state.sessions.lock().unwrap();let session=sessions.get_mut(&snapshot.session_id).ok_or("Editor session expired")?;session.root=encode_vehicles_like(&snapshot.vehicles,&session.root);storage::save(Path::new(&path),&session.root,&session.container).map_err(|e|format!("Could not save file: {e:#}"))}
fn campaign_anim_path(mission_path:&str)->Result<PathBuf,String>{Path::new(mission_path).parent().map(|dir|dir.join("anim.zbd")).filter(|path|path.is_file()).ok_or_else(||format!("No anim.zbd next to {mission_path}"))}
fn animation_key(path:&Path)->String{path.to_string_lossy().to_ascii_lowercase()}
// Async so the multi-second anim.zbd parse does not block the UI thread.
#[tauri::command]async fn load_animation_index(root_path:String,mission_path:String,state:tauri::State<'_,AppState>)->Result<animation::AnimationIndex,String>{let path=campaign_anim_path(&mission_path)?;let key=animation_key(&path);let mut cache=state.animations.lock().unwrap();if !cache.contains_key(&key){let loaded=animation::read_anim_file(&path)?;cache.insert(key.clone(),loaded);}let relative=path.strip_prefix(&root_path).map(rel_string).unwrap_or_else(|_|rel_string(&path));Ok(animation::build_index(&path,relative,&cache[&key]))}
#[tauri::command]async fn load_animation_definition(path:String,index:usize,state:tauri::State<'_,AppState>)->Result<Value,String>{let path=PathBuf::from(path);let key=animation_key(&path);let mut cache=state.animations.lock().unwrap();if !cache.contains_key(&key){let loaded=animation::read_anim_file(&path)?;cache.insert(key.clone(),loaded);}cache[&key].definitions.get(index).map(|(_,definition)|definition.clone()).ok_or_else(||format!("Animation definition {index} is not in {}",path.display()))}
#[cfg_attr(mobile,tauri::mobile_entry_point)]pub fn run(){tauri::Builder::default().plugin(tauri_plugin_dialog::init()).manage(AppState::default()).invoke_handler(tauri::generate_handler![scan_game_root,open_workspace,refresh_workspace,apply_valve_graph_edits,export_workspace,open_document,save_document,load_ai_layout,save_ai_layout,load_animation_index,load_animation_definition]).run(tauri::generate_context!()).expect("error while running application")}

#[cfg(test)]mod tests{use super::*;#[test]fn duplicate_names_roundtrip(){let root=serde_json::json!(["A",[1,[2,3,4],5,6,"mystery",[7]],"A",[8,[9,10,11],12,13]]);let v=decode_vehicles(&root).unwrap();assert_eq!(encode_vehicles_like(&v,&root),root);assert_ne!(v[0].id,v[1].id)}#[test]fn wrapped_root_and_trailer_are_preserved(){let root=serde_json::json!([["A",[1,[2,3,4],5,6],"trailer"]]);let v=decode_vehicles(&root).unwrap();assert_eq!(v.len(),1);assert_eq!(encode_vehicles_like(&v,&root),root)}#[test]fn cross_document_reference_index(){let aiv=WorkspaceDocument{id:"a".into(),archive_id:"m".into(),entry_name:"aiv.zrd".into(),kind:"aiv".into(),layer:"mission".into(),root:serde_json::json!([["player",[0,[0,0,0],0,1,"vlist",[[1,0,0,0,0,0,0,0,1,"reg_test",0,0,0,0,1,"begin",0]]]]])};let reg=WorkspaceDocument{id:"r".into(),archive_id:"m".into(),entry_name:"reg_test.zrd".into(),kind:"region".into(),layer:"mission".into(),root:serde_json::json!([["version",[108],"name",["reg_test"]]])};let(refs,diagnostics)=build_index(&[aiv,reg]);assert!(refs.iter().any(|r|r.name=="begin"&&r.role=="writes-valve"));assert!(!diagnostics.iter().any(|d|d.code=="missing-region"))}}

#[cfg(test)]
mod installed_game_tests {
    use super::*;

    fn installed_workspace(root: &Path, mission: PathBuf) -> Option<WorkspaceSnapshot> {
        if !mission.exists() { return None }
        let mut archives = Vec::new();
        for (path, layer) in [
            (root.join("zbd").join("reader.zbd"), "global-base"),
            (mission.parent()?.join("reader.zbd"), "campaign-base"),
            (mission.clone(), "mission"),
        ] {
            if !path.exists() { continue }
            let archive = storage::load_archive(&path).expect("load Reader layer");
            let relative_path = path.strip_prefix(root).unwrap().to_owned();
            archives.push(WorkspaceArchiveState {
                id: format!("a{}", archives.len()),
                relative_path,
                layer: layer.into(),
                archive,
            });
        }
        let session = WorkspaceSession { root_path: root.to_owned(), mission_path: mission, archives };
        Some(workspace_snapshot(1, &session))
    }

    #[test]
    fn layered_reader_workspace_builds_from_installed_game() {
        let Ok(root_value) = std::env::var("MW3_TEST_ROOT") else { return };
        let root = PathBuf::from(root_value);
        let Some(snapshot) = installed_workspace(&root, root.join("zbd").join("c1").join("readerm1.zbd")) else { return };
        assert!(snapshot.documents.iter().any(|d| d.kind == "aiv"));
        assert!(snapshot.documents.iter().any(|d| d.kind == "valves"));
        assert!(snapshot.documents.iter().any(|d| d.kind == "network"));
        assert!(snapshot.documents.iter().any(|d| d.kind == "region"));
        assert!(snapshot.references.iter().any(|r| r.target_kind == "valve"));
        assert!(snapshot.archives.iter().any(|a| a.layer == "mission" && !a.locked));
        assert!(snapshot.archives.iter().filter(|a| a.layer != "mission").all(|a| a.locked));
    }

    #[test]
    fn readonly_dev_index_uses_the_layered_behavior_graph() {
        let Ok(root_value) = std::env::var("MW3_TEST_ROOT") else { return };
        let root = PathBuf::from(root_value);
        let mission = root.join("zbd").join("c1").join("readerm1.zbd");
        if !mission.exists() { return }
        let index = load_readonly_mission_index(
            root.to_string_lossy().into_owned(),
            mission.to_string_lossy().into_owned(),
        ).expect("build read-only mission index");
        assert!(index.channels.iter().any(|channel| !channel.consumers.is_empty()));
        assert!(index.channels.iter().any(|channel| !channel.producers.is_empty()));
        for channel in &index.channels {
            let mut ids = HashSet::new();
            assert!(channel.consumers.iter().chain(&channel.producers).all(|item| ids.insert(&item.id)));
        }
    }

    #[test]
    fn t1m1_installed_graph_includes_objective_assignment() {
        let Ok(root_value) = std::env::var("MW3_TEST_ROOT") else { return };
        let root = PathBuf::from(root_value);
        let Some(snapshot) = installed_workspace(&root, root.join("zbd").join("t1").join("readerm1.zbd")) else { return };
        let mis_end = snapshot
            .valve_graph
            .channels
            .iter()
            .find(|channel| channel.name == "mis_end")
            .expect("mis_end channel");
        assert_eq!(mis_end.producer_count, 1);
        assert!(snapshot.valve_graph.nodes.iter().any(|node| {
            node.kind == "objective-set-valve"
                && node.ports.iter().any(|port| port.channel == "mis_end" && port.value == Some(1))
        }));
        assert!(snapshot.valve_graph.nodes.iter().any(|node| {
            node.kind == "objective-observer"
                && node.ports.iter().any(|port| port.channel == "mis_success" && port.value == Some(1))
        }));
        assert!(!snapshot.valve_graph.nodes.iter().any(|node| {
            node.ports.iter().any(|port| port.channel == "target_fire")
        }));
    }

    #[test]
    fn c1m1_bramp_is_written_by_the_call_ramp_animation_callback() {
        let Ok(root_value) = std::env::var("MW3_TEST_ROOT") else { return };
        let root = PathBuf::from(root_value);
        let Some(snapshot) = installed_workspace(&root, root.join("zbd").join("c1").join("readerm1.zbd")) else { return };
        let producer = snapshot
            .valve_graph
            .nodes
            .iter()
            .find(|node| node.kind == "animation-callback" && node.ports.iter().any(|port| port.channel == "bramp"))
            .expect("bramp animation callback producer");
        assert_eq!(producer.properties["animationName"], "call_ramp");
        assert!(producer.properties["calledBy"]
            .as_array()
            .unwrap()
            .iter()
            .any(|caller| caller.as_str().is_some_and(|text| text.starts_with("start_single_player_m1"))));
        assert!(snapshot.references.iter().any(|r| r.name == "bramp" && r.role == "writes-valve-animation"));
    }

    #[test]
    fn c1m1_truck_go_gate_routes_to_the_destroy_truck_assignments() {
        let Ok(root_value) = std::env::var("MW3_TEST_ROOT") else { return };
        let root = PathBuf::from(root_value);
        let Some(snapshot) = installed_workspace(&root, root.join("zbd").join("c1").join("readerm1.zbd")) else { return };
        let graph = &snapshot.valve_graph;
        let node = |kind: &str, channel: &str| graph.nodes.iter()
            .find(|node| node.kind == kind && node.ports.iter().any(|port| port.channel == channel) && node.subtitle.contains("node_08"))
            .map(|node| node.id.clone());
        let gate = node("net-condition", "truck_go").expect("net_01 node_08 truck_go gate");
        let route = graph.edges.iter()
            .find(|edge| edge.kind.as_deref() == Some("route") && edge.from == gate)
            .expect("route edge from the node_08 gate");
        let assignment = graph.nodes.iter().find(|node| node.id == route.to).unwrap();
        assert_eq!(assignment.kind, "net-assignment");
        assert_eq!(assignment.ports[0].channel, "destroy_truck_01");
        assert!(route.channel.contains("link 8→15"));
    }
}
