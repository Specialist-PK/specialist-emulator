use crate::cpu::{Bus, Cpu};

pub const ADDRESS_SPACE_SIZE: usize = 65_536;
pub const SCREEN_ADDRESS: u16 = 0x9000;
pub const SCREEN_WIDTH: usize = 384;
pub const SCREEN_HEIGHT: usize = 256;
pub const FRAMEBUFFER_SIZE: usize = SCREEN_WIDTH / 8 * SCREEN_HEIGHT;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunResult {
    pub instructions: u32,
    pub cycles: u32,
    pub halted: bool,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MachineError {
    #[error("address {address:#x} is outside the 16-bit address space")]
    InvalidAddress { address: u32 },
    #[error("range at {address:#x} with length {length} exceeds the 16-bit address space")]
    RangeOutOfBounds { address: u32, length: u32 },
}

pub struct SystemBus {
    memory: Box<[u8; ADDRESS_SPACE_SIZE]>,
    ports: [u8; 256],
}

impl Default for SystemBus {
    fn default() -> Self {
        Self {
            memory: Box::new([0; ADDRESS_SPACE_SIZE]),
            ports: [0; 256],
        }
    }
}

impl Bus for SystemBus {
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
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
}

impl Default for Machine {
    fn default() -> Self {
        Self {
            cpu: Cpu::default(),
            bus: SystemBus::default(),
            cycles: 0,
        }
    }
}

impl Machine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.cpu = Cpu::default();
        self.cycles = 0;
        self.bus.ports = [0; 256];
    }

    pub fn clear_memory(&mut self) {
        self.bus.memory.fill(0);
    }

    pub fn step(&mut self) -> u32 {
        let cycles = self.cpu.step(&mut self.bus);
        self.cycles = self.cycles.wrapping_add(cycles as u64);
        cycles
    }

    pub fn run(&mut self, maximum_instructions: u32) -> RunResult {
        let mut instructions = 0;
        let mut cycles: u32 = 0;
        while instructions < maximum_instructions && !self.cpu.halted {
            cycles = cycles.wrapping_add(self.step());
            instructions += 1;
        }
        RunResult {
            instructions,
            cycles,
            halted: self.cpu.halted,
        }
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
        self.bus.memory[destination].copy_from_slice(program);
        self.cpu.pc = entry_point as u16;
        self.cpu.halted = false;
        Ok(())
    }

    pub fn load_demo(&mut self) {
        self.reset();
        self.clear_memory();
        self.load_program(0, 0, DEMO_PROGRAM)
            .expect("the built-in demo fits in memory");
    }

    pub fn read_memory(&self, address: u32, length: u32) -> Result<&[u8], MachineError> {
        Ok(&self.bus.memory[checked_range(address, length)?])
    }

    pub fn framebuffer(&self) -> &[u8] {
        let start = SCREEN_ADDRESS as usize;
        &self.bus.memory[start..start + FRAMEBUFFER_SIZE]
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            cpu: self.cpu,
            cycles: self.cycles,
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

    #[test]
    fn demo_draws_through_emulated_instructions_and_halts() {
        let mut machine = Machine::new();
        machine.load_demo();
        let result = machine.run(200_000);

        assert!(result.halted);
        assert_eq!(machine.snapshot().cpu.pc, 0x0014);
        assert_eq!(machine.snapshot().cpu.sp, 0x8fff);
        assert_eq!(machine.snapshot().cpu.bc(), 0);
        assert_eq!(machine.snapshot().cpu.hl(), 0xc000);
        assert_eq!(machine.framebuffer().len(), FRAMEBUFFER_SIZE);
        assert_eq!(machine.framebuffer()[0], 0x90);
        assert_eq!(machine.framebuffer()[1], 0x91);
        assert_eq!(machine.framebuffer()[FRAMEBUFFER_SIZE - 1], 0x40);
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
        machine.step();
        assert!(machine.snapshot().cpu.halted);

        machine.reset();
        assert_eq!(machine.snapshot().cpu.pc, 0);
        assert_eq!(machine.read_memory(0x100, 1).unwrap(), &[0x76]);
    }
}
