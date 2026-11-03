# 0003. Reproducible environment and editor dev-server

[Русская версия](0003-reproducible-development-environment.ru.md)

Status: accepted (2026-11-03)

## Context

The project needs Rust, bun, a matching `wasm-bindgen-cli`, and native libraries. Duplicating versions and packages across CI, documentation, and editor configuration leads to drift. Browser debugging also needs a running Vite server; manual startup made the ported editor configuration behave differently from upstream.

## Decision

Use one source for each environment input: `rust-toolchain.toml` for Rust, `package.json` for bun, `Cargo.lock` for wasm-bindgen, `tools/system-deps/debian.txt` for Debian packages, and matching workflow pins for cargo-deny.

`tools/doctor` checks the environment without installing anything and runs repository consistency tests. The optional `.devcontainer/` uses the same package list and toolchain files, with named volumes for artifacts.

`tools/dev-server` provides `dev:web:start` and `dev:web:stop`. It reuses a healthy server, builds WASM before starting Vite, waits for readiness, stores PID and logs under `target/`, and leaves the server running for subsequent debugger sessions. Zed and VS Code launch scenarios call the same task; attach scenarios do not manage the server.

## Alternatives

- **Duplicate setup instructions.** Simple initially, but versions and packages diverge.
- **Devcontainer only.** Reproducible Linux environment, but no native window or macOS/Windows host builds.
- **Manual Vite startup.** Less tooling, but unreliable and inconsistent editor launches.
- **Start a new server for every debug session.** Isolation is unnecessary here and causes port conflicts and repeated WASM builds.

## Consequences

- (+) CI, local setup, container, and editors use consistent inputs.
- (+) Browser debugging works with one launch action and reuses existing work.
- (−) The helper and consistency tests need maintenance as the workspace evolves.
- (−) The container is large and does not replace host-specific verification.
