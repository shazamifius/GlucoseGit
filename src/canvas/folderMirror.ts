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

import { invoke } from "@tauri-apps/api/core";
import type {
  Annotation,
  FolderMirrorSource,
  FolderSortMode,
  FolderTreeNode,
} from "../types";
import { makeSourceSticky } from "./dropHandler";

export type { FolderTreeNode } from "../types";

/** Nœud brut renvoyé par `scan_tree` (Rust). */
interface DirNode {
  path: string;
  name: string;
  is_dir: boolean;
  ext: string;
  size: number;
  modified: number; // epoch secondes
  children: DirNode[];
}

const MAX_ENTRIES = 5_000;
const MAX_DEPTH = 8;
const CELL = 220;
const PADDING = 80;

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
  const folderW = Math.max(400, cols * CELL + PADDING * 2);
  const folderH = Math.max(300, rows * CELL + PADDING * 2);

  const annotations: Annotation[] = [];
  const children: FolderTreeNode[] = [];

  entries.forEach((e, i) => {
    const col = i % cols;
    const row = Math.floor(i / cols);
    const x = PADDING + col * CELL;
    const y = PADDING + row * CELL;

    if (e.is_dir) {
      // Sous-dossier → folder navigable (positionné DANS le board courant).
      children.push(buildFolderNode(e, x, y, sortBy));
    } else {
      // Fichier → launcher icôné (double-clic → open_in_app).
      annotations.push(makeSourceSticky(e.path, x, y));
    }
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
      width: folderW,
      height: folderH,
      mirrorSource,
    },
    annotations,
    children,
  };
}

function countEntries(node: FolderTreeNode): number {
  let n = node.annotations.length;
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
