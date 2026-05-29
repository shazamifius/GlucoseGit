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
  // Toute autre forme : laisser tel quel
  return src;
}

/** Indique si un `src` est encore en base64 (legacy à migrer). */
export function isLegacyDataUrl(src: string | undefined): boolean {
  return !!src && src.startsWith("data:");
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
