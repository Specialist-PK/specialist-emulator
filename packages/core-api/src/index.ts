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

export const SPECIALIST_KEYS = [
  "cr",
  "bsp",
  "period",
  "colon",
  "minus",
  "clear",
  "lf",
  "slash",
  "backslash",
  "h",
  "0",
  "f11",
  "right",
  "comma",
  "v",
  "z",
  "9",
  "f10",
  "rpt",
  "at",
  "d",
  "rbracket",
  "8",
  "f9",
  "left",
  "b",
  "l",
  "lbracket",
  "7",
  "f8",
  "space",
  "x",
  "o",
  "g",
  "6",
  "f7",
  "esc",
  "t",
  "r",
  "n",
  "5",
  "f6",
  "tab",
  "i",
  "p",
  "e",
  "4",
  "f5",
  "down",
  "m",
  "a",
  "k",
  "3",
  "f4",
  "up",
  "s",
  "w",
  "u",
  "2",
  "f3",
  "home",
  "caret",
  "y",
  "c",
  "1",
  "f2",
  "lang",
  "q",
  "f",
  "j",
  "semicolon",
  "f1",
  "shift",
] as const;

export type SpecialistKey = (typeof SPECIALIST_KEYS)[number];

export interface SpecialistKeyEvent {
  key: SpecialistKey;
  pressed: boolean;
  cycle?: number;
}

export type EmulatorError =
  | { kind: "invalidAddress"; address: number }
  | { kind: "rangeOutOfBounds"; address: number; length: number }
  | { kind: "memoryNotWritable"; address: number; length: number }
  | { kind: "invalidRomSize"; expected: number; actual: number }
  | { kind: "missingRom" }
  | { kind: "unknownKey"; key: string }
  | { kind: "eventCycleInPast"; cycle: number };

export interface Emulator {
  reset(): Promise<EmulatorSnapshot>;
  loadDemo(): Promise<EmulatorSnapshot>;
  loadMonitor2Rom(rom: Uint8Array): Promise<EmulatorSnapshot>;
  loadProgram(program: Uint8Array, address: number, entryPoint?: number): Promise<EmulatorSnapshot>;
  keyEvent(event: SpecialistKeyEvent): Promise<EmulatorSnapshot>;
  run(instructions: number): Promise<EmulatorSnapshot>;
  snapshot(): Promise<EmulatorSnapshot>;
  framebuffer(): Promise<Uint8Array>;
  readMemory(address: number, length: number): Promise<Uint8Array>;
  dispose(): void;
}

function emulatorErrorMessage(error: EmulatorError): string {
  switch (error.kind) {
    case "invalidAddress":
      return `Invalid address: ${error.address}`;
    case "rangeOutOfBounds":
      return `Memory range is out of bounds: ${error.address} + ${error.length}`;
    case "memoryNotWritable":
      return `Memory range is not writable: ${error.address} + ${error.length}`;
    case "invalidRomSize":
      return `Invalid Monitor 2 ROM size: expected ${error.expected}, got ${error.actual}`;
    case "missingRom":
      return "Monitor 2 ROM is not loaded";
    case "unknownKey":
      return `Unknown Specialist key: ${error.key}`;
    case "eventCycleInPast":
      return `Key event cycle is in the past: ${error.cycle}`;
  }
}

export class EmulatorException extends Error {
  readonly error: EmulatorError;

  constructor(error: EmulatorError) {
    super(emulatorErrorMessage(error));
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
    case "unknownKey":
      return typeof error["key"] === "string";
    case "eventCycleInPast":
      return typeof error["cycle"] === "number";
    default:
      return false;
  }
}
