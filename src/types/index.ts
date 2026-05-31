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

// ── Ancrage temporel (Phase 6) ────────────────────────────────
// Date du CONTENU décrit par le nœud (pas date d'édition).
// Année calendaire entière, négatif pour avant J.-C. (ex: -3000 = 3000 BC).
// Une plage [start, end] représente un intervalle (ex: Renaissance 1400..1600).
// Pour un point unique, start == end. `label` est optionnel et purement décoratif.
export interface TemporalAnchor {
  start: number;        // année (peut être négative)
  end: number;          // année — toujours >= start
  label?: string;       // texte affiché (ex: "Révolution française")
}

// ── Assets (R-EMB-01 — Sprint 2) ──────────────────────────────
//
// Un asset est une payload binaire (image, vidéo, fichier futur). Deux modes :
//
//   - `embed` : les octets vivent dans `Project.blobs[sha256]`. C'est le mode
//      par défaut désormais : copier le `.glucose` ailleurs préserve tout.
//      La dédup par sha256 est NATIVE — Automerge stocke un blob N×
//      référencé une seule fois.
//
//   - `link`  : référence externe par URL/chemin. Utilisé pour :
//        • images web (http(s)://...)
//        • fichiers de très gros volume qu'on ne veut pas embeded
//          (folder-mirror R-FIL-02, vidéos lourdes, etc.)
//
// La migration legacy `src: "asset:..."` / `src: "data:..."` / `src: "http..."`
// est faite à `loadProject` (cf. `utils/projectMigration.ts`).
export type AssetRef =
  | { mode: "embed"; sha256: string; mime: string; sizeBytes?: number }
  | { mode: "link"; href: string; sha256?: string; sizeBytes?: number };

export function isEmbedAsset(
  a: AssetRef
): a is Extract<AssetRef, { mode: "embed" }> {
  return a.mode === "embed";
}
export function isLinkAsset(
  a: AssetRef
): a is Extract<AssetRef, { mode: "link" }> {
  return a.mode === "link";
}

// ── Images ────────────────────────────────────────────────────
export interface BoardImage {
  id: string;
  /** Référence à l'asset binaire (image/vidéo). Source de vérité Sprint 2+. */
  asset?: AssetRef;
  /** Legacy field : `asset:<file>` | `data:...` | `http(s)://...`.
   *  Migré vers `asset` au load par `migrateImagesAssets`. Conservé en
   *  fallback pour les rendus en transition tant qu'il existe.
   *  ⚠️ Ne plus écrire dans ce champ — utiliser `asset`. */
  src?: string;
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
  temporalAnchor?: TemporalAnchor; // Phase 6 — date du contenu décrit
}

// ── Annotations ───────────────────────────────────────────────
//
// R-TYP-01 (Sprint 1) — Union discriminée stricte par `type`.
// Le narrowing TypeScript s'active à chaque `switch(ann.type)` ou
// `if (ann.type === "arrow")` : plus besoin de `ann.x2!` ni de
// `ann.predicate ?? ...` partout.
//
// Pour les listes hétérogènes, les helpers `isText / isSticky / isArrow /
// isMembrane` permettent de filtrer-narrower en une passe.

export type AnnotationType = "text" | "sticky" | "arrow" | "membrane";

export type ArrowPredicate =
  | "est_precurseur"
  | "contredit"
  | "herite_de"
  | "inspire"
  | "depend_de"
  | "illustre";

/** Champs communs à toutes les annotations. */
interface AnnotationBase {
  id: string;
  x: number;
  y: number;
  domains?: DomainAssignment[];     // Phase 3
  mirrorOf?: string;                // Phase 4 — id de l'original (alias)
  temporalAnchor?: TemporalAnchor;  // Phase 6 — date du contenu décrit
}

/** Bloc de texte avec rendu Markdown + LaTeX. */
export interface TextAnnotation extends AnnotationBase {
  type: "text";
  text: string;
  fontSize?: number;
  color?: string;
  width?: number;
  height?: number;
  cursorPos?: number;
}

/** Note collante avec fond coloré + opérateur logique optionnel. */
export interface StickyAnnotation extends AnnotationBase {
  type: "sticky";
  text: string;
  fontSize?: number;
  color?: string;
  bgColor?: string;
  width?: number;
  height?: number;
  cursorPos?: number;
  operator?: "AND" | "OR" | "BUT" | "BECAUSE"; // Phase 5 — sticky-opérateur
  sourceFile?: string; // App Bridge — chemin absolu vers le fichier source
}

/** Flèche orientée entre deux nœuds avec prédicat sémantique optionnel. */
export interface ArrowAnnotation extends AnnotationBase {
  type: "arrow";
  x2: number;
  y2: number;
  text?: string;                               // label court affiché sur la flèche
  fontSize?: number;                           // taille du label
  color?: string;
  arrowType?: "straight" | "curved";
  arrowBidirectional?: boolean;
  predicate?: ArrowPredicate;
  strokeWidth?: number;
  waypoints?: { x: number; y: number }[];     // points de passage
  sourceId?: string;                           // node attaché au début
  targetId?: string;                           // node attaché à la fin
  sourceBlockId?: string;                      // sous-bloc départ
  targetBlockId?: string;                      // sous-bloc arrivée
  sourceTextSel?: string;                      // texte exact sélectionné côté source
  targetTextSel?: string;                      // texte exact sélectionné côté cible
  longText?: string;                           // Phase 5 — description Markdown
  targetBoardId?: string;                      // Phase 5 — flèche portail
}

/** Zone colorée organisationnelle (membrane manuelle dessinée à l'outil M). */
export interface MembraneAnnotation extends AnnotationBase {
  type: "membrane";
  width: number;
  height: number;
  color?: string;
  text?: string; // légende fixe optionnelle
}

export type Annotation =
  | TextAnnotation
  | StickyAnnotation
  | ArrowAnnotation
  | MembraneAnnotation;

// ── Type guards (narrowing par filter/map sans switch) ────────────
export const isTextAnnotation     = (a: Annotation): a is TextAnnotation     => a.type === "text";
export const isStickyAnnotation   = (a: Annotation): a is StickyAnnotation   => a.type === "sticky";
export const isArrowAnnotation    = (a: Annotation): a is ArrowAnnotation    => a.type === "arrow";
export const isMembraneAnnotation = (a: Annotation): a is MembraneAnnotation => a.type === "membrane";

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

/** R-FIL-02 (Sprint 2) — Lien vers un dossier OS dont le contenu est
 *  reflété dans le `CanvasFolder`. Si défini, le folder est un *mirror*
 *  d'un dossier disque ; sinon, c'est un folder Glucose vanilla. */
/** R-FIL-03 — Modes de tri façon explorateur Windows pour un folder miroir. */
export type FolderSortMode =
  | "name-asc"      // A → Z
  | "name-desc"     // Z → A
  | "type"          // par extension, puis nom
  | "size-desc"     // du plus gros au plus petit
  | "size-asc"      // du plus petit au plus gros
  | "modified-desc" // modifié récemment d'abord
  | "modified-asc"; // plus ancien d'abord

export interface FolderMirrorSource {
  /** Chemin absolu du dossier OS scanné. */
  rootPath: string;
  /** `snapshot` = un scan unique au drop ; `live` = watcher (R-FIL-02 v2). */
  mode: "snapshot" | "live";
  /** Timestamp ms du dernier scan. */
  lastScannedAt: number;
  /** Glob optionnel (ex: "*.blend") — null = tout (sauf binaires interdits). */
  pattern?: string;
  /** True si on scanne aussi les sous-dossiers (R-FIL-02 v2 = navigables). */
  recursive: boolean;
  /** R-FIL-03 — ordre d'affichage. Défaut: dossiers d'abord puis A→Z. */
  sortBy?: FolderSortMode;
}

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
  /** R-FIL-02 — Si défini, ce folder reflète un dossier OS. */
  mirrorSource?: FolderMirrorSource;
}

/**
 * R-FIL-02 v2 — Arbre d'un folder miroir à créer (récursif). Produit par le
 * scan filesystem (folderMirror), consommé par `createFolderTree` (store).
 *   - `annotations` = fichiers de ce niveau (stickies launchers).
 *   - `children`    = sous-dossiers navigables (mêmes nœuds, récursif).
 */
export interface FolderTreeNode {
  folder: Omit<CanvasFolder, "id" | "childBoardId">;
  annotations: Annotation[];
  children: FolderTreeNode[];
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
  /** R-EMB-01 (Sprint 2) — Map content-addressed sha256 → bytes pour les
   *  assets `mode: "embed"`. Optionnelle pour compat ascendante : un projet
   *  legacy (avant Sprint 2) n'a pas ce champ ; il est créé à la 1re embed. */
  blobs?: Record<string, Uint8Array>;
  createdAt: number;
  updatedAt: number;
}

export type Tool = "select" | "pan" | "text" | "sticky" | "arrow" | "folder" | "membrane" | "zone-select";
