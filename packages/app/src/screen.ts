export const SCREEN_WIDTH = 384;
export const SCREEN_HEIGHT = 256;
export const FRAMEBUFFER_SIZE = (SCREEN_WIDTH / 8) * SCREEN_HEIGHT;

export function framebufferToRgba(
  framebuffer: Uint8Array,
  width = SCREEN_WIDTH,
  height = SCREEN_HEIGHT,
): Uint8ClampedArray {
  if (framebuffer.length !== (width / 8) * height) {
    throw new Error("Invalid framebuffer size");
  }
  const rgba = new Uint8ClampedArray(width * height * 4);
  for (let row = 0; row < height; row += 1) {
    for (let byte = 0; byte < width / 8; byte += 1) {
      const value = framebuffer[row * (width / 8) + byte] ?? 0;
      for (let bit = 0; bit < 8; bit += 1) {
        const offset = (row * width + byte * 8 + bit) * 4;
        const ink = value & (0x80 >> bit) ? 220 : 5;
        rgba[offset] = ink;
        rgba[offset + 1] = ink;
        rgba[offset + 2] = ink;
        rgba[offset + 3] = 255;
      }
    }
  }
  return rgba;
}
