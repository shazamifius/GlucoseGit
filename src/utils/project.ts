import { save as saveDialog, open as openDialog } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { homeDir } from "@tauri-apps/api/path";
import { Project } from "../types";
import { parseProjectFile } from "../store/projectSchema";
// Phase 7.0 — migration des images base64 legacy vers asset:<hash>.<ext>
import { migrateLegacyAssets } from "./assets";
// Phase 7.2 — format binaire `.glucose` v2 via Automerge
import * as A from "../store/automerge";

// ────────────────────────────────────────────────────────────────────────────
// base64 ↔ Uint8Array (transport entre Rust et JS pour le binaire Automerge)
// ────────────────────────────────────────────────────────────────────────────

function bytesToBase64(bytes: Uint8Array): string {
  // Évite "Maximum call stack size" sur les gros buffers (chunks de 32 KB).
  const CHUNK = 32_768;
  let binary = "";
  for (let i = 0; i < bytes.length; i += CHUNK) {
    binary += String.fromCharCode.apply(null, Array.from(bytes.subarray(i, i + CHUNK)));
  }
  return btoa(binary);
}

function base64ToBytes(b64: string): Uint8Array {
  const binary = atob(b64);
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i);
  return out;
}

// ────────────────────────────────────────────────────────────────────────────
// Save — toujours en v2 binaire (Automerge)
// ────────────────────────────────────────────────────────────────────────────
//
// `saveProject` accepte indifféremment un Project plain (cas legacy / tests)
// OU un Doc<Project> Automerge (cas standard depuis le store CRDT). Si on
// reçoit un Doc on save TEL QUEL — l'historique CRDT est préservé. Si on reçoit
// un Project, on crée un doc neuf (l'historique commence ici).

export async function saveProject(
  projectOrDoc: Project | A.Doc<Project>,
  existingPath?: string,
): Promise<string | null> {
  let path = existingPath;
  // Détecte si c'est déjà un doc Automerge ou un project plain
  const isDoc = (val: unknown): val is A.Doc<Project> => {
    try { A.getHeads(val as A.Doc<Project>); return true; } catch { return false; }
  };

  // Pour le nom par défaut, on lit `name` (fonctionne pour les deux types via Proxy)
  const projectName = (projectOrDoc as Project).name || "projet";

  if (!path) {
    let defaultPath: string;
    try {
      const home = await homeDir();
      defaultPath = `${home}/${projectName}.glucose`;
    } catch (_) {
      defaultPath = `${projectName}.glucose`;
    }
    const result = await saveDialog({
      defaultPath,
      filters: [{ name: "Projet Glucose Git", extensions: ["glucose"] }],
    });
    path = result ?? undefined;
  }
  if (!path) return null;

  if (!path.endsWith(".glucose")) path += ".glucose";

  let bytes: Uint8Array;
  if (isDoc(projectOrDoc)) {
    // Cas standard : le doc est la source de vérité, on save tel quel.
    // L'historique Automerge complet est préservé dans le binaire.
    bytes = A.save(projectOrDoc);
  } else {
    // Cas legacy : project plain → on crée un doc neuf.
    const plain = projectOrDoc as Project;
    const stamped: Project = {
      ...plain,
      version: "2.0.0",
      updatedAt: Date.now(),
    };
    const doc = A.create<Project>(stamped);
    bytes = A.save(doc);
  }
  const b64 = bytesToBase64(bytes);
  await invoke("write_glucose_binary", { path, base64Data: b64 });
  return path;
}

// ────────────────────────────────────────────────────────────────────────────
// Load — détecte automatiquement v1 (JSON) ou v2 (binaire Automerge)
// ────────────────────────────────────────────────────────────────────────────
//
// Renvoie soit un `doc` (v2 → préserve l'historique Automerge complet à l'ouverture),
// soit un `project` plain (v1 → l'historique commencera au prochain change). Le
// store appelle `loadDoc(doc)` ou `loadProject(project)` selon.

export interface LoadProjectResult {
  /** Présent si v2 binaire — la source de vérité Automerge originale. */
  doc?: A.Doc<Project>;
  /** Toujours présent — vue plain pour les checks et migrations legacy. */
  project: Project;
  path: string;
}

export async function loadProject(): Promise<LoadProjectResult | null> {
  let defaultPath: string | undefined;
  try {
    defaultPath = await homeDir();
  } catch (_) {}

  const result = await openDialog({
    defaultPath,
    filters: [{ name: "Projet Glucose Git", extensions: ["glucose", "atelier"] }],
    multiple: false,
  });
  const path = result as string | null;
  if (!path) return null;

  let project: Project | null = null;
  let doc: A.Doc<Project> | undefined;

  // 1) Tentative v2 binaire
  try {
    const b64 = await invoke<string>("read_glucose_binary", { path });
    const bytes = base64ToBytes(b64);
    const loaded = A.load<Project>(bytes);
    const plain = A.asPlain(loaded);
    if (plain && Array.isArray((plain as Project).boards)) {
      project = plain as Project;
      doc = loaded;
      console.info("[loadProject] format v2 (binaire Automerge) détecté");
    }
  } catch (binErr) {
    console.debug("[loadProject] v2 binaire KO, tentative v1 JSON :", binErr);
  }

  // 2) Tentative v1 JSON (legacy)
  if (!project) {
    try {
      const data = await invoke<string>("read_project_file", { path });
      const raw = JSON.parse(data);
      const parsed = parseProjectFile(raw);
      if (!parsed.ok) {
        throw new Error(`Le fichier .glucose est invalide ou corrompu (${parsed.error}).`);
      }
      project = parsed.project;
      console.info("[loadProject] format v1 (JSON legacy) détecté — sera migré au prochain save");
    } catch (jsonErr) {
      throw new Error(
        `Impossible de lire le fichier .glucose. Ni JSON valide ni binaire Automerge.\n${(jsonErr as Error).message}`
      );
    }
  }

  // Phase 7.0 — Migration des assets base64 legacy vers asset:<hash>.<ext>.
  const migration = await migrateLegacyAssets(project);
  if (migration.migrated > 0) {
    console.info(`[loadProject] ${migration.migrated} image(s) legacy migrée(s) vers asset:`);
    // Si on avait un doc v2 mais qu'on a migré des assets, le doc est désynchronisé
    // → on force la création d'un nouveau doc à partir du project migré
    doc = undefined;
  }
  if (migration.failed > 0) {
    console.warn(`[loadProject] ${migration.failed} image(s) legacy n'ont pas pu être migrées`);
  }
  return { project: migration.project, doc, path };
}
