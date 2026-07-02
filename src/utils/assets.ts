// ────────────────────────────────────────────────────────────────────────────
// Phase 7.0 — Externalisation des assets (CRDT-friendly).
// ────────────────────────────────────────────────────────────────────────────
//
// Les images sont désormais stockées dans `app_data_dir/assets/<hash>.<ext>`
// (côté disque) et référencées par un identifiant logique côté projet :
//   `asset:abc123.png`
//
// Trois familles de `src` cohabitent dans un projet :
//   1. `asset:<filename>` → asset géré, résolu via `convertFileSrc(assetsDir + name)`
//   2. `data:image/...`   → legacy ; ne devrait plus exister après load (migré)
//                          mais possible en cours d'import async (1 frame max)
//   3. `http(s)://`       → image web directe (rare, fallback)
//   4. `asset://...`      → URL Tauri canonicalisée (vidéos via convertFileSrc)
//
// Le projet sérialisé ne contient JAMAIS de chemin absolu : c'est ce qui le
// rend portable d'une machine à l'autre (cf. CLEANUP B-04, audit C-1).

import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { AssetRef, Project } from "../types";
import { resolveAssetRefSync, extFromMime, buildLinkRef } from "./assetRef";
import { toAbsolute, isAbsolutePath } from "./pathResolver";

let assetsDirCache: string | null = null;

/**
 * Récupère le dossier assets en absolute path (cache mémoire).
 * Utilisé une fois au boot, puis réutilisé par toutes les résolutions.
 */
export async function getAssetsDir(): Promise<string> {
  if (assetsDirCache) return assetsDirCache;
  const dir = await invoke<string>("get_assets_dir");
  assetsDirCache = dir;
  return dir;
}

/** Réinitialise le cache (tests). */
export function _resetAssetsDirCache(): void {
  assetsDirCache = null;
}

/**
 * Sauvegarde un base64 (data URL ou base64 brut) dans le dossier assets via le
 * backend Rust. Renvoie l'identifiant logique `asset:<filename>`.
 *
 * @param base64Data - data URL complète ou base64 brut
 * @param extHint - extension à utiliser si non déductible du data URL
 */
export async function saveAsset(base64Data: string, extHint = "png"): Promise<string> {
  const filename = await invoke<string>("save_asset", { base64Data, extHint });
  return `asset:${filename}`;
}

/**
 * B-STORE — Persiste des octets bruts (image décodée) dans le store
 * content-addressed sur disque et renvoie l'identifiant logique `asset:<file>`.
 *
 * C'est la primitive « du dur » : au lieu d'embarquer les bytes dans le doc
 * Automerge (`project.blobs`, ce qui le gonfle et fait freezer `A.save`), on
 * écrit l'image une seule fois sur disque (dédup par hash côté Rust) et on ne
 * garde dans le doc qu'une référence `link`.
 */
export async function saveAssetFromBytes(bytes: Uint8Array, extHint = "png"): Promise<string> {
  // base64 par chunks pour éviter "Maximum call stack size" sur les gros buffers.
  const CHUNK = 32_768;
  let binary = "";
  for (let i = 0; i < bytes.length; i += CHUNK) {
    binary += String.fromCharCode.apply(null, Array.from(bytes.subarray(i, i + CHUNK)));
  }
  const b64 = btoa(binary);
  return saveAsset(b64, extHint);
}

/**
 * Convertit un `src` stocké dans le projet en URL utilisable par PixiJS / `<img>`.
 *
 * - `asset:<name>` → `convertFileSrc(<assetsDir>/<name>)`
 * - `data:...`     → renvoyé tel quel
 * - `http(s)://`   → renvoyé tel quel
 * - `asset://`     → renvoyé tel quel (déjà résolu Tauri pour vidéos)
 * - autre          → renvoyé tel quel (chemin absolu legacy ; warning console)
 */
export async function resolveAssetSrc(src: string): Promise<string> {
  if (!src) return src;
  if (src.startsWith("asset:")) {
    // Forme `asset:filename.png` → résoudre via le dossier assets
    const filename = src.slice("asset:".length);
    const dir = await getAssetsDir();
    // Construit un chemin absolu portable (séparateur natif)
    const sep = dir.includes("\\") ? "\\" : "/";
    return convertFileSrc(`${dir}${sep}${filename}`);
  }
  
  // Si c'est un chemin local (relatif, absolu ou file://). ⚠️ On exclut TOUTE URL
  // à schéma (`data:`, `http:`, `mailto:`…) : `data:` n'a pas de `//` mais ne doit
  // PAS être traité comme un chemin (sinon les images embarquées legacy cassent).
  // Un lecteur Windows `C:/…` a un « schéma » d'une lettre mais est capté avant par
  // isAbsolutePath, donc l'ordre du OR le préserve.
  const hasUriScheme = /^[a-z][a-z0-9+.-]*:/i.test(src);
  const isLocal = src.startsWith("file://") || isAbsolutePath(src) || !hasUriScheme;
  if (isLocal) {
    const abs = toAbsolute(src);
    const cleanAbs = abs.startsWith("file://") ? abs.slice(7) : abs;
    return convertFileSrc(cleanAbs);
  }

  // Toute autre forme (http/https/data) : laisser tel quel
  return src;
}

/**
 * R-EMB-01 (Sprint 2) — Résolveur unifié image → URL renderable.
 *
 * Stratégie :
 *   1. Si `asset` (AssetRef) défini → résolveur dédié
 *      - mode "embed" → blob URL depuis project.blobs[sha256]
 *      - mode "link"  → href tel quel (ou convertFileSrc si chemin local)
 *   2. Sinon `src` legacy (string) → resolveAssetSrc
 *   3. Sinon → chaîne vide
 *
 * Utilisé par PixiJS sprite loading, organize panel, etc.
 */
export async function resolveImageSrc(
  asset: AssetRef | undefined,
  src: string | undefined,
  blobs: Record<string, Uint8Array> | undefined,
): Promise<string> {
  if (asset) {
    if (asset.mode === "embed") {
      // Blob URL → utilisable directement par Pixi.Assets.load(url) et <img>
      return resolveAssetRefSync(asset, blobs);
    }
    // asset.mode === "link" — résout via la chaîne usuelle (asset:/data:/http)
    return resolveAssetSrc(asset.href);
  }
  // Fallback legacy
  if (src) return resolveAssetSrc(src);
  return "";
}

/** Indique si un `src` est encore en base64 (legacy à migrer). */
export function isLegacyDataUrl(src: string | undefined): boolean {
  return !!src && src.startsWith("data:");
}

/** Normalise un blob lu depuis le doc (peut être un proxy / objet indexé) en
 *  vrai Uint8Array. */
function toU8(raw: unknown): Uint8Array {
  if (raw instanceof Uint8Array) return raw;
  if (ArrayBuffer.isView(raw)) {
    const v = raw as ArrayBufferView;
    return new Uint8Array(v.buffer, v.byteOffset, v.byteLength);
  }
  const obj = raw as Record<number, number> & { length?: number };
  const len = obj?.length ?? Object.keys(obj ?? {}).length;
  const u8 = new Uint8Array(len);
  for (let i = 0; i < len; i++) u8[i] = obj[i] ?? 0;
  return u8;
}

/**
 * B-STORE — Migration « du dur ». Sort toutes les images embarquées
 * (`asset.mode === "embed"`, bytes dans `project.blobs`) vers le store disque
 * content-addressed, convertit leurs refs en `link`, puis supprime `project.blobs`.
 *
 * Effet : le document Automerge ne contient plus aucun octet d'image → `A.save`
 * redevient minuscule (fin du freeze 4-5 s). Idempotent : un projet déjà « du
 * dur » (pas de blobs) est renvoyé inchangé.
 *
 * À appeler au `loadProject` (comme les autres migrations). Le caller force
 * `doc = undefined` pour repartir d'un doc neuf sans l'historique gonflé.
 */
export async function externalizeEmbeddedBlobs(
  project: Project,
): Promise<{ project: Project; externalized: number; failed: number }> {
  const blobs = project.blobs;
  if (!blobs || Object.keys(blobs).length === 0) {
    return { project, externalized: 0, failed: 0 };
  }

  let externalized = 0;
  let failed = 0;

  const newBoards = await Promise.all(
    project.boards.map(async (b) => {
      const newImages = await Promise.all(
        b.images.map(async (img) => {
          if (img.asset?.mode !== "embed") return img;
          const sha = img.asset.sha256;
          const raw = blobs[sha];
          if (!raw) { failed++; return img; } // blob manquant → on laisse tel quel
          try {
            const ext = extFromMime(img.asset.mime);
            const assetId = await saveAssetFromBytes(toU8(raw), ext);
            externalized++;
            return {
              ...img,
              asset: buildLinkRef(assetId, { sha256: sha, sizeBytes: img.asset.sizeBytes }),
            };
          } catch (e) {
            console.warn("[externalizeEmbeddedBlobs] échec", sha.slice(0, 12), e);
            failed++;
            return img;
          }
        }),
      );
      return { ...b, images: newImages };
    }),
  );

  // On supprime entièrement les blobs : le doc ne porte plus aucun octet.
  const next: Project = { ...project, boards: newBoards };
  delete next.blobs;
  return { project: next, externalized, failed };
}

/**
 * Migration `.glucose` legacy : parcourt toutes les images et externalise les
 * `data:image/...` inline vers le dossier assets. Renvoie le projet patché.
 *
 * Idempotent : si tout est déjà externalisé, retourne le projet inchangé.
 * Tolère les échecs individuels (l'image legacy reste en data: si externalisation
 * échoue — ce qui ne casse pas le rendu, juste ne corrige pas le bloat).
 */
export async function migrateLegacyAssets<P extends { boards: { images: { src?: string }[] }[] }>(
  project: P
): Promise<{ project: P; migrated: number; failed: number }> {
  let migrated = 0;
  let failed = 0;

  // Détection rapide : y a-t-il au moins un data: ? Sinon on ne touche rien.
  let hasLegacy = false;
  for (const b of project.boards) {
    for (const img of b.images) {
      if (img.src && isLegacyDataUrl(img.src)) { hasLegacy = true; break; }
    }
    if (hasLegacy) break;
  }
  if (!hasLegacy) return { project, migrated: 0, failed: 0 };

  // Parcours profond + remplacement immutable
  const newBoards = await Promise.all(
    project.boards.map(async (b) => {
      const newImages = await Promise.all(
        b.images.map(async (img) => {
          if (!img.src || !isLegacyDataUrl(img.src)) return img;
          try {
            const ext = guessExtFromDataUrl(img.src);
            const assetSrc = await saveAsset(img.src, ext);
            migrated++;
            return { ...img, src: assetSrc };
          } catch (e) {
            console.warn("Migration legacy asset échec :", e);
            failed++;
            return img;
          }
        })
      );
      return { ...b, images: newImages };
    })
  );

  return {
    project: { ...project, boards: newBoards },
    migrated,
    failed,
  };
}

function guessExtFromDataUrl(dataUrl: string): string {
  const m = dataUrl.match(/^data:image\/([a-z0-9+]+)[;,]/i);
  if (!m) return "png";
  const sub = m[1].toLowerCase();
  if (sub === "jpeg") return "jpg";
  if (sub === "svg+xml") return "svg";
  return sub;
}
