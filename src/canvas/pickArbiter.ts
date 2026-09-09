// ────────────────────────────────────────────────────────────────────────────
// PICK-1 — Aiguilleur de clic entre les couches.
//
// `hitPriority` dit QUI doit recevoir le clic ; ce module sait COMMENT le lui
// remettre. Chaque couche (Pixi images, HtmlAnnotationLayer, SvgAnnotationLayer,
// FolderSvgLayer) enregistre ici sa fonction « démarre une interaction sur cet
// élément ». GlucoseCanvas, qui intercepte le `pointerdown` en phase CAPTURE au
// niveau du wrapper, appelle ensuite `beginPick` sur le gagnant.
//
// Pourquoi un registre plutôt qu'un re-dispatch d'event synthétique sur le nœud
// DOM du gagnant : les couches n'ont pas toutes un nœud DOM (les images sont des
// sprites Pixi), et re-dispatcher rejouerait tout le pipeline d'events React
// (double détection de double-clic, boucles de capture). Appeler directement le
// handler de la couche est plus simple, synchrone, et sans effet de bord.
//
// Le registre est un singleton de module (et non un contexte React) : les quatre
// couches sont des sœurs, aucune n'englobe les autres, et un contexte obligerait
// à faire descendre un provider à travers tout GlucoseCanvas pour rien.
//
// ── LA PORTÉE (MEMB-8) ─────────────────────────────────────────────────────
//
// Un rideau monte les MÊMES couches que la scène, sur un autre board et une
// autre caméra. Un registre plat, indexé par le seul type de couche, ne peut
// pas décrire ça : la seconde instance écrasait la première, et cliquer une
// image dehors pilotait le rideau. Le registre est donc indexé par
// (PORTÉE, type de couche). `MAIN_SCOPE` est la scène ; chaque rideau prend
// `curtainScope(boardId)`. Deux portées ne se voient pas — c'est exactement ce
// qu'on veut, puisqu'elles décrivent deux mondes disjoints.
//
// Le drapeau de détournement suit la même règle, et il le DOIT : la scène et le
// rideau posent chacun leur garde `dblclick`, et un drapeau global ferait
// qu'un clic détourné dans le rideau avale le double-clic de la scène (et
// réciproquement). Le drapeau vit donc lui aussi par portée.
// ────────────────────────────────────────────────────────────────────────────

import type { PickOwner } from "./hitPriority";

/** Démarre l'interaction (sélection + éventuel drag/resize) sur `id`.
 *  `corner` non vide = poignée de redimensionnement. */
export type PickBeginFn = (id: string, e: PointerEvent, corner?: string) => void;

/** Portée d'un registre : un monde de clic autonome (une caméra, un board). */
export type PickScope = string;

/** La scène principale. Valeur par défaut partout : les appelants qui ne
 *  connaissent pas la notion de portée continuent de marcher tels quels. */
export const MAIN_SCOPE: PickScope = "main";

/** Portée d'un rideau. Dérivée du board pour qu'elle soit stable et unique :
 *  deux rideaux ouverts (un par personne) ne partagent jamais leur board. */
export function curtainScope(boardId: string): PickScope {
  return `curtain:${boardId}`;
}

const handlers = new Map<PickScope, Map<PickOwner, PickBeginFn>>();

/** Enregistre le handler d'une couche dans une portée. Renvoie la fonction de
 *  désinscription (à retourner tel quel depuis un `useEffect`). */
export function registerPickHandler(
  owner: PickOwner,
  fn: PickBeginFn,
  scope: PickScope = MAIN_SCOPE,
): () => void {
  let byOwner = handlers.get(scope);
  if (!byOwner) {
    byOwner = new Map();
    handlers.set(scope, byOwner);
  }
  byOwner.set(owner, fn);
  return () => {
    const m = handlers.get(scope);
    if (!m) return;
    if (m.get(owner) === fn) m.delete(owner);
    // Une portée vide ne doit pas survivre : un rideau fermé, un board
    // supprimé, et la Map fuirait une entrée par ouverture.
    if (m.size === 0) handlers.delete(scope);
  };
}

/** Remet le clic à la couche propriétaire de cette portée. `false` si personne
 *  n'est inscrit — l'appelant laisse alors filer l'event. */
export function beginPick(
  owner: PickOwner,
  id: string,
  e: PointerEvent,
  corner?: string,
  scope: PickScope = MAIN_SCOPE,
): boolean {
  const fn = handlers.get(scope)?.get(owner);
  if (!fn) return false;
  fn(id, e, corner);
  return true;
}

// ── Suppression du double-clic natif après un détournement ──────────────────
//
// Quand l'arbitre détourne un clic (ex. le curseur est sur une membrane mais
// c'est l'image en dessous qui gagne), on stoppe la propagation du `pointerdown`
// — mais le navigateur émettra quand même `click` puis `dblclick` sur la cible
// DOM d'origine. Sans garde, un double-clic « image sous membrane » ouvrirait
// l'éditeur de la MEMBRANE. On mémorise donc si le dernier pointerdown a été
// détourné, et l'arbitre de la portée avale le `dblclick` correspondant.
//
// Aucune perte : toutes les couches détectent le double-clic sur `pointerdown`
// (cf. `lastClickRef`) ou sur `pointerup` (dossiers), jamais via l'event natif.

const hijacked = new Map<PickScope, boolean>();

export function markHijack(h: boolean, scope: PickScope = MAIN_SCOPE) {
  hijacked.set(scope, h);
}

export function wasHijacked(scope: PickScope = MAIN_SCOPE): boolean {
  return hijacked.get(scope) === true;
}
