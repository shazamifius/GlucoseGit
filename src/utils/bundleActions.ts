// ────────────────────────────────────────────────────────────────────────────
// Glue UI pour les bundles portables (dialogues + orchestration + toast).
// Garde ExportMenu / App.tsx minces ; toute la logique testée vit dans bundle.ts.
// ────────────────────────────────────────────────────────────────────────────

import { save as saveDialog, open as openDialog } from "@tauri-apps/plugin-dialog";
import * as A from "../store/automerge";
import type { Project } from "../types";
import { useGlucoseStore } from "../store";
import { showToast } from "../components/Toast";
import { exportBundle, importBundle } from "./bundle";
import { loadProject } from "./project";
import { resetAutoVersionAccumulator } from "./autoVersion";

function sanitize(name: string): string {
  return (name || "glucose").replace(/[\\/:*?"<>|]+/g, "_").trim() || "glucose";
}

/**
 * Exporte le projet courant en bundle portable (dossier auto-suffisant :
 * doc + objects/ + manifeste). Ouvre un dialogue pour choisir l'emplacement/nom.
 */
export async function exportPortableBundle(): Promise<void> {
  const { _doc, project } = useGlucoseStore.getState();
  const dest = await saveDialog({
    title: "Exporter en projet portable — nom du dossier à créer",
    defaultPath: `${sanitize(project.name)}-portable`,
  });
  if (!dest) return;

  const res = await exportBundle(_doc, dest);

  const parts = [`${res.included} image${res.included > 1 ? "s" : ""}`];
  if (res.missing.length) parts.push(`${res.missing.length} introuvable${res.missing.length > 1 ? "s" : ""}`);
  if (res.corrupt.length) parts.push(`${res.corrupt.length} corrompue${res.corrupt.length > 1 ? "s" : ""}`);
  showToast(`Bundle portable créé — ${parts.join(", ")}`, "🎒");
}

export interface OpenBundleDeps {
  /** Adopte un doc v2 (historique préservé). */
  loadDoc: (doc: A.Doc<Project>) => void;
  /** Adopte un projet plain (v1 / migration). */
  loadStore: (project: Project) => void;
  /** Mémorise le chemin du doc ouvert (pathRef + currentPath). */
  setPath: (path: string) => void;
}

/**
 * Ouvre un bundle portable : choisit un dossier, ré-hydrate ses assets dans le
 * magasin global, puis ouvre le doc via le flux `loadProject` normal.
 */
export async function openPortableBundle(deps: OpenBundleDeps): Promise<void> {
  const dir = await openDialog({
    directory: true,
    multiple: false,
    title: "Ouvrir un bundle portable — choisis le dossier « …-portable » (contient bundle.json)",
  });
  if (!dir || typeof dir !== "string") return;

  const imp = await importBundle(dir);
  const r = await loadProject(imp.docPath);
  if (!r) return;

  deps.setPath(r.path);
  if (r.doc) deps.loadDoc(r.doc);
  else deps.loadStore(r.project);
  resetAutoVersionAccumulator();

  const warn = imp.missing.length ? ` (${imp.missing.length} image${imp.missing.length > 1 ? "s" : ""} manquante${imp.missing.length > 1 ? "s" : ""})` : "";
  showToast(`Bundle ouvert — ${imp.rehydrated} image${imp.rehydrated > 1 ? "s" : ""} restaurée${imp.rehydrated > 1 ? "s" : ""}${warn}`, "🎒");
}
