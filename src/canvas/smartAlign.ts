// ────────────────────────────────────────────────────────────────────────────
// SNAP-1 — Alignement intelligent : MOTEUR PUR (ni React, ni PixiJS, ni store).
//
// Avant, la mécanique de snap était dupliquée à l'identique dans deux couches
// (`HtmlAnnotationLayer` pour les textes/notes, `GlucoseCanvas` pour les images),
// ne couvrait QUE le déplacement d'un élément déjà créé, ignorait membranes et
// dossiers, et utilisait un seuil en unités MONDE (donc un aimant 10× plus fort
// dézoomé que zoomé). Tout est désormais ici, en une seule implémentation :
//
//   • un modèle géométrique unique — `AlignRect` en coordonnées MONDE, en
//     coin-haut-gauche. Les images (ancre 0.5 = centre) sont converties à
//     l'entrée par `rectOfImage` : plus aucun site d'appel ne jongle avec deux
//     conventions ;
//   • un seuil exprimé en PIXELS ÉCRAN (`SNAP_SCREEN_PX`) divisé par l'échelle
//     du viewport → l'aimant a la même force à tous les zooms ;
//   • trois primitives couvrant TOUS les gestes : `snapMove` (déplacement),
//     `snapResize` (mise à l'échelle par une poignée) et `snapPoint`
//     (placement d'un élément pas encore créé, coin d'une zone en cours de
//     tracé…).
//
// Tout est pur et testé (`smartAlign.test.ts`). Les helpers qui lisent le store
// et poussent les guides à l'écran vivent dans `smartAlignRuntime.ts`.
// ────────────────────────────────────────────────────────────────────────────

import type { Annotation, Board, BoardImage, CanvasFolder } from "../types";

/** Boîte alignable, en coordonnées MONDE, ancrée au coin haut-gauche. */
export interface AlignRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

export type AlignKind = "image" | "text" | "sticky" | "membrane" | "folder";

/** Un élément du board contre lequel on peut s'aligner. */
export interface AlignTarget {
  id: string;
  kind: AlignKind;
  rect: AlignRect;
}

/** Lignes de guidage à afficher (coordonnées monde). Même forme que `store.guides`. */
export interface SnapGuides {
  x?: number[];
  y?: number[];
}

/** Distance d'accrochage, en pixels ÉCRAN (constante à tous les zooms). */
export const SNAP_SCREEN_PX = 8;

/** Taille de repli d'une annotation dont le rendu n'a pas encore été mesuré
 *  (`syncAnnotationSize` renseigne width/height dès le premier rendu). */
export const DEFAULT_ANN_W = 200;
export const DEFAULT_ANN_H = 100;

export interface SnapOptions {
  /** Échelle du viewport (monde → écran). Défaut 1. */
  scale?: number;
  /** Seuil d'accrochage en px écran. Défaut `SNAP_SCREEN_PX`. */
  thresholdPx?: number;
  /** Désactive un axe (ex. resize à ratio verrouillé piloté sur un seul axe). */
  axes?: { x?: boolean; y?: boolean };
}

export interface MoveSnap {
  /** Correction à AJOUTER au déplacement proposé (0 si rien n'a accroché). */
  dx: number;
  dy: number;
  guides: SnapGuides;
}

export interface ResizeSnap {
  rect: AlignRect;
  guides: SnapGuides;
}

export interface PointSnap {
  x: number;
  y: number;
  guides: SnapGuides;
}

// ── Conversions vers le modèle unique ───────────────────────────────────────

/** Les sprites images ont `x,y` = CENTRE (anchor 0.5) — on ramène au coin. */
export function rectOfImage(img: Pick<BoardImage, "x" | "y" | "width" | "height">): AlignRect {
  return {
    left: img.x - img.width / 2,
    top: img.y - img.height / 2,
    width: img.width,
    height: img.height,
  };
}

/** Les annotations ont `x,y` = COIN haut-gauche. Les flèches (segments x2/y2)
 *  n'ont pas de boîte → non alignables, `null`. */
export function rectOfAnnotation(ann: Annotation): AlignRect | null {
  if (ann.type === "arrow") return null;
  return {
    left: ann.x,
    top: ann.y,
    width: ann.width || DEFAULT_ANN_W,
    height: ann.height || DEFAULT_ANN_H,
  };
}

export function rectOfFolder(f: Pick<CanvasFolder, "x" | "y" | "width" | "height">): AlignRect {
  return { left: f.x, top: f.y, width: f.width, height: f.height };
}

/** Rectangle englobant d'un lot de boîtes (déplacement d'une multi-sélection). */
export function unionRect(rects: AlignRect[]): AlignRect | null {
  if (!rects.length) return null;
  let l = Infinity, t = Infinity, r = -Infinity, b = -Infinity;
  for (const rc of rects) {
    l = Math.min(l, rc.left);
    t = Math.min(t, rc.top);
    r = Math.max(r, rc.left + rc.width);
    b = Math.max(b, rc.top + rc.height);
  }
  return { left: l, top: t, width: r - l, height: b - t };
}

/**
 * Toutes les cibles d'alignement d'un board : images, annotations (texte, note,
 * membrane — pas les flèches) ET dossiers. `exclude` retire les éléments qu'on
 * est en train de bouger (sinon ils s'aligneraient sur eux-mêmes).
 */
export function collectAlignTargets(
  board: Pick<Board, "images" | "annotations"> & { folders?: CanvasFolder[] },
  exclude?: Iterable<string>,
): AlignTarget[] {
  const skip = exclude instanceof Set ? exclude : new Set(exclude ?? []);
  const out: AlignTarget[] = [];

  for (const img of board.images) {
    if (skip.has(img.id)) continue;
    out.push({ id: img.id, kind: "image", rect: rectOfImage(img) });
  }
  for (const ann of board.annotations) {
    if (skip.has(ann.id)) continue;
    const rect = rectOfAnnotation(ann);
    if (!rect) continue;
    out.push({ id: ann.id, kind: ann.type as AlignKind, rect });
  }
  for (const f of board.folders ?? []) {
    if (skip.has(f.id)) continue;
    out.push({ id: f.id, kind: "folder", rect: rectOfFolder(f) });
  }
  return out;
}

// ── Noyau : recherche de la meilleure accroche sur un axe ───────────────────

/** Lignes verticales d'une cible : bord gauche, centre, bord droit. */
function targetLinesX(rect: AlignRect): [number, number, number] {
  return [rect.left, rect.left + rect.width / 2, rect.left + rect.width];
}
/** Lignes horizontales d'une cible : bord haut, centre, bord bas. */
function targetLinesY(rect: AlignRect): [number, number, number] {
  return [rect.top, rect.top + rect.height / 2, rect.top + rect.height];
}

interface AxisSnap {
  /** Correction à ajouter à la position courante. */
  delta: number;
  /** Coordonnée monde de la ligne accrochée (= le guide à afficher). */
  line: number;
}

/**
 * Cherche, parmi les lignes `mine` de l'élément mobile, celle qui tombe le plus
 * près d'une ligne `theirs` d'une cible. `mine` est ordonné par PRIORITÉ : à
 * distance égale, la première l'emporte (on passe le centre en tête pour les
 * déplacements — c'est l'alignement que l'œil attend).
 */
function bestAxisSnap(mine: number[], theirs: number[], threshold: number): AxisSnap | null {
  let best: AxisSnap | null = null;
  let bestDist = threshold;
  for (const m of mine) {
    for (const t of theirs) {
      const d = t - m;
      const dist = Math.abs(d);
      if (dist < bestDist) {
        bestDist = dist;
        best = { delta: d, line: t };
      }
    }
  }
  return best;
}

function thresholdOf(opts: SnapOptions): number {
  const scale = opts.scale && opts.scale > 0 ? opts.scale : 1;
  return (opts.thresholdPx ?? SNAP_SCREEN_PX) / scale;
}

function axisEnabled(opts: SnapOptions, axis: "x" | "y"): boolean {
  return opts.axes?.[axis] !== false;
}

// ── Primitives publiques ────────────────────────────────────────────────────

/**
 * DÉPLACEMENT — `rect` est la boîte À LA POSITION PROPOSÉE (déjà translatée du
 * geste). Retourne la correction `dx/dy` à ajouter pour coller aux cibles.
 * Fonctionne aussi bien pour un élément seul que pour la boîte englobante
 * d'une multi-sélection (cf. `unionRect`).
 */
export function snapMove(rect: AlignRect, targets: AlignTarget[], opts: SnapOptions = {}): MoveSnap {
  const threshold = thresholdOf(opts);
  const cx = rect.left + rect.width / 2;
  const cy = rect.top + rect.height / 2;
  // Centre en tête : à distance égale, on préfère un alignement de centres.
  const mineX = [cx, rect.left, rect.left + rect.width];
  const mineY = [cy, rect.top, rect.top + rect.height];

  const theirsX: number[] = [];
  const theirsY: number[] = [];
  for (const t of targets) {
    theirsX.push(...targetLinesX(t.rect));
    theirsY.push(...targetLinesY(t.rect));
  }

  const sx = axisEnabled(opts, "x") ? bestAxisSnap(mineX, theirsX, threshold) : null;
  const sy = axisEnabled(opts, "y") ? bestAxisSnap(mineY, theirsY, threshold) : null;

  return {
    dx: sx?.delta ?? 0,
    dy: sy?.delta ?? 0,
    guides: {
      x: sx ? [sx.line] : undefined,
      y: sy ? [sy.line] : undefined,
    },
  };
}

/** Poignée de redimensionnement. `t/b/l/r` = un seul bord, les paires = un coin. */
export type ResizeHandle = "tl" | "tr" | "bl" | "br" | "t" | "b" | "l" | "r";

export interface ResizeOptions extends SnapOptions {
  minWidth?: number;
  minHeight?: number;
}

/**
 * MISE À L'ÉCHELLE — `rect` est la boîte À LA TAILLE PROPOSÉE. Seuls les bords
 * que la poignée déplace peuvent accrocher (tirer le coin bas-droit ne bouge
 * jamais les bords haut/gauche : ils ne produisent donc aucun guide). Une
 * accroche qui violerait `minWidth/minHeight` est abandonnée plutôt que clampée
 * — sinon le guide affiché mentirait sur la position réelle du bord.
 */
export function snapResize(
  rect: AlignRect,
  handle: ResizeHandle,
  targets: AlignTarget[],
  opts: ResizeOptions = {},
): ResizeSnap {
  const threshold = thresholdOf(opts);
  const minW = opts.minWidth ?? 1;
  const minH = opts.minHeight ?? 1;

  const movesLeft = handle.includes("l");
  const movesRight = handle.includes("r");
  const movesTop = handle.includes("t");
  const movesBottom = handle.includes("b");

  const theirsX: number[] = [];
  const theirsY: number[] = [];
  for (const t of targets) {
    theirsX.push(...targetLinesX(t.rect));
    theirsY.push(...targetLinesY(t.rect));
  }

  let { left, top, width, height } = rect;
  const guides: SnapGuides = {};

  if (axisEnabled(opts, "x")) {
    const mineX: number[] = [];
    if (movesLeft) mineX.push(left);
    if (movesRight) mineX.push(left + width);
    const s = mineX.length ? bestAxisSnap(mineX, theirsX, threshold) : null;
    if (s) {
      if (movesLeft) {
        const right = left + width;
        const nw = right - s.line;
        if (nw >= minW) { left = s.line; width = nw; guides.x = [s.line]; }
      } else {
        const nw = s.line - left;
        if (nw >= minW) { width = nw; guides.x = [s.line]; }
      }
    }
  }

  if (axisEnabled(opts, "y")) {
    const mineY: number[] = [];
    if (movesTop) mineY.push(top);
    if (movesBottom) mineY.push(top + height);
    const s = mineY.length ? bestAxisSnap(mineY, theirsY, threshold) : null;
    if (s) {
      if (movesTop) {
        const bottom = top + height;
        const nh = bottom - s.line;
        if (nh >= minH) { top = s.line; height = nh; guides.y = [s.line]; }
      } else {
        const nh = s.line - top;
        if (nh >= minH) { height = nh; guides.y = [s.line]; }
      }
    }
  }

  return { rect: { left, top, width, height }, guides };
}

/**
 * POINT — accroche un point isolé : position de POSE d'un élément pas encore
 * créé, coin d'une membrane/d'un dossier en cours de tracé, curseur d'un resize
 * d'image à ratio verrouillé. C'est ce qui manquait pour que le snap serve AUSSI
 * au premier placement et pas seulement à la retouche.
 */
export function snapPoint(x: number, y: number, targets: AlignTarget[], opts: SnapOptions = {}): PointSnap {
  const r = snapMove({ left: x, top: y, width: 0, height: 0 }, targets, opts);
  return { x: x + r.dx, y: y + r.dy, guides: r.guides };
}

// ── Utilitaire d'affichage ──────────────────────────────────────────────────

function sameLines(a?: number[], b?: number[]): boolean {
  if (!a || !a.length) return !b || !b.length;
  if (!b || a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (Math.abs(a[i] - b[i]) > 0.01) return false;
  }
  return true;
}

/** Deux jeux de guides sont-ils visuellement identiques ? Sert à ne PAS
 *  re-rendre l'overlay à chaque frame de drag quand rien n'a changé. */
export function sameGuides(a: SnapGuides | null, b: SnapGuides | null): boolean {
  const ax = a?.x, ay = a?.y, bx = b?.x, by = b?.y;
  const aEmpty = (!ax || !ax.length) && (!ay || !ay.length);
  const bEmpty = (!bx || !bx.length) && (!by || !by.length);
  if (aEmpty || bEmpty) return aEmpty && bEmpty;
  return sameLines(ax, bx) && sameLines(ay, by);
}

/** `null` si aucune ligne — le store attend `null` pour « pas de guide ». */
export function normalizeGuides(g: SnapGuides): SnapGuides | null {
  const hasX = !!g.x?.length;
  const hasY = !!g.y?.length;
  if (!hasX && !hasY) return null;
  return { x: hasX ? g.x : undefined, y: hasY ? g.y : undefined };
}
