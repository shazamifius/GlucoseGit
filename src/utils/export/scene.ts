// Modèle de SCÈNE d'export — couche partagée par SVG / PNG / HTML.
//
// Transforme un `Board` (du `Project`) en une description géométrique normalisée
// et AUTO-SUFFISANTE : positions monde résolues, couleurs symbiotiques calculées,
// flèches routées (mêmes maths que le canvas live), images embarquées en dataURL.
// Aucun accès au DOM ni à PixiJS : reproductible et testable hors écran.
import {
  Project, Board, Annotation, BoardImage,
  isTextAnnotation, isStickyAnnotation, isArrowAnnotation, isMembraneAnnotation,
} from "../../types";
import { getSymbioticHue } from "../symbioticHue";
import { stripInlineMarkdown, splitBlocks } from "./markdownText";

// ── Types de scène ────────────────────────────────────────────────────────
export interface Box { left: number; right: number; top: number; bottom: number; }
export interface Pt { x: number; y: number; }

export interface SceneCard {
  id: string;
  x: number; y: number; w: number; h: number;
  text: string;            // markdown brut
  fontSize: number;
  textColor: string;       // couleur du texte (blanc par défaut)
  auraColor: string;       // teinte symbiotique (glow + accents)
}
export interface SceneSticky {
  id: string;
  x: number; y: number; w: number; h: number;
  text: string;
  color: string;           // texte
  bgColor: string;
  operator?: "AND" | "OR" | "BUT" | "BECAUSE";
}
export interface SceneMembrane {
  id: string;
  x: number; y: number; w: number; h: number;
  color: string;
  text?: string;
}
export interface SceneArrow {
  id: string;
  sourceId?: string;
  targetId?: string;
  points: Pt[];
  strokeWidth: number;
  colStart: string;
  colEnd: string;
  curved: boolean;
  bidirectional: boolean;
  predicate?: string;
  predicateColor?: string;
  predicateLabel?: string;
  label?: string;          // libellé court sur la flèche
  longText?: string;       // description Markdown (badge « i »)
  mid: Pt;                 // point milieu (placement badge/label)
}
export interface SceneImage {
  id: string;
  x: number; y: number; w: number; h: number; // top-left résolu
  rotation: number;
  href: string;            // dataURL (embed) ou URL (link)
}

export interface ExportScene {
  projectName: string;
  boardName: string;
  background: string;
  bbox: Box;               // englobe tout + marge
  width: number;           // bbox.right - bbox.left
  height: number;          // bbox.bottom - bbox.top
  cards: SceneCard[];
  stickies: SceneSticky[];
  membranes: SceneMembrane[];
  arrows: SceneArrow[];
  images: SceneImage[];
}

export const SCENE_BG = "#0d0d0d";
const MARGIN = 120;        // marge autour du contenu

const PREDICATE_COLORS: Record<string, string> = {
  est_precurseur: "#f59e0b", contredit: "#ef4444", herite_de: "#8b5cf6",
  inspire: "#10b981", depend_de: "#3b82f6", illustre: "#f472b6",
};
const PREDICATE_LABELS: Record<string, string> = {
  est_precurseur: "→", contredit: "✗", herite_de: "⊂",
  inspire: "✦", depend_de: "⊕", illustre: "◎",
};

// ── Mesure de texte (canvas si dispo, sinon approximation) ─────────────────
export type MeasureFn = (text: string, fontSize: number, bold: boolean, italic: boolean) => number;

export function makeMeasure(): MeasureFn {
  let ctx: CanvasRenderingContext2D | null = null;
  try {
    if (typeof document !== "undefined") {
      ctx = document.createElement("canvas").getContext("2d");
    }
  } catch { /* environnement sans DOM (tests) */ }
  return (text, fontSize, bold, italic) => {
    if (ctx) {
      ctx.font = `${italic ? "italic " : ""}${bold ? "600 " : ""}${fontSize}px system-ui, sans-serif`;
      return ctx.measureText(text).width;
    }
    // Fallback : ~0.52em par caractère
    return text.length * fontSize * 0.52;
  };
}

const LINE_H = 1.45;
const PAD_X = 24;
const PAD_Y = 16;
const HEADING_SCALE: Record<string, number> = { h1: 1.5, h2: 1.3, h3: 1.15 };

/** Estime (w,h) d'une carte texte. Respecte width/height explicites si fournis. */
export function measureCard(
  text: string, fontSize: number, measure: MeasureFn,
  fixedW?: number, fixedH?: number,
): { w: number; h: number } {
  const MAXW = 600;
  const blocks = splitBlocks(text || "Texte");

  // Largeur naturelle = plus longue ligne (bornée), sinon largeur imposée.
  let contentW: number;
  if (fixedW) {
    contentW = fixedW - PAD_X * 2;
  } else {
    let natural = 0;
    for (const b of blocks) {
      const scale = HEADING_SCALE[b.kind] ?? 1;
      const plain = b.runs.map((r) => r.text).join("");
      natural = Math.max(natural, measure(plain, fontSize * scale, b.kind.startsWith("h"), false));
    }
    contentW = Math.min(natural, MAXW - PAD_X * 2);
  }
  contentW = Math.max(contentW, 40);

  // Hauteur = somme des lignes physiques (wrap simple par mesure).
  let h = PAD_Y * 2;
  for (const b of blocks) {
    const scale = HEADING_SCALE[b.kind] ?? 1;
    const fs = fontSize * scale;
    const plain = b.runs.map((r) => r.text).join("");
    const words = plain.split(/\s+/).filter(Boolean);
    let lineW = 0, lines = 1;
    const bulletIndent = b.kind === "bullet" ? fs * 1.2 : 0;
    for (const word of words) {
      const ww = measure(word + " ", fs, b.kind.startsWith("h"), false);
      if (lineW + ww > contentW - bulletIndent && lineW > 0) { lines++; lineW = ww; }
      else lineW += ww;
    }
    h += lines * fs * LINE_H + (b.kind.startsWith("h") ? 6 : 2);
  }

  return {
    w: fixedW ?? Math.round(contentW + PAD_X * 2),
    h: fixedH ?? Math.round(h),
  };
}

// ── Boîtes de nœuds (anchoring + obstacles), cohérent avec le rendu ────────
function annBox(ann: Annotation, measure: MeasureFn): Box | null {
  if (isTextAnnotation(ann)) {
    const { w, h } = measureCard(ann.text, ann.fontSize ?? 14, measure, ann.width, ann.height);
    return { left: ann.x, right: ann.x + w, top: ann.y, bottom: ann.y + h };
  }
  if (isStickyAnnotation(ann)) {
    const w = ann.width ?? 160, h = ann.height ?? 120;
    return { left: ann.x, right: ann.x + w, top: ann.y, bottom: ann.y + h };
  }
  if (isMembraneAnnotation(ann)) {
    return { left: ann.x, right: ann.x + ann.width, top: ann.y, bottom: ann.y + ann.height };
  }
  return null; // les flèches ne sont pas des obstacles
}
function imgBox(img: BoardImage): Box {
  return {
    left: img.x - img.width / 2, right: img.x + img.width / 2,
    top: img.y - img.height / 2, bottom: img.y + img.height / 2,
  };
}

// ── Routage flèches (porté de ArrowSvgLayer, fonctions pures) ──────────────
function linesIntersect(a: Pt, b: Pt, c: Pt, d: Pt): boolean {
  const det = (b.x - a.x) * (d.y - c.y) - (b.y - a.y) * (d.x - c.x);
  if (det === 0) return false;
  const lambda = ((d.y - c.y) * (d.x - a.x) + (c.x - d.x) * (d.y - a.y)) / det;
  const gamma = ((a.y - b.y) * (d.x - a.x) + (b.x - a.x) * (d.y - a.y)) / det;
  return (0 < lambda && lambda < 1) && (0 < gamma && gamma < 1);
}
function lineIntersectsBox(p1: Pt, p2: Pt, box: Box): boolean {
  const tl = { x: box.left, y: box.top }, tr = { x: box.right, y: box.top };
  const bl = { x: box.left, y: box.bottom }, br = { x: box.right, y: box.bottom };
  if (linesIntersect(p1, p2, tl, tr)) return true;
  if (linesIntersect(p1, p2, tr, br)) return true;
  if (linesIntersect(p1, p2, br, bl)) return true;
  if (linesIntersect(p1, p2, bl, tl)) return true;
  const inside = (p: Pt) => p.x >= box.left && p.x <= box.right && p.y >= box.top && p.y <= box.bottom;
  return inside(p1) || inside(p2);
}
interface Obstacle extends Box { id: string; }
function getDynamicRoute(p1: Pt, p2: Pt, boxes: Obstacle[], visited = new Set<string>(), depth = 0): Pt[] {
  if (depth > 10) return [];
  let closest: Obstacle | null = null;
  let minT = Infinity;
  for (const box of boxes) {
    if (visited.has(box.id)) continue;
    if (lineIntersectsBox(p1, p2, box)) {
      const d = Math.hypot((box.left + box.right) / 2 - p1.x, (box.top + box.bottom) / 2 - p1.y);
      if (d < minT) { minT = d; closest = box; }
    }
  }
  if (!closest) return [];
  const PAD = 32;
  const cTL = { x: closest.left - PAD, y: closest.top - PAD };
  const cTR = { x: closest.right + PAD, y: closest.top - PAD };
  const cBL = { x: closest.left - PAD, y: closest.bottom + PAD };
  const cBR = { x: closest.right + PAD, y: closest.bottom + PAD };
  const paths: Pt[][] = [
    [cTL], [cTR], [cBL], [cBR],
    [cTL, cTR], [cTR, cBR], [cBR, cBL], [cBL, cTL],
    [cTR, cTL], [cBR, cTR], [cBL, cBR], [cTL, cBL],
  ];
  let bestPath: Pt[] = [];
  let bestDist = Infinity;
  for (const path of paths) {
    let valid = true;
    const localPts = [p1, ...path, p2];
    for (let i = 0; i < localPts.length - 1; i++) {
      if (lineIntersectsBox(localPts[i], localPts[i + 1], closest)) { valid = false; break; }
    }
    if (valid) {
      let d = 0;
      for (let i = 0; i < localPts.length - 1; i++) d += Math.hypot(localPts[i + 1].x - localPts[i].x, localPts[i + 1].y - localPts[i].y);
      if (d < bestDist) { bestDist = d; bestPath = path; }
    }
  }
  const newVisited = new Set(visited);
  newVisited.add(closest.id);
  const full: Pt[] = [];
  let cur = p1;
  for (const pt of bestPath) {
    full.push(...getDynamicRoute(cur, pt, boxes, newVisited, depth + 1));
    full.push(pt);
    cur = pt;
  }
  full.push(...getDynamicRoute(cur, p2, boxes, newVisited, depth + 1));
  return full;
}

// ── Conversion bytes → dataURL (embed auto-suffisant) ──────────────────────
function bytesToDataUrl(bytes: Uint8Array, mime: string): string {
  const CHUNK = 32_768;
  let binary = "";
  for (let i = 0; i < bytes.length; i += CHUNK) {
    binary += String.fromCharCode.apply(null, Array.from(bytes.subarray(i, i + CHUNK)));
  }
  return `data:${mime || "image/png"};base64,${btoa(binary)}`;
}

function resolveImageHref(img: BoardImage, blobs?: Record<string, Uint8Array>): string | null {
  const asset = img.asset;
  if (asset) {
    if (asset.mode === "embed") {
      const bytes = blobs?.[asset.sha256];
      if (bytes) return bytesToDataUrl(bytes, asset.mime);
      return null;
    }
    if (asset.mode === "link") return asset.href;
  }
  // Legacy : src direct (data:/http) — utilisable tel quel s'il est auto-porteur.
  if (img.src && (img.src.startsWith("data:") || img.src.startsWith("http"))) return img.src;
  return null;
}

// ── Build ──────────────────────────────────────────────────────────────────
export interface BuildOptions {
  /** Board ciblé ; par défaut le board actif du projet. */
  board?: Board;
  /** Inclure les images (peut alourdir). Défaut: true. */
  includeImages?: boolean;
}

export function buildScene(project: Project, opts: BuildOptions = {}): ExportScene {
  const board = opts.board ?? project.boards.find((b) => b.id === project.activeBoardId) ?? project.boards[0];
  const includeImages = opts.includeImages !== false;
  const measure = makeMeasure();
  const anns = board?.annotations ?? [];

  const cards: SceneCard[] = [];
  const stickies: SceneSticky[] = [];
  const membranes: SceneMembrane[] = [];
  const images: SceneImage[] = [];

  // Index des boîtes (anchoring + obstacles)
  const boxById = new Map<string, Box>();
  for (const ann of anns) {
    const b = annBox(ann, measure);
    if (b) boxById.set(ann.id, b);
  }
  for (const img of board?.images ?? []) boxById.set(img.id, imgBox(img));

  // Cartes / stickies / membranes
  for (const ann of anns) {
    if (isTextAnnotation(ann)) {
      const box = boxById.get(ann.id)!;
      const isWhite = !ann.color || ann.color === "#ffffff" || ann.color === "#fff";
      const hue = getSymbioticHue(ann, anns);
      cards.push({
        id: ann.id, x: box.left, y: box.top,
        w: box.right - box.left, h: box.bottom - box.top,
        text: ann.text, fontSize: ann.fontSize ?? 14,
        textColor: ann.color || "#ffffff",
        auraColor: isWhite ? `hsl(${hue.toFixed(1)}, 75%, 65%)` : ann.color!,
      });
    } else if (isStickyAnnotation(ann)) {
      const box = boxById.get(ann.id)!;
      stickies.push({
        id: ann.id, x: box.left, y: box.top,
        w: box.right - box.left, h: box.bottom - box.top,
        text: ann.text, color: ann.color || "#1a1a1a",
        bgColor: ann.bgColor || "#fde68a", operator: ann.operator,
      });
    } else if (isMembraneAnnotation(ann)) {
      membranes.push({
        id: ann.id, x: ann.x, y: ann.y, w: ann.width, h: ann.height,
        color: ann.color || "#8b5cf6", text: ann.text,
      });
    }
  }

  // Images
  if (includeImages) {
    for (const img of board?.images ?? []) {
      const href = resolveImageHref(img, project.blobs);
      if (!href) continue;
      const box = boxById.get(img.id)!;
      images.push({
        id: img.id, x: box.left, y: box.top,
        w: box.right - box.left, h: box.bottom - box.top,
        rotation: img.rotation || 0, href,
      });
    }
  }

  // Flèches
  const arrows: SceneArrow[] = [];
  const allObstacles: Obstacle[] = [...boxById.entries()].map(([id, b]) => ({ id, ...b }));

  const anchorOf = (refId: string | undefined, fallbackX: number, fallbackY: number): { x: number; y: number; box?: Box } => {
    if (!refId) return { x: fallbackX, y: fallbackY };
    const box = boxById.get(refId);
    if (box) return { x: (box.left + box.right) / 2, y: (box.top + box.bottom) / 2, box };
    return { x: fallbackX, y: fallbackY };
  };

  for (const ann of anns) {
    if (!isArrowAnnotation(ann)) continue;
    const aStart = anchorOf(ann.sourceId, ann.x, ann.y);
    const aEnd = anchorOf(ann.targetId, ann.x2 ?? ann.x + 100, ann.y2 ?? ann.y);

    const targetForStart = ann.waypoints?.length ? ann.waypoints[0] : aEnd;
    const targetForEnd = ann.waypoints?.length ? ann.waypoints[ann.waypoints.length - 1] : aStart;

    const pStart = { x: aStart.x, y: aStart.y };
    const pEnd = { x: aEnd.x, y: aEnd.y };
    const EDGE = 12;
    if (aStart.box) pStart.x = targetForStart.x < aStart.x ? aStart.box.left - EDGE : aStart.box.right + EDGE;
    if (aEnd.box) pEnd.x = targetForEnd.x < aEnd.x ? aEnd.box.left - EDGE : aEnd.box.right + EDGE;

    const pts: Pt[] = [{ x: pStart.x, y: pStart.y }];
    if (ann.waypoints && ann.waypoints.length > 0) {
      pts.push(...ann.waypoints.map((w) => ({ x: w.x, y: w.y })));
    } else {
      const valid = allObstacles.filter((o) => o.id !== ann.sourceId && o.id !== ann.targetId);
      const route = getDynamicRoute(pStart, pEnd, valid);
      if (route.length) pts.push(...route);
    }
    pts.push({ x: pEnd.x, y: pEnd.y });
    const n = pts.length;

    const srcAnn = ann.sourceId ? anns.find((a) => a.id === ann.sourceId) : null;
    const tgtAnn = ann.targetId ? anns.find((a) => a.id === ann.targetId) : null;
    const srcHue = srcAnn ? getSymbioticHue(srcAnn, anns) : getSymbioticHue({ ...ann, x: pts[0].x, y: pts[0].y }, anns);
    const tgtHue = tgtAnn ? getSymbioticHue(tgtAnn, anns) : getSymbioticHue({ ...ann, x: pts[n - 1].x, y: pts[n - 1].y }, anns);

    const midSeg = Math.floor(n / 2);
    const mp1 = pts[midSeg - 1] ?? pts[0];
    const mp2 = pts[midSeg] ?? pts[n - 1];

    arrows.push({
      id: ann.id,
      sourceId: ann.sourceId,
      targetId: ann.targetId,
      points: pts,
      strokeWidth: ann.strokeWidth ?? 2,
      colStart: `hsl(${srcHue.toFixed(1)}, 80%, 65%)`,
      colEnd: `hsl(${tgtHue.toFixed(1)}, 80%, 65%)`,
      curved: ann.arrowType === "curved",
      bidirectional: !!ann.arrowBidirectional,
      predicate: ann.predicate,
      predicateColor: ann.predicate ? (PREDICATE_COLORS[ann.predicate] ?? "#888") : undefined,
      predicateLabel: ann.predicate ? (PREDICATE_LABELS[ann.predicate] ?? "?") : undefined,
      label: ann.text || undefined,
      longText: ann.longText || undefined,
      mid: { x: (mp1.x + mp2.x) / 2, y: (mp1.y + mp2.y) / 2 },
    });
  }

  // Bounding box globale
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  const extend = (l: number, t: number, r: number, b: number) => {
    minX = Math.min(minX, l); minY = Math.min(minY, t);
    maxX = Math.max(maxX, r); maxY = Math.max(maxY, b);
  };
  for (const c of cards) extend(c.x, c.y, c.x + c.w, c.y + c.h);
  for (const s of stickies) extend(s.x, s.y, s.x + s.w, s.y + s.h);
  for (const m of membranes) extend(m.x, m.y, m.x + m.w, m.y + m.h);
  for (const im of images) extend(im.x, im.y, im.x + im.w, im.y + im.h);
  for (const ar of arrows) for (const p of ar.points) extend(p.x, p.y, p.x, p.y);

  if (!isFinite(minX)) { minX = 0; minY = 0; maxX = 800; maxY = 600; }

  const bbox: Box = {
    left: minX - MARGIN, top: minY - MARGIN,
    right: maxX + MARGIN, bottom: maxY + MARGIN,
  };

  return {
    projectName: project.name || "Glucose",
    boardName: board?.name || "Board",
    background: SCENE_BG,
    bbox,
    width: bbox.right - bbox.left,
    height: bbox.bottom - bbox.top,
    cards, stickies, membranes, arrows, images,
  };
}

// Réexports utilitaires (consommés par les renderers)
export { stripInlineMarkdown, splitBlocks };
