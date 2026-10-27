// Runtime valve-registry monitor and direct writer for the Switchboard app.
use crate::animations::{self, AnimationState};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::Read,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};

const EXPECTED_BUILD: &str = "Mech3.exe 1.2.22";
const EXPECTED_SHA256: &str = "95bc2c114c9b2e5c5ada8e40ca78cc79470f862e45313ffad0d1e8e6d4d916bf";
pub(crate) const IMAGE_BASE: u64 = 0x0040_0000;
const VALVE_LIST_HEAD_RVA: u64 = 0x0060_b504 - IMAGE_BASE;
const VALVE_LIST_COUNT_RVA: u64 = 0x0060_b508 - IMAGE_BASE;
const MAX_VALVES: usize = 32_768;
const MAX_VALVE_NAME_BYTES: usize = 128;
const MAX_ANIMATION_EVENTS: usize = 250;
/// The animation table is read every second worker tick (5 Hz).
const ANIMATION_POLL_TICKS: u64 = 2;
/// Timer and loop-count updates without a state transition are sent at most this often.
const ANIMATION_QUIET_EMIT_MS: u64 = 1000;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessCandidate {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub compatible: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValveState {
    pub name: String,
    pub value: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_at: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorStatus {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
    pub valve_count: usize,
    pub write_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_snapshot_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationTransition {
    pub at: u64,
    pub index: u32,
    pub anim_name: String,
    pub name: String,
    pub from_state: u8,
    pub to_state: u8,
    pub running_instances: u32,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationSnapshot {
    pub revision: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_read_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub animations: Vec<AnimationState>,
    /// Newest first.
    pub events: VecDeque<AnimationTransition>,
    #[serde(skip)]
    last_emit_ms: u64,
}

impl Default for MonitorStatus {
    fn default() -> Self {
        Self {
            state: "disconnected".into(),
            pid: None,
            build: None,
            valve_count: 0,
            write_enabled: false,
            last_snapshot_ms: None,
            message: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchboardSnapshot {
    pub status: MonitorStatus,
    pub valves: Vec<ValveState>,
    pub animations: AnimationSnapshot,
}

#[derive(Clone, Debug)]
struct ModuleInfo {
    base_address: u64,
    path: String,
}

#[derive(Default)]
struct MonitorInner {
    status: MonitorStatus,
    valves: Vec<ValveState>,
    previous_values: HashMap<String, i32>,
    initialized: bool,
    animations: AnimationSnapshot,
}

pub struct RuntimeMonitor {
    inner: Arc<Mutex<MonitorInner>>,
    generation: Arc<AtomicU64>,
}

impl Default for RuntimeMonitor {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(MonitorInner {
                status: MonitorStatus {
                    state: "disconnected".into(),
                    pid: None,
                    build: None,
                    valve_count: 0,
                    write_enabled: false,
                    last_snapshot_ms: None,
                    message: Some("Waiting for a compatible Mech3.exe process.".into()),
                },
                ..Default::default()
            })),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl RuntimeMonitor {
    pub fn status(&self) -> MonitorStatus {
        self.inner.lock().unwrap().status.clone()
    }

    pub fn detach(&self) -> MonitorStatus {
        self.generation.fetch_add(1, Ordering::SeqCst);
        let mut inner = self.inner.lock().unwrap();
        inner.status = MonitorStatus {
            state: "disconnected".into(),
            pid: None,
            build: None,
            valve_count: 0,
            write_enabled: false,
            last_snapshot_ms: None,
            message: Some("Detached. No game memory is being read.".into()),
        };
        inner.valves.clear();
        inner.previous_values.clear();
        inner.initialized = false;
        inner.animations = AnimationSnapshot::default();
        inner.status.clone()
    }

    pub fn set_write_enabled(&self, enabled: bool) -> Result<MonitorStatus, String> {
        let mut inner = self.inner.lock().unwrap();
        if enabled && inner.status.pid.is_none() {
            return Err("Attach to a verified Mech3.exe mission before enabling direct value writes.".into());
        }
        inner.status.write_enabled = enabled;
        inner.status.message = Some(if enabled {
            "Direct value writes enabled for this session. These writes do not invoke game notifications.".into()
        } else {
            "Direct value writes disabled. The switchboard is monitoring only.".into()
        });
        Ok(inner.status.clone())
    }

    pub fn set_valve(&self, name: String, value: i32, expected_value: i32) -> Result<ValveState, String> {
        let pid = {
            let state = self.inner.lock().unwrap();
            if !state.status.write_enabled {
                return Err("Direct value writes are disabled. Enable them explicitly before changing a valve.".into());
            }
            state.status.pid.ok_or_else(|| "No Mech3.exe process is attached.".to_string())?
        };
        let module = inspect_compatible_process(pid)?;
        let actual_value = write_live_valve(pid, module.base_address, &name, expected_value, value)?;
        let changed_at = (actual_value != expected_value).then_some(epoch_ms());
        let mut state = self.inner.lock().unwrap();
        if state.status.pid != Some(pid) {
            return Err("The attached process changed before the write could be recorded.".into());
        }
        if let Some(existing) = state.valves.iter_mut().find(|valve| valve.name == name) {
            existing.value = actual_value;
            existing.changed_at = changed_at;
        }
        state.previous_values.insert(name.clone(), actual_value);
        state.status.message = Some(format!("Wrote {name} = {actual_value}. The next live poll will confirm it."));
        Ok(ValveState { name, value: actual_value, changed_at })
    }

    pub fn attach(&self, pid: u32, app: AppHandle) -> Result<MonitorStatus, String> {
        let module = inspect_compatible_process(pid)?;
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        {
            let mut inner = self.inner.lock().unwrap();
            inner.status = MonitorStatus {
                state: "waitingForRegistry".into(),
                pid: Some(pid),
                build: Some(EXPECTED_BUILD.into()),
                valve_count: 0,
                write_enabled: false,
                last_snapshot_ms: None,
                message: Some("Attached with monitoring only; direct writes remain disabled.".into()),
            };
            inner.valves.clear();
            inner.previous_values.clear();
            inner.initialized = false;
            inner.animations = AnimationSnapshot::default();
        }
        self.start_worker(pid, module.base_address, generation, app);
        Ok(self.status())
    }

    fn start_worker(&self, pid: u32, module_base: u64, generation: u64, app: AppHandle) {
        let inner = Arc::clone(&self.inner);
        let counter = Arc::clone(&self.generation);
        thread::spawn(move || {
            let mut tick = 0_u64;
            while counter.load(Ordering::SeqCst) == generation {
                if tick % ANIMATION_POLL_TICKS == 0 {
                    let result = read_live_animations(pid, module_base);
                    if let Some(snapshot) = update_animations(&inner, result) {
                        let _ = app.emit("switchboard:animations", snapshot);
                    }
                }
                tick += 1;
                match read_live_valves(pid, module_base) {
                    Ok(values) => {
                        let snapshot = update_snapshot(&inner, pid, values);
                        let _ = app.emit("switchboard:snapshot", snapshot);
                    }
                    Err(error) => {
                        let mut state = inner.lock().unwrap();
                        if is_process_exit_error(&error) {
                            state.status = MonitorStatus {
                                state: "disconnected".into(),
                                pid: None,
                                build: None,
                                valve_count: 0,
                                write_enabled: false,
                                last_snapshot_ms: None,
                                message: Some("The attached Mech3.exe process has exited.".into()),
                            };
                            state.valves.clear();
                            state.previous_values.clear();
                            state.initialized = false;
                            state.animations = AnimationSnapshot::default();
                            let _ = app.emit("switchboard:animations", state.animations.clone());
                            break;
                        }
                        state.status.state = "waitingForRegistry".into();
                        state.status.message = Some(error);
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
    }
}

fn update_snapshot(inner: &Arc<Mutex<MonitorInner>>, pid: u32, values: Vec<(String, i32)>) -> SwitchboardSnapshot {
    let now = epoch_ms();
    let mut state = inner.lock().unwrap();
    let was_initialized = state.initialized;
    let mut next_values = HashMap::with_capacity(values.len());
    let mut valves = Vec::with_capacity(values.len());
    for (name, value) in values {
        let changed = was_initialized && state.previous_values.get(&name).is_some_and(|old| *old != value);
        next_values.insert(name.clone(), value);
        valves.push(ValveState {
            name,
            value,
            changed_at: changed.then_some(now),
        });
    }
    state.previous_values = next_values;
    state.initialized = true;
    state.status = MonitorStatus {
        state: "attached".into(),
        pid: Some(pid),
        build: Some(EXPECTED_BUILD.into()),
        valve_count: valves.len(),
        write_enabled: state.status.write_enabled,
        last_snapshot_ms: Some(now),
        message: Some(if state.status.write_enabled { "Reading the live registry; direct value writes are enabled for this session.".into() } else { "Reading the live registry; direct value writes are disabled.".into() }),
    };
    state.valves = valves.clone();
    SwitchboardSnapshot {
        status: state.status.clone(),
        valves,
        // Animations travel on their own event so the 10 Hz valve stream stays small.
        animations: AnimationSnapshot::default(),
    }
}

/// Stores a fresh animation read and returns the snapshot to emit when anything visible changed.
fn update_animations(inner: &Arc<Mutex<MonitorInner>>, result: Result<Vec<AnimationState>, String>) -> Option<AnimationSnapshot> {
    let now = epoch_ms();
    let mut state = inner.lock().unwrap();
    let current = &mut state.animations;
    let mut next = match result {
        Ok(next) => next,
        Err(error) => {
            if current.message.as_deref() == Some(error.as_str()) {
                return None;
            }
            current.message = Some(error);
            current.revision += 1;
            return Some(current.clone());
        }
    };
    // A table with different records (another mission) restarts the log instead of reporting every record.
    let same_table = current.animations.len() == next.len()
        && current.animations.iter().zip(&next).all(|(old, new)| old.index == new.index && old.anim_name == new.anim_name);
    let mut events = Vec::new();
    if same_table {
        for (old, animation) in current.animations.iter().zip(&mut next) {
            if old.transition_key() != animation.transition_key() {
                animation.changed_at = Some(now);
                events.push(AnimationTransition {
                    at: now,
                    index: animation.index,
                    anim_name: animation.anim_name.clone(),
                    name: animation.name.clone(),
                    from_state: old.state,
                    to_state: animation.state,
                    running_instances: animation.running_instances,
                });
            } else {
                animation.changed_at = old.changed_at;
            }
        }
        let quiet = events.is_empty() && current.message.is_none();
        if quiet && (current.animations == next || now.saturating_sub(current.last_emit_ms) < ANIMATION_QUIET_EMIT_MS) {
            current.last_read_ms = Some(now);
            return None;
        }
        for event in events {
            current.events.push_front(event);
        }
        current.events.truncate(MAX_ANIMATION_EVENTS);
    } else {
        current.events.clear();
    }
    current.animations = next;
    current.message = None;
    current.last_read_ms = Some(now);
    current.last_emit_ms = now;
    current.revision += 1;
    Some(current.clone())
}

fn epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn is_process_exit_error(error: &str) -> bool {
    error.starts_with("Could not open Mech3.exe") || error.starts_with("The Mech3.exe process is no longer available")
}

pub fn list_candidates() -> Result<Vec<ProcessCandidate>, String> {
    let mut candidates = enumerate_mech3_processes()?;
    candidates.sort_by_key(|candidate| candidate.pid);
    Ok(candidates)
}

pub fn snapshot(monitor: &RuntimeMonitor) -> SwitchboardSnapshot {
    let state = monitor.inner.lock().unwrap();
    SwitchboardSnapshot {
        status: state.status.clone(),
        valves: state.valves.clone(),
        animations: state.animations.clone(),
    }
}

fn inspect_compatible_process(pid: u32) -> Result<ModuleInfo, String> {
    let module = module_for_process(pid)?;
    let hash = sha256_file(Path::new(&module.path))?;
    if !is_supported_hash(&hash) {
        return Err(format!(
            "Mech3.exe at {} is not the supported {} build (SHA-256 mismatch).",
            module.path, EXPECTED_BUILD
        ));
    }
    validate_module_layout(&module)?;
    Ok(module)
}

fn is_supported_hash(hash: &str) -> bool {
    hash.eq_ignore_ascii_case(EXPECTED_SHA256)
}

fn validate_module_layout(module: &ModuleInfo) -> Result<(), String> {
    if module.base_address != IMAGE_BASE {
        return Err(format!(
            "Mech3.exe is loaded at 0x{:08x}; the supported {} profile requires 0x{IMAGE_BASE:08x}.",
            module.base_address, EXPECTED_BUILD
        ));
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub(crate) trait MemoryReader {
    fn read_bytes(&self, address: u64, length: usize) -> Result<Vec<u8>, String>;
}

trait MemoryWriter: MemoryReader {
    fn write_bytes(&self, address: u64, bytes: &[u8]) -> Result<(), String>;
}

#[derive(Clone, Copy)]
struct RegistryLayout {
    list_head_pointer: u64,
    count_pointer: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LiveValve {
    name: String,
    value: i32,
    value_address: u64,
}

fn registry_layout(module_base: u64) -> RegistryLayout {
    RegistryLayout {
        list_head_pointer: module_base + VALVE_LIST_HEAD_RVA,
        count_pointer: module_base + VALVE_LIST_COUNT_RVA,
    }
}

fn read_live_animations(pid: u32, module_base: u64) -> Result<Vec<AnimationState>, String> {
    let memory = ProcessMemory::open(pid)?;
    animations::read_animation_table(&memory, module_base)
}

fn read_live_valves(pid: u32, module_base: u64) -> Result<Vec<(String, i32)>, String> {
    let memory = ProcessMemory::open(pid)?;
    Ok(read_stable_registry(&memory, registry_layout(module_base))?
        .into_iter()
        .map(|valve| (valve.name, valve.value))
        .collect())
}

fn read_stable_registry<M: MemoryReader>(memory: &M, layout: RegistryLayout) -> Result<Vec<LiveValve>, String> {
    let first = enumerate_registry_entries(memory, layout)?;
    let second = enumerate_registry_entries(memory, layout)?;
    if first != second {
        return Err("The valve registry changed while it was being read; retrying.".into());
    }
    Ok(first)
}

#[cfg(test)]
fn enumerate_registry<M: MemoryReader>(memory: &M, layout: RegistryLayout) -> Result<Vec<(String, i32)>, String> {
    Ok(enumerate_registry_entries(memory, layout)?
        .into_iter()
        .map(|valve| (valve.name, valve.value))
        .collect())
}

fn enumerate_registry_entries<M: MemoryReader>(memory: &M, layout: RegistryLayout) -> Result<Vec<LiveValve>, String> {
    let count = read_u32(memory, layout.count_pointer)? as usize;
    if count > MAX_VALVES {
        return Err(format!("Valve registry count {count} exceeds the safe limit {MAX_VALVES}."));
    }
    let sentinel = read_pointer(memory, layout.list_head_pointer)?;
    if count == 0 {
        return Ok(Vec::new());
    }
    validate_pointer(sentinel, "valve registry sentinel")?;
    let first_node = read_pointer(memory, sentinel)?;
    validate_pointer(first_node, "first valve node")?;

    let mut node = first_node;
    let mut seen = HashSet::with_capacity(count);
    let mut valves = Vec::with_capacity(count);
    for _ in 0..count {
        validate_pointer(node, "valve list node")?;
        if node == sentinel {
            return Err("Valve registry ended before its recorded count.".into());
        }
        if !seen.insert(node) {
            return Err("Valve registry contains a cycle before its recorded count.".into());
        }
        let next = read_pointer(memory, node)?;
        let record = read_pointer(memory, node + 8)?;
        validate_pointer(record, "valve record")?;
        let name_pointer = read_pointer(memory, record)?;
        let value_object = read_pointer(memory, record + 4)?;
        validate_pointer(name_pointer, "valve name")?;
        validate_pointer(value_object, "valve value object")?;
        let name = read_c_string(memory, name_pointer)?;
        let value = read_i32(memory, value_object + 4)?;
        valves.push(LiveValve { name, value, value_address: value_object + 4 });
        node = next;
    }
    if node != sentinel {
        return Err("Valve registry has more nodes than its recorded count.".into());
    }
    let final_count = read_u32(memory, layout.count_pointer)? as usize;
    let final_sentinel = read_pointer(memory, layout.list_head_pointer)?;
    if final_count != count || final_sentinel != sentinel {
        return Err("The valve registry changed while it was being read; retrying.".into());
    }
    valves.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()).then(a.name.cmp(&b.name)));
    Ok(valves)
}

pub(crate) fn validate_pointer(pointer: u64, label: &str) -> Result<(), String> {
    if !(0x0001_0000..0x8000_0000).contains(&pointer) {
        return Err(format!("Invalid {label} pointer 0x{pointer:08x}."));
    }
    Ok(())
}

pub(crate) fn read_pointer<M: MemoryReader>(memory: &M, address: u64) -> Result<u64, String> {
    Ok(read_u32(memory, address)? as u64)
}

fn read_u32<M: MemoryReader>(memory: &M, address: u64) -> Result<u32, String> {
    let bytes = memory.read_bytes(address, 4)?;
    Ok(u32::from_le_bytes(bytes.try_into().map_err(|_| format!("Could not read 4 bytes at 0x{address:08x}."))?))
}

fn read_i32<M: MemoryReader>(memory: &M, address: u64) -> Result<i32, String> {
    Ok(read_u32(memory, address)? as i32)
}

fn write_i32<M: MemoryWriter>(memory: &M, address: u64, value: i32) -> Result<(), String> {
    memory.write_bytes(address, &value.to_le_bytes())
}

fn write_live_valve(pid: u32, module_base: u64, name: &str, expected_value: i32, value: i32) -> Result<i32, String> {
    let memory = ProcessMemory::open_for_write(pid)?;
    let valves = read_stable_registry(&memory, registry_layout(module_base))?;
    let valve = valves.into_iter().find(|valve| valve.name == name)
        .ok_or_else(|| format!("Valve '{name}' is not currently registered by this mission."))?;
    if valve.value != expected_value {
        return Err(format!("Valve '{name}' changed from {expected_value} to {} before the write. Refresh and try again.", valve.value));
    }
    write_i32(&memory, valve.value_address, value)?;
    let confirmed = read_i32(&memory, valve.value_address)?;
    if confirmed != value {
        return Err(format!("Mech3.exe did not retain the requested value for '{name}'."));
    }
    Ok(confirmed)
}

fn read_c_string<M: MemoryReader>(memory: &M, address: u64) -> Result<String, String> {
    let mut bytes = Vec::with_capacity(MAX_VALVE_NAME_BYTES);
    for offset in 0..MAX_VALVE_NAME_BYTES {
        let byte = memory.read_bytes(address + offset as u64, 1)?[0];
        if byte == 0 {
            break;
        }
        bytes.push(byte);
    }
    if bytes.len() == MAX_VALVE_NAME_BYTES {
        return Err(format!("Valve name at 0x{address:08x} exceeds {MAX_VALVE_NAME_BYTES} bytes."));
    }
    if bytes.is_empty() {
        return Err(format!("Valve name at 0x{address:08x} is empty."));
    }
    let name = std::str::from_utf8(&bytes).map_err(|_| format!("Valve name at 0x{address:08x} is not UTF-8."))?;
    if !name.bytes().all(|byte| byte.is_ascii_graphic() || byte == b' ') {
        return Err(format!("Valve name '{name}' contains unsupported control characters."));
    }
    Ok(name.into())
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::{ffi::c_void, mem::zeroed};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, GetLastError, INVALID_HANDLE_VALUE},
        System::{
            Diagnostics::{
                Debug::{ReadProcessMemory, WriteProcessMemory},
                ToolHelp::{CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, Process32FirstW, Process32NextW, MODULEENTRY32W, PROCESSENTRY32W, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS},
            },
            Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE},
        },
    };

    pub struct ProcessMemory(*mut c_void);

    impl ProcessMemory {
        pub fn open(pid: u32) -> Result<Self, String> {
            Self::open_with_access(pid, PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ)
        }

        pub fn open_for_write(pid: u32) -> Result<Self, String> {
            Self::open_with_access(pid, PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION)
        }

        fn open_with_access(pid: u32, access: u32) -> Result<Self, String> {
            let handle = unsafe { OpenProcess(access, 0, pid) };
            if handle.is_null() {
                return Err(format!("Could not open Mech3.exe process {pid} with the requested access."));
            }
            Ok(Self(handle))
        }
    }

    impl Drop for ProcessMemory {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    impl MemoryReader for ProcessMemory {
        fn read_bytes(&self, address: u64, length: usize) -> Result<Vec<u8>, String> {
            let mut bytes = vec![0_u8; length];
            let mut read = 0_usize;
            let ok = unsafe {
                ReadProcessMemory(
                    self.0,
                    address as *const c_void,
                    bytes.as_mut_ptr() as *mut c_void,
                    length,
                    &mut read,
                )
            };
            if ok == 0 || read != length {
                return Err(format!("Could not read {length} bytes from Mech3.exe at 0x{address:08x}."));
            }
            Ok(bytes)
        }
    }

    impl MemoryWriter for ProcessMemory {
        fn write_bytes(&self, address: u64, bytes: &[u8]) -> Result<(), String> {
            let mut written = 0_usize;
            let ok = unsafe {
                WriteProcessMemory(
                    self.0,
                    address as *const c_void,
                    bytes.as_ptr() as *const c_void,
                    bytes.len(),
                    &mut written,
                )
            };
            if ok == 0 || written != bytes.len() {
                return Err(format!("Could not write {} bytes to Mech3.exe at 0x{address:08x}.", bytes.len()));
            }
            Ok(())
        }
    }

    pub fn enumerate_mech3_processes() -> Result<Vec<ProcessCandidate>, String> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return Err("Could not enumerate Windows processes.".into());
        }
        let mut entry: PROCESSENTRY32W = unsafe { zeroed() };
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut candidates = Vec::new();
        let mut found = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
        while found {
            let name = wide_string(&entry.szExeFile);
            if name.eq_ignore_ascii_case("Mech3.exe") {
                let pid = entry.th32ProcessID;
                match module_for_process(pid) {
                    Ok(module) => {
                        let path = Some(module.path.clone());
                        match validate_module_layout(&module) {
                            Err(error) => candidates.push(ProcessCandidate { pid, name, path, compatible: false, detail: error }),
                            Ok(()) => match sha256_file(Path::new(&module.path)) {
                                Ok(hash) if is_supported_hash(&hash) => candidates.push(ProcessCandidate { pid, name, path, compatible: true, detail: format!("Supported {EXPECTED_BUILD} build.") }),
                                Ok(_) => candidates.push(ProcessCandidate { pid, name, path, compatible: false, detail: format!("Unsupported Mech3.exe build; only {EXPECTED_BUILD} is enabled.") }),
                                Err(error) => candidates.push(ProcessCandidate { pid, name, path, compatible: false, detail: error }),
                            },
                        }
                    }
                    Err(error) => candidates.push(ProcessCandidate { pid, name, path: None, compatible: false, detail: error }),
                }
            }
            found = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
        }
        unsafe { CloseHandle(snapshot) };
        Ok(candidates)
    }

    pub fn module_for_process(pid: u32) -> Result<ModuleInfo, String> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) };
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(format!("Could not query modules for Mech3.exe PID {pid} (Windows error {}).", unsafe { GetLastError() }));
        }
        let mut entry: MODULEENTRY32W = unsafe { zeroed() };
        entry.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;
        let mut found = unsafe { Module32FirstW(snapshot, &mut entry) } != 0;
        let mut module = None;
        while found {
            if wide_string(&entry.szModule).eq_ignore_ascii_case("Mech3.exe") {
                module = Some(ModuleInfo { base_address: entry.modBaseAddr as usize as u64, path: wide_string(&entry.szExePath) });
                break;
            }
            found = unsafe { Module32NextW(snapshot, &mut entry) } != 0;
        }
        unsafe { CloseHandle(snapshot) };
        module.ok_or_else(|| format!("Could not find the Mech3.exe module in PID {pid}."))
    }

    fn wide_string(input: &[u16]) -> String {
        let length = input.iter().position(|value| *value == 0).unwrap_or(input.len());
        String::from_utf16_lossy(&input[..length])
    }
}

#[cfg(not(windows))]
mod platform {
    use super::*;
    pub struct ProcessMemory;
    impl ProcessMemory {
        pub fn open(_pid: u32) -> Result<Self, String> { Err("The Valve Switchboard is Windows-only.".into()) }
    }
    impl MemoryReader for ProcessMemory {
        fn read_bytes(&self, _address: u64, _length: usize) -> Result<Vec<u8>, String> { Err("The Valve Switchboard is Windows-only.".into()) }
    }
    impl MemoryWriter for ProcessMemory {
        fn write_bytes(&self, _address: u64, _bytes: &[u8]) -> Result<(), String> { Err("The Valve Switchboard is Windows-only.".into()) }
    }
    pub fn enumerate_mech3_processes() -> Result<Vec<ProcessCandidate>, String> { Ok(Vec::new()) }
    pub fn module_for_process(_pid: u32) -> Result<ModuleInfo, String> { Err("The Valve Switchboard is Windows-only.".into()) }
}

use platform::{enumerate_mech3_processes, module_for_process, ProcessMemory};

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[derive(Default)]
    struct MockMemory { bytes: HashMap<u64, u8> }
    impl MockMemory {
        fn put(&mut self, address: u64, bytes: &[u8]) { for (offset, byte) in bytes.iter().enumerate() { self.bytes.insert(address + offset as u64, *byte); } }
        fn put_u32(&mut self, address: u64, value: u32) { self.put(address, &value.to_le_bytes()); }
        fn put_string(&mut self, address: u64, value: &str) { self.put(address, value.as_bytes()); self.put(address + value.len() as u64, &[0]); }
    }
    impl MemoryReader for MockMemory {
        fn read_bytes(&self, address: u64, length: usize) -> Result<Vec<u8>, String> { (0..length).map(|offset| self.bytes.get(&(address + offset as u64)).copied().ok_or_else(|| format!("Missing mock memory at 0x{:08x}", address + offset as u64))).collect() }
    }
    struct ChangingCountMemory { memory: MockMemory, count_reads: Cell<usize> }
    impl MemoryReader for ChangingCountMemory {
        fn read_bytes(&self, address: u64, length: usize) -> Result<Vec<u8>, String> {
            if address == 0x20004 && length == 4 {
                let reads = self.count_reads.get();
                self.count_reads.set(reads + 1);
                if reads > 0 { return Ok(2_u32.to_le_bytes().to_vec()); }
            }
            self.memory.read_bytes(address, length)
        }
    }
    fn layout() -> RegistryLayout { RegistryLayout { list_head_pointer: 0x20000, count_pointer: 0x20004 } }
    fn add_valve(memory: &mut MockMemory, node: u64, next: u64, record: u64, name: u64, value_object: u64, valve_name: &str, value: i32) {
        memory.put_u32(node, next as u32); memory.put_u32(node + 8, record as u32); memory.put_u32(record, name as u32); memory.put_u32(record + 4, value_object as u32); memory.put_u32(value_object + 4, value as u32); memory.put_string(name, valve_name);
    }
    #[test]
    fn reads_and_sorts_live_valves() {
        let mut memory = MockMemory::default();
        memory.put_u32(0x20000, 0x21000); memory.put_u32(0x20004, 2); memory.put_u32(0x21000, 0x22000);
        add_valve(&mut memory, 0x22000, 0x23000, 0x24000, 0x25000, 0x26000, "zeta", 3);
        add_valve(&mut memory, 0x23000, 0x21000, 0x27000, 0x28000, 0x29000, "alpha", 1);
        assert_eq!(enumerate_registry(&memory, layout()).unwrap(), vec![("alpha".into(), 1), ("zeta".into(), 3)]);
    }
    #[test]
    fn rejects_cycles_and_malformed_names() {
        let mut memory = MockMemory::default();
        memory.put_u32(0x20000, 0x21000); memory.put_u32(0x20004, 2); memory.put_u32(0x21000, 0x22000);
        add_valve(&mut memory, 0x22000, 0x22000, 0x24000, 0x25000, 0x26000, "loop", 1);
        assert!(enumerate_registry(&memory, layout()).unwrap_err().contains("cycle"));
        memory.put_u32(0x20004, 1); memory.put_u32(0x22000, 0x21000); memory.put(0x25000, &[b'x'; MAX_VALVE_NAME_BYTES]);
        assert!(enumerate_registry(&memory, layout()).unwrap_err().contains("exceeds"));
    }
    #[test]
    fn rejects_unstable_and_malformed_registry_data() {
        let mut memory = MockMemory::default();
        memory.put_u32(0x20000, 0x21000); memory.put_u32(0x20004, 1); memory.put_u32(0x21000, 0x22000);
        add_valve(&mut memory, 0x22000, 0x21000, 0x24000, 0x25000, 0x26000, "alpha", -7);
        let changing = ChangingCountMemory { memory, count_reads: Cell::new(0) };
        assert!(enumerate_registry(&changing, layout()).unwrap_err().contains("changed"));

        let mut bad_pointer = MockMemory::default();
        bad_pointer.put_u32(0x20000, 0x21000); bad_pointer.put_u32(0x20004, 1); bad_pointer.put_u32(0x21000, 0x22000);
        add_valve(&mut bad_pointer, 0x22000, 0x21000, 0x24000, 0x25000, 0x26000, "alpha", 3);
        bad_pointer.put_u32(0x22008, 0x10);
        assert!(enumerate_registry(&bad_pointer, layout()).unwrap_err().contains("Invalid valve record pointer"));

        let mut invalid_name = MockMemory::default();
        invalid_name.put_u32(0x20000, 0x21000); invalid_name.put_u32(0x20004, 1); invalid_name.put_u32(0x21000, 0x22000);
        add_valve(&mut invalid_name, 0x22000, 0x21000, 0x24000, 0x25000, 0x26000, "alpha", 3);
        invalid_name.put(0x25000, &[0xff, 0]);
        assert!(enumerate_registry(&invalid_name, layout()).unwrap_err().contains("not UTF-8"));
    }
    #[test]
    fn snapshots_mark_only_changed_values() {
        let inner = Arc::new(Mutex::new(MonitorInner::default()));
        let first = update_snapshot(&inner, 42, vec![("alpha".into(), 0), ("bravo".into(), -4)]);
        assert!(first.valves.iter().all(|valve| valve.changed_at.is_none()));
        let second = update_snapshot(&inner, 42, vec![("alpha".into(), 1), ("bravo".into(), -4)]);
        assert!(second.valves.iter().find(|valve| valve.name == "alpha").unwrap().changed_at.is_some());
        assert!(second.valves.iter().find(|valve| valve.name == "bravo").unwrap().changed_at.is_none());
        assert_eq!(second.status.state, "attached");
        assert_eq!(second.status.valve_count, 2);
    }
    #[test]
    fn rejects_unsupported_fingerprint_and_layout() {
        assert!(is_supported_hash(EXPECTED_SHA256));
        assert!(!is_supported_hash("not-the-supported-build"));
        let unsupported_layout = ModuleInfo { base_address: 0x0050_0000, path: "test.exe".into() };
        assert!(validate_module_layout(&unsupported_layout).unwrap_err().contains("requires"));
    }
    #[test]
    #[ignore = "requires a running, supported Mech3.exe mission"]
    fn reads_supported_running_mission() {
        let candidates = list_candidates().unwrap();
        eprintln!("Mech3.exe candidates: {candidates:#?}");
        let candidate = candidates.into_iter().find(|candidate| candidate.compatible)
            .expect("start the supported Mech3.exe build before running this test");
        let module = inspect_compatible_process(candidate.pid).unwrap();
        let valves = read_live_valves(candidate.pid, module.base_address).unwrap();
        assert!(!valves.is_empty(), "the mission valve registry was empty");
        eprintln!("Read {} live valves from PID {}: {:?}", valves.len(), candidate.pid, valves.iter().take(12).collect::<Vec<_>>());
    }
    #[test]
    fn animation_updates_log_transitions_and_skip_unchanged_reads() {
        let inner = Arc::new(Mutex::new(MonitorInner::default()));
        let animation = |state: u8| AnimationState { index: 1, anim_name: "call_ramp".into(), name: "bramp".into(), root_name: "bramp".into(), state, activation: 0, flags: 0, instances: 1, running_instances: u32::from(animations::is_running(state)), reset_time: -1.0, timer: 0.0, max_health: 0.0, cur_health: 0.0, sequences: Vec::new(), changed_at: None };
        let first = update_animations(&inner, Ok(vec![animation(1)])).unwrap();
        assert!(first.events.is_empty());
        assert!(update_animations(&inner, Ok(vec![animation(1)])).is_none());
        let running = update_animations(&inner, Ok(vec![animation(2)])).unwrap();
        assert_eq!((running.events[0].from_state, running.events[0].to_state), (1, 2));
        assert!(running.animations[0].changed_at.is_some());
        let failed = update_animations(&inner, Err("No animation definitions are loaded yet.".into())).unwrap();
        assert_eq!(failed.animations.len(), 1);
        assert!(update_animations(&inner, Err("No animation definitions are loaded yet.".into())).is_none());
    }
    #[test]
    #[ignore = "requires a running, supported Mech3.exe mission"]
    fn reads_live_animation_table() {
        let candidate = list_candidates().unwrap().into_iter().find(|candidate| candidate.compatible)
            .expect("start the supported Mech3.exe build before running this test");
        let module = inspect_compatible_process(candidate.pid).unwrap();
        let animations = read_live_animations(candidate.pid, module.base_address).unwrap();
        let mut states: std::collections::BTreeMap<u8, usize> = Default::default();
        for animation in &animations { *states.entry(animation.state).or_default() += 1; }
        eprintln!("{} animations, states {states:?}", animations.len());
        for animation in animations.iter().filter(|animation| animation.running_instances > 0 && !animation.anim_name.contains(".flt")).take(10) { eprintln!("{animation:?}"); }
        for animation in animations.iter().filter(|animation| animation.anim_name.starts_with("start_") || animation.anim_name.contains("ramp") || animation.state == 0 || animation.state == 4).take(16) { eprintln!("{} {} {} st={} act={} inst={}/{} health={}/{} reset={}/{}", animation.index, animation.anim_name, animation.name, animation.state, animation.activation, animation.running_instances, animation.instances, animation.cur_health, animation.max_health, animation.timer, animation.reset_time); }
        assert!(animations.iter().any(|animation| animation.anim_name.starts_with("start_")));
    }
    #[test]
    #[ignore = "requires a running, supported Mech3.exe mission and write permission"]
    fn writes_back_a_live_valves_current_value() {
        let candidate = list_candidates().unwrap().into_iter().find(|candidate| candidate.compatible)
            .expect("start the supported Mech3.exe build before running this test");
        let module = inspect_compatible_process(candidate.pid).unwrap();
        let valve = read_live_valves(candidate.pid, module.base_address).unwrap().into_iter()
            .find(|(name, _)| name == "all_units")
            .expect("expected the c1m1 all_units valve");
        assert_eq!(write_live_valve(candidate.pid, module.base_address, &valve.0, valve.1, valve.1).unwrap(), valve.1);
    }
}
