# 0011. Journal Specialist with Monitor 2 as the first reference configuration

[Русская версия](0011-reference-specialist-configuration.md)

Status: accepted (2026-11-03)

## Context

The current core provides a flat 64 KiB RAM and does not distinguish real machines. A verifiable model needs documented addresses, ROM, and keyboard behavior without mixing incompatible variants.

Research produced three evidence levels: the 1987-02 journal schematic, Aleksey Morozov's Monitor 2 memory map, and Emu80's implementation contract. Some Yandex Disk issues are unavailable, so implementation details are explicitly separated from journal-confirmed facts.

## Decision

The first configuration is the journal Specialist with Monitor 2:

- 36 KiB RAM at `0000–8FFF`, including the conventionally reserved `4000–7FFF`, and 12 KiB video memory at `9000–BFFF`;
- 6 KiB ROM at `C000–D7FF`: loader, Monitor 2, and character generator;
- system KR580VV55A at `F800–FFFF` with registers repeated every 4 bytes;
- a 12 × 6 keyboard matrix plus Shift, active-low values, and two-pass scanning;
- a separate explicit ROM-free `demo` that does not imitate a real machine boot.

Required ROM is `0x1800` bytes and starts at `C000`. Other sizes require separate configurations. Missing ROM is an error before the first instruction, not a reason to switch models implicitly.

The full specification is in `docs/reference-machine.md` and `.en.md`. Emu80 rules are verified by tests; MX, Monitor 1, color, and additional controllers are not mixed into the reference.

## Alternatives

- **Keep an abstract i8080.** Simpler, but does not provide compatibility with a specific Specialist.
- **Choose MX.** More devices and memory, but a different map and more unverified decisions for the first stage.
- **Choose Monitor 1.** Historically valid, but Monitor 2 is described as more compatible.
- **Treat Emu80 as the sole reference.** Faster, but hides implementation differences and proves nothing by itself.

## Consequences

- (+) Memory, ROM, keyboard, and tests get a narrow verifiable contract.
- (+) Reserved RAM remains usable, and the ROM-free demo is not confused with a Monitor boot.
- (+) Unproven electrical and open-bus details remain explicitly open.
- (−) Some journal issues are unavailable, so further comparison with scans is required.
- (−) The reference is limited to Monitor 2 and does not promise automatic clone support.
