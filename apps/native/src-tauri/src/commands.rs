use std::sync::Mutex;

use serde::Serialize;
use specialist_core::{Machine, MachineError};
use specta::Type;
use tauri::State;

#[derive(Default)]
pub struct AppState {
    machine: Mutex<Machine>,
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FlagsSnapshot {
    sign: bool,
    zero: bool,
    auxiliary_carry: bool,
    parity: bool,
    carry: bool,
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CpuSnapshot {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    flags: FlagsSnapshot,
    interrupts_enabled: bool,
    halted: bool,
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorSnapshot {
    cpu: CpuSnapshot,
    cycles: String,
}

#[derive(Debug, Serialize, Type, thiserror::Error)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CommandError {
    #[error("address {address:#x} is outside the 16-bit address space")]
    InvalidAddress { address: u32 },
    #[error("range at {address:#x} with length {length} exceeds the address space")]
    RangeOutOfBounds { address: u32, length: u32 },
    #[error("memory range at {address:#x} with length {length} is not writable")]
    MemoryNotWritable { address: u32, length: u32 },
    #[error("Monitor 2 ROM must be exactly {expected:#x} bytes, got {actual:#x}")]
    InvalidRomSize { expected: u32, actual: u32 },
    #[error("Monitor 2 ROM is not loaded")]
    MissingRom,
}

impl From<MachineError> for CommandError {
    fn from(error: MachineError) -> Self {
        match error {
            MachineError::InvalidAddress { address } => Self::InvalidAddress { address },
            MachineError::RangeOutOfBounds { address, length } => {
                Self::RangeOutOfBounds { address, length }
            }
            MachineError::MemoryNotWritable { address, length } => {
                Self::MemoryNotWritable { address, length }
            }
            MachineError::InvalidRomSize { expected, actual } => {
                Self::InvalidRomSize { expected, actual }
            }
            MachineError::MissingRom => Self::MissingRom,
        }
    }
}

fn make_snapshot(machine: &Machine) -> EmulatorSnapshot {
    let snapshot = machine.snapshot();
    EmulatorSnapshot {
        cpu: CpuSnapshot {
            a: snapshot.cpu.a,
            b: snapshot.cpu.b,
            c: snapshot.cpu.c,
            d: snapshot.cpu.d,
            e: snapshot.cpu.e,
            h: snapshot.cpu.h,
            l: snapshot.cpu.l,
            sp: snapshot.cpu.sp,
            pc: snapshot.cpu.pc,
            flags: FlagsSnapshot {
                sign: snapshot.cpu.flags.sign,
                zero: snapshot.cpu.flags.zero,
                auxiliary_carry: snapshot.cpu.flags.auxiliary_carry,
                parity: snapshot.cpu.flags.parity,
                carry: snapshot.cpu.flags.carry,
            },
            interrupts_enabled: snapshot.cpu.interrupts_enabled,
            halted: snapshot.cpu.halted,
        },
        cycles: snapshot.cycles.to_string(),
    }
}

#[tauri::command]
#[specta::specta]
pub fn reset(state: State<'_, AppState>) -> EmulatorSnapshot {
    let mut machine = state.machine.lock().expect("emulator mutex poisoned");
    machine.reset();
    make_snapshot(&machine)
}

#[tauri::command]
#[specta::specta]
pub fn load_demo(state: State<'_, AppState>) -> EmulatorSnapshot {
    let mut machine = state.machine.lock().expect("emulator mutex poisoned");
    machine.load_demo();
    make_snapshot(&machine)
}

#[tauri::command]
#[specta::specta]
pub fn load_monitor2_rom(
    state: State<'_, AppState>,
    rom: Vec<u8>,
) -> Result<EmulatorSnapshot, CommandError> {
    let mut machine = state.machine.lock().expect("emulator mutex poisoned");
    machine.load_monitor2_rom(&rom)?;
    Ok(make_snapshot(&machine))
}

#[tauri::command]
#[specta::specta]
pub fn load_program(
    state: State<'_, AppState>,
    address: u32,
    entry_point: u32,
    program: Vec<u8>,
) -> Result<EmulatorSnapshot, CommandError> {
    let mut machine = state.machine.lock().expect("emulator mutex poisoned");
    machine.load_program(address, entry_point, &program)?;
    Ok(make_snapshot(&machine))
}

#[tauri::command]
#[specta::specta]
pub fn run(
    state: State<'_, AppState>,
    instructions: u32,
) -> Result<EmulatorSnapshot, CommandError> {
    let mut machine = state.machine.lock().expect("emulator mutex poisoned");
    machine.run(instructions)?;
    Ok(make_snapshot(&machine))
}

#[tauri::command]
#[specta::specta]
pub fn snapshot(state: State<'_, AppState>) -> EmulatorSnapshot {
    make_snapshot(&state.machine.lock().expect("emulator mutex poisoned"))
}

#[tauri::command]
#[specta::specta]
pub fn framebuffer(state: State<'_, AppState>) -> Vec<u8> {
    state
        .machine
        .lock()
        .expect("emulator mutex poisoned")
        .framebuffer()
        .to_vec()
}

#[tauri::command]
#[specta::specta]
pub fn read_memory(
    state: State<'_, AppState>,
    address: u32,
    length: u32,
) -> Result<Vec<u8>, CommandError> {
    let machine = state.machine.lock().expect("emulator mutex poisoned");
    Ok(machine.read_memory(address, length)?.to_vec())
}
