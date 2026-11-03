import { render } from "@solidjs/web";
import { App, CoreContext } from "@specialist/app";

import { createIpcCore } from "./core";

import "./index.css";

render(
  () => (
    <CoreContext value={createIpcCore()}>
      <App subtitle="Нативное Rust-ядро через Tauri IPC" />
    </CoreContext>
  ),
  document.getElementById("root")!,
);
