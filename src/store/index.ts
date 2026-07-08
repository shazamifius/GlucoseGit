// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// Phase 7.2.C â€” Store CRDT-first (Automerge source de vÃ©ritÃ©)
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
//
// Le store maintient dÃ©sormais un `_doc: Doc<Project>` Automerge comme source
// de vÃ©ritÃ©. La vue React (`project`) est le doc lui-mÃªme castÃ© en `Project`
// (les Proxy Automerge se comportent comme des objets/arrays JS standard pour
// les lectures, ce qui suffit Ã  React/PixiJS).
//
// Toutes les mutations passent par `mutate(message, mutator)` qui :
//   1. snapshot le doc courant dans `_undoStack` (Automerge dÃ©dupplique en
//      mÃ©moire grÃ¢ce au structural sharing)
//   2. applique `Automerge.change` avec le label `message`
//   3. met Ã  jour `_doc` ET `project` (nouvelle rÃ©fÃ©rence â†’ React re-render)
//   4. vide `_redoStack` (toute mutation invalide le redo)
//
// `undo()` / `redo()` font simplement pop/push dans les stacks Doc â€” pas de
// `Automerge.viewAt` ici car on veut un Ã©tat modifiable, pas une vue.
//
// IMPORTANT : Ã  l'intÃ©rieur d'un mutator Automerge, on travaille sur un draft
// (Proxy). Donc :
//   - `arr.push(x)` : OK (mute le doc)
//   - `arr.splice(i, n)` : OK
//   - `arr.filter(...)` / `arr.map(...)` : NE MUTENT PAS â€” utiliser splice/index
//   - `obj.foo = bar` : OK
//   - `Object.assign(obj, patch)` : OK
//   - delete `obj.foo` : utiliser `delete obj.foo` (Automerge **refuse**
//     `obj.foo = undefined` avec une RangeError â€” `undefined` n'est pas JSON).
//
// La lecture est ergonomique : `d.boards[0].images.find(i => i.id === x)`
// renvoie un proxy mutable de l'item.

import { create } from "zustand";
import {
  Annotation, ArrowAnnotation, Board, BoardImage, BoardZone, CanvasFolder, Domain,
  FolderTreeNode, Preset,
  Project, StoryboardPanel, StoryboardSettings, Tool, Viewport
} from "../types";
import { DEFAULT_PRESETS } from "../data/defaultPresets";
import { nanoid } from "../utils/nanoid";
import { wouldCreateMirrorCycle } from "./mirrorGraph";
import * as A from "./automerge";
import { LIMITS } from "../constants";
import { getCollabHandle } from "../multiplayer/collabHandle";
import { recordAction } from "../telemetry/telemetry";

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€ Bornes (CLEANUP R-02) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
const COORD_LIMIT = 1_000_000;
const SIZE_LIMIT = 200_000;
const MIN_SCALE = 0.005;
const MAX_SCALE = 50;
const UNDO_DEPTH = LIMITS.UNDO_DEPTH;
const clampNum = (v: number, min: number, max: number) =>
  Number.isFinite(v) ? Math.min(max, Math.max(min, v)) : 0;
const clampViewport = (vp: Viewport): Viewport => ({
  x: clampNum(vp.x, -COORD_LIMIT * 50, COORD_LIMIT * 50),
  y: clampNum(vp.y, -COORD_LIMIT * 50, COORD_LIMIT * 50),
  scale: clampNum(vp.scale, MIN_SCALE, MAX_SCALE) || 1,
});
const COORD_FIELDS = ["x", "y", "x2", "y2"] as const;
const SIZE_FIELDS = ["width", "height"] as const;
function clampSpatial<T extends object>(obj: T): T {
  // Automerge n'accepte pas les valeurs `undefined` lors d'une insertion : il
  // faut omettre la clÃ©. On strippe d'abord, sinon `b.annotations.push(ann)`
  // jette `Cannot assign undefined value at .../bgColor` et l'annotation n'est
  // jamais crÃ©Ã©e (bug bloquant la crÃ©ation de texte/sticky depuis Phase 7.2.C).
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(obj)) {
    if (v !== undefined) out[k] = v;
  }
  for (const f of COORD_FIELDS) {
    if (typeof out[f] === "number") out[f] = clampNum(out[f] as number, -COORD_LIMIT, COORD_LIMIT);
  }
  for (const f of SIZE_FIELDS) {
    if (typeof out[f] === "number") out[f] = clampNum(out[f] as number, 1, SIZE_LIMIT);
  }
  if (Array.isArray(out.waypoints)) {
    out.waypoints = (out.waypoints as Array<{ x: number; y: number }>).map((p) => ({
      x: clampNum(p.x, -COORD_LIMIT, COORD_LIMIT),
      y: clampNum(p.y, -COORD_LIMIT, COORD_LIMIT),
    }));
  }
  return out as T;
}

/** ClÃ©s du patch dont la valeur est explicitement `undefined` â€” Automerge
 *  refuse `obj.prop = undefined`, donc on les applique via `delete` aprÃ¨s le
 *  Object.assign du reste. */
function undefinedKeys(patch: object): string[] {
  const keys: string[] = [];
  for (const [k, v] of Object.entries(patch)) {
    if (v === undefined) keys.push(k);
  }
  return keys;
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€ Defaults â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function newBoard(name: string): import("../types").Board {
  return {
    id: nanoid(),
    name,
    images: [],
    annotations: [],
    panels: [],
    zones: [],
    folders: [],
    viewport: { x: 0, y: 0, scale: 1 },
    createdAt: Date.now(),
    updatedAt: Date.now(),
  };
}

const DEFAULT_BOARD_ID = nanoid();
const DEFAULT_PROJECT: Project = {
  version: "2.0.0",
  name: "Nouveau projet",
  boards: [{ ...newBoard("Board principal"), id: DEFAULT_BOARD_ID }],
  activeBoardId: DEFAULT_BOARD_ID,
  presets: [],
  domains: [],
  createdAt: Date.now(),
  updatedAt: Date.now(),
};

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€ Helpers Automerge-mutator â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// Dans un draft, `arr.filter(...)` ne modifie pas le doc â€” il faut splice.
function removeWhere<T>(arr: T[], predicate: (x: T) => boolean): T[] {
  const removed: T[] = [];
  for (let i = arr.length - 1; i >= 0; i--) {
    if (predicate(arr[i])) {
      removed.push(arr[i]);
      arr.splice(i, 1);
    }
  }
  return removed;
}
function indexById<T extends { id: string }>(arr: T[], id: string): number {
  return arr.findIndex((x) => x.id === id);
}

// ─────────── UNDO-1 — état de vue préservé à travers undo/redo ───────────────
/** Reconstruit le chemin de dossiers (folderStack UI) qui mène à `activeBoardId`
 *  à partir de la structure des folders. Pur (pas d'accès au store). */
function buildFolderStack(
  boards: Project["boards"],
  activeBoardId: string,
): Array<{ boardId: string; folderId: string }> {
  const parentMap = new Map<string, { boardId: string; folderId: string }>();
  for (const b of boards) {
    for (const f of b.folders ?? []) {
      parentMap.set(f.childBoardId, { boardId: b.id, folderId: f.id });
    }
  }
  const stack: Array<{ boardId: string; folderId: string }> = [];
  let curr = activeBoardId;
  let guard = 256;
  while (parentMap.has(curr) && guard-- > 0) {
    const parent = parentMap.get(curr)!;
    stack.unshift(parent);
    curr = parent.boardId;
  }
  return stack;
}

/** UNDO-1 — Après un undo/redo on restaure le CONTENU mais on garde la caméra et
 *  le dossier courant là où l'utilisateur se trouve : pas de téléportation. Mute
 *  `restored` (état passé, EN PLAIN JS) pour adopter la caméra de chaque board
 *  encore présent + l'activeBoardId courant s'il existe toujours après le restore.
 *
 *  CRITIQUE — pourquoi en PLAIN JS et plus via `A.change` sur un snapshot :
 *  cloner un snapshot puis `A.change()` dessus force Automerge à RECONSTRUIRE tout
 *  l'op-set ; sur un doc dont l'historique a la moindre incohérence (ex. fichier
 *  abîmé par d'anciens appends incrémentaux de collab), ça PANIQUE côté WASM
 *  (« MissingOps » → `unreachable`) et TUE l'app — toute édition suivante meurt.
 *  L'undo applique donc l'état passé EN AVANT (un nouveau change sur le doc vivant,
 *  exactement comme un déplacement, qui n'ajoute que des ops et ne panique pas). */
function preserveViewPlain(restored: Project, cur: Project): void {
  // Board actif : on reste où on est, SAUF si ce board n'existe plus après le
  // restore (ex : on annule la création du dossier dans lequel on était).
  if (restored.boards.some((b) => b.id === cur.activeBoardId)) {
    restored.activeBoardId = cur.activeBoardId;
  }
  // Viewport : on recopie la caméra courante de chaque board encore présent.
  for (const b of restored.boards) {
    const cb = cur.boards.find((x) => x.id === b.id);
    if (cb?.viewport) {
      b.viewport = { x: cb.viewport.x, y: cb.viewport.y, scale: cb.viewport.scale };
    }
  }
}

/** Réécrit EN AVANT le contenu d'un document pour qu'il corresponde à `plain`.
 *  Contrairement à un échange de snapshot (qui ferait reculer l'historique
 *  Automerge — interdit en mode collaboratif), ceci ajoute un nouveau change qui
 *  remplace le contenu. Réutilisé par `restoreToPreview`, `undo` et `redo` quand
 *  un handle de collaboration est attaché. NB : `blobs` n'est pas réécrit (les
 *  octets ne font que s'ajouter ; un blob orphelin est inoffensif). */
function rewriteProjectContent(d: Project, plain: Project): void {
  d.name = plain.name;
  d.activeBoardId = plain.activeBoardId;
  d.boards.splice(0, d.boards.length, ...plain.boards);
  d.presets.splice(0, d.presets.length, ...plain.presets);
  if (d.domains) d.domains.splice(0, d.domains.length, ...(plain.domains ?? []));
  else d.domains = plain.domains ?? [];
  d.updatedAt = Date.now();
}

export interface GlucoseStore {
  // ── Source de vérité Automerge ──────────────────────────────────────────
  _doc: A.Doc<Project>;
  /** Vue React-friendly du doc. Reflète soit `_doc` (présent), soit l'état preview
   *  Time Machine quand `_previewHeads` est défini. Nouvelle référence à chaque update. */
  project: Project;
  /** Helper central : toute mutation passe par là. Blocked si preview actif. */
  mutate: (message: string, mutator: (d: Project) => void) => void;
  /** UNDO-1 — Comme `mutate` mais SANS entrée undo/redo. Réservé à l'état de
   *  vue/navigation (viewport, activeBoardId) : ça ne doit jamais consommer un
   *  Ctrl+Z ni invalider le redo en attente. */
  mutateView: (message: string, mutator: (d: Project) => void) => void;
  /** UNDO-1 — Transaction d'interaction : regroupe un drag / tracé de flèche en
   *  UNE seule entrée undo. `beginLiveEdit` prend un snapshot unique ; tant que
   *  la transaction est ouverte, `mutate` applique les changements live SANS
   *  empiler. `endLiveEdit` referme. Idempotent ; `end` est sûr même sans `begin`. */
  _liveEdit: boolean;
  beginLiveEdit: () => void;
  endLiveEdit: () => void;

  // â”€â”€ Time Machine (Phase 7.4) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  /** Si dÃ©fini, on est en mode "preview historique" : `project` est figÃ© sur cet Ã©tat. */
  _previewHeads: A.Heads | null;
  /** Active/dÃ©sactive le mode preview. `null` = retour au prÃ©sent. */
  setPreviewHeads: (heads: A.Heads | null) => void;
  /** CrÃ©e un commit nommÃ© sans changement de donnÃ©es (jalon visible dans la timeline). */
  commitNamed: (message: string) => void;
  /** Applique l'Ã©tat preview comme nouveau commit. Sort du mode preview. */
  restoreToPreview: () => void;
  /** Git #1 — Restaure le CONTENU d'une version durable (chargée depuis disque)
   *  EN AVANT sur le doc vivant (forward-revert sûr : jamais de clone+change d'un
   *  snapshot, cf. memory undo-forward-revert-wasm-panic). Sort du mode preview. */
  restoreFromPlain: (plain: Project) => void;

  // ── Outil / sélection / navigation (état UI local, hors doc) ─
  activeTool: Tool;
  selectedImageIds: string[];
  selectedAnnotationIds: string[];
  folderStack: Array<{ boardId: string; folderId: string }>;
  activeBoardId: string;
  setActiveTool: (tool: Tool) => void;
  setSelectedImageIds: (ids: string[]) => void;
  setSelectedAnnotationIds: (ids: string[]) => void;

  // â”€â”€ Canal d'assets (collab) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  /** Compteur incrémenté quand des octets d'image ont été matérialisés sur le
   *  disque local (pair qui reçoit les images d'une chaîne collab). Le canvas
   *  s'y abonne pour purger sa blacklist de textures 404 et re-tenter le
   *  chargement des images qui manquaient. */
  _assetEpoch: number;
  /** Signale qu'un ou plusieurs assets viennent d'apparaître sur le disque. */
  bumpAssetEpoch: () => void;

  // â”€â”€ Undo / Redo (stacks de Doc Automerge) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  _undoStack: A.Doc<Project>[];
  _redoStack: A.Doc<Project>[];
  /** PrÃ©servÃ© pour rÃ©tro-compat : pousse manuellement le doc courant dans undoStack. */
  pushHistory: () => void;
  /** Annule la dernière édition. Renvoie `true` si quelque chose a été annulé
   *  (pile non vide), `false` sinon → permet un feedback honnête (pas de toast
   *  « Annulé » quand il n'y a rien à annuler). */
  undo: () => boolean;
  redo: () => boolean;

  // â”€â”€ Viewport (CAMÉRA LOCALE, jamais synchronisée) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  /** Caméra par board, LOCALE à cet utilisateur — n'est PAS dans le doc
   *  Automerge en collaboration (sinon le zoom/déplacement d'un pair bougerait
   *  l'écran de l'autre). En solo on la recopie aussi dans le doc pour qu'elle
   *  soit sauvegardée dans le fichier .glucose. */
  localViewports: Record<string, Viewport>;
  setViewport: (boardId: string, vp: Viewport) => void;
  /** Caméra effective d'un board : override local d'abord, sinon valeur du doc
   *  (fichier chargé), sinon défaut. */
  getViewport: (boardId: string) => Viewport;

  // â”€â”€ Images â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  /**
   * Ajoute une image au board. Si `embedBytes` est fourni ET que `img.asset`
   * est en mode "embed", les octets sont écrits dans `project.blobs[sha256]`
   * dans la MÊME mutation (atomique pour undo). Pas d'écriture disque.
   * R-EMB-01 (Sprint 2).
   */
  addImage: (boardId: string, img: BoardImage, embedBytes?: Uint8Array) => void;
  updateImage: (boardId: string, id: string, patch: Partial<BoardImage>) => void;
  removeImages: (boardId: string, ids: string[]) => void;
  updateMultipleImages: (boardId: string, updates: { id: string; patch: Partial<BoardImage> }[]) => void;

  // â”€â”€ SÃ©lection â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  selectAll: (boardId: string) => void;
  deleteSelected: (boardId: string) => void;
  duplicateSelected: (boardId: string) => void;
  moveSelected: (boardId: string, dx: number, dy: number) => void;

  // â”€â”€ Annotations â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  addAnnotation: (boardId: string, ann: Annotation) => void;
  updateAnnotation: (boardId: string, id: string, patch: Partial<Annotation>) => void;
  /** UNDO-1 — Réconcilie la taille MESURÉE d'une annotation (ResizeObserver du
   *  rendu). NON annulable (passe par mutateView) : c'est une taille dérivée du
   *  rendu auto-fit, pas une édition utilisateur — ça ne doit jamais consommer un
   *  Ctrl+Z ni vider le redo. */
  syncAnnotationSize: (boardId: string, id: string, width: number, height: number) => void;
  removeAnnotations: (boardId: string, ids: string[]) => void;

  // â”€â”€ Storyboard â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  setStoryboardSettings: (boardId: string, settings: StoryboardSettings) => void;
  clearStoryboard: (boardId: string) => void;
  addPanel: (boardId: string, panel: StoryboardPanel) => void;
  updatePanel: (boardId: string, id: string, patch: Partial<StoryboardPanel>) => void;
  removePanel: (boardId: string, id: string) => void;
  reorderPanels: (boardId: string) => void;

  // â”€â”€ Boards â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  addBoard: (name: string) => string;
  /** Ajoute un board complet (ex. produit par un plugin) SANS écraser le projet. */
  importBoard: (board: Board) => string;
  removeBoard: (id: string) => void;
  renameBoard: (id: string, name: string) => void;
  reorderBoards: (fromIndex: number, toIndex: number) => void;
  setActiveBoardId: (id: string) => void;
  duplicateBoard: (id: string) => void;
  applyPresetToBoard: (boardId: string, presetId: string | null, worldX?: number, worldY?: number) => void;
  setBoardZones: (boardId: string, zones: BoardZone[]) => void;

  // â”€â”€ Presets â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  addPreset: (preset: Preset) => void;
  removePreset: (id: string) => void;
  updatePreset: (id: string, patch: Partial<Preset>) => void;
  getAllPresets: () => Preset[];

  // â”€â”€ Domaines (Phase 3) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  addDomain: (domain: Domain) => void;
  updateDomain: (id: string, patch: Partial<Domain>) => void;
  removeDomain: (id: string) => void;
  getDomains: () => Domain[];
  assignDomainToNode: (boardId: string, nodeId: string, domainId: string, weight: number) => void;

  // â”€â”€ Miroirs (Phase 4) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  mirrorAnnotation: (boardId: string, originalId: string, x: number, y: number) => string | null;
  mirrorImage: (boardId: string, originalId: string, x: number, y: number) => string | null;
  mirrorFolder: (parentBoardId: string, originalFolderId: string, x: number, y: number) => string | null;
  findOriginalAnnotation: (id: string) => Annotation | undefined;
  findOriginalImage: (id: string) => BoardImage | undefined;
  findOriginalFolder: (id: string) => CanvasFolder | undefined;

  // â”€â”€ Canvas Folders â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  createFolder: (parentBoardId: string, folder: Omit<CanvasFolder, "childBoardId">) => void;
  /**
   * R-FIL-02 (Sprint 2) — crée un folder + son child board peuplé de
   * `seedAnnotations` (typiquement issu d'un scan filesystem). Atomique pour
   * undo/redo. Renvoie l'id du folder créé.
   */
  createFolderWithContent: (
    parentBoardId: string,
    folder: Omit<CanvasFolder, "id" | "childBoardId">,
    seedAnnotations: Annotation[],
  ) => string;
  /**
   * R-FIL-02 v2 — crée un arbre de folders miroir : 1 child board par dossier,
   * sous-dossiers navigables. Atomique pour undo/redo. Renvoie l'id du folder
   * racine.
   */
  createFolderTree: (parentBoardId: string, tree: FolderTreeNode) => string;
  /**
   * R-FIL-02 v3 (scan paresseux) — remplit le child board d'un folder
   * `pendingScan` avec UN niveau scanné (tiles + sous-boîtes vides), puis
   * marque le folder comme scanné. Idempotent.
   */
  expandFolder: (parentBoardId: string, folderId: string, level: FolderTreeNode) => void;
  updateFolder: (boardId: string, folderId: string, patch: Partial<CanvasFolder>) => void;
  removeFolders: (boardId: string, folderIds: string[]) => void;
  enterFolder: (folderId: string) => void;
  exitFolder: () => void;
  exitToRoot: () => void;

  // â”€â”€ Project â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  setProjectName: (name: string) => void;
  /** Charge un Project plain â†’ reconstruit un nouveau doc Automerge propre. */
  loadProject: (project: Project) => void;
  /** Variante CRDT : remplace directement `_doc` (load v2 binaire). */
  loadDoc: (doc: A.Doc<Project>) => void;
  /** Git #1 Phase 4 p2 — adopte le doc COMPACTÉ (lignée neuve, historique fin
   *  aplati). CONSERVE `_undoStack`/`_redoStack` : leurs snapshots se ré-appliquent
   *  EN AVANT (forward-revert), indépendamment de la lignée → l'undo reste valide
   *  à travers la compaction et l'utilisateur garde toute sa profondeur. SOLO
   *  uniquement (garde-fou amont dans runCompaction). */
  compactCurrentDoc: (compacted: A.Doc<Project>) => void;

  /** Applique des changes Automerge venus d'un peer LAN (Phase 7.5bis).
   *  Ne touche pas `_undoStack` (les actions distantes ne sont pas dans l'undo
   *  local, Ctrl+Z annule uniquement TES propres actions). */
  applyRemoteChanges: (changes: Uint8Array[]) => void;

  // ── Smart guides ──────────────────────────────────────────
  smartGuidesEnabled: boolean;
  toggleSmartGuides: () => void;
  guides: { x?: number[]; y?: number[] } | null;
  setGuides: (guides: { x?: number[]; y?: number[] } | null) => void;

  // â”€â”€ UI panels â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  rightPanelOpen: boolean;
  setRightPanelOpen: (open: boolean) => void;

  // â”€â”€ Hover & toggles â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  hoveredNodeId: string | null;
  setHoveredNodeId: (id: string | null) => void;
  transDomainVisible: boolean;
  toggleTransDomainVisible: () => void;

  // â”€â”€ RÃ©glette temporelle (Phase 6) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  temporalFilter: { start: number; end: number } | null;
  setTemporalFilter: (filter: { start: number; end: number } | null) => void;

  // â”€â”€ Pomodoro â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  pomodoroTotal: number;
  pomodoroLeft: number;
  pomodoroRunning: boolean;
  pomodoroDone: boolean;
  pomodoroStart: () => void;
  pomodoroPause: () => void;
  pomodoroReset: (total: number) => void;
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€ Pomodoro module-level â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
let _pomInterval: ReturnType<typeof setInterval> | null = null;

function _fireChime() {
  try {
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain); gain.connect(ctx.destination);
    osc.type = "sine";
    osc.frequency.setValueAtTime(880, ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(440, ctx.currentTime + 0.6);
    gain.gain.setValueAtTime(0.4, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.8);
    osc.start(ctx.currentTime);
    osc.stop(ctx.currentTime + 0.8);
  } catch (_) { /* AudioContext not available */ }
}

function _fireNotification(totalSeconds: number) {
  const mins = Math.round(totalSeconds / 60);
  if (typeof Notification === "undefined") return;
  if (Notification.permission === "granted") {
    new Notification("Pomodoro terminÃ© !", { body: `Session de ${mins} min Ã©coulÃ©e.`, silent: true });
  } else if (Notification.permission !== "denied") {
    Notification.requestPermission().then((p) => {
      if (p === "granted") new Notification("Pomodoro terminÃ© !", { body: `Session de ${mins} min Ã©coulÃ©e.`, silent: true });
    });
  }
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€ CrÃ©ation du doc initial â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
const INITIAL_DOC = A.create<Project>(DEFAULT_PROJECT);

export const useGlucoseStore = create<GlucoseStore>((set, get) => ({
  _doc: INITIAL_DOC,
  project: INITIAL_DOC as unknown as Project,
  activeBoardId: DEFAULT_BOARD_ID,

  mutate: (message, mutator) => {
    const _t0 = performance.now(); // télémétrie : temps par action (no-op si non consenti)
    const handle = getCollabHandle();
    if (handle) {
      // ── Mode COLLAB ── la mutation passe par le handle automerge-repo, qui
      // synchronise et persiste automatiquement. On lit ensuite le doc résultant
      // et on met à jour la vue + la pile undo, exactement comme en solo.
      const s = get();
      if (s._previewHeads !== null) {
        console.warn("[mutate] Mutation ignorée : mode Time Machine actif. Sors-en pour modifier.");
        return;
      }
      // IMPORTANT : en collab on mute TOUJOURS via `handle.change` (jamais
      // `A.change` brut sur le doc du handle — ça corrompt l'objet WASM
      // « recursive use / unsafe aliasing »). Pendant un geste (_liveEdit) on ne
      // pousse simplement pas d'entrée undo (1 seul snapshot pris au début).
      const before = s._doc;
      handle.change((d) => mutator(d as Project), { message });
      const next = handle.doc() as unknown as A.Doc<Project>;
      set((st) =>
        st._liveEdit
          ? { _doc: next, project: next as unknown as Project }
          : {
              _doc: next,
              project: next as unknown as Project,
              _undoStack: [...st._undoStack.slice(-(UNDO_DEPTH - 1)), before],
              _redoStack: [],
            }
      );
      recordAction(message, performance.now() - _t0);
      return;
    }

    // ── Mode SOLO ── (comportement historique, inchangé)
    set((s) => {
      // Phase 7.4 â€” bloquer les mutations en mode preview Time Machine.
      if (s._previewHeads !== null) {
        console.warn("[mutate] Mutation ignorÃ©e : mode Time Machine actif. Sors-en pour modifier.");
        return s;
      }
      const before = s._doc;
      const next = A.change(before, message, (d) => mutator(d as Project));
      // Pendant une transaction d'interaction (drag, tracé de flèche), la pile a
      // déjà été snapshottée au beginLiveEdit → on n'empile pas à chaque frame.
      if (s._liveEdit) {
        return { _doc: next, project: next as unknown as Project };
      }
      return {
        _doc: next,
        project: next as unknown as Project,
        _undoStack: [...s._undoStack.slice(-(UNDO_DEPTH - 1)), before],
        _redoStack: [],
      };
    });
    recordAction(message, performance.now() - _t0);
  },

  _liveEdit: false,
  beginLiveEdit: () => {
    set((s) => {
      if (s._liveEdit || s._previewHeads !== null) return s; // déjà ouverte / Time Machine
      // 1 seul snapshot pour TOUTE l'interaction ; démarrer une édition vide le redo.
      return {
        _liveEdit: true,
        _undoStack: [...s._undoStack.slice(-(UNDO_DEPTH - 1)), s._doc],
        _redoStack: [],
      };
    });
  },
  endLiveEdit: () => {
    if (get()._liveEdit) set({ _liveEdit: false });
  },

  mutateView: (message, mutator) => {
    // UNDO-1 — Mutation de l'état de VUE uniquement (viewport / activeBoardId).
    // On applique le change Automerge (donc c'est persisté + visible Time Machine)
    // mais on NE touche NI `_undoStack` NI `_redoStack` : naviguer/zoomer ne doit
    // jamais s'empiler dans l'undo ni détruire un redo en attente.
    const _t0 = performance.now();
    const handle = getCollabHandle();
    if (handle) {
      if (get()._previewHeads !== null) return; // bloqué en mode Time Machine
      handle.change((d) => mutator(d as Project), { message });
      const next = handle.doc() as unknown as A.Doc<Project>;
      set({ _doc: next, project: next as unknown as Project });
      recordAction(message, performance.now() - _t0);
      return;
    }
    set((s) => {
      if (s._previewHeads !== null) return s; // bloqué en mode Time Machine
      const next = A.change(s._doc, message, (d) => mutator(d as Project));
      return { _doc: next, project: next as unknown as Project };
    });
    recordAction(message, performance.now() - _t0);
  },

  // â”€â”€ Time Machine â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  _previewHeads: null,
  setPreviewHeads: (heads) => {
    set((s) => {
      // Évite le re-calcul si les heads n'ont pas changé (comparaison d'arrays de hashes)
      if (s._previewHeads === heads) return s;
      if (
        s._previewHeads &&
        heads &&
        s._previewHeads.length === heads.length &&
        s._previewHeads.every((h, i) => h === heads[i])
      ) {
        return s;
      }
      if (heads === null) {
        // Sortie du mode preview : project revient au doc courant
        return { _previewHeads: null, project: s._doc as unknown as Project };
      }
      try {
        const viewed = A.viewAt<Project>(s._doc, heads);
        return { _previewHeads: heads, project: viewed as unknown as Project };
      } catch (e) {
        console.error("[setPreviewHeads] viewAt failed:", e);
        return s;
      }
    });
  },

  commitNamed: (message) => {
    // Un commit Automerge requiert au moins une "modification" â€” on touche
    // updatedAt pour matÃ©rialiser le jalon dans l'historique. Le `message` est
    // ce qui s'affichera dans la Time Machine.
    const trimmed = message.trim() || "Jalon";
    get().mutate(`ðŸ“Œ ${trimmed}`, (d) => { d.updatedAt = Date.now(); });
  },

  restoreToPreview: () => {
    const s = get();
    if (s._previewHeads === null) return;
    let pastPlain: Project;
    try {
      const viewed = A.viewAt<Project>(s._doc, s._previewHeads);
      pastPlain = A.asPlain(viewed);
    } catch (e) {
      console.error("[restoreToPreview] viewAt failed:", e);
      return;
    }
    // 1) Sortir du mode preview pour autoriser la mutation
    set({ _previewHeads: null, project: s._doc as unknown as Project });
    // 2) Commit qui rÃ©Ã©crit tout le contenu pour matcher l'Ã©tat passÃ©.
    //    L'historique antÃ©rieur reste prÃ©servÃ© : c'est un commit en avant
    //    qui dit "reviens Ã  l'Ã©tat du jalon X".
    get().mutate("âª Restauration depuis la timeline", (d) => {
      rewriteProjectContent(d, pastPlain);
    });
  },

  restoreFromPlain: (plain) => {
    // Sort d'un eventuel mode preview (sinon `mutate` est bloque), puis reecrit
    // tout le contenu EN AVANT pour matcher la version durable. Aucun clone+change
    // d'un snapshot -> pas de reconstruction d'op-set -> pas de panic MissingOps.
    set((s) => (s._previewHeads !== null
      ? { _previewHeads: null, project: s._doc as unknown as Project }
      : s));
    get().mutate("Restauration d'une version durable", (d) => {
      rewriteProjectContent(d, plain);
    });
  },

  // â”€â”€ Ã‰tat UI local (jamais dans le doc) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  activeTool: "select",
  selectedImageIds: [],
  selectedAnnotationIds: [],
  folderStack: [],
  smartGuidesEnabled: true,
  guides: null,
  rightPanelOpen: false,
  hoveredNodeId: null,
  transDomainVisible: true,
  temporalFilter: null,
  setActiveTool: (tool) => set({ activeTool: tool }),
  setSelectedImageIds: (ids) => set({ selectedImageIds: ids }),
  setSelectedAnnotationIds: (ids) => set({ selectedAnnotationIds: ids }),
  setHoveredNodeId: (id) => { if (get().hoveredNodeId !== id) set({ hoveredNodeId: id }); },
  toggleTransDomainVisible: () => set((s) => ({ transDomainVisible: !s.transDomainVisible })),
  setTemporalFilter: (filter) => set({ temporalFilter: filter }),
  setRightPanelOpen: (open) => { if (get().rightPanelOpen !== open) set({ rightPanelOpen: open }); },
  toggleSmartGuides: () => set((s) => ({ smartGuidesEnabled: !s.smartGuidesEnabled })),
  setGuides: (guides) => set({ guides }),

  // â”€â”€ Canal d'assets (collab) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  _assetEpoch: 0,
  bumpAssetEpoch: () => set((s) => ({ _assetEpoch: s._assetEpoch + 1 })),

  // â”€â”€ Undo / Redo â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  _undoStack: [],
  _redoStack: [],
  pushHistory: () => {
    // Compat : pousse un snapshot du doc actuel dans undoStack sans muter.
    set((s) => ({ _undoStack: [...s._undoStack.slice(-(UNDO_DEPTH - 1)), s._doc], _redoStack: [] }));
  },
  undo: () => {
    if (get()._undoStack.length === 0) return false;
    const handle = getCollabHandle();
    if (handle) {
      // Mode COLLAB : on ne peut pas faire reculer un doc Automerge partagé. On
      // applique l'état passé EN AVANT (nouveau change qui réécrit le contenu),
      // tout en conservant la caméra courante via preserveView.
      const s = get();
      const before = s._doc;
      // FORWARD-REVERT — état passé lu EN PLAIN (sûr) puis appliqué EN AVANT. On ne
      // clone/`change()` JAMAIS un snapshot : ça reconstruit l'op-set et panique en
      // WASM sur un doc abîmé (cf. preserveViewPlain), tuant l'app.
      const restoredPlain = A.asPlain(s._undoStack[s._undoStack.length - 1]) as Project;
      preserveViewPlain(restoredPlain, before as unknown as Project);
      handle.change((d) => rewriteProjectContent(d as Project, restoredPlain), { message: "Annuler" });
      const next = handle.doc() as unknown as A.Doc<Project>;
      const proj = next as unknown as Project;
      set((st) => ({
        _doc: next,
        project: proj,
        _undoStack: st._undoStack.slice(0, -1),
        _redoStack: [...st._redoStack.slice(-(UNDO_DEPTH - 1)), before],
        _previewHeads: null,
        _liveEdit: false,
        selectedImageIds: [],
        selectedAnnotationIds: [],
        folderStack: buildFolderStack(proj.boards, proj.activeBoardId),
      }));
      return true;
    }
    set((s) => {
      if (s._undoStack.length === 0) return s;
      // Clone le doc avant restauration : un doc Automerge dÃ©jÃ  utilisÃ© comme
      // input Ã  `A.change()` est gelÃ©, et la prochaine mutation jetterait
      // Â« Attempting to change an outdated document Â». Le clone est cheap
      // (structural sharing).
      const restoredPlain = A.asPlain(s._undoStack[s._undoStack.length - 1]) as Project;
      // UNDO-1 — on restaure le contenu mais on GARDE la caméra et le dossier
      // courant (pas de téléportation) ; le folderStack est reconstruit pour
      // matcher l'activeBoardId final.
      preserveViewPlain(restoredPlain, s._doc as unknown as Project);
      const prev = A.change(s._doc, "Annuler", (d) => rewriteProjectContent(d as Project, restoredPlain));
      const proj = prev as unknown as Project;
      return {
        _doc: prev,
        project: proj,
        _undoStack: s._undoStack.slice(0, -1),
        _redoStack: [...s._redoStack.slice(-(UNDO_DEPTH - 1)), s._doc],
        // Sortir du preview Time Machine si actif (l'undo est une opÃ©ration "live")
        _previewHeads: null,
        _liveEdit: false,
        selectedImageIds: [],
        selectedAnnotationIds: [],
        folderStack: buildFolderStack(proj.boards, proj.activeBoardId),
      };
    });
    return true;
  },
  redo: () => {
    if (get()._redoStack.length === 0) return false;
    const handle = getCollabHandle();
    if (handle) {
      const s = get();
      const before = s._doc;
      const restoredPlain = A.asPlain(s._redoStack[s._redoStack.length - 1]) as Project;
      preserveViewPlain(restoredPlain, before as unknown as Project);
      handle.change((d) => rewriteProjectContent(d as Project, restoredPlain), { message: "Rétablir" });
      const next = handle.doc() as unknown as A.Doc<Project>;
      const proj = next as unknown as Project;
      set((st) => ({
        _doc: next,
        project: proj,
        _redoStack: st._redoStack.slice(0, -1),
        _undoStack: [...st._undoStack.slice(-(UNDO_DEPTH - 1)), before],
        _previewHeads: null,
        _liveEdit: false,
        selectedImageIds: [],
        selectedAnnotationIds: [],
        folderStack: buildFolderStack(proj.boards, proj.activeBoardId),
      }));
      return true;
    }
    set((s) => {
      if (s._redoStack.length === 0) return s;
      const restoredPlain = A.asPlain(s._redoStack[s._redoStack.length - 1]) as Project;
      preserveViewPlain(restoredPlain, s._doc as unknown as Project);
      const next = A.change(s._doc, "Rétablir", (d) => rewriteProjectContent(d as Project, restoredPlain));
      const proj = next as unknown as Project;
      return {
        _doc: next,
        project: proj,
        _redoStack: s._redoStack.slice(0, -1),
        _undoStack: [...s._undoStack.slice(-(UNDO_DEPTH - 1)), s._doc],
        _previewHeads: null,
        _liveEdit: false,
        selectedImageIds: [],
        selectedAnnotationIds: [],
        folderStack: buildFolderStack(proj.boards, proj.activeBoardId),
      };
    });
    return true;
  },

  // â”€â”€ Viewport â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  localViewports: {},
  setViewport: (boardId, vp) => {
    const safe = clampViewport(vp);
    // 1) Override LOCAL (ce que lit le canvas) — jamais synchronisé.
    set((s) => ({ localViewports: { ...s.localViewports, [boardId]: safe } }));
    // 2) En SOLO uniquement, on recopie aussi dans le doc pour que la caméra
    //    soit sauvegardée dans le .glucose. En collab on n'y touche PAS (sinon
    //    la caméra d'un pair s'imposerait à l'autre et saturerait le réseau).
    if (!getCollabHandle()) {
      get().mutateView("setViewport", (d) => {
        const b = d.boards.find((x) => x.id === boardId);
        if (b) b.viewport = safe;
      });
    }
  },
  getViewport: (boardId) => {
    const s = get();
    const local = s.localViewports[boardId];
    if (local) return local;
    const b = s.project.boards.find((x) => x.id === boardId);
    return b?.viewport ?? { x: 0, y: 0, scale: 1 };
  },

  // â”€â”€ Images â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  addImage: (boardId, img, embedBytes) => {
    const safe = clampSpatial(img);
    get().mutate("addImage", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      b.images.push(safe);
      // R-EMB-01 (Sprint 2) : si on a des bytes à embedder pour cette image,
      // on les ajoute à project.blobs dans la même mutation. Dédup naturelle :
      // si un autre image partage déjà ce sha, on ne réécrit pas.
      if (embedBytes && safe.asset?.mode === "embed") {
        if (!d.blobs) d.blobs = {};
        if (!d.blobs[safe.asset.sha256]) {
          d.blobs[safe.asset.sha256] = embedBytes;
        }
      }
      b.updatedAt = Date.now();
      d.updatedAt = Date.now();
    });
  },

  updateImage: (boardId, id, patch) => {
    const safe = clampSpatial(patch);
    const toDelete = undefinedKeys(patch);
    get().mutate("updateImage", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      const idx = indexById(b.images, id);
      if (idx === -1) return;
      const old = b.images[idx];
      const dx = (safe.x !== undefined) ? safe.x - old.x : 0;
      const dy = (safe.y !== undefined) ? safe.y - old.y : 0;
      Object.assign(b.images[idx], safe);
      for (const k of toDelete) delete (b.images[idx] as unknown as Record<string, unknown>)[k];
      // DÃ©placement induit des flÃ¨ches attachÃ©es
      if (dx || dy) {
        for (const a of b.annotations) {
          if (a.type !== "arrow") continue;
          if (a.sourceId === id) { a.x += dx; a.y += dy; }
          if (a.targetId === id) {
            a.x2 = a.x2 + dx;
            a.y2 = a.y2 + dy;
          }
        }
      }
      b.updatedAt = Date.now();
    });
  },

  updateMultipleImages: (boardId, updates) => {
    const map = new Map(updates.map((u) => [u.id, clampSpatial(u.patch)]));
    get().mutate("updateMultipleImages", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      for (let i = 0; i < b.images.length; i++) {
        const p = map.get(b.images[i].id);
        if (p) Object.assign(b.images[i], p);
      }
      b.updatedAt = Date.now();
    });
  },

  removeImages: (_boardId, ids) => {
    // _boardId : conservÃ© pour signature stable, mais la cascade miroirs traverse
    // tous les boards (un miroir peut Ãªtre ailleurs).
    const idSet = new Set(ids);
    get().mutate("removeImages", (d) => {
      // Cascade miroirs : supprimer aussi les images dont mirrorOf est dans toRemove
      const toRemove = new Set(idSet);
      let grew = true;
      while (grew) {
        grew = false;
        for (const b of d.boards) {
          for (const img of b.images) {
            if (img.mirrorOf && toRemove.has(img.mirrorOf) && !toRemove.has(img.id)) {
              toRemove.add(img.id);
              grew = true;
            }
          }
        }
      }
      // Suppression dans tous les boards (les miroirs peuvent Ãªtre ailleurs)
      for (const b of d.boards) {
        const removed = removeWhere(b.images, (img) => toRemove.has(img.id));
        // FlÃ¨ches orphelines (source ou cible supprimÃ©e)
        removeWhere(b.annotations, (a) => {
          if (a.type !== "arrow") return false;
          return (!!a.sourceId && toRemove.has(a.sourceId)) ||
                 (!!a.targetId && toRemove.has(a.targetId));
        });
        if (removed.length > 0) b.updatedAt = Date.now();
      }
      // R-EMB-01 — Nettoyage des blobs orphelins : on supprime de project.blobs
      // les sha256 qui ne sont plus référencés par aucune image restante.
      if (d.blobs) {
        const usedSha = new Set<string>();
        for (const b of d.boards) {
          for (const img of b.images) {
            if (img.asset?.mode === "embed") usedSha.add(img.asset.sha256);
          }
        }
        for (const key of Object.keys(d.blobs)) {
          if (!usedSha.has(key)) delete d.blobs[key];
        }
      }
    });
    set({ selectedImageIds: [] });
  },

  // â”€â”€ SÃ©lection â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  selectAll: (boardId) => set((s) => {
    const board = s.project.boards.find((b) => b.id === boardId);
    if (!board) return s;
    return {
      selectedImageIds: board.images.map((img) => img.id),
      selectedAnnotationIds: board.annotations.map((a) => a.id),
    };
  }),

  deleteSelected: (boardId) => {
    const { selectedImageIds, selectedAnnotationIds } = get();
    if (selectedImageIds.length > 0) get().removeImages(boardId, selectedImageIds);
    if (selectedAnnotationIds.length > 0) get().removeAnnotations(boardId, selectedAnnotationIds);
  },

  duplicateSelected: (boardId) => {
    const { selectedImageIds, selectedAnnotationIds, project } = get();
    const board = project.boards.find((b) => b.id === boardId);
    if (!board) return;
    const OFFSET = 20;
    // PrÃ©-calcule les copies (ids gÃ©nÃ©rÃ©s hors du mutator). Deep-clone via
    // JSON pour casser les rÃ©fÃ©rences Proxy de l'arbre Automerge â€” sans Ã§a,
    // les sous-arrays (tags, domains, waypoints) seraient des refs proxy que
    // `push` refuserait. Puis clampSpatial pour stripper les undefined.
    const newImages: BoardImage[] = board.images
      .filter((img) => selectedImageIds.includes(img.id))
      .map((img) => {
        const plain = JSON.parse(JSON.stringify(img)) as BoardImage;
        return clampSpatial({ ...plain, id: nanoid(), x: plain.x + OFFSET, y: plain.y + OFFSET });
      });
    const newAnnotations: Annotation[] = board.annotations
      .filter((ann) => selectedAnnotationIds.includes(ann.id))
      .map((ann) => {
        const plain = JSON.parse(JSON.stringify(ann)) as Annotation;
        return clampSpatial({ ...plain, id: nanoid(), x: plain.x + OFFSET, y: plain.y + OFFSET });
      });

    get().mutate("duplicateSelected", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      for (const img of newImages) b.images.push(img);
      for (const a of newAnnotations) b.annotations.push(a);
      b.updatedAt = Date.now();
    });
    set({
      selectedImageIds: newImages.map((i) => i.id),
      selectedAnnotationIds: newAnnotations.map((a) => a.id),
    });
  },

  moveSelected: (boardId, dx, dy) => {
    const { selectedImageIds, selectedAnnotationIds } = get();
    const selImg = new Set(selectedImageIds);
    const selAnn = new Set(selectedAnnotationIds);
    if (selImg.size === 0 && selAnn.size === 0) return;

    get().mutate("moveSelected", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      // Images sÃ©lectionnÃ©es
      for (const img of b.images) {
        if (selImg.has(img.id)) { img.x += dx; img.y += dy; }
      }
      // Annotations
      for (const a of b.annotations) {
        if (selAnn.has(a.id)) {
          a.x += dx; a.y += dy;
          if (a.type === "arrow") {
            a.x2 += dx;
            a.y2 += dy;
            if (a.waypoints) {
              for (const wp of a.waypoints) { wp.x += dx; wp.y += dy; }
            }
          }
          continue;
        }
        // FlÃ¨che dont source/cible attachÃ©e bouge
        if (a.type === "arrow") {
          if (a.sourceId && (selImg.has(a.sourceId) || selAnn.has(a.sourceId))) {
            a.x += dx; a.y += dy;
          }
          if (a.targetId && (selImg.has(a.targetId) || selAnn.has(a.targetId))) {
            a.x2 = (a.x2 ?? a.x + 100) + dx;
            a.y2 = (a.y2 ?? a.y) + dy;
          }
        }
      }
      b.updatedAt = Date.now();
    });
  },

  // â”€â”€ Annotations â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  addAnnotation: (boardId, ann) => {
    const safe = clampSpatial(ann);
    get().mutate("addAnnotation", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      b.annotations.push(safe);
      b.updatedAt = Date.now();
      d.updatedAt = Date.now();
    });
  },

  updateAnnotation: (boardId, id, patch) => {
    const safe = clampSpatial(patch);
    const toDelete = undefinedKeys(patch);
    get().mutate("updateAnnotation", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      const idx = indexById(b.annotations, id);
      if (idx === -1) return;
      const old = b.annotations[idx];
      const dx = (safe.x !== undefined) ? safe.x - old.x : 0;
      const dy = (safe.y !== undefined) ? safe.y - old.y : 0;
      Object.assign(b.annotations[idx], safe);
      for (const k of toDelete) delete (b.annotations[idx] as unknown as Record<string, unknown>)[k];
      if (dx || dy) {
        for (const a of b.annotations) {
          if (a.type !== "arrow" || a.id === id) continue;
          if (a.sourceId === id) { a.x += dx; a.y += dy; }
          if (a.targetId === id) {
            a.x2 = a.x2 + dx;
            a.y2 = a.y2 + dy;
          }
        }
      }
    });
  },

  syncAnnotationSize: (boardId, id, width, height) => {
    // UNDO-1 — taille auto-fit mesurée par le rendu (ResizeObserver) → mutateView
    // (jamais d'entrée d'undo). Sinon chaque reflow du markdown empilait un Ctrl+Z
    // « fantôme » APRÈS la fermeture de la transaction d'édition → undo « inutile ».
    const w = clampNum(width, 1, SIZE_LIMIT);
    const h = clampNum(height, 1, SIZE_LIMIT);
    get().mutateView("syncAnnotationSize", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      const a = b.annotations.find((x) => x.id === id);
      if (a && a.type !== "arrow") { a.width = w; a.height = h; }
    });
  },

  removeAnnotations: (boardId, ids) => {
    const idSet = new Set(ids);
    get().mutate("removeAnnotations", (d) => {
      // Cascade miroirs
      const toRemove = new Set(idSet);
      let grew = true;
      while (grew) {
        grew = false;
        for (const b of d.boards) {
          for (const a of b.annotations) {
            if (a.mirrorOf && toRemove.has(a.mirrorOf) && !toRemove.has(a.id)) {
              toRemove.add(a.id);
              grew = true;
            }
          }
        }
      }
      for (const b of d.boards) {
        removeWhere(b.annotations, (a) => {
          if (toRemove.has(a.id)) return true;
          if (a.type === "arrow") {
            if (a.sourceId && toRemove.has(a.sourceId)) return true;
            if (a.targetId && toRemove.has(a.targetId)) return true;
          }
          return false;
        });
        if (b.id === boardId) b.updatedAt = Date.now();
      }
    });
    set({ selectedAnnotationIds: [] });
  },

  // â”€â”€ Storyboard â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  setStoryboardSettings: (boardId, settings) => {
    get().mutate("setStoryboardSettings", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (b) b.storyboard = settings;
    });
  },
  clearStoryboard: (boardId) => {
    get().mutate("clearStoryboard", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      delete b.storyboard;
      b.panels.splice(0, b.panels.length);
    });
  },
  addPanel: (boardId, panel) => {
    get().mutate("addPanel", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      b.panels.push(panel);
      b.updatedAt = Date.now();
    });
  },
  updatePanel: (boardId, id, patch) => {
    get().mutate("updatePanel", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      const idx = indexById(b.panels, id);
      if (idx !== -1) Object.assign(b.panels[idx], patch);
    });
  },
  removePanel: (boardId, id) => {
    get().mutate("removePanel", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      removeWhere(b.panels, (p) => p.id === id);
    });
  },
  reorderPanels: (boardId) => {
    get().mutate("reorderPanels", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      for (let i = 0; i < b.panels.length; i++) b.panels[i].order = i;
    });
  },

  // â”€â”€ Boards â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  addBoard: (name) => {
    const board = newBoard(name);
    get().mutate("addBoard", (d) => {
      d.boards.push(board);
      d.activeBoardId = board.id;
      d.updatedAt = Date.now();
    });
    set({ activeBoardId: board.id, selectedImageIds: [], selectedAnnotationIds: [] });
    return board.id;
  },

  // Ajoute un board déjà constitué (résultat d'un plugin, déjà validé par
  // parseProjectFile) à la suite des boards existants — le travail en cours
  // n'est jamais écrasé. On clone en plain (le draft Automerge exige du JSON)
  // et on régénère l'id du board pour écarter toute collision ; les annotations
  // gardent leurs ids (les flèches s'y réfèrent via sourceId/targetId).
  importBoard: (board) => {
    const fresh: Board = {
      ...(JSON.parse(JSON.stringify(board)) as Board),
      id: nanoid(),
      updatedAt: Date.now(),
    };
    get().mutate("importBoard", (d) => {
      d.boards.push(fresh);
      d.activeBoardId = fresh.id;
      d.updatedAt = Date.now();
    });
    set({ activeBoardId: fresh.id, selectedImageIds: [], selectedAnnotationIds: [], folderStack: [] });
    return fresh.id;
  },

  removeBoard: (id) => {
    if (get().project.boards.length <= 1) return;
    get().mutate("removeBoard", (d) => {
      const removeIdx = d.boards.findIndex((b) => b.id === id);
      if (removeIdx === -1) return;
      // Patch les flèches portail orphelines avant de retirer le board
      for (const b of d.boards) {
        for (const a of b.annotations) {
          if (a.type === "arrow" && a.targetBoardId === id) delete a.targetBoardId;
        }
      }
      d.boards.splice(removeIdx, 1);
      // Bascule activeBoardId si c'était celui supprimé
      if (d.activeBoardId === id) {
        const nextIdx = Math.max(0, removeIdx - 1);
        d.activeBoardId = d.boards[nextIdx]?.id ?? d.boards[0].id;
      }
      d.updatedAt = Date.now();
    });
    set((s) => {
      let activeBoardId = s.activeBoardId;
      if (activeBoardId === id) {
        const removeIdx = s.project.boards.findIndex((b) => b.id === id);
        const nextIdx = Math.max(0, removeIdx - 1);
        activeBoardId = s.project.boards[nextIdx]?.id ?? s.project.boards[0]?.id ?? DEFAULT_BOARD_ID;
      }
      return {
        activeBoardId,
        selectedImageIds: [],
        selectedAnnotationIds: [],
        folderStack: s.folderStack.filter(
          (f) => f.boardId !== id && s.project.boards.some((b) => b.id === f.boardId)
        ),
      };
    });
  },

  renameBoard: (id, name) => {
    get().mutate("renameBoard", (d) => {
      const b = d.boards.find((x) => x.id === id);
      if (b) b.name = name;
    });
  },

  reorderBoards: (fromIndex, toIndex) => {
    get().mutate("reorderBoards", (d) => {
      if (fromIndex < 0 || fromIndex >= d.boards.length) return;
      if (toIndex < 0 || toIndex >= d.boards.length) return;
      // Snapshot plain : le proxy retourné par splice() devient invalide après
      // suppression de son emplacement, Automerge refuse de le réinsérer.
      const moved = JSON.parse(JSON.stringify(d.boards[fromIndex]));
      d.boards.splice(fromIndex, 1);
      d.boards.splice(toIndex, 0, moved);
    });
  },

  setActiveBoardId: (id) => {
    // Navigation pure → mutateView (jamais dans l'undo). Reconstruit folderStack
    // pour pointer vers le nouveau board s'il est sous-dossier.
    const stack = buildFolderStack(get().project.boards, id);
    if (!getCollabHandle()) {
      get().mutateView("setActiveBoardId", (d) => { d.activeBoardId = id; });
    }
    set({ activeBoardId: id, selectedImageIds: [], selectedAnnotationIds: [], folderStack: stack });
  },

  duplicateBoard: (id) => {
    const src = get().project.boards.find((b) => b.id === id);
    if (!src) return;
    // Deep-copy en plain JS (le draft Automerge n'accepterait pas un proxy)
    const copy = JSON.parse(JSON.stringify({ ...src, id: nanoid(), name: `${src.name} (copie)`, createdAt: Date.now(), updatedAt: Date.now() }));
    get().mutate("duplicateBoard", (d) => {
      d.boards.push(copy);
      d.activeBoardId = copy.id;
    });
    set({ activeBoardId: copy.id });
  },

  applyPresetToBoard: (boardId, presetId, worldX, worldY) => {
    const all = get().getAllPresets();
    const preset = presetId ? all.find((p) => p.id === presetId) : null;
    const vp = get().getViewport(boardId);
    let zones: BoardZone[] = [];
    if (preset) {
      const ZONE_W = 340, ZONE_H = 700, GAP = 30;
      const n = preset.slots.length;
      const totalW = n * ZONE_W + (n - 1) * GAP;
      const offsetX = worldX !== undefined ? worldX - totalW / 2 : (-vp.x / vp.scale);
      const offsetY = worldY !== undefined ? worldY - ZONE_H / 2 : (-vp.y / vp.scale);
      zones = preset.slots.map((slot, i) => ({
        slotId: slot.id, x: offsetX + i * (ZONE_W + GAP), y: offsetY, width: ZONE_W, height: ZONE_H,
      }));
    }
    get().mutate("applyPresetToBoard", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      if (presetId) b.presetId = presetId; else delete b.presetId;
      // Replace zones array
      b.zones.splice(0, b.zones.length, ...zones);
      d.updatedAt = Date.now();
    });
  },

  setBoardZones: (boardId, zones) => {
    // Deep-copy : zones est probablement un proxy (lecture du store)
    const safe = JSON.parse(JSON.stringify(zones));
    get().mutate("setBoardZones", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      b.zones.splice(0, b.zones.length, ...safe);
    });
  },

  // â”€â”€ Presets â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  addPreset: (preset) => {
    get().mutate("addPreset", (d) => { d.presets.push(preset); });
  },
  removePreset: (id) => {
    get().mutate("removePreset", (d) => {
      removeWhere(d.presets, (p) => p.id === id);
    });
  },
  updatePreset: (id, patch) => {
    get().mutate("updatePreset", (d) => {
      const idx = indexById(d.presets, id);
      if (idx !== -1) Object.assign(d.presets[idx], patch);
    });
  },
  getAllPresets: () => [...DEFAULT_PRESETS, ...get().project.presets],

  // â”€â”€ Domaines (Phase 3) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  addDomain: (domain) => {
    get().mutate("addDomain", (d) => {
      if (!d.domains) d.domains = [];
      d.domains.push(domain);
      d.updatedAt = Date.now();
    });
  },
  updateDomain: (id, patch) => {
    get().mutate("updateDomain", (d) => {
      if (!d.domains) return;
      const idx = indexById(d.domains, id);
      if (idx !== -1) Object.assign(d.domains[idx], patch);
      d.updatedAt = Date.now();
    });
  },
  removeDomain: (id) => {
    get().mutate("removeDomain", (d) => {
      if (d.domains) removeWhere(d.domains, (x) => x.id === id);
      // Cascade : retire l'assignation de tous les nÅ“uds
      for (const b of d.boards) {
        for (const a of b.annotations) {
          if (a.domains) removeWhere(a.domains, (da) => da.domainId === id);
        }
        for (const img of b.images) {
          if (img.domains) removeWhere(img.domains, (da) => da.domainId === id);
        }
      }
      d.updatedAt = Date.now();
    });
  },
  getDomains: () => get().project.domains ?? [],
  assignDomainToNode: (boardId, nodeId, domainId, weight) => {
    get().mutate("assignDomainToNode", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      const upsert = (arr: { domainId: string; weight: number }[] | undefined) => {
        if (!arr) return weight <= 0 ? [] : [{ domainId, weight }];
        if (weight <= 0) {
          removeWhere(arr, (x) => x.domainId === domainId);
          return arr;
        }
        const idx = arr.findIndex((x) => x.domainId === domainId);
        if (idx === -1) arr.push({ domainId, weight });
        else arr[idx].weight = weight;
        return arr;
      };
      const a = b.annotations.find((x) => x.id === nodeId);
      if (a) {
        if (!a.domains) a.domains = [];
        upsert(a.domains);
      }
      const img = b.images.find((x) => x.id === nodeId);
      if (img) {
        if (!img.domains) img.domains = [];
        upsert(img.domains);
      }
      d.updatedAt = Date.now();
    });
  },

  // â”€â”€ Miroirs (Phase 4) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  // Les findOriginal* sont des LECTURES â†’ pas de mutate.
  findOriginalAnnotation: (id) => {
    const project = get().project;
    let cur: Annotation | undefined;
    let safety = 16;
    let next: string | undefined = id;
    while (next && safety-- > 0) {
      cur = undefined;
      for (const b of project.boards) {
        const found = b.annotations.find((a) => a.id === next);
        if (found) { cur = found; break; }
      }
      if (!cur) return undefined;
      if (!cur.mirrorOf) return cur;
      next = cur.mirrorOf;
    }
    if (safety <= 0) console.error(`[findOriginalAnnotation] Chain too deep for ${id}`);
    return cur;
  },
  findOriginalImage: (id) => {
    const project = get().project;
    let cur: BoardImage | undefined;
    let safety = 16;
    let next: string | undefined = id;
    while (next && safety-- > 0) {
      cur = undefined;
      for (const b of project.boards) {
        const found = b.images.find((i) => i.id === next);
        if (found) { cur = found; break; }
      }
      if (!cur) return undefined;
      if (!cur.mirrorOf) return cur;
      next = cur.mirrorOf;
    }
    if (safety <= 0) console.error(`[findOriginalImage] Chain too deep for ${id}`);
    return cur;
  },
  findOriginalFolder: (id) => {
    const project = get().project;
    let cur: CanvasFolder | undefined;
    let safety = 16;
    let next: string | undefined = id;
    while (next && safety-- > 0) {
      cur = undefined;
      for (const b of project.boards) {
        const found = (b.folders ?? []).find((f) => f.id === next);
        if (found) { cur = found; break; }
      }
      if (!cur) return undefined;
      if (!cur.mirrorOf) return cur;
      next = cur.mirrorOf;
    }
    if (safety <= 0) console.error(`[findOriginalFolder] Chain too deep for ${id}`);
    return cur;
  },

  mirrorAnnotation: (boardId, originalId, x, y) => {
    const original = get().findOriginalAnnotation(originalId);
    if (!original) return null;
    const newId = nanoid();
    // Snapshot plain (le draft refusera un proxy importÃ©)
    const ann: Annotation = JSON.parse(JSON.stringify({ ...original, id: newId, x, y, mirrorOf: original.id }));
    get().mutate("mirrorAnnotation", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (b) b.annotations.push(ann);
      d.updatedAt = Date.now();
    });
    return newId;
  },

  mirrorImage: (boardId, originalId, x, y) => {
    const original = get().findOriginalImage(originalId);
    if (!original) return null;
    const newId = nanoid();
    const img: BoardImage = JSON.parse(JSON.stringify({ ...original, id: newId, x, y, mirrorOf: original.id }));
    get().mutate("mirrorImage", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (b) b.images.push(img);
      d.updatedAt = Date.now();
    });
    return newId;
  },

  mirrorFolder: (parentBoardId, originalFolderId, x, y) => {
    const original = get().findOriginalFolder(originalFolderId);
    if (!original) return null;
    if (wouldCreateMirrorCycle(get().project.boards, original.id, parentBoardId)) {
      console.warn(`[mirrorFolder] Cycle refusÃ© pour "${original.name}"`);
      return null;
    }
    const newId = nanoid();
    const mirror: CanvasFolder = {
      id: newId,
      name: original.name,
      color: original.color,
      x, y,
      width: original.width,
      height: original.height,
      childBoardId: original.childBoardId, // partage le mÃªme childBoard
      mirrorOf: original.id,
    };
    get().mutate("mirrorFolder", (d) => {
      const b = d.boards.find((x) => x.id === parentBoardId);
      if (!b) return;
      if (!b.folders) b.folders = [];
      b.folders.push(mirror);
      d.updatedAt = Date.now();
    });
    return newId;
  },

  // â”€â”€ Canvas Folders â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  // Phase 7.5 â€” Capture des blocs au drag-create par-dessus.
  createFolder: (parentBoardId, folderData) => {
    const childBoardId = nanoid();
    const folder: CanvasFolder = clampSpatial({
      ...folderData,
      id: folderData.id ?? nanoid(),
      childBoardId,
    });

    // On collecte d'abord les items capturÃ©s HORS du mutator (lecture pure)
    const proj = get().project;
    const parent = proj.boards.find((b) => b.id === parentBoardId);

    const fx0 = folder.x, fy0 = folder.y;
    const fx1 = folder.x + folder.width, fy1 = folder.y + folder.height;
    const inside = (cx: number, cy: number) =>
      cx >= fx0 && cx <= fx1 && cy >= fy0 && cy <= fy1;

    const capturedImageIds = new Set<string>();
    const capturedFolderIds = new Set<string>();
    const capturedAnnIds = new Set<string>();

    if (parent) {
      for (const img of parent.images) {
        if (inside(img.x, img.y)) capturedImageIds.add(img.id);
      }
      for (const f of parent.folders ?? []) {
        const cx = f.x + f.width / 2, cy = f.y + f.height / 2;
        if (inside(cx, cy)) capturedFolderIds.add(f.id);
      }
      for (const a of parent.annotations) {
        if (a.type === "arrow") continue;
        const w = a.width ?? 100, h = a.height ?? 40;
        const cx = a.x + w / 2, cy = a.y + h / 2;
        if (inside(cx, cy)) capturedAnnIds.add(a.id);
      }
    }
    // FlÃ¨ches : capturÃ©es si les deux extrÃ©mitÃ©s le sont
    const capturedArrowIds = new Set<string>();
    if (parent) {
      for (const a of parent.annotations) {
        if (a.type !== "arrow") continue;
        const srcCap = a.sourceId ? (capturedAnnIds.has(a.sourceId) || capturedImageIds.has(a.sourceId)) : false;
        const tgtCap = a.targetId ? (capturedAnnIds.has(a.targetId) || capturedImageIds.has(a.targetId)) : false;
        const headIn = inside(a.x, a.y);
        const tailIn = a.x2 !== undefined && a.y2 !== undefined && inside(a.x2, a.y2);
        const fully = (a.sourceId && a.targetId) ? (srcCap && tgtCap) : (headIn && tailIn);
        if (fully) capturedArrowIds.add(a.id);
      }
    }

    get().mutate("createFolder", (d) => {
      // 1) CrÃ©e le child board. ATTENTION : aprÃ¨s push, on doit re-rÃ©cupÃ©rer
      // le PROXY Automerge â€” la variable JS d'origine n'est plus reliÃ©e au doc.
      d.boards.push({ ...newBoard(folderData.name), id: childBoardId });
      const childBoard = d.boards.find((b) => b.id === childBoardId);
      if (!childBoard) return;

      const par = d.boards.find((b) => b.id === parentBoardId);
      if (!par) return;

      // 2) TransfÃ¨re les images capturÃ©es (deep-clone via JSON pour casser
      // les refs proxy avant push â€” sinon Automerge refuse)
      const imagesToMove: BoardImage[] = [];
      removeWhere(par.images, (img) => {
        if (capturedImageIds.has(img.id)) {
          const plain = JSON.parse(JSON.stringify(img)) as BoardImage;
          imagesToMove.push({ ...plain, x: plain.x - fx0, y: plain.y - fy0 });
          return true;
        }
        return false;
      });
      for (const img of imagesToMove) childBoard.images.push(img);

      // 3) TransfÃ¨re les sous-folders
      if (par.folders) {
        const foldersToMove: CanvasFolder[] = [];
        removeWhere(par.folders, (f) => {
          if (capturedFolderIds.has(f.id)) {
            const plain = JSON.parse(JSON.stringify(f)) as CanvasFolder;
            foldersToMove.push({ ...plain, x: plain.x - fx0, y: plain.y - fy0 });
            return true;
          }
          return false;
        });
        if (!childBoard.folders) childBoard.folders = [];
        for (const f of foldersToMove) childBoard.folders.push(f);
      }

      // 4) TransfÃ¨re les annotations capturÃ©es
      const annsToMove: Annotation[] = [];
      removeWhere(par.annotations, (a) => {
        if (capturedAnnIds.has(a.id)) {
          const plain = JSON.parse(JSON.stringify(a)) as Annotation;
          annsToMove.push({ ...plain, x: plain.x - fx0, y: plain.y - fy0 });
          return true;
        }
        if (capturedArrowIds.has(a.id) && a.type === "arrow") {
          const plain = JSON.parse(JSON.stringify(a)) as ArrowAnnotation;
          // clampSpatial strippe les clÃ©s undefined (ex: waypoints absents)
          // qu'Automerge refuserait Ã  l'insertion.
          annsToMove.push(clampSpatial({
            ...plain,
            x: plain.x - fx0, y: plain.y - fy0,
            x2: plain.x2 - fx0,
            y2: plain.y2 - fy0,
            waypoints: plain.waypoints?.map((p) => ({ x: p.x - fx0, y: p.y - fy0 })),
          }));
          return true;
        }
        return false;
      });
      for (const a of annsToMove) childBoard.annotations.push(a);

      // 5) Ajoute le folder dans le parent
      if (!par.folders) par.folders = [];
      par.folders.push(folder);
      par.updatedAt = Date.now();
      d.updatedAt = Date.now();
    });
  },

  createFolderWithContent: (parentBoardId, folderData, seedAnnotations) => {
    // R-FIL-02 — Variante de createFolder qui pré-peuple le child board.
    // Utilisée par le drop d'un dossier OS (folderMirror.scanFolderForMirror).
    const childBoardId = nanoid();
    const folderId = nanoid();
    const folder: CanvasFolder = clampSpatial({
      ...folderData,
      id: folderId,
      childBoardId,
    });

    get().mutate("createFolderWithContent", (d) => {
      // 1) Crée le child board pré-peuplé.
      // ATTENTION : après push, on doit récupérer le PROXY Automerge pour
      // pouvoir y push les annotations (cf. fix Sprint 1 — variables JS
      // déconnectées du doc après push).
      d.boards.push({ ...newBoard(folderData.name), id: childBoardId });
      const childBoard = d.boards.find((b) => b.id === childBoardId);
      if (!childBoard) return;

      // 2) Insère les annotations dans le child board (deep-clone pour
      // casser les refs avant push — pattern Sprint 1).
      for (const ann of seedAnnotations) {
        const plain = JSON.parse(JSON.stringify(ann)) as Annotation;
        const safe = clampSpatial(plain);
        childBoard.annotations.push(safe);
      }

      // 3) Ajoute le folder au parent board.
      const par = d.boards.find((b) => b.id === parentBoardId);
      if (!par) return;
      if (!par.folders) par.folders = [];
      par.folders.push(folder);
      par.updatedAt = Date.now();
      d.updatedAt = Date.now();
    });
    return folderId;
  },

  createFolderTree: (parentBoardId, tree) => {
    // R-FIL-02 v2 — crée 1 child board par dossier. Toute l'arborescence dans
    // UNE mutation = undo atomique.
    //
    // PERF : on aplatit l'arbre en JS pur (Phase 1) AVANT la mutation, puis on
    // applique à plat avec un index Map id→proxy (Phase 2). Évite le O(n²) de
    // `d.boards.find` par nœud qui gelait l'app sur les gros dossiers (et
    // pouvait tronquer le rendu). Voir retours utilisateur "perd 70%".
    const rootFolderId = nanoid();

    interface BoardSpec { boardId: string; name: string; annotations: Annotation[]; images: BoardImage[]; }
    interface Placement { parentBoardId: string; folder: CanvasFolder; }
    const boardSpecs: BoardSpec[] = [];
    const placements: Placement[] = [];

    // Phase 1 — aplatissement pur (récursion JS rapide, hors doc Automerge).
    const walk = (parentId: string, node: FolderTreeNode, folderId: string): void => {
      const childBoardId = nanoid();
      boardSpecs.push({
        boardId: childBoardId,
        name: node.folder.name,
        annotations: node.annotations,
        images: node.images ?? [],
      });
      placements.push({
        parentBoardId: parentId,
        folder: { ...node.folder, id: folderId, childBoardId } as CanvasFolder,
      });
      for (const child of node.children) walk(childBoardId, child, nanoid());
    };
    walk(parentBoardId, tree, rootFolderId);

    get().mutate("createFolderTree", (d) => {
      // Phase 2a — crée tous les child boards d'un coup.
      for (const spec of boardSpecs) {
        d.boards.push({ ...newBoard(spec.name), id: spec.boardId });
      }
      // Index O(1) (re-fetch des proxies après les push — cf. fix Sprint 1).
      const byId = new Map<string, (typeof d.boards)[number]>();
      for (const b of d.boards) byId.set(b.id, b);

      // Phase 2b — peuple chaque board (annotations + images).
      for (const spec of boardSpecs) {
        const board = byId.get(spec.boardId);
        if (!board) continue;
        for (const ann of spec.annotations) {
          board.annotations.push(clampSpatial(JSON.parse(JSON.stringify(ann)) as Annotation));
        }
        for (const img of spec.images) {
          board.images.push(clampSpatial(JSON.parse(JSON.stringify(img)) as BoardImage));
        }
      }

      // Phase 2c — place les folder boxes dans leurs parents.
      const now = Date.now();
      for (const pl of placements) {
        const par = byId.get(pl.parentBoardId);
        if (!par) continue;
        if (!par.folders) par.folders = [];
        par.folders.push(clampSpatial(JSON.parse(JSON.stringify(pl.folder)) as CanvasFolder));
        par.updatedAt = now;
      }
      d.updatedAt = now;
    });
    return rootFolderId;
  },

  expandFolder: (parentBoardId, folderId, level) => {
    // R-FIL-02 v3 — scan paresseux. `level` = un niveau scanné dont la racine
    // correspond au folder DÉJÀ existant (folderId). On verse son contenu dans
    // le child board existant et on crée des sous-boîtes vides (pendingScan).
    const proj = get().project;
    const parent = proj.boards.find((b) => b.id === parentBoardId);
    const folder = (parent?.folders ?? []).find((f) => f.id === folderId);
    if (!folder) return;
    if (folder.mirrorSource && folder.mirrorSource.pendingScan === false) return; // déjà scanné
    const childBoardId = folder.childBoardId;

    // Aplatissement des sous-boîtes (children = sous-dossiers vides pendingScan).
    interface BoardSpec { boardId: string; name: string; annotations: Annotation[]; images: BoardImage[]; }
    interface Placement { parentBoardId: string; folder: CanvasFolder; }
    const boardSpecs: BoardSpec[] = [];
    const placements: Placement[] = [];
    for (const child of level.children) {
      const subBoardId = nanoid();
      boardSpecs.push({ boardId: subBoardId, name: child.folder.name, annotations: child.annotations, images: child.images ?? [] });
      placements.push({ parentBoardId: childBoardId, folder: { ...child.folder, id: nanoid(), childBoardId: subBoardId } as CanvasFolder });
    }

    // UNDO-1 — le scan paresseux est un effet de bord de NAVIGATION (déclenché
    // en entrant dans un dossier pendingScan), pas une édition : il ne doit pas
    // consommer d'undo ni vider le redo. C'est idempotent + persisté (Automerge).
    get().mutateView("expandFolder", (d) => {
      // 1) Crée les sous-child-boards (vides).
      for (const spec of boardSpecs) {
        d.boards.push({ ...newBoard(spec.name), id: spec.boardId });
      }
      const byId = new Map<string, (typeof d.boards)[number]>();
      for (const b of d.boards) byId.set(b.id, b);

      // 2) Verse le contenu de CE niveau dans le child board existant.
      const cb = byId.get(childBoardId);
      if (cb) {
        for (const ann of level.annotations) {
          cb.annotations.push(clampSpatial(JSON.parse(JSON.stringify(ann)) as Annotation));
        }
        for (const img of level.images ?? []) {
          cb.images.push(clampSpatial(JSON.parse(JSON.stringify(img)) as BoardImage));
        }
      }

      // 3) Place les sous-boîtes (vides) dans le child board.
      const now = Date.now();
      for (const pl of placements) {
        const par = byId.get(pl.parentBoardId);
        if (!par) continue;
        if (!par.folders) par.folders = [];
        par.folders.push(clampSpatial(JSON.parse(JSON.stringify(pl.folder)) as CanvasFolder));
      }

      // 4) Marque le folder comme scanné.
      const par2 = d.boards.find((b) => b.id === parentBoardId);
      const f2 = (par2?.folders ?? []).find((f) => f.id === folderId);
      if (f2?.mirrorSource) {
        f2.mirrorSource.pendingScan = false;
        f2.mirrorSource.lastScannedAt = now;
      }
      d.updatedAt = now;
    });
  },

  updateFolder: (boardId, folderId, patch) => {
    const safe = clampSpatial(patch);
    const toDelete = undefinedKeys(patch);
    get().mutate("updateFolder", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b || !b.folders) return;
      const idx = indexById(b.folders, folderId);
      if (idx !== -1) {
        Object.assign(b.folders[idx], safe);
        for (const k of toDelete) delete (b.folders[idx] as unknown as Record<string, unknown>)[k];
      }
      d.updatedAt = Date.now();
    });
  },

  removeFolders: (boardId, folderIds) => {
    // PrÃ©-calcul des childBoardIds Ã  potentiellement supprimer (BFS hors mutator)
    const proj = get().project;
    const parent = proj.boards.find((b) => b.id === boardId);
    const removed = (parent?.folders ?? []).filter((f) => folderIds.includes(f.id));
    const childIds = new Set(removed.map((f) => f.childBoardId));

    get().mutate("removeFolders", (d) => {
      // 1) Retire les folders du parent
      const par = d.boards.find((b) => b.id === boardId);
      if (par?.folders) removeWhere(par.folders, (f) => folderIds.includes(f.id));

      // 2) BFS pour identifier les child boards orphelins (cascade rÃ©cursive)
      const stillReferenced = new Set<string>();
      for (const b of d.boards) {
        for (const f of b.folders ?? []) stillReferenced.add(f.childBoardId);
      }
      const toDelete = new Set<string>();
      const queue: string[] = [];
      for (const id of childIds) if (!stillReferenced.has(id)) queue.push(id);
      while (queue.length > 0) {
        const id = queue.shift()!;
        if (toDelete.has(id)) continue;
        toDelete.add(id);
        const board = d.boards.find((b) => b.id === id);
        if (!board) continue;
        for (const f of board.folders ?? []) {
          let elsewhere = false;
          for (const b of d.boards) {
            if (b.id === id || toDelete.has(b.id)) continue;
            if ((b.folders ?? []).some((fd) => fd.childBoardId === f.childBoardId)) {
              elsewhere = true; break;
            }
          }
          if (!elsewhere) queue.push(f.childBoardId);
        }
      }

      // 3) Supprime les child boards orphelins
      removeWhere(d.boards, (b) => toDelete.has(b.id));

      // 4) Si le board actif est supprimÃ©, retombe sur le parent
      if (toDelete.has(d.activeBoardId)) d.activeBoardId = boardId;
      d.updatedAt = Date.now();
    });

    // Nettoie le folderStack (Ã©tat UI)
    set((s) => ({
      folderStack: s.folderStack.filter(
        (entry) => !folderIds.includes(entry.folderId) &&
          get().project.boards.some((b) => b.id === entry.boardId)
      ),
    }));
  },

  enterFolder: (folderId) => {
    const s = get();
    const boardId = s.activeBoardId;
    const board = s.project.boards.find((b) => b.id === boardId);
    const folder = (board?.folders ?? []).find((f) => f.id === folderId);
    if (!folder) return;
    const targetBoardId = folder.childBoardId;
    if (!getCollabHandle()) {
      get().mutateView("enterFolder", (d) => { d.activeBoardId = targetBoardId; });
    }
    set({
      activeBoardId: targetBoardId,
      folderStack: [...s.folderStack, { boardId, folderId }],
      selectedImageIds: [],
      selectedAnnotationIds: [],
    });
  },

  exitFolder: () => {
    const s = get();
    if (s.folderStack.length === 0) return;
    const prev = s.folderStack[s.folderStack.length - 1];
    if (!getCollabHandle()) {
      get().mutateView("exitFolder", (d) => { d.activeBoardId = prev.boardId; });
    }
    set({
      activeBoardId: prev.boardId,
      folderStack: s.folderStack.slice(0, -1),
      selectedImageIds: [],
      selectedAnnotationIds: [],
    });
  },

  exitToRoot: () => {
    const s = get();
    if (s.folderStack.length === 0) return;
    const root = s.folderStack[0];
    if (!getCollabHandle()) {
      get().mutateView("exitToRoot", (d) => { d.activeBoardId = root.boardId; });
    }
    set({
      activeBoardId: root.boardId,
      folderStack: [],
      selectedImageIds: [],
      selectedAnnotationIds: [],
    });
  },

  // â”€â”€ Project â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  setProjectName: (name) => {
    get().mutate("setProjectName", (d) => {
      d.name = name;
      d.updatedAt = Date.now();
    });
  },

  loadProject: (project) => {
    const normalized: Project = { ...project, domains: project.domains ?? [] };
    const newDoc = A.create<Project>(normalized);
    set({
      _doc: newDoc,
      project: newDoc as unknown as Project,
      activeBoardId: normalized.activeBoardId || normalized.boards[0]?.id || DEFAULT_BOARD_ID,
      _undoStack: [],
      _redoStack: [],
      _liveEdit: false,
      _previewHeads: null,
      selectedImageIds: [],
      selectedAnnotationIds: [],
      folderStack: buildFolderStack(normalized.boards, normalized.activeBoardId || normalized.boards[0]?.id || DEFAULT_BOARD_ID),
      // Nouveau projet → on repart des caméras du doc (pas d'override périmé).
      localViewports: {},
    });
  },

  loadDoc: (doc) => {
    const proj = doc as unknown as Project;
    set({
      _doc: doc,
      project: proj,
      activeBoardId: proj.activeBoardId || proj.boards[0]?.id || DEFAULT_BOARD_ID,
      _undoStack: [],
      _redoStack: [],
      _liveEdit: false,
      _previewHeads: null,
      selectedImageIds: [],
      selectedAnnotationIds: [],
      folderStack: buildFolderStack(proj.boards, proj.activeBoardId || proj.boards[0]?.id || DEFAULT_BOARD_ID),
      localViewports: {},
    });
  },

  compactCurrentDoc: (compacted) => {
    // On ne touche NI `_undoStack` NI `_redoStack` : le forward-revert lit
    // `A.asPlain` d'un snapshot et le ré-applique en avant sur le doc vivant →
    // lignée-agnostique. Zustand fusionne en surface, donc les piles absentes de
    // cet objet sont préservées telles quelles.
    set({
      _doc: compacted,
      project: compacted as unknown as Project,
      _previewHeads: null,
    });
  },

  applyRemoteChanges: (changes) => {
    if (changes.length === 0) return;
    set((s) => {
      try {
        const next = A.applyChanges(s._doc, changes);
        if (next === s._doc) return s; // pas de nouveauté (changes déjà connus)
        let project = next as unknown as Project;
        if (s._previewHeads !== null) {
          try {
            project = A.viewAt<Project>(next, s._previewHeads) as unknown as Project;
          } catch (e) {
            console.error("[applyRemoteChanges] viewAt failed, resetting preview:", e);
          }
        }
        return {
          _doc: next,
          project,
          // Pas de modification de _undoStack — les actions distantes ne sont pas
          // dans la pile undo locale.
        };
      } catch (e) {
        console.error("[applyRemoteChanges] failed:", e);
        return s;
      }
    });
  },

  // â”€â”€ Pomodoro â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
  pomodoroTotal: 25 * 60,
  pomodoroLeft: 25 * 60,
  pomodoroRunning: false,
  pomodoroDone: false,
  pomodoroStart: () => {
    const { pomodoroLeft, pomodoroDone } = get();
    if (pomodoroLeft <= 0 || pomodoroDone) return;
    set({ pomodoroRunning: true, pomodoroDone: false });
    if (_pomInterval) clearInterval(_pomInterval);
    _pomInterval = setInterval(() => {
      const left = get().pomodoroLeft;
      if (left <= 1) {
        clearInterval(_pomInterval!); _pomInterval = null;
        set({ pomodoroLeft: 0, pomodoroRunning: false, pomodoroDone: true });
        _fireChime();
        _fireNotification(get().pomodoroTotal);
      } else {
        set({ pomodoroLeft: left - 1 });
      }
    }, 1000);
  },
  pomodoroPause: () => {
    if (_pomInterval) { clearInterval(_pomInterval); _pomInterval = null; }
    set({ pomodoroRunning: false });
  },
  pomodoroReset: (total) => {
    if (_pomInterval) { clearInterval(_pomInterval); _pomInterval = null; }
    set({ pomodoroTotal: total, pomodoroLeft: total, pomodoroRunning: false, pomodoroDone: false });
  },
}));

export function getActiveBoard(project: Project) {
  const activeId = useGlucoseStore.getState().activeBoardId;
  return project.boards.find((b) => b.id === activeId) ?? project.boards[0];
}

// Subscrire pour s'assurer que activeBoardId pointe toujours vers un board existant
useGlucoseStore.subscribe((state) => {
  if (state.project && state.project.boards) {
    const activeExists = state.project.boards.some((b) => b.id === state.activeBoardId);
    if (!activeExists && state.project.boards.length > 0) {
      const fallbackId = state.project.boards[0].id;
      useGlucoseStore.setState({ activeBoardId: fallbackId });
    }
  }
});
