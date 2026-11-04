# Reference Specialist configuration

[Русская версия](reference-machine.md)

## Purpose and status

This document defines the first reference configuration for Stage 1: the journal Specialist with Monitor 2. It is a narrow, verifiable model rather than a union of every clone's features. Monitor 2 was selected because Aleksey Morozov describes it as the most compatible version for existing programs.

Evidence levels differ:

- **Journal schematic** — CPU, RAM, video memory, resolution, and software keyboard handling are confirmed by the 1987-02 Modelist-Konstruktor scan.
- **Monitor 2** — ROM composition and addresses are confirmed by Morozov's dedicated memory map.
- **Registers and bits** — taken from Emu80 as an implementation contract to be verified by tests.
- It is **not claimed** that every electrical detail and every Specialist variant is already proven.

## Base parameters

| Parameter | Reference value |
| --- | --- |
| Model | Journal Specialist with Monitor 2 |
| CPU | KR580VM80A / Intel 8080, 2 MHz |
| RAM | 48 KiB: 36 KiB at `0000–8FFF` and 12 KiB video memory at `9000–BFFF` |
| ROM | 6 KiB at `C000–D7FF`: loader, Monitor 2, character generator |
| Display | Monochrome, 384 × 256 pixels |
| System interface | KR580VV55A at `F800–FFFF` |
| Keyboard | 12 × 6 matrix plus a separate Shift |
| Sound and tape | Deferred to Stage 2 except for required system-port lines |

The 32 KiB RAM variant, MX, Monitor 1, color upgrades, SD, the VI53 timer, and additional controllers are outside this configuration.

## Memory map

| Range | Purpose | Source and notes |
| --- | --- | --- |
| `0000–3FFF` | 16 KiB user RAM | Journal schematic |
| `4000–7FFF` | 16 KiB RAM marked as reserved | Journal schematic; this is not a separate device or a prohibition on program use |
| `8000–8FFF` | 4 KiB user RAM | Journal schematic; Monitor 2 also uses part of upper RAM for system purposes |
| `9000–BFFF` | 12 KiB video RAM | Journal schematic and video description |
| `C000–C7FF` | 2 KiB loader ROM | Monitor 2 memory map |
| `C800–CFFF` | 2 KiB Monitor 2 ROM | Monitor 2 memory map |
| `D000–D7FF` | 2 KiB character-generator ROM | Monitor 2 memory map |
| `D800–EFFF` | Undefined for this reference | Not to be used without a separate configuration |
| `F000–F7FF` | Optional printer port | Outside the base configuration |
| `F800–FFFF` | Keyboard and system KR580VV55A port | Monitor 2 memory map |

Reserved RAM remains RAM: the configuration must not return a special value or reject writes. Conventional usage labels do not change the physical address map.

## System port and registers

The KR580VV55A port is memory-mapped rather than accessed through the 8-bit `IN`/`OUT` space.

| Address | Purpose |
| --- | --- |
| `F800` | Port A: keyboard matrix rows 0–7 |
| `F801` | Port B: bits 2–7 are matrix columns; bit 1 is Shift; bit 0 is tape input |
| `F802` | Port C: bits 0–3 are rows 8–11; bit 5 is sound output; bit 7 is tape output |
| `F803` | KR580VV55A control register |

Emu80 repeats these four registers every 4 bytes throughout `F800–FFFF`. This is accepted as an implementation rule to be tested; the journal schematic does not independently confirm it.

Pressed keys and releases are represented by active-low values. Port direction is set through the control register. The matrix is scanned in two passes, rows then columns; an unpressed line reads as `1`.

Port C color bits belong to upgrades and are not used by the monochrome reference.

## Keyboard contract

State is stored as 12 rows of 6 bits plus Shift. An external event is OS-independent:

```text
KeyEvent {
  key: key or Shift identifier,
  pressed: true | false,
  cycle?: emulator cycle number
}
```

Rules:

- event sequences are deterministic; `cycle` is supplied only by the caller, and the core never reads system clocks;
- press sets the key bit, release clears it;
- Shift is stored separately and inverted as active-low when port B is read;
- identical events from an identical initial state produce identical port values;
- simultaneous presses remain in matrix state, but limitations of the two-pass scan are not hidden.

Emu80's `original` matrix maps logical keys to cells. MX, Lik, Eureka, and SP-580 layouts do not replace it automatically.

## ROM and ROM-free mode

For `specialist-monitor2`, the ROM file is exactly `0x1800` bytes (6 KiB), loads at `C000`, and starts at `C000`:

1. `C000–C7FF` — loader 2;
2. `C800–CFFF` — Monitor 2;
3. `D000–D7FF` — character generator 2.

Monitor 2 and the character generator are optional in some machines, but the reference includes them for predictable compatibility. A different ROM size is not accepted silently: a separate loader, Monitor 1, or extended-ROM configuration is required.

A missing required ROM stops startup with a clear error before any instruction executes. The model must not change implicitly.

`demo` is a separate explicit ROM-free mode with its own program and entry point `0000`. It does not claim Monitor ROM compatibility or imitate a real Specialist boot.

ROM images are not added to the repository until redistribution rights are confirmed.

## Exclusions and open questions

- Exact video/audio timing, tape, and state snapshots — Stage 2;
- Additional controllers, SD, VI53 timer, second printer port, and color — separate configurations;
- Reads from unmapped addresses and open-bus behavior — currently unspecified;
- Electrical ghosting with multiple keys — modeled only after separate verification;
- ROM rights and the origin of each image require separate confirmation.

## Verification

After implementing memory and ports, verify:

1. reads and writes in every RAM region, including the reserved area;
2. rejection of ROM writes without a special mode;
3. pixel order: `col * 256 + row`, most significant bit leftmost, `1` meaning a lit pixel;
4. register repetition every 4 bytes and independence of `F803` from data registers;
5. press, release, Shift, and simultaneous keys in both matrix directions;
6. identical state and framebuffer in CLI and WASM for one test program;
7. a clear missing-ROM error before the first instruction and no implicit model change.

## Sources

- [Modelist-Konstruktor 1987-02, pages 20–22: memory map, specifications, and keyboard](https://disk.yandex.ru/d/zVvjNM8_13_BMg).
- [Aleksey Morozov: Specialist — RAM, video, Monitor 2, and keyboard scanning](https://alemorf.github.io/retro_computers/computer.html?id=Specialist).
- [Aleksey Morozov: Monitor 2 — memory map and ROM composition](https://alemorf.github.io/retro_computers/Specialist/russian.monitor2.html).
- [Emu80: `spec.conf` — RAM/ROM, device ranges, and start address](https://github.com/vpyk/emu80v4/blob/master/dist/spec/spec.conf).
- [Emu80: `Specialist.cpp` and `Specialist.h` — system port and keyboard matrix](https://github.com/vpyk/emu80v4/tree/master/src).
- [special20k — a separate FPGA configuration, not the reference for the original map](https://github.com/Specialist-PK/special20k/blob/main/docs/architecture.md).
