use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{atomic::{AtomicBool,Ordering},Arc,Mutex},
    thread,
    time::{Duration,SystemTime,UNIX_EPOCH},
};

const MAX_OBSERVATIONS:usize=4096;

#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct TraceObservation{
    pub channel:String,
    pub kind:String,
    pub address:u32,
    pub module_offset:Option<u32>,
    pub hit_count:u64,
    pub last_seen_ms:u64,
}

#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct TraceStatus{
    pub active:bool,
    pub pid:Option<u32>,
    pub observation_count:usize,
    pub getter_address:Option<u32>,
    pub message:String,
}

#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct TraceSnapshot{pub status:TraceStatus,pub observations:Vec<TraceObservation>}

struct Inner{status:TraceStatus,observations:HashMap<String,TraceObservation>}

pub struct TraceManager{
    inner:Arc<Mutex<Inner>>,
    stop:Arc<AtomicBool>,
    worker:Mutex<Option<thread::JoinHandle<()>>>,
}

impl Default for TraceManager{
    fn default()->Self{Self{
        inner:Arc::new(Mutex::new(Inner{status:TraceStatus{active:false,pid:None,observation_count:0,getter_address:None,message:"Native trace is disabled.".into()},observations:HashMap::new()})),
        stop:Arc::new(AtomicBool::new(false)),worker:Mutex::new(None),
    }}
}

impl TraceManager{
    pub fn start(&self,pid:u32)->Result<TraceStatus,String>{
        self.stop(false);
        self.stop.store(false,Ordering::SeqCst);
        {
            let mut inner=self.inner.lock().unwrap();
            inner.observations.clear();
            inner.status=TraceStatus{active:false,pid:Some(pid),observation_count:0,getter_address:None,message:"Attaching debugger-style hardware trace…".into()};
        }
        let inner=Arc::clone(&self.inner);
        let stop=Arc::clone(&self.stop);
        let (ready_tx,ready_rx)=std::sync::mpsc::channel();
        let worker=thread::spawn(move||platform::run_trace(pid,inner,stop,ready_tx));
        *self.worker.lock().unwrap()=Some(worker);
        match ready_rx.recv_timeout(Duration::from_secs(5)){
            Ok(Ok(()))=>Ok(self.snapshot().status),
            Ok(Err(error))=>{self.stop(false);Err(error)}
            Err(_)=>{self.stop(false);Err("Timed out while attaching the native trace.".into())}
        }
    }

    pub fn stop(&self,clear:bool)->TraceStatus{
        self.stop.store(true,Ordering::SeqCst);
        if let Some(worker)=self.worker.lock().unwrap().take(){let _=worker.join();}
        let mut inner=self.inner.lock().unwrap();
        if clear{inner.observations.clear()}
        inner.status.active=false;
        inner.status.pid=None;
        inner.status.observation_count=inner.observations.len();
        inner.status.message=if clear{"Native trace stopped; session observations cleared."}else{"Native trace is disabled."}.into();
        inner.status.clone()
    }

    pub fn snapshot(&self)->TraceSnapshot{
        let inner=self.inner.lock().unwrap();
        let mut observations:Vec<_>=inner.observations.values().cloned().collect();
        observations.sort_by(|a,b|a.channel.to_ascii_lowercase().cmp(&b.channel.to_ascii_lowercase()).then(a.kind.cmp(&b.kind)).then(a.address.cmp(&b.address)));
        TraceSnapshot{status:inner.status.clone(),observations}
    }
}

impl Drop for TraceManager{fn drop(&mut self){let _=self.stop(true);}}

fn epoch_ms()->u64{SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().try_into().unwrap_or(u64::MAX)}

fn add_observation(inner:&Arc<Mutex<Inner>>,channel:String,kind:&str,address:u32){
    let key=format!("{channel}\0{kind}\0{address:08x}");
    let mut state=inner.lock().unwrap();
    if let Some(item)=state.observations.get_mut(&key){item.hit_count=item.hit_count.saturating_add(1);item.last_seen_ms=epoch_ms();return}
    if state.observations.len()>=MAX_OBSERVATIONS{return}
    state.observations.insert(key,TraceObservation{channel,kind:kind.into(),address,module_offset:address.checked_sub(0x0040_0000),hit_count:1,last_seen_ms:epoch_ms()});
    state.status.observation_count=state.observations.len();
}

#[cfg(windows)]
mod platform{
    use super::*;
    use std::{collections::{HashMap,HashSet},ffi::c_void,mem::zeroed};
    use windows_sys::Win32::{
        Foundation::{CloseHandle,DBG_CONTINUE,DBG_EXCEPTION_NOT_HANDLED,ERROR_SEM_TIMEOUT,EXCEPTION_BREAKPOINT,EXCEPTION_SINGLE_STEP,GetLastError,HANDLE},
        System::{
            Diagnostics::Debug::{ContinueDebugEvent,CREATE_PROCESS_DEBUG_EVENT,CREATE_THREAD_DEBUG_EVENT,DEBUG_EVENT,DebugActiveProcess,DebugActiveProcessStop,DebugSetProcessKillOnExit,EXCEPTION_DEBUG_EVENT,EXIT_PROCESS_DEBUG_EVENT,LOAD_DLL_DEBUG_EVENT,ReadProcessMemory,WaitForDebugEvent,Wow64GetThreadContext,Wow64SetThreadContext,WOW64_CONTEXT,WOW64_CONTEXT_ALL},
            Threading::{OpenProcess,OpenThread,PROCESS_QUERY_LIMITED_INFORMATION,PROCESS_VM_READ,THREAD_GET_CONTEXT,THREAD_QUERY_INFORMATION,THREAD_SET_CONTEXT},
        },
    };

    const REGISTRY_HEAD:u32=0x0060_b504;
    const REGISTRY_COUNT:u32=0x0060_b508;
    const COMPARE_ADDRESS:u32=0x004d_4490;
    const NOTIFY_ADDRESS:u32=0x004d_3bf0;
    const MAX_VALVES:usize=32_768;

    struct Handle(HANDLE);
    impl Drop for Handle{fn drop(&mut self){if !self.0.is_null(){unsafe{CloseHandle(self.0);}}}}

    struct RegistryMap{by_object:HashMap<u32,String>,getter:u32}

    pub fn run_trace(pid:u32,inner:Arc<Mutex<Inner>>,stop:Arc<AtomicBool>,ready:std::sync::mpsc::Sender<Result<(),String>>){
        let process=match open_process(pid){Ok(value)=>value,Err(error)=>{let _=ready.send(Err(error));return}};
        let mut registry=match read_registry(process.0){Ok(value)=>value,Err(error)=>{let _=ready.send(Err(error));return}};
        if unsafe{DebugActiveProcess(pid)}==0{let error=format!("Could not attach the Windows debugger trace to PID {pid} (error {}).",unsafe{GetLastError()});let _=ready.send(Err(error));return}
        unsafe{DebugSetProcessKillOnExit(0)};
        {
            let mut state=inner.lock().unwrap();
            state.status.active=true;
            state.status.getter_address=Some(registry.getter);
            state.status.message=format!("Tracing channel reads at 0x{:08X}; comparison and notification probes are active.",registry.getter);
        }
        let _=ready.send(Ok(()));
        let mut thread_ids=HashSet::new();
        let mut pending_compare:HashMap<u32,u32>=HashMap::new();
        let mut last_registry_refresh=std::time::Instant::now();
        let mut process_exited=false;
        while !stop.load(Ordering::SeqCst){
            let mut event:DEBUG_EVENT=unsafe{zeroed()};
            if unsafe{WaitForDebugEvent(&mut event,50)}==0{
                if unsafe{GetLastError()}==ERROR_SEM_TIMEOUT{continue}
                break
            }
            thread_ids.insert(event.dwThreadId);
            let mut continue_status=DBG_CONTINUE;
            if event.dwDebugEventCode==EXCEPTION_DEBUG_EVENT{
                let exception=unsafe{event.u.Exception};
                let code=exception.ExceptionRecord.ExceptionCode;
                if code==EXCEPTION_SINGLE_STEP{
                    if let Ok(mut context)=thread_context(event.dwThreadId){
                        let return_address=read_u32(process.0,context.Esp).unwrap_or(0);
                        if context.Dr6&1!=0||context.Eip==registry.getter{
                            if let Some(channel)=registry.by_object.get(&context.Ecx).cloned(){
                                if let Some(caller)=pending_compare.remove(&event.dwThreadId){add_observation(&inner,channel,"comparison caller",caller)}
                                else{add_observation(&inner,channel,"value reader",return_address)}
                            }
                        }
                        if context.Dr6&2!=0||context.Eip==COMPARE_ADDRESS{pending_compare.insert(event.dwThreadId,return_address);}
                        if context.Dr6&4!=0||context.Eip==NOTIFY_ADDRESS{
                            if let Some(channel)=resolve_channel(&registry,&context,process.0){add_observation(&inner,channel,"notification dispatch",return_address)}
                        }
                        install_breakpoints(&mut context,&registry);
                        let _=set_thread_context(event.dwThreadId,&context);
                    }
                }else if code==EXCEPTION_BREAKPOINT{continue_status=DBG_CONTINUE}else{continue_status=DBG_EXCEPTION_NOT_HANDLED}
            }
            if event.dwDebugEventCode==CREATE_PROCESS_DEBUG_EVENT{let info=unsafe{event.u.CreateProcessInfo};if !info.hFile.is_null(){unsafe{CloseHandle(info.hFile);}}}
            if event.dwDebugEventCode==LOAD_DLL_DEBUG_EVENT{let info=unsafe{event.u.LoadDll};if !info.hFile.is_null(){unsafe{CloseHandle(info.hFile);}}}
            if event.dwDebugEventCode==CREATE_THREAD_DEBUG_EVENT||event.dwDebugEventCode==CREATE_PROCESS_DEBUG_EVENT{let _=configure_thread(event.dwThreadId,&registry);}
            if event.dwDebugEventCode==EXIT_PROCESS_DEBUG_EVENT{process_exited=true;}
            unsafe{ContinueDebugEvent(event.dwProcessId,event.dwThreadId,continue_status)};
            if process_exited{break}
            if last_registry_refresh.elapsed()>=Duration::from_secs(1){if let Ok(next)=read_registry(process.0){registry=next;for id in &thread_ids{let _=configure_thread(*id,&registry);}}last_registry_refresh=std::time::Instant::now();}
        }
        for id in thread_ids{let _=clear_thread(id);}
        if !process_exited{unsafe{DebugActiveProcessStop(pid)};}
        let mut state=inner.lock().unwrap();
        state.status.active=false;
        state.status.message=if process_exited{"Mech3.exe exited; native trace stopped."}else{"Native trace stopped cleanly."}.into();
    }

    fn open_process(pid:u32)->Result<Handle,String>{let handle=unsafe{OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION|PROCESS_VM_READ,0,pid)};if handle.is_null(){Err(format!("Could not open Mech3.exe PID {pid} for tracing."))}else{Ok(Handle(handle))}}
    fn open_thread(id:u32)->Result<Handle,String>{let handle=unsafe{OpenThread(THREAD_GET_CONTEXT|THREAD_SET_CONTEXT|THREAD_QUERY_INFORMATION,0,id)};if handle.is_null(){Err(format!("Could not open traced thread {id}."))}else{Ok(Handle(handle))}}
    fn thread_context(id:u32)->Result<WOW64_CONTEXT,String>{let thread=open_thread(id)?;let mut context:WOW64_CONTEXT=unsafe{zeroed()};context.ContextFlags=WOW64_CONTEXT_ALL;if unsafe{Wow64GetThreadContext(thread.0,&mut context)}==0{Err(format!("Could not read context for traced thread {id}."))}else{Ok(context)}}
    fn set_thread_context(id:u32,context:&WOW64_CONTEXT)->Result<(),String>{let thread=open_thread(id)?;if unsafe{Wow64SetThreadContext(thread.0,context)}==0{Err(format!("Could not update context for traced thread {id}."))}else{Ok(())}}
    fn install_breakpoints(context:&mut WOW64_CONTEXT,registry:&RegistryMap){context.ContextFlags=WOW64_CONTEXT_ALL;context.Dr0=registry.getter;context.Dr1=COMPARE_ADDRESS;context.Dr2=NOTIFY_ADDRESS;context.Dr3=0;context.Dr6=0;context.Dr7=0x15;}
    fn configure_thread(id:u32,registry:&RegistryMap)->Result<(),String>{let mut context=thread_context(id)?;install_breakpoints(&mut context,registry);set_thread_context(id,&context)}
    fn clear_thread(id:u32)->Result<(),String>{let mut context=thread_context(id)?;context.Dr0=0;context.Dr1=0;context.Dr2=0;context.Dr3=0;context.Dr6=0;context.Dr7=0;set_thread_context(id,&context)}
    fn resolve_channel(registry:&RegistryMap,context:&WOW64_CONTEXT,process:HANDLE)->Option<String>{
        for candidate in [context.Ecx,context.Eax,context.Edx,read_u32(process,context.Esp.wrapping_add(4)).ok()?,read_u32(process,context.Esp.wrapping_add(8)).ok()?]{if let Some(name)=registry.by_object.get(&candidate){return Some(name.clone())}}
        None
    }
    fn read_registry(process:HANDLE)->Result<RegistryMap,String>{
        let count=read_u32(process,REGISTRY_COUNT)? as usize;if count==0{return Err("The mission channel registry is empty.".into())}if count>MAX_VALVES{return Err(format!("Channel count {count} exceeds the safe limit."))}
        let sentinel=read_u32(process,REGISTRY_HEAD)?;let mut node=read_u32(process,sentinel)?;let mut by_object=HashMap::new();let mut seen=HashSet::new();
        for _ in 0..count{if node==sentinel||!seen.insert(node){return Err("The channel registry changed while preparing the trace.".into())}let next=read_u32(process,node)?;let record=read_u32(process,node+8)?;let name_ptr=read_u32(process,record)?;let object=read_u32(process,record+4)?;by_object.insert(object,read_string(process,name_ptr)?);node=next;}
        let object=*by_object.keys().next().ok_or("No channel value object was found.")?;let vtable=read_u32(process,object)?;let getter=read_u32(process,vtable+4)?;if !(0x0040_0000..0x0080_0000).contains(&getter){return Err(format!("Unverified channel getter address 0x{getter:08X}."))}
        Ok(RegistryMap{by_object,getter})
    }
    fn read_u32(process:HANDLE,address:u32)->Result<u32,String>{let mut value=0u32;let mut read=0usize;let ok=unsafe{ReadProcessMemory(process,address as *const c_void,&mut value as *mut u32 as *mut c_void,4,&mut read)};if ok==0||read!=4{Err(format!("Could not read Mech3.exe at 0x{address:08X}."))}else{Ok(value)}}
    fn read_string(process:HANDLE,address:u32)->Result<String,String>{let mut bytes=Vec::new();for offset in 0..128{let mut byte=0u8;let mut read=0usize;let ok=unsafe{ReadProcessMemory(process,(address+offset) as *const c_void,&mut byte as *mut u8 as *mut c_void,1,&mut read)};if ok==0||read!=1{return Err(format!("Could not read channel name at 0x{address:08X}."))}if byte==0{break}bytes.push(byte)}String::from_utf8(bytes).map_err(|_|"A live channel name is not UTF-8.".into())}
}

#[cfg(not(windows))]
mod platform{
    use super::*;
    pub fn run_trace(_pid:u32,_inner:Arc<Mutex<Inner>>,_stop:Arc<AtomicBool>,ready:std::sync::mpsc::Sender<Result<(),String>>){let _=ready.send(Err("Native tracing is available only on Windows.".into()));}
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn repeated_observations_are_deduplicated(){let inner=Arc::new(Mutex::new(Inner{status:TraceStatus{active:true,pid:Some(1),observation_count:0,getter_address:Some(1),message:String::new()},observations:HashMap::new()}));add_observation(&inner,"alpha".into(),"value reader",0x401000);add_observation(&inner,"alpha".into(),"value reader",0x401000);let state=inner.lock().unwrap();assert_eq!(state.observations.len(),1);assert_eq!(state.observations.values().next().unwrap().hit_count,2);}
}
