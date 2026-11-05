# 0003. Reproducible environment and editor dev-server

[Русская версия](0003-reproducible-development-environment.md)

Status: accepted (2026-11-03)

## Context

The project needs Rust, bun, a matching `wasm-bindgen-cli`, and native libraries. Duplicating versions and packages across CI, documentation, and editor configuration leads to drift. Browser debugging also needs a running Vite server; manual startup made the ported editor configuration behave differently from upstream.

## Decision

Use one source for each environment input: `rust-toolchain.toml` for Rust, `package.json` for bun, `Cargo.lock` for wasm-bindgen, `tools/system-deps/debian.txt` for Debian packages, and matching workflow pins for cargo-deny.

`tools/doctor` checks the environment without installing anything and runs repository consistency tests. `tools/setup` provides the primary local setup path: `bun run setup` installs JavaScript dependencies from the lockfile and checks the CLI/web environment, while `bun run setup --install` also installs the Rust toolchain and `wasm-bindgen-cli`. The helper never runs `sudo`; desktop dependencies remain a separate explicit step. The optional `.devcontainer/` remains an isolated alternative and uses the same package list and toolchain files, with named volumes for artifacts.

### Why keep both paths

Setup and the devcontainer serve different purposes rather than duplicating each other:

- **Setup is better for day-to-day host development.** It does not require Docker, uses the developer's usual editor and system tools, supports a native Tauri window and host-platform builds, and quickly repeats environment checks after the initial setup.
- **The devcontainer is better for a clean, reproducible start.** It includes system libraries and the pinned toolchain, reduces differences between machines, and isolates the project environment from the rest of the system. This is useful for onboarding, working on an unprepared machine, and investigating environment-specific problems.
- **Using both does not create two independent configurations.** Both paths read the same version files and system package list, and `doctor` checks the result in the same way. Setup lowers the cost of daily work, while the devcontainer limits environment drift; neither one replaces verification on both a real host and a clean environment.

`tools/dev-server` provides `dev:web:start` and `dev:web:stop`. It reuses a healthy server, builds WASM before starting Vite, waits for readiness, stores PID and logs under `target/`, and leaves the server running for subsequent debugger sessions. Zed and VS Code launch scenarios call the same task; attach scenarios do not manage the server.

## Alternatives

- **Duplicate setup instructions.** Simple initially, but versions and packages diverge.
- **Devcontainer only.** Reproducible Linux environment, but no native window or macOS/Windows host builds.
- **Replace the devcontainer with the setup helper.** Local setup is more convenient, but it does not provide an isolated Linux environment or install system dependencies automatically.
- **Manual Vite startup.** Less tooling, but unreliable and inconsistent editor launches.
- **Start a new server for every debug session.** Isolation is unnecessary here and causes port conflicts and repeated WASM builds.

## Consequences

- (+) CI, local setup, container, and editors use consistent inputs.
- (+) Browser debugging works with one launch action and reuses existing work.
- (−) The helpers and consistency tests need maintenance as the workspace evolves.
- (−) The container is large and does not replace host-specific verification.
