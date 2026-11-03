import type { EmulatorSnapshot } from "@specialist/core-api";
import { createSignal, onSettled, Show, useContext } from "solid-js";

import { CoreContext } from "./core-context";
import { describeError } from "./errors";
import { Registers } from "./Registers";
import { Screen } from "./Screen";

const INSTRUCTIONS_PER_FRAME = 20_000;

export function App(props: { subtitle?: string }) {
  const core = useContext(CoreContext);
  const [snapshot, setSnapshot] = createSignal<EmulatorSnapshot>();
  const [frame, setFrame] = createSignal<Uint8Array>();
  const [running, setRunning] = createSignal(false);
  const [error, setError] = createSignal("");
  const [address, setAddress] = createSignal("0000");

  async function perform(action: () => Promise<EmulatorSnapshot>) {
    try {
      const next = await action();
      setSnapshot(next);
      setFrame(await core.framebuffer());
      setError("");
      return next;
    } catch (cause) {
      setError(describeError(cause));
      setRunning(false);
      return undefined;
    }
  }

  async function tick() {
    const next = await perform(() => core.run(INSTRUCTIONS_PER_FRAME));
    if (next?.cpu.halted) setRunning(false);
    if (next && running() && !next.cpu.halted) {
      requestAnimationFrame(() => void tick());
    }
  }

  function start() {
    if (running()) return;
    setRunning(true);
    void tick();
  }

  async function loadDemo() {
    setRunning(false);
    await perform(() => core.loadDemo());
  }

  async function loadFile(file: File) {
    setRunning(false);
    try {
      if (!/^[\da-fA-F]{1,4}$/.test(address())) {
        throw new Error("Адрес должен содержать от 1 до 4 шестнадцатеричных цифр");
      }
      const startAddress = Number.parseInt(address(), 16);
      const program = new Uint8Array(await file.arrayBuffer());
      await perform(() => core.loadProgram(program, startAddress));
    } catch (cause) {
      setError(describeError(cause));
    }
  }

  onSettled(() => {
    void loadDemo();
    return () => setRunning(false);
  });

  return (
    <main class="mx-auto flex max-w-5xl flex-col gap-5 p-4 sm:p-6">
      <header class="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 class="text-2xl font-semibold">Специалист</h1>
          <p class="text-sm text-ink/60">Эмулятор Intel 8080 · 384 × 256</p>
          <Show when={props.subtitle}>{(text) => <p class="text-xs text-ink/50">{text()}</p>}</Show>
        </div>
        <output class="font-mono text-sm text-ink/70">
          {running() ? "Выполняется" : snapshot()?.cpu.halted ? "HLT" : "Остановлен"}
          {snapshot() ? ` · ${snapshot()!.cycles.toLocaleString("ru-RU")} циклов` : ""}
        </output>
      </header>

      <Screen frame={frame()} />

      <section class="flex flex-wrap items-center gap-2" aria-label="Управление эмулятором">
        <Show
          when={running()}
          fallback={
            <button class="rounded-control bg-accent px-4 py-2 text-accent-ink" onClick={start}>
              Пуск
            </button>
          }
        >
          <button
            class="rounded-control bg-surface-muted px-4 py-2"
            onClick={() => setRunning(false)}
          >
            Пауза
          </button>
        </Show>
        <button
          class="rounded-control bg-surface-muted px-4 py-2 disabled:opacity-40"
          disabled={running()}
          onClick={() => void perform(() => core.run(1))}
        >
          Шаг
        </button>
        <button
          class="rounded-control bg-surface-muted px-4 py-2 disabled:opacity-40"
          disabled={running()}
          onClick={() => void perform(() => core.reset())}
        >
          Сброс
        </button>
        <button
          class="rounded-control bg-surface-muted px-4 py-2 disabled:opacity-40"
          disabled={running()}
          onClick={() => void loadDemo()}
        >
          Демо
        </button>
      </section>

      <section class="flex flex-wrap items-end gap-3 rounded-control bg-surface-muted p-3">
        <label class="text-sm">
          <span class="mb-1 block text-ink/60">Адрес загрузки (hex)</span>
          <input
            class="w-28 rounded bg-surface px-3 py-2 font-mono"
            value={address()}
            onInput={(event) => setAddress(event.currentTarget.value)}
          />
        </label>
        <label class="cursor-pointer rounded-control bg-accent px-4 py-2 text-sm text-accent-ink">
          Загрузить BIN
          <input
            type="file"
            class="hidden"
            onChange={(event) => {
              const file = event.currentTarget.files?.[0];
              if (file) void loadFile(file);
              event.currentTarget.value = "";
            }}
          />
        </label>
      </section>

      <Show when={error()}>
        {(message) => (
          <p role="alert" class="text-red-400">
            {message()}
          </p>
        )}
      </Show>
      <Show when={snapshot()}>{(state) => <Registers snapshot={state()} />}</Show>
    </main>
  );
}
