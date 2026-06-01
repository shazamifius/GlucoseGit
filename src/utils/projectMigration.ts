// ────────────────────────────────────────────────────────────────────────────
// R-EMB-01 (Sprint 2) — Migration des images legacy `src: string` vers
// `asset: AssetRef`. Idempotente.
//
// Stratégie :
//   - `src: "asset:<filename>"`  → invoque `fetchAssetBytes` (Tauri en prod,
//                                   stub en tests) → embed (Project.blobs).
//   - `src: "data:..."`          → décode → embed.
//   - `src: "http(s)://..."`     → link (href tel quel).
//   - `src: "blob:..."`          → skip (éphémère navigateur, ne doit JAMAIS
//                                   être enregistré dans le projet).
//   - `src: "asset://..."`       → link (URL Tauri canonicalisée pour vidéos).
//   - Si l'image a déjà `asset`  → skip (re-run = no-op).
//
// La migration n'efface PAS `src` immédiatement : on garde le champ legacy
// pour permettre un rollback / inspection visuelle pendant la fenêtre de
// transition. Une passe de cleanup ultérieure (R-HYG futur) le strippera.
// ────────────────────────────────────────────────────────────────────────────

import type { Project, BoardImage, AssetRef } from "../types";
import {
  buildEmbedRef,
  buildLinkRef,
  dataUrlToBytes,
  mimeFromExt,
} from "./assetRef";

/** Source de bytes pour les références `asset:<filename>` (managed asset
 *  dir). Retourne `null` si introuvable / interdit. */
export type AssetBytesFetcher = (filename: string) => Promise<{
  bytes: Uint8Array;
  mime: string;
} | null>;

/** Résultat de migration : projet patché + stats. */
export interface MigrationResult {
  project: Project;
  migrated: number;
  skipped: number;
  failed: number;
  blobsAdded: number;
}

/**
 * Migre toutes les images d'un projet du modèle `src: string` vers le modèle
 * `asset: AssetRef`. Idempotent.
 *
 * @param project — projet en entrée (non muté)
 * @param fetchAssetBytes — résolveur Tauri pour les `asset:<filename>` ;
 *                          en tests, on passe un stub.
 * @returns nouveau projet + stats
 */
export async function migrateProjectAssets(
  project: Project,
  fetchAssetBytes: AssetBytesFetcher,
): Promise<MigrationResult> {
  let migrated = 0;
  let skipped = 0;
  let failed = 0;

  // On accumule les nouveaux blobs dans une map locale puis on merge à la fin
  // pour ne pas faire muter project.blobs (immutabilité).
  const newBlobs: Record<string, Uint8Array> = {};

  const migrateImage = async (img: BoardImage): Promise<BoardImage> => {
    // Skip si déjà migré
    if (img.asset) { skipped++; return img; }
    const src = img.src;
    if (!src) { skipped++; return img; }

    try {
      const ref = await srcToAssetRef(src, fetchAssetBytes, newBlobs);
      if (!ref) { skipped++; return img; }
      migrated++;
      return { ...img, asset: ref };
    } catch (e) {
      console.warn(`[migrateProjectAssets] échec pour image ${img.id} (src=${src.slice(0, 50)}...) :`, e);
      failed++;
      return img;
    }
  };

  // Parcours profond immutable
  const newBoards = await Promise.all(
    project.boards.map(async (b) => {
      const newImages = await Promise.all(b.images.map(migrateImage));
      return { ...b, images: newImages };
    })
  );

  const mergedBlobs = { ...(project.blobs ?? {}), ...newBlobs };
  const blobsAdded = Object.keys(newBlobs).length;

  return {
    project: {
      ...project,
      boards: newBoards,
      blobs: mergedBlobs,
    },
    migrated,
    skipped,
    failed,
    blobsAdded,
  };
}

/**
 * Convertit une valeur `src` legacy en `AssetRef`. Si `asset:<filename>`,
 * lit les bytes via `fetchAssetBytes` et ajoute à `outBlobs`.
 *
 * Renvoie `null` si la src ne représente pas un asset (ex: blob: éphémère).
 *
 * Exporté pour la testabilité unitaire.
 */
export async function srcToAssetRef(
  src: string,
  fetchAssetBytes: AssetBytesFetcher,
  outBlobs: Record<string, Uint8Array>,
): Promise<AssetRef | null> {
  // 1. blob:URL → éphémère, on ne migre pas
  if (src.startsWith("blob:")) return null;

  // 2. data:URL → décode + embed
  if (src.startsWith("data:")) {
    const { bytes, mime } = dataUrlToBytes(src);
    const ref = await buildEmbedRef(bytes, mime);
    outBlobs[ref.sha256] = bytes;
    return ref;
  }

  // 3. asset:<filename> (managed) → Tauri load + embed
  if (src.startsWith("asset:") && !src.startsWith("asset://")) {
    const filename = src.slice("asset:".length);
    const fetched = await fetchAssetBytes(filename);
    if (!fetched) {
      throw new Error(`asset introuvable côté disque : ${filename}`);
    }
    // Si fetched.mime est vide, on essaie de déduire de l'extension
    const ext = filename.split(".").pop()?.toLowerCase() ?? "";
    const mime = fetched.mime || mimeFromExt(ext);
    const ref = await buildEmbedRef(fetched.bytes, mime);
    outBlobs[ref.sha256] = fetched.bytes;
    return ref;
  }

  // 4. http(s)://, asset:// (URL Tauri canonicalisée) → link
  if (src.startsWith("http://") || src.startsWith("https://") || src.startsWith("asset://")) {
    return buildLinkRef(src);
  }

  // 5. Inconnu → link best-effort (chemin absolu legacy ?)
  return buildLinkRef(src);
}
