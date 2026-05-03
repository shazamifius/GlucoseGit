// ── Domaines (Phase 3) ────────────────────────────────────────
// Un domaine est une catégorie sémantique (Science, Art, JV, Conlang…). Un nœud
// peut appartenir à plusieurs domaines avec un poids relatif (somme arbitraire).
// Les couleurs des membranes dérivent de cette pondération.
export interface Domain {
  id: string;
  name: string;
  color: string;        // couleur primaire (HSL/hex)
  icon: string;         // emoji ou symbole (1-2 chars) — affiché dans le badge
  createdAt: number;
}

export interface DomainAssignment {
  domainId: string;
  weight: number;       // 0..1 — poids relatif dans la signature du nœud
}

// ── Images ────────────────────────────────────────────────────
export interface BoardImage {
  id: string;
  src: string;
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
  locked: boolean;
  tags: string[];
  slotId?: string;
  sourceUrl?: string;
  originalWidth: number;
  originalHeight: number;
  isVideo?: boolean;
  domains?: DomainAssignment[]; // Phase 3
  mirrorOf?: string;            // Phase 4 — id de l'image originale (alias / lien vivant)
}

// ── Annotations ───────────────────────────────────────────────
export type AnnotationType = "text" | "sticky" | "arrow" | "membrane";

export type ArrowPredicate =
  | "est_precurseur"
  | "contredit"
  | "herite_de"
  | "inspire"
  | "depend_de"
  | "illustre";

export interface Annotation {
  id: string;
  type: AnnotationType;
  x: number;
  y: number;
  text?: string;
  fontSize?: number;
  color?: string;        // text / arrow color
  bgColor?: string;      // background (sticky)
  width?: number;
  height?: number;
  x2?: number;           // arrow end point
  y2?: number;
  arrowType?: "straight" | "curved";
  arrowBidirectional?: boolean;
  predicate?: ArrowPredicate;
  strokeWidth?: number;
  waypoints?: { x: number; y: number }[];  // points de passage intermédiaires
  sourceId?: string;  // ID image/annotation attachée au début de la flèche
  targetId?: string;  // ID image/annotation attachée à la fin
  sourceBlockId?: string; // Sous-bloc (paragraphe) de départ
  targetBlockId?: string; // Sous-bloc (paragraphe) d'arrivée
  // ── Sélection de texte précise (mode édition flèche) ──
  sourceTextSel?: string;  // Texte exact sélectionné côté source
  targetTextSel?: string;  // Texte exact sélectionné côté cible
  sourceFile?: string; // App Bridge — chemin absolu vers le fichier source
  cursorPos?: number; // Dernière position du curseur
  pinned?: boolean; // Flèche épinglée → toujours visible quel que soit le LOD (Phase 2)
  domains?: DomainAssignment[]; // Phase 3
  mirrorOf?: string;            // Phase 4 — id de l'annotation originale (alias / lien vivant)
  // Phase 5 — Flèches Sémantiques Premium
  longText?: string;            // Description longue Markdown attachée à une flèche (ouvre un panneau coulissant)
  targetBoardId?: string;       // Si défini, la flèche pointe vers un nœud d'un autre board (mode portail)
  operator?: "AND" | "OR" | "BUT" | "BECAUSE"; // Sticky-opérateur logique (type === "sticky")
}

// ── Storyboard ────────────────────────────────────────────────
export type AspectRatio = "16:9" | "4:3" | "2.35:1" | "1:1" | "9:16";

export interface StoryboardPanel {
  id: string;
  order: number;
  description: string;
  imageId?: string;     // image assigned to this panel
  x: number;
  y: number;
  width: number;
  height: number;       // frame only, not including description area
}

export interface StoryboardSettings {
  aspectRatio: AspectRatio;
  panelWidth: number;
  cols: number;
  gap: number;
}

// ── Presets ───────────────────────────────────────────────────
export interface PresetSlot {
  id: string;
  name: string;
  color: string;
  description: string;
  order: number;
}

export interface Preset {
  id: string;
  name: string;
  description: string;
  slots: PresetSlot[];
  isBuiltin: boolean;
  createdAt: number;
}

export interface BoardZone {
  slotId: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

// ── Canvas Folders (sous-canvases imbriqués) ──────────────────
export interface CanvasFolder {
  id: string;
  name: string;
  color: string;
  x: number;
  y: number;
  width: number;
  height: number;
  childBoardId: string;        // Important : un dossier-miroir partage le SAME childBoardId
                               // que l'original → toute mutation propage automatiquement.
  mirrorOf?: string;           // Phase 4 — id du dossier original (alias)
}

// ── Boards ────────────────────────────────────────────────────
export interface Board {
  id: string;
  name: string;
  images: BoardImage[];
  annotations: Annotation[];
  panels: StoryboardPanel[];
  storyboard?: StoryboardSettings;
  viewport: Viewport;
  presetId?: string;
  zones: BoardZone[];
  folders: CanvasFolder[];
  bookmarks?: Record<string, Viewport>; // "1"–"9" → saved viewport
  createdAt: number;
  updatedAt: number;
}

export interface Viewport {
  x: number;
  y: number;
  scale: number;
}

// ── Project ───────────────────────────────────────────────────
export interface Project {
  version: string;
  name: string;
  boards: Board[];
  activeBoardId: string;
  presets: Preset[];
  domains?: Domain[]; // Phase 3 — partagé entre tous les boards
  createdAt: number;
  updatedAt: number;
}

export type Tool = "select" | "pan" | "text" | "sticky" | "arrow" | "folder" | "membrane" | "zone-select";
