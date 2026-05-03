import { invoke } from "@tauri-apps/api/core";
import { Annotation, BoardImage } from "../types";
import { nanoid } from "../utils/nanoid";
import { addImagesFromFiles } from "./fileImport";
import { getCDNCandidates } from "../utils/imageUpgrade";

type AddImageFn      = (boardId: string, img: BoardImage) => void;
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

  // 1. Source files from File objects (Tauri exposes .path on File objects)
  //    Accepte n'importe quel fichier non-image
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
      // Pas de chemin absolu disponible → laisser passer au handler text/uri-list ci-dessous
    }
  }

  // 2. Local files with image mime type (from file manager via webview)
  const imageFiles = files.filter((f) => f.type.startsWith("image/"));
  for (let i = 0; i < imageFiles.length; i++) {
    const file = imageFiles[i];
    const src = await fileToDataUrl(file);
    const { width, height } = await getImageDimensions(src);
    addImage(boardId, makeImage(src, worldX + i * 24, worldY + i * 24, width, height));
  }
  if (imageFiles.length > 0) return;

  // 3. text/uri-list — handles both file:// URIs (Dolphin) and http(s):// image URLs
  for (const item of items) {
    if (item.kind === "string" && item.type === "text/uri-list") {
      const raw = await readItem(item);
      const uris = raw.split(/\r?\n/).map((s) => s.trim()).filter((s) => s && !s.startsWith("#"));

      // Source file URIs — tout fichier non-image depuis file://
      if (addAnnotation) {
        const sourcePaths = uris
          .filter((u) => u.startsWith("file://") && !IMAGE_EXTS.test(u))
          .map(uriToPath);
        if (sourcePaths.length > 0) {
          sourcePaths.forEach((p, i) => addAnnotation(boardId, makeSourceSticky(p, worldX + i * 24, worldY + i * 24)));
          return;
        }
      }

      const filePaths = uris
        .filter((u) => u.startsWith("file://"))
        .map(uriToPath)
        .filter((p) => IMAGE_EXTS.test(p));

      if (filePaths.length > 0) {
        await addImagesFromFiles(filePaths, worldX, worldY, boardId, addImage);
        return;
      }

      const webUrl = uris.find((u) => (u.startsWith("http://") || u.startsWith("https://")) && IMAGE_EXTS.test(u));
      if (webUrl) {
        const fetched = await fetchBestImage(webUrl);
        if (fetched) addImage(boardId, makeImage(fetched.src, worldX, worldY, fetched.width, fetched.height, webUrl));
        return;
      }
    }
  }

  // 3. HTML img tag with srcset (from browser drag of img element)
  for (const item of items) {
    if (item.kind === "string" && item.type === "text/html") {
      const html = await readItem(item);
      const bestUrl = extractBestImageFromHtml(html);
      if (bestUrl) {
        const fetched = await fetchBestImage(bestUrl);
        if (fetched) {
          addImage(boardId, makeImage(fetched.src, worldX, worldY, fetched.width, fetched.height, bestUrl));
        }
        return;
      }
    }
  }

  // 4. Plain text URL
  for (const item of items) {
    if (item.kind === "string" && item.type === "text/uri-list") {
      const url = await readItem(item);
      if (url && isImageUrl(url)) {
        const fetched = await fetchBestImage(url);
        if (fetched) addImage(boardId, makeImage(fetched.src, worldX, worldY, fetched.width, fetched.height, url));
        return;
      }
    }

    if (item.kind === "string" && item.type === "text/html") {
      const html = await readItem(item);
      const bestUrl = extractBestImageFromHtml(html);
      if (bestUrl) {
        const fetched = await fetchBestImage(bestUrl);
        if (fetched) {
          const { src, width, height } = fetched;
          addImage(boardId, makeImage(src, worldX, worldY, width, height, bestUrl));
        }
        return;
      }
    }
  }

  // 5. Fallback: plain text URL
  const text = e.dataTransfer?.getData("text/plain") || "";
  if (isImageUrl(text)) {
    const fetched = await fetchBestImage(text);
    if (fetched) addImage(boardId, makeImage(fetched.src, worldX, worldY, fetched.width, fetched.height, text));
  }
}

async function fetchBestImage(url: string): Promise<{ src: string; width: number; height: number } | null> {
  // Try CDN-upgraded URLs first (higher resolution, same image, no API key needed)
  const candidates = getCDNCandidates(url);
  for (const candidate of candidates) {
    try {
      const dataUrl: string = await invoke("fetch_image", { url: candidate });
      const { width, height } = await getImageDimensions(dataUrl);
      return { src: dataUrl, width, height };
    } catch {
      // Candidate unavailable — try next
    }
  }

  // Fall back to the original URL
  try {
    const dataUrl: string = await invoke("fetch_image", { url });
    const { width, height } = await getImageDimensions(dataUrl);
    return { src: dataUrl, width, height };
  } catch (err) {
    console.error("Failed to fetch image:", err);
    try {
      const { width, height } = await getImageDimensions(url);
      return { src: url, width, height };
    } catch {
      return null;
    }
  }
}

function extractBestImageFromHtml(html: string): string | null {
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, "text/html");

  // Priority: srcset > src
  const img = doc.querySelector("img");
  if (!img) return null;

  const srcset = img.getAttribute("srcset");
  if (srcset) {
    const best = parseSrcset(srcset);
    if (best) return best;
  }

  const src = img.getAttribute("src");
  return src ? normalizeUrl(src) : null;
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

function isImageUrl(url: string): boolean {
  try {
    const u = new URL(url);
    return /\.(jpg|jpeg|png|gif|webp|avif|svg|bmp|tiff?)(\?.*)?$/i.test(u.pathname) ||
      u.protocol === "data:";
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
