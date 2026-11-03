use specialist_core::{Machine, Snapshot};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
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
        _ => return Err(usage()),
    }

    let result = machine.run(maximum_instructions);
    print_snapshot(&machine.snapshot());
    println!(
        "executed {} instructions, {} cycles{}",
        result.instructions,
        result.cycles,
        if result.halted { ", halted" } else { "" }
    );
    Ok(())
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
    "usage:\n  specialist demo [instruction-limit]\n  specialist run <file> <address> <entry-point> [instruction-limit]\naddresses accept decimal or 0x-prefixed hexadecimal values".into()
}
