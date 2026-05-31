// ────────────────────────────────────────────────────────────────────────────
// R-FIL-02 v2 (Sprint 2) — Drop d'un dossier OS → arbre de CanvasFolders
// miroir, sous-dossiers NAVIGABLES, fichiers en launchers icônés.
//
// Stratégie :
//   1. invoke `scan_tree(path, max_entries, max_depth)` côté Rust → arbre
//      complet borné (DirNode récursif).
//   2. On transforme l'arbre en `FolderTreeNode` :
//        - fichier   → sticky launcher (sourceFile → icône + double-clic open)
//        - dossier   → FolderTreeNode enfant (folder box navigable)
//   3. Layout : grille carrée 220×220 px par cellule, à chaque niveau.
//   4. Tri R-FIL-03 appliqué à chaque niveau (dossiers d'abord façon Windows).
//
// Le store crée l'arbre via `createFolderTree` (récursif : 1 child board par
// dossier).
// ────────────────────────────────────────────────────────────────────────────

import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type {
  Annotation,
  BoardImage,
  FolderMirrorSource,
  FolderSortMode,
  FolderTreeNode,
} from "../types";
import { nanoid } from "../utils/nanoid";
import { useGlucoseStore } from "../store";
import { makeSourceSticky, makeTextNodeFromFile } from "./dropHandler";

export type { FolderTreeNode } from "../types";

// Médias affichables directement (liés via convertFileSrc — chemin relatif,
// PAS d'embed : un folder mirror ne doit pas gonfler le .glucose).
const IMAGE_EXTS = new Set(["png", "jpg", "jpeg", "gif", "webp", "avif", "svg", "bmp"]);
const VIDEO_EXTS = new Set(["mp4", "mov", "avi", "mkv", "webm", "m4v"]);

const MEDIA_TILE_W = 190;
const MEDIA_TILE_H = 150;
const TEXT_TILE_W = 210;
const TEXT_TILE_H = 180;

/** Nœud brut renvoyé par `scan_tree` (Rust). */
interface DirNode {
  path: string;
  name: string;
  is_dir: boolean;
  ext: string;
  size: number;
  modified: number; // epoch secondes
  /** Contenu texte inline si le fichier est lisible et sous la limite. */
  text?: string | null;
  children: DirNode[];
}

/** Image/vidéo liée (chemin disque via convertFileSrc). La boîte est carrée
 *  mais le sprite se cadre DEDANS en préservant le ratio (`fit:"contain"`) —
 *  jamais d'image déformée. */
function makeLinkedMedia(path: string, isVideo: boolean, x: number, y: number): BoardImage {
  return {
    id: nanoid(),
    src: convertFileSrc(path),
    isVideo: isVideo || undefined,
    fit: "contain",
    x, y,
    width: MEDIA_TILE_W,
    height: MEDIA_TILE_H,
    rotation: 0,
    locked: false,
    tags: [],
    sourceUrl: path,
    originalWidth: MEDIA_TILE_W,
    originalHeight: MEDIA_TILE_H,
  };
}

// Plafond d'entrées pour UN niveau (un seul dossier). Largement suffisant même
// pour un dossier à plusieurs milliers de fichiers directs ; le reste de
// l'arbo est scanné paresseusement à l'entrée de chaque sous-dossier.
const MAX_ENTRIES = 20_000;
const CELL = 220;
const PADDING = 80;
// Boîte de dossier COMPACTE (style explorateur). Son contenu vit dans le child
// board (qu'on voit en entrant), donc la boîte n'a PAS besoin d'être taillée à
// son contenu — sinon les dossiers se chevauchent en un gros tas illisible.
const FOLDER_BOX_W = 200;
const FOLDER_BOX_H = 168;

export interface ScanFolderResult {
  tree: FolderTreeNode;
  /** Nb total d'entrées (tous niveaux) — pour toast. */
  totalEntries: number;
  /** True si le scan a été tronqué (budget atteint). */
  truncated: boolean;
}

// ── Tri R-FIL-03 ────────────────────────────────────────────────────────────

function compareByMode(a: DirNode, b: DirNode, mode: FolderSortMode): number {
  switch (mode) {
    case "name-desc":
      return b.name.localeCompare(a.name, undefined, { numeric: true });
    case "type": {
      const e = a.ext.localeCompare(b.ext);
      return e !== 0 ? e : a.name.localeCompare(b.name, undefined, { numeric: true });
    }
    case "size-desc":
      return b.size - a.size || a.name.localeCompare(b.name);
    case "size-asc":
      return a.size - b.size || a.name.localeCompare(b.name);
    case "modified-desc":
      return b.modified - a.modified || a.name.localeCompare(b.name);
    case "modified-asc":
      return a.modified - b.modified || a.name.localeCompare(b.name);
    default: // "name-asc"
      return a.name.localeCompare(b.name, undefined, { numeric: true });
  }
}

/** Trie en gardant les dossiers d'abord (comportement explorateur Windows). */
function sortNodes(nodes: DirNode[], mode: FolderSortMode): DirNode[] {
  const dirs = nodes.filter((n) => n.is_dir).sort((a, b) => compareByMode(a, b, mode));
  const files = nodes.filter((n) => !n.is_dir).sort((a, b) => compareByMode(a, b, mode));
  return [...dirs, ...files];
}

// ── Construction d'UN niveau (scan paresseux) ────────────────────────────────

/** Boîte d'un sous-dossier NON encore scanné (pendingScan). Vide jusqu'à ce
 *  qu'on y entre → expandFolder le remplit. */
function makePendingFolderChild(
  dir: DirNode,
  x: number,
  y: number,
  sortBy: FolderSortMode,
): FolderTreeNode {
  return {
    folder: {
      name: dir.name,
      color: "#60a5fa",
      x, y,
      width: FOLDER_BOX_W,
      height: FOLDER_BOX_H,
      mirrorSource: {
        rootPath: dir.path,
        mode: "snapshot",
        lastScannedAt: 0,
        recursive: true,
        sortBy,
        pendingScan: true,
      },
    },
    annotations: [],
    images: [],
    children: [],
  };
}

/**
 * Construit UN SEUL niveau (les enfants directs de `dir`). Les sous-dossiers
 * deviennent des boîtes `pendingScan` vides (scannées à l'entrée). Pas de
 * récursion → import instantané quelle que soit la profondeur/taille.
 */
function buildLevelNode(
  dir: DirNode,
  folderX: number,
  folderY: number,
  sortBy: FolderSortMode,
  pendingScan: boolean,
): FolderTreeNode {
  const entries = sortNodes(dir.children, sortBy);

  const annotations: Annotation[] = [];
  const images: BoardImage[] = [];
  const children: FolderTreeNode[] = [];

  // R-FIL-02 v3 — LAYOUT PAR CATÉGORIES (en 3 bandes verticales) pour éviter le
  // « gros bordel » où vidéos/images (sprites sous le z-order) se superposent
  // aux icônes/texte (divs HTML) :
  //   1) Dossiers + lanceurs (icônes)
  //   2) Images + vidéos (médias)
  //   3) Blocs texte (.md/.txt/code)
  const isMedia = (e: DirNode) => IMAGE_EXTS.has(e.ext) || VIDEO_EXTS.has(e.ext);
  const isText = (e: DirNode) => typeof e.text === "string";
  const iconGroup = entries.filter((e) => e.is_dir || (!isMedia(e) && !isText(e)));
  const mediaGroup = entries.filter((e) => !e.is_dir && isMedia(e));
  const textGroup = entries.filter((e) => !e.is_dir && isText(e));

  const BAND_GAP = 180; // espace entre deux catégories
  let bandY = PADDING;

  /** Place un groupe en grille à partir de `bandY`, puis avance `bandY`. */
  const placeGroup = (group: DirNode[], make: (e: DirNode, x: number, y: number) => void) => {
    if (group.length === 0) return;
    const cols = Math.max(1, Math.ceil(Math.sqrt(group.length)));
    const rows = Math.ceil(group.length / cols);
    group.forEach((e, i) => {
      const x = PADDING + (i % cols) * CELL;
      const y = bandY + Math.floor(i / cols) * CELL;
      make(e, x, y);
    });
    bandY += rows * CELL + BAND_GAP;
  };

  placeGroup(iconGroup, (e, x, y) => {
    if (e.is_dir) children.push(makePendingFolderChild(e, x, y, sortBy));
    else annotations.push(makeSourceSticky(e.path, x, y));
  });
  placeGroup(mediaGroup, (e, x, y) => {
    images.push(makeLinkedMedia(e.path, VIDEO_EXTS.has(e.ext), x, y));
  });
  placeGroup(textGroup, (e, x, y) => {
    const node = makeTextNodeFromFile(e.name, e.text as string, false, x, y);
    annotations.push({
      ...node,
      sourceFile: e.path,
      width: TEXT_TILE_W,
      height: TEXT_TILE_H,
    } as Annotation);
  });

  const mirrorSource: FolderMirrorSource = {
    rootPath: dir.path,
    mode: "snapshot",
    lastScannedAt: Date.now(),
    recursive: true,
    sortBy,
    pendingScan,
  };

  return {
    folder: {
      name: dir.name,
      color: "#60a5fa",
      x: folderX,
      y: folderY,
      width: FOLDER_BOX_W,
      height: FOLDER_BOX_H,
      mirrorSource,
    },
    annotations,
    images,
    children,
  };
}

function countEntries(node: FolderTreeNode): number {
  let n = node.annotations.length + node.images.length;
  for (const c of node.children) n += 1 + countEntries(c);
  return n;
}

/**
 * Scanne UN niveau d'un dossier OS (scan paresseux). Les sous-dossiers
 * deviennent des boîtes `pendingScan` vides — on ne descend pas. Import
 * instantané et complet quelle que soit la taille (cf. `expandFolder` à
 * l'entrée).
 *
 * @param rootPath chemin OS canonique
 * @param folderX  position x du folder racine dans le board parent
 * @param folderY  position y du folder racine dans le board parent
 * @param sortBy   ordre de tri (R-FIL-03), défaut "name-asc"
 */
export async function scanFolderForMirror(
  rootPath: string,
  folderX: number,
  folderY: number,
  sortBy: FolderSortMode = "name-asc",
): Promise<ScanFolderResult> {
  // maxDepth = 1 → enfants directs seulement (paresseux).
  const root: DirNode = await invoke("scan_tree", {
    path: rootPath,
    maxEntries: MAX_ENTRIES,
    maxDepth: 1,
  });

  const tree = buildLevelNode(root, folderX, folderY, sortBy, false);
  const totalEntries = countEntries(tree);

  return {
    tree,
    totalEntries,
    truncated: totalEntries >= MAX_ENTRIES,
  };
}

/**
 * R-FIL-02 v3 — Si le folder est `pendingScan`, scanne son niveau et remplit
 * son child board (via `expandFolder`). Idempotent. Utilisé AUSSI par la
 * navigation breadcrumb (sinon : page vide sur un dossier pas encore scanné).
 */
export async function expandFolderIfPending(
  parentBoardId: string,
  folderId: string,
): Promise<void> {
  const st = useGlucoseStore.getState();
  const parent = st.project.boards.find((b) => b.id === parentBoardId);
  const folder = (parent?.folders ?? []).find((f) => f.id === folderId);
  if (!folder?.mirrorSource?.pendingScan) return;
  const result = await scanFolderForMirror(
    folder.mirrorSource.rootPath, 0, 0, folder.mirrorSource.sortBy,
  );
  useGlucoseStore.getState().expandFolder(parentBoardId, folderId, result.tree);
}
