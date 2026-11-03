# 0005. CI OS matrix, aggregate gate, and path selection

[Русская версия](0005-ci-os-matrix-gate-and-path-selection.ru.md)

Status: accepted (2026-11-03)

## Context

Linux-only local checks missed two real failures: case-insensitive resolution of `Screen.tsx`/`screen.ts` on macOS/Windows and CRLF/LF mismatches in generated Tauri bindings on Windows. Running every job for documentation-only changes would also waste the expensive native matrix.

## Decision

CI runs `rust`, `ts`, and `native` on Ubuntu, macOS, and Windows with `fail-fast: false`. WASM, web build, dependency boundaries, and browser E2E run on Ubuntu where the required tools and Chrome are available. `deny` checks dependency policy.

`tools/ci` selects jobs from changed files with tested rules. Pull requests compare `HEAD^1` with the merge commit; other events run everything. Rules list known-irrelevant files, so an unknown path triggers relevant checks rather than skipping them. `deny` uses a positive list of dependency and workflow files.

One aggregate `ci` job depends on every other job with `if: always()`. Skipped jobs count as success; failed or cancelled jobs fail the gate. Branch protection requires only `ci`.

`.gitattributes` enforces LF in every working copy. The framebuffer helper is named `framebuffer.ts`, avoiding the case-only collision with `Screen.tsx`.

## Alternatives

- **Always run everything.** Simpler, but expensive for irrelevant changes.
- **Workflow `paths:` filters or an external action.** Less local code, but harder to test and easy to combine incorrectly with required checks.
- **Linux-only CI.** Cheaper, but misses filesystem, shell, resource, and line-ending differences.
- **Case-insensitive imports without renaming.** Does not remove ambiguity for TypeScript and bundlers.

## Consequences

- (+) The original failures are addressed at their causes and covered by the OS matrix.
- (+) One required check supports skipped jobs and an evolving matrix.
- (−) Irrelevant-path lists require maintenance and tests.
- (−) Native Windows remains the slowest job; unknown files intentionally cause extra work.
