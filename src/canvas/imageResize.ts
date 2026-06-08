// ────────────────────────────────────────────────────────────────────────────
// Redimensionnement d'image — géométrie PURE (testable, sans PixiJS).
//
// Les images ont `x,y` = CENTRE (ancre sprite 0.5). Deux modes (cf. style.md) :
//   • défaut    → ancrage au COIN OPPOSÉ à la poignée tirée (le coin diagonalement
//                 opposé reste fixe) ; le centre se déplace.
//   • fromCenter → ancrage au CENTRE (croît symétriquement) ; le centre ne bouge pas.
// Ratio TOUJOURS verrouillé (jamais de déformation), taille minimale plancher.
// ────────────────────────────────────────────────────────────────────────────

export interface ResizeState {
  /** Centre de l'image au moment du grab (utilisé en mode fromCenter). */
  cx: number;
  cy: number;
  /** Ratio largeur/hauteur verrouillé. */
  aspect: number;
  /** Coin OPPOSÉ à la poignée tirée (ancre fixe par défaut), en coords monde. */
  ax: number;
  ay: number;
}

export interface ResizeResult {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Plancher de taille (monde) — évite les images dégénérées. */
export const MIN_IMAGE_SIZE = 24;

/** Verrouille le ratio en retenant l'axe qui donne la plus grande boîte. */
function lockAspect(width: number, height: number, aspect: number): [number, number] {
  if (width / aspect >= height) return [width, width / aspect];
  return [height * aspect, height];
}

/**
 * Calcule la nouvelle géométrie d'une image redimensionnée par une poignée de
 * coin amenée en `(wx, wy)` (coords monde).
 *
 * @param fromCenter true = ancrage centre (Ctrl) ; false = ancrage coin opposé.
 */
export function computeResize(
  s: ResizeState,
  wx: number,
  wy: number,
  fromCenter: boolean,
): ResizeResult {
  if (fromCenter) {
    let [width, height] = lockAspect(Math.abs(wx - s.cx) * 2, Math.abs(wy - s.cy) * 2, s.aspect);
    if (width < MIN_IMAGE_SIZE) { width = MIN_IMAGE_SIZE; height = MIN_IMAGE_SIZE / s.aspect; }
    return { x: s.cx, y: s.cy, width, height };
  }
  let [width, height] = lockAspect(Math.abs(wx - s.ax), Math.abs(wy - s.ay), s.aspect);
  if (width < MIN_IMAGE_SIZE) { width = MIN_IMAGE_SIZE; height = MIN_IMAGE_SIZE / s.aspect; }
  // Le centre se replace à mi-chemin entre l'ancre (coin opposé) et le coin tiré,
  // du bon côté de l'ancre selon la position du curseur.
  const sx = wx >= s.ax ? 1 : -1;
  const sy = wy >= s.ay ? 1 : -1;
  return { x: s.ax + (sx * width) / 2, y: s.ay + (sy * height) / 2, width, height };
}
