use serde::Serialize;
use specialist_core::{Cpu, Flags, Machine, Snapshot};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FlagsOutput {
    sign: bool,
    zero: bool,
    auxiliary_carry: bool,
    parity: bool,
    carry: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CpuOutput {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    flags: FlagsOutput,
    interrupts_enabled: bool,
    halted: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotOutput {
    cpu: CpuOutput,
    cycles: u64,
    frame: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonOutput<'a> {
    snapshot: SnapshotOutput,
    framebuffer: &'a [u8],
}

impl From<Flags> for FlagsOutput {
    fn from(flags: Flags) -> Self {
        Self {
            sign: flags.sign,
            zero: flags.zero,
            auxiliary_carry: flags.auxiliary_carry,
            parity: flags.parity,
            carry: flags.carry,
        }
    }
}

impl From<Cpu> for CpuOutput {
    fn from(cpu: Cpu) -> Self {
        Self {
            a: cpu.a,
            b: cpu.b,
            c: cpu.c,
            d: cpu.d,
            e: cpu.e,
            h: cpu.h,
            l: cpu.l,
            sp: cpu.sp,
            pc: cpu.pc,
            flags: cpu.flags.into(),
            interrupts_enabled: cpu.interrupts_enabled,
            halted: cpu.halted,
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    run_arguments(std::env::args().skip(1).collect())
}

fn run_arguments(arguments: Vec<String>) -> Result<(), String> {
    let mut machine = Machine::new();
    let maximum_instructions;

    match arguments.first().map(String::as_str) {
        Some("demo") => {
            machine.load_demo();
            maximum_instructions = parse_limit(arguments.get(1))?;
        }
        Some("run") => {
            let path = arguments.get(1).ok_or_else(usage)?;
            let address = parse_number(arguments.get(2).ok_or_else(usage)?)?;
            let entry_point = parse_number(arguments.get(3).ok_or_else(usage)?)?;
            let program = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
            machine
                .load_program(address, entry_point, &program)
                .map_err(|error| error.to_string())?;
            maximum_instructions = parse_limit(arguments.get(4))?;
        }
        Some("monitor2") => {
            let path = arguments.get(1).ok_or_else(usage)?;
            let rom = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
            machine
                .load_monitor2_rom(&rom)
                .map_err(|error| error.to_string())?;
            maximum_instructions = parse_limit(arguments.get(2))?;
        }
        _ => return Err(usage()),
    }

    let json = match arguments.first().map(String::as_str) {
        Some("monitor2") => match arguments.get(3).map(String::as_str) {
            Some("--json") => true,
            Some(_) => return Err(usage()),
            None => false,
        },
        _ => false,
    };
    let result = machine
        .run(maximum_instructions)
        .map_err(|error| error.to_string())?;
    if json {
        print_json(&machine);
    } else {
        print_snapshot(&machine.snapshot());
        println!(
            "executed {} instructions, {} cycles{}",
            result.instructions,
            result.cycles,
            if result.halted { ", halted" } else { "" }
        );
    }
    Ok(())
}

fn print_json(machine: &Machine) {
    let snapshot = machine.snapshot();
    let output = JsonOutput {
        snapshot: SnapshotOutput {
            cpu: snapshot.cpu.into(),
            cycles: snapshot.cycles,
            frame: snapshot.frame,
        },
        framebuffer: machine.framebuffer(),
    };
    println!(
        "{}",
        serde_json::to_string(&output).expect("serializing the CLI snapshot")
    );
}

fn parse_limit(value: Option<&String>) -> Result<u32, String> {
    value.map_or(Ok(200_000), |value| {
        value
            .parse()
            .map_err(|_| format!("invalid instruction limit: {value}"))
    })
}

fn parse_number(value: &str) -> Result<u32, String> {
    if let Some(hex) = value.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).map_err(|_| format!("invalid address: {value}"))
    } else {
        value
            .parse()
            .map_err(|_| format!("invalid address: {value}"))
    }
}

fn print_snapshot(snapshot: &Snapshot) {
    let cpu = snapshot.cpu;
    println!(
        "A={:02X} BC={:02X}{:02X} DE={:02X}{:02X} HL={:02X}{:02X} SP={:04X} PC={:04X}",
        cpu.a, cpu.b, cpu.c, cpu.d, cpu.e, cpu.h, cpu.l, cpu.sp, cpu.pc
    );
    println!(
        "flags: S={} Z={} AC={} P={} C={}; cycles={}",
        u8::from(cpu.flags.sign),
        u8::from(cpu.flags.zero),
        u8::from(cpu.flags.auxiliary_carry),
        u8::from(cpu.flags.parity),
        u8::from(cpu.flags.carry),
        snapshot.cycles
    );
}

fn usage() -> String {
    "usage:\n  specialist demo [instruction-limit]\n  specialist run <file> <address> <entry-point> [instruction-limit]\n  specialist monitor2 <rom-file> [instruction-limit] [--json]\naddresses accept decimal or 0x-prefixed hexadecimal values".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor2_command_runs_an_external_rom() {
        let path =
            std::env::temp_dir().join(format!("specialist-monitor2-{}.rom", std::process::id()));
        let mut rom = vec![0; 0x1800];
        rom[0] = 0x76;
        std::fs::write(&path, rom).unwrap();

        let result = run_arguments(vec![
            "monitor2".into(),
            path.to_string_lossy().into_owned(),
            "1".into(),
        ]);
        std::fs::remove_file(path).ok();
        assert!(result.is_ok());
    }
}
