import { describe, expect, it } from "vitest";

import { runSetup, type SetupIO } from "./setup";

const LOCK = `[[package]]
name = "wasm-bindgen"
version = "0.2.130"
`;

function fixture(lock?: string | null) {
  const calls: { command: string; args: string[] }[] = [];
  const messages: string[] = [];
  const failures = new Map<string, number>();
  const io: SetupIO = {
    read: () => (lock === null ? undefined : (lock ?? LOCK)),
    run: (command, args) => {
      calls.push({ command, args });
      return failures.get(JSON.stringify([command, ...args])) ?? 0;
    },
    log: (message) => messages.push(message),
  };
  return { calls, messages, failures, io };
}

const INSTALL = ["install", "--frozen-lockfile"];
const DOCTOR = ["tools/doctor/src/index.ts", "core", "wasm"];

describe("runSetup", () => {
  it("installs frozen JS dependencies and checks only CLI/web by default", () => {
    const { calls, messages, io } = fixture();

    expect(runSetup([], io)).toBe(0);
    expect(calls).toEqual([
      { command: "bun", args: INSTALL },
      { command: "bun", args: DOCTOR },
    ]);
    expect(messages.join("\n")).toContain("bun run doctor native");
    expect(calls.some(({ command }) => command === "sudo" || command === "apt-get")).toBe(false);
  });

  it("installs the declared Rust toolchain and locked wasm-bindgen CLI on request", () => {
    const { calls, io } = fixture();

    expect(runSetup(["--install"], io)).toBe(0);
    expect(calls).toEqual([
      { command: "rustup", args: ["toolchain", "install"] },
      {
        command: "cargo",
        args: ["install", "wasm-bindgen-cli", "--locked", "--version", "0.2.130"],
      },
      { command: "bun", args: INSTALL },
      { command: "bun", args: DOCTOR },
    ]);
  });

  it("stops at the first failed command", () => {
    const { calls, failures, io } = fixture();
    failures.set(JSON.stringify(["bun", ...INSTALL]), 1);

    expect(runSetup([], io)).toBe(1);
    expect(calls).toEqual([{ command: "bun", args: INSTALL }]);
  });

  it("does not continue installation when rustup fails", () => {
    const { calls, failures, io } = fixture();
    failures.set(JSON.stringify(["rustup", "toolchain", "install"]), 1);

    expect(runSetup(["--install"], io)).toBe(1);
    expect(calls).toEqual([{ command: "rustup", args: ["toolchain", "install"] }]);
  });

  it("reports a failed final doctor check", () => {
    const { failures, messages, io } = fixture();
    failures.set(JSON.stringify(["bun", ...DOCTOR]), 1);

    expect(runSetup([], io)).toBe(1);
    expect(messages.join("\n")).not.toContain("окружение настроено");
  });

  it("does not guess a wasm-bindgen version when the lock entry is missing", () => {
    const { calls, messages, io } = fixture(null);

    expect(runSetup(["--install"], io)).toBe(1);
    expect(calls).toEqual([{ command: "rustup", args: ["toolchain", "install"] }]);
    expect(messages.join("\n")).toContain("Cargo.lock");
  });

  it("rejects unknown arguments without changing the environment", () => {
    const { calls, io } = fixture();

    expect(runSetup(["--force"], io)).toBe(1);
    expect(calls).toEqual([]);
  });

  it("shows help without changing the environment", () => {
    const { calls, messages, io } = fixture();

    expect(runSetup(["--help"], io)).toBe(0);
    expect(calls).toEqual([]);
    expect(messages.join("\n")).toContain("bun run setup --install");
  });
});
