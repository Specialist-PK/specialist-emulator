import { describe, expect, it } from "vitest";

import { framebufferToRgba } from "./framebuffer";

describe("framebufferToRgba", () => {
  it("places the most significant bit on the left", () => {
    const rgba = framebufferToRgba(new Uint8Array([0x80, 0x01]), 16, 1);
    expect(Array.from(rgba.slice(0, 4))).toEqual([220, 220, 220, 255]);
    expect(Array.from(rgba.slice(4, 8))).toEqual([5, 5, 5, 255]);
    expect(Array.from(rgba.slice(60, 64))).toEqual([220, 220, 220, 255]);
  });

  it("rejects an invalid buffer size", () => {
    expect(() => framebufferToRgba(new Uint8Array(1))).toThrow("Invalid framebuffer size");
  });
});
