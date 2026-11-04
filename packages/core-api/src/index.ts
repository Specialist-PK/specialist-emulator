export interface FlagsSnapshot {
  sign: boolean;
  zero: boolean;
  auxiliaryCarry: boolean;
  parity: boolean;
  carry: boolean;
}

export interface CpuSnapshot {
  a: number;
  b: number;
  c: number;
  d: number;
  e: number;
  h: number;
  l: number;
  sp: number;
  pc: number;
  flags: FlagsSnapshot;
  interruptsEnabled: boolean;
  halted: boolean;
}

export interface EmulatorSnapshot {
  cpu: CpuSnapshot;
  cycles: number;
}

export type EmulatorError =
  | { kind: "invalidAddress"; address: number }
  | { kind: "rangeOutOfBounds"; address: number; length: number }
  | { kind: "memoryNotWritable"; address: number; length: number }
  | { kind: "invalidRomSize"; expected: number; actual: number }
  | { kind: "missingRom" };

export interface Emulator {
  reset(): Promise<EmulatorSnapshot>;
  loadDemo(): Promise<EmulatorSnapshot>;
  loadMonitor2Rom(rom: Uint8Array): Promise<EmulatorSnapshot>;
  loadProgram(program: Uint8Array, address: number, entryPoint?: number): Promise<EmulatorSnapshot>;
  run(instructions: number): Promise<EmulatorSnapshot>;
  snapshot(): Promise<EmulatorSnapshot>;
  framebuffer(): Promise<Uint8Array>;
  readMemory(address: number, length: number): Promise<Uint8Array>;
  dispose(): void;
}

export class EmulatorException extends Error {
  readonly error: EmulatorError;

  constructor(error: EmulatorError) {
    super(
      error.kind === "invalidAddress"
        ? `Invalid address: ${error.address}`
        : error.kind === "rangeOutOfBounds"
          ? `Memory range is out of bounds: ${error.address} + ${error.length}`
          : error.kind === "memoryNotWritable"
            ? `Memory range is not writable: ${error.address} + ${error.length}`
            : error.kind === "invalidRomSize"
              ? `Invalid Monitor 2 ROM size: expected ${error.expected}, got ${error.actual}`
              : "Monitor 2 ROM is not loaded",
    );
    this.name = "EmulatorException";
    this.error = error;
  }
}

export function isEmulatorError(value: unknown): value is EmulatorError {
  if (typeof value !== "object" || value === null) return false;
  const error = value as Record<string, unknown>;
  switch (error["kind"]) {
    case "invalidAddress":
      return typeof error["address"] === "number";
    case "rangeOutOfBounds":
    case "memoryNotWritable":
      return typeof error["address"] === "number" && typeof error["length"] === "number";
    case "invalidRomSize":
      return typeof error["expected"] === "number" && typeof error["actual"] === "number";
    case "missingRom":
      return true;
    default:
      return false;
  }
}
