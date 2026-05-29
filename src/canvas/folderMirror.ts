// ────────────────────────────────────────────────────────────────────────────
// R-FIL-02 (Sprint 2) — Drop d'un dossier OS → CanvasFolder miroir avec
// FileNodes en grille.
//
// Stratégie :
//   1. invoke `scan_directory(path, max_files)` côté Rust qui renvoie les
//      entrées filtrées (pas d'.exe/.bat/etc., pas de cachés sauf .env etc.).
//   2. Pour chaque entrée file, on crée :
//        - une TextAnnotation si extension texte/code (réutilise R-FIL-01)
//        - un sticky launcher (sourceFile) sinon
//   3. Les sous-dossiers sont rendus comme stickies "📁 name" (futur :
//      sous-folders Glucose imbriqués).
//   4. Layout : grille carrée 220×220 px par cellule.
//
// Pas encore implémenté (futur R-FIL-02 v2) :
//   - Mode `live` (watcher fs)
//   - Sub-folders comme CanvasFolders imbriqués
//   - Pattern glob (filter UI)
// ────────────────────────────────────────────────────────────────────────────

import { invoke } from "@tauri-apps/api/core";
import type { Annotation, CanvasFolder, FolderMirrorSource } from "../types";
import { nanoid } from "../utils/nanoid";
import { makeSourceSticky, makeTextNodeFromFile } from "./dropHandler";

interface DirEntryDto {
  path: string;
  name: string;
  size: number;
  is_dir: boolean;
  ext: string;
}

const MAX_FILES = 10_000;
const CELL = 220;
const TEXT_FILE_EXTS = new Set([
  "txt", "md", "markdown", "json", "jsonl", "csv", "tsv", "log",
  "yaml", "yml", "toml", "ini", "env", "xml", "html", "htm",
  "conf", "cfg", "gitignore", "gitattributes",
]);
const CODE_FILE_EXTS = new Set([
  "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "rs", "go",
  "c", "cc", "cpp", "h", "hpp", "java", "rb", "php", "swift", "kt",
  "cs", "scala", "sh", "bash", "zsh", "fish", "ps1", "sql", "lua",
  "r", "jl", "hs", "erl", "ex", "exs", "clj", "fs", "dart", "nim",
  "zig", "asm", "s", "vim", "tex", "bib",
]);

const TEXT_INLINE_MAX_BYTES = 100_000;

export interface ScanFolderResult {
  /** Métadonnées du folder mirror à créer. */
  folder: Omit<CanvasFolder, "id" | "childBoardId">;
  /** Annotations à placer dans le child board. */
  annotations: Annotation[];
  /** Nb d'entrées scannées (utile pour toast). */
  totalEntries: number;
  /** Nb d'entrées tronquées si dépassement max. */
  truncated: boolean;
}

/**
 * Scanne un dossier OS et prépare les données pour créer un
 * `CanvasFolder` miroir + son contenu.
 *
 * @param rootPath chemin OS canonique
 * @param folderX position x du folder dans le board parent
 * @param folderY position y dans le board parent
 */
export async function scanFolderForMirror(
  rootPath: string,
  folderX: number,
  folderY: number,
): Promise<ScanFolderResult> {
  const entries: DirEntryDto[] = await invoke("scan_directory", {
    path: rootPath,
    maxFiles: MAX_FILES,
  });

  // Layout en grille carrée : cols = ceil(sqrt(N))
  const N = entries.length;
  const cols = Math.max(1, Math.ceil(Math.sqrt(N)));
  const rows = Math.max(1, Math.ceil(N / cols));

  // Le folder est dimensionné pour contenir la grille avec un padding
  const padding = 80;
  const folderW = Math.max(400, cols * CELL + padding * 2);
  const folderH = Math.max(300, rows * CELL + padding * 2);

  const annotations: Annotation[] = [];

  // Lecture parallèle bornée des fichiers texte (lecture côté Tauri).
  // Pour rester simple : pour cette première itération, on ne charge PAS le
  // contenu des fichiers texte ici. Ils sont représentés comme launcher.
  // Une itération future ajoutera un `read_text_file_at` qui lit en bytes.
  for (let i = 0; i < entries.length; i++) {
    const e = entries[i];
    const col = i % cols;
    const row = Math.floor(i / cols);
    // Coordonnées RELATIVES au child board (le folder lui-même est placé
    // dans le parent ; ses items vivent dans le child board à partir de
    // (0, 0)).
    const x = padding + col * CELL;
    const y = padding + row * CELL;

    if (e.is_dir) {
      // Sous-dossier : sticky placeholder (R-FIL-02 v2 → folder imbriqué)
      annotations.push({
        id: nanoid(),
        type: "sticky",
        x, y,
        text: `📁 ${e.name}`,
        sourceFile: e.path,
        bgColor: "#3a3a4a",
        color: "#dddddd",
        width: 200, height: 80,
        fontSize: 11,
      });
      continue;
    }

    // Fichier texte / code : on garde en LAUNCHER pour cette v1 (lecture
    // de chaque fichier dépasse le scope du scan). L'utilisateur peut
    // glisser-déposer le fichier individuellement pour le voir inline.
    if (TEXT_FILE_EXTS.has(e.ext) || CODE_FILE_EXTS.has(e.ext)) {
      // Marqueur visuel : couleur claire pour les fichiers lisibles
      annotations.push({
        id: nanoid(),
        type: "sticky",
        x, y,
        text: e.name,
        sourceFile: e.path,
        bgColor: "#2a3a4a",
        color: "#cccccc",
        width: 200, height: 80,
        fontSize: 11,
      });
      continue;
    }

    // Autre fichier : launcher classique
    annotations.push(makeSourceSticky(e.path, x, y));
  }
  // Avoid unused warning for the future-use helper
  void makeTextNodeFromFile;
  void TEXT_INLINE_MAX_BYTES;

  const rootName = rootPath.split(/[\\/]/).pop() || rootPath;
  const mirrorSource: FolderMirrorSource = {
    rootPath,
    mode: "snapshot",
    lastScannedAt: Date.now(),
    recursive: false,
  };

  return {
    folder: {
      name: rootName,
      color: "#60a5fa",
      x: folderX,
      y: folderY,
      width: folderW,
      height: folderH,
      mirrorSource,
    },
    annotations,
    totalEntries: entries.length,
    truncated: entries.length >= MAX_FILES,
  };
}
