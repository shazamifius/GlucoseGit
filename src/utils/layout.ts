import { BoardImage } from "../types";

interface LayoutResult {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

interface GridOptions {
  cols: number;         // 0 = auto
  targetSize: number;   // target width per image
  gap: number;
  startX: number;
  startY: number;
}

interface CompactOptions {
  targetHeight: number;
  maxRowWidth: number;  // 0 = unlimited
  gap: number;
  startX: number;
  startY: number;
  cols?: number;
}

/** Grid: même largeur par colonne, hauteur proportionnelle — JAMAIS d'étirement */
export function gridLayout(images: BoardImage[], opts: GridOptions): LayoutResult[] {
  const cols = opts.cols > 0 ? opts.cols : Math.max(1, Math.round(Math.sqrt(images.length)));
  const w = opts.targetSize;
  const results: LayoutResult[] = [];

  let y = opts.startY;
  for (let rowStart = 0; rowStart < images.length; rowStart += cols) {
    const row = images.slice(rowStart, rowStart + cols);
    const rowHeights = row.map((img) => {
      const ratio = img.originalWidth / img.originalHeight || 1;
      return w / ratio;
    });
    const maxH = Math.max(...rowHeights);

    row.forEach((img, i) => {
      const ratio = img.originalWidth / img.originalHeight || 1;
      const h = w / ratio;
      results.push({
        id: img.id,
        x: opts.startX + i * (w + opts.gap) + w / 2,
        y: y + h / 2,
        width: w,
        height: h,
      });
    });
    y += maxH + opts.gap;
  }
  return results;
}

/** Compact rows: respecte les ratios, remplit des rangées — JAMAIS d'étirement */
export function compactRowsLayout(images: BoardImage[], opts: CompactOptions): LayoutResult[] {
  const targetH = opts.targetHeight;
  const maxW = opts.maxRowWidth > 0 ? opts.maxRowWidth : targetH * 8;
  const results: LayoutResult[] = [];

  const scaled = images.map((img) => {
    const ratio = img.originalWidth / img.originalHeight || 1;
    return { id: img.id, w: targetH * ratio, h: targetH };
  });

  let currentRow: typeof scaled = [];
  let currentRowWidth = 0;
  const rows: (typeof scaled)[] = [];

  for (const item of scaled) {
    if (currentRowWidth + item.w > maxW && currentRow.length > 0) {
      rows.push(currentRow);
      currentRow = [item];
      currentRowWidth = item.w + opts.gap;
    } else {
      currentRow.push(item);
      currentRowWidth += item.w + opts.gap;
    }
  }
  if (currentRow.length > 0) rows.push(currentRow);

  let y = opts.startY;
  rows.forEach((row) => {
    const totalW = row.reduce((s, it) => s + it.w, 0) + opts.gap * (row.length - 1);
    // Scale to fill row width, but cap at 1.15× to avoid distortion feeling
    const scale = Math.min(1.15, maxW / totalW);
    let x = opts.startX;
    row.forEach((item) => {
      const w = item.w * scale;
      const h = item.h * scale;
      results.push({ id: item.id, x: x + w / 2, y: y + h / 2, width: w, height: h });
      x += w + opts.gap;
    });
    y += row[0].h * scale + opts.gap;
  });

  return results;
}

/** Même hauteur: toutes les images à la même hauteur, largeur proportionnelle */
export function sameHeightLayout(images: BoardImage[], opts: CompactOptions): LayoutResult[] {
  const h = opts.targetHeight;
  const cols = opts.cols && opts.cols > 0 ? opts.cols : 0;
  const results: LayoutResult[] = [];

  if (cols > 0) {
    // Track y per column (masonry)
    const colX = Array.from({ length: cols }, (_, i) => opts.startX + i * (h * 1.5 + opts.gap));
    const colY = new Array(cols).fill(opts.startY);

    images.forEach((img, i) => {
      const col = i % cols;
      const ratio = img.originalWidth / img.originalHeight || 1;
      const w = h * ratio;
      results.push({
        id: img.id,
        x: colX[col] + w / 2,
        y: colY[col] + h / 2,
        width: w,
        height: h,
      });
      colY[col] += h + opts.gap;
    });
  } else {
    let x = opts.startX;
    images.forEach((img) => {
      const ratio = img.originalWidth / img.originalHeight || 1;
      const w = h * ratio;
      results.push({ id: img.id, x: x + w / 2, y: opts.startY + h / 2, width: w, height: h });
      x += w + opts.gap;
    });
  }

  return results;
}

/** Masonry: colonnes indépendantes, chaque image avec sa vraie proportion */
export function masonryLayout(images: BoardImage[], opts: GridOptions): LayoutResult[] {
  const cols = opts.cols > 0 ? opts.cols : Math.max(2, Math.round(Math.sqrt(images.length)));
  const w = opts.targetSize;
  const colY = new Array(cols).fill(opts.startY);
  const results: LayoutResult[] = [];

  images.forEach((img) => {
    // Place in the shortest column
    const col = colY.indexOf(Math.min(...colY));
    const ratio = img.originalWidth / img.originalHeight || 1;
    const h = w / ratio;
    const x = opts.startX + col * (w + opts.gap);
    results.push({ id: img.id, x: x + w / 2, y: colY[col] + h / 2, width: w, height: h });
    colY[col] += h + opts.gap;
  });
  return results;
}

/** Par slot preset: groupe les images par slotId, colonnes séparées */
export function bySlotLayout(
  images: BoardImage[],
  slotOrder: string[],
  opts: { colWidth: number; gap: number; startX: number; startY: number }
): LayoutResult[] {
  const groups = new Map<string | undefined, BoardImage[]>();
  images.forEach((img) => {
    const key = img.slotId;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(img);
  });

  const results: LayoutResult[] = [];
  const allSlots = [...slotOrder, undefined as string | undefined];

  allSlots.forEach((slotId, colIdx) => {
    const imgs = groups.get(slotId) || [];
    let y = opts.startY;
    imgs.forEach((img) => {
      const ratio = img.originalWidth / img.originalHeight || 1;
      const w = opts.colWidth;
      const h = w / ratio;
      const x = opts.startX + colIdx * (opts.colWidth + opts.gap) + w / 2;
      results.push({ id: img.id, x, y: y + h / 2, width: w, height: h });
      y += h + opts.gap;
    });
  });

  return results;
}

export function centerOfImages(images: BoardImage[]): { x: number; y: number } {
  if (!images.length) return { x: 0, y: 0 };
  const x = images.reduce((s, img) => s + img.x, 0) / images.length;
  const y = images.reduce((s, img) => s + img.y, 0) / images.length;
  return { x, y };
}

export function boundsOfImages(images: BoardImage[]): { minX: number; minY: number; maxX: number; maxY: number } {
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  images.forEach((img) => {
    minX = Math.min(minX, img.x - img.width / 2);
    minY = Math.min(minY, img.y - img.height / 2);
    maxX = Math.max(maxX, img.x + img.width / 2);
    maxY = Math.max(maxY, img.y + img.height / 2);
  });
  return { minX, minY, maxX, maxY };
}

export function aspectRatioToFloat(ar: string): number {
  const [w, h] = ar.split(":").map(Number);
  return w / h;
}
