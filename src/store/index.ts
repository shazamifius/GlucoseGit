// ────────────────────────────────────────────────────────────────────────────
// Phase 7.2.C — Store CRDT-first (Automerge source de vérité)
// ────────────────────────────────────────────────────────────────────────────
//
// Le store maintient désormais un `_doc: Doc<Project>` Automerge comme source
// de vérité. La vue React (`project`) est le doc lui-même casté en `Project`
// (les Proxy Automerge se comportent comme des objets/arrays JS standard pour
// les lectures, ce qui suffit à React/PixiJS).
//
// Toutes les mutations passent par `mutate(message, mutator)` qui :
//   1. snapshot le doc courant dans `_undoStack` (Automerge dédupplique en
//      mémoire grâce au structural sharing)
//   2. applique `Automerge.change` avec le label `message`
//   3. met à jour `_doc` ET `project` (nouvelle référence → React re-render)
//   4. vide `_redoStack` (toute mutation invalide le redo)
//
// `undo()` / `redo()` font simplement pop/push dans les stacks Doc — pas de
// `Automerge.viewAt` ici car on veut un état modifiable, pas une vue.
//
// IMPORTANT : à l'intérieur d'un mutator Automerge, on travaille sur un draft
// (Proxy). Donc :
//   - `arr.push(x)` : OK (mute le doc)
//   - `arr.splice(i, n)` : OK
//   - `arr.filter(...)` / `arr.map(...)` : NE MUTENT PAS — utiliser splice/index
//   - `obj.foo = bar` : OK
//   - `Object.assign(obj, patch)` : OK
//   - delete `obj.foo` : utiliser `obj.foo = undefined` ou helper Automerge
//
// La lecture est ergonomique : `d.boards[0].images.find(i => i.id === x)`
// renvoie un proxy mutable de l'item.

import { create } from "zustand";
import {
  Annotation, BoardImage, BoardZone, CanvasFolder, Domain, Preset,
  Project, StoryboardPanel, StoryboardSettings, Tool, Viewport
} from "../types";
import { DEFAULT_PRESETS } from "../data/defaultPresets";
import { nanoid } from "../utils/nanoid";
import { wouldCreateMirrorCycle } from "./mirrorGraph";
import * as A from "./automerge";
import { LIMITS } from "../constants";

// ─────────── Bornes (CLEANUP R-02) ──────────────────────────────────────────
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
  // faut omettre la clé. On strippe d'abord, sinon `b.annotations.push(ann)`
  // jette `Cannot assign undefined value at .../bgColor` et l'annotation n'est
  // jamais créée (bug bloquant la création de texte/sticky depuis Phase 7.2.C).
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

// ─────────── Defaults ───────────────────────────────────────────────────────
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

// ─────────── Helpers Automerge-mutator ──────────────────────────────────────
// Dans un draft, `arr.filter(...)` ne modifie pas le doc — il faut splice.
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

interface GlucoseStore {
  // ── Source de vérité Automerge ────────────────────────────
  _doc: A.Doc<Project>;
  /** Vue React-friendly du doc. Reflète soit `_doc` (présent), soit l'état preview
   *  Time Machine quand `_previewHeads` est défini. Nouvelle référence à chaque update. */
  project: Project;
  /** Helper central : toute mutation passe par là. Blocked si preview actif. */
  mutate: (message: string, mutator: (d: Project) => void) => void;

  // ── Time Machine (Phase 7.4) ──────────────────────────────
  /** Si défini, on est en mode "preview historique" : `project` est figé sur cet état. */
  _previewHeads: A.Heads | null;
  /** Active/désactive le mode preview. `null` = retour au présent. */
  setPreviewHeads: (heads: A.Heads | null) => void;
  /** Crée un commit nommé sans changement de données (jalon visible dans la timeline). */
  commitNamed: (message: string) => void;
  /** Applique l'état preview comme nouveau commit. Sort du mode preview. */
  restoreToPreview: () => void;

  // ── Outil / sélection / navigation (état UI local, hors doc) ─
  activeTool: Tool;
  selectedImageIds: string[];
  selectedAnnotationIds: string[];
  folderStack: Array<{ boardId: string; folderId: string }>;
  setActiveTool: (tool: Tool) => void;
  setSelectedImageIds: (ids: string[]) => void;
  setSelectedAnnotationIds: (ids: string[]) => void;

  // ── Undo / Redo (stacks de Doc Automerge) ────────────────
  _undoStack: A.Doc<Project>[];
  _redoStack: A.Doc<Project>[];
  /** Préservé pour rétro-compat : pousse manuellement le doc courant dans undoStack. */
  pushHistory: () => void;
  undo: () => void;
  redo: () => void;

  // ── Viewport ──────────────────────────────────────────────
  setViewport: (boardId: string, vp: Viewport) => void;

  // ── Images ────────────────────────────────────────────────
  addImage: (boardId: string, img: BoardImage) => void;
  updateImage: (boardId: string, id: string, patch: Partial<BoardImage>) => void;
  removeImages: (boardId: string, ids: string[]) => void;
  updateMultipleImages: (boardId: string, updates: { id: string; patch: Partial<BoardImage> }[]) => void;

  // ── Sélection ─────────────────────────────────────────────
  selectAll: (boardId: string) => void;
  deleteSelected: (boardId: string) => void;
  duplicateSelected: (boardId: string) => void;
  moveSelected: (boardId: string, dx: number, dy: number) => void;

  // ── Annotations ───────────────────────────────────────────
  addAnnotation: (boardId: string, ann: Annotation) => void;
  updateAnnotation: (boardId: string, id: string, patch: Partial<Annotation>) => void;
  removeAnnotations: (boardId: string, ids: string[]) => void;

  // ── Storyboard ────────────────────────────────────────────
  setStoryboardSettings: (boardId: string, settings: StoryboardSettings) => void;
  clearStoryboard: (boardId: string) => void;
  addPanel: (boardId: string, panel: StoryboardPanel) => void;
  updatePanel: (boardId: string, id: string, patch: Partial<StoryboardPanel>) => void;
  removePanel: (boardId: string, id: string) => void;
  reorderPanels: (boardId: string) => void;

  // ── Boards ────────────────────────────────────────────────
  addBoard: (name: string) => string;
  removeBoard: (id: string) => void;
  renameBoard: (id: string, name: string) => void;
  reorderBoards: (fromIndex: number, toIndex: number) => void;
  setActiveBoardId: (id: string) => void;
  duplicateBoard: (id: string) => void;
  applyPresetToBoard: (boardId: string, presetId: string | null, worldX?: number, worldY?: number) => void;
  setBoardZones: (boardId: string, zones: BoardZone[]) => void;

  // ── Presets ───────────────────────────────────────────────
  addPreset: (preset: Preset) => void;
  removePreset: (id: string) => void;
  updatePreset: (id: string, patch: Partial<Preset>) => void;
  getAllPresets: () => Preset[];

  // ── Domaines (Phase 3) ────────────────────────────────────
  addDomain: (domain: Domain) => void;
  updateDomain: (id: string, patch: Partial<Domain>) => void;
  removeDomain: (id: string) => void;
  getDomains: () => Domain[];
  assignDomainToNode: (boardId: string, nodeId: string, domainId: string, weight: number) => void;

  // ── Miroirs (Phase 4) ─────────────────────────────────────
  mirrorAnnotation: (boardId: string, originalId: string, x: number, y: number) => string | null;
  mirrorImage: (boardId: string, originalId: string, x: number, y: number) => string | null;
  mirrorFolder: (parentBoardId: string, originalFolderId: string, x: number, y: number) => string | null;
  findOriginalAnnotation: (id: string) => Annotation | undefined;
  findOriginalImage: (id: string) => BoardImage | undefined;
  findOriginalFolder: (id: string) => CanvasFolder | undefined;

  // ── Canvas Folders ────────────────────────────────────────
  createFolder: (parentBoardId: string, folder: Omit<CanvasFolder, "childBoardId">) => void;
  updateFolder: (boardId: string, folderId: string, patch: Partial<CanvasFolder>) => void;
  removeFolders: (boardId: string, folderIds: string[]) => void;
  enterFolder: (folderId: string) => void;
  exitFolder: () => void;
  exitToRoot: () => void;

  // ── Project ───────────────────────────────────────────────
  setProjectName: (name: string) => void;
  /** Charge un Project plain → reconstruit un nouveau doc Automerge propre. */
  loadProject: (project: Project) => void;
  /** Variante CRDT : remplace directement `_doc` (load v2 binaire). */
  loadDoc: (doc: A.Doc<Project>) => void;

  /** Applique des changes Automerge venus d'un peer LAN (Phase 7.5bis).
   *  Ne touche pas `_undoStack` (les actions distantes ne sont pas dans l'undo
   *  local, Ctrl+Z annule uniquement TES propres actions). */
  applyRemoteChanges: (changes: Uint8Array[]) => void;

  // ── Smart guides ─────────────────────────────────────────
  smartGuidesEnabled: boolean;
  toggleSmartGuides: () => void;

  // ── UI panels ────────────────────────────────────────────
  rightPanelOpen: boolean;
  setRightPanelOpen: (open: boolean) => void;

  // ── Hover & toggles ──────────────────────────────────────
  hoveredNodeId: string | null;
  setHoveredNodeId: (id: string | null) => void;
  transDomainVisible: boolean;
  toggleTransDomainVisible: () => void;

  // ── Réglette temporelle (Phase 6) ────────────────────────
  temporalFilter: { start: number; end: number } | null;
  setTemporalFilter: (filter: { start: number; end: number } | null) => void;

  // ── Pomodoro ─────────────────────────────────────────────
  pomodoroTotal: number;
  pomodoroLeft: number;
  pomodoroRunning: boolean;
  pomodoroDone: boolean;
  pomodoroStart: () => void;
  pomodoroPause: () => void;
  pomodoroReset: (total: number) => void;
}

// ─────────── Pomodoro module-level ──────────────────────────────────────────
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
    new Notification("Pomodoro terminé !", { body: `Session de ${mins} min écoulée.`, silent: true });
  } else if (Notification.permission !== "denied") {
    Notification.requestPermission().then((p) => {
      if (p === "granted") new Notification("Pomodoro terminé !", { body: `Session de ${mins} min écoulée.`, silent: true });
    });
  }
}

// ─────────── Création du doc initial ────────────────────────────────────────
const INITIAL_DOC = A.create<Project>(DEFAULT_PROJECT);

export const useGlucoseStore = create<GlucoseStore>((set, get) => ({
  _doc: INITIAL_DOC,
  project: INITIAL_DOC as unknown as Project,

  mutate: (message, mutator) => {
    set((s) => {
      // Phase 7.4 — bloquer les mutations en mode preview Time Machine.
      if (s._previewHeads !== null) {
        console.warn("[mutate] Mutation ignorée : mode Time Machine actif. Sors-en pour modifier.");
        return s;
      }
      const before = s._doc;
      const next = A.change(before, message, (d) => mutator(d as Project));
      return {
        _doc: next,
        project: next as unknown as Project,
        _undoStack: [...s._undoStack.slice(-(UNDO_DEPTH - 1)), before],
        _redoStack: [],
      };
    });
  },

  // ── Time Machine ─────────────────────────────────────────
  _previewHeads: null,
  setPreviewHeads: (heads) => {
    set((s) => {
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
    // Un commit Automerge requiert au moins une "modification" — on touche
    // updatedAt pour matérialiser le jalon dans l'historique. Le `message` est
    // ce qui s'affichera dans la Time Machine.
    const trimmed = message.trim() || "Jalon";
    get().mutate(`📌 ${trimmed}`, (d) => { d.updatedAt = Date.now(); });
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
    // 2) Commit qui réécrit tout le contenu pour matcher l'état passé.
    //    L'historique antérieur reste préservé : c'est un commit en avant
    //    qui dit "reviens à l'état du jalon X".
    get().mutate("⏪ Restauration depuis la timeline", (d) => {
      d.name = pastPlain.name;
      d.activeBoardId = pastPlain.activeBoardId;
      d.updatedAt = Date.now();
      // Replace boards array (Automerge accepte splice + spread)
      d.boards.splice(0, d.boards.length, ...pastPlain.boards);
      d.presets.splice(0, d.presets.length, ...pastPlain.presets);
      if (d.domains) d.domains.splice(0, d.domains.length, ...(pastPlain.domains ?? []));
      else d.domains = pastPlain.domains ?? [];
    });
  },

  // ── État UI local (jamais dans le doc) ─────────────────────
  activeTool: "select",
  selectedImageIds: [],
  selectedAnnotationIds: [],
  folderStack: [],
  smartGuidesEnabled: true,
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

  // ── Undo / Redo ───────────────────────────────────────────
  _undoStack: [],
  _redoStack: [],
  pushHistory: () => {
    // Compat : pousse un snapshot du doc actuel dans undoStack sans muter.
    set((s) => ({ _undoStack: [...s._undoStack.slice(-(UNDO_DEPTH - 1)), s._doc], _redoStack: [] }));
  },
  undo: () => {
    set((s) => {
      if (s._undoStack.length === 0) return s;
      const prev = s._undoStack[s._undoStack.length - 1];
      return {
        _doc: prev,
        project: prev as unknown as Project,
        _undoStack: s._undoStack.slice(0, -1),
        _redoStack: [...s._redoStack.slice(-(UNDO_DEPTH - 1)), s._doc],
        // Sortir du preview Time Machine si actif (l'undo est une opération "live")
        _previewHeads: null,
        selectedImageIds: [],
        selectedAnnotationIds: [],
      };
    });
  },
  redo: () => {
    set((s) => {
      if (s._redoStack.length === 0) return s;
      const next = s._redoStack[s._redoStack.length - 1];
      return {
        _doc: next,
        project: next as unknown as Project,
        _redoStack: s._redoStack.slice(0, -1),
        _undoStack: [...s._undoStack.slice(-(UNDO_DEPTH - 1)), s._doc],
        _previewHeads: null,
        selectedImageIds: [],
        selectedAnnotationIds: [],
      };
    });
  },

  // ── Viewport ──────────────────────────────────────────────
  setViewport: (boardId, vp) => {
    const safe = clampViewport(vp);
    get().mutate("setViewport", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (b) b.viewport = safe;
    });
  },

  // ── Images ────────────────────────────────────────────────
  addImage: (boardId, img) => {
    const safe = clampSpatial(img);
    get().mutate("addImage", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      b.images.push(safe);
      b.updatedAt = Date.now();
      d.updatedAt = Date.now();
    });
  },

  updateImage: (boardId, id, patch) => {
    const safe = clampSpatial(patch);
    get().mutate("updateImage", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      const idx = indexById(b.images, id);
      if (idx === -1) return;
      const old = b.images[idx];
      const dx = (safe.x !== undefined) ? safe.x - old.x : 0;
      const dy = (safe.y !== undefined) ? safe.y - old.y : 0;
      Object.assign(b.images[idx], safe);
      // Déplacement induit des flèches attachées
      if (dx || dy) {
        for (const a of b.annotations) {
          if (a.type !== "arrow") continue;
          if (a.sourceId === id) { a.x += dx; a.y += dy; }
          if (a.targetId === id) {
            a.x2 = (a.x2 ?? a.x + 100) + dx;
            a.y2 = (a.y2 ?? a.y) + dy;
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
    // _boardId : conservé pour signature stable, mais la cascade miroirs traverse
    // tous les boards (un miroir peut être ailleurs).
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
      // Suppression dans tous les boards (les miroirs peuvent être ailleurs)
      for (const b of d.boards) {
        const removed = removeWhere(b.images, (img) => toRemove.has(img.id));
        // Flèches orphelines (source ou cible supprimée)
        removeWhere(b.annotations, (a) => {
          if (a.type !== "arrow") return false;
          return (!!a.sourceId && toRemove.has(a.sourceId)) ||
                 (!!a.targetId && toRemove.has(a.targetId));
        });
        if (removed.length > 0) b.updatedAt = Date.now();
      }
    });
    set({ selectedImageIds: [] });
  },

  // ── Sélection ─────────────────────────────────────────────
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
    // Pré-calcule les copies (ids générés hors du mutator)
    const newImages: BoardImage[] = board.images
      .filter((img) => selectedImageIds.includes(img.id))
      .map((img) => ({ ...img, id: nanoid(), x: img.x + OFFSET, y: img.y + OFFSET }));
    const newAnnotations: Annotation[] = board.annotations
      .filter((ann) => selectedAnnotationIds.includes(ann.id))
      .map((ann) => ({ ...ann, id: nanoid(), x: ann.x + OFFSET, y: ann.y + OFFSET }));

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
      // Images sélectionnées
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
        // Flèche dont source/cible attachée bouge
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

  // ── Annotations ───────────────────────────────────────────
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
    get().mutate("updateAnnotation", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b) return;
      const idx = indexById(b.annotations, id);
      if (idx === -1) return;
      const old = b.annotations[idx];
      const dx = (safe.x !== undefined) ? safe.x - old.x : 0;
      const dy = (safe.y !== undefined) ? safe.y - old.y : 0;
      Object.assign(b.annotations[idx], safe);
      if (dx || dy) {
        for (const a of b.annotations) {
          if (a.type !== "arrow" || a.id === id) continue;
          if (a.sourceId === id) { a.x += dx; a.y += dy; }
          if (a.targetId === id) {
            a.x2 = (a.x2 ?? a.x + 100) + dx;
            a.y2 = (a.y2 ?? a.y) + dy;
          }
        }
      }
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

  // ── Storyboard ────────────────────────────────────────────
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
      b.storyboard = undefined;
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

  // ── Boards ────────────────────────────────────────────────
  addBoard: (name) => {
    const board = newBoard(name);
    get().mutate("addBoard", (d) => {
      d.boards.push(board);
      d.activeBoardId = board.id;
      d.updatedAt = Date.now();
    });
    set({ selectedImageIds: [], selectedAnnotationIds: [] });
    return board.id;
  },

  removeBoard: (id) => {
    if (get().project.boards.length <= 1) return;
    get().mutate("removeBoard", (d) => {
      const removeIdx = d.boards.findIndex((b) => b.id === id);
      if (removeIdx === -1) return;
      // Patch les flèches portail orphelines avant de retirer le board
      for (const b of d.boards) {
        for (const a of b.annotations) {
          if (a.type === "arrow" && a.targetBoardId === id) a.targetBoardId = undefined;
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
    set((s) => ({
      selectedImageIds: [],
      selectedAnnotationIds: [],
      folderStack: s.folderStack.filter(
        (f) => f.boardId !== id && s.project.boards.some((b) => b.id === f.boardId)
      ),
    }));
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
      const [moved] = d.boards.splice(fromIndex, 1);
      d.boards.splice(toIndex, 0, moved);
    });
  },

  setActiveBoardId: (id) => {
    // Reconstruit folderStack pour pointer vers le nouveau board s'il est sous-dossier
    const proj = get().project;
    const parentMap = new Map<string, { boardId: string; folderId: string }>();
    for (const b of proj.boards) {
      for (const f of b.folders ?? []) {
        parentMap.set(f.childBoardId, { boardId: b.id, folderId: f.id });
      }
    }
    const stack: Array<{ boardId: string; folderId: string }> = [];
    let curr = id;
    while (parentMap.has(curr)) {
      const parent = parentMap.get(curr)!;
      stack.unshift(parent);
      curr = parent.boardId;
    }
    get().mutate("setActiveBoardId", (d) => { d.activeBoardId = id; });
    set({ selectedImageIds: [], selectedAnnotationIds: [], folderStack: stack });
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
  },

  applyPresetToBoard: (boardId, presetId, worldX, worldY) => {
    const all = get().getAllPresets();
    const preset = presetId ? all.find((p) => p.id === presetId) : null;
    const board = get().project.boards.find((b) => b.id === boardId);
    const vp = board?.viewport ?? { x: 0, y: 0, scale: 1 };
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
      b.presetId = presetId ?? undefined;
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

  // ── Presets ───────────────────────────────────────────────
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

  // ── Domaines (Phase 3) ────────────────────────────────────
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
      // Cascade : retire l'assignation de tous les nœuds
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

  // ── Miroirs (Phase 4) ─────────────────────────────────────
  // Les findOriginal* sont des LECTURES → pas de mutate.
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
    // Snapshot plain (le draft refusera un proxy importé)
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
      console.warn(`[mirrorFolder] Cycle refusé pour "${original.name}"`);
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
      childBoardId: original.childBoardId, // partage le même childBoard
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

  // ── Canvas Folders ────────────────────────────────────────
  // Phase 7.5 — Capture des blocs au drag-create par-dessus.
  createFolder: (parentBoardId, folderData) => {
    const childBoardId = nanoid();
    const folder: CanvasFolder = clampSpatial({
      ...folderData,
      id: folderData.id ?? nanoid(),
      childBoardId,
    });

    // On collecte d'abord les items capturés HORS du mutator (lecture pure)
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
    // Flèches : capturées si les deux extrémités le sont
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
      // 1) Crée le child board
      const childBoard = { ...newBoard(folderData.name), id: childBoardId };
      d.boards.push(childBoard);

      const par = d.boards.find((b) => b.id === parentBoardId);
      if (!par) return;

      // 2) Transfère les images capturées
      const imagesToMove: BoardImage[] = [];
      removeWhere(par.images, (img) => {
        if (capturedImageIds.has(img.id)) {
          imagesToMove.push({ ...img, x: img.x - fx0, y: img.y - fy0 });
          return true;
        }
        return false;
      });
      for (const img of imagesToMove) childBoard.images.push(img);

      // 3) Transfère les sous-folders
      if (par.folders) {
        const foldersToMove: CanvasFolder[] = [];
        removeWhere(par.folders, (f) => {
          if (capturedFolderIds.has(f.id)) {
            foldersToMove.push({ ...f, x: f.x - fx0, y: f.y - fy0 });
            return true;
          }
          return false;
        });
        if (!childBoard.folders) childBoard.folders = [];
        for (const f of foldersToMove) childBoard.folders.push(f);
      }

      // 4) Transfère les annotations capturées
      const annsToMove: Annotation[] = [];
      removeWhere(par.annotations, (a) => {
        if (capturedAnnIds.has(a.id)) {
          annsToMove.push({ ...a, x: a.x - fx0, y: a.y - fy0 });
          return true;
        }
        if (capturedArrowIds.has(a.id) && a.type === "arrow") {
          annsToMove.push({
            ...a,
            x: a.x - fx0, y: a.y - fy0,
            x2: a.x2 - fx0,
            y2: a.y2 - fy0,
            waypoints: a.waypoints?.map((p) => ({ x: p.x - fx0, y: p.y - fy0 })),
          });
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

  updateFolder: (boardId, folderId, patch) => {
    const safe = clampSpatial(patch);
    get().mutate("updateFolder", (d) => {
      const b = d.boards.find((x) => x.id === boardId);
      if (!b || !b.folders) return;
      const idx = indexById(b.folders, folderId);
      if (idx !== -1) Object.assign(b.folders[idx], safe);
      d.updatedAt = Date.now();
    });
  },

  removeFolders: (boardId, folderIds) => {
    // Pré-calcul des childBoardIds à potentiellement supprimer (BFS hors mutator)
    const proj = get().project;
    const parent = proj.boards.find((b) => b.id === boardId);
    const removed = (parent?.folders ?? []).filter((f) => folderIds.includes(f.id));
    const childIds = new Set(removed.map((f) => f.childBoardId));

    get().mutate("removeFolders", (d) => {
      // 1) Retire les folders du parent
      const par = d.boards.find((b) => b.id === boardId);
      if (par?.folders) removeWhere(par.folders, (f) => folderIds.includes(f.id));

      // 2) BFS pour identifier les child boards orphelins (cascade récursive)
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

      // 4) Si le board actif est supprimé, retombe sur le parent
      if (toDelete.has(d.activeBoardId)) d.activeBoardId = boardId;
      d.updatedAt = Date.now();
    });

    // Nettoie le folderStack (état UI)
    set((s) => ({
      folderStack: s.folderStack.filter(
        (entry) => !folderIds.includes(entry.folderId) &&
          get().project.boards.some((b) => b.id === entry.boardId)
      ),
    }));
  },

  enterFolder: (folderId) => {
    const s = get();
    const boardId = s.project.activeBoardId;
    const board = s.project.boards.find((b) => b.id === boardId);
    const folder = (board?.folders ?? []).find((f) => f.id === folderId);
    if (!folder) return;
    const targetBoardId = folder.childBoardId;
    get().mutate("enterFolder", (d) => { d.activeBoardId = targetBoardId; });
    set({
      folderStack: [...s.folderStack, { boardId, folderId }],
      selectedImageIds: [],
      selectedAnnotationIds: [],
    });
  },

  exitFolder: () => {
    const s = get();
    if (s.folderStack.length === 0) return;
    const prev = s.folderStack[s.folderStack.length - 1];
    get().mutate("exitFolder", (d) => { d.activeBoardId = prev.boardId; });
    set({
      folderStack: s.folderStack.slice(0, -1),
      selectedImageIds: [],
      selectedAnnotationIds: [],
    });
  },

  exitToRoot: () => {
    const s = get();
    if (s.folderStack.length === 0) return;
    const root = s.folderStack[0];
    get().mutate("exitToRoot", (d) => { d.activeBoardId = root.boardId; });
    set({
      folderStack: [],
      selectedImageIds: [],
      selectedAnnotationIds: [],
    });
  },

  // ── Project ───────────────────────────────────────────────
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
      _undoStack: [],
      _redoStack: [],
      _previewHeads: null,
      selectedImageIds: [],
      selectedAnnotationIds: [],
      folderStack: [],
    });
  },

  loadDoc: (doc) => {
    set({
      _doc: doc,
      project: doc as unknown as Project,
      _undoStack: [],
      _redoStack: [],
      _previewHeads: null,
      selectedImageIds: [],
      selectedAnnotationIds: [],
      folderStack: [],
    });
  },

  applyRemoteChanges: (changes) => {
    if (changes.length === 0) return;
    set((s) => {
      try {
        const next = A.applyChanges(s._doc, changes);
        if (next === s._doc) return s; // pas de nouveauté (changes déjà connus)
        return {
          _doc: next,
          project: next as unknown as Project,
          // Pas de modification de _undoStack — les actions distantes ne sont pas
          // dans la pile undo locale.
        };
      } catch (e) {
        console.error("[applyRemoteChanges] failed:", e);
        return s;
      }
    });
  },

  // ── Pomodoro ─────────────────────────────────────────────
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
  return project.boards.find((b) => b.id === project.activeBoardId) ?? project.boards[0];
}
