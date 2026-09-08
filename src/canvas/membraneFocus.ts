// ────────────────────────────────────────────────────────────────────────────
// MEMB-2 — Mode Focus : « zoomer assez sur une membrane et n'avoir plus qu'elle ».
//
// Ce module décide TOUT et n'affiche RIEN : quand entrer, quand sortir, où
// cadrer la caméra, et qui reste visible. L'appelant n'a plus qu'à appliquer.
// C'est délibéré — une bascule de vue automatique est exactement le genre de
// chose qui clignote, et un clignotement se démontre absent bien plus sûrement
// qu'il ne s'observe à l'œil.
//
// POURQUOI ÇA NE PEUT PAS OSCILLER. Le piège est classique : on entre en focus
// parce que la membrane couvre l'écran, on recadre la caméra pour la faire
// tenir — et le recadrage, en ajoutant des marges, fait retomber la couverture
// sous le seuil d'entrée, donc on ressort, donc on rentre… Les conditions
// d'entrée et de sortie sont donc ASYMÉTRIQUES et ne portent pas sur la même
// grandeur :
//
//   • entrée  — la membrane couvre ≥ ENTER_COVERAGE de l'écran ;
//   • sortie  — on a dézoomé de EXIT_SCALE_RATIO depuis l'échelle du CADRAGE,
//               ou le centre de l'écran a quitté la membrane.
//
// Une fois entré, la couverture n'est plus jamais consultée : le recadrage ne
// peut donc pas provoquer sa propre annulation. Et après une sortie, l'échelle
// est plus basse qu'au cadrage, donc la couverture est sous le seuil d'entrée —
// pas de ré-entrée immédiate non plus. Un temps mort couvre le reste.
//
// L'ÉCHELLE DU CONTENU EST NEUTRALISÉE AILLEURS (cf. `membraneSpace`, option
// `focusedMembraneId`) : le focus n'a rien à faire pour que le contenu d'une
// membrane minimisée retrouve sa taille réelle, il suffit de ne pas appliquer
// la réduction. C'est ce qui tient la promesse « en focus on ne s'aperçoit de
// rien » sans jamais toucher aux données.
// ────────────────────────────────────────────────────────────────────────────

import {
  containedIn,
  contentExtent,
  parentMap,
  type Box,
  type ResolvedItem,
  type SpaceItem,
} from "./membraneSpace";
import type { Annotation, Board, CanvasFolder } from "../types";

export const FOCUS = {
  /** Part de l'écran que la membrane doit couvrir pour déclencher le focus.
   *  Très haut volontairement : on entre quand elle DÉBORDE de l'écran, donc au
   *  moment où ce qui l'entoure n'était déjà quasiment plus visible. La bascule
   *  est ainsi imperceptible. */
  ENTER_COVERAGE: 0.92,
  /** On sort après avoir dézoomé de ce facteur depuis l'échelle du cadrage. */
  EXIT_SCALE_RATIO: 0.8,
  /** Débordement toléré du centre de l'écran hors de la membrane avant sortie,
   *  en fraction de la boîte. Évite de sortir pour un pan d'un cheveu. */
  EXIT_CENTER_MARGIN: 0.35,
  /** Temps mort après une transition. Filet de sécurité : la démonstration
   *  ci-dessus rend l'oscillation impossible, ceci la rend impensable. */
  COOLDOWN_MS: 400,
  /** Air laissé autour de la membrane au cadrage, par côté. */
  FIT_PADDING: 0.06,
  /** Durée du glissement de caméra à l'entrée en focus. DOIT rester STRICTEMENT
   *  inférieure à COOLDOWN_MS : aucune décision ne doit être prise pendant
   *  l'animation, sinon le cadrage pourrait déclencher sa propre annulation.
   *  Un test verrouille cette relation. */
  FIT_ANIM_MS: 320,
} as const;

export interface Viewport {
  x: number;
  y: number;
  scale: number;
}

export interface ScreenSize {
  width: number;
  height: number;
}

export interface FocusState {
  membraneId: string | null;
  /** Échelle atteinte APRÈS cadrage — la référence de la condition de sortie. */
  enterScale: number;
  /** Horodatage de la dernière transition (temps mort). */
  t: number;
}

export const NO_FOCUS: FocusState = { membraneId: null, enterScale: 0, t: 0 };

// ════════════════════════════════════════════════════════════════════════════
// Géométrie écran
// ════════════════════════════════════════════════════════════════════════════

/** Boîte monde → boîte écran, sous un viewport donné. */
export function toScreen(b: Box, vp: Viewport): Box {
  return {
    x: b.x * vp.scale + vp.x,
    y: b.y * vp.scale + vp.y,
    width: b.width * vp.scale,
    height: b.height * vp.scale,
  };
}

/** Fraction de l'écran que recouvre `b`. 1 = elle déborde de partout. */
export function coverage(b: Box, vp: Viewport, screen: ScreenSize): number {
  const total = screen.width * screen.height;
  if (total <= 0) return 0;
  const s = toScreen(b, vp);
  const ix = Math.max(0, Math.min(s.x + s.width, screen.width) - Math.max(s.x, 0));
  const iy = Math.max(0, Math.min(s.y + s.height, screen.height) - Math.max(s.y, 0));
  return (ix * iy) / total;
}

/** Point monde au centre de l'écran. */
export function screenCenterWorld(vp: Viewport, screen: ScreenSize): { x: number; y: number } {
  return {
    x: (screen.width / 2 - vp.x) / vp.scale,
    y: (screen.height / 2 - vp.y) / vp.scale,
  };
}

/**
 * Boîte à cadrer pour une membrane focalisée.
 *
 * Sous focus le contenu reprend sa taille naturelle : une membrane minimisée
 * doit donc s'agrandir pour le contenir, sinon il déborderait de son propre
 * cadre. C'est le « la membrane s'étire complètement » — un simple max, la
 * membrane ne rétrécit jamais.
 */
export function focusBox(membrane: SpaceItem, children: SpaceItem[]): Box {
  const ext = contentExtent(membrane, children);
  return {
    x: membrane.x,
    y: membrane.y,
    width: Math.max(membrane.width, ext.w),
    height: Math.max(membrane.height, ext.h),
  };
}

/** Viewport qui fait tenir `b` dans l'écran, centré, avec une marge. */
export function fitViewport(
  b: Box,
  screen: ScreenSize,
  pad: number = FOCUS.FIT_PADDING,
): Viewport {
  const usableW = screen.width * (1 - pad * 2);
  const usableH = screen.height * (1 - pad * 2);
  const scale = Math.min(
    usableW / Math.max(1e-6, b.width),
    usableH / Math.max(1e-6, b.height),
  );
  return {
    scale,
    x: screen.width / 2 - (b.x + b.width / 2) * scale,
    y: screen.height / 2 - (b.y + b.height / 2) * scale,
  };
}

// ════════════════════════════════════════════════════════════════════════════
// Décision
// ════════════════════════════════════════════════════════════════════════════

export interface FocusInput {
  items: SpaceItem[];
  /** Géométrie effective courante (hors focus), pour juger de la couverture. */
  resolved: Map<string, ResolvedItem>;
  vp: Viewport;
  screen: ScreenSize;
  state: FocusState;
  now: number;
}

export type FocusAction =
  | { kind: "stay"; state: FocusState }
  | { kind: "enter"; state: FocusState; membraneId: string; fit: Viewport }
  | { kind: "exit"; state: FocusState };

function boxOf(r: ResolvedItem): Box {
  return { x: r.x, y: r.y, width: r.width, height: r.height };
}

/**
 * Faut-il entrer, sortir, ou ne rien faire. Fonction TOTALE et déterministe :
 * même entrée, même sortie — c'est ce qui rend l'absence d'oscillation
 * démontrable plutôt qu'observable.
 */
export function focusDecision(input: FocusInput): FocusAction {
  const { items, resolved, vp, screen, state, now } = input;

  if (now - state.t < FOCUS.COOLDOWN_MS) return { kind: "stay", state };

  // ── Déjà en focus : seule la sortie est évaluée ─────────────────────────
  if (state.membraneId) {
    const membrane = items.find((i) => i.id === state.membraneId && i.kind === "membrane");
    if (!membrane) return { kind: "exit", state: { ...NO_FOCUS, t: now } };

    if (vp.scale < state.enterScale * FOCUS.EXIT_SCALE_RATIO) {
      return { kind: "exit", state: { ...NO_FOCUS, t: now } };
    }

    // Le centre de l'écran a-t-il quitté la membrane ? On raisonne sur la boîte
    // de focus (contenu compris), pas sur la boîte nue.
    const kids = childrenOf(items, membrane.id);
    const fb = focusBox(membrane, kids);
    const c = screenCenterWorld(vp, screen);
    const mx = fb.width * FOCUS.EXIT_CENTER_MARGIN;
    const my = fb.height * FOCUS.EXIT_CENTER_MARGIN;
    const inside = c.x >= fb.x - mx && c.x <= fb.x + fb.width + mx
      && c.y >= fb.y - my && c.y <= fb.y + fb.height + my;
    if (!inside) return { kind: "exit", state: { ...NO_FOCUS, t: now } };

    return { kind: "stay", state };
  }

  // ── Hors focus : chercher la membrane qui remplit l'écran ───────────────
  const center = screenCenterWorld(vp, screen);
  let best: SpaceItem | null = null;
  let bestArea = Number.POSITIVE_INFINITY;

  for (const it of items) {
    if (it.kind !== "membrane") continue;
    const r = resolved.get(it.id);
    if (!r) continue;
    const b = boxOf(r);
    if (center.x < b.x || center.x > b.x + b.width) continue;
    if (center.y < b.y || center.y > b.y + b.height) continue;
    if (coverage(b, vp, screen) < FOCUS.ENTER_COVERAGE) continue;
    // La plus PETITE gagne — même règle que la priorité au clic et que
    // l'appartenance : une seule notion de « dedans » dans toute l'app.
    const a = Math.abs(b.width * b.height);
    if (a < bestArea) { best = it; bestArea = a; }
  }

  if (!best) return { kind: "stay", state };

  const fit = fitViewport(focusBox(best, childrenOf(items, best.id)), screen);
  // enterScale = l'échelle APRÈS cadrage : c'est ce qui empêche le recadrage de
  // provoquer sa propre annulation.
  return {
    kind: "enter",
    membraneId: best.id,
    fit,
    state: { membraneId: best.id, enterScale: fit.scale, t: now },
  };
}

function childrenOf(items: SpaceItem[], membraneId: string): SpaceItem[] {
  const parents = parentMap(items);
  return items.filter((i) => parents.get(i.id) === membraneId);
}

/** Boîte monde cadrée par le focus, ou `null` hors focus. */
export function focusFrameOf(items: SpaceItem[], focusedId: string | null): Box | null {
  if (!focusedId) return null;
  const m = items.find((i) => i.id === focusedId && i.kind === "membrane");
  if (!m) return null;
  return focusBox(m, childrenOf(items, focusedId));
}

// ════════════════════════════════════════════════════════════════════════════
// Visibilité
// ════════════════════════════════════════════════════════════════════════════

/**
 * Ids visibles sous focus : la membrane et TOUTE sa descendance.
 *
 * `null` signifie « aucun filtre » (hors focus) — un `Set` vide voudrait dire
 * « rien n'est visible », ce qui n'est jamais l'intention.
 *
 * Le filet géométrique existe pour les projets d'AVANT la fonctionnalité : ils
 * n'ont aucun `membraneId` stocké, et sans lui focaliser une membrane
 * historique afficherait un écran vide. Sous focus l'échelle vaut 1, donc
 * naturel == effectif et l'inclusion géométrique est exacte.
 */
export function visibleUnderFocus(
  items: SpaceItem[],
  focusedId: string | null,
): Set<string> | null {
  if (!focusedId) return null;
  const membrane = items.find((i) => i.id === focusedId);
  if (!membrane) return null;

  const parents = parentMap(items);
  const kidsOf = new Map<string, SpaceItem[]>();
  for (const it of items) {
    const p = parents.get(it.id);
    if (!p) continue;
    const l = kidsOf.get(p);
    if (l) l.push(it);
    else kidsOf.set(p, [it]);
  }

  const out = new Set<string>([focusedId]);
  const stack = [focusedId];
  while (stack.length > 0) {
    const id = stack.pop()!;
    for (const kid of kidsOf.get(id) ?? []) {
      if (out.has(kid.id)) continue;
      out.add(kid.id);
      stack.push(kid.id);
    }
  }

  for (const it of containedIn(items, membrane)) {
    if (!parents.has(it.id)) out.add(it.id);
  }
  return out;
}

/**
 * Une flèche est-elle visible sous focus ?
 *
 * Elle n'a pas de boîte propre : elle suit les nœuds qu'elle relie. Attachée,
 * elle est visible si TOUS ses nœuds le sont — une flèche à moitié dehors
 * pointerait vers le vide. Libre, on juge ses deux extrémités.
 */
export function arrowVisibleUnderFocus(
  ann: Annotation,
  visible: Set<string> | null,
  focus: Box | null,
): boolean {
  if (!visible || !focus) return true;
  if (ann.type !== "arrow") return true;

  const ends = [ann.sourceId, ann.targetId].filter((v): v is string => !!v);
  if (ends.length > 0) return ends.every((id) => visible.has(id));

  const inBox = (x: number, y: number) =>
    x >= focus.x && x <= focus.x + focus.width && y >= focus.y && y <= focus.y + focus.height;
  return inBox(ann.x, ann.y) && inBox(ann.x2, ann.y2);
}

/**
 * Une annotation quelconque est-elle visible sous focus ? Point d'entrée unique
 * des couches : elles n'ont plus qu'à filtrer.
 *
 * Le cas subtil est le bloc PAS ENCORE MESURÉ. Sans largeur ni hauteur il
 * n'entre pas dans le jeu de boîtes, donc il n'a aucune appartenance calculable
 * et le filtre le ferait disparaître — le temps d'une image, à chaque création
 * de texte. On se rabat alors sur son point d'ancrage. C'est transitoire : le
 * ResizeObserver fournit la taille au tour suivant.
 */
export function annotationVisibleUnderFocus(
  ann: Annotation,
  visible: Set<string> | null,
  frame: Box | null,
): boolean {
  if (!visible) return true;
  if (ann.type === "arrow") return arrowVisibleUnderFocus(ann, visible, frame);
  if (visible.has(ann.id)) return true;

  const measured = (ann.width ?? 0) > 0 && (ann.height ?? 0) > 0;
  if (measured || !frame) return false;
  return ann.x >= frame.x && ann.x <= frame.x + frame.width
    && ann.y >= frame.y && ann.y <= frame.y + frame.height;
}

// ════════════════════════════════════════════════════════════════════════════
// Fond de scène
// ════════════════════════════════════════════════════════════════════════════

function parseHex(c: string): [number, number, number] | null {
  const m = c.trim().replace(/^#/, "");
  if (/^[0-9a-fA-F]{3}$/.test(m)) {
    return [
      parseInt(m[0] + m[0], 16),
      parseInt(m[1] + m[1], 16),
      parseInt(m[2] + m[2], 16),
    ];
  }
  if (/^[0-9a-fA-F]{6}$/.test(m)) {
    return [
      parseInt(m.slice(0, 2), 16),
      parseInt(m.slice(2, 4), 16),
      parseInt(m.slice(4, 6), 16),
    ];
  }
  return null;
}

/**
 * Fond de scène en mode Focus : « on aura la couleur de fond de cette membrane ».
 *
 * Teinte, pas aplat : à pleine saturation la couleur écraserait le contenu, qui
 * est justement ce qu'on vient regarder. On mélange donc une petite part de la
 * couleur de la membrane au noir du canvas — assez pour que la région se
 * reconnaisse d'un coup d'œil, pas assez pour gêner la lecture.
 *
 * Toute couleur illisible retombe sur le fond habituel : le focus ne doit jamais
 * pouvoir noircir ou blanchir la scène par accident.
 */
export function focusBackground(color: string | null, base = "#0d0d0d", amount = 0.16): string {
  if (!color) return base;
  const c = parseHex(color);
  const b = parseHex(base);
  if (!c || !b) return base;
  const t = Math.min(1, Math.max(0, amount));
  const mix = (i: number) => Math.round(b[i] + (c[i] - b[i]) * t);
  const hex = (v: number) => v.toString(16).padStart(2, "0");
  return `#${hex(mix(0))}${hex(mix(1))}${hex(mix(2))}`;
}

// ════════════════════════════════════════════════════════════════════════════
// Vue focalisée — ce que les couches reçoivent
// ════════════════════════════════════════════════════════════════════════════

export interface FocusView {
  /** Ids visibles, ou `null` hors focus (aucun filtre). */
  visible: Set<string> | null;
  /** Cadre de la membrane focalisée, contenu compris. */
  frame: Box | null;
  /** Annotations à rendre. */
  annotations: Annotation[];
  /** Dossiers à rendre — aucun sous focus : ce sont des portes, pas du contenu. */
  folders: CanvasFolder[];
}

/**
 * Tout ce que le rendu doit savoir du focus, en un appel.
 *
 * Regroupé ici plutôt qu'éparpillé dans le canvas pour une raison précise : le
 * test d'intégration exerce ainsi EXACTEMENT le code que l'application exécute,
 * et non une réimplémentation qui pourrait diverger sans que rien ne le dise.
 *
 * Hors focus, tout est rendu à l'identique et sans copie de tableau.
 */
export function focusView(
  board: Pick<Board, "images" | "annotations"> & { folders?: CanvasFolder[] },
  items: SpaceItem[],
  focusedId: string | null,
): FocusView {
  const visible = visibleUnderFocus(items, focusedId);
  if (!visible) {
    return { visible: null, frame: null, annotations: board.annotations, folders: board.folders ?? [] };
  }
  const frame = focusFrameOf(items, focusedId);
  return {
    visible,
    frame,
    annotations: board.annotations.filter((a) => annotationVisibleUnderFocus(a, visible, frame)),
    folders: [],
  };
}
