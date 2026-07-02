// ────────────────────────────────────────────────────────────────────────────
// Git #1 (north star « indestructible ») — BUNDLE PORTABLE
// ────────────────────────────────────────────────────────────────────────────
//
// Problème : les images d'un projet ne vivent PAS dans le `.glucose`. Le doc ne
// porte qu'une référence logique `asset:<hash>.<ext>` (mode "link") ; les octets
// sont dans un magasin GLOBAL `app_data_dir/assets/`, partagé entre tous les
// projets. Déplacer un `.glucose` (clé USB, autre machine) laisse le magasin
// derrière → images mortes.
//
// Solution (approche 1, additive, chemin chaud INTACT) : un « bundle portable »
// = un dossier auto-suffisant, façon `objects/` de git :
//
//     MonProjet-portable/
//       project.glucose        ← le doc (A.save du projet courant)
//       objects/<hash>.<ext>   ← les octets des assets référencés, content-addressed
//       bundle.json            ← manifeste (liste + sha256 + tailles)
//
// « Exporter » copie exactement les assets que le doc référence. « Ouvrir »
// ré-hydrate ces octets dans le magasin global (dédup par hash) puis ouvre le doc
// → toutes les images résolvent, sur n'importe quelle machine.
//
// NOTE : les assets `mode:"embed"` (octets dans `project.blobs`) voyagent DÉJÀ
// dans le `.glucose` — le bundle ne s'occupe QUE des refs `asset:` externes.
// Cf. [[image-storage-git-bundle]] (cible) et [[collab-images-embed-vs-link]]
// (jamais ré-embarquer dans le doc). Tout se fait via les primitives existantes
// (plugin-fs + load_asset/save_asset) → aucune nouvelle commande Rust, aucun rebuild.

import { writeFile, readFile, mkdir } from "@tauri-apps/plugin-fs";
import { invoke } from "@tauri-apps/api/core";
import * as A from "../store/automerge";
import type { Project } from "../types";
import { sha256Hex, dataUrlToBytes } from "./assetRef";
import { saveAssetFromBytes } from "./assets";

/** Échec de bundle (format invalide, intégrité, I/O). */
export class BundleError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "BundleError";
  }
}

/** Un asset référencé par le doc, tel qu'il vit dans le magasin content-addressed. */
export interface ReferencedAsset {
  /** Nom de fichier dans le magasin : `<hash16>.<ext>` (= l'identifiant `asset:` sans le préfixe). */
  name: string;
  /** SHA-256 complet (64 hex) si le doc le connaît (AssetRef.sha256). */
  sha256?: string;
  /** Taille en octets déclarée par le doc (indicatif). */
  sizeBytes?: number;
}

// ── PUR ─────────────────────────────────────────────────────────────────────

/**
 * Énumère TOUS les assets `asset:<name>` référencés par un projet (images, mode
 * "link" ou legacy `src`). PUR : aucune I/O.
 *
 * - Dédupliqué par nom (une image posée 10× = 1 objet).
 * - Trié (ordre stable) → manifeste reproductible, tests déterministes.
 * - Ignore les `mode:"embed"` (leurs octets sont dans le doc, déjà portables) et
 *   les liens non-`asset:` (http, chemins de fichiers du folder-mirror).
 */
export function collectReferencedAssets(project: Project): ReferencedAsset[] {
  const byName = new Map<string, ReferencedAsset>();

  const consider = (name: string, sha256?: string, sizeBytes?: number) => {
    if (byName.has(name)) return;
    byName.set(name, { name, ...(sha256 ? { sha256 } : {}), ...(sizeBytes !== undefined ? { sizeBytes } : {}) });
  };

  for (const board of project.boards ?? []) {
    for (const img of board.images ?? []) {
      const a = img.asset;
      if (a && a.mode === "link" && a.href?.startsWith("asset:")) {
        consider(a.href.slice("asset:".length), a.sha256, a.sizeBytes);
      } else if (img.src?.startsWith("asset:")) {
        consider(img.src.slice("asset:".length));
      }
    }
  }

  return [...byName.values()].sort((x, y) => x.name.localeCompare(y.name));
}

/** Version du format de bundle (incrémentée si la structure change). */
export const BUNDLE_FORMAT = "glucose-bundle" as const;
export const BUNDLE_VERSION = 1 as const;
export const BUNDLE_DOC_NAME = "project.glucose" as const;
export const BUNDLE_MANIFEST_NAME = "bundle.json" as const;
export const BUNDLE_OBJECTS_DIR = "objects" as const;

export interface BundleManifest {
  format: typeof BUNDLE_FORMAT;
  version: typeof BUNDLE_VERSION;
  /** Nom du projet (décoratif). */
  name: string;
  /** Nom du fichier doc dans le bundle. */
  doc: typeof BUNDLE_DOC_NAME;
  createdAt: number;
  assets: ReferencedAsset[];
}

/** Construit le manifeste. PUR. */
export function buildBundleManifest(project: Project, assets: ReferencedAsset[]): BundleManifest {
  return {
    format: BUNDLE_FORMAT,
    version: BUNDLE_VERSION,
    name: project.name || "glucose",
    doc: BUNDLE_DOC_NAME,
    createdAt: Date.now(),
    assets,
  };
}

/**
 * Vérifie que des octets correspondent à un nom d'asset content-addressed.
 * Le nom du magasin ENCODE le hash (`<hash16>.<ext>`, cf. Rust `save_asset`) →
 * l'intégrité est vérifiable même sans sha256 complet dans le doc.
 *
 * @returns true si `sha256(bytes)` commence par le stem du nom ET (si fourni)
 *          égale le sha256 complet attendu.
 */
export async function assetBytesMatch(
  name: string,
  bytes: Uint8Array,
  expectedSha?: string,
): Promise<boolean> {
  const stem = (name.split(".")[0] || "").toLowerCase();
  if (!stem) return false;
  const full = await sha256Hex(bytes);
  if (!full.startsWith(stem)) return false;
  if (expectedSha && full !== expectedSha.toLowerCase()) return false;
  return true;
}

/** Joint des segments de chemin en respectant le séparateur natif du chemin de base. */
function joinPath(base: string, ...parts: string[]): string {
  const sep = base.includes("\\") ? "\\" : "/";
  const trimmed = base.replace(/[\\/]+$/, "");
  return [trimmed, ...parts].join(sep);
}

// ── ORCHESTRATION (I/O Tauri) ────────────────────────────────────────────────

export interface BundleExportResult {
  dir: string;
  /** Nombre d'assets effectivement inclus dans objects/. */
  included: number;
  /** Assets introuvables dans le magasin global (bytes perdus). */
  missing: string[];
  /** Assets présents mais dont le hash ne correspond pas (corruption disque). */
  corrupt: string[];
  /** Octets d'assets écrits (hors doc). */
  bytesWritten: number;
}

/**
 * Écrit un bundle portable auto-suffisant dans `destDir` à partir de l'état
 * courant `doc`. Le projet source n'est JAMAIS touché (on ne fait que créer un
 * nouveau dossier). Les assets introuvables/corrompus sont SIGNALÉS (pas
 * d'exception) et l'export se termine quand même → honnête sur ce qui a pu être
 * embarqué.
 */
export async function exportBundle(doc: A.Doc<Project>, destDir: string): Promise<BundleExportResult> {
  const project = A.asPlain<Project>(doc);
  const assets = collectReferencedAssets(project);

  const objectsDir = joinPath(destDir, BUNDLE_OBJECTS_DIR);
  await mkdir(destDir, { recursive: true });
  await mkdir(objectsDir, { recursive: true });

  const missing: string[] = [];
  const corrupt: string[] = [];
  let included = 0;
  let bytesWritten = 0;

  for (const a of assets) {
    let bytes: Uint8Array;
    try {
      const dataUrl = await invoke<string>("load_asset", { filename: a.name });
      bytes = dataUrlToBytes(dataUrl).bytes;
    } catch {
      missing.push(a.name); // octets absents du magasin global
      continue;
    }
    if (!(await assetBytesMatch(a.name, bytes, a.sha256))) {
      corrupt.push(a.name); // présents mais hash faux → on ne fabrique pas un bundle corrompu
      continue;
    }
    await writeFile(joinPath(objectsDir, a.name), bytes);
    included++;
    bytesWritten += bytes.length;
  }

  // Le doc en dernier : A.save(doc) = état courant exact (édits non sauvegardés compris).
  await writeFile(joinPath(destDir, BUNDLE_DOC_NAME), A.save(doc));

  const manifest = buildBundleManifest(project, assets);
  const manifestBytes = new TextEncoder().encode(JSON.stringify(manifest, null, 2));
  await writeFile(joinPath(destDir, BUNDLE_MANIFEST_NAME), manifestBytes);

  return { dir: destDir, included, missing, corrupt, bytesWritten };
}

export interface BundleImportResult {
  /** Chemin du doc à ouvrir ensuite (flux loadProject normal). */
  docPath: string;
  /** Nombre d'assets ré-hydratés dans le magasin global. */
  rehydrated: number;
  /** Assets du manifeste absents/corrompus dans le bundle (images qui manqueront). */
  missing: string[];
}

/**
 * Ré-hydrate un bundle : recopie ses `objects/` dans le magasin global (dédup par
 * hash côté Rust) après vérification d'intégrité, puis renvoie le chemin du doc à
 * ouvrir. N'ouvre PAS le projet lui-même (le caller enchaîne sur `loadProject`).
 */
export async function importBundle(bundleDir: string): Promise<BundleImportResult> {
  // Lecture du manifeste — on distingue « fichier absent » (mauvais dossier
  // choisi) de « JSON invalide ». `String(e)` car Tauri jette souvent une string
  // (pas un Error) → `.message` donnerait « undefined » et cacherait la cause.
  const manifestPath = joinPath(bundleDir, BUNDLE_MANIFEST_NAME);
  let raw: Uint8Array;
  try {
    raw = await readFile(manifestPath);
  } catch (e) {
    throw new BundleError(
      `${BUNDLE_MANIFEST_NAME} introuvable dans « ${bundleDir} ». Sélectionne le DOSSIER du bundle `
      + `(celui qui contient ${BUNDLE_MANIFEST_NAME}, ${BUNDLE_DOC_NAME} et ${BUNDLE_OBJECTS_DIR}/). [${String(e)}]`,
    );
  }
  let manifest: BundleManifest;
  try {
    manifest = JSON.parse(new TextDecoder().decode(raw)) as BundleManifest;
  } catch (e) {
    throw new BundleError(`${BUNDLE_MANIFEST_NAME} illisible (JSON invalide) : ${String(e)}`);
  }
  if (manifest?.format !== BUNDLE_FORMAT) {
    throw new BundleError("ce dossier n'est pas un bundle Glucose (format inattendu)");
  }

  const objectsDir = joinPath(bundleDir, BUNDLE_OBJECTS_DIR);
  const missing: string[] = [];
  let rehydrated = 0;

  for (const a of manifest.assets ?? []) {
    let bytes: Uint8Array;
    try {
      bytes = await readFile(joinPath(objectsDir, a.name));
    } catch {
      missing.push(a.name); // objet absent du bundle
      continue;
    }
    if (!(await assetBytesMatch(a.name, bytes, a.sha256))) {
      missing.push(a.name); // corrompu → on ne l'installe pas
      continue;
    }
    const ext = a.name.split(".").pop() || "png";
    await saveAssetFromBytes(bytes, ext); // → magasin global, dédup automatique
    rehydrated++;
  }

  return { docPath: joinPath(bundleDir, BUNDLE_DOC_NAME), rehydrated, missing };
}
