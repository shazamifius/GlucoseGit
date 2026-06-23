// ────────────────────────────────────────────────────────────────────────────
// Git #1 — Jalons DURABLES (versions incorruptibles)
// ────────────────────────────────────────────────────────────────────────────
//
// Un jalon de la Time Machine est, en interne, un simple repère (heads) DANS le
// doc Automerge : pratique pour naviguer, mais il MEURT si le doc se corrompt.
// Pour le north star « incorruptible », chaque jalon est aussi écrit comme un
// `.glucose` COMPLET et INDÉPENDANT, à côté du fichier principal, dans
// `<chemin>.versions/`. Chacun se recharge seul (`A.loadResilient`) → c'est la
// save « qu'on retrouve à coup sûr » et le filet de secours si le doc vivant
// casse (cf. memory indestructible-incorruptible-north-star).
//
// Format auto-descriptif par NOM DE FICHIER (zéro index central = zéro surface
// de corruption supplémentaire) : `<time>__<kind>__<slug>.glucose`.
//   • time = Date.now() à la création (tri chronologique).
//   • kind = "manuel" (l'user a marqué) | "auto" (grosse modif détectée, Phase 3).
//   • slug = label lisible aplati (sûr pour un nom de fichier).

import { writeFile, rename, readFile, readDir, mkdir, remove } from "@tauri-apps/plugin-fs";
import * as A from "../store/automerge";
import type { Project } from "../types";

export type VersionKind = "manuel" | "auto";

export interface VersionMeta {
  /** Chemin absolu du fichier de version. */
  path: string;
  /** Nom de fichier brut. */
  file: string;
  /** Horodatage de création (ms unix), décodé du nom. */
  time: number;
  kind: VersionKind;
  /** Label lisible (slug décodé — peut différer légèrement de l'original saisi). */
  label: string;
}

const VERSIONS_SUFFIX = ".versions";

/** Dossier des versions pour un `.glucose` donné : `<chemin>.versions/`. */
export function versionsDirFor(mainPath: string): string {
  return `${mainPath}${VERSIONS_SUFFIX}`;
}

// ── Encodage / décodage du nom de fichier (PUR — testable hors Tauri) ─────────

/** Aplatit un label en un fragment sûr pour un nom de fichier (et réversible en
 *  affichage « assez proche »). Espaces → `-`, on retire ce qui casse un nom de
 *  fichier, on borne la longueur. Jamais vide. */
export function slugifyLabel(label: string): string {
  const s = label
    .trim()
    .replace(/[\s]+/g, "-")
    .replace(/[\\/:*?"<>|]+/g, "") // caractères interdits dans un nom de fichier
    .replace(/_{2,}/g, "_") // `__` est notre séparateur → on l'évite dans le slug
    .replace(/-{2,}/g, "-")
    .slice(0, 60)
    .replace(/^[-_]+|[-_]+$/g, "");
  return s.length > 0 ? s : "jalon";
}

/** Réaffiche un slug en label lisible (tirets → espaces). */
function unslug(slug: string): string {
  return slug.replace(/-/g, " ").trim() || "Jalon";
}

/** Construit le nom de fichier d'une version. */
export function formatVersionFile(time: number, kind: VersionKind, label: string): string {
  return `${time}__${kind}__${slugifyLabel(label)}.glucose`;
}

/** Parse un nom de fichier de version. Renvoie null si ce n'en est pas un. */
export function parseVersionFile(file: string): Omit<VersionMeta, "path"> | null {
  if (!file.endsWith(".glucose")) return null;
  const base = file.slice(0, -".glucose".length);
  const parts = base.split("__");
  if (parts.length < 3) return null;
  const time = Number(parts[0]);
  if (!Number.isFinite(time)) return null;
  const kind: VersionKind = parts[1] === "auto" ? "auto" : "manuel";
  const slug = parts.slice(2).join("__");
  return { file, time, kind, label: unslug(slug) };
}

// ── I/O Tauri ─────────────────────────────────────────────────────────────────

/**
 * Écrit une version durable (save complet, atomique tmp+rename) dans le dossier
 * `versions/`. Crée le dossier au besoin. Renvoie la méta du fichier écrit.
 */
export async function saveVersion(
  mainPath: string,
  doc: A.Doc<Project>,
  label: string,
  kind: VersionKind = "manuel",
): Promise<VersionMeta> {
  const dir = versionsDirFor(mainPath);
  await mkdir(dir, { recursive: true }); // idempotent
  const time = Date.now();
  const file = formatVersionFile(time, kind, label);
  const path = `${dir}/${file}`;
  const bytes = A.save(doc);
  // Écriture ATOMIQUE : tmp + rename (un crash ne laisse jamais une version
  // à moitié écrite, donc jamais une « fausse » save corrompue).
  const tmp = `${path}.tmp`;
  await writeFile(tmp, bytes);
  await rename(tmp, path);
  return { path, file, time, kind, label: parseVersionFile(file)?.label ?? label };
}

/** Liste les versions durables d'un projet, plus récentes en premier. */
export async function listVersions(mainPath: string): Promise<VersionMeta[]> {
  const dir = versionsDirFor(mainPath);
  let entries: { name: string; isFile?: boolean; isDirectory?: boolean }[];
  try {
    entries = (await readDir(dir)) as typeof entries;
  } catch {
    return []; // dossier absent = aucune version encore
  }
  const out: VersionMeta[] = [];
  for (const e of entries) {
    if (e.isDirectory) continue;
    if (e.name.endsWith(".tmp")) continue; // écriture en cours
    const parsed = parseVersionFile(e.name);
    if (!parsed) continue;
    out.push({ ...parsed, path: `${dir}/${e.name}` });
  }
  out.sort((a, b) => b.time - a.time);
  return out;
}

/** Recharge le doc Automerge d'une version durable (tolérant à une fin tronquée). */
export async function loadVersionDoc(meta: VersionMeta): Promise<A.Doc<Project>> {
  const bytes = await readFile(meta.path);
  return A.loadResilient<Project>(bytes).doc;
}

/**
 * Élague les versions AUTO pour n'en garder que `keep` (les plus récentes). Ne
 * touche JAMAIS aux versions manuelles (l'user les a posées exprès). Phase 3
 * (jalons auto) s'appuiera dessus pour que l'historique auto ne gonfle pas.
 */
export async function pruneAutoVersions(mainPath: string, keep: number): Promise<number> {
  const autos = (await listVersions(mainPath)).filter((v) => v.kind === "auto");
  const toRemove = autos.slice(keep); // listVersions trie déjà récent→ancien
  let removed = 0;
  for (const v of toRemove) {
    try {
      await remove(v.path);
      removed++;
    } catch (e) {
      console.warn("[versions] échec suppression", v.file, e);
    }
  }
  return removed;
}
