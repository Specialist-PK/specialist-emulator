# Specialist Emulator

[Русская версия](README.md)

[![CI](https://github.com/Specialist-PK/specialist-emulator/actions/workflows/ci.yml/badge.svg)](https://github.com/Specialist-PK/specialist-emulator/actions/workflows/ci.yml)

A cross-platform emulator of the Soviet Specialist home computer. One Rust core runs in a console application, in the browser through WebAssembly, and in a Tauri 2 desktop application.

The project builds on the architecture and development process of [another-useless-app](https://github.com/DrUNE/another-useless-app): independent builds, shared UI, thin platform adapters, a single core contract, documented decisions, and reproducible checks.

## Current capabilities

- Base set of 256 Intel 8080 opcodes, registers, flags, and I/O ports;
- 64 KiB address space and instruction stepping;
- Original Specialist memory map and external loading of a 6 KiB Monitor 2 ROM;
- KR580VV55A system PPI, `original` keyboard matrix, and deterministic key events;
- Monochrome 384 × 256 display with video memory at `0x9000`;
- Run, pause, reset, and single-step controls;
- Registers, flags, and cycle counter display;
- BIN and Monitor 2 ROM loading through the shared UI;
- Physical QWERTY keyboard input with keys released on focus loss;
- Built-in i8080 demo that fills video memory;
- Shared Solid UI for web and desktop;
- Matching emulation scenarios through WASM and Tauri IPC.

This is an early i8080 core, not yet a complete model of a specific Specialist. The memory map, Monitor 2 ROM, system PPI, keyboard matrix, and QWERTY host bindings are implemented. JCUKEN layout, tape, sound, computer variants, exact timing, and a DAP debugger are not. Third-party ROMs and programs are not included.

## Structure

```text
crates/
  core/              # i8080, memory, machine, and demo
  core-wasm/         # thin wasm-bindgen wrapper
  cli/               # console application, specialist binary
apps/
  web/               # browser entry point: WASM + shared UI
  native/            # Tauri 2: IPC + shared UI
packages/
  core-api/          # shared TypeScript Emulator contract
  core-wasm/         # WASM build and contract adapter
  app/               # shared Solid UI
  config/            # shared TypeScript and Tailwind configuration
tools/
  ci/                # tested CI job selection
  doctor/            # environment and repository consistency checks
  boundaries/        # declared dependency boundary checks
  dev-server/        # background Vite for editor debugging
  system-deps/       # single Debian/Ubuntu package list
```

Dependency direction:

```text
CLI and Tauri → specialist-core
Web           → specialist-core-wasm → specialist-core
Web and Tauri → shared UI → core-api
```

The core does not use filesystem, DOM, Tauri, or other platform APIs. Browser and native implementations share the `Emulator` interface, so the UI does not know where Rust executes. Declared dependency graphs are checked by `bun run boundaries`.

## Development environment

Rust, bun, and `wasm-bindgen-cli` are required for web builds. Desktop additionally requires Tauri system dependencies.

```sh
rustup toolchain install
bun install
bun run doctor
```

Versions have one source: `rust-toolchain.toml`, `package.json`, `Cargo.lock`, and `tools/system-deps/debian.txt`. On Debian/Ubuntu, desktop packages can be installed with:

```sh
xargs -a tools/system-deps/debian.txt sudo apt-get install -y --no-install-recommends
```

Alternatively, open the repository in the provided devcontainer. It contains Rust, WASM, bun, and native build dependencies. Graphical Tauri windows, macOS/Windows builds, and browser E2E still require the host or additional tools.

## Run and build

```sh
# Console demo: run to HLT and print CPU state
cargo run -p specialist-cli -- demo

# Load a binary; addresses accept decimal or 0x-prefixed hexadecimal values
cargo run -p specialist-cli -- run program.bin 0x0000 0x0000 200000

# Run an external Monitor 2 ROM (exactly 0x1800 bytes, entry point 0xC000)
cargo run -p specialist-cli -- monitor2 monitor2.rom 200000

# Browser
bun run dev:web
bun run build:web

# Native Tauri window
bun run dev:native

# Native release build without installers
bun run build:native -- --no-bundle
```

WebAssembly is generated into `packages/core-wasm/pkg`; this directory is not committed. The reusable web packages are `@specialist/core-wasm` and `@specialist/app`; standalone npm publishing is not configured.

## GitHub Pages

The emulator is published at [specialist-pk.github.io/specialist-emulator](https://specialist-pk.github.io/specialist-emulator/) by `.github/workflows/deploy.yml`. A merge into `main` triggers a production build and deployment; manual dispatch is also available. Pull requests run the same build and browser checks with the Pages base path, but never enable Pages or deploy. Repository Pages settings use GitHub Actions as the source. See [ADR-0008](docs/adr/0008-github-pages-deployment.md).

## Editors and debugging

Workspace settings are provided for VS Code (`.vscode/`) and Zed (`.zed/`). Both use local Oxc/Oxfmt and rust-analyzer. Recommended VS Code extensions are also included in the devcontainer.

Shared tasks and scenarios cover Chromium, CLI, and native debugging; VS Code additionally supports Firefox. Web launch scenarios call `web: ensure dev server`: they build WASM, reuse or start Vite, wait for readiness, and leave it running for later sessions. Stop it with `web: stop dev server` or `bun run dev:web:stop`.

Before native development debugging, run `native: dev server`. For a native release scenario in Zed, run `native: build frontend` first; VS Code runs it through `preLaunchTask`. CLI development and release scenarios run the built-in demo through CodeLLDB with debug symbols enabled.

An emulator-internal DAP adapter remains a future architectural step.

## Checks

```sh
bun run test:rust       # Rust tests without Tauri
bun run test:native     # real Tauri IPC and generated binding freshness
bun run test:wasm       # demo execution in real WASM
bun run test:e2e        # Chromium: step, run to HLT, and load BIN through UI
bun run test            # TypeScript and tooling tests
bun run typecheck
bun run check:wasm      # WASM plus dependent web types
bun run boundaries      # declared Rust and TypeScript dependency graphs
bun run deny            # Rust advisories, licenses, bans, and sources
bun run lint
bun run format:check
bun run format:check:rust
```

After changing Tauri commands, regenerate `apps/native/src/bindings.ts` with `bun run update:bindings`; do not edit it manually.

E2E uses an installed browser and supports `CHROME_PATH`. Without a browser it skips locally; CI sets `E2E_REQUIRE_BROWSER=1`, making a missing browser an error.

## Roadmap and process

Plans live in the pinned [Roadmap issue](https://github.com/Specialist-PK/specialist-emulator/issues/1) and linked issues. Work follows issue → branch → pull request (`Closes #N`) → squash merge after the aggregate `ci` check. Details for people and agents are in [AGENTS.md](AGENTS.md). Significant decisions are recorded in [`docs/adr/`](docs/adr/README.md).

## License

MIT. See [LICENSE](LICENSE).
