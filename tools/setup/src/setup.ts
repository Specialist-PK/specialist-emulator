import { lockedVersion } from "@specialist/doctor/parse";

const USAGE = `Использование:
  bun run setup            Установить JS-зависимости и проверить CLI/web-окружение
  bun run setup --install  Дополнительно установить Rust toolchain и wasm-bindgen-cli

Системные пакеты и sudo-команды не выполняются.`;

const NATIVE_HINT = `Для desktop дополнительно установите системные зависимости:
  Debian/Ubuntu: xargs -a tools/system-deps/debian.txt sudo apt-get install -y --no-install-recommends
  Другие ОС: https://v2.tauri.app/start/prerequisites/
Затем проверьте их командой bun run doctor native.`;

export interface SetupIO {
  read(path: string): string | undefined;
  run(command: string, args: string[]): number;
  log(message: string): void;
}

function step(io: SetupIO, command: string, args: string[]): boolean {
  io.log(`\n$ ${[command, ...args].join(" ")}`);
  const exitCode = io.run(command, args);
  if (exitCode !== 0) io.log(`Команда завершилась с кодом ${exitCode}.`);
  return exitCode === 0;
}

export function runSetup(args: string[], io: SetupIO): number {
  if (args.includes("--help") || args.includes("-h")) {
    io.log(USAGE);
    return 0;
  }

  const unknown = args.filter((arg) => arg !== "--install");
  if (unknown.length > 0) {
    io.log(`Неизвестные аргументы: ${unknown.join(", ")}\n\n${USAGE}`);
    return 1;
  }

  if (args.includes("--install")) {
    if (!step(io, "rustup", ["toolchain", "install"])) return 1;

    const version = lockedVersion(io.read("Cargo.lock") ?? "", "wasm-bindgen");
    if (version === undefined) {
      io.log("Не удалось определить версию wasm-bindgen в Cargo.lock.");
      return 1;
    }
    if (!step(io, "cargo", ["install", "wasm-bindgen-cli", "--locked", "--version", version]))
      return 1;
  }

  if (!step(io, "bun", ["install", "--frozen-lockfile"])) return 1;
  if (!step(io, "bun", ["tools/doctor/src/index.ts", "core", "wasm"])) return 1;

  io.log("\nCLI/web-окружение настроено и проверено.");
  io.log(NATIVE_HINT);
  return 0;
}
