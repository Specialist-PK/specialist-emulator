pub trait Bus {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn input(&self, port: u8) -> u8;
    fn output(&mut self, port: u8, value: u8);
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    pub sign: bool,
    pub zero: bool,
    pub auxiliary_carry: bool,
    pub parity: bool,
    pub carry: bool,
}

impl Flags {
    pub fn to_byte(self) -> u8 {
        u8::from(self.sign) << 7
            | u8::from(self.zero) << 6
            | u8::from(self.auxiliary_carry) << 4
            | u8::from(self.parity) << 2
            | 0b10
            | u8::from(self.carry)
    }

    pub fn from_byte(value: u8) -> Self {
        Self {
            sign: value & 0x80 != 0,
            zero: value & 0x40 != 0,
            auxiliary_carry: value & 0x10 != 0,
            parity: value & 0x04 != 0,
            carry: value & 0x01 != 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Cpu {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub flags: Flags,
    pub interrupts_enabled: bool,
    pub halted: bool,
}

impl Cpu {
    pub fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }

    pub fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }

    pub fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }

    pub fn step<B: Bus>(&mut self, bus: &mut B) -> u32 {
        let opcode = self.fetch_byte(bus);
        match opcode {
            0x01 | 0x11 | 0x21 | 0x31 => {
                let value = self.fetch_word(bus);
                self.write_pair((opcode >> 4) & 3, value);
                10
            }
            0x02 => {
                bus.write(self.bc(), self.a);
                7
            }
            0x12 => {
                bus.write(self.de(), self.a);
                7
            }
            0x0a => {
                self.a = bus.read(self.bc());
                7
            }
            0x1a => {
                self.a = bus.read(self.de());
                7
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                let pair = (opcode >> 4) & 3;
                self.write_pair(pair, self.read_pair(pair).wrapping_add(1));
                5
            }
            0x0b | 0x1b | 0x2b | 0x3b => {
                let pair = (opcode >> 4) & 3;
                self.write_pair(pair, self.read_pair(pair).wrapping_sub(1));
                5
            }
            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x34 | 0x3c => {
                let register = (opcode >> 3) & 7;
                let value = self.read_register(register, bus);
                let result = value.wrapping_add(1);
                self.flags.auxiliary_carry = (value & 0x0f) == 0x0f;
                self.set_szp(result);
                self.write_register(register, result, bus);
                if register == 6 { 10 } else { 5 }
            }
            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x35 | 0x3d => {
                let register = (opcode >> 3) & 7;
                let value = self.read_register(register, bus);
                let result = value.wrapping_sub(1);
                self.flags.auxiliary_carry = (value & 0x0f) == 0;
                self.set_szp(result);
                self.write_register(register, result, bus);
                if register == 6 { 10 } else { 5 }
            }
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x36 | 0x3e => {
                let register = (opcode >> 3) & 7;
                let value = self.fetch_byte(bus);
                self.write_register(register, value, bus);
                if register == 6 { 10 } else { 7 }
            }
            0x07 => {
                self.flags.carry = self.a & 0x80 != 0;
                self.a = self.a.rotate_left(1);
                4
            }
            0x0f => {
                self.flags.carry = self.a & 1 != 0;
                self.a = self.a.rotate_right(1);
                4
            }
            0x17 => {
                let carry = u8::from(self.flags.carry);
                self.flags.carry = self.a & 0x80 != 0;
                self.a = (self.a << 1) | carry;
                4
            }
            0x1f => {
                let carry = u8::from(self.flags.carry) << 7;
                self.flags.carry = self.a & 1 != 0;
                self.a = (self.a >> 1) | carry;
                4
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                let value = self.read_pair((opcode >> 4) & 3) as u32;
                let hl = self.hl() as u32;
                let result = hl + value;
                self.flags.carry = result > 0xffff;
                self.set_hl(result as u16);
                10
            }
            0x22 => {
                let address = self.fetch_word(bus);
                write_word(bus, address, self.hl());
                16
            }
            0x2a => {
                let address = self.fetch_word(bus);
                self.set_hl(read_word(bus, address));
                16
            }
            0x32 => {
                let address = self.fetch_word(bus);
                bus.write(address, self.a);
                13
            }
            0x3a => {
                let address = self.fetch_word(bus);
                self.a = bus.read(address);
                13
            }
            0x27 => {
                self.daa();
                4
            }
            0x2f => {
                self.a = !self.a;
                4
            }
            0x37 => {
                self.flags.carry = true;
                4
            }
            0x3f => {
                self.flags.carry = !self.flags.carry;
                4
            }
            0x40..=0x7f => {
                if opcode == 0x76 {
                    self.halted = true;
                    7
                } else {
                    let source = opcode & 7;
                    let destination = (opcode >> 3) & 7;
                    let value = self.read_register(source, bus);
                    self.write_register(destination, value, bus);
                    if source == 6 || destination == 6 {
                        7
                    } else {
                        5
                    }
                }
            }
            0x80..=0xbf => {
                let value = self.read_register(opcode & 7, bus);
                self.alu((opcode >> 3) & 7, value);
                if opcode & 7 == 6 { 7 } else { 4 }
            }
            0xc0 | 0xc8 | 0xd0 | 0xd8 | 0xe0 | 0xe8 | 0xf0 | 0xf8 => {
                if self.condition((opcode >> 3) & 7) {
                    self.pc = self.pop(bus);
                    11
                } else {
                    5
                }
            }
            0xc1 | 0xd1 | 0xe1 | 0xf1 => {
                let value = self.pop(bus);
                if opcode == 0xf1 {
                    self.a = (value >> 8) as u8;
                    self.flags = Flags::from_byte(value as u8);
                } else {
                    self.write_pair((opcode >> 4) & 3, value);
                }
                10
            }
            0xc2 | 0xca | 0xd2 | 0xda | 0xe2 | 0xea | 0xf2 | 0xfa | 0xc3 => {
                let address = self.fetch_word(bus);
                if opcode == 0xc3 || self.condition((opcode >> 3) & 7) {
                    self.pc = address;
                }
                10
            }
            0xc4 | 0xcc | 0xd4 | 0xdc | 0xe4 | 0xec | 0xf4 | 0xfc | 0xcd => {
                let address = self.fetch_word(bus);
                if opcode == 0xcd || self.condition((opcode >> 3) & 7) {
                    self.push(bus, self.pc);
                    self.pc = address;
                    17
                } else {
                    11
                }
            }
            0xc5 | 0xd5 | 0xe5 | 0xf5 => {
                let value = if opcode == 0xf5 {
                    u16::from_be_bytes([self.a, self.flags.to_byte()])
                } else {
                    self.read_pair((opcode >> 4) & 3)
                };
                self.push(bus, value);
                11
            }
            0xc6 | 0xce | 0xd6 | 0xde | 0xe6 | 0xee | 0xf6 | 0xfe => {
                let value = self.fetch_byte(bus);
                self.alu((opcode >> 3) & 7, value);
                7
            }
            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => {
                self.push(bus, self.pc);
                self.pc = (opcode & 0x38) as u16;
                11
            }
            0xc9 => {
                self.pc = self.pop(bus);
                10
            }
            0xd3 => {
                let port = self.fetch_byte(bus);
                bus.output(port, self.a);
                10
            }
            0xdb => {
                let port = self.fetch_byte(bus);
                self.a = bus.input(port);
                10
            }
            0xe3 => {
                let value = read_word(bus, self.sp);
                write_word(bus, self.sp, self.hl());
                self.set_hl(value);
                18
            }
            0xe9 => {
                self.pc = self.hl();
                5
            }
            0xeb => {
                let de = self.de();
                self.set_de(self.hl());
                self.set_hl(de);
                5
            }
            0xf3 => {
                self.interrupts_enabled = false;
                4
            }
            0xfb => {
                self.interrupts_enabled = true;
                4
            }
            _ => 4,
        }
    }

    fn fetch_byte<B: Bus>(&mut self, bus: &B) -> u8 {
        let value = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    fn fetch_word<B: Bus>(&mut self, bus: &B) -> u16 {
        let low = self.fetch_byte(bus);
        let high = self.fetch_byte(bus);
        u16::from_le_bytes([low, high])
    }

    fn read_pair(&self, pair: u8) -> u16 {
        match pair {
            0 => self.bc(),
            1 => self.de(),
            2 => self.hl(),
            3 => self.sp,
            _ => unreachable!(),
        }
    }

    fn write_pair(&mut self, pair: u8, value: u16) {
        match pair {
            0 => self.set_bc(value),
            1 => self.set_de(value),
            2 => self.set_hl(value),
            3 => self.sp = value,
            _ => unreachable!(),
        }
    }

    fn set_bc(&mut self, value: u16) {
        [self.b, self.c] = value.to_be_bytes();
    }

    fn set_de(&mut self, value: u16) {
        [self.d, self.e] = value.to_be_bytes();
    }

    fn set_hl(&mut self, value: u16) {
        [self.h, self.l] = value.to_be_bytes();
    }

    fn read_register<B: Bus>(&self, register: u8, bus: &B) -> u8 {
        match register {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => bus.read(self.hl()),
            7 => self.a,
            _ => unreachable!(),
        }
    }

    fn write_register<B: Bus>(&mut self, register: u8, value: u8, bus: &mut B) {
        match register {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => bus.write(self.hl(), value),
            7 => self.a = value,
            _ => unreachable!(),
        }
    }

    fn push<B: Bus>(&mut self, bus: &mut B, value: u16) {
        self.sp = self.sp.wrapping_sub(2);
        write_word(bus, self.sp, value);
    }

    fn pop<B: Bus>(&mut self, bus: &B) -> u16 {
        let value = read_word(bus, self.sp);
        self.sp = self.sp.wrapping_add(2);
        value
    }

    fn condition(&self, condition: u8) -> bool {
        match condition {
            0 => !self.flags.zero,
            1 => self.flags.zero,
            2 => !self.flags.carry,
            3 => self.flags.carry,
            4 => !self.flags.parity,
            5 => self.flags.parity,
            6 => !self.flags.sign,
            7 => self.flags.sign,
            _ => unreachable!(),
        }
    }

    fn set_szp(&mut self, value: u8) {
        self.flags.sign = value & 0x80 != 0;
        self.flags.zero = value == 0;
        self.flags.parity = value.count_ones().is_multiple_of(2);
    }

    fn alu(&mut self, operation: u8, value: u8) {
        match operation {
            0 => self.add(value, false),
            1 => self.add(value, true),
            2 => self.subtract(value, false, false),
            3 => self.subtract(value, true, false),
            4 => {
                self.a &= value;
                self.flags.carry = false;
                self.flags.auxiliary_carry = true;
                self.set_szp(self.a);
            }
            5 => {
                self.a ^= value;
                self.flags.carry = false;
                self.flags.auxiliary_carry = false;
                self.set_szp(self.a);
            }
            6 => {
                self.a |= value;
                self.flags.carry = false;
                self.flags.auxiliary_carry = false;
                self.set_szp(self.a);
            }
            7 => self.subtract(value, false, true),
            _ => unreachable!(),
        }
    }

    fn add(&mut self, value: u8, with_carry: bool) {
        let carry = u8::from(with_carry && self.flags.carry);
        let wide = self.a as u16 + value as u16 + carry as u16;
        self.flags.carry = wide > 0xff;
        self.flags.auxiliary_carry =
            (self.a & 0x0f) as u16 + (value & 0x0f) as u16 + carry as u16 > 0x0f;
        self.a = wide as u8;
        self.set_szp(self.a);
    }

    fn subtract(&mut self, value: u8, with_borrow: bool, compare_only: bool) {
        let borrow = u8::from(with_borrow && self.flags.carry);
        let subtrahend = value as u16 + borrow as u16;
        let result = self.a.wrapping_sub(value).wrapping_sub(borrow);
        self.flags.carry = (self.a as u16) < subtrahend;
        self.flags.auxiliary_carry = (self.a & 0x0f) < (value & 0x0f).wrapping_add(borrow);
        self.set_szp(result);
        if !compare_only {
            self.a = result;
        }
    }

    fn daa(&mut self) {
        let mut correction = 0;
        if self.a & 0x0f > 9 || self.flags.auxiliary_carry {
            correction |= 0x06;
        }
        let high_carry = self.a > 0x99 || self.flags.carry;
        if high_carry {
            correction |= 0x60;
        }
        let low = self.a & 0x0f;
        self.a = self.a.wrapping_add(correction);
        self.flags.auxiliary_carry = low + (correction & 0x0f) > 0x0f;
        self.flags.carry |= high_carry;
        self.set_szp(self.a);
    }
}

fn read_word<B: Bus>(bus: &B, address: u16) -> u16 {
    u16::from_le_bytes([bus.read(address), bus.read(address.wrapping_add(1))])
}

fn write_word<B: Bus>(bus: &mut B, address: u16, value: u16) {
    let [low, high] = value.to_le_bytes();
    bus.write(address, low);
    bus.write(address.wrapping_add(1), high);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBus {
        memory: [u8; 65_536],
        ports: [u8; 256],
    }

    impl Default for TestBus {
        fn default() -> Self {
            Self {
                memory: [0; 65_536],
                ports: [0; 256],
            }
        }
    }

    impl Bus for TestBus {
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

    fn run(program: &[u8]) -> (Cpu, TestBus) {
        let mut cpu = Cpu::default();
        let mut bus = TestBus::default();
        bus.memory[..program.len()].copy_from_slice(program);
        for _ in 0..100 {
            if cpu.halted {
                break;
            }
            cpu.step(&mut bus);
        }
        (cpu, bus)
    }

    #[test]
    fn arithmetic_sets_flags() {
        let (cpu, _) = run(&[0x3e, 0x0f, 0x37, 0x3c, 0xc6, 0xf0, 0x76]);
        assert_eq!(cpu.a, 0);
        assert!(cpu.flags.zero);
        assert!(cpu.flags.carry);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(cpu.flags.parity);
    }

    #[test]
    fn inr_preserves_carry_and_sets_auxiliary_carry() {
        let (cpu, _) = run(&[0x3e, 0x0f, 0x37, 0x3c, 0x76]);
        assert_eq!(cpu.a, 0x10);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.auxiliary_carry);
    }

    #[test]
    fn calls_and_returns_use_the_stack() {
        let (cpu, bus) = run(&[
            0x31, 0x00, 0x20, 0xcd, 0x08, 0x00, 0x76, 0x00, 0x3e, 0x42, 0xc9,
        ]);
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.sp, 0x2000);
        assert_eq!(bus.memory[0x1ffe], 0x06);
        assert_eq!(bus.memory[0x1fff], 0x00);
    }

    #[test]
    fn register_pairs_and_memory_use_little_endian_words() {
        let (cpu, bus) = run(&[
            0x21, 0x34, 0x12, 0x22, 0x00, 0x40, 0x21, 0x00, 0x00, 0x2a, 0x00, 0x40, 0x76,
        ]);
        assert_eq!(cpu.hl(), 0x1234);
        assert_eq!(bus.memory[0x4000], 0x34);
        assert_eq!(bus.memory[0x4001], 0x12);
    }

    #[test]
    fn psw_round_trip_restores_flags() {
        let (cpu, _) = run(&[
            0x31, 0x00, 0x20, 0x3e, 0x00, 0xb7, 0x37, 0xf5, 0x3e, 0xff, 0x3f, 0xf1, 0x76,
        ]);
        assert_eq!(cpu.a, 0);
        assert!(cpu.flags.zero);
        assert!(cpu.flags.carry);
    }

    #[test]
    fn io_instructions_access_ports() {
        let (cpu, bus) = run(&[0x3e, 0x5a, 0xd3, 0x10, 0x3e, 0x00, 0xdb, 0x10, 0x76]);
        assert_eq!(bus.ports[0x10], 0x5a);
        assert_eq!(cpu.a, 0x5a);
    }

    #[test]
    fn every_base_opcode_can_execute_without_panicking() {
        for opcode in 0..=u8::MAX {
            let mut cpu = Cpu::default();
            let mut bus = TestBus::default();
            bus.memory[0] = opcode;
            bus.memory[1] = 0;
            bus.memory[2] = 0;
            cpu.step(&mut bus);
        }
    }
}
