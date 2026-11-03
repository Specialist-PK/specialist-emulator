# 0006. Rust dependency supply-chain checks

[Русская версия](0006-dependency-supply-chain-checks.ru.md)

Status: accepted (2026-11-03)

## Context

Tauri brings a large transitive Rust graph that cannot be reviewed manually. Vulnerability records appear without code changes, while license, source, and banned-crate policies need repeatable enforcement.

## Decision

Use `cargo-deny` through `bun run deny` and `deny.toml` for advisories, licenses, bans, and sources.

Yanked releases are denied. Every advisory exception has a reason. The license allowlist permits the project's permissive dependencies and MPL-2.0; private workspace crates also declare MIT. OpenSSL, `openssl-sys`, and `native-tls` are forbidden in favor of rustls. Registry and git sources outside crates.io are forbidden. Duplicate crate versions produce warnings rather than failures.

CI runs the full check when Cargo manifests, the lockfile, `deny.toml`, or workflows change. A separate weekly `audit.yml` checks advisories and supports manual dispatch. Both workflows use the same pinned cargo-deny version, enforced by a consistency test.

## Alternatives

- **cargo-audit only.** Covers advisories but not licenses, banned crates, and sources.
- **Dependabot or Renovate.** Helps updates but does not define project policy.
- **Hide all unmaintained warnings.** Less noise, but loses potentially important dependency information.

## Consequences

- (+) Dependency policy is executable locally and in CI, including newly published advisories.
- (+) Exceptions are explicit and reviewed alongside code.
- (−) Advisory checks require network access and scheduled workflows can be disabled after repository inactivity.
- (−) JavaScript dependencies and dependency update automation are outside this decision.
