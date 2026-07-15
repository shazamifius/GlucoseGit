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
    // `mcp/` inclus : la géométrie du pont décide où atterrissent les notes de
    // l'utilisateur. Elle est restée non testée tant qu'elle vivait dans un
    // fichier qui démarre un serveur au chargement — d'où geometry.mjs, pur et
    // importable.
    include: ["src/**/*.test.{ts,tsx}", "mcp/**/*.test.mjs"],
    setupFiles: ["./src/test-setup.ts"],
  },
});
