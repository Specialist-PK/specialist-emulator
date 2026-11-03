# Specialist Emulator roadmap

[Русская версия](roadmap.md)

## Planning principles

The roadmap is not tied to calendar dates. Each stage has a verifiable outcome; moving forward does not require completing every optional improvement from the previous stage.

Architectural constraints are described in [architecture.en.md](architecture.en.md). The editor extension is developed in [specialist-zed](https://github.com/Specialist-PK/specialist-zed).

## Current state

The architectural prototype is complete:

- basic Intel 8080 with all 256 standard opcodes;
- flat 64 KiB RAM, ports, and video memory;
- CLI, WebAssembly, and Tauri 2;
- shared Solid UI and a TypeScript contract;
- built-in demo and BIN loading;
- unit, WASM, native, and browser checks;
- native CLI debugging in Zed through CodeLLDB.

This is not yet a complete model of a real Specialist. Monitor ROM, keyboard, tape, sound, exact timing, clone configurations, and DAP are not implemented.

## Stage 1. Verifiable base machine

**Goal:** replace the flat demonstration model with a documented Specialist configuration.

Tasks:

- describe the reference configuration, memory map, ports, and information sources;
- separate RAM, ROM, video memory, and peripheral registers;
- load ROM externally with size validation and an explicit model selection;
- implement the keyboard matrix and basic system controller;
- add deterministic input events;
- retain the built-in demo as an independent ROM-free test mode.

Acceptance criteria:

- one test program produces the same state and output in CLI and WASM;
- memory-map and port boundaries are covered by tests;
- documentation distinguishes confirmed behavior from assumptions;
- a missing ROM produces a clear error rather than an implicit model change.

## Stage 2. Timing, video, sound, and tape

**Goal:** reproduce time-dependent machine behavior.

Tasks:

- connect CPU cycles to timers and peripherals;
- implement video frames and modes of the selected configuration;
- add an audio path without allowing audio output to affect the core;
- implement tape reading and writing through a documented format;
- define a deterministic snapshot format;
- add performance measurements without changing cycle logic.

Acceptance criteria:

- repeated runs with identical events produce identical snapshots;
- long programs do not overflow counters or lose precision at boundaries;
- video and sound are tested independently of the CPU;
- saving and restoring does not change subsequent execution.

## Stage 3. Debug core

**Goal:** provide platform-independent operations required by a debugger.

Tasks:

- add an execution controller and stop reasons;
- implement address breakpoints and instruction stepping;
- add an Intel 8080 disassembler;
- expose registers, flags, memory, and stack reads;
- define source and symbol map formats for assembly programs;
- cover command sequences and boundary states with tests.

Acceptance criteria:

- a breakpoint triggers before its instruction and is not skipped after continue;
- pause and termination never leave the machine in an inconsistent state;
- the disassembler handles the `0xFFFF` boundary correctly;
- missing sources do not prevent machine-code debugging.

## Stage 4. DAP adapter

**Goal:** release `specialist-dap` as a separate binary in the emulator workspace.

Tasks:

- implement DAP over stdio;
- support `initialize`, `launch`, breakpoint configuration, execution, and state reads;
- define a scenario JSON Schema and configuration protocol version;
- add version diagnostics and clear launch errors;
- explicitly reject unsupported `attach` until its semantics are implemented;
- prepare reproducible release binaries for supported operating systems.

Acceptance criteria:

- a protocol test covers the path from initialize to terminate;
- the adapter does not use UI and works without Zed;
- invalid addresses, programs, and configuration versions return DAP errors;
- repeated and parallel sessions do not share machine state.

## Stage 5. Zed integration

**Goal:** debug Specialist programs from the editor through [specialist-zed](https://github.com/Specialist-PK/specialist-zed).

Tasks on the emulator side:

- stabilize the `specialist-dap` path and arguments;
- publish a schema and `debug.json` examples;
- maintain adapter/extension version compatibility;
- provide a test program and expected states for end-to-end verification.

Acceptance criteria:

- a user installs the dev extension and runs the test program without manually building the core;
- breakpoints, continue, pause, step, registers, memory, and disassembly work;
- launch errors are visible and contain sufficient diagnostics;
- independent Rust emulator debugging through CodeLLDB continues to work.

## Stage 6. Application scenarios and releases

**Goal:** make the emulator a convenient standalone tool.

Tasks:

- add keyboard, tape, and peripheral controls to web and desktop;
- implement configuration selection and machine diagnostics;
- configure CI for Rust, TypeScript, WASM, native, and E2E;
- automate dependency, license, and artifact checks;
- prepare web deployment and native releases;
- document supported OS, Zed, and browser versions.

Acceptance criteria:

- a clean checkout builds using the documented instructions;
- releases contain verified artifacts and checksums;
- web and desktop pass one shared set of functional scenarios;
- public contract changes include migration notes.

## Later directions

After the base machine stabilizes:

- additional models, clones, and peripheral controllers;
- execution profiling and tracing;
- shared test suites with `special20k`;
- assembly and symbol-map generation tools;
- research snapshots, execution history, and run comparisons.

Every new direction must preserve core independence and must not require Zed to use the emulator.
