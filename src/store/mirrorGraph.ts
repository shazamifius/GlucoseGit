// ════════════════════════════════════════════════════════════════════════════
// Vérificateur acyclique pour les miroirs de dossiers (Phase 4)
//
// Pourquoi : un dossier-miroir partage le même `childBoardId` que son original.
// Si on autorisait : DossierA contient un miroir de DossierB ET DossierB contient
// un miroir de DossierA, alors entrer dans A → on voit B → entrer dans (le miroir
// de) B → on voit A → entrer dans (le miroir de) A → on voit B → ... à l'infini.
// PixiJS et le rendu boucleraient instantanément.
//
// Ce module fournit `wouldCreateMirrorCycle()` : un BFS strict sur l'arbre des
// boards (en suivant `folder.childBoardId`) qui rejette toute création susceptible
// de fermer une boucle, à n'importe quelle profondeur.
// ════════════════════════════════════════════════════════════════════════════

import { Board, CanvasFolder } from "../types";

/**
 * Renvoie `true` si placer un miroir du dossier `originalFolderId` à l'intérieur
 * du board `targetBoardId` créerait un cycle (Inception).
 *
 * Algorithme :
 *   1. On part du `childBoardId` du dossier original.
 *   2. On parcourt en BFS tous les boards atteignables en descendant dans
 *      les dossiers ET les miroirs de dossiers (qui partagent le même childBoardId).
 *   3. Si on rencontre `targetBoardId` à un moment donné → cycle confirmé.
 *
 * Cette fonction est PURE : aucune mutation du store, aucun side-effect.
 *
 * Cas pathologique de "self-mirror" : si on essaie de mettre un miroir de A
 * directement dans A (ou dans son enfant direct, ou dans n'importe quel descendant),
 * la BFS atteint `targetBoardId` immédiatement.
 */
export function wouldCreateMirrorCycle(
  boards: Board[],
  originalFolderId: string,
  targetBoardId: string,
): boolean {
  // Helpers indexés pour O(1) lookup
  const boardById = new Map<string, Board>();
  const folderById = new Map<string, CanvasFolder>();
  for (const b of boards) {
    boardById.set(b.id, b);
    for (const f of b.folders ?? []) folderById.set(f.id, f);
  }

  const original = folderById.get(originalFolderId);
  if (!original) return false; // Dossier introuvable → pas de cycle possible

  // BFS sur les boards descendants
  const visited = new Set<string>();
  const queue: string[] = [original.childBoardId];

  while (queue.length > 0) {
    const boardId = queue.shift()!;
    if (boardId === targetBoardId) return true; // ⚠️ Cycle détecté
    if (visited.has(boardId)) continue;
    visited.add(boardId);

    const board = boardById.get(boardId);
    if (!board) continue;

    // On enfile tous les childBoards (folders + folders-miroirs partageant childBoardId)
    for (const f of board.folders ?? []) {
      queue.push(f.childBoardId);
    }
  }

  return false;
}

/**
 * Renvoie le board qui contient le dossier d'id `folderId`, ou `undefined` si
 * aucun. Utilisé en amont de `wouldCreateMirrorCycle` pour identifier le board
 * cible quand on veut placer un miroir à proximité d'un dossier existant.
 */
export function findBoardContainingFolder(
  boards: Board[],
  folderId: string,
): Board | undefined {
  return boards.find((b) => (b.folders ?? []).some((f) => f.id === folderId));
}
