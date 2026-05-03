import { invoke } from "@tauri-apps/api/core";
import { BoardImage } from "../types";
import { nanoid } from "../utils/nanoid";

type AddImageFn = (boardId: string, img: BoardImage) => void;

export async function addImagesFromFiles(
  paths: string[],
  startX: number,
  startY: number,
  boardId: string,
  addImage: AddImageFn
): Promise<void> {
  for (let i = 0; i < paths.length; i++) {
    const path = paths[i];
    try {
      const dataUrl: string = await invoke("read_image_file", { path });
      const { width, height } = await getImageDimensions(dataUrl);
      const offset = i * 20;
      const maxW = 600;
      const scale = width > maxW ? maxW / width : 1;
      const img: BoardImage = {
        id: nanoid(),
        src: dataUrl,
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
      addImage(boardId, img);
    } catch (err) {
      console.error(`Failed to load ${path}:`, err);
    }
  }
}

function getImageDimensions(src: string): Promise<{ width: number; height: number }> {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => resolve({ width: img.naturalWidth, height: img.naturalHeight });
    img.onerror = () => resolve({ width: 400, height: 300 });
    img.src = src;
  });
}
