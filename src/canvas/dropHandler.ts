import { invoke } from "@tauri-apps/api/core";
import { Annotation, BoardImage } from "../types";
import { nanoid } from "../utils/nanoid";
import { addImagesFromFiles } from "./fileImport";
import { getCDNCandidates } from "../utils/imageUpgrade";
// Phase 7.0 — assets externalisés (cf. PRE-PHASE7-AUDIT.md C-1)
// R-EMB-01 (Sprint 2) : on construit AssetRef embed à partir des bytes et on
// les ajoute via `addImage(boardId, img, bytes)` plutôt que de passer par
// le disque. Plus de `src: "asset:..."` créé par les nouveaux drops.
import { buildEmbedRef, dataUrlToBytes, mimeFromExt } from "../utils/assetRef";

type AddImageFn      = (boardId: string, img: BoardImage, embedBytes?: Uint8Array) => void;
type AddAnnotationFn = (boardId: string, ann: Annotation) => void;

const IMAGE_EXTS  = /\.(png|jpg|jpeg|gif|webp|avif|svg|bmp|tiff?)(\?.*)?$/i;
export const VIDEO_FILE_EXTS = /\.(mp4|mov|avi|mkv|webm|m4v)$/i;
export const VIDEO_URL_RE = /(?:youtube\.com\/(?:watch|shorts)|youtu\.be\/|tiktok\.com\/@[^/]+\/video\/|instagram\.com\/(?:reel|p)\/|vimeo\.com\/\d)/i;

export const EXT_COLOR: Record<string, string> = {
  blend: "#e87d0d", psd: "#31a8ff", kra: "#2d9b27", xcf: "#7b3ebb",
  ai: "#ff9a00", pdf: "#e53e3e",
  mp4: "#f87171", mov: "#f87171", avi: "#f87171", mkv: "#f87171",
  zip: "#a78bfa", rar: "#a78bfa", "7z": "#a78bfa",
  doc: "#2b579a", docx: "#2b579a", xls: "#217346", xlsx: "#217346",
  txt: "#888888", md: "#888888", json: "#f59e0b", py: "#3b82f6",
  js: "#f59e0b", ts: "#3b82f6", rs: "#f97316", cpp: "#60a5fa", c: "#60a5fa",
  obj: "#f87171", fbx: "#f87171", glb: "#f87171", gltf: "#f87171",
  c4d: "#086adb", ma: "#00aaff", mb: "#00aaff",
  aep: "#9999ff", prproj: "#9999ff",
};

export function makeSourceSticky(filePath: string, x: number, y: number): Annotation {
  const name = filePath.split(/[\\/]/).pop() ?? filePath;
  const ext  = name.split(".").pop()?.toLowerCase() ?? "";
  return {
    id: nanoid(), type: "sticky",
    x, y,
    text: name,
    sourceFile: filePath,
    bgColor: EXT_COLOR[ext] ?? "#1a1a2e",
    color: "#cccccc",
    width: 220, height: 80,
    fontSize: 11,
  };
}

// Convert Windows file:// pathname (/C:/foo) → C:/foo
function uriToPath(u: string): string {
  return decodeURIComponent(new URL(u).pathname).replace(/^\/([A-Za-z]:)/, "$1");
}

export async function addImagesFromDrop(
  e: DragEvent,
  worldX: number,
  worldY: number,
  boardId: string,
  addImage: AddImageFn,
  addAnnotation?: AddAnnotationFn,
): Promise<void> {
  const items = Array.from(e.dataTransfer?.items || []);
  const files = Array.from(e.dataTransfer?.files || []);
  const types = Array.from(e.dataTransfer?.types || []);

  // Diagnostic — accessible via DevTools (Ctrl+Shift+I) onglet Console
  console.debug("[drop] types:", types, "items:", items.length, "files:", files.length);

  // ── 1. Fichiers locaux non-image → source sticky (App Bridge) ─────────
  if (addAnnotation) {
    const srcFiles = files.filter((f) => !IMAGE_EXTS.test(f.name) && !f.type.startsWith("image/"));
    if (srcFiles.length > 0) {
      const withPaths = srcFiles
        .map((f) => (f as File & { path?: string }).path)
        .filter((p): p is string => !!p && (p.includes("/") || p.includes("\\")));
      if (withPaths.length > 0) {
        withPaths.forEach((path, i) =>
          addAnnotation(boardId, makeSourceSticky(path, worldX + i * 24, worldY + i * 24))
        );
        return;
      }
    }
  }

  // ── 2. Fichiers image locaux (drag depuis explorer/file manager) ──────
  const imageFiles = files.filter((f) => f.type.startsWith("image/"));
  for (let i = 0; i < imageFiles.length; i++) {
    const file = imageFiles[i];
    const dataUrl = await fileToDataUrl(file);
    const { width, height } = await getImageDimensions(dataUrl);
    const ext = file.type.split("/")[1] || "png";
    // R-EMB-01 : embed direct dans le doc, plus de bypass disque
    const { bytes, mime: detectedMime } = dataUrlToBytes(dataUrl);
    const mime = detectedMime || mimeFromExt(ext);
    const img = await makeImageFromBytes(bytes, mime, worldX + i * 24, worldY + i * 24, width, height);
    addImage(boardId, img, bytes);
  }
  if (imageFiles.length > 0) return;

  // ── 3. Drag depuis un navigateur web ──────────────────────────────────
  // On collecte TOUTES les URLs candidates depuis tous les types disponibles
  // (text/uri-list, text/html, text/plain) puis on les essaie dans l'ordre
  // de pertinence — ressemble-à-une-image-d'abord. Plus robuste qu'un
  // ordre figé qui peut rater Pinterest si l'URL est sans extension classique.
  const candidates: string[] = [];

  for (const item of items) {
    if (item.kind !== "string") continue;
    const data = await readItem(item);
    if (!data) continue;

    if (item.type === "text/uri-list") {
      const uris = data.split(/\r?\n/).map((s) => s.trim()).filter((s) => s && !s.startsWith("#"));

      // Sous-cas A : file:// non-image → source sticky
      if (addAnnotation) {
        const sourcePaths = uris.filter((u) => u.startsWith("file://") && !IMAGE_EXTS.test(u)).map(uriToPath);
        if (sourcePaths.length > 0) {
          sourcePaths.forEach((p, i) => addAnnotation(boardId, makeSourceSticky(p, worldX + i * 24, worldY + i * 24)));
          return;
        }
      }
      // Sous-cas B : file:// image
      const filePaths = uris
        .filter((u) => u.startsWith("file://"))
        .map(uriToPath)
        .filter((p) => IMAGE_EXTS.test(p));
      if (filePaths.length > 0) {
        await addImagesFromFiles(filePaths, worldX, worldY, boardId, addImage);
        return;
      }
      // Sous-cas C : URLs web → candidats à fetcher
      for (const u of uris) {
        if (u.startsWith("http://") || u.startsWith("https://")) candidates.push(u);
      }
    }

    if (item.type === "text/html") {
      const best = extractBestImageFromHtml(data);
      if (best) candidates.unshift(best); // priorité haute : extrait du HTML
    }

    if (item.type === "text/plain") {
      const t = data.trim();
      if (t.startsWith("http://") || t.startsWith("https://")) candidates.push(t);
    }
  }

  // Filet final : si dataTransfer.getData("text/plain") existe et n'est pas
  // déjà capturé ci-dessus
  const plain = (e.dataTransfer?.getData("text/plain") || "").trim();
  if (plain && (plain.startsWith("http://") || plain.startsWith("https://")) && !candidates.includes(plain)) {
    candidates.push(plain);
  }

  // Déduplique et trie : URLs qui ressemblent à de l'image en priorité
  const unique = Array.from(new Set(candidates));
  unique.sort((a, b) => Number(isImageUrl(b)) - Number(isImageUrl(a)));

  console.debug("[drop] candidates ordered:", unique);
  if (unique.length === 0) {
    console.warn("[drop] Aucun candidat exploitable. types reçus :", types);
    return;
  }

  // Essai séquentiel — on prend le premier succès
  for (const url of unique) {
    const fetched = await fetchBestImage(url);
    if (fetched) {
      // R-EMB-01 : si on a les bytes, embed direct ; sinon link sur URL
      if (fetched.bytes) {
        const img = await makeImageFromBytes(
          fetched.bytes, fetched.mime,
          worldX, worldY, fetched.width, fetched.height, url,
        );
        addImage(boardId, img, fetched.bytes);
      } else {
        addImage(boardId, makeImage(fetched.src, worldX, worldY, fetched.width, fetched.height, url));
      }
      console.debug("[drop] success via:", url);
      return;
    }
  }
  console.warn("[drop] tous les candidats ont échoué :", unique);
}

interface FetchedImage {
  /** Bytes décodés si dispo (préféré → embed). */
  bytes?: Uint8Array;
  /** MIME du blob ou de l'URL link. */
  mime: string;
  /** URL légacy de fallback (utilisée si bytes absent). */
  src: string;
  width: number;
  height: number;
}

async function fetchBestImage(url: string): Promise<FetchedImage | null> {
  // R-EMB-01 (Sprint 2) : on télécharge en data URL via Rust, on décode en
  // bytes → AssetRef embed. Plus de bypass disque (saveAsset retiré).
  const extFromUrl = (u: string) => {
    const m = u.toLowerCase().match(/\.([a-z0-9]+)(?:\?.*)?$/);
    return m ? m[1] : "png";
  };

  const tryUrl = async (target: string): Promise<FetchedImage | null> => {
    try {
      const dataUrl: string = await invoke("fetch_image", { url: target });
      const { width, height } = await getImageDimensions(dataUrl);
      const { bytes, mime: detectedMime } = dataUrlToBytes(dataUrl);
      const mime = detectedMime || mimeFromExt(extFromUrl(target));
      return { bytes, mime, src: target, width, height };
    } catch {
      return null;
    }
  };

  // Try CDN-upgraded URLs first (higher resolution, same image, no API key needed)
  const candidates = getCDNCandidates(url);
  for (const candidate of candidates) {
    const out = await tryUrl(candidate);
    if (out) return out;
  }

  // Fall back to the original URL
  const out = await tryUrl(url);
  if (out) return out;

  // Tout fetch a échoué : on tente de récupérer JUSTE les dimensions via
  // un <img> côté DOM (CORS-permitting) et on garde l'URL en mode link.
  try {
    const { width, height } = await getImageDimensions(url);
    return { mime: mimeFromExt(extFromUrl(url)), src: url, width, height };
  } catch (err) {
    console.error("Failed to fetch image:", err);
    return null;
  }
}

/**
 * Extrait la meilleure URL d'image possible du HTML draggué.
 * Stratégies essayées dans l'ordre, on garde la première qui marche :
 *   1. <picture><source srcset> (formats modernes, plusieurs résolutions)
 *   2. <img srcset> → plus haute résolution
 *   3. <img src>
 *   4. <img data-src> / <img data-original> / <img data-lazy-src> (lazy loading,
 *      typique de Pinterest, Instagram, Tumblr…)
 *   5. <meta property="og:image"> (Open Graph) — souvent l'image canonique
 *   6. <meta property="og:image:secure_url">
 *   7. background-image: url(...) dans un style inline
 */
function extractBestImageFromHtml(html: string): string | null {
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, "text/html");

  // 1) <picture><source srcset>
  const sources = Array.from(doc.querySelectorAll("source[srcset]"));
  for (const src of sources) {
    const srcset = src.getAttribute("srcset");
    if (srcset) {
      const best = parseSrcset(srcset);
      if (best) return best;
    }
  }

  // 2-4) <img>
  const img = doc.querySelector("img");
  if (img) {
    const srcset = img.getAttribute("srcset");
    if (srcset) {
      const best = parseSrcset(srcset);
      if (best) return best;
    }
    // Plusieurs sites lazy-load : src reste un placeholder, la vraie URL est
    // dans data-src / data-original / data-lazy-src
    for (const attr of ["src", "data-src", "data-original", "data-lazy-src", "data-fallback-src"]) {
      const v = img.getAttribute(attr);
      if (v && !v.startsWith("data:image/svg")) return normalizeUrl(v);
    }
  }

  // 5-6) Meta Open Graph
  for (const sel of ['meta[property="og:image:secure_url"]', 'meta[property="og:image"]', 'meta[name="twitter:image"]']) {
    const m = doc.querySelector(sel);
    const v = m?.getAttribute("content");
    if (v) return normalizeUrl(v);
  }

  // 7) background-image: url(...)
  const styled = Array.from(doc.querySelectorAll<HTMLElement>("[style*='background-image']"));
  for (const el of styled) {
    const style = el.getAttribute("style") ?? "";
    const m = style.match(/background-image\s*:\s*url\((['"]?)([^'")]+)\1\)/i);
    if (m && m[2]) return normalizeUrl(m[2]);
  }

  return null;
}

function parseSrcset(srcset: string): string | null {
  const entries = srcset
    .split(",")
    .map((s) => s.trim().split(/\s+/))
    .filter((parts) => parts.length >= 1)
    .map((parts) => ({
      url: normalizeUrl(parts[0]),
      width: parseInt(parts[1]?.replace("w", "") || "0") || 0,
    }));

  if (entries.length === 0) return null;
  // Return highest resolution
  return entries.sort((a, b) => b.width - a.width)[0].url;
}

// Normalize protocol-relative URLs (//example.com/...) to https://
function normalizeUrl(url: string): string {
  if (url.startsWith("//")) return "https:" + url;
  return url;
}

/** Domaines CDN connus pour héberger principalement des images, même sans
 * extension de fichier dans l'URL. Permet d'accepter les URLs Pinterest/Twitter/
 * Instagram/etc. qui sont parfois minifiées sans `.jpg`. */
const IMAGE_CDN_HOSTS = [
  /(^|\.)pinimg\.com$/i,
  /(^|\.)pbs\.twimg\.com$/i,
  /(^|\.)cdninstagram\.com$/i,
  /(^|\.)fbcdn\.net$/i,
  /(^|\.)redd\.it$/i,
  /(^|\.)imgur\.com$/i,
  /(^|\.)tumblr\.com$/i,
  /(^|\.)wallhaven\.cc$/i,
  /(^|\.)wixmp\.com$/i,
  /artstation\.com\/p\/assets\//i,
  /(^|\.)deviantart\.net$/i,
  /(^|\.)staticflickr\.com$/i,
  /(^|\.)media-amazon\.com$/i,
  /(^|\.)googleusercontent\.com$/i,
];

function isImageUrl(url: string): boolean {
  try {
    const u = new URL(url);
    if (u.protocol === "data:") return true;
    if (u.protocol !== "http:" && u.protocol !== "https:") return false;
    // 1) Extension classique
    if (/\.(jpg|jpeg|png|gif|webp|avif|svg|bmp|tiff?)(\?.*)?$/i.test(u.pathname)) return true;
    // 2) Hôte CDN d'images connu (Pinterest, Twitter, Instagram, etc.)
    if (IMAGE_CDN_HOSTS.some((rx) => rx.test(u.hostname) || rx.test(u.href))) return true;
    return false;
  } catch {
    return false;
  }
}

function fileToDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

function getImageDimensions(src: string): Promise<{ width: number; height: number }> {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => resolve({ width: img.naturalWidth, height: img.naturalHeight });
    img.onerror = () => resolve({ width: 400, height: 300 });
    img.src = src;
  });
}

function readItem(item: DataTransferItem): Promise<string> {
  return new Promise((resolve) => item.getAsString(resolve));
}

function makeImage(
  src: string,
  x: number,
  y: number,
  width: number,
  height: number,
  sourceUrl?: string
): BoardImage {
  const maxW = 600;
  const scale = width > maxW ? maxW / width : 1;
  return {
    id: nanoid(),
    src,
    x,
    y,
    width: width * scale,
    height: height * scale,
    rotation: 0,
    locked: false,
    tags: [],
    sourceUrl,
    originalWidth: width,
    originalHeight: height,
  };
}

/**
 * R-EMB-01 (Sprint 2) — construit une BoardImage `mode: "embed"` à partir
 * des bytes (ex: data URL drag depuis le web décodé). Le caller passe les
 * bytes à `addImage(boardId, img, bytes)` pour qu'ils soient ajoutés au
 * project.blobs dans la même mutation.
 */
async function makeImageFromBytes(
  bytes: Uint8Array,
  mime: string,
  x: number,
  y: number,
  width: number,
  height: number,
  sourceUrl?: string,
): Promise<BoardImage> {
  const asset = await buildEmbedRef(bytes, mime);
  const maxW = 600;
  const scale = width > maxW ? maxW / width : 1;
  return {
    id: nanoid(),
    asset,
    x, y,
    width: width * scale,
    height: height * scale,
    rotation: 0,
    locked: false,
    tags: [],
    sourceUrl,
    originalWidth: width,
    originalHeight: height,
  };
}
