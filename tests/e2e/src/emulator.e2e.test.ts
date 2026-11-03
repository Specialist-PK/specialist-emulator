import { spawn, type ChildProcess } from "node:child_process";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { chromium, type Browser } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

let server: ChildProcess | undefined;
let browser: Browser | undefined;

function browserPath(): string | undefined {
  return [
    process.env["CHROME_PATH"],
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/usr/bin/google-chrome",
    "/opt/google/chrome/chrome",
  ].find((path) => path !== undefined && existsSync(path));
}

beforeAll(async () => {
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
      ready = (await fetch("http://127.0.0.1:4173")).ok;
    } catch {}
    if (ready) break;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  if (!ready) throw new Error("Vite preview did not start");
  browser = await chromium.launch({
    executablePath: browserPath(),
    headless: true,
    args: ["--no-sandbox"],
  });
}, 30_000);

afterAll(async () => {
  await browser?.close();
  server?.kill();
});

describe("web emulator", () => {
  it("steps, runs to HLT and loads a binary through the real UI", async () => {
    const page = await browser!.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto("http://127.0.0.1:4173");
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
    await page.locator('input[type="file"]').setInputFiles({
      name: "halt.bin",
      mimeType: "application/octet-stream",
      buffer: Buffer.from([0x76]),
    });
    await page.waitForFunction(() => document.querySelector("dl")?.textContent?.includes("PC0100"));
    await page.screenshot({ path: "target/e2e-specialist.png", fullPage: true });
    expect(errors).toEqual([]);
    await page.close();
  }, 30_000);
});
