import { spawn, type ChildProcess } from "node:child_process";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { chromium, type Browser } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const browserPath = [
  process.env["CHROME_PATH"],
  "/usr/bin/chromium",
  "/usr/bin/chromium-browser",
  "/usr/bin/google-chrome",
  "/opt/google/chrome/chrome",
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
].find((path) => path !== undefined && path !== "" && existsSync(path));
const required = process.env["E2E_REQUIRE_BROWSER"] === "1";
const baseUrl = new URL(process.env["BASE_PATH"] ?? "/", "http://127.0.0.1:4173").href;

if (browserPath === undefined && !required) {
  console.warn("E2E skipped: Chrome was not found; set CHROME_PATH to enable it");
}

describe.runIf(browserPath !== undefined || required)("web emulator", () => {
  let server: ChildProcess | undefined;
  let browser: Browser | undefined;

  beforeAll(async () => {
    if (browserPath === undefined) throw new Error("Chrome was not found: set CHROME_PATH");
    const root = fileURLToPath(new URL("../../..", import.meta.url));
    server = spawn(
      "bun",
      [
        "run",
        "--filter",
        "@specialist/web",
        "preview",
        "--",
        "--host",
        "127.0.0.1",
        "--port",
        "4173",
        "--strictPort",
      ],
      { cwd: root, stdio: "ignore" },
    );
    let ready = false;
    for (let attempt = 0; attempt < 100; attempt += 1) {
      try {
        ready = (await fetch(baseUrl)).ok;
      } catch {}
      if (ready) break;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    if (!ready) throw new Error("Vite preview did not start");
    browser = await chromium.launch({
      executablePath: browserPath,
      headless: true,
      args: ["--no-sandbox"],
    });
  }, 30_000);

  afterAll(async () => {
    await browser?.close();
    server?.kill();
  });

  it("steps, runs to HLT and loads a binary through the real UI", async () => {
    const page = await browser!.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto(baseUrl);
    await page.getByRole("button", { name: "Шаг" }).waitFor();
    await page.waitForFunction(() => document.querySelector("dl")?.textContent?.includes("PC0000"));

    await page.getByRole("button", { name: "Шаг" }).click();
    await page.waitForFunction(() => document.querySelector("dl")?.textContent?.includes("PC0003"));

    await page.getByRole("button", { name: "Пуск" }).click();
    await page.waitForFunction(() =>
      document.querySelector("output")?.textContent?.includes("HLT"),
    );
    await page.waitForFunction(() => document.querySelector("dl")?.textContent?.includes("PC0014"));

    await page.getByRole("textbox").fill("0100");
    await page.locator("#program-bin").setInputFiles({
      name: "halt.bin",
      mimeType: "application/octet-stream",
      buffer: Buffer.from([0x76]),
    });
    await page.waitForFunction(() => document.querySelector("dl")?.textContent?.includes("PC0100"));
    await page.screenshot({ path: "target/e2e-specialist.png", fullPage: true });
    expect(errors).toEqual([]);
    await page.close();
  }, 30_000);

  it("loads Monitor 2 ROM and scans a physical key", async () => {
    const page = await browser!.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto(baseUrl);
    await page.getByRole("button", { name: "Шаг" }).waitFor();
    await page.waitForFunction(() => document.querySelector("dl")?.textContent?.includes("PC0000"));

    await page.locator("#monitor2-rom").setInputFiles({
      name: "monitor2-test.rom",
      mimeType: "application/octet-stream",
      buffer: Buffer.alloc(0x1800),
    });
    await page.waitForFunction(() => document.querySelector("dl")?.textContent?.includes("PCC000"));

    await page.locator("#program-bin").setInputFiles({
      name: "keyboard-test.bin",
      mimeType: "application/octet-stream",
      buffer: Buffer.from([
        0x3e, 0x82, 0x32, 0x03, 0xf8, 0xaf, 0x32, 0x00, 0xf8, 0x32, 0x02, 0xf8, 0x3a, 0x01, 0xf8,
        0xfe, 0xef, 0xc2, 0x15, 0x00, 0x76, 0x76,
      ]),
    });
    await page.waitForFunction(() => document.querySelector("dl")?.textContent?.includes("PC0000"));
    await page.keyboard.down("a");
    await page.getByRole("button", { name: "Пуск" }).click();
    await page.waitForFunction(() => document.querySelector("dl")?.textContent?.includes("PC0015"));
    await page.keyboard.up("a");

    expect(errors).toEqual([]);
    await page.close();
  }, 30_000);
});
