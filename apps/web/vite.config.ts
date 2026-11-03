import solid from "@solidjs/vite-plugin";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  base: process.env["BASE_PATH"] ?? "/",
  plugins: [solid(), tailwindcss()],
});
