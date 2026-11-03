# AGENTS.md

Инструкции для агентов, работающих с проектом. Общение с пользователем — на русском; код, идентификаторы и коммиты — на английском.

## Архитектура

- `crates/core` — платформенно-независимое ядро i8080. Не добавлять в него DOM, Tauri, файловую систему или зависимости, несовместимые с `wasm32-unknown-unknown`.
- `crates/core-wasm` — только граница `wasm-bindgen`, без отдельной реализации логики эмулятора.
- `packages/core-api` — единый TypeScript-контракт `Emulator`; он не зависит от UI и платформ.
- `packages/app` — общий Solid UI. Он знает только `core-api` и получает ядро через `CoreContext`.
- `apps/web` и `apps/native` подключают реализации ядра в точках входа и не зависят друг от друга.
- CLI не требует bun/Vite; web не требует Tauri; desktop использует нативное ядро через IPC.

## Границы и генерация

- `packages/core-wasm/pkg` создаётся через `bun run build:wasm` и не коммитится. Версии `wasm-bindgen` и `wasm-bindgen-cli` должны совпадать.
- `apps/native/src/bindings.ts` генерируется через `bun run update:bindings`; вручную его не редактировать. Новые команды Tauri также добавляются в `build.rs` и обе capabilities.
- Новая возможность ядра требует обновления контракта `Emulator`, обеих реализаций (WASM и IPC) и проверок на реальном выполнении.
- Не добавлять ROM и сторонние программы без подтверждённого права на распространение.

## Проверки

После изменений запускать профильные проверки из README: форматирование, lint, TypeScript и Rust-тесты. При изменении ядра или WASM также `bun run test:wasm` и `bun run check:wasm`; при изменении web — `bun run build:web` и `bun run test:e2e`; при изменении Tauri — `bun run test:native` и `bun run build:native -- --no-bundle`. E2E использует системный Chromium, путь можно переопределить через `CHROME_PATH`.

`bun run dev:native` открывает настоящее окно: без просьбы пользователя его не запускать. Коммиты, пуши, issues и pull requests выполнять только с явного разрешения пользователя.

Не коммитить `target`, `node_modules`, `dist` и сгенерированный WASM-пакет.
