[Русская версия](PULL_REQUEST_TEMPLATE.md)

## What and why

<!-- Briefly: what changes and why. Which decisions were chosen and which alternatives were rejected. -->

Closes #

## Verification

- [ ] `bun run test:rust`
- [ ] `bun run typecheck`
- [ ] The affected target (cli / web / native) builds independently
- [ ] The core (`specialist-core`) gained no platform dependencies

## Architecture

- [ ] Significant decisions are recorded in `docs/adr/` (or there are none)
- [ ] README and AGENTS.md are updated if commands or structure changed

## Not verified

<!-- Honestly list anything that could not be checked (platform, target, tool). -->
