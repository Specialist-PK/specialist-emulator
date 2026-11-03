# 0008. GitHub Pages deployment after merging into `main`

[Русская версия](0008-github-pages-deployment.md)

Status: accepted (2026-11-03)

## Context

The web emulator should become available on GitHub Pages after a pull request is merged. Before choosing the configuration, we examined `DrUNE/perfect-todo`, including its workflow history and failed runs.

That project showed several concrete failure modes:

- runs [41080322756](https://github.com/DrUNE/perfect-todo/actions/runs/41080322756) and [41084857434](https://github.com/DrUNE/perfect-todo/actions/runs/41084857434) failed in `configure-pages` with `Not Found` because Pages was not enabled with GitHub Actions as its source;
- older action versions targeted deprecated Node 20 and had to be upgraded;
- a follow-up commit replaced a duplicated Node version with `node-version-file`;
- the project needs a Vite `base` derived from the Pages URL, and deployments must not cancel each other midway.

Our application additionally requires Rust and a matching wasm-bindgen CLI. Publishing a PR must never deploy it, but the Pages-specific production path should be tested before merging.

## Decision

Use `.github/workflows/deploy.yml` with `pull_request`, `push` to `main`, and `workflow_dispatch` triggers.

The build job uses the repository version sources, runs lint, formatting, typecheck, TypeScript/tooling tests, WASM tests, a Pages-base production build, and mandatory browser E2E. `actions/configure-pages@v6` supplies `base_path`; Vite reads it through `BASE_PATH`. `enablement` is allowed only outside pull requests, so a push can provision a missing Pages site while an untrusted PR cannot change repository settings.

The deploy job runs only outside pull requests and publishes `apps/web/dist` through `actions/deploy-pages@v5`. Permissions are scoped per job: build receives Pages configuration access; deploy receives `pages: write` and `id-token: write`. The `pages` concurrency group queues runs with `cancel-in-progress: false`.

The application has one page and no router, so a SPA `404.html` fallback is unnecessary. Current Pages settings already use `build_type: workflow`; automatic enablement is retained for portability.

## Alternatives

- **Trigger only on `main`.** Simpler and used by `perfect-todo`, but does not validate the Pages base path before merging.
- **Top-level write permissions.** Fewer lines, but gives unrelated steps and PRs more authority than necessary.
- **Cancel older deployments.** Reduces waiting but can leave publication in an inconsistent intermediate state.
- **Hard-code `/specialist-emulator/`.** Works for one project URL, but not for forks or a different Pages base path.
- **Copy the Node/npm workflow unchanged.** Inappropriate for this repository's bun and Rust/WASM toolchain.

## Consequences

- (+) Every merge into `main` publishes the tested emulator without a manual deployment step.
- (+) The failures found in `perfect-todo` have explicit safeguards and consistency tests.
- (+) Pull requests validate the real subpath but cannot deploy or enable Pages.
- (−) PRs run an additional web test suite alongside the main CI workflow.
- (−) Pages configuration and the first production deployment still depend on GitHub availability and repository settings.
