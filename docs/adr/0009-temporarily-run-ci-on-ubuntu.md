# 0009. Temporarily run CI only on Ubuntu

[Русская версия](0009-temporarily-run-ci-on-ubuntu.ru.md)

Status: accepted (2026-11-03)

## Context

ADR-0005 introduced a Linux/macOS/Windows matrix. While setting up Pages, macOS jobs repeatedly spent a long time queued and two were abandoned. This blocked the deployment work without providing timely feedback. The user requested a temporary reduction to the checks needed for emulator development and Pages publishing.

## Decision

Temporarily remove the operating-system matrix from the `rust`, `ts`, and `native` jobs and run them only on `ubuntu-latest`. Keep Rust tests, TypeScript/tooling checks, the Ubuntu Tauri build, WASM and browser E2E, dependency checks, the aggregate `ci` gate, and the Pages build/deploy workflow.

The aggregate gate continues to accept only `success` and `skipped`. Path-based job selection remains unchanged, and branch protection still requires only `ci`.

This is a temporary operational decision, not a claim that macOS and Windows support is unnecessary. Restore the matrix when deployment stabilizes or cross-platform work resumes; record that restoration as a new decision.

## Alternatives

- **Keep the full matrix.** Preserves coverage but leaves the current development workflow blocked by runner queues.
- **Disable native builds entirely.** Faster, but removes useful Ubuntu Tauri IPC and build verification needed for desktop development.
- **Run non-Linux jobs only manually.** Reduces routine load but adds a separate trigger and more configuration than the temporary matrix removal.
- **Accept abandoned jobs.** Would make the aggregate gate unreliable and allow unverified merges.

## Consequences

- (+) CI provides faster feedback for core, web, desktop Linux, and Pages development.
- (+) The required branch-protection configuration does not change.
- (−) macOS and Windows regressions are not detected automatically while the pause is active.
- (−) Restoring cross-platform verification requires an explicit later change rather than happening automatically.
