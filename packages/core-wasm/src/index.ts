import {
  type Emulator,
  type EmulatorSnapshot,
  EmulatorException,
  isEmulatorError,
} from "@specialist/core-api";

import init, { initSync, type SyncInitInput, WasmEmulator } from "../pkg/specialist_core_wasm.js";

export interface CreateCoreOptions {
  wasm?: SyncInitInput;
}

let ready: Promise<unknown> | undefined;

function loadWasm(options: CreateCoreOptions) {
  if (options.wasm !== undefined) return Promise.resolve(initSync({ module: options.wasm }));
  ready ??= init().catch((error: unknown) => {
    ready = undefined;
    throw error;
  });
  return ready;
}

function call<T>(operation: () => T): T {
  try {
    return operation();
  } catch (error) {
    throw isEmulatorError(error) ? new EmulatorException(error) : error;
  }
}

export async function createCore(options: CreateCoreOptions = {}): Promise<Emulator> {
  await loadWasm(options);
  const machine = new WasmEmulator();
  return {
    async reset() {
      return call(() => machine.reset() as EmulatorSnapshot);
    },
    async loadDemo() {
      return call(() => machine.loadDemo() as EmulatorSnapshot);
    },
    async loadMonitor2Rom(rom) {
      return call(() => machine.loadMonitor2Rom(rom) as EmulatorSnapshot);
    },
    async loadProgram(program, address, entryPoint = address) {
      return call(() => machine.loadProgram(address, entryPoint, program) as EmulatorSnapshot);
    },
    async keyEvent(event) {
      return call(
        () => machine.keyEvent(event.key, event.pressed, event.cycle) as EmulatorSnapshot,
      );
    },
    async run(instructions) {
      return call(() => machine.run(instructions) as EmulatorSnapshot);
    },
    async snapshot() {
      return call(() => machine.snapshot() as EmulatorSnapshot);
    },
    async framebuffer() {
      return call(() => machine.framebuffer());
    },
    async readMemory(address, length) {
      return call(() => machine.readMemory(address, length));
    },
    dispose() {
      machine.free();
    },
  };
}
