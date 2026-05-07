import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
// Phase 7 — Automerge expose un module WASM en ESM, qui nécessite ces plugins
// Vite pour être correctement bundlé.
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react(), tailwindcss(), wasm(), topLevelAwait()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  // Phase 7 — Automerge utilise WebAssembly. Vite doit savoir l'inclure
  // dans le bundle final (pas tenter de le tree-shake comme du JS).
  optimizeDeps: {
    exclude: ["@automerge/automerge"],
  },
  // CLEANUP B-01 — éclate le bundle JS en chunks parallèles (gain démarrage,
  // meilleur cache HTTP côté Tauri local). Ciblé sur les libs lourdes.
  build: {
    target: "esnext",
    chunkSizeWarningLimit: 800,
    rollupOptions: {
      output: {
        manualChunks: {
          "vendor-react": ["react", "react-dom"],
          "vendor-pixi": ["pixi.js", "@pixi/react"],
          "vendor-markdown": [
            "react-markdown",
            "remark-gfm",
            "remark-math",
            "remark-breaks",
            "rehype-katex",
          ],
          "vendor-katex": ["katex"],
          "vendor-automerge": ["@automerge/automerge"],
        },
      },
    },
  },
}));
