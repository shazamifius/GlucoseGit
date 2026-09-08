// ────────────────────────────────────────────────────────────────────────────
// SNAP-1 — Pont entre le moteur pur (`smartAlign.ts`) et l'application vivante.
//
// Toutes les couches (images PixiJS, textes/notes HTML, membranes SVG, dossiers
// SVG, outils de création) passent par ces helpers. Chacun :
//   1. respecte l'interrupteur `smartGuidesEnabled` (bouton du toolbar / touche) ;
//   2. collecte les cibles du board ACTIF en excluant les éléments manipulés ;
//   3. pousse les guides à l'écran (le store déduplique) ;
//   4. retourne un résultat NEUTRE quand le snap est coupé → aucun site d'appel
//      n'a de branche `if (smartEnabled)` à écrire.
//
// `endSnap()` referme le geste (efface les guides) — à appeler à chaque pointerup.
// ────────────────────────────────────────────────────────────────────────────

import { getActiveBoard, useGlucoseStore } from "../store";
import {
  collectAlignTargets,
  normalizeGuides,
  rectOfAnnotation,
  rectOfImage,
  snapMove,
  snapPoint,
  snapResize,
  unionRect,
  type AlignRect,
  type AlignTarget,
  type MoveSnap,
  type PointSnap,
  type ResizeHandle,
  type ResizeOptions,
  type ResizeSnap,
  type SnapGuides,
  type SnapOptions,
} from "./smartAlign";

// Mémo des cibles. `collectAlignTargets` est O(n) sur tout le board et on
// l'appelle à CHAQUE pointermove — y compris en simple survol, outil armé, où
// rien ne change. Invalidation : la référence `project`. Automerge rend un
// NOUVEAU document à chaque mutation, donc dès qu'un élément bouge, est créé ou
// supprimé (y compris par un pair en collaboration), la référence change et le
// cache saute. Pendant un drag, le projet mute à chaque frame → on recalcule
// comme avant (aucune régression) ; en survol, on ne recalcule plus rien.
let targetsCache: { project: unknown; boardId: string; key: string; targets: AlignTarget[] } | null = null;

/** Cibles du board actif, ou `null` si l'alignement intelligent est coupé. */
function liveTargets(exclude?: Iterable<string>): AlignTarget[] | null {
  const st = useGlucoseStore.getState();
  if (!st.smartGuidesEnabled) return null;
  const board = getActiveBoard(st.project);
  if (!board) return null;

  const key = exclude ? [...exclude].sort().join(",") : "";
  const c = targetsCache;
  if (c && c.project === st.project && c.boardId === board.id && c.key === key) return c.targets;

  const targets = collectAlignTargets(board, exclude);
  targetsCache = { project: st.project, boardId: board.id, key, targets };
  return targets;
}

function publish(guides: SnapGuides) {
  useGlucoseStore.getState().setGuides(normalizeGuides(guides));
}

/** Efface les guides — fin de geste (pointerup, Échap, changement d'outil). */
export function endSnap() {
  useGlucoseStore.getState().setGuides(null);
}

/**
 * Déplacement d'UN élément dont le site d'appel connaît la position ABSOLUE
 * proposée (dossiers, zones…). Retourne la position corrigée.
 */
export function snapMoveLive(
  rect: AlignRect,
  exclude: Iterable<string>,
  opts: SnapOptions,
): MoveSnap {
  const targets = liveTargets(exclude);
  if (!targets) { endSnap(); return { dx: 0, dy: 0, guides: {} }; }
  const res = snapMove(rect, targets, opts);
  publish(res.guides);
  return res;
}

/** Mise à l'échelle. `rect` = boîte à la taille PROPOSÉE (monde). */
export function snapResizeLive(
  rect: AlignRect,
  handle: ResizeHandle,
  exclude: Iterable<string>,
  opts: ResizeOptions,
): ResizeSnap {
  const targets = liveTargets(exclude);
  if (!targets) { endSnap(); return { rect, guides: {} }; }
  const res = snapResize(rect, handle, targets, opts);
  publish(res.guides);
  return res;
}

/** Point isolé : pose d'un élément pas encore créé, coin d'une zone en tracé,
 *  curseur d'un resize d'image à ratio verrouillé. */
export function snapPointLive(
  x: number,
  y: number,
  exclude: Iterable<string>,
  opts: SnapOptions,
): PointSnap {
  const targets = liveTargets(exclude);
  if (!targets) { endSnap(); return { x, y, guides: {} }; }
  const res = snapPoint(x, y, targets, opts);
  publish(res.guides);
  return res;
}

/**
 * Ne publie que les guides que la géométrie FINALE touche réellement.
 *
 * Sert au redimensionnement d'image, où le ratio est verrouillé : on accroche le
 * curseur sur les deux axes, mais le verrou de ratio n'en retient qu'un. Sans ce
 * filtre, on afficherait un trait sur un bord qui ne s'y trouve pas.
 */
export function publishConfirmedGuides(rect: AlignRect, guides: SnapGuides) {
  const EPS = 0.5;
  const near = (a: number, b: number) => Math.abs(a - b) < EPS;
  const x = guides.x?.filter((v) => near(v, rect.left) || near(v, rect.left + rect.width));
  const y = guides.y?.filter((v) => near(v, rect.top) || near(v, rect.top + rect.height));
  publish({ x, y });
}

// ── Session de déplacement d'une sélection ──────────────────────────────────

export interface SelectionSnapSession {
  /**
   * `rawDX/rawDY` = déplacement TOTAL du curseur depuis le grab (pas frame à
   * frame). Retourne le delta INCRÉMENTAL à passer à `moveSelected`.
   */
  move(rawDX: number, rawDY: number, opts?: SnapOptions): { dx: number; dy: number };
  /** Fin du geste : efface les guides. */
  end(): void;
}

/**
 * Ouvre une session de déplacement pour la sélection COURANTE (images +
 * annotations, la boîte englobante des deux). À appeler au pointerdown.
 *
 * Pourquoi une session plutôt qu'un appel par frame : l'ancienne mécanique
 * ajoutait la correction de snap au point de référence du drag (`pStartX +=`).
 * Chaque accroche décalait donc DÉFINITIVEMENT l'élément par rapport au
 * curseur, et les décalages s'accumulaient — l'élément « dérivait » sous la
 * souris au fil des aimantations. Ici on mémorise la boîte AU GRAB, on
 * repositionne toujours en ABSOLU depuis elle, et on ne renvoie que l'écart à
 * appliquer : sortir d'une zone d'accroche rend exactement la position du
 * curseur, sans dérive.
 */
export function beginSelectionSnap(): SelectionSnapSession {
  const st = useGlucoseStore.getState();
  const board = getActiveBoard(st.project);
  const ids = new Set<string>([...st.selectedImageIds, ...st.selectedAnnotationIds]);

  const rects: AlignRect[] = [];
  if (board) {
    for (const img of board.images) if (ids.has(img.id)) rects.push(rectOfImage(img));
    for (const ann of board.annotations) {
      if (!ids.has(ann.id)) continue;
      const r = rectOfAnnotation(ann); // les flèches (segments) n'ont pas de boîte
      if (r) rects.push(r);
    }
  }
  const base = unionRect(rects);

  let appliedX = 0;
  let appliedY = 0;

  return {
    move(rawDX, rawDY, opts = {}) {
      let targetX = rawDX;
      let targetY = rawDY;
      const targets = base ? liveTargets(ids) : null;
      if (targets && base) {
        const res = snapMove(
          { left: base.left + rawDX, top: base.top + rawDY, width: base.width, height: base.height },
          targets,
          opts,
        );
        targetX += res.dx;
        targetY += res.dy;
        publish(res.guides);
      } else {
        endSnap();
      }
      const dx = targetX - appliedX;
      const dy = targetY - appliedY;
      appliedX = targetX;
      appliedY = targetY;
      return { dx, dy };
    },
    end: endSnap,
  };
}
