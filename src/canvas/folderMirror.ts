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

/** Image/vidéo liée (chemin disque via convertFileSrc), tuile de taille fixe. */
function makeLinkedMedia(path: string, isVideo: boolean, x: number, y: number): BoardImage {
  return {
    id: nanoid(),
    src: convertFileSrc(path),
    isVideo: isVideo || undefined,
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

const MAX_ENTRIES = 5_000;
const MAX_DEPTH = 8;
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

// ── Construction de l'arbre ─────────────────────────────────────────────────

function buildFolderNode(
  dir: DirNode,
  folderX: number,
  folderY: number,
  sortBy: FolderSortMode,
): FolderTreeNode {
  const entries = sortNodes(dir.children, sortBy);
  const N = entries.length;
  const cols = Math.max(1, Math.ceil(Math.sqrt(N)));
  const rows = Math.max(1, Math.ceil(N / cols));
  void rows; // (cols/rows servent au placement ci-dessous)

  const annotations: Annotation[] = [];
  const images: BoardImage[] = [];
  const children: FolderTreeNode[] = [];

  entries.forEach((e, i) => {
    const col = i % cols;
    const row = Math.floor(i / cols);
    const x = PADDING + col * CELL;
    const y = PADDING + row * CELL;

    // Sous-dossier → folder navigable (positionné DANS le board courant).
    if (e.is_dir) {
      children.push(buildFolderNode(e, x, y, sortBy));
      return;
    }
    // Image → vignette liée (affiche la vraie image, chemin relatif).
    if (IMAGE_EXTS.has(e.ext)) {
      images.push(makeLinkedMedia(e.path, false, x, y));
      return;
    }
    // Vidéo → lecteur lié.
    if (VIDEO_EXTS.has(e.ext)) {
      images.push(makeLinkedMedia(e.path, true, x, y));
      return;
    }
    // Texte/code lu au scan → bloc texte (markdown/LaTeX) au format tuile.
    if (typeof e.text === "string") {
      const node = makeTextNodeFromFile(e.name, e.text, false, x, y);
      annotations.push({ ...node, width: TEXT_TILE_W, height: TEXT_TILE_H } as Annotation);
      return;
    }
    // Sinon (binaire, ou texte trop gros) → launcher icôné (double-clic = ouvrir).
    annotations.push(makeSourceSticky(e.path, x, y));
  });

  const mirrorSource: FolderMirrorSource = {
    rootPath: dir.path,
    mode: "snapshot",
    lastScannedAt: Date.now(),
    recursive: true,
    sortBy,
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
 * Scanne un dossier OS récursivement et prépare l'arbre de folders miroir.
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
  const root: DirNode = await invoke("scan_tree", {
    path: rootPath,
    maxEntries: MAX_ENTRIES,
    maxDepth: MAX_DEPTH,
  });

  const tree = buildFolderNode(root, folderX, folderY, sortBy);
  const totalEntries = countEntries(tree);

  return {
    tree,
    totalEntries,
    truncated: totalEntries >= MAX_ENTRIES,
  };
}
