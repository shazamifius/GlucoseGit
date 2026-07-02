// ────────────────────────────────────────────────────────────────────────────
// Bundle portable — cœur PUR (énumération des assets, manifeste, intégrité).
// Aucune I/O : on teste la logique qui décide QUOI embarquer.
// ────────────────────────────────────────────────────────────────────────────

import { describe, expect, it } from "vitest";
import type { BoardImage, Project } from "../types";
import { sha256Hex } from "./assetRef";
import {
  assetBytesMatch,
  buildBundleManifest,
  BUNDLE_FORMAT,
  BUNDLE_VERSION,
  collectReferencedAssets,
} from "./bundle";

function mkProject(boardsImages: BoardImage[][]): Project {
  return {
    version: "2.0.0",
    name: "mon projet",
    boards: boardsImages.map((imgs, i) => ({
      id: `b${i}`, name: `B${i}`, images: imgs, annotations: [], panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0,
    })),
    activeBoardId: "b0", presets: [], domains: [], createdAt: 0, updatedAt: 0,
  };
}

function baseImg(id: string): BoardImage {
  return {
    id, x: 0, y: 0, width: 10, height: 10, rotation: 0, locked: false,
    tags: [], originalWidth: 10, originalHeight: 10,
  };
}

function linkImg(id: string, name: string, sha256?: string, sizeBytes?: number): BoardImage {
  return {
    ...baseImg(id),
    asset: { mode: "link", href: `asset:${name}`, ...(sha256 ? { sha256 } : {}), ...(sizeBytes !== undefined ? { sizeBytes } : {}) },
  };
}

describe("collectReferencedAssets — cœur pur", () => {
  it("collecte les refs asset: (mode link) avec sha256 + taille", () => {
    const p = mkProject([[linkImg("i1", "aaa0000011112222.png", "aaa0000011112222" + "0".repeat(48), 1234)]]);
    const assets = collectReferencedAssets(p);
    expect(assets).toEqual([
      { name: "aaa0000011112222.png", sha256: "aaa0000011112222" + "0".repeat(48), sizeBytes: 1234 },
    ]);
  });

  it("collecte aussi le champ legacy `src: asset:...`", () => {
    const img: BoardImage = { ...baseImg("i2"), src: "asset:bbb1.jpg" };
    const assets = collectReferencedAssets(mkProject([[img]]));
    expect(assets.map((a) => a.name)).toEqual(["bbb1.jpg"]);
  });

  it("déduplique un même asset posé sur plusieurs boards/images", () => {
    const p = mkProject([
      [linkImg("i1", "dup.png"), linkImg("i2", "dup.png")],
      [linkImg("i3", "dup.png")],
    ]);
    expect(collectReferencedAssets(p)).toHaveLength(1);
  });

  it("ignore les embed (déjà dans le doc) et les liens non-asset (http, fichier)", () => {
    const embed: BoardImage = { ...baseImg("e1"), asset: { mode: "embed", sha256: "deadbeef", mime: "image/png" } };
    const web: BoardImage = { ...baseImg("w1"), asset: { mode: "link", href: "https://site/x.png" } };
    const file: BoardImage = { ...baseImg("f1"), asset: { mode: "link", href: "C:/Users/x/photo.png" } };
    const keep = linkImg("k1", "keep.png");
    const assets = collectReferencedAssets(mkProject([[embed, web, file, keep]]));
    expect(assets.map((a) => a.name)).toEqual(["keep.png"]);
  });

  it("renvoie un ordre déterministe (trié par nom)", () => {
    const p = mkProject([[linkImg("i1", "zzz.png"), linkImg("i2", "aaa.png"), linkImg("i3", "mmm.png")]]);
    expect(collectReferencedAssets(p).map((a) => a.name)).toEqual(["aaa.png", "mmm.png", "zzz.png"]);
  });

  it("projet sans image → liste vide (pas de crash sur boards/images absents)", () => {
    expect(collectReferencedAssets(mkProject([[]]))).toEqual([]);
    expect(collectReferencedAssets({ ...mkProject([]), boards: undefined as unknown as Project["boards"] })).toEqual([]);
  });
});

describe("buildBundleManifest — cœur pur", () => {
  it("produit un manifeste au bon format/version, portant le nom + les assets", () => {
    const p = mkProject([[linkImg("i1", "a.png")]]);
    const assets = collectReferencedAssets(p);
    const m = buildBundleManifest(p, assets);
    expect(m.format).toBe(BUNDLE_FORMAT);
    expect(m.version).toBe(BUNDLE_VERSION);
    expect(m.name).toBe("mon projet");
    expect(m.doc).toBe("project.glucose");
    expect(m.assets).toEqual(assets);
    expect(typeof m.createdAt).toBe("number");
  });
});

describe("assetBytesMatch — intégrité content-addressed", () => {
  it("vrai quand le nom encode bien le hash du contenu", async () => {
    const bytes = new Uint8Array([1, 2, 3, 4, 5]);
    const full = await sha256Hex(bytes);
    const name = `${full.slice(0, 16)}.png`;
    expect(await assetBytesMatch(name, bytes)).toBe(true);
    expect(await assetBytesMatch(name, bytes, full)).toBe(true);
  });

  it("faux si le stem du nom ne correspond pas au contenu", async () => {
    const bytes = new Uint8Array([9, 9, 9]);
    expect(await assetBytesMatch("ffff000011112222.png", bytes)).toBe(false);
  });

  it("faux si le sha256 complet attendu diffère", async () => {
    const bytes = new Uint8Array([1, 2, 3, 4, 5]);
    const full = await sha256Hex(bytes);
    const name = `${full.slice(0, 16)}.png`;
    expect(await assetBytesMatch(name, bytes, "0".repeat(64))).toBe(false);
  });
});
