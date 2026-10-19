import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "tailwindcss";
import autoprefixer from "autoprefixer";

export default defineConfig({
  plugins: [react()],
  // Tailwind 3 (same major as the Hyperswitch Control Center); tokens in tailwind.config.js
  css: { postcss: { plugins: [tailwindcss(), autoprefixer()] } },
  // The Rust engine runs in the page as WebAssembly (src/engine/wasm), so no /api server is needed.
  // Relative paths, so the built site works from any folder or host path.
  base: "./",
});
