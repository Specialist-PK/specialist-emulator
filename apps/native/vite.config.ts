import solid from "@solidjs/vite-plugin";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [solid(), tailwindcss()],
  server: { port: 1420, strictPort: true },
  clearScreen: false,
});
