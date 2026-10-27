// Read-only view of the live animation-definition table (zEffect anim info).
//
// Layout evidence (Mech3.exe 1.2.22):
// - 0x0073f720 is the loaded AnimInfo; +10 holds the u16 definition count and
//   +12 the definition array pointer. _zEffResetActivationRecord (0x0050b970)
//   walks that array from index 1 with stride 0x13c.
// - Each record is mech3ax's AnimDefC (316 bytes). +0x138 links further runtime
//   instances of the same definition (FUN_00508310 allocates them when a running
//   definition is called again).
// - +0x98 is anim_state: FUN_00508310 (call) sets 0 before starting,
//   FUN_00508170 (reset) sets 1 or 4, FUN_00506ca0 (stop) acts only on 2 and 6
//   and leaves 3 or 4; the save code logs 3 as EXECUTED and 4 as INVALID.
//   Every entry point refuses state 5.
use crate::runtime::{read_pointer, validate_pointer, MemoryReader};
use serde::Serialize;

const ANIM_DEF_COUNT_RVA: u64 = 0x0073_f72a - crate::runtime::IMAGE_BASE;
const ANIM_DEFS_POINTER_RVA: u64 = 0x0073_f72c - crate::runtime::IMAGE_BASE;
pub const ANIM_DEF_SIZE: usize = 0x13c;
const MAX_ANIM_DEFS: usize = 8192;
const MAX_INSTANCES: usize = 64;
const MAX_SEQUENCES: usize = 64;
const SEQ_DEF_SIZE: usize = 64;

const OFFSET_NAME: usize = 32;
const OFFSET_ROOT_NAME: usize = 68;
const OFFSET_FLAGS: usize = 0x94;
const OFFSET_STATE: usize = 0x98;
const OFFSET_ACTIVATION: usize = 0x99;
const OFFSET_RESET_TIME: usize = 0xa4;
const OFFSET_TIMER: usize = 0xa8;
const OFFSET_MAX_HEALTH: usize = 0xac;
const OFFSET_CUR_HEALTH: usize = 0xb0;
const OFFSET_SEQ_DEFS: usize = 0xc4;
const OFFSET_SEQ_COUNT: usize = 0x108;
const OFFSET_NEXT_INSTANCE: usize = 0x138;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceState {
    pub name: String,
    pub state: u8,
    pub loop_count: u32,
    /// The sequence has a current event pointer, i.e. it is part-way through its events.
    pub has_current_event: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationState {
    pub index: u32,
    pub anim_name: String,
    pub name: String,
    pub root_name: String,
    pub state: u8,
    pub activation: u8,
    pub flags: u32,
    pub instances: u32,
    pub running_instances: u32,
    pub reset_time: f32,
    /// +0xa8, advanced by the frame time (FUN_005085b0) and compared against reset_time.
    pub timer: f32,
    pub max_health: f32,
    pub cur_health: f32,
    pub sequences: Vec<SequenceState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_at: Option<u64>,
}

impl AnimationState {
    /// The values whose change is reported as a transition.
    pub fn transition_key(&self) -> (u8, u32) {
        (self.state, self.running_instances)
    }
}

pub fn is_running(state: u8) -> bool {
    state == 2 || state == 6
}

pub fn read_animation_table<M: MemoryReader>(memory: &M, module_base: u64) -> Result<Vec<AnimationState>, String> {
    let count_address = module_base + ANIM_DEF_COUNT_RVA;
    let pointer_address = module_base + ANIM_DEFS_POINTER_RVA;
    let count = read_u16(memory, count_address)? as usize;
    let defs = read_pointer(memory, pointer_address)?;
    if count <= 1 || defs == 0 {
        return Err("No animation definitions are loaded yet.".into());
    }
    if count > MAX_ANIM_DEFS {
        return Err(format!("Animation definition count {count} exceeds the safe limit {MAX_ANIM_DEFS}."));
    }
    validate_pointer(defs, "animation definition array")?;
    let table = memory.read_bytes(defs, count * ANIM_DEF_SIZE)?;
    let mut animations = Vec::with_capacity(count - 1);
    // Index 0 is the engine's empty placeholder record.
    for index in 1..count {
        let record = &table[index * ANIM_DEF_SIZE..(index + 1) * ANIM_DEF_SIZE];
        animations.push(read_animation(memory, index as u32, record)?);
    }
    if read_u16(memory, count_address)? as usize != count || read_pointer(memory, pointer_address)? != defs {
        return Err("The animation table changed while it was being read; retrying.".into());
    }
    Ok(animations)
}

fn read_animation<M: MemoryReader>(memory: &M, index: u32, record: &[u8]) -> Result<AnimationState, String> {
    let anim_name = ascii(&record[..32], index, "anim_name")?;
    let mut state = record[OFFSET_STATE];
    let mut instances = 1;
    let mut running_instances = u32::from(is_running(state));
    let mut next = u32_at(record, OFFSET_NEXT_INSTANCE) as u64;
    let mut seen = Vec::new();
    while next != 0 {
        validate_pointer(next, "animation instance")?;
        if seen.contains(&next) || seen.len() >= MAX_INSTANCES {
            return Err(format!("Animation '{anim_name}' has a cyclic or oversized instance chain."));
        }
        seen.push(next);
        let instance = memory.read_bytes(next, ANIM_DEF_SIZE)?;
        instances += 1;
        if is_running(instance[OFFSET_STATE]) {
            running_instances += 1;
        }
        next = u32_at(&instance, OFFSET_NEXT_INSTANCE) as u64;
    }
    // A running clone means the definition as a whole is running.
    if running_instances > 0 && !is_running(state) {
        state = 2;
    }
    let sequences = if running_instances > 0 { read_sequences(memory, record)? } else { Vec::new() };
    Ok(AnimationState {
        index,
        name: ascii(&record[OFFSET_NAME..OFFSET_NAME + 32], index, "name")?,
        root_name: ascii(&record[OFFSET_ROOT_NAME..OFFSET_ROOT_NAME + 32], index, "anim_root_name")?,
        anim_name,
        state,
        activation: record[OFFSET_ACTIVATION],
        flags: u32_at(record, OFFSET_FLAGS),
        instances,
        running_instances,
        reset_time: f32_at(record, OFFSET_RESET_TIME),
        timer: f32_at(record, OFFSET_TIMER),
        max_health: f32_at(record, OFFSET_MAX_HEALTH),
        cur_health: f32_at(record, OFFSET_CUR_HEALTH),
        sequences,
        changed_at: None,
    })
}

fn read_sequences<M: MemoryReader>(memory: &M, record: &[u8]) -> Result<Vec<SequenceState>, String> {
    let count = record[OFFSET_SEQ_COUNT] as usize;
    let pointer = u32_at(record, OFFSET_SEQ_DEFS) as u64;
    if count == 0 || pointer == 0 {
        return Ok(Vec::new());
    }
    if count > MAX_SEQUENCES {
        return Err(format!("Sequence count {count} exceeds the safe limit {MAX_SEQUENCES}."));
    }
    validate_pointer(pointer, "sequence array")?;
    let bytes = memory.read_bytes(pointer, count * SEQ_DEF_SIZE)?;
    Ok(bytes
        .chunks_exact(SEQ_DEF_SIZE)
        .map(|seq| SequenceState {
            name: c_str(&seq[..32]),
            state: seq[32],
            loop_count: u32_at(seq, 48),
            has_current_event: u32_at(seq, 52) != 0,
        })
        .collect())
}

fn read_u16<M: MemoryReader>(memory: &M, address: u64) -> Result<u16, String> {
    let bytes = memory.read_bytes(address, 2)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn f32_at(bytes: &[u8], offset: usize) -> f32 {
    f32::from_bits(u32_at(bytes, offset))
}

fn c_str(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

fn ascii(bytes: &[u8], index: u32, field: &str) -> Result<String, String> {
    let value = c_str(bytes);
    if !value.bytes().all(|byte| byte.is_ascii_graphic() || byte == b' ') {
        return Err(format!("Animation {index} has a malformed {field}; the table is not ready."));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct MockMemory { bytes: HashMap<u64, u8> }
    impl MockMemory {
        fn put(&mut self, address: u64, bytes: &[u8]) { for (offset, byte) in bytes.iter().enumerate() { self.bytes.insert(address + offset as u64, *byte); } }
    }
    impl MemoryReader for MockMemory {
        fn read_bytes(&self, address: u64, length: usize) -> Result<Vec<u8>, String> { (0..length).map(|offset| self.bytes.get(&(address + offset as u64)).copied().ok_or_else(|| format!("Missing mock memory at 0x{:08x}", address + offset as u64))).collect() }
    }

    fn record(anim_name: &str, name: &str, state: u8, next: u32) -> Vec<u8> {
        let mut bytes = vec![0_u8; ANIM_DEF_SIZE];
        bytes[..anim_name.len()].copy_from_slice(anim_name.as_bytes());
        bytes[OFFSET_NAME..OFFSET_NAME + name.len()].copy_from_slice(name.as_bytes());
        bytes[OFFSET_STATE] = state;
        bytes[OFFSET_MAX_HEALTH..OFFSET_MAX_HEALTH + 4].copy_from_slice(&70.0_f32.to_le_bytes());
        bytes[OFFSET_NEXT_INSTANCE..OFFSET_NEXT_INSTANCE + 4].copy_from_slice(&next.to_le_bytes());
        bytes
    }

    fn table(records: &[Vec<u8>]) -> MockMemory {
        let mut memory = MockMemory::default();
        memory.put(0x0073_f72a, &(records.len() as u16 + 1).to_le_bytes());
        memory.put(0x0073_f72c, &0x0010_0000_u32.to_le_bytes());
        memory.put(0x0010_0000, &vec![0; ANIM_DEF_SIZE]);
        for (index, record) in records.iter().enumerate() {
            memory.put(0x0010_0000 + ((index + 1) * ANIM_DEF_SIZE) as u64, record);
        }
        memory
    }

    #[test]
    fn reads_definitions_after_the_placeholder_and_folds_running_instances() {
        let mut memory = table(&[record("call_ramp", "bramp", 3, 0), record("xplflare.flt", "fx", 3, 0x0020_0000)]);
        memory.put(0x0020_0000, &record("xplflare.flt", "fx", 2, 0));
        let animations = read_animation_table(&memory, crate::runtime::IMAGE_BASE).unwrap();
        assert_eq!(animations.len(), 2);
        assert_eq!((animations[0].index, animations[0].anim_name.as_str(), animations[0].name.as_str(), animations[0].state), (1, "call_ramp", "bramp", 3));
        assert_eq!(animations[0].max_health, 70.0);
        assert_eq!((animations[1].instances, animations[1].running_instances, animations[1].state), (2, 1, 2));
    }

    #[test]
    fn rejects_an_empty_table_and_instance_cycles() {
        let mut empty = MockMemory::default();
        empty.put(0x0073_f72a, &0_u16.to_le_bytes());
        empty.put(0x0073_f72c, &0_u32.to_le_bytes());
        assert!(read_animation_table(&empty, crate::runtime::IMAGE_BASE).unwrap_err().contains("No animation"));
        let mut cyclic = table(&[record("loop", "loop", 1, 0x0020_0000)]);
        cyclic.put(0x0020_0000, &record("loop", "loop", 1, 0x0020_0000));
        assert!(read_animation_table(&cyclic, crate::runtime::IMAGE_BASE).unwrap_err().contains("cyclic"));
    }
}
