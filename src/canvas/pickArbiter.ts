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
// ────────────────────────────────────────────────────────────────────────────

import type { PickOwner } from "./hitPriority";

/** Démarre l'interaction (sélection + éventuel drag/resize) sur `id`.
 *  `corner` non vide = poignée de redimensionnement. */
export type PickBeginFn = (id: string, e: PointerEvent, corner?: string) => void;

const handlers = new Map<PickOwner, PickBeginFn>();

/** Enregistre le handler d'une couche. Renvoie la fonction de désinscription
 *  (à retourner tel quel depuis un `useEffect`). */
export function registerPickHandler(owner: PickOwner, fn: PickBeginFn): () => void {
  handlers.set(owner, fn);
  return () => {
    if (handlers.get(owner) === fn) handlers.delete(owner);
  };
}

/** Remet le clic à la couche propriétaire. `false` si personne n'est inscrit. */
export function beginPick(owner: PickOwner, id: string, e: PointerEvent, corner?: string): boolean {
  const fn = handlers.get(owner);
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
// détourné, et GlucoseCanvas avale le `dblclick` correspondant.
//
// Aucune perte : toutes les couches détectent le double-clic sur `pointerdown`
// (cf. `lastClickRef`) ou sur `pointerup` (dossiers), jamais via l'event natif.

let lastDownHijacked = false;

export function markHijack(hijacked: boolean) {
  lastDownHijacked = hijacked;
}

export function wasHijacked(): boolean {
  return lastDownHijacked;
}
