// ────────────────────────────────────────────────────────────────────────────
// R-EMB-01 — Tests de la migration `src` → `AssetRef`.
// ────────────────────────────────────────────────────────────────────────────

import { describe, expect, it } from "vitest";
import type { Project, BoardImage } from "../types";
import {
  migrateProjectAssets,
  srcToAssetRef,
  type AssetBytesFetcher,
} from "./projectMigration";

// ── Helpers de fixture ─────────────────────────────────────────────────────
function mkImage(overrides: Partial<BoardImage> = {}): BoardImage {
  return {
    id: "img-1",
    src: "data:image/png;base64,iVBORw0KGgo=",
    x: 0, y: 0, width: 100, height: 100,
    rotation: 0, locked: false, tags: [],
    originalWidth: 100, originalHeight: 100,
    ...overrides,
  };
}
function mkProject(images: BoardImage[]): Project {
  return {
    version: "2.0.0", name: "test",
    boards: [{
      id: "main", name: "main",
      images,
      annotations: [],
      panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 },
      createdAt: 0, updatedAt: 0,
    }],
    activeBoardId: "main",
    presets: [],
    domains: [],
    createdAt: 0, updatedAt: 0,
  };
}

const stubFetcherFromMap = (map: Record<string, { bytes: Uint8Array; mime: string }>): AssetBytesFetcher =>
  async (filename) => map[filename] ?? null;

// ── Tests srcToAssetRef ───────────────────────────────────────────────────
describe("srcToAssetRef", () => {
  it("blob: → null (éphémère, jamais persisté)", async () => {
    const out: Record<string, Uint8Array> = {};
    const ref = await srcToAssetRef("blob:http://localhost/abc", async () => null, out);
    expect(ref).toBeNull();
    expect(out).toEqual({});
  });

  it("data:URL → embed + ajoute bytes dans outBlobs", async () => {
    const out: Record<string, Uint8Array> = {};
    const ref = await srcToAssetRef(
      "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=",
      async () => null,
      out,
    );
    expect(ref?.mode).toBe("embed");
    if (ref?.mode === "embed") {
      expect(ref.mime).toBe("image/png");
      expect(ref.sha256).toMatch(/^[0-9a-f]{64}$/);
      expect(out[ref.sha256]).toBeInstanceOf(Uint8Array);
    }
  });

  it("asset:filename → fetch via callback + embed", async () => {
    const fakeBytes = new Uint8Array([1, 2, 3, 4, 5]);
    const fetcher = stubFetcherFromMap({
      "abc.png": { bytes: fakeBytes, mime: "image/png" },
    });
    const out: Record<string, Uint8Array> = {};
    const ref = await srcToAssetRef("asset:abc.png", fetcher, out);
    expect(ref?.mode).toBe("embed");
    if (ref?.mode === "embed") {
      expect(ref.mime).toBe("image/png");
      expect(out[ref.sha256]).toBe(fakeBytes);
    }
  });

  it("asset:filename introuvable → throw", async () => {
    const out: Record<string, Uint8Array> = {};
    await expect(srcToAssetRef("asset:gone.png", async () => null, out)).rejects.toThrow(/introuvable/);
  });

  it("http(s)://URL → link, pas d'embed", async () => {
    const out: Record<string, Uint8Array> = {};
    const ref = await srcToAssetRef("https://x.com/a.png", async () => null, out);
    expect(ref?.mode).toBe("link");
    if (ref?.mode === "link") expect(ref.href).toBe("https://x.com/a.png");
    expect(out).toEqual({});
  });

  it("asset:// (URL Tauri canonicalisée) → link", async () => {
    const out: Record<string, Uint8Array> = {};
    const ref = await srcToAssetRef("asset://localhost/x.mp4", async () => null, out);
    expect(ref?.mode).toBe("link");
  });
});

// ── Tests migrateProjectAssets ────────────────────────────────────────────
describe("migrateProjectAssets", () => {
  const noopFetcher: AssetBytesFetcher = async () => null;

  it("projet vide → stats à zéro", async () => {
    const project = mkProject([]);
    const result = await migrateProjectAssets(project, noopFetcher);
    expect(result.migrated).toBe(0);
    expect(result.skipped).toBe(0);
    expect(result.failed).toBe(0);
    expect(result.blobsAdded).toBe(0);
  });

  it("image data:URL → migrée en embed, blobs peuplé", async () => {
    const project = mkProject([mkImage({
      id: "i1",
      src: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=",
    })]);
    const result = await migrateProjectAssets(project, noopFetcher);
    expect(result.migrated).toBe(1);
    expect(result.blobsAdded).toBe(1);
    const img0 = result.project.boards[0].images[0];
    expect(img0.asset?.mode).toBe("embed");
    if (img0.asset?.mode === "embed") {
      expect(result.project.blobs?.[img0.asset.sha256]).toBeInstanceOf(Uint8Array);
    }
    // L'ancien `src` est PRESERVÉ pour compat (nettoyage ultérieur)
    expect(img0.src).toBeTruthy();
  });

  it("plusieurs images même contenu → dédup (1 blob, 2 refs)", async () => {
    const sameDataUrl = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=";
    const project = mkProject([
      mkImage({ id: "i1", src: sameDataUrl }),
      mkImage({ id: "i2", src: sameDataUrl }),
    ]);
    const result = await migrateProjectAssets(project, noopFetcher);
    expect(result.migrated).toBe(2);
    expect(result.blobsAdded).toBe(1); // ← dédup
    const i1 = result.project.boards[0].images[0];
    const i2 = result.project.boards[0].images[1];
    if (i1.asset?.mode === "embed" && i2.asset?.mode === "embed") {
      expect(i1.asset.sha256).toBe(i2.asset.sha256);
    }
  });

  it("asset:filename → fetch + embed", async () => {
    const bytes = new Uint8Array([42, 43, 44, 45, 46, 47, 48, 49]);
    const fetcher = stubFetcherFromMap({
      "snap.png": { bytes, mime: "image/png" },
    });
    const project = mkProject([mkImage({ id: "i1", src: "asset:snap.png" })]);
    const result = await migrateProjectAssets(project, fetcher);
    expect(result.migrated).toBe(1);
    expect(result.blobsAdded).toBe(1);
    const img0 = result.project.boards[0].images[0];
    expect(img0.asset?.mode).toBe("embed");
    if (img0.asset?.mode === "embed") {
      expect(img0.asset.mime).toBe("image/png");
      expect(result.project.blobs?.[img0.asset.sha256]).toBe(bytes);
    }
  });

  it("http(s)://URL → link (pas de blobs)", async () => {
    const project = mkProject([mkImage({ id: "i1", src: "https://example.com/a.png" })]);
    const result = await migrateProjectAssets(project, noopFetcher);
    expect(result.migrated).toBe(1);
    expect(result.blobsAdded).toBe(0);
    const img0 = result.project.boards[0].images[0];
    expect(img0.asset?.mode).toBe("link");
    if (img0.asset?.mode === "link") expect(img0.asset.href).toBe("https://example.com/a.png");
  });

  it("idempotence : 2e passe ne re-migre rien", async () => {
    const project = mkProject([mkImage({
      id: "i1",
      src: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=",
    })]);
    const r1 = await migrateProjectAssets(project, noopFetcher);
    expect(r1.migrated).toBe(1);
    const r2 = await migrateProjectAssets(r1.project, noopFetcher);
    expect(r2.migrated).toBe(0);
    expect(r2.skipped).toBe(1);
    expect(r2.blobsAdded).toBe(0);
  });

  it("blob: éphémère → skip (pas d'erreur ni de migration)", async () => {
    const project = mkProject([mkImage({ id: "i1", src: "blob:http://localhost/abc" })]);
    const result = await migrateProjectAssets(project, noopFetcher);
    expect(result.skipped).toBe(1);
    expect(result.migrated).toBe(0);
    expect(result.failed).toBe(0);
  });

  it("asset:filename introuvable → failed, image inchangée", async () => {
    const project = mkProject([mkImage({ id: "i1", src: "asset:vanished.png" })]);
    const result = await migrateProjectAssets(project, noopFetcher);
    expect(result.failed).toBe(1);
    expect(result.migrated).toBe(0);
    expect(result.project.boards[0].images[0].asset).toBeUndefined();
  });

  it("mix de tous les cas → comptage correct", async () => {
    const bytes = new Uint8Array([10, 20, 30, 40]);
    const fetcher = stubFetcherFromMap({ "ok.png": { bytes, mime: "image/png" } });
    const project = mkProject([
      mkImage({ id: "i1", src: "data:image/png;base64,iVBORw0KGgo=" }),
      mkImage({ id: "i2", src: "https://x.com/y.png" }),
      mkImage({ id: "i3", src: "asset:ok.png" }),
      mkImage({ id: "i4", src: "asset:missing.png" }),
      mkImage({ id: "i5", src: "blob:http://localhost/xyz" }),
    ]);
    const result = await migrateProjectAssets(project, fetcher);
    // i1 + i2 + i3 = migrés ; i4 = failed ; i5 = skipped
    expect(result.migrated).toBe(3);
    expect(result.failed).toBe(1);
    expect(result.skipped).toBe(1);
  });
});
