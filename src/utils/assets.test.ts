// ────────────────────────────────────────────────────────────────────────────
// Routage de resolveAssetSrc — verrouille notamment la régression `data:` (une
// URL data: n'a pas de `//` mais ne doit JAMAIS être traitée comme un chemin de
// fichier, sinon les images embarquées legacy cassent).
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === "get_assets_dir") return "C:/assets";
    throw new Error(`invoke non mocké: ${cmd}`);
  }),
  convertFileSrc: vi.fn((p: string) => `asset-src://${p}`),
}));

import { resolveAssetSrc, _resetAssetsDirCache } from "./assets";
import { setCurrentPath } from "./currentPath";

beforeEach(() => {
  _resetAssetsDirCache();
  setCurrentPath(null);
});

describe("resolveAssetSrc — routage des sources", () => {
  it("RÉGRESSION — data: renvoyée telle quelle (pas traitée comme un chemin)", async () => {
    const d = "data:image/png;base64,AAAABBBBCCCC";
    expect(await resolveAssetSrc(d)).toBe(d);
  });

  it("http(s):// renvoyé tel quel", async () => {
    expect(await resolveAssetSrc("https://exemple.com/y.png")).toBe("https://exemple.com/y.png");
    expect(await resolveAssetSrc("http://exemple.com/y.png")).toBe("http://exemple.com/y.png");
  });

  it("asset:<file> → convertFileSrc(<dossier assets>/<file>)", async () => {
    const r = await resolveAssetSrc("asset:abc.jpg");
    expect(r).toBe("asset-src://C:/assets/abc.jpg");
  });

  it("chemin absolu local → convertFileSrc(chemin)", async () => {
    expect(await resolveAssetSrc("C:/photos/x.png")).toBe("asset-src://C:/photos/x.png");
  });

  it("chemin relatif → résolu via le .glucose courant puis convertFileSrc", async () => {
    setCurrentPath("C:/proj/notes.glucose");
    expect(await resolveAssetSrc("images/x.png")).toBe("asset-src://C:/proj/images/x.png");
  });

  it("chaîne vide → renvoyée telle quelle", async () => {
    expect(await resolveAssetSrc("")).toBe("");
  });
});
