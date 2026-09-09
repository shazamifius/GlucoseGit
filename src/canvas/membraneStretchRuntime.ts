// ────────────────────────────────────────────────────────────────────────────
// MEMB-4 — Pont entre le moteur d'étirement (pur) et le document vivant.
//
// Même partage des rôles que `smartAlign` / `smartAlignRuntime` : toute la
// décision est dans `membraneStretch`, il ne reste ici que la lecture du board
// et l'écriture des tailles.
//
// UN SEUL POINT D'ÉCRITURE, et il n'écrit QUE ce qui bouge. `planBoardStretch`
// ne rend rien quand rien ne change : le cas courant — un glisser qui ne touche
// aucune membrane étirée — ne produit donc aucune mutation, donc aucune entrée
// d'annulation parasite et aucun message de synchro envoyé aux pairs.
//
// L'APPELANT OUVRE LA TRANSACTION. On n'appelle ni `beginLiveEdit` ni
// `endLiveEdit` ici : l'étirement est la CONSÉQUENCE d'un geste (un dépôt, une
// conversion de mode), et il doit tomber dans la même entrée d'annulation que
// lui. Annuler devrait rendre l'état d'avant le geste ENTIER — sinon un Ctrl+Z
// laisserait une membrane agrandie autour d'un contenu qui n'y est plus.
// ────────────────────────────────────────────────────────────────────────────

import { getActiveBoard, useGlucoseStore } from "../store";
import type { Annotation } from "../types";
import { itemsOfBoard } from "./membraneSpace";
import { planBoardStretch, type StretchOutcome } from "./membraneStretch";

/**
 * Fait grandir les membranes étirées du board et rend ce qui s'est passé.
 *
 * Le tableau rendu contient aussi les membranes qui ont BUTÉ sans grandir :
 * c'est ce dont l'interface a besoin pour signaler l'obstacle. Filtrer sur
 * `blocked` pour l'avertissement, sur `grew` pour savoir si le document a bougé.
 */
export function applyBoardStretch(boardId: string): StretchOutcome[] {
  const st = useGlucoseStore.getState();
  const board = st.project.boards.find((b) => b.id === boardId);
  if (!board) return [];

  const outcomes = planBoardStretch(itemsOfBoard(board));
  for (const o of outcomes) {
    if (!o.grew) continue;
    st.updateAnnotation(boardId, o.membraneId, {
      width: o.width, height: o.height,
    } as Partial<Annotation>);
  }
  return outcomes;
}

/** Idem, sur le board actif. */
export function applyActiveBoardStretch(): StretchOutcome[] {
  return applyBoardStretch(getActiveBoard(useGlucoseStore.getState().project).id);
}
