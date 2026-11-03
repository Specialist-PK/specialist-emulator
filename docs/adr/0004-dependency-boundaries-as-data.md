# 0004. Declared dependency boundaries as data

[Русская версия](0004-dependency-boundaries-as-data.ru.md)

Status: accepted (2026-11-03)

## Context

Import linting does not see dependencies declared but unused in source, and a WASM build catches only part of the platform graph. The core, UI, CLI, web, and native applications need enforceable boundaries based on real manifests.

## Decision

Use `tools/boundaries` through `bun run boundaries` with rules in `tools/boundaries/rules.json`.

Rust graphs come from `cargo metadata --locked --filter-platform`; TypeScript graphs come from workspace manifests. `directOnly` defines an exhaustive allowlist for direct dependencies, while `forbidTransitive` rejects packages anywhere in the reachable graph and reports a dependency path. Missing rule subjects are errors so stale rules cannot silently pass.

Rules describe the current architecture: `specialist-core` has no platform dependencies, the WASM crate is a thin boundary, `core-api` is dependency-free, shared UI knows only the contract, and applications do not depend on each other. New rules must be checked by intentionally violating a real manifest, then reverting the violation.

## Alternatives

- **Import lint only.** Does not cover manifest-only or transitive dependencies.
- **cargo-deny for architecture.** Useful for supply-chain policy, but awkward for exact per-package allowlists.
- **Builds only.** Do not detect every forbidden edge, especially on a different target platform.

## Consequences

- (+) Violations fail with a concrete dependency chain and reason.
- (+) The rules are readable as an architectural inventory and covered by tests.
- (−) Rules must be updated with every relevant manifest or architecture change.
- (−) External TypeScript dependency graphs are treated as leaves; import checks remain a separate layer.
