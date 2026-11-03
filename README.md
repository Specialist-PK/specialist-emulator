# Specialist Emulator

[English version](README.en.md)

[![CI](https://github.com/Specialist-PK/specialist-emulator/actions/workflows/ci.yml/badge.svg)](https://github.com/Specialist-PK/specialist-emulator/actions/workflows/ci.yml)

Кроссплатформенный эмулятор советского домашнего компьютера «Специалист». Одно Rust-ядро используется в консольном приложении, браузере через WebAssembly и нативном приложении на Tauri 2.

Проект перенимает архитектуру и процесс разработки [another-useless-app](https://github.com/DrUNE/another-useless-app): независимые сборки, общий UI, тонкие платформенные адаптеры, единый контракт ядра, задокументированные решения и воспроизводимые проверки.

## Текущие возможности

- базовый набор из 256 опкодов Intel 8080, регистры, флаги и порты ввода-вывода;
- 64 КиБ памяти и пошаговое выполнение инструкций;
- монохромный экран 384 × 256, видеопамять с адреса `0x9000`;
- запуск, пауза, сброс и выполнение одной инструкции;
- отображение регистров, флагов и счётчика циклов;
- загрузка BIN-файла по указанному адресу;
- встроенная демонстрационная программа, заполняющая видеопамять;
- общий Solid-интерфейс для web и desktop;
- одинаковые сценарии эмуляции через WASM и Tauri IPC.

Это ранняя версия ядра, а ещё не полная модель конкретного «Специалиста». Память пока представлена плоским массивом RAM. Monitor ROM, клавиатура, магнитофон, звук, клоны компьютера, точные временные характеристики и DAP-отладчик ещё не реализованы. ROM и программы сторонних авторов не входят в репозиторий.

## Структура

```text
crates/
  core/              # i8080, память, машина и демонстрационная программа
  core-wasm/         # тонкая wasm-bindgen обёртка
  cli/               # консольный запуск, бинарник specialist
apps/
  web/               # браузерная точка входа: WASM + общий UI
  native/            # Tauri 2: IPC + общий UI
packages/
  core-api/          # единый TypeScript-контракт Emulator
  core-wasm/         # сборка WASM и адаптер контракта
  app/               # общий Solid-интерфейс
  config/            # общие настройки TypeScript и Tailwind
tools/
  ci/                # покрытый тестами выбор CI jobs
  doctor/            # окружение и согласованность репозитория
  boundaries/        # границы объявленных зависимостей
  dev-server/        # фоновый Vite для отладки из редакторов
  system-deps/       # единый список пакетов Debian/Ubuntu
```

Направление зависимостей:

```text
CLI и Tauri → specialist-core
Web         → specialist-core-wasm → specialist-core
Web и Tauri → общий UI → core-api
```

Ядро не использует файловую систему, DOM, Tauri или другие платформенные API. Браузер и нативное приложение реализуют один интерфейс `Emulator`, поэтому общий UI не знает, где выполняется Rust-код. Графы объявленных зависимостей проверяются командой `bun run boundaries`.

## Окружение разработки

Для web-сборки нужны Rust, bun и `wasm-bindgen-cli`. Desktop дополнительно требует системные зависимости Tauri.

```sh
rustup toolchain install
bun install
bun run doctor
```

У версий один источник: `rust-toolchain.toml`, `package.json`, `Cargo.lock` и `tools/system-deps/debian.txt`. На Debian/Ubuntu пакеты для desktop устанавливаются командой:

```sh
xargs -a tools/system-deps/debian.txt sudo apt-get install -y --no-install-recommends
```

Либо откройте репозиторий в прилагаемом devcontainer. Он содержит Rust, WASM, bun и нативные зависимости сборки. Графическое окно Tauri, сборки macOS/Windows и браузерный E2E требуют хоста или дополнительных инструментов.

## Запуск и сборка

```sh
# Консольная демонстрация: выполнить до HLT и показать состояние CPU
cargo run -p specialist-cli -- demo

# Загрузить собственный бинарник; адреса принимают decimal или 0x-prefixed hex
cargo run -p specialist-cli -- run program.bin 0x0000 0x0000 200000

# Браузерная версия
bun run dev:web
bun run build:web

# Нативное окно Tauri
bun run dev:native

# Релизная сборка нативного приложения без установочных пакетов
bun run build:native -- --no-bundle
```

WebAssembly генерируется в `packages/core-wasm/pkg`; этот каталог не коммитится. Для повторного использования ядра предназначены `@specialist/core-wasm` и `@specialist/app`; публикация самостоятельного npm-пакета не настроена.

## GitHub Pages

Эмулятор публикуется по адресу [specialist-pk.github.io/specialist-emulator](https://specialist-pk.github.io/specialist-emulator/) через `.github/workflows/deploy.yml`. Merge в `main` запускает production-сборку и deployment; доступен и ручной запуск. Pull requests выполняют те же проверки сборки и браузера с base Pages, но не включают Pages и ничего не публикуют. В настройках репозитория источник Pages — GitHub Actions. См. [ADR-0008](docs/adr/0008-github-pages-deployment.md).

## Редакторы и отладка

Настройки workspace предоставлены для VS Code (`.vscode/`) и Zed (`.zed/`). Оба используют локальные Oxc/Oxfmt и rust-analyzer. Рекомендуемые расширения VS Code также включены в devcontainer.

Общие задачи и сценарии охватывают Chromium, CLI и native; VS Code дополнительно поддерживает Firefox. Web launch вызывает `web: ensure dev server`: собирает WASM, переиспользует или запускает Vite, ждёт готовности и оставляет сервер для следующих сессий. Остановка — `web: stop dev server` или `bun run dev:web:stop`.

Перед native development запустите `native: dev server`. Для native release в Zed сначала выполните `native: build frontend`; VS Code делает это через `preLaunchTask`. Сценарии CLI development/release запускают встроенную демонстрацию через CodeLLDB с отладочными символами.

Собственный DAP-адаптер для программы внутри эмулятора остаётся следующим архитектурным этапом.

## Проверки

```sh
bun run test:rust       # Rust-тесты без Tauri
bun run test:native     # реальный Tauri IPC и актуальность TS-привязок
bun run test:wasm       # выполнение демонстрации в настоящем WASM
bun run test:e2e        # Chromium: шаг, запуск до HLT и загрузка BIN через UI
bun run test            # TypeScript-тесты и тесты инструментов
bun run typecheck
bun run check:wasm      # WASM и типы зависимого web-кода
bun run boundaries      # графы объявленных зависимостей Rust и TypeScript
bun run deny            # уязвимости, лицензии, запреты и источники Rust
bun run lint
bun run format:check
bun run format:check:rust
```

После изменения команд Tauri обновите `apps/native/src/bindings.ts` через `bun run update:bindings`; вручную файл не редактируется.

E2E использует установленный браузер и поддерживает `CHROME_PATH`. Без браузера локально пропускается; CI задаёт `E2E_REQUIRE_BROWSER=1`, превращая отсутствие браузера в ошибку.

## Roadmap и процесс

Планы находятся в закреплённом [Roadmap issue](https://github.com/Specialist-PK/specialist-emulator/issues/1) и связанных задачах. Работа идёт по схеме issue → ветка → pull request (`Closes #N`) → squash merge после aggregate `ci`. Правила для людей и агентов описаны в [AGENTS.md](AGENTS.md). Значимые решения записываются в [`docs/adr/`](docs/adr/README.md).

## Лицензия

MIT. См. [LICENSE](LICENSE).
