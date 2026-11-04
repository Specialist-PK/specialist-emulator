# AGENTS.md

[English version](AGENTS.en.md)

Инструкции для ИИ-агентов, работающих с репозиторием. Обзор проекта находится в `README.md`.

## Контекст

Проект наследует учебный процесс `another-useless-app`: понимание решений не менее важно, чем сам эмулятор.

- Работайте маленькими шагами: одна идея за раз, с проверкой сборкой и тестами.
- Объясняйте выбранное решение, его причину и альтернативу.
- Перед изменением архитектуры читайте `docs/adr/`. Значимые решения записывайте в новый ADR по формату `docs/adr/0001-*`; принятые ADR не переписывайте, а заменяйте новыми.
- Опыт пользователя: TypeScript и React — высокий; Rust, Solid, Tailwind — начальный. Применяйте лучшие практики и объясняйте нетривиальные отличия через аналогии с React/TypeScript.
- Компоненты Solid выполняются один раз, сигналы являются функциями, деструктуризация props ломает реактивность, а `Show`/`For` заменяют условный рендеринг и `map` в стиле React. В Rust используется `Result` вместо исключений; trait не тождествен TypeScript-интерфейсу.
- Общение с пользователем — на русском; код, идентификаторы и коммиты — на английском.

## Структура

- `crates/core` — платформенно-независимое ядро i8080.
- `crates/core-wasm` — только тонкая граница `wasm-bindgen`.
- `crates/cli` — консольное приложение (бинарник `specialist`).
- `packages/core-api` — независимый TypeScript-контракт `Emulator`.
- `packages/core-wasm` — сборка WASM и TypeScript-реализация контракта.
- `packages/app` — общий Solid UI; ядро поступает через `CoreContext`.
- `apps/web` и `apps/native` — независимые точки входа, подключающие конкретные ядра.
- `tools/doctor` — диагностика окружения без изменений системы и проверки согласованности.
- `tools/boundaries` — проверка объявленных зависимостей по правилам `rules.json`.
- `tools/ci` — покрытый тестами выбор CI jobs по изменённым файлам.
- `tools/dev-server` — фоновый запуск Vite, ожидание готовности и остановка для отладки из редакторов.
- `tools/system-deps` — единый список системных пакетов Debian/Ubuntu.
- `tests/e2e` — web E2E в настоящем Chromium.

## Жёсткие архитектурные правила

1. `crates/core` не должен зависеть от DOM, Tauri, файловой системы или зависимостей, несовместимых с `wasm32-unknown-unknown`.
2. `crates/core-wasm` не содержит отдельной реализации эмулятора.
3. `packages/core-api` не зависит от UI и платформ.
4. `packages/app` знает только `core-api` и получает ядро через `CoreContext`.
5. Приложения не зависят друг от друга. Только точки входа подключают конкретные ядра.
6. CLI не требует bun/Vite; web не требует Tauri; desktop использует нативное ядро через IPC.
7. Новая возможность ядра требует метода `Emulator`, обеих реализаций (WASM и IPC) и проверок реального выполнения.
8. `packages/core-wasm/pkg` создаётся через `bun run build:wasm` и не коммитится. Версии крейта `wasm-bindgen` и CLI должны совпадать.
9. `apps/native/src/bindings.ts` генерируется через `bun run update:bindings`; вручную его не редактировать. Новые команды Tauri также добавляются в `build.rs` и обе capabilities.
10. Границы зависимостей хранятся данными в `tools/boundaries/rules.json`. Обновляйте их вместе с архитектурой и проверяйте новые правила намеренным нарушением в настоящем манифесте.
11. После каждого изменения `Cargo.toml`/`Cargo.lock` запускайте `bun run deny`. Исключения требуют причины; OpenSSL/native-tls и пакеты вне crates.io запрещены.
12. Не добавляйте ROM и сторонние программы без подтверждённого права на распространение.

## Команды

```sh
bun install
bun run doctor
bun run test:rust
bun run test:native
bun run update:bindings
bun run typecheck
bun run lint
bun run boundaries
bun run deny
bun run format
bun run format:check
bun run format:check:rust
bun run test
bun run check:wasm
bun run test:wasm
bun run test:e2e
bun run dev:web
bun run dev:web:start
bun run dev:web:stop
bun run build:web
bun run dev:native
bun run build:native -- --no-bundle
cargo run -p specialist-cli -- demo
cargo run -p specialist-cli -- run program.bin 0x0000 0x0000 200000
cargo run -p specialist-cli -- monitor2 monitor2.rom 200000
cargo run -p specialist-cli -- monitor2 monitor2.rom 200000 --json
```

Перед завершением запускайте `bun run format:check`, `bun run format:check:rust`, `bun run lint`, `bun run test:rust`, `bun run typecheck` и `bun run test`. При изменении ядра или WASM также `bun run check:wasm` и `bun run test:wasm`; при изменении web — `bun run build:web` и `bun run test:e2e`; при изменении Tauri — `bun run test:native` и `bun run build:native -- --no-bundle`. Всё, что не удалось проверить, указывайте явно.

`bun run dev:native` открывает настоящее окно: без просьбы пользователя его не запускать. E2E использует системный Chromium, поддерживает `CHROME_PATH`, локально пропускается без браузера и падает при `E2E_REQUIRE_BROWSER=1`.

## Окружение

`bun run doctor` сообщает о недостающих инструментах и ничего не устанавливает. У версий один источник:

- Rust — `rust-toolchain.toml`;
- bun — `package.json` (`packageManager`);
- `wasm-bindgen-cli` — `Cargo.lock`;
- системные пакеты Debian — `tools/system-deps/debian.txt`;
- cargo-deny — совпадающие закреплённые версии в обоих workflow.

Не дублируйте эти версии в CI и скриптах. Node.js не требуется. Необязательный devcontainer в `.devcontainer/` использует те же источники и именованные тома для артефактов сборки.

## Стек и версии

- bun workspaces; версии зависимостей закреплены в catalog корневого `package.json`. Добавляйте пакеты через `"catalog:"`.
- Solid 2 RC и нативный `tsc` TypeScript 7; для отличающихся от Solid 1 и React API обращайтесь к локальным исходникам.
- Tailwind 4 в CSS-first режиме настраивается в `packages/config/theme.css`.
- Rust edition 2024, resolver 3. Общие зависимости крейтов объявляются в `[workspace.dependencies]`.

## Стиль

- Rust: `cargo fmt`, ошибки библиотек через `thiserror`.
- TS/TSX/JSON/CSS: `oxfmt`; type-aware `oxlint`.
- Генерируемые файлы вручную не редактируются, не форматируются и не линтуются.
- TypeScript работает в strict-режиме с `verbatimModuleSyntax` и `noUncheckedIndexedAccess`.
- Комментарии объясняют «почему», а не «что». Обычные `//` комментарии — на русском. Rustdoc и TSDoc — на английском для генерируемых объявлений и подсказок IDE.
- Документация двуязычная: русская каноническая (`.md`), английская рядом (`.en.md`), с перекрёстными ссылками. Каждый документ имеет обе версии. Специальные имена GitHub могут содержать оба языка в одном файле.
- Код и документация проекта остаются под MIT; изменение лицензии документации — отдельное решение.

## Работа в редакторах

Общие сценарии и задачи VS Code и Zed должны оставаться синхронизированными. Web launch вызывает `web: ensure dev server`; attach не запускает и не останавливает сервер. Помощник переиспользует существующий сервер, перед новым запуском собирает WASM, ждёт готовности и оставляет сервер для следующих сессий. Остановка — `web: stop dev server`.

## Deployment

`.github/workflows/deploy.yml` публикует web-эмулятор на GitHub Pages после push в `main` и при ручном запуске. Pull requests выполняют production-сборку с base Pages и браузерный E2E, но не должны включать Pages или выполнять deploy. Permissions остаются на уровне jobs, очередь `pages` не отменяет текущий deployment, а Vite `base` вычисляется через `actions/configure-pages`, без захардкоженного имени репозитория. См. ADR-0008.

## GitHub workflow

Работа идёт через issues и pull requests. GitHub Issues — источник планов; закреплённый Roadmap issue даёт общий обзор.

1. **Issue.** Каждая задача начинается с issue (шаблон `Task` или `Bug`). При необходимости создайте его и свяжите с Roadmap.
2. **Ветка.** Одна ветка на issue от свежего `main`: `<type>/<summary>`. Прямые коммиты в `main` запрещены.
3. **Коммиты.** Небольшие Conventional Commits на английском (`feat:`, `fix:`, `docs:`, `chore:`, `ci:`), одна идея в каждом.
4. **Pull request.** Заполните шаблон и укажите `Closes #N`. Заголовок — в стиле коммитов на английском, описание — на русском. Пропущенные проверки честно перечисляйте в «Не проверено».
5. **Merge.** Только squash после зелёного aggregate `ci`. Пропущенные по путям jobs считаются успехом. Ветка удаляется автоматически. Добавление ОС или job не требует изменения защиты ветки. При добавлении каталогов или связей между областями обновляйте `tools/ci/src/select.ts` и его тесты.

Коммиты, пуши, создание/закрытие issues и PR и merge требуют подтверждения пользователя. Разрешение на одно названное действие не разрешает merge или другие действия.

Не добавляйте в репозиторий `node_modules`, `target`, `dist` и сгенерированный WASM-пакет.
