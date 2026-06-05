import { invoke } from "@tauri-apps/api/core";
import { BoardImage } from "../types";
import { nanoid } from "../utils/nanoid";
// B-STORE — images « du dur » : on écrit l'image sur disque (store hashé) et on
// ne garde dans le doc qu'une référence `link`. Plus d'embed dans `project.blobs`
// (qui gonflait le doc et faisait freezer `A.save` à chaque navigation/sauvegarde).
import { dataUrlToBytes, buildLinkRef, sha256Hex } from "../utils/assetRef";
import { saveAsset } from "../utils/assets";

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

      // B-STORE : décode pour le hash/taille, écrit sur disque, ref `link`.
      const { bytes } = dataUrlToBytes(dataUrl);
      const ext = guessExtFromPath(path);
      const assetId = await saveAsset(dataUrl, ext);
      const sha256 = await sha256Hex(bytes);
      const assetRef = buildLinkRef(assetId, { sha256, sizeBytes: bytes.length });

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
