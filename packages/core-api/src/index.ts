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
  | { kind: "rangeOutOfBounds"; address: number; length: number };

export interface Emulator {
  reset(): Promise<EmulatorSnapshot>;
  loadDemo(): Promise<EmulatorSnapshot>;
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
        : `Memory range is out of bounds: ${error.address} + ${error.length}`,
    );
    this.name = "EmulatorException";
    this.error = error;
  }
}

export function isEmulatorError(value: unknown): value is EmulatorError {
  if (typeof value !== "object" || value === null) return false;
  const error = value as Record<string, unknown>;
  if (typeof error["address"] !== "number") return false;
  if (error["kind"] === "invalidAddress") return true;
  return error["kind"] === "rangeOutOfBounds" && typeof error["length"] === "number";
}
