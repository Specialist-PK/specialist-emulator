import {
  type Emulator,
  type EmulatorError,
  EmulatorException,
  type EmulatorSnapshot,
} from "@specialist/core-api";

import { type CommandError, commands, type EmulatorSnapshot as IpcSnapshot } from "./bindings";

const toEmulatorError = (error: CommandError): EmulatorError => error;

function unwrap<T>(
  result: { status: "ok"; data: T } | { status: "error"; error: CommandError },
): T {
  if (result.status === "error") throw new EmulatorException(toEmulatorError(result.error));
  return result.data;
}

function normalize(snapshot: IpcSnapshot): EmulatorSnapshot {
  return { ...snapshot, cycles: Number(snapshot.cycles) };
}

export function createIpcCore(): Emulator {
  return {
    async reset() {
      return normalize(await commands.reset());
    },
    async loadDemo() {
      return normalize(await commands.loadDemo());
    },
    async loadMonitor2Rom(rom) {
      return normalize(unwrap(await commands.loadMonitor2Rom(Array.from(rom))));
    },
    async loadProgram(program, address, entryPoint = address) {
      return normalize(
        unwrap(await commands.loadProgram(address, entryPoint, Array.from(program))),
      );
    },
    async keyEvent(event) {
      return normalize(
        unwrap(await commands.keyEvent(event.key, event.pressed, event.cycle ?? null)),
      );
    },
    async run(instructions) {
      return normalize(unwrap(await commands.run(instructions)));
    },
    async snapshot() {
      return normalize(await commands.snapshot());
    },
    async framebuffer() {
      return new Uint8Array(await commands.framebuffer());
    },
    async readMemory(address, length) {
      return new Uint8Array(unwrap(await commands.readMemory(address, length)));
    },
    dispose() {},
  };
}
