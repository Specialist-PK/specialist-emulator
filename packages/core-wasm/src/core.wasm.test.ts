import { readFile } from "node:fs/promises";

import { EmulatorException } from "@specialist/core-api";
import { describe, expect, it } from "vitest";

import { createCore } from "./index";

async function createTestCore() {
  const wasm = await readFile(new URL("../pkg/specialist_core_wasm_bg.wasm", import.meta.url));
  return createCore({ wasm });
}

describe("WASM emulator", () => {
  it("runs the demo and exposes real video memory", async () => {
    const core = await createTestCore();
    const loaded = await core.loadDemo();
    expect(loaded.cpu.pc).toBe(0);

    const stopped = await core.run(200_000);
    expect(stopped.cpu.halted).toBe(true);
    expect(stopped.cpu.pc).toBe(0x14);
    expect(stopped.cpu.sp).toBe(0x8fff);

    const frame = await core.framebuffer();
    expect(frame).toHaveLength(12_288);
    expect(frame[0]).toBe(0x90);
    expect(frame[1]).toBe(0x91);
    expect(frame[12_287]).toBe(0x40);
    core.dispose();
  });

  it("returns typed errors for invalid memory ranges", async () => {
    const core = await createTestCore();
    const result = await core.readMemory(0xffff, 2).catch((error: unknown) => error);
    expect(result).toBeInstanceOf(EmulatorException);
    expect((result as EmulatorException).error).toEqual({
      kind: "rangeOutOfBounds",
      address: 0xffff,
      length: 2,
    });
    core.dispose();
  });

  it("loads Monitor 2 ROM and reports an invalid size", async () => {
    const core = await createTestCore();
    const rom = new Uint8Array(0x1800);
    rom[0] = 0x76;
    const loaded = await core.loadMonitor2Rom(rom);
    expect(loaded.cpu.pc).toBe(0xc000);

    const stopped = await core.run(1);
    expect(stopped.cpu.halted).toBe(true);
    expect(stopped.cpu.pc).toBe(0xc001);

    const result = await core.loadMonitor2Rom(new Uint8Array(1)).catch((error: unknown) => error);
    expect(result).toBeInstanceOf(EmulatorException);
    expect((result as EmulatorException).error).toEqual({
      kind: "invalidRomSize",
      expected: 0x1800,
      actual: 1,
    });
    core.dispose();
  });
});
