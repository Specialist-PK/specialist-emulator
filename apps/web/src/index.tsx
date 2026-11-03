import { render } from "@solidjs/web";
import { App, CoreContext, describeError } from "@specialist/app";
import { createCore } from "@specialist/core-wasm";

import "./index.css";

const root = document.getElementById("root")!;
try {
  const core = await createCore();
  render(
    () => (
      <CoreContext value={core}>
        <App subtitle="Rust-ядро выполняется в WebAssembly" />
      </CoreContext>
    ),
    root,
  );
} catch (error) {
  render(
    () => (
      <p role="alert" class="p-6 text-red-400">
        Не удалось запустить ядро: {describeError(error)}
      </p>
    ),
    root,
  );
}
