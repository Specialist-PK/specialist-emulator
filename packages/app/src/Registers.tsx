import type { EmulatorSnapshot } from "@specialist/core-api";
import { For } from "solid-js";

function hex(value: number, width: number): string {
  return value.toString(16).toUpperCase().padStart(width, "0");
}

export function Registers(props: { snapshot: EmulatorSnapshot }) {
  const registers = [
    { name: "A", value: () => hex(props.snapshot.cpu.a, 2) },
    { name: "BC", value: () => `${hex(props.snapshot.cpu.b, 2)}${hex(props.snapshot.cpu.c, 2)}` },
    { name: "DE", value: () => `${hex(props.snapshot.cpu.d, 2)}${hex(props.snapshot.cpu.e, 2)}` },
    { name: "HL", value: () => `${hex(props.snapshot.cpu.h, 2)}${hex(props.snapshot.cpu.l, 2)}` },
    { name: "SP", value: () => hex(props.snapshot.cpu.sp, 4) },
    { name: "PC", value: () => hex(props.snapshot.cpu.pc, 4) },
    { name: "S", value: () => Number(props.snapshot.cpu.flags.sign).toString() },
    { name: "Z", value: () => Number(props.snapshot.cpu.flags.zero).toString() },
    { name: "AC", value: () => Number(props.snapshot.cpu.flags.auxiliaryCarry).toString() },
    { name: "P", value: () => Number(props.snapshot.cpu.flags.parity).toString() },
    { name: "C", value: () => Number(props.snapshot.cpu.flags.carry).toString() },
  ];

  return (
    <dl class="grid grid-cols-3 gap-2 font-mono text-sm sm:grid-cols-6">
      <For each={registers}>
        {(register) => (
          <div class="rounded bg-surface-muted px-3 py-2">
            <dt class="text-xs text-ink/60">{register.name}</dt>
            <dd>{register.value()}</dd>
          </div>
        )}
      </For>
    </dl>
  );
}
