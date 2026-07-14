use anyhow::{Context, Result};
use mech3ax_api_types::archive::ArchiveEntry;
use mech3ax_archive::{read_archive,write_archive,Mode,Version};
use mech3ax_common::io_ext::{CountingReader,CountingWriter};
use mech3ax_reader::{read_reader,write_reader};
use serde_json::Value;
use std::{fs::File,io::{BufReader,BufWriter,Cursor},path::Path};

#[derive(Clone)] pub enum Container { Json, Zrd, Zbd{version:Version,manifest:Vec<ArchiveEntry>,entries:Vec<(String,Vec<u8>,usize)>,aiv_index:usize} }
impl Container { pub fn kind(&self)->&'static str{match self{Self::Json=>"json",Self::Zrd=>"zrd",Self::Zbd{..}=>"zbd"}} pub fn entry_name(&self)->Option<String>{match self{Self::Zbd{entries,aiv_index,..}=>Some(entries[*aiv_index].0.clone()),_=>None}} }
pub struct Loaded { pub root:Value,pub container:Container,pub linked_valves:Vec<String>,pub linked_nets:Vec<String>,pub warnings:Vec<String> }

fn parse_zrd(data:Vec<u8>,offset:usize)->Result<Value>{let mut r=CountingReader::new(Cursor::new(data));r.offset=offset;read_reader(&mut r).map_err(|e|anyhow::anyhow!(e))}
fn symbols(root:&Value)->Vec<String>{root.as_array().map(|a|a.iter().step_by(2).filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default()}
pub fn load(path:&Path)->Result<Loaded>{
 let ext=path.extension().and_then(|x|x.to_str()).unwrap_or("").to_ascii_lowercase();
 if ext=="json" {let root=serde_json::from_reader(BufReader::new(File::open(path)?)).context("invalid JSON")?;return Ok(Loaded{root,container:Container::Json,linked_valves:vec![],linked_nets:vec![],warnings:vec![]})}
 if ext=="zrd" {let data=std::fs::read(path)?;let root=parse_zrd(data,0)?;return Ok(Loaded{root,container:Container::Zrd,linked_valves:vec![],linked_nets:vec![],warnings:vec![]})}
 if ext!="zbd" {anyhow::bail!("Expected .json, .zrd, or .zbd")}
 let bytes=std::fs::read(path)?; let mut last=None;
 for version in [Version::One,Version::Two(Mode::Reader)] { let mut entries=Vec::new();let mut r=CountingReader::new(Cursor::new(&bytes)); let result:Result<Vec<ArchiveEntry>>=read_archive(&mut r,|name,data,offset| -> Result<()> {entries.push((name.to_owned(),data,offset));Ok(())},version); match result {Ok(manifest)=>{let aiv_index=entries.iter().position(|(n,_,_)|n.eq_ignore_ascii_case("aiv.zrd")).context("archive has no aiv.zrd")?;let root=parse_zrd(entries[aiv_index].1.clone(),entries[aiv_index].2)?;let mut valves=vec![];let mut nets=vec![];for (name,data,offset) in &entries {if name.eq_ignore_ascii_case("valves.zrd"){if let Ok(v)=parse_zrd(data.clone(),*offset){valves=symbols(&v)}}else if name.to_ascii_lowercase().starts_with("net_")&&name.to_ascii_lowercase().ends_with(".zrd"){if let Ok(v)=parse_zrd(data.clone(),*offset){nets.extend(symbols(&v))}}}return Ok(Loaded{root,container:Container::Zbd{version,manifest,entries,aiv_index},linked_valves:valves,linked_nets:nets,warnings:vec![]})},Err(e)=>last=Some(e)} }
 Err(last.unwrap_or_else(||anyhow::anyhow!("unsupported archive")))
}
pub fn save(path:&Path,root:&Value,container:&Container)->Result<()> {match container {
 Container::Json=>{serde_json::to_writer_pretty(BufWriter::new(File::create(path)?),root)?;Ok(())},
 Container::Zrd=>{let mut w=CountingWriter::new(BufWriter::new(File::create(path)?),0);write_reader(&mut w,root).map_err(|e|anyhow::anyhow!(e))},
 Container::Zbd{version,manifest,entries,aiv_index}=>{let mut output=CountingWriter::new(BufWriter::new(File::create(path)?),0);let result:Result<()>=write_archive(&mut output,manifest,|name,offset| -> Result<Vec<u8>> {let idx=entries.iter().position(|(n,_,_)|n==name).context("missing archive entry")?;if idx==*aiv_index{let mut w=CountingWriter::new(Vec::new(),offset);write_reader(&mut w,root).map_err(|e|anyhow::anyhow!(e))?;Ok(w.into_inner())}else{Ok(entries[idx].1.clone())}},*version);result}
}}

#[cfg(test)] mod tests {
 use super::*;
 #[test] fn actual_reader_archive_roundtrip() {
  let Ok(source)=std::env::var("MW3_TEST_READER") else { return };
  let loaded=load(Path::new(&source)).expect("load real Reader archive");
  assert!(loaded.root.as_array().is_some_and(|x| !x.is_empty()));
  let root_len=loaded.root.as_array().map_or(0,Vec::len); let nested_len=loaded.root.as_array().and_then(|a|a.first()).and_then(Value::as_array).map(Vec::len);
  assert!(!crate::decode_vehicles(&loaded.root).expect("decode aiv vehicle pairs").is_empty(),"root_len={root_len}, nested_len={nested_len:?}");
  let output=std::env::temp_dir().join("mech3-ai-editor-roundtrip.zbd");
  save(&output,&loaded.root,&loaded.container).expect("save archive copy");
  let second=load(&output).expect("read saved archive");
  assert_eq!(second.root,loaded.root);
  std::fs::remove_file(output).ok();
 }
}
