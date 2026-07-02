import { getCurrentPath } from "./currentPath";

/**
 * Normalise les slashes et nettoie les préfixes file://
 */
export function normalizePath(p: string): string {
  let clean = p.replace(/\\/g, "/");
  if (clean.startsWith("file:///")) {
    clean = clean.slice(8);
  } else if (clean.startsWith("file://")) {
    clean = clean.slice(7);
  }
  return clean;
}

/**
 * Indique si un chemin est un chemin local absolu.
 */
export function isAbsolutePath(p: string): boolean {
  if (!p) return false;
  // Les protocoles web et asset: ne sont pas des chemins locaux absolus
  if (/^[a-z0-9+-.]+:\/\//i.test(p) && !p.startsWith("file://")) {
    return false;
  }
  const clean = normalizePath(p);
  // Lecteur Windows (ex: C:/) ou UNC (//) ou chemin UNIX (/usr)
  if (/^[a-zA-Z]:\//.test(clean)) return true;
  if (clean.startsWith("/")) return true;
  return false;
}

/**
 * Extrait le dossier parent d'un chemin de fichier.
 */
export function getParentDir(filePath: string): string {
  const normalized = normalizePath(filePath);
  const idx = normalized.lastIndexOf("/");
  if (idx === -1) return "";
  return normalized.slice(0, idx);
}

/**
 * Calcule le chemin relatif depuis `fromDir` vers `toFile`.
 */
export function computeRelativePath(fromDir: string, toFile: string): string {
  const from = normalizePath(fromDir).split("/").filter(Boolean);
  const to = normalizePath(toFile).split("/").filter(Boolean);

  // Si on est sous Windows et que les lecteurs diffèrent (ex: C: et D:), impossible d'avoir un chemin relatif
  if (from[0] && to[0] && from[0].toLowerCase() !== to[0].toLowerCase()) {
    return toFile;
  }

  let commonIdx = 0;
  while (
    commonIdx < from.length &&
    commonIdx < to.length &&
    from[commonIdx].toLowerCase() === to[commonIdx].toLowerCase()
  ) {
    commonIdx++;
  }

  const upCount = from.length - commonIdx;
  const parentDirs = Array(upCount).fill("..");
  const subDirs = to.slice(commonIdx);

  return [...parentDirs, ...subDirs].join("/");
}

/**
 * Convertit un chemin absolu en chemin relatif par rapport au fichier .glucose actif.
 */
export function toRelative(filePath: string, projectFilePath: string | null = getCurrentPath()): string {
  if (!filePath || !projectFilePath) return filePath;
  // Ne pas modifier les URLs non locales ou déjà relatives
  if (!isAbsolutePath(filePath)) return filePath;

  const projectDir = getParentDir(projectFilePath);
  if (!projectDir) return filePath;

  const isFileUri = filePath.startsWith("file://");
  const cleanPath = normalizePath(filePath);
  const rel = computeRelativePath(projectDir, cleanPath);

  return isFileUri ? `file://${rel}` : rel;
}

/**
 * Résout un chemin (potentiellement relatif) en chemin absolu par rapport au fichier .glucose actif.
 */
export function toAbsolute(filePath: string, projectFilePath: string | null = getCurrentPath()): string {
  if (!filePath || !projectFilePath) return filePath;
  // Si c'est déjà absolu ou une URL non locale, on renvoie tel quel
  if (isAbsolutePath(filePath)) return filePath;
  if (/^[a-z0-9+-.]+:\/\//i.test(filePath) && !filePath.startsWith("file://")) {
    return filePath;
  }

  const projectDir = getParentDir(projectFilePath);
  if (!projectDir) return filePath;

  const isFileUri = filePath.startsWith("file://");
  const cleanPath = normalizePath(filePath);

  const combined = `${projectDir}/${cleanPath}`;
  const parts = combined.split("/").filter(Boolean);
  const resolvedParts: string[] = [];

  // Résolution des segments .. et .
  for (const part of parts) {
    if (part === "." || part === "") continue;
    if (part === "..") {
      if (resolvedParts.length > 0 && resolvedParts[resolvedParts.length - 1] !== "..") {
        resolvedParts.pop();
      } else {
        resolvedParts.push("..");
      }
    } else {
      resolvedParts.push(part);
    }
  }

  // Reconstruction du chemin absolu
  let abs = resolvedParts.join("/");
  // Sous Windows, si le premier segment ne se termine pas par un deux-points (ex: C:), et qu'on a besoin d'un slash initial
  const isWindowsDrive = resolvedParts[0] && /^[a-zA-Z]:$/.test(resolvedParts[0]);
  if (!isWindowsDrive && combined.startsWith("/")) {
    abs = `/${abs}`;
  }

  return isFileUri ? `file://${abs}` : abs;
}
