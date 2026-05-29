import { invoke } from "@tauri-apps/api/core";
import { BoardImage } from "../types";
import { nanoid } from "../utils/nanoid";
// R-EMB-01 (Sprint 2) — embed direct dans le doc Automerge plutôt que
// d'externaliser en `asset:<hash>.<ext>` sur disque. Plus de migration
// nécessaire au prochain load : self-contained dès la création.
import { buildEmbedRef, dataUrlToBytes, mimeFromExt } from "../utils/assetRef";

type AddImageFn = (boardId: string, img: BoardImage, embedBytes?: Uint8Array) => void;

export async function addImagesFromFiles(
  paths: string[],
  startX: number,
  startY: number,
  boardId: string,
  addImage: AddImageFn,
): Promise<void> {
  for (let i = 0; i < paths.length; i++) {
    const path = paths[i];
    try {
      // Le backend retourne toujours une data URL `data:<mime>;base64,<payload>`.
      const dataUrl: string = await invoke("read_image_file", { path });
      const { width, height } = await getImageDimensions(dataUrl);

      // R-EMB-01 : décode → bytes + build AssetRef embed
      const { bytes, mime: detectedMime } = dataUrlToBytes(dataUrl);
      const ext = guessExtFromPath(path);
      const mime = detectedMime || mimeFromExt(ext);
      const assetRef = await buildEmbedRef(bytes, mime);

      const offset = i * 20;
      const maxW = 600;
      const scale = width > maxW ? maxW / width : 1;
      const img: BoardImage = {
        id: nanoid(),
        asset: assetRef,
        x: startX + offset,
        y: startY + offset,
        width: width * scale,
        height: height * scale,
        rotation: 0,
        locked: false,
        tags: [],
        sourceUrl: `file://${path}`,
        originalWidth: width,
        originalHeight: height,
      };
      addImage(boardId, img, bytes);
    } catch (err) {
      console.error(`Failed to load ${path}:`, err);
    }
  }
}

function guessExtFromPath(p: string): string {
  const m = p.toLowerCase().match(/\.([a-z0-9]+)$/);
  return m ? m[1] : "png";
}

function getImageDimensions(src: string): Promise<{ width: number; height: number }> {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => resolve({ width: img.naturalWidth, height: img.naturalHeight });
    img.onerror = () => resolve({ width: 400, height: 300 });
    img.src = src;
  });
}
