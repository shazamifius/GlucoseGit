// ────────────────────────────────────────────────────────────────────────────
// NAV — logique pure de navigation adaptative entre dossiers (R-FIL).
//
// Extrait de GlucoseCanvas.tsx pour être TESTABLE et garder le canvas léger.
// Aucune dépendance PixiJS/React ici : juste de la géométrie.
// ────────────────────────────────────────────────────────────────────────────

export interface FolderBox {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface NavViewport {
  /** Translation monde→écran en X (px écran). */
  x: number;
  /** Translation monde→écran en Y (px écran). */
  y: number;
  /** Échelle courante. */
  scale: number;
}

/** Rectangle visible (en coordonnées MONDE) pour un viewport + une taille écran. */
export function visibleWorldRect(vp: NavViewport, screenW: number, screenH: number) {
  return {
    left: (0 - vp.x) / vp.scale,
    top: (0 - vp.y) / vp.scale,
    right: (screenW - vp.x) / vp.scale,
    bottom: (screenH - vp.y) / vp.scale,
  };
}

/**
 * NAV-1 — Décide si le zoom courant doit faire ENTRER dans un dossier.
 *
 * Règle (demande user) : on entre dans un dossier **uniquement quand il est le
 * SEUL dossier frère visible à l'écran** (aucun autre ne croise le viewport) ET
 * qu'il remplit au moins `minCoverage` d'une dimension écran.
 *
 * Pourquoi : tant que 2 dossiers (ou +) sont visibles, il y a ambiguïté sur
 * « lequel » → on ne plonge jamais au hasard. Et comme le critère est
 * géométrique, il tient compte de la **taille réelle** des dossiers : un gros
 * dossier déclenche à un zoom plus faible qu'un petit (fini le scale fixe).
 *
 * @returns l'id du dossier où entrer, ou `null` si on ne doit pas entrer.
 */
export function folderToEnter(
  folders: readonly FolderBox[],
  vp: NavViewport,
  screenW: number,
  screenH: number,
  minCoverage: number,
  minScale: number,
): string | null {
  if (vp.scale < minScale) return null;
  if (screenW <= 0 || screenH <= 0) return null;

  const { left, top, right, bottom } = visibleWorldRect(vp, screenW, screenH);

  // Dossiers dont la bbox croise (strictement) le rectangle visible.
  let only: FolderBox | null = null;
  let count = 0;
  for (const f of folders) {
    const intersects =
      f.x < right && f.x + f.width > left && f.y < bottom && f.y + f.height > top;
    if (intersects) {
      count++;
      if (count > 1) return null; // ≥ 2 dossiers visibles → ambigu, on n'entre pas.
      only = f;
    }
  }
  if (count !== 1 || !only) return null;

  // Plancher : le dossier doit remplir l'essentiel de l'écran. Sinon on
  // n'entre pas dans un petit dossier isolé juste parce qu'il est le seul visible.
  const covW = (only.width * vp.scale) / screenW;
  const covH = (only.height * vp.scale) / screenH;
  if (Math.max(covW, covH) < minCoverage) return null;

  return only.id;
}
