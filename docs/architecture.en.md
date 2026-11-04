# Specialist Emulator architecture

[Русская версия](architecture.md)

## Purpose of this document

This document records the current project boundaries and the target architecture of a complete Specialist emulator. Staged development plans are in [roadmap.en.md](roadmap.en.md).

“Implemented” refers to code in the repository. Sections about the target architecture describe proposed solutions and do not imply that those components already exist.

## Goals and constraints

Primary goals:

- one deterministic core for CLI, browser, native application, and debugger;
- reproducible behavior of specific Specialist configurations, not only an abstract Intel 8080;
- identical execution results on native platforms and in WebAssembly;
- independent core execution without a graphical interface;
- debugging programs inside the emulator through Debug Adapter Protocol (DAP);
- debugging the emulator itself with standard native Rust tools.

The project does not aim to move the entire interface into an editor extension. The display, keyboard, and peripherals remain part of emulator applications; Zed uses the standard debugging interface.

## Overall scheme

```text
                         ┌──────────────────────────┐
                         │      specialist-core     │
                         │ CPU · Bus · Machine · RAM│
                         └────────────┬─────────────┘
                                      │
              ┌───────────────────────┼────────────────────────┐
              │                       │                        │
       specialist-cli       specialist-core-wasm       native/Tauri IPC
              │                       │                        │
          terminal                 web adapter             desktop adapter
                                      │                        │
                                      └──────────┬─────────────┘
                                                 │
                                     shared Solid UI + core-api

Zed → specialist-zed → stdio/DAP → specialist-dap → specialist-core
```

`specialist-dap` and Zed integration belong to the target architecture. The extension is developed in [specialist-zed](https://github.com/Specialist-PK/specialist-zed).

## Current components

| Component            | Responsibility                                                                                      |
| -------------------- | --------------------------------------------------------------------------------------------------- |
| `crates/core`        | Intel 8080, registers, flags, stack, Demo/Monitor 2 memory maps, ports, `Machine`, and video memory |
| `crates/core-wasm`   | Thin `wasm-bindgen` boundary around `Machine`                                                       |
| `crates/cli`         | Console execution of the demo, BIN files, and an external Monitor 2 ROM                             |
| `packages/core-api`  | Independent TypeScript `Emulator` contract, state snapshots, and errors                             |
| `packages/core-wasm` | WASM initialization and core adaptation to `Emulator`                                               |
| `packages/app`       | Shared Solid UI: display, registers, and execution controls                                         |
| `apps/web`           | Browser application entry point                                                                     |
| `apps/native`        | Tauri entry point, native IPC, and `Machine` ownership                                              |
| `tests/e2e`          | End-to-end browser UI verification in Chromium                                                      |

Dependency direction:

```text
CLI and Tauri → specialist-core
Web           → specialist-core-wasm → specialist-core
Web and Tauri → @specialist/app → @specialist/core-api
```

Reverse dependencies are forbidden. Shared UI knows nothing about WASM, Tauri, or a specific application.

## Execution model

### Processor and memory

`Machine` combines the CPU, system bus, selected memory map, and cycle counter. All 256 base Intel 8080 opcodes and 256 I/O ports are implemented. Video memory starts at `0x9000`; the display is 384 × 256 pixels.

`Demo` mode retains flat RAM for the built-in program and arbitrary BIN files. In `Monitor2` mode, `0000–BFFF` is writable RAM and `C000–D7FF` is write-protected ROM. An external ROM of exactly `0x1800` bytes is loaded at `C000`; execution without the required ROM returns an error before the first instruction.

The KR580VV55A system PPI is connected to the bus at `F800–FFFF`, with registers repeated every 4 bytes. The keyboard matrix and `KeyEvent` queue live in the core and do not depend on system clocks. The shared Solid UI translates `KeyboardEvent.code` into logical keys and serializes events. Unassigned addresses, JCUKEN layout, and electrical ghosting remain separate tasks.

Memory and port access goes through the `Bus` abstraction. New devices must connect through it rather than bypass it with direct CPU access.

### Determinism

For one initial state, configuration, and sequence of external events, execution results must match on every platform. The core does not read system clocks, filesystem, or DOM and does not use platform multithreading.

Time is defined by machine cycles and events. Audio and video output consume core state and must not affect execution.

The cycle counter is `u64`. It is passed as a string across TypeScript boundaries and normalized by adapters to avoid precision loss.

### Execution limits and safety

Batch execution has an instruction limit. It must not be removed: otherwise a loaded program can block CLI, the WASM thread, or a native call.

All addresses and ranges are checked at platform boundaries. BIN loading must not silently overwrite memory outside the 16-bit address space.

## Target machine model

The first reference configuration is defined in [reference-machine.en.md](reference-machine.en.md) and [ADR-0011](adr/0011-reference-specialist-configuration.en.md).

The transition from a demonstration computer to specific models should separate:

- `Cpu` — Intel 8080 execution;
- memory map — RAM, ROM, video memory, and banking windows;
- peripheral controllers — keyboard, timer, tape, sound, and disk interfaces;
- video controller — memory reads, scanline generation, and frame production;
- `MachineConfig` — model identifier, memory map, frequencies, and devices;
- `Machine` — component composition and deterministic execution loop.

Configurations should be data or small factories rather than scattered conditionals inside the CPU. Model names and features must be supported by sources; unverified assumptions are marked in documentation.

ROM is loaded separately from the core. Only materials with confirmed redistribution rights are added to the repository.

## Platform adapters

### WebAssembly

`crates/core` remains compatible with `wasm32-unknown-unknown`. `crates/core-wasm` only exports `Machine` capabilities; duplicating processor or device logic in the adapter is forbidden.

The generated `packages/core-wasm/pkg` directory is not source code and is not committed.

### Tauri

Tauri owns one `Machine` through `AppState` and `Mutex`. Commands accept and return typed data, and the TypeScript adapter implements the same `Emulator` as WASM.

Adding a command requires synchronized updates to the command, Specta registration, the `build.rs` list, both capabilities, and generated bindings.

### CLI and embedding

CLI uses the core directly and does not depend on Bun, Vite, or Tauri. This path is also the basis for test harnesses and the future DAP process.

## Debugging architecture

Two tasks must be distinguished:

1. **Debugging a Specialist program.** Performed through a dedicated DAP adapter controlling `Machine`.
2. **Debugging the emulator implementation.** Performed through CodeLLDB or another native Rust debugger; the existing `.zed/debug.json` belongs to this task.

Target program-debugging scheme:

```text
┌─────┐   extension API   ┌────────────────┐   stdio + DAP   ┌────────────────┐
│ Zed │ ────────────────→ │ specialist-zed │ ──────────────→ │ specialist-dap │
└─────┘                   └────────────────┘                 └───────┬────────┘
                                                                      │
                                                             ┌────────▼────────┐
                                                             │ DebugController │
                                                             │ + Machine       │
                                                             └─────────────────┘
```

`specialist-dap` must live in the emulator workspace and use `specialist-core` directly. The Zed extension contains no second emulator implementation and does not implement DAP over unstable TypeScript IPC.

### Debug session states

The minimum model: `launch` creates a machine, loads a program, and leaves it stopped or runs it to the first breakpoint. The DAP adapter processes commands sequentially and publishes stop and termination events.

Initially one emulated process and one logical thread are supported. `attach` to an existing instance appears only after transport and session ownership are defined; creating a new machine must not be presented as attach.

### Debug operations

Base contract:

- set and remove a breakpoint at a 16-bit address;
- stop before executing the instruction at the breakpoint;
- continue, pause, instruction step, and reset;
- registers, flags, stack, and memory reads;
- disassembly around PC;
- clean session termination without leftover processes.

Intel 8080 does not provide enough information for reliable call-stack reconstruction for an arbitrary program. The initial version should show one current pseudo-frame instead of inventing a call stack. CALL/RET analysis can be added later as a separate, explicitly approximate feature.

### Sources and addresses

DAP works with source files and addresses. For BIN without debug information, disassembly is the primary mode. Linking to assembly sources requires a separate address or symbol map format; this format must be versioned and independent of Zed.

## Contract compatibility

Stable integration points are:

- the public `specialist-core` API for Rust components;
- the TypeScript `Emulator` interface for applications;
- the `specialist-dap` executable, its arguments, and the Zed scenario JSON Schema;
- versioned machine configuration, symbol, and snapshot formats.

Changing DAP configuration requires synchronized updates to the schema, documentation, and compatibility with `specialist-zed`. The adapter version must be available to the extension for diagnostics.

## Architecture verification

Minimum verification levels:

1. CPU, memory, and device unit tests.
2. Shared executable test vectors for native core and WASM.
3. Tauri IPC and TypeScript contract integration tests.
4. End-to-end UI tests.
5. DAP protocol tests: initialize → launch → breakpoint → continue → variables/stackTrace → terminate.
6. An end-to-end scenario with real Zed and the dev extension.

Shared test programs and results from [special20k](https://github.com/Specialist-PK/special20k) are useful for hardware compatibility, but differences must first be checked against real machine documentation rather than automatically treating either implementation as the reference.

## Decisions

- The core remains in `specialist-emulator`; the editor extension does not own emulation.
- Zed integration uses an external process and standard DAP.
- Browser and native UI share the contract but not platform adapters.
- A verifiable machine model and debugging primitives come before editor integration.
- Accuracy and reproducibility matter more than speed; optimizations must not change observable behavior without updating tests and documentation.
