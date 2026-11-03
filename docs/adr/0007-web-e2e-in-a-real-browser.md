# 0007. Web E2E in a real browser

[Русская версия](0007-web-e2e-in-a-real-browser.ru.md)

Status: accepted (2026-11-03)

## Context

Unit tests and WASM-boundary tests do not exercise the complete UI, browser module loading, Canvas, or file input. Browser availability differs between a developer host, CI, and the optional container.

## Decision

Keep the E2E scenario in `tests/e2e` using Vitest and `playwright-core`. It builds the web application, serves production output, uses an installed Chrome/Chromium, and checks stepping, running to HLT, and BIN loading through the real UI without JavaScript errors.

`CHROME_PATH` overrides browser discovery, with common Linux, macOS, and Windows locations supported. If no browser is found, the suite skips locally with a warning. `E2E_REQUIRE_BROWSER=1` turns absence into an error and is always set in CI. No browser is downloaded by the test.

## Alternatives

- **UI unit tests only.** Faster, but do not cover the real WASM/Vite/browser chain.
- **Download a pinned Playwright browser.** More reproducible, but adds a large artifact and another installation step.
- **Always fail without a browser.** Appropriate for CI, unnecessarily restrictive for core-only local work.

## Consequences

- (+) The complete user scenario is verified in the same kind of browser used for development.
- (+) Local optional behavior and mandatory CI behavior are explicit.
- (−) Firefox and Safari are not covered.
- (−) Runner browser updates can expose new failures independently of application changes.
