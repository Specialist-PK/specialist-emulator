# Specialist Emulator

Кроссплатформенный эмулятор советского домашнего компьютера «Специалист». Одно Rust-ядро используется в консольном приложении, браузере через WebAssembly и нативном приложении на Tauri 2.

Проект основан на архитектурных решениях прототипа [another-useless-app](https://github.com/DrUNE/another-useless-app): независимые сборки, общий UI, тонкие платформенные адаптеры и единый контракт ядра.

## Текущие возможности

- базовый набор из 256 опкодов Intel 8080, регистры, флаги и порты ввода-вывода;
- 64 КиБ памяти и пошаговое выполнение инструкций;
- монохромный экран 384 × 256, видеопамять с адреса `0x9000`;
- запуск, пауза, сброс и выполнение одной инструкции;
- отображение регистров, флагов и счётчика циклов;
- загрузка BIN-файла по указанному адресу;
- встроенная демонстрационная программа, написанная на i8080 и заполняющая видеопамять;
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
  core-api/          # единый TypeScript-контракт эмулятора
  core-wasm/         # сборка WASM и адаптер контракта
  app/               # общий Solid-интерфейс
  config/            # общие настройки TypeScript и Tailwind
```

Направление зависимостей:

```text
CLI и Tauri → specialist-core
Web         → specialist-core-wasm → specialist-core
Web и Tauri → общий UI → core-api
```

Ядро не использует файловую систему, DOM, Tauri или другие платформенные API. Браузер и нативное приложение реализуют один интерфейс `Emulator`, поэтому общий UI не знает, где выполняется Rust-код.

## Документация

- [Архитектура](docs/architecture.md) — текущие компоненты, границы ядра, целевая модель машины и отладочная схема.
- [Дорожная карта](docs/roadmap.md) — этапы развития и критерии готовности.
- [Specialist Zed](https://github.com/Specialist-PK/specialist-zed) — отдельный репозиторий расширения для DAP-отладки в Zed.

## Подготовка окружения

Нужны Rust, bun и `wasm-bindgen-cli` для web-сборки. Desktop дополнительно требует системные зависимости Tauri.

```bash
rustup toolchain install
bun install
cargo install wasm-bindgen-cli --locked --version 0.2.130
```

Версии Rust и bun закреплены в `rust-toolchain.toml` и `package.json`.

## Запуск и сборка

```bash
# Консольная демонстрация: выполнить до HLT и показать состояние CPU
cargo run -p specialist-cli -- demo

# Загрузить собственный бинарник; адреса можно писать в hex
cargo run -p specialist-cli -- run program.bin 0x0000 0x0000 200000

# Браузерная версия
bun run dev:web
bun run build:web

# Нативное окно Tauri
bun run dev:native

# Релизная сборка нативного приложения без создания установочных пакетов
bun run build:native -- --no-bundle
```

WebAssembly генерируется в `packages/core-wasm/pkg`; этот каталог не коммитится. Для повторного использования ядра в другой web-странице предназначены пакеты `@specialist/core-wasm` и `@specialist/app`; публикация самостоятельного npm-пакета пока не настроена.

## Проверки

```bash
bun run test:rust       # тесты ядра, CLI и WASM-крейта без Tauri
bun run test:native     # реальный Tauri IPC и актуальность TS-привязок
bun run test:wasm       # выполнение демонстрации в настоящем WASM
bun run test:e2e        # Chromium: шаг, запуск до HLT и загрузка BIN через UI
bun run test            # TypeScript-тесты
bun run typecheck
bun run check:wasm      # WASM + типы зависимого web-кода
bun run lint
bun run format:check
bun run format:check:rust
```

После изменения команд Tauri TypeScript-привязки обновляются командой `bun run update:bindings` и коммитятся. Файл `apps/native/src/bindings.ts` вручную не редактируется.

## Отладка в Zed

В `.zed/debug.json` есть сценарий `CLI: demo` для отладки нативного ядра через CodeLLDB. Он собирает консольное приложение и запускает встроенную демонстрацию. Отладка программы внутри эмулятора через собственный DAP-адаптер — следующий архитектурный этап.

## Лицензия

MIT. См. [LICENSE](LICENSE).
