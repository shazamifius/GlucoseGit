// ────────────────────────────────────────────────────────────────────────────
// R-EMB-01 — Test de roundtrip end-to-end (le scénario utilisateur).
//
// Scenario réel :
//   1. Projet legacy avec images `src: "asset:<file>"` (modèle Phase 7.0).
//   2. Migration → embed dans project.blobs.
//   3. A.create depuis le projet migré → Automerge doc.
//   4. A.save → binaire portable (le fichier .glucose self-contained).
//   5. A.load(binaire) sur une NOUVELLE machine (sans le dossier assets/).
//   6. resolveImageSrc → blob URL renderable.
//
// Si tous ces étages tiennent dans un test isolé, on a la garantie que
// l'utilisateur peut transférer son .glucose vers un autre poste.
// ────────────────────────────────────────────────────────────────────────────

import { describe, expect, it } from "vitest";
import type { Project } from "../types";
import * as A from "../store/automerge";
import { migrateProjectAssets, type AssetBytesFetcher } from "./projectMigration";
import { resolveImageSrc } from "./assets";

function mkBytes(seed: number, len: number): Uint8Array {
  const arr = new Uint8Array(len);
  crypto.getRandomValues(arr);
  arr[0] = seed; // marqueur de seed pour debug
  return arr;
}

describe("R-EMB-01 end-to-end : projet legacy → embed → save → load → render", () => {
  it("scenario complet utilisateur (asset:<file> sur disque devient embed self-contained)", async () => {
    // ── ÉTAPE 1 : projet legacy avec images en mode `asset:<file>` ───────
    const bytes1 = mkBytes(0x11, 1024);
    const bytes2 = mkBytes(0x22, 2048);
    const legacyProject: Project = {
      version: "2.0.0", name: "user-project",
      boards: [{
        id: "b1", name: "main",
        images: [
          {
            id: "img-1", src: "asset:photo-a.png",
            x: 100, y: 100, width: 400, height: 300,
            rotation: 0, locked: false, tags: [],
            originalWidth: 400, originalHeight: 300,
          },
          {
            id: "img-2", src: "asset:photo-b.jpg",
            x: 600, y: 100, width: 320, height: 240,
            rotation: 0, locked: false, tags: [],
            originalWidth: 320, originalHeight: 240,
          },
        ],
        annotations: [],
        panels: [], zones: [], folders: [],
        viewport: { x: 0, y: 0, scale: 1 },
        createdAt: 0, updatedAt: 0,
      }],
      activeBoardId: "b1",
      presets: [],
      domains: [],
      createdAt: 0, updatedAt: 0,
    };

    // Fetcher qui simule le dossier assets/ sur disque (étape "ancien poste")
    const oldDiskFetcher: AssetBytesFetcher = async (filename) => {
      if (filename === "photo-a.png") return { bytes: bytes1, mime: "image/png" };
      if (filename === "photo-b.jpg") return { bytes: bytes2, mime: "image/jpeg" };
      return null;
    };

    // ── ÉTAPE 2 : migration ────────────────────────────────────────────
    const migrated = await migrateProjectAssets(legacyProject, oldDiskFetcher);
    expect(migrated.migrated).toBe(2);
    expect(migrated.blobsAdded).toBe(2);
    expect(migrated.failed).toBe(0);
    expect(migrated.project.blobs).toBeDefined();
    expect(Object.keys(migrated.project.blobs ?? {}).length).toBe(2);

    // ── ÉTAPE 3 : A.create depuis le projet migré ──────────────────────
    const doc = A.create<Project>(migrated.project);
    expect(doc.boards[0].images.length).toBe(2);
    expect(doc.boards[0].images[0].asset).toBeDefined();
    expect(doc.boards[0].images[0].asset?.mode).toBe("embed");

    // ── ÉTAPE 4 : A.save → binaire ─────────────────────────────────────
    const binary = A.save(doc);
    expect(binary.length).toBeGreaterThan(bytes1.length + bytes2.length); // contient les blobs

    // ── ÉTAPE 5 : SIMULATION transfert sur autre machine ───────────────
    // On charge le binaire sans rien d'autre — pas de fetcher disque !
    const loadedDoc = A.load<Project>(binary);
    const loaded = A.asPlain(loadedDoc);

    expect(loaded.boards[0].images.length).toBe(2);
    expect(loaded.blobs).toBeDefined();

    // Les bytes doivent être strictement identiques après le roundtrip binaire
    const img1 = loaded.boards[0].images[0];
    expect(img1.asset?.mode).toBe("embed");
    if (img1.asset?.mode === "embed") {
      const loadedBytes1 = loaded.blobs?.[img1.asset.sha256];
      expect(loadedBytes1).toBeInstanceOf(Uint8Array);
      expect(loadedBytes1?.length).toBe(bytes1.length);
      for (let i = 0; i < bytes1.length; i++) {
        expect(loadedBytes1?.[i]).toBe(bytes1[i]);
      }
    }

    // ── ÉTAPE 6 : resolveImageSrc rend une URL utilisable ──────────────
    const url1 = await resolveImageSrc(img1.asset, img1.src, loaded.blobs);
    expect(url1).toMatch(/^blob:/);

    const img2 = loaded.boards[0].images[1];
    const url2 = await resolveImageSrc(img2.asset, img2.src, loaded.blobs);
    expect(url2).toMatch(/^blob:/);
    // Les 2 URLs doivent être différentes (2 blobs distincts)
    expect(url1).not.toBe(url2);
  });

  it("dédup : 5 images mêmes octets → 1 seul blob dans .glucose", async () => {
    const sharedBytes = mkBytes(0xab, 4096);
    const legacyProject: Project = {
      version: "2.0.0", name: "dedup-test",
      boards: [{
        id: "b1", name: "main",
        images: Array.from({ length: 5 }, (_, i) => ({
          id: `img-${i}`, src: "asset:same.png",
          x: i * 100, y: 0, width: 100, height: 100,
          rotation: 0, locked: false, tags: [],
          originalWidth: 100, originalHeight: 100,
        })),
        annotations: [],
        panels: [], zones: [], folders: [],
        viewport: { x: 0, y: 0, scale: 1 },
        createdAt: 0, updatedAt: 0,
      }],
      activeBoardId: "b1",
      presets: [],
      domains: [],
      createdAt: 0, updatedAt: 0,
    };
    const fetcher: AssetBytesFetcher = async () => ({ bytes: sharedBytes, mime: "image/png" });

    const migrated = await migrateProjectAssets(legacyProject, fetcher);
    expect(migrated.migrated).toBe(5);
    expect(migrated.blobsAdded).toBe(1); // dédup : 1 seul blob malgré 5 refs

    const doc = A.create<Project>(migrated.project);
    const binary = A.save(doc);

    // Taille attendue : ~4 KB blob + structure (pas 5×4 KB = 20 KB)
    expect(binary.length).toBeLessThan(sharedBytes.length * 2);
  });

  it("mix embed + link : seuls les embeds sont dans le binaire", async () => {
    const localBytes = mkBytes(0x33, 512);
    const legacyProject: Project = {
      version: "2.0.0", name: "mix",
      boards: [{
        id: "b1", name: "main",
        images: [
          { id: "embed-1", src: "asset:local.png",
            x: 0, y: 0, width: 100, height: 100,
            rotation: 0, locked: false, tags: [],
            originalWidth: 100, originalHeight: 100 },
          { id: "link-1", src: "https://example.com/remote.png",
            x: 200, y: 0, width: 100, height: 100,
            rotation: 0, locked: false, tags: [],
            originalWidth: 100, originalHeight: 100 },
        ],
        annotations: [],
        panels: [], zones: [], folders: [],
        viewport: { x: 0, y: 0, scale: 1 },
        createdAt: 0, updatedAt: 0,
      }],
      activeBoardId: "b1",
      presets: [],
      domains: [],
      createdAt: 0, updatedAt: 0,
    };
    const fetcher: AssetBytesFetcher = async (f) =>
      f === "local.png" ? { bytes: localBytes, mime: "image/png" } : null;

    const migrated = await migrateProjectAssets(legacyProject, fetcher);
    expect(migrated.migrated).toBe(2);
    expect(migrated.blobsAdded).toBe(1); // seul l'embed ajoute un blob

    const doc = A.create<Project>(migrated.project);
    const reloaded = A.asPlain(A.load<Project>(A.save(doc)));

    expect(reloaded.boards[0].images[0].asset?.mode).toBe("embed");
    expect(reloaded.boards[0].images[1].asset?.mode).toBe("link");
  });
});
