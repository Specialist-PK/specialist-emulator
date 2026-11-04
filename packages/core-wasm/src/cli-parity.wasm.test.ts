import { execFile } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import type { Emulator, EmulatorSnapshot } from "@specialist/core-api";
import { describe, expect, it } from "vitest";

import { createCore } from "./index";

const execFileAsync = promisify(execFile);
const repository = fileURLToPath(new URL("../../..", import.meta.url));
const program = Uint8Array.from([
  0x31, 0xff, 0x8f, 0x21, 0x00, 0x90, 0x01, 0x00, 0x30, 0x7d, 0xac, 0x77, 0x23, 0x0b, 0x78, 0xb1,
  0xc2, 0x09, 0xc0, 0x76,
]);

describe("CLI and WASM parity", () => {
  it("returns the same snapshot and framebuffer for one Monitor 2 ROM", async () => {
    const directory = await mkdtemp(join(tmpdir(), "specialist-cli-wasm-"));
    const romPath = join(directory, "monitor2-test.rom");
    const rom = new Uint8Array(0x1800);
    rom.set(program);
    let core: Emulator | undefined;

    try {
      await writeFile(romPath, rom);
      const { stdout } = await execFileAsync(
        "cargo",
        ["run", "--quiet", "-p", "specialist-cli", "--", "monitor2", romPath, "200000", "--json"],
        { cwd: repository, maxBuffer: 2 * 1024 * 1024 },
      );
      const cli = JSON.parse(stdout) as { snapshot: EmulatorSnapshot; framebuffer: number[] };

      const wasm = await readFile(new URL("../pkg/specialist_core_wasm_bg.wasm", import.meta.url));
      core = await createCore({ wasm });
      await core.loadMonitor2Rom(rom);
      await core.run(200_000);

      expect(cli.snapshot).toEqual(await core.snapshot());
      expect(cli.framebuffer).toEqual(Array.from(await core.framebuffer()));
      expect(cli.snapshot.cpu.pc).toBe(0xc014);
      expect(cli.snapshot.cycles).toBe(552_997);
      expect(cli.snapshot.frame).toBe(14);
      expect(cli.framebuffer).toHaveLength(12_288);
      expect(cli.framebuffer[0]).toBe(0x90);
      expect(cli.framebuffer[1]).toBe(0x91);
      expect(cli.framebuffer[12_287]).toBe(0);
    } finally {
      core?.dispose();
      await rm(directory, { recursive: true, force: true });
    }
  }, 30_000);
});
