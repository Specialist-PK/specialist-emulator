import { createEffect } from "solid-js";

import { FRAMEBUFFER_SIZE, framebufferToRgba, SCREEN_HEIGHT, SCREEN_WIDTH } from "./screen";

export function Screen(props: { frame?: Uint8Array }) {
  let canvas: HTMLCanvasElement | undefined;

  createEffect(
    () => props.frame,
    (frame) => {
      if (!canvas || !frame) return;
      const context = canvas.getContext("2d");
      if (!context) return;
      const image = new ImageData(SCREEN_WIDTH, SCREEN_HEIGHT);
      image.data.set(framebufferToRgba(frame));
      context.putImageData(image, 0, 0);
    },
  );

  return (
    <canvas
      ref={(element) => {
        canvas = element;
      }}
      width={SCREEN_WIDTH}
      height={SCREEN_HEIGHT}
      data-framebuffer-size={FRAMEBUFFER_SIZE}
      class="aspect-[3/2] w-full rounded border border-ink/20 bg-black [image-rendering:pixelated]"
      aria-label="Экран Специалиста"
    />
  );
}
