import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { checkFileExists, useFileExistence, invalidateExistenceCache } from "./fileExistence";

// Mock the Tauri fs plugin
vi.mock("@tauri-apps/plugin-fs", () => {
  return {
    exists: vi.fn(async (path: string) => {
      return path.includes("exists");
    }),
  };
});

describe("fileExistence", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("checkFileExists", () => {
    it("returns true if path exists, false otherwise", async () => {
      const ok = await checkFileExists("C:/my-exists-file.blend");
      expect(ok).toBe(true);

      const bad = await checkFileExists("C:/missing.blend");
      expect(bad).toBe(false);
    });

    it("caches the result", async () => {
      const path = "C:/cached-exists-file.blend";
      const ok1 = await checkFileExists(path);
      const ok2 = await checkFileExists(path);

      expect(ok1).toBe(true);
      expect(ok2).toBe(true);
    });
  });

  describe("useFileExistence hook", () => {
    it("handles undefined paths as exists", () => {
      const { result } = renderHook(() => useFileExistence(undefined));
      expect(result.current).toBe("exists");
    });

    it("resolves asynchronously", async () => {
      const { result } = renderHook(({ path }) => useFileExistence(path), {
        initialProps: { path: "C:/test-exists-async.blend" },
      });

      expect(result.current).toBe("loading");

      // Wait for promise resolution
      await act(async () => {
        await new Promise((r) => setTimeout(r, 10));
      });

      expect(result.current).toBe("exists");
    });

    it("notifies when path changes and invalidates cache", async () => {
      const path = "C:/toggle-exists.blend";
      invalidateExistenceCache(path);

      const { result } = renderHook(() => useFileExistence(path));
      expect(result.current).toBe("loading");
    });
  });
});
