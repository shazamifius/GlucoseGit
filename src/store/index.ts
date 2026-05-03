import { create } from "zustand";
import {
  Annotation, BoardImage, BoardZone, CanvasFolder, Domain, Preset,
  Project, StoryboardPanel, StoryboardSettings, Tool, Viewport
} from "../types";
import type { LOD } from "../canvas/lod";
import { DEFAULT_PRESETS } from "../data/defaultPresets";
import { nanoid } from "../utils/nanoid";
import { wouldCreateMirrorCycle } from "./mirrorGraph";

// CLEANUP R-02 — Bornes pour éviter les coords/scales aberrants qui produisent
// du NaN PixiJS (Float32) ou des fitView impossibles à inverser.
const COORD_LIMIT = 1_000_000; // ±1M pixels world space
const MIN_SCALE = 0.005;
const MAX_SCALE = 50;
const clampNum = (v: number, min: number, max: number) =>
  Number.isFinite(v) ? Math.min(max, Math.max(min, v)) : 0;
const clampViewport = (vp: Viewport): Viewport => ({
  x: clampNum(vp.x, -COORD_LIMIT * 50, COORD_LIMIT * 50),
  y: clampNum(vp.y, -COORD_LIMIT * 50, COORD_LIMIT * 50),
  scale: clampNum(vp.scale, MIN_SCALE, MAX_SCALE) || 1,
});

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

const DEFAULT_PROJECT: Project = {
  version: "1.0.0",
  name: "Nouveau projet",
  boards: [{ ...newBoard("Board principal"), id: "main" }],
  activeBoardId: "main",
  presets: [],
  domains: [],
  createdAt: Date.now(),
  updatedAt: Date.now(),
};

interface GlucoseStore {
  project: Project;
  activeTool: Tool;
  selectedImageIds: string[];
  selectedAnnotationIds: string[];
  folderStack: Array<{ boardId: string; folderId: string }>;

  // ── Undo / Redo ───────────────────────────────────────────
  _history: Project[];
  _future:  Project[];
  pushHistory: () => void;
  undo: () => void;
  redo: () => void;

  setViewport: (boardId: string, vp: Viewport) => void;

  // ── Images ────────────────────────────────────────────────
  addImage: (boardId: string, img: BoardImage) => void;
  updateImage: (boardId: string, id: string, patch: Partial<BoardImage>) => void;
  removeImages: (boardId: string, ids: string[]) => void;
  updateMultipleImages: (boardId: string, updates: { id: string; patch: Partial<BoardImage> }[]) => void;

  // ── Selection ─────────────────────────────────────────────
  setSelectedImageIds: (ids: string[]) => void;
  setSelectedAnnotationIds: (ids: string[]) => void;
  selectAll: (boardId: string) => void;
  deleteSelected: (boardId: string) => void;
  duplicateSelected: (boardId: string) => void;
  moveSelected: (boardId: string, dx: number, dy: number) => void;

  // ── Tool ──────────────────────────────────────────────────
  setActiveTool: (tool: Tool) => void;

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
  // Assigne (ou retire si weight = 0) un domaine à un nœud annotation/image
  assignDomainToNode: (boardId: string, nodeId: string, domainId: string, weight: number) => void;

  // ── Miroirs / Alias (Phase 4) ─────────────────────────────
  // Crée un miroir d'une annotation à (x, y) sur le même board ou un autre.
  mirrorAnnotation: (boardId: string, originalId: string, x: number, y: number) => string | null;
  mirrorImage: (boardId: string, originalId: string, x: number, y: number) => string | null;
  // Crée un miroir d'un dossier — RETOURNE null si la création créerait un cycle.
  mirrorFolder: (parentBoardId: string, originalFolderId: string, x: number, y: number) => string | null;
  // Trouve l'annotation/image/dossier original à partir d'un id (suit la chaîne de mirrorOf jusqu'à la racine).
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
  loadProject: (project: Project) => void;

  // ── Pomodoro ─────────────────────────────────────────────
  pomodoroTotal: number;
  pomodoroLeft: number;
  pomodoroRunning: boolean;
  pomodoroDone: boolean;
  pomodoroStart: () => void;
  pomodoroPause: () => void;
  pomodoroReset: (total: number) => void;
  smartGuidesEnabled: boolean;
  toggleSmartGuides: () => void;

  // ── UI : panel droit ouvert (Phase 4) — la minimap se décale en conséquence
  rightPanelOpen: boolean;
  setRightPanelOpen: (open: boolean) => void;

  // ── LOD (Phase 2 : zoom sémantique) ───────────────────────
  currentLod: LOD;
  setCurrentLod: (lod: LOD) => void;
  hoveredNodeId: string | null;        // id annotation/image sous le curseur (null sinon)
  setHoveredNodeId: (id: string | null) => void;
  transDomainVisible: boolean;          // toggle "afficher liens trans-domaines"
  toggleTransDomainVisible: () => void;
}

// ── Pomodoro module-level helpers (survive panel unmount) ──────
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

export const useGlucoseStore = create<GlucoseStore>((set, get) => ({
  project: DEFAULT_PROJECT,
  activeTool: "select",
  selectedImageIds: [],
  selectedAnnotationIds: [],
  folderStack: [],
  smartGuidesEnabled: true,
  currentLod: "micro",
  setCurrentLod: (lod) => {
    if (get().currentLod !== lod) set({ currentLod: lod });
  },
  hoveredNodeId: null,
  setHoveredNodeId: (id) => {
    if (get().hoveredNodeId !== id) set({ hoveredNodeId: id });
  },
  transDomainVisible: true,
  toggleTransDomainVisible: () => set((s) => ({ transDomainVisible: !s.transDomainVisible })),
  rightPanelOpen: false,
  setRightPanelOpen: (open) => {
    if (get().rightPanelOpen !== open) set({ rightPanelOpen: open });
  },

  _history: [],
  _future: [],

  pushHistory: () => {
    const { project, _history } = get();
    set({ _history: [..._history.slice(-49), project], _future: [] });
  },

  undo: () => {
    const { project, _history, _future } = get();
    if (_history.length === 0) return;
    const prev = _history[_history.length - 1];
    set({
      project: prev,
      _history: _history.slice(0, -1),
      _future: [project, ..._future.slice(0, 49)],
      selectedImageIds: [],
      selectedAnnotationIds: [],
    });
  },

  redo: () => {
    const { project, _history, _future } = get();
    if (_future.length === 0) return;
    const next = _future[0];
    set({
      project: next,
      _history: [..._history.slice(-49), project],
      _future: _future.slice(1),
      selectedImageIds: [],
      selectedAnnotationIds: [],
    });
  },

  setViewport: (boardId, vp) => set((s) => ({
    // CLEANUP R-02 : clamp défensif avant persistence
    project: { ...s.project, boards: s.project.boards.map((b) => b.id === boardId ? { ...b, viewport: clampViewport(vp) } : b) },
  })),

  addImage: (boardId, img) => {
    get().pushHistory();
    set((s) => ({
      project: {
        ...s.project,
        boards: s.project.boards.map((b) =>
          b.id === boardId ? { ...b, images: [...b.images, img], updatedAt: Date.now() } : b
        ),
        updatedAt: Date.now(),
      },
    }));
  },

  updateImage: (boardId, id, patch) => set((s) => {
    const board = s.project.boards.find((b) => b.id === boardId);
    const oldImg = board?.images.find((img) => img.id === id);
    const deltaX = (oldImg && patch.x !== undefined) ? patch.x - oldImg.x : 0;
    const deltaY = (oldImg && patch.y !== undefined) ? patch.y - oldImg.y : 0;
    return {
      project: {
        ...s.project,
        boards: s.project.boards.map((b) => {
          if (b.id !== boardId) return b;
          const images = b.images.map((img) => img.id === id ? { ...img, ...patch } : img);
          const annotations = (deltaX || deltaY)
            ? b.annotations.map((a) => {
                if (a.type !== "arrow") return a;
                let upd = a;
                if (a.sourceId === id) upd = { ...upd, x: a.x + deltaX, y: a.y + deltaY };
                if (a.targetId === id) upd = { ...upd, x2: (a.x2 ?? a.x + 100) + deltaX, y2: (a.y2 ?? a.y) + deltaY };
                return upd;
              })
            : b.annotations;
          return { ...b, images, annotations, updatedAt: Date.now() };
        }),
      },
    };
  }),

  removeImages: (boardId, ids) => {
    get().pushHistory();
    set((s) => ({
      project: {
        ...s.project,
        boards: s.project.boards.map((b) => {
          if (b.id !== boardId) return b;
          return {
            ...b,
            images: b.images.filter((img) => !ids.includes(img.id)),
            annotations: (b.annotations || []).filter((a) => {
              if (a.type === "arrow" && ((a.sourceId && ids.includes(a.sourceId)) || (a.targetId && ids.includes(a.targetId)))) {
                return false;
              }
              return true;
            }),
            updatedAt: Date.now()
          };
        }),
      },
    }));
  },

  updateMultipleImages: (boardId, updates) => {
    get().pushHistory();
    const map = new Map(updates.map((u) => [u.id, u.patch]));
    set((s) => ({
      project: {
        ...s.project,
        boards: s.project.boards.map((b) =>
          b.id === boardId
            ? { ...b, images: b.images.map((img) => map.has(img.id) ? { ...img, ...map.get(img.id) } : img), updatedAt: Date.now() }
            : b
        ),
      },
    }));
  },

  setSelectedImageIds: (ids) => set({ selectedImageIds: ids }),
  setSelectedAnnotationIds: (ids) => set({ selectedAnnotationIds: ids }),

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
    if (selectedImageIds.length > 0) {
      get().removeImages(boardId, selectedImageIds);
      set({ selectedImageIds: [] });
    }
    if (selectedAnnotationIds.length > 0) {
      get().removeAnnotations(boardId, selectedAnnotationIds);
      set({ selectedAnnotationIds: [] });
    }
  },

  duplicateSelected: (boardId) => {
    const { selectedImageIds, selectedAnnotationIds } = get();
    const board = get().project.boards.find((b) => b.id === boardId);
    if (!board) return;
    get().pushHistory();
    const OFFSET = 20;
    const newImages = board.images
      .filter((img) => selectedImageIds.includes(img.id))
      .map((img) => ({ ...img, id: nanoid(), x: img.x + OFFSET, y: img.y + OFFSET }));
    const newAnnotations = board.annotations
      .filter((ann) => selectedAnnotationIds.includes(ann.id))
      .map((ann) => ({ ...ann, id: nanoid(), x: ann.x + OFFSET, y: ann.y + OFFSET }));
    set((s) => ({
      project: {
        ...s.project,
        boards: s.project.boards.map((b) =>
          b.id === boardId ? {
            ...b,
            images: [...b.images, ...newImages],
            annotations: [...b.annotations, ...newAnnotations],
            updatedAt: Date.now(),
          } : b
        ),
      },
      selectedImageIds: newImages.map((i) => i.id),
      selectedAnnotationIds: newAnnotations.map((a) => a.id),
    }));
  },

  moveSelected: (boardId, dx, dy) => set((s) => {
    const { selectedImageIds, selectedAnnotationIds } = s;
    const board = s.project.boards.find((b) => b.id === boardId);
    if (!board) return s;

    const newImages = board.images.map((img) =>
      selectedImageIds.includes(img.id) ? { ...img, x: img.x + dx, y: img.y + dy } : img
    );

    const newAnnotations = board.annotations.map((ann) => {
      let nx = ann.x;
      let ny = ann.y;
      let nx2 = ann.x2;
      let ny2 = ann.y2;

      const isSelected = selectedAnnotationIds.includes(ann.id);
      if (isSelected) {
        nx += dx; ny += dy;
        if (nx2 !== undefined) nx2 += dx;
        if (ny2 !== undefined) ny2 += dy;
        const waypoints = ann.waypoints?.map(wp => ({ x: wp.x + dx, y: wp.y + dy }));
        return { ...ann, x: nx, y: ny, x2: nx2, y2: ny2, waypoints };
      }

      // Si la flèche n'est PAS sélectionnée mais ses éléments attachés le sont, on la déplace aussi
      if (ann.type === "arrow") {
        let moved = false;
        if (ann.sourceId && (selectedImageIds.includes(ann.sourceId) || selectedAnnotationIds.includes(ann.sourceId))) {
          nx += dx; ny += dy; moved = true;
        }
        if (ann.targetId && (selectedImageIds.includes(ann.targetId) || selectedAnnotationIds.includes(ann.targetId))) {
          nx2 = (nx2 ?? ann.x + 100) + dx; ny2 = (ny2 ?? ann.y) + dy; moved = true;
        }
        if (moved) return { ...ann, x: nx, y: ny, x2: nx2, y2: ny2 };
      }
      return ann;
    });

    return {
      project: {
        ...s.project,
        boards: s.project.boards.map((b) =>
          b.id === boardId ? { ...b, images: newImages, annotations: newAnnotations, updatedAt: Date.now() } : b
        ),
      },
    };
  }),

  setActiveTool: (tool) => set({ activeTool: tool }),

  // ── Annotations ───────────────────────────────────────────
  addAnnotation: (boardId, ann) => {
    get().pushHistory();
    set((s) => ({
      project: {
        ...s.project,
        boards: s.project.boards.map((b) =>
          b.id === boardId ? { ...b, annotations: [...(b.annotations || []), ann], updatedAt: Date.now() } : b
        ),
      },
    }));
  },

  updateAnnotation: (boardId, id, patch) => set((s) => {
    const board = s.project.boards.find((b) => b.id === boardId);
    const oldAnn = board?.annotations.find((a) => a.id === id);
    const deltaX = (oldAnn && patch.x !== undefined) ? patch.x - oldAnn.x : 0;
    const deltaY = (oldAnn && patch.y !== undefined) ? patch.y - oldAnn.y : 0;
    return {
      project: {
        ...s.project,
        boards: s.project.boards.map((b) => {
          if (b.id !== boardId) return b;
          const annotations = b.annotations.map((a) => {
            if (a.id === id) return { ...a, ...patch };
            if (a.type === "arrow" && (deltaX || deltaY)) {
              let upd = a;
              if (a.sourceId === id) upd = { ...upd, x: a.x + deltaX, y: a.y + deltaY };
              if (a.targetId === id) upd = { ...upd, x2: (a.x2 ?? a.x + 100) + deltaX, y2: (a.y2 ?? a.y) + deltaY };
              return upd;
            }
            return a;
          });
          return { ...b, annotations };
        }),
      },
    };
  }),

  removeAnnotations: (boardId, ids) => {
    get().pushHistory();
    set((s) => ({
      project: {
        ...s.project,
        boards: s.project.boards.map((b) => {
          if (b.id !== boardId) return b;
          return {
            ...b,
            annotations: (b.annotations || []).filter((a) => {
              if (ids.includes(a.id)) return false;
              if (a.type === "arrow" && ((a.sourceId && ids.includes(a.sourceId)) || (a.targetId && ids.includes(a.targetId)))) {
                return false;
              }
              return true;
            })
          };
        }),
      },
    }));
  },

  // ── Storyboard ────────────────────────────────────────────
  setStoryboardSettings: (boardId, settings) => set((s) => ({
    project: {
      ...s.project,
      boards: s.project.boards.map((b) => b.id === boardId ? { ...b, storyboard: settings } : b),
    },
  })),

  clearStoryboard: (boardId) => set((s) => ({
    project: {
      ...s.project,
      boards: s.project.boards.map((b) => b.id === boardId ? { ...b, storyboard: undefined, panels: [] } : b),
    },
  })),

  addPanel: (boardId, panel) => set((s) => ({
    project: {
      ...s.project,
      boards: s.project.boards.map((b) =>
        b.id === boardId ? { ...b, panels: [...(b.panels || []), panel], updatedAt: Date.now() } : b
      ),
    },
  })),

  updatePanel: (boardId, id, patch) => set((s) => ({
    project: {
      ...s.project,
      boards: s.project.boards.map((b) =>
        b.id === boardId
          ? { ...b, panels: (b.panels || []).map((p) => p.id === id ? { ...p, ...patch } : p) }
          : b
      ),
    },
  })),

  removePanel: (boardId, id) => set((s) => ({
    project: {
      ...s.project,
      boards: s.project.boards.map((b) =>
        b.id === boardId ? { ...b, panels: (b.panels || []).filter((p) => p.id !== id) } : b
      ),
    },
  })),

  reorderPanels: (boardId) => set((s) => ({
    project: {
      ...s.project,
      boards: s.project.boards.map((b) =>
        b.id === boardId
          ? { ...b, panels: (b.panels || []).map((p, i) => ({ ...p, order: i })) }
          : b
      ),
    },
  })),

  // ── Boards ────────────────────────────────────────────────
  addBoard: (name) => {
    const board = newBoard(name);
    set((s) => ({
      project: { ...s.project, boards: [...s.project.boards, board], activeBoardId: board.id, updatedAt: Date.now() },
      selectedImageIds: [], selectedAnnotationIds: [],
    }));
    return board.id;
  },

  removeBoard: (id) => {
    get().pushHistory();
    set((s) => {
      if (s.project.boards.length <= 1) return s;
      const boards = s.project.boards.filter((b) => b.id !== id);
      const activeBoardId = s.project.activeBoardId === id
        ? (boards[Math.max(0, s.project.boards.findIndex((b) => b.id === id) - 1)] ?? boards[0]).id
        : s.project.activeBoardId;
      const folderStack = s.folderStack.filter(
        (f) => f.boardId !== id && boards.some((b) => b.id === f.boardId)
      );
      return { project: { ...s.project, boards, activeBoardId, updatedAt: Date.now() }, selectedImageIds: [], selectedAnnotationIds: [], folderStack };
    });
  },

  renameBoard: (id, name) => set((s) => ({
    project: { ...s.project, boards: s.project.boards.map((b) => b.id === id ? { ...b, name } : b) },
  })),

  reorderBoards: (fromIndex, toIndex) => set((s) => {
    const boards = [...s.project.boards];
    const [moved] = boards.splice(fromIndex, 1);
    boards.splice(toIndex, 0, moved);
    return { project: { ...s.project, boards } };
  }),

  setActiveBoardId: (id) => set((s) => {
    // Reconstruire le folderStack pour correspondre au board cible (si c'est un sous-dossier, on restaure le chemin, sinon on le vide)
    const parentMap = new Map<string, { boardId: string; folderId: string }>();
    for (const b of s.project.boards) {
      for (const f of b.folders ?? []) {
        parentMap.set(f.childBoardId, { boardId: b.id, folderId: f.id });
      }
    }
    const stack = [];
    let curr = id;
    while (parentMap.has(curr)) {
      const parent = parentMap.get(curr)!;
      stack.unshift(parent);
      curr = parent.boardId;
    }
    
    return {
      project: { ...s.project, activeBoardId: id },
      selectedImageIds: [], selectedAnnotationIds: [],
      folderStack: stack,
    };
  }),

  duplicateBoard: (id) => set((s) => {
    const src = s.project.boards.find((b) => b.id === id);
    if (!src) return s;
    const copy = { ...src, id: nanoid(), name: `${src.name} (copie)`, createdAt: Date.now(), updatedAt: Date.now() };
    return { project: { ...s.project, boards: [...s.project.boards, copy], activeBoardId: copy.id } };
  }),

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
      // Si worldX/worldY fournis : centrer la grille sur ce point
      // Sinon : coin supérieur-gauche du viewport
      const offsetX = worldX !== undefined ? worldX - totalW / 2 : (-vp.x / vp.scale);
      const offsetY = worldY !== undefined ? worldY - ZONE_H / 2  : (-vp.y / vp.scale);
      zones = preset.slots.map((slot, i) => ({
        slotId: slot.id, x: offsetX + i * (ZONE_W + GAP), y: offsetY, width: ZONE_W, height: ZONE_H,
      }));
    }
    set((s) => ({
      project: {
        ...s.project,
        boards: s.project.boards.map((b) =>
          b.id === boardId ? { ...b, presetId: presetId ?? undefined, zones } : b
        ),
        updatedAt: Date.now(),
      },
    }));
  },

  setBoardZones: (boardId, zones) => set((s) => ({
    project: { ...s.project, boards: s.project.boards.map((b) => b.id === boardId ? { ...b, zones } : b) },
  })),

  addPreset: (preset) => set((s) => ({
    project: { ...s.project, presets: [...s.project.presets, preset] },
  })),

  removePreset: (id) => set((s) => ({
    project: { ...s.project, presets: s.project.presets.filter((p) => p.id !== id) },
  })),

  updatePreset: (id, patch) => set((s) => ({
    project: { ...s.project, presets: s.project.presets.map((p) => p.id === id ? { ...p, ...patch } : p) },
  })),

  getAllPresets: () => [...DEFAULT_PRESETS, ...get().project.presets],

  // ── Domaines (Phase 3) ────────────────────────────────────────────────────
  addDomain: (domain) => {
    get().pushHistory();
    set((s) => ({
      project: { ...s.project, domains: [...(s.project.domains ?? []), domain], updatedAt: Date.now() },
    }));
  },
  updateDomain: (id, patch) => {
    get().pushHistory();
    set((s) => ({
      project: {
        ...s.project,
        domains: (s.project.domains ?? []).map((d) => d.id === id ? { ...d, ...patch } : d),
        updatedAt: Date.now(),
      },
    }));
  },
  removeDomain: (id) => {
    get().pushHistory();
    set((s) => ({
      project: {
        ...s.project,
        domains: (s.project.domains ?? []).filter((d) => d.id !== id),
        // On retire aussi l'assignation de tous les nœuds qui portaient ce domaine
        boards: s.project.boards.map((b) => ({
          ...b,
          annotations: b.annotations.map((a) => a.domains
            ? { ...a, domains: a.domains.filter((da) => da.domainId !== id) }
            : a),
          images: b.images.map((img) => img.domains
            ? { ...img, domains: img.domains.filter((da) => da.domainId !== id) }
            : img),
        })),
        updatedAt: Date.now(),
      },
    }));
  },
  getDomains: () => get().project.domains ?? [],
  assignDomainToNode: (boardId, nodeId, domainId, weight) => {
    get().pushHistory();
    const upsert = (current: { domainId: string; weight: number }[] | undefined) => {
      const arr = current ?? [];
      if (weight <= 0) return arr.filter((d) => d.domainId !== domainId);
      const idx = arr.findIndex((d) => d.domainId === domainId);
      if (idx === -1) return [...arr, { domainId, weight }];
      const next = [...arr];
      next[idx] = { domainId, weight };
      return next;
    };
    set((s) => ({
      project: {
        ...s.project,
        boards: s.project.boards.map((b) => b.id !== boardId ? b : {
          ...b,
          annotations: b.annotations.map((a) => a.id === nodeId ? { ...a, domains: upsert(a.domains) } : a),
          images: b.images.map((img) => img.id === nodeId ? { ...img, domains: upsert(img.domains) } : img),
        }),
        updatedAt: Date.now(),
      },
    }));
  },

  // ── Miroirs / Alias (Phase 4) ─────────────────────────────────────────────
  // CLEANUP R-03 : signale via console.error quand le garde-fou anti-cycle est
  // déclenché — symptôme d'une chaîne de mirrorOf cassée.
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
    if (safety <= 0) {
      console.error(`[findOriginalAnnotation] Chain too deep for ${id} — possible cycle`);
    }
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
    if (safety <= 0) {
      console.error(`[findOriginalImage] Chain too deep for ${id} — possible cycle`);
    }
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
    if (safety <= 0) {
      console.error(`[findOriginalFolder] Chain too deep for ${id} — possible cycle`);
    }
    return cur;
  },

  mirrorAnnotation: (boardId, originalId, x, y) => {
    const original = get().findOriginalAnnotation(originalId);
    if (!original) return null;
    const newId = nanoid();
    get().pushHistory();
    set((s) => ({
      project: {
        ...s.project,
        boards: s.project.boards.map((b) => b.id !== boardId ? b : {
          ...b,
          annotations: [...b.annotations, {
            ...original,
            id: newId,
            x, y,
            mirrorOf: original.id, // pointe TOUJOURS vers la racine, pas vers un autre miroir
          }],
        }),
        updatedAt: Date.now(),
      },
    }));
    return newId;
  },

  mirrorImage: (boardId, originalId, x, y) => {
    const original = get().findOriginalImage(originalId);
    if (!original) return null;
    const newId = nanoid();
    get().pushHistory();
    set((s) => ({
      project: {
        ...s.project,
        boards: s.project.boards.map((b) => b.id !== boardId ? b : {
          ...b,
          images: [...b.images, {
            ...original,
            id: newId,
            x, y,
            mirrorOf: original.id,
          }],
        }),
        updatedAt: Date.now(),
      },
    }));
    return newId;
  },

  mirrorFolder: (parentBoardId, originalFolderId, x, y) => {
    const original = get().findOriginalFolder(originalFolderId);
    if (!original) return null;

    // ⚠️ Vérification acyclique stricte AVANT toute mutation.
    // Si placer un miroir de `original` dans `parentBoardId` créerait un cycle
    // (Inception : A contient B contient A), on refuse net.
    if (wouldCreateMirrorCycle(get().project.boards, original.id, parentBoardId)) {
      console.warn(`[mirrorFolder] Cycle refusé : miroir de "${original.name}" dans le board ${parentBoardId} créerait une boucle Inception.`);
      return null;
    }

    const newId = nanoid();
    const mirrorFolder: CanvasFolder = {
      id: newId,
      name: original.name,
      color: original.color,
      x, y,
      width: original.width,
      height: original.height,
      childBoardId: original.childBoardId, // CRITIQUE : on partage le même childBoard que l'original
      mirrorOf: original.id,
    };
    get().pushHistory();
    set((s) => ({
      project: {
        ...s.project,
        boards: s.project.boards.map((b) => b.id !== parentBoardId ? b : {
          ...b,
          folders: [...(b.folders ?? []), mirrorFolder],
        }),
        updatedAt: Date.now(),
      },
    }));
    return newId;
  },

  // ── Canvas Folders ────────────────────────────────────────────────────────
  createFolder: (parentBoardId, folderData) => {
    get().pushHistory();
    const childBoardId = nanoid();
    const childBoard = { ...newBoard(folderData.name), id: childBoardId };
    const folder: CanvasFolder = { ...folderData, id: folderData.id ?? nanoid(), childBoardId };
    set((s) => ({
      project: {
        ...s.project,
        boards: [
          ...s.project.boards.map((b) =>
            b.id === parentBoardId
              ? { ...b, folders: [...(b.folders ?? []), folder] }
              : b
          ),
          childBoard,
        ],
        updatedAt: Date.now(),
      },
    }));
  },

  updateFolder: (boardId, folderId, patch) => set((s) => ({
    project: {
      ...s.project,
      boards: s.project.boards.map((b) =>
        b.id === boardId
          ? { ...b, folders: (b.folders ?? []).map((f) => f.id === folderId ? { ...f, ...patch } : f) }
          : b
      ),
      updatedAt: Date.now(),
    },
  })),

  removeFolders: (boardId, folderIds) => {
    get().pushHistory();
    return set((s) => ({
    project: {
      ...s.project,
      boards: s.project.boards.map((b) =>
        b.id === boardId
          ? { ...b, folders: (b.folders ?? []).filter((f) => !folderIds.includes(f.id)) }
          : b
      ),
      updatedAt: Date.now(),
    },
  }));
  },

  enterFolder: (folderId) => {
    const s = get();
    const boardId = s.project.activeBoardId;
    const board = s.project.boards.find((b) => b.id === boardId);
    const folder = (board?.folders ?? []).find((f) => f.id === folderId);
    if (!folder) return;
    set({
      folderStack: [...s.folderStack, { boardId, folderId }],
      project: { ...s.project, activeBoardId: folder.childBoardId },
      selectedImageIds: [],
      selectedAnnotationIds: [],
    });
  },

  exitFolder: () => {
    const s = get();
    if (s.folderStack.length === 0) return;
    const prev = s.folderStack[s.folderStack.length - 1];
    set({
      folderStack: s.folderStack.slice(0, -1),
      project: { ...s.project, activeBoardId: prev.boardId },
      selectedImageIds: [],
      selectedAnnotationIds: [],
    });
  },

  exitToRoot: () => {
    const s = get();
    if (s.folderStack.length === 0) return;
    const root = s.folderStack[0];
    set({
      folderStack: [],
      project: { ...s.project, activeBoardId: root.boardId },
      selectedImageIds: [],
      selectedAnnotationIds: [],
    });
  },

  setProjectName: (name) => set((s) => ({ project: { ...s.project, name, updatedAt: Date.now() } })),

  loadProject: (project) => set({
    // Normalise les nouveaux champs absents des projets legacy (Phase 3 : domains)
    project: { ...project, domains: project.domains ?? [] },
    selectedImageIds: [],
    selectedAnnotationIds: [],
  }),

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
  toggleSmartGuides: () => set((s) => ({ smartGuidesEnabled: !s.smartGuidesEnabled })),
}));

export function getActiveBoard(project: Project) {
  return project.boards.find((b) => b.id === project.activeBoardId) ?? project.boards[0];
}
