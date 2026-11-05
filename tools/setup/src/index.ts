import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { runSetup } from "./setup";

const root = join(dirname(fileURLToPath(import.meta.url)), "../../..");

const exitCode = runSetup(process.argv.slice(2), {
  read: (path) => {
    try {
      return readFileSync(join(root, path), "utf8");
    } catch (error) {
      if ((error as { code?: string }).code === "ENOENT") return undefined;
      throw error;
    }
  },
  run: (command, args) => {
    try {
      return Bun.spawnSync([command, ...args], {
        cwd: root,
        stdio: ["ignore", "inherit", "inherit"],
      }).exitCode;
    } catch (error) {
      console.error(error instanceof Error ? error.message : String(error));
      return 1;
    }
  },
  log: console.log,
});

process.exitCode = exitCode;
