import { describe, expect, it } from "vitest";

import { EmulatorException, isEmulatorError } from "./index";

describe("EmulatorException", () => {
  it("keeps a typed error from the platform boundary", () => {
    const error = { kind: "invalidAddress", address: 65_536 } as const;
    expect(isEmulatorError(error)).toBe(true);
    expect(new EmulatorException(error).error).toEqual(error);
  });

  it("rejects malformed values", () => {
    expect(isEmulatorError(null)).toBe(false);
    expect(isEmulatorError({ kind: "rangeOutOfBounds", address: 1 })).toBe(false);
  });
});
