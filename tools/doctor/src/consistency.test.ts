import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

import { packageManagerVersion, readPackageList, toolchainChannel } from "./parse";

// Версии живут в нескольких файлах (package.json, devcontainer, CI). Этот тест ловит расхождение до того,
// как оно проявится как «у меня работает».
const root = join(import.meta.dirname, "../../..");
const read = (path: string) => readFileSync(join(root, path), "utf8");

function markdownFiles(directory: string): string[] {
  return readdirSync(join(root, directory), { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name).replaceAll("\\", "/");
    if (entry.isDirectory()) return markdownFiles(path);
    return entry.name.endsWith(".md") ? [path] : [];
  });
}

describe("documentation languages", () => {
  it("requires a canonical Russian document and an English companion", () => {
    const documents = [
      "README.md",
      "README.en.md",
      "AGENTS.md",
      "AGENTS.en.md",
      ".github/PULL_REQUEST_TEMPLATE.md",
      ".github/PULL_REQUEST_TEMPLATE.en.md",
      ...markdownFiles("docs"),
    ];
    for (const document of documents) {
      if (document.endsWith(".en.md")) {
        expect(documents, document).toContain(document.replace(/\.en\.md$/, ".md"));
      } else {
        expect(documents, document).toContain(document.replace(/\.md$/, ".en.md"));
      }
    }
  });
});

describe("version pins agree across the repository", () => {
  it("devcontainer builds with the same bun as packageManager", () => {
    const devcontainer = JSON.parse(read(".devcontainer/devcontainer.json")) as {
      build: { args: { BUN_VERSION: string } };
    };
    expect(devcontainer.build.args.BUN_VERSION).toBe(packageManagerVersion(read("package.json")));
  });

  it("devcontainer and workspace recommend the same VS Code extensions", () => {
    const devcontainer = JSON.parse(read(".devcontainer/devcontainer.json")) as {
      customizations: { vscode: { extensions: string[] } };
    };
    const workspace = JSON.parse(read(".vscode/extensions.json")) as { recommendations: string[] };
    expect(devcontainer.customizations.vscode.extensions.toSorted()).toEqual(
      workspace.recommendations.toSorted(),
    );
  });

  it("Rust is pinned to an exact version, not a floating channel", () => {
    expect(toolchainChannel(read("rust-toolchain.toml"))).toMatch(/^\d+\.\d+\.\d+$/);
  });

  it("CI takes versions from the files instead of hard-coding them", () => {
    const ci = read(".github/workflows/ci.yml");
    expect(ci).not.toMatch(/bun-version:\s*\d/);
    expect(ci).not.toContain("dtolnay/rust-toolchain");
    expect(ci).toContain("bun-version-file: package.json");
    expect(ci).toContain("tools/system-deps/debian.txt");
    expect(ci).toContain("for result in $RESULTS; do");
    expect(ci).toContain("success|skipped");
    expect(ci).toContain('echo "unexpected job result: $result"; exit 1');
    expect(ci).not.toContain("macos-latest");
    expect(ci).not.toContain("windows-latest");
  });

  it("Pages deployment keeps its safeguards and version sources", () => {
    const deploy = read(".github/workflows/deploy.yml");
    expect(deploy).toContain("pull_request:");
    expect(deploy).toContain("branches: [main]");
    expect(deploy).toContain("workflow_dispatch:");
    expect(deploy).toContain("if: github.event_name != 'pull_request'");
    expect(deploy).toContain("group: pages");
    expect(deploy).toContain("cancel-in-progress: false");
    expect(deploy).toContain("pages: write");
    expect(deploy).toContain("id-token: write");
    expect(deploy).toContain("actions/configure-pages@v6");
    expect(deploy).toContain("enablement: ${{ github.event_name != 'pull_request' }}");
    expect(deploy).toContain("bun-version-file: package.json");
    expect(deploy).toContain("cargo pkgid wasm-bindgen");
    expect(deploy).toContain("steps.pages.outputs.base_path");
    expect(deploy).toContain("path: apps/web/dist");
    expect(deploy).not.toMatch(/bun-version:\s*\d/);
    expect(read("apps/web/vite.config.ts")).toContain('base: process.env["BASE_PATH"] ?? "/"');
    expect(read("tests/e2e/src/emulator.e2e.test.ts")).toContain(
      'new URL(process.env["BASE_PATH"] ?? "/", "http://127.0.0.1:4173")',
    );
  });

  it("CI and the image share one system package list", () => {
    expect(read(".devcontainer/Dockerfile")).toContain("tools/system-deps/debian.txt");
    expect(readPackageList(read("tools/system-deps/debian.txt"))).toContain(
      "libwebkit2gtk-4.1-dev",
    );
  });
  it("both workflows run the same cargo-deny", () => {
    const pinned = (file: string) => /cargo-deny@([\d.]+)/.exec(read(file))?.[1];
    expect(pinned(".github/workflows/ci.yml")).toMatch(/^\d+\.\d+\.\d+$/);
    expect(pinned(".github/workflows/audit.yml")).toBe(pinned(".github/workflows/ci.yml"));
  });

  it("every advisory exception in deny.toml states a reason", () => {
    const exceptions = read("deny.toml")
      .split("\n")
      .filter((line) => line.includes('{ id = "RUSTSEC-'));
    expect(exceptions.length).toBeGreaterThan(0);
    for (const line of exceptions) expect(line, line).toMatch(/reason = ".{20,}"/);
  });
  it("VS Code and Zed keep shared debug scenarios and tasks in sync", () => {
    const vscodeLaunch = JSON.parse(read(".vscode/launch.json")) as {
      configurations: Array<{ name: string; request: string; preLaunchTask?: string }>;
    };
    const zedDebug = JSON.parse(read(".zed/debug.json")) as Array<{
      label: string;
      request: string;
      adapter?: string;
      build?: string | { command: string };
    }>;
    const vscodeNames = vscodeLaunch.configurations.map((config) => config.name);
    for (const config of zedDebug) {
      expect(vscodeNames).toContain(config.label);
      if (config.adapter === "CodeLLDB" && typeof config.build === "object")
        expect(config.build.command).toBe("cargo");
      if (config.label.startsWith("Web frontend:")) {
        expect(config.build === "web: ensure dev server").toBe(config.request === "launch");
      }
    }
    for (const config of vscodeLaunch.configurations) {
      if (config.name.startsWith("Web frontend:")) {
        expect(config.preLaunchTask === "web: ensure dev server").toBe(config.request === "launch");
      }
    }
    expect(vscodeNames).toContain("Web frontend: launch Firefox");
    expect(vscodeNames).toContain("Web frontend: attach Firefox");

    const vscodeTasks = JSON.parse(read(".vscode/tasks.json")) as {
      tasks: Array<{ label: string }>;
    };
    const zedTasks = JSON.parse(read(".zed/tasks.json")) as Array<{ label: string }>;
    expect(vscodeTasks.tasks.map((task) => task.label).toSorted()).toEqual(
      zedTasks.map((task) => task.label).toSorted(),
    );
  });

  it("the MIT license is in place and every manifest declares it", () => {
    const mit = read("LICENSE");
    expect(mit).toMatch(/^MIT License\n/);
    expect(mit).toContain("Copyright (c) 2026 Specialist-PK contributors");
    expect(mit).toContain("Permission is hereby granted, free of charge");

    expect(read("Cargo.toml")).toMatch(/^license = "MIT"$/m);
    expect(JSON.parse(read("package.json")).license).toBe("MIT");
    const crates = ["crates/core", "crates/cli", "crates/core-wasm", "apps/native/src-tauri"];
    for (const crate of crates)
      expect(read(`${crate}/Cargo.toml`), crate).toContain("license.workspace = true");
  });
});
