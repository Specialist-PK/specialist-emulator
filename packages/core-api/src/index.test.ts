import { describe, expect, it } from "vitest";

import { EmulatorException, isEmulatorError } from "./index";

describe("EmulatorException", () => {
  it("keeps a typed error from the platform boundary", () => {
    const error = { kind: "invalidAddress", address: 65_536 } as const;
    expect(isEmulatorError(error)).toBe(true);
    expect(new EmulatorException(error).error).toEqual(error);
  });

  it("accepts typed ROM errors", () => {
    expect(isEmulatorError({ kind: "invalidRomSize", expected: 0x1800, actual: 1 })).toBe(true);
    expect(isEmulatorError({ kind: "missingRom" })).toBe(true);
  });

  it("rejects malformed values", () => {
    expect(isEmulatorError(null)).toBe(false);
    expect(isEmulatorError({ kind: "rangeOutOfBounds", address: 1 })).toBe(false);
    expect(isEmulatorError({ kind: "invalidRomSize", expected: 0x1800 })).toBe(false);
    expect(isEmulatorError({ kind: "unknown" })).toBe(false);
  });
});
