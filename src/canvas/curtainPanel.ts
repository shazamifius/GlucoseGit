// ────────────────────────────────────────────────────────────────────────────
// MEMB-3 — Rideau : le panneau personnel qui se pose sur une membrane focalisée.
//
// Un rideau n'existe qu'en mode Focus (cf. `membraneFocus`) et nulle part
// ailleurs. Il est collé au BORD DROIT, façon console Quake ou Slide Over :
// il RECOUVRE le canvas focalisé, qui lui ne bouge jamais. La seule grandeur
// qui varie est la largeur du panneau.
//
//   replié   — une languette (≈ 1/10 de l'écran) reste visible à droite ;
//   déployé  — le panneau prend l'essentiel de l'écran, et il reste une bande
//              de canvas à gauche.
//
// Les DEUX sont donc toujours visibles, quel que soit l'état. C'est la règle
// qui borne les réglages : ni la languette ni la bande de canvas ne peuvent
// tomber à zéro (cf. `normalizeConfig`).
//
// Le basculement se fait au SURVOL, sans clic : la souris entre sur le panneau,
// il se déploie ; elle retourne sur le canvas, il se replie.
//
// POURQUOI CE SURVOL NE PEUT PAS BATTRE. Un survol qui redimensionne la zone
// qu'on survole est le cas d'école du clignotement : la frontière bouge, passe
// sous le curseur, inverse la décision, revient… Ici c'est impossible, et pas
// par réglage fin — par géométrie :
//
//   • se déployer déplace la frontière vers la GAUCHE. Un curseur qui était à
//     droite d'elle (donc sur le panneau, ce qui a déclenché le déploiement) y
//     reste, puisqu'elle s'en éloigne.
//   • se replier déplace la frontière vers la DROITE. Un curseur qui était à
//     gauche d'elle (donc sur le canvas) y reste, pour la même raison.
//
// L'animation RENFORCE donc toujours la condition qui l'a déclenchée. Le
// mouvement est auto-stabilisant, et `isOverPanel` le vérifie sur toutes les
// positions intermédiaires. Une temporisation s'ajoute par-dessus, non pas
// contre le battement mais contre l'ouverture accidentelle : la souris traverse
// souvent le bord droit de l'écran en travaillant.
// ────────────────────────────────────────────────────────────────────────────

export const CURTAIN = {
  /** Largeur du rideau replié, en fraction de l'écran — la « languette ». */
  DEFAULT_COLLAPSED: 0.1,
  /** Largeur une fois déployé ; le reste laisse voir le canvas focalisé. */
  DEFAULT_EXPANDED: 0.9,

  // Bornes de réglage. Elles garantissent l'invariant « les deux surfaces
  // restent visibles » : la languette ne disparaît jamais, la bande de canvas
  // non plus.
  MIN_COLLAPSED: 0.02,
  MAX_COLLAPSED: 0.3,
  MIN_EXPANDED: 0.3,
  MAX_EXPANDED: 0.97,
  /** Écart minimal entre replié et déployé — sinon le geste n'a plus de sens. */
  MIN_SPAN: 0.05,

  /** Temporisation avant déploiement. Contre l'ouverture accidentelle en
   *  frôlant le bord droit, pas contre le battement (impossible par géométrie). */
  EXPAND_DWELL_MS: 90,
  /** Repli plus vif : on veut récupérer son canvas sans attendre. */
  COLLAPSE_DWELL_MS: 40,
  /** Constante de temps du glissement. Lissage exponentiel : jamais de
   *  dépassement, et indépendant de la cadence d'images. */
  ANIM_MS: 260,
} as const;

export type CurtainPhase = "collapsed" | "expanded";

export interface CurtainConfig {
  /** Fraction d'écran occupée replié. */
  collapsed: number;
  /** Fraction d'écran occupée déployé. */
  expanded: number;
}

export interface CurtainState {
  /** Fraction d'écran occupée MAINTENANT (animée entre les deux réglages). */
  ratio: number;
  /** État visé. */
  phase: CurtainPhase;
  /** Bascule réclamée par le survol, en attente de temporisation. */
  pending: CurtainPhase | null;
  /** Instant où cette réclamation a commencé. */
  pendingSince: number;
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * Réglages valides, quoi qu'on lui donne.
 *
 * Garantit l'invariant du module : `0 < collapsed < expanded < 1`. Les deux
 * surfaces restent donc toujours visibles, y compris si un projet arrive avec
 * des valeurs aberrantes (fichier édité à la main, version future, bug amont).
 */
export function normalizeConfig(cfg?: Partial<CurtainConfig> | null): CurtainConfig {
  const collapsed = clamp(
    cfg?.collapsed ?? CURTAIN.DEFAULT_COLLAPSED,
    CURTAIN.MIN_COLLAPSED,
    CURTAIN.MAX_COLLAPSED,
  );
  const expanded = clamp(
    cfg?.expanded ?? CURTAIN.DEFAULT_EXPANDED,
    Math.max(CURTAIN.MIN_EXPANDED, collapsed + CURTAIN.MIN_SPAN),
    CURTAIN.MAX_EXPANDED,
  );
  return { collapsed, expanded };
}

function clamp(v: number, lo: number, hi: number): number {
  if (!Number.isFinite(v)) return lo;
  return Math.min(hi, Math.max(lo, v));
}

/** État initial : replié. */
export function initialState(cfg: CurtainConfig): CurtainState {
  return { ratio: cfg.collapsed, phase: "collapsed", pending: null, pendingSince: 0 };
}

// ════════════════════════════════════════════════════════════════════════════
// Géométrie
// ════════════════════════════════════════════════════════════════════════════

/** Rectangle du panneau, ancré au bord DROIT. */
export function panelRect(ratio: number, screen: { width: number; height: number }): Rect {
  const width = screen.width * ratio;
  return { x: screen.width - width, y: 0, width, height: screen.height };
}

/**
 * Bande de canvas focalisé restée visible à gauche du panneau.
 *
 * Définie comme le COMPLÉMENT EXACT du panneau, et non par `width * (1 - ratio)` :
 * en virgule flottante les deux ne se rejoignent pas toujours (1000 × (1 − 0,9)
 * vaut 99,999…) et il resterait un liseré d'un pixel entre les deux surfaces.
 */
export function canvasStrip(ratio: number, screen: { width: number; height: number }): Rect {
  return { x: 0, y: 0, width: screen.width - panelRect(ratio, screen).width, height: screen.height };
}

/** Le curseur est-il sur le panneau ? `null` = souris hors fenêtre. */
export function isOverPanel(
  cursorX: number | null,
  ratio: number,
  screen: { width: number },
): boolean {
  if (cursorX === null) return false;
  return cursorX >= screen.width * (1 - ratio);
}

// ════════════════════════════════════════════════════════════════════════════
// Décision & animation
// ════════════════════════════════════════════════════════════════════════════

/**
 * Met à jour la phase visée d'après la position du curseur, temporisation
 * comprise. N'anime rien — c'est `advance` qui déplace le panneau.
 *
 * Souris hors fenêtre (`cursorX === null`) : on se replie. Le rideau ne doit
 * pas rester ouvert derrière une fenêtre qu'on vient de quitter.
 */
export function decide(
  state: CurtainState,
  cursorX: number | null,
  screen: { width: number },
  now: number,
): CurtainState {
  const want: CurtainPhase = isOverPanel(cursorX, state.ratio, screen) ? "expanded" : "collapsed";

  if (want === state.phase) {
    return state.pending === null ? state : { ...state, pending: null, pendingSince: 0 };
  }

  if (state.pending !== want) {
    return { ...state, pending: want, pendingSince: now };
  }

  const dwell = want === "expanded" ? CURTAIN.EXPAND_DWELL_MS : CURTAIN.COLLAPSE_DWELL_MS;
  if (now - state.pendingSince < dwell) return state;

  return { ...state, phase: want, pending: null, pendingSince: 0 };
}

/**
 * Rapproche le panneau de sa cible. Lissage exponentiel : monotone, sans
 * dépassement, et le pas ne dépend pas de la cadence d'images — une image
 * sautée ne fait pas sauter le rideau.
 */
export function advance(state: CurtainState, cfg: CurtainConfig, dtMs: number): CurtainState {
  const target = state.phase === "expanded" ? cfg.expanded : cfg.collapsed;
  if (state.ratio === target) return state;

  const tau = CURTAIN.ANIM_MS / 3;
  const k = 1 - Math.exp(-Math.max(0, dtMs) / tau);
  let ratio = state.ratio + (target - state.ratio) * k;
  // Fin de course : on colle à la cible plutôt que de l'approcher indéfiniment.
  if (Math.abs(target - ratio) < 0.0005) ratio = target;
  return { ...state, ratio };
}

/** Un pas complet : décider, puis animer. */
export function step(
  state: CurtainState,
  cursorX: number | null,
  screen: { width: number },
  cfg: CurtainConfig,
  now: number,
  dtMs: number,
): CurtainState {
  return advance(decide(state, cursorX, screen, now), cfg, dtMs);
}
