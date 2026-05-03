import { save as saveDialog, open as openDialog } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { homeDir } from "@tauri-apps/api/path";
import { Project } from "../types";
import { parseProjectFile } from "../store/projectSchema";

export async function saveProject(project: Project, existingPath?: string): Promise<string | null> {
  let path = existingPath;
  if (!path) {
    let defaultPath: string;
    try {
      const home = await homeDir();
      defaultPath = `${home}/${project.name}.glucose`;
    } catch (_) {
      defaultPath = `${project.name}.glucose`;
    }
    const result = await saveDialog({
      defaultPath,
      filters: [{ name: "Projet Glucose Git", extensions: ["glucose"] }],
    });
    path = result ?? undefined;
  }
  if (!path) return null;

  // Garantit l'extension même si le dialog ne l'ajoute pas automatiquement
  if (!path.endsWith(".glucose")) path += ".glucose";

  const data = JSON.stringify({ ...project, updatedAt: Date.now() }, null, 2);
  await invoke("write_project_file", { path, contents: data });
  return path;
}

export async function loadProject(): Promise<{ project: Project; path: string } | null> {
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

  const data = await invoke<string>("read_project_file", { path });
  // CLEANUP R-01 : parsing JSON safe + validation Zod
  let raw: unknown;
  try {
    raw = JSON.parse(data);
  } catch (e) {
    throw new Error(`Le fichier n'est pas un JSON valide : ${(e as Error).message}`);
  }
  const parsed = parseProjectFile(raw);
  if (!parsed.ok) {
    throw new Error(`Le fichier .glucose est invalide ou corrompu (${parsed.error}). Le projet courant n'a pas été modifié.`);
  }
  return { project: parsed.project, path };
}
