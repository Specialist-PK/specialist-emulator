use std::collections::VecDeque;

use crate::{
    cpu::{Bus, Cpu},
    keyboard::KeyEvent,
    system_ppi::SystemPpi,
    video::VideoController,
};

pub use crate::video::FRAMEBUFFER_SIZE;

pub const ADDRESS_SPACE_SIZE: usize = 65_536;
pub const SCREEN_ADDRESS: u16 = 0x9000;
pub const SCREEN_WIDTH: usize = 384;
pub const SCREEN_HEIGHT: usize = 256;
pub const MONITOR2_ROM_ADDRESS: u16 = 0xc000;
pub const MONITOR2_ROM_SIZE: usize = 0x1800;

const RAM_END: u16 = 0xbfff;
const MONITOR2_ROM_END: u16 = MONITOR2_ROM_ADDRESS + MONITOR2_ROM_SIZE as u16 - 1;

pub const DEMO_PROGRAM: &[u8] = &[
    0x31, 0xff, 0x8f, // LXI SP,8FFF
    0x21, 0x00, 0x90, // LXI H,9000
    0x01, 0x00, 0x30, // LXI B,3000
    0x7d, // MOV A,L
    0xac, // XRA H
    0x77, // MOV M,A
    0x23, // INX H
    0x0b, // DCX B
    0x78, // MOV A,B
    0xb1, // ORA C
    0xc2, 0x09, 0x00, // JNZ 0009
    0x76, // HLT
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub cpu: Cpu,
    pub cycles: u64,
    pub frame: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunResult {
    pub instructions: u32,
    pub cycles: u32,
    pub halted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineModel {
    Demo,
    Monitor2,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MachineError {
    #[error("address {address:#x} is outside the 16-bit address space")]
    InvalidAddress { address: u32 },
    #[error("range at {address:#x} with length {length} exceeds the 16-bit address space")]
    RangeOutOfBounds { address: u32, length: u32 },
    #[error(
        "memory range at {address:#x} with length {length} is not writable in this configuration"
    )]
    MemoryNotWritable { address: u32, length: u32 },
    #[error("Monitor 2 ROM must be exactly {expected:#x} bytes, got {actual:#x}")]
    InvalidRomSize { expected: u32, actual: u32 },
    #[error("Monitor 2 ROM is not loaded")]
    MissingRom,
    #[error("unknown Specialist key: {key}")]
    UnknownKey { key: String },
    #[error("key event cycle {cycle} is in the past")]
    EventCycleInPast { cycle: u32 },
}

pub struct SystemBus {
    memory: Box<[u8; ADDRESS_SPACE_SIZE]>,
    ports: [u8; 256],
    model: MachineModel,
    ppi: SystemPpi,
}

impl SystemBus {
    fn with_model(model: MachineModel) -> Self {
        Self {
            memory: Box::new([0; ADDRESS_SPACE_SIZE]),
            ports: [0; 256],
            model,
            ppi: SystemPpi::default(),
        }
    }

    fn is_ram(&self, address: u16) -> bool {
        self.model == MachineModel::Demo || address <= RAM_END
    }
}

impl Default for SystemBus {
    fn default() -> Self {
        Self::with_model(MachineModel::Demo)
    }
}

impl Bus for SystemBus {
    fn read(&self, address: u16) -> u8 {
        if self.model == MachineModel::Monitor2 && address >= 0xf800 {
            self.ppi.read((address & 3) as u8)
        } else {
            self.memory[address as usize]
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if self.model == MachineModel::Monitor2 && address >= 0xf800 {
            self.ppi.write((address & 3) as u8, value);
        } else if self.is_ram(address) {
            self.memory[address as usize] = value;
        }
    }

    fn input(&self, port: u8) -> u8 {
        self.ports[port as usize]
    }

    fn output(&mut self, port: u8, value: u8) {
        self.ports[port as usize] = value;
    }
}

pub struct Machine {
    cpu: Cpu,
    bus: SystemBus,
    cycles: u64,
    model: MachineModel,
    rom_loaded: bool,
    key_events: VecDeque<KeyEvent>,
    video: VideoController,
}

impl Default for Machine {
    fn default() -> Self {
        Self {
            cpu: Cpu::default(),
            bus: SystemBus::default(),
            cycles: 0,
            model: MachineModel::Demo,
            rom_loaded: false,
            key_events: VecDeque::new(),
            video: VideoController::default(),
        }
    }
}

impl Machine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_monitor2() -> Self {
        let mut machine = Self::default();
        machine.model = MachineModel::Monitor2;
        machine.bus.model = MachineModel::Monitor2;
        machine.reset();
        machine
    }

    pub fn reset(&mut self) {
        self.cpu = Cpu::default();
        self.cpu.pc = match self.model {
            MachineModel::Demo => 0,
            MachineModel::Monitor2 => MONITOR2_ROM_ADDRESS,
        };
        self.cycles = 0;
        self.bus.ports = [0; 256];
        self.bus.ppi.reset();
        self.key_events.clear();
        self.video.reset();
    }

    pub fn clear_memory(&mut self) {
        match self.model {
            MachineModel::Demo => self.bus.memory.fill(0),
            MachineModel::Monitor2 => {
                self.bus.memory[..MONITOR2_ROM_ADDRESS as usize].fill(0);
                self.bus.memory[MONITOR2_ROM_END as usize + 1..].fill(0);
            }
        }
    }

    pub fn key_event(&mut self, event: KeyEvent) -> Result<(), MachineError> {
        if let Some(cycle) = event.cycle {
            if u64::from(cycle) < self.cycles {
                return Err(MachineError::EventCycleInPast { cycle });
            }
            let position = self
                .key_events
                .iter()
                .position(|queued| {
                    queued
                        .cycle
                        .map_or(false, |queued_cycle| queued_cycle > cycle)
                })
                .unwrap_or(self.key_events.len());
            self.key_events.insert(position, event);
        } else {
            self.bus.ppi.set_key(event.key, event.pressed);
        }
        Ok(())
    }

    fn apply_due_key_events(&mut self) {
        while self.key_events.front().is_some_and(|event| {
            event
                .cycle
                .is_some_and(|cycle| u64::from(cycle) <= self.cycles)
        }) {
            let event = self.key_events.pop_front().expect("a queued event exists");
            self.bus.ppi.set_key(event.key, event.pressed);
        }
    }

    fn ensure_rom_loaded(&self) -> Result<(), MachineError> {
        if self.model == MachineModel::Monitor2 && !self.rom_loaded {
            return Err(MachineError::MissingRom);
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<u32, MachineError> {
        self.ensure_rom_loaded()?;
        self.apply_due_key_events();
        let cycles = self.cpu.step(&mut self.bus);
        self.cycles = self.cycles.wrapping_add(cycles as u64);
        let start = SCREEN_ADDRESS as usize;
        self.video
            .advance(cycles, &self.bus.memory[start..start + FRAMEBUFFER_SIZE]);
        Ok(cycles)
    }

    pub fn run(&mut self, maximum_instructions: u32) -> Result<RunResult, MachineError> {
        self.ensure_rom_loaded()?;
        let mut instructions = 0;
        let mut cycles: u32 = 0;
        while instructions < maximum_instructions && !self.cpu.halted {
            cycles = cycles.wrapping_add(self.step()?);
            instructions += 1;
        }
        Ok(RunResult {
            instructions,
            cycles,
            halted: self.cpu.halted,
        })
    }

    pub fn load_program(
        &mut self,
        address: u32,
        entry_point: u32,
        program: &[u8],
    ) -> Result<(), MachineError> {
        let destination = checked_range(address, program.len() as u32)?;
        if entry_point > u16::MAX as u32 {
            return Err(MachineError::InvalidAddress {
                address: entry_point,
            });
        }
        if self.model == MachineModel::Monitor2
            && destination
                .clone()
                .any(|index| !self.bus.is_ram(index as u16))
        {
            return Err(MachineError::MemoryNotWritable {
                address,
                length: program.len() as u32,
            });
        }
        self.bus.memory[destination].copy_from_slice(program);
        self.cpu.pc = entry_point as u16;
        self.cpu.halted = false;
        Ok(())
    }

    pub fn load_monitor2_rom(&mut self, rom: &[u8]) -> Result<(), MachineError> {
        if rom.len() != MONITOR2_ROM_SIZE {
            return Err(MachineError::InvalidRomSize {
                expected: MONITOR2_ROM_SIZE as u32,
                actual: rom.len() as u32,
            });
        }
        self.model = MachineModel::Monitor2;
        self.bus.model = MachineModel::Monitor2;
        self.rom_loaded = false;
        self.reset();
        self.clear_memory();
        let start = MONITOR2_ROM_ADDRESS as usize;
        self.bus.memory[start..start + MONITOR2_ROM_SIZE].copy_from_slice(rom);
        self.rom_loaded = true;
        Ok(())
    }

    pub fn load_demo(&mut self) {
        self.model = MachineModel::Demo;
        self.bus.model = MachineModel::Demo;
        self.rom_loaded = false;
        self.reset();
        self.clear_memory();
        self.load_program(0, 0, DEMO_PROGRAM)
            .expect("the built-in demo fits in memory");
    }

    pub fn read_memory(&self, address: u32, length: u32) -> Result<&[u8], MachineError> {
        Ok(&self.bus.memory[checked_range(address, length)?])
    }

    pub fn framebuffer(&self) -> &[u8] {
        self.video.framebuffer()
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            cpu: self.cpu,
            cycles: self.cycles,
            frame: self.video.frame_index(),
        }
    }
}

fn checked_range(address: u32, length: u32) -> Result<std::ops::Range<usize>, MachineError> {
    if address > u16::MAX as u32 {
        return Err(MachineError::InvalidAddress { address });
    }
    let end = address
        .checked_add(length)
        .ok_or(MachineError::RangeOutOfBounds { address, length })?;
    if end > ADDRESS_SPACE_SIZE as u32 {
        return Err(MachineError::RangeOutOfBounds { address, length });
    }
    Ok(address as usize..end as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard::Key;

    #[test]
    fn demo_draws_through_emulated_instructions_and_halts() {
        let mut machine = Machine::new();
        machine.load_demo();
        let result = machine.run(200_000).unwrap();

        assert!(result.halted);
        assert_eq!(machine.snapshot().frame, 14);
        assert_eq!(machine.snapshot().cpu.pc, 0x0014);
        assert_eq!(machine.snapshot().cpu.sp, 0x8fff);
        assert_eq!(machine.snapshot().cpu.bc(), 0);
        assert_eq!(machine.snapshot().cpu.hl(), 0xc000);
        assert_eq!(machine.framebuffer().len(), FRAMEBUFFER_SIZE);
        assert_eq!(machine.framebuffer()[0], 0x90);
        assert_eq!(machine.framebuffer()[1], 0x91);
        assert_eq!(machine.framebuffer()[FRAMEBUFFER_SIZE - 1], 0);
    }

    #[test]
    fn rejects_memory_ranges_outside_the_address_space() {
        let mut machine = Machine::new();
        assert_eq!(
            machine.load_program(0x10000, 0, &[1]),
            Err(MachineError::InvalidAddress { address: 0x10000 })
        );
        assert_eq!(
            machine.load_program(0xffff, 0, &[1, 2]),
            Err(MachineError::RangeOutOfBounds {
                address: 0xffff,
                length: 2
            })
        );
        assert_eq!(
            machine.read_memory(0xffff, 2),
            Err(MachineError::RangeOutOfBounds {
                address: 0xffff,
                length: 2
            })
        );
    }

    #[test]
    fn reset_preserves_the_loaded_program() {
        let mut machine = Machine::new();
        machine.load_program(0x100, 0x100, &[0x76]).unwrap();
        machine.step().unwrap();
        assert!(machine.snapshot().cpu.halted);

        machine.reset();
        assert_eq!(machine.snapshot().cpu.pc, 0);
        assert_eq!(machine.read_memory(0x100, 1).unwrap(), &[0x76]);
    }

    #[test]
    fn monitor2_ram_is_writable_and_rom_is_protected() {
        let mut bus = SystemBus::with_model(MachineModel::Monitor2);
        for address in [
            0x0000, 0x3fff, 0x4000, 0x7fff, 0x8000, 0x8fff, 0x9000, 0xbfff,
        ] {
            bus.write(address, 0xa5);
            assert_eq!(bus.read(address), 0xa5, "RAM address {address:#x}");
        }
        for address in [MONITOR2_ROM_ADDRESS, MONITOR2_ROM_END] {
            bus.write(address, 0xa5);
            assert_eq!(bus.read(address), 0, "ROM address {address:#x}");
        }
    }

    #[test]
    fn monitor2_requires_the_exact_rom_before_execution() {
        let mut machine = Machine::new_monitor2();
        assert_eq!(machine.run(1), Err(MachineError::MissingRom));
        assert_eq!(machine.snapshot().cycles, 0);
        assert_eq!(
            machine.load_monitor2_rom(&[0; 17]),
            Err(MachineError::InvalidRomSize {
                expected: MONITOR2_ROM_SIZE as u32,
                actual: 17
            })
        );
        assert_eq!(machine.step(), Err(MachineError::MissingRom));
    }

    #[test]
    fn monitor2_loads_rom_and_prevents_cpu_writes_to_it() {
        let mut machine = Machine::new();
        let mut rom = vec![0; MONITOR2_ROM_SIZE];
        rom[MONITOR2_ROM_SIZE - 1] = 0x7e;
        machine.load_monitor2_rom(&rom).unwrap();
        assert_eq!(machine.snapshot().cpu.pc, MONITOR2_ROM_ADDRESS);
        assert_eq!(
            machine.read_memory(MONITOR2_ROM_END as u32, 1).unwrap(),
            &[0x7e]
        );

        machine
            .load_program(0, 0, &[0x21, 0x00, 0xc0, 0x36, 0x5a, 0x76])
            .unwrap();
        machine.run(3).unwrap();
        assert_eq!(
            machine.read_memory(MONITOR2_ROM_ADDRESS as u32, 1).unwrap(),
            &[0]
        );
    }

    #[test]
    fn monitor2_rejects_program_loading_outside_ram() {
        let mut machine = Machine::new();
        machine
            .load_monitor2_rom(&vec![0; MONITOR2_ROM_SIZE])
            .unwrap();
        assert_eq!(
            machine.load_program(MONITOR2_ROM_ADDRESS as u32, 0, &[0x76]),
            Err(MachineError::MemoryNotWritable {
                address: MONITOR2_ROM_ADDRESS as u32,
                length: 1
            })
        );
    }

    #[test]
    fn monitor2_reset_restores_the_rom_entry_point() {
        let mut machine = Machine::new();
        let mut rom = vec![0; MONITOR2_ROM_SIZE];
        rom[0] = 0x76;
        machine.load_monitor2_rom(&rom).unwrap();
        machine.run(1).unwrap();
        assert_eq!(machine.snapshot().cpu.pc, MONITOR2_ROM_ADDRESS + 1);

        machine.reset();
        assert_eq!(machine.snapshot().cpu.pc, MONITOR2_ROM_ADDRESS);
        assert_eq!(
            machine.read_memory(MONITOR2_ROM_ADDRESS as u32, 1).unwrap(),
            &[0x76]
        );
    }

    #[test]
    fn monitor2_ppi_registers_mirror_every_four_bytes() {
        let mut bus = SystemBus::with_model(MachineModel::Monitor2);
        bus.write(0xf803, 0x80);
        bus.write(0xf800, 0x5a);
        assert_eq!(bus.read(0xf800), 0x5a);
        assert_eq!(bus.read(0xf804), 0x5a);
        assert_eq!(bus.read(0xfffc), 0x5a);
        assert_eq!(bus.read(0xf803), 0xff);
        assert_eq!(bus.read(0xffff), 0xff);
    }

    #[test]
    fn monitor2_key_events_wait_for_their_cycle_and_reset_clears_input() {
        let mut machine = Machine::new();
        machine
            .load_monitor2_rom(&vec![0; MONITOR2_ROM_SIZE])
            .unwrap();
        machine.bus.ppi.write(3, 0x82);
        machine.bus.ppi.write(0, 0);
        machine.bus.ppi.write(2, 0);
        machine
            .key_event(KeyEvent {
                key: Key::A,
                pressed: true,
                cycle: Some(4),
            })
            .unwrap();

        assert_eq!(machine.bus.ppi.read(1), 0xff);
        machine.step().unwrap();
        assert_eq!(machine.snapshot().cycles, 4);
        assert_eq!(machine.bus.ppi.read(1), 0xff);
        machine.step().unwrap();
        assert_eq!(machine.bus.ppi.read(1), 0xef);
        assert_eq!(
            machine.key_event(KeyEvent {
                key: Key::Q,
                pressed: true,
                cycle: Some(0),
            }),
            Err(MachineError::EventCycleInPast { cycle: 0 })
        );

        machine.reset();
        assert_eq!(machine.bus.ppi.read(1), 0xff);
        assert!(machine.key_events.is_empty());
    }
}
