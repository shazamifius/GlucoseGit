import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// Config dédiée Vitest. Séparée du vite.config.ts principal pour éviter de
// trimballer la stack Tauri (server port 1420, manualChunks, etc.) lors des
// tests, et pour pouvoir activer jsdom uniquement ici.
export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
  optimizeDeps: { exclude: ["@automerge/automerge"] },
  test: {
    environment: "jsdom",
    globals: false,
    include: ["src/**/*.test.{ts,tsx}"],
    setupFiles: ["./src/test-setup.ts"],
  },
});
