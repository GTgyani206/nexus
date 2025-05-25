// vortex-lang-wasm/vite.config.js
import { defineConfig } from "vite";

export default defineConfig({
  root: "www",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    target: "esnext",
  },
});
