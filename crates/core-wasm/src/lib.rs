use js_sys::{Object, Reflect};
use serde::Serialize;
use specialist_core::{Machine, MachineError};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_ERROR: &str = r#"
export type WasmEmulatorError =
  | { kind: "invalidAddress"; address: number }
  | { kind: "rangeOutOfBounds"; address: number; length: number }
  | { kind: "memoryNotWritable"; address: number; length: number }
  | { kind: "invalidRomSize"; expected: number; actual: number }
  | { kind: "missingRom" };
"#;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FlagsSnapshot {
    sign: bool,
    zero: bool,
    auxiliary_carry: bool,
    parity: bool,
    carry: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CpuSnapshot {
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EmulatorSnapshot {
    cpu: CpuSnapshot,
    cycles: f64,
}

fn snapshot(machine: &Machine) -> Result<JsValue, JsValue> {
    let snapshot = machine.snapshot();
    serde_wasm_bindgen::to_value(&EmulatorSnapshot {
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
        cycles: snapshot.cycles as f64,
    })
    .map_err(|error| JsValue::from_str(&error.to_string()))
}

fn error_object(error: MachineError) -> JsValue {
    let object = Object::new();
    let set = |key: &str, value: JsValue| {
        Reflect::set(&object, &key.into(), &value).expect("setting a property on a plain object");
    };
    match error {
        MachineError::InvalidAddress { address } => {
            set("kind", "invalidAddress".into());
            set("address", address.into());
        }
        MachineError::RangeOutOfBounds { address, length } => {
            set("kind", "rangeOutOfBounds".into());
            set("address", address.into());
            set("length", length.into());
        }
        MachineError::MemoryNotWritable { address, length } => {
            set("kind", "memoryNotWritable".into());
            set("address", address.into());
            set("length", length.into());
        }
        MachineError::InvalidRomSize { expected, actual } => {
            set("kind", "invalidRomSize".into());
            set("expected", expected.into());
            set("actual", actual.into());
        }
        MachineError::MissingRom => {
            set("kind", "missingRom".into());
        }
    }
    object.into()
}

#[wasm_bindgen]
pub struct WasmEmulator {
    machine: Machine,
}

#[wasm_bindgen]
impl WasmEmulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            machine: Machine::new(),
        }
    }

    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.machine.reset();
        snapshot(&self.machine)
    }

    #[wasm_bindgen(js_name = loadDemo)]
    pub fn load_demo(&mut self) -> Result<JsValue, JsValue> {
        self.machine.load_demo();
        snapshot(&self.machine)
    }

    #[wasm_bindgen(js_name = loadMonitor2Rom)]
    pub fn load_monitor2_rom(&mut self, rom: &[u8]) -> Result<JsValue, JsValue> {
        self.machine.load_monitor2_rom(rom).map_err(error_object)?;
        snapshot(&self.machine)
    }

    #[wasm_bindgen(js_name = loadProgram)]
    pub fn load_program(
        &mut self,
        address: u32,
        entry_point: u32,
        program: &[u8],
    ) -> Result<JsValue, JsValue> {
        self.machine
            .load_program(address, entry_point, program)
            .map_err(error_object)?;
        snapshot(&self.machine)
    }

    pub fn run(&mut self, instructions: u32) -> Result<JsValue, JsValue> {
        self.machine.run(instructions).map_err(error_object)?;
        snapshot(&self.machine)
    }

    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        snapshot(&self.machine)
    }

    pub fn framebuffer(&self) -> Vec<u8> {
        self.machine.framebuffer().to_vec()
    }

    #[wasm_bindgen(js_name = readMemory)]
    pub fn read_memory(&self, address: u32, length: u32) -> Result<Vec<u8>, JsValue> {
        self.machine
            .read_memory(address, length)
            .map(<[u8]>::to_vec)
            .map_err(error_object)
    }
}
