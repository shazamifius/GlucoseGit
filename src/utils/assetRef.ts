// ────────────────────────────────────────────────────────────────────────────
// R-EMB-01 (Sprint 2) — Helpers pour le modèle d'assets dual embed/link.
//
// Modèle :
//   - `AssetRef { mode: "embed", sha256, mime }` → bytes dans `project.blobs`
//   - `AssetRef { mode: "link",  href, sha256? }` → URL/chemin externe
//
// Ce module fournit les primitives pures :
//   - sha256Hex(bytes)                  → hash hex stable
//   - dataUrlToBytes(dataUrl)           → décode un data URL en Uint8Array
//   - mimeFromExt(ext)                  → table MIME → extension
//   - extFromMime(mime)                 → inverse
//   - buildEmbedRef(bytes, mime?)       → AssetRef mode embed (calcule sha)
//   - buildLinkRef(href, sha256?)       → AssetRef mode link
//   - resolveAssetRefSync(asset, blobs) → URL renderable
//      • embed → blob URL créé à la volée (cache LRU 256 entrées)
//      • link  → href tel quel (le caller fait convertFileSrc si chemin)
//   - releaseAllBlobUrls()              → libère le cache (cleanup tests)
//
// Pas de dépendance React / Tauri ici : tout est utilisable côté tests Node.
// ────────────────────────────────────────────────────────────────────────────

import type { AssetRef } from "../types";

/** Hash SHA-256 hex (64 chars) d'un Uint8Array. Utilise Web Crypto. */
export async function sha256Hex(data: Uint8Array): Promise<string> {
  const buf = await crypto.subtle.digest("SHA-256", data);
  const arr = new Uint8Array(buf);
  let out = "";
  for (let i = 0; i < arr.length; i++) {
    out += arr[i].toString(16).padStart(2, "0");
  }
  return out;
}

/** Décode un data URL `data:<mime>;base64,<payload>` en `{ bytes, mime }`.
 *  Lève si le format n'est pas reconnu. */
export function dataUrlToBytes(dataUrl: string): { bytes: Uint8Array; mime: string } {
  if (!dataUrl.startsWith("data:")) {
    throw new Error(`dataUrlToBytes : non-data URL ("${dataUrl.slice(0, 20)}...")`);
  }
  const commaIdx = dataUrl.indexOf(",");
  if (commaIdx === -1) throw new Error("dataUrlToBytes : virgule manquante");
  const header = dataUrl.slice(5, commaIdx); // "image/png;base64"
  const payload = dataUrl.slice(commaIdx + 1);

  const [mimePart, ...flags] = header.split(";");
  const mime = mimePart || "application/octet-stream";
  const isBase64 = flags.includes("base64");

  if (!isBase64) {
    // data URL non-base64 (rare) : URL-decode
    const decoded = decodeURIComponent(payload);
    const bytes = new Uint8Array(decoded.length);
    for (let i = 0; i < decoded.length; i++) bytes[i] = decoded.charCodeAt(i) & 0xff;
    return { bytes, mime };
  }
  // base64 → binary
  const bin = atob(payload);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return { bytes, mime };
}

const MIME_BY_EXT: Record<string, string> = {
  png: "image/png", jpg: "image/jpeg", jpeg: "image/jpeg",
  webp: "image/webp", gif: "image/gif", avif: "image/avif",
  bmp: "image/bmp", svg: "image/svg+xml",
  mp4: "video/mp4", webm: "video/webm", mov: "video/quicktime", mkv: "video/x-matroska",
  pdf: "application/pdf",
  // Text / code (cf. R-FIL-01)
  txt: "text/plain", md: "text/markdown", json: "application/json",
  csv: "text/csv", yaml: "application/yaml", toml: "application/toml",
  ts: "text/typescript", tsx: "text/typescript",
  js: "text/javascript", jsx: "text/javascript",
  py: "text/x-python", rs: "text/x-rust", go: "text/x-go",
  c: "text/x-c", cpp: "text/x-c++", java: "text/x-java",
};

/** Devine un MIME à partir d'une extension (sans le point). Retourne
 *  `application/octet-stream` si inconnu. */
export function mimeFromExt(ext: string): string {
  return MIME_BY_EXT[ext.toLowerCase().replace(/^\./, "")]
    ?? "application/octet-stream";
}

/** Devine une extension (sans le point) à partir d'un MIME. */
export function extFromMime(mime: string): string {
  const m = mime.toLowerCase();
  for (const [ext, mm] of Object.entries(MIME_BY_EXT)) {
    if (mm === m) return ext;
  }
  // Cas fréquents non bijectifs : `image/jpeg` → jpg
  if (m === "image/jpeg") return "jpg";
  return "bin";
}

/** Construit un `AssetRef` mode embed. Calcule sha256 + déduit mime. */
export async function buildEmbedRef(
  bytes: Uint8Array,
  mimeHint?: string,
): Promise<Extract<AssetRef, { mode: "embed" }>> {
  const sha256 = await sha256Hex(bytes);
  return {
    mode: "embed",
    sha256,
    mime: mimeHint ?? "application/octet-stream",
    sizeBytes: bytes.length,
  };
}

/** Construit un `AssetRef` mode link. */
export function buildLinkRef(href: string, opts: { sha256?: string; sizeBytes?: number } = {}): Extract<AssetRef, { mode: "link" }> {
  const ref: Extract<AssetRef, { mode: "link" }> = { mode: "link", href };
  if (opts.sha256) ref.sha256 = opts.sha256;
  if (opts.sizeBytes !== undefined) ref.sizeBytes = opts.sizeBytes;
  return ref;
}

// ── Cache LRU de blob URLs pour les embeds ─────────────────────────────────
// Sans cache, chaque résolution d'un même asset créerait un nouveau Blob et un
// nouveau URL — bloat mémoire et perf rendering. La clé est `sha256` (stable).
// LRU borné à 256 entrées (~ images visibles à un instant T sur gros projet).

const BLOB_URL_CACHE = new Map<string, string>();
const BLOB_URL_LIMIT = 256;

function cacheGet(key: string): string | undefined {
  const v = BLOB_URL_CACHE.get(key);
  if (v === undefined) return undefined;
  // Touch : remettre en fin de Map pour LRU
  BLOB_URL_CACHE.delete(key);
  BLOB_URL_CACHE.set(key, v);
  return v;
}

function cacheSet(key: string, value: string): void {
  BLOB_URL_CACHE.set(key, value);
  // Eviction si dépassement
  while (BLOB_URL_CACHE.size > BLOB_URL_LIMIT) {
    const oldestKey = BLOB_URL_CACHE.keys().next().value;
    if (oldestKey === undefined) break;
    const oldUrl = BLOB_URL_CACHE.get(oldestKey);
    BLOB_URL_CACHE.delete(oldestKey);
    if (oldUrl && oldUrl.startsWith("blob:")) {
      try { URL.revokeObjectURL(oldUrl); } catch { /* ignore */ }
    }
  }
}

/** Libère TOUS les blob URLs du cache (tests / unload). */
export function releaseAllBlobUrls(): void {
  for (const url of BLOB_URL_CACHE.values()) {
    if (url.startsWith("blob:")) {
      try { URL.revokeObjectURL(url); } catch { /* ignore */ }
    }
  }
  BLOB_URL_CACHE.clear();
}

/**
 * Résout un `AssetRef` en URL renderable (synchrone).
 *
 * - `embed` → blob URL stable par sha256 (caché LRU 256). Si le blob est
 *   introuvable dans `blobs`, renvoie `""` — le caller décide du fallback.
 * - `link`  → renvoie `href` tel quel. Pour un chemin local, le caller doit
 *   appliquer `convertFileSrc` (côté Tauri).
 *
 * Note : on n'utilise pas `await` ici car `URL.createObjectURL` est sync.
 * Le SHA est déjà connu côté ref → pas de recompute.
 */
export function resolveAssetRefSync(
  asset: AssetRef,
  blobs: Record<string, Uint8Array> | undefined,
): string {
  if (asset.mode === "link") return asset.href;

  // mode === "embed"
  const cached = cacheGet(asset.sha256);
  if (cached) return cached;

  const bytes = blobs?.[asset.sha256];
  if (!bytes) return ""; // blob manquant — caller traite

  // Construction du blob URL et mise en cache LRU
  const blob = new Blob([bytes], { type: asset.mime });
  const url = URL.createObjectURL(blob);
  cacheSet(asset.sha256, url);
  return url;
}

/**
 * Variante async pour les cas où on veut éventuellement re-hasher pour valider
 * (debug / repair). Pour le rendu normal, utiliser `resolveAssetRefSync`.
 */
export async function resolveAssetRef(
  asset: AssetRef,
  blobs: Record<string, Uint8Array> | undefined,
): Promise<string> {
  return resolveAssetRefSync(asset, blobs);
}
