// Export PNG — rasterise le SVG natif (sceneToSvg) en bitmap plein-board HD.
// Contrairement à l'ancien export (screenshot WebGL du viewport visible, qui
// ratait les cartes DOM et les flèches SVG), on rend ICI toute la scène à haute
// résolution via un <canvas>.
import { ExportScene } from "./scene";
import { sceneToSvg } from "./toSvg";

/** Budget pixel max (évite d'exploser la mémoire sur de très grands boards). */
const MAX_DIM = 8000;
const MAX_AREA = 40_000_000; // ~40 Mpx

/** Rasterise une chaîne SVG en dataURL PNG aux dimensions pixel données. */
export async function svgToPngDataUrl(svg: string, pxW: number, pxH: number, bg?: string): Promise<string> {
  const blob = new Blob([svg], { type: "image/svg+xml;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  try {
    const img = new Image();
    img.decoding = "sync";
    await new Promise<void>((resolve, reject) => {
      img.onload = () => resolve();
      img.onerror = () => reject(new Error("Échec du chargement du SVG pour rasterisation."));
      img.src = url;
    });
    const canvas = document.createElement("canvas");
    canvas.width = Math.max(1, Math.round(pxW));
    canvas.height = Math.max(1, Math.round(pxH));
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("Contexte 2D indisponible.");
    if (bg) { ctx.fillStyle = bg; ctx.fillRect(0, 0, canvas.width, canvas.height); }
    ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
    return canvas.toDataURL("image/png");
  } finally {
    URL.revokeObjectURL(url);
  }
}

/**
 * Rend la scène en PNG. `scale` = facteur de suréchantillonnage souhaité (2 = HD).
 * Borné par MAX_DIM / MAX_AREA pour rester raisonnable.
 */
export async function sceneToPngDataUrl(scene: ExportScene, scale = 2): Promise<string> {
  const baseW = Math.max(1, scene.width);
  const baseH = Math.max(1, scene.height);

  let s = scale;
  // bornage par dimension
  s = Math.min(s, MAX_DIM / baseW, MAX_DIM / baseH);
  // bornage par aire
  if (baseW * baseH * s * s > MAX_AREA) s = Math.sqrt(MAX_AREA / (baseW * baseH));
  s = Math.max(0.25, s);

  const svg = sceneToSvg(scene, { transparent: false });
  return svgToPngDataUrl(svg, baseW * s, baseH * s, scene.background);
}
