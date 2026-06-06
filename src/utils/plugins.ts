// Phase 8 — Système de plugins (côté frontend).
//
// Un plugin = un binaire compagnon + un manifeste, installés côté Rust dans
// `app_data_dir/plugins/<id>/`. Ici on expose juste les wrappers typés des
// commandes Tauri `list_plugins` / `run_plugin`, plus le « tronc » :
// exécuter un plugin sur un texte puis CHARGER son résultat comme un nouveau
// board (sans écraser le travail en cours).

import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { homeDir } from "@tauri-apps/api/path";
import { parseProjectFile } from "../store/projectSchema";
import { useGlucoseStore } from "../store";
import type { Board } from "../types";

/** Un choix d'une option de type "enum". */
export interface PluginOptionChoice {
  value: string;
  label: string;
}

/** Une option déclarée par le plugin (la "recette") — l'UI la rend automatiquement. */
export interface PluginOption {
  id: string;
  label: string;
  type?: string; // "enum" | "bool"
  choices?: PluginOptionChoice[];
  default?: string;
  description?: string;
}

/** Reflet du `PluginManifest` Rust (champs optionnels = forward-compatibles). */
export interface PluginManifest {
  id: string;
  name: string;
  description?: string;
  version?: string;
  binary?: string;
  command?: string;
  options?: PluginOption[];
}

/** Sonde matérielle + modèle Ollama recommandé (champs = snake_case côté Rust). */
export interface SystemSpecs {
  ram_gb: number;
  cores: number;
  vram_gb: number | null;
  recommended_model: string;
}

/** État du démon Ollama local. */
export interface OllamaStatus {
  reachable: boolean;
  models: string[];
}

/** Découvre les plugins installés. Liste vide si aucun (pas une erreur). */
export async function listPlugins(): Promise<PluginManifest[]> {
  return await invoke<PluginManifest[]>("list_plugins");
}

/** Installe un plugin depuis un dossier choisi (manifest.json + binaire). */
export async function installPluginFromDir(): Promise<PluginManifest | null> {
  const dir = await openDialog({ directory: true, multiple: false, title: "Dossier du plugin" });
  if (!dir) return null;
  return await invoke<PluginManifest>("install_plugin", { srcDir: dir as string });
}

/** Sonde la machine (RAM/cœurs/VRAM) et renvoie le modèle Ollama conseillé. */
export async function systemSpecs(): Promise<SystemSpecs> {
  return await invoke<SystemSpecs>("system_specs");
}

/** Démon Ollama joignable ? quels modèles installés ? */
export async function ollamaStatus(): Promise<OllamaStatus> {
  return await invoke<OllamaStatus>("ollama_status");
}

/** Télécharge un modèle via `ollama pull` (progression via onModelProgress). */
export async function pullModel(model: string): Promise<void> {
  await invoke("pull_model", { model });
}

/** Installe Ollama si absent (winget) puis démarre le serveur. Renvoie un message. */
export async function installOllama(): Promise<string> {
  return await invoke<string>("install_ollama");
}

/** S'abonne à la progression de l'installation d'Ollama. */
export function onOllamaInstallProgress(cb: (line: string) => void): Promise<UnlistenFn> {
  return listen<{ line: string }>("ollama-install-progress", (e) => cb(e.payload.line));
}

/** S'abonne à la progression du MOTEUR (chaque ligne de sortie). */
export function onPluginProgress(cb: (line: string) => void): Promise<UnlistenFn> {
  return listen<{ line: string }>("plugin-progress", (e) => cb(e.payload.line));
}

/** S'abonne à la progression du TÉLÉCHARGEMENT de modèle. */
export function onModelProgress(cb: (line: string) => void): Promise<UnlistenFn> {
  return listen<{ line: string }>("model-progress", (e) => cb(e.payload.line));
}

/** Ouvre un sélecteur de fichier TEXTE et renvoie son chemin (ou null). */
export async function pickTextFile(): Promise<string | null> {
  let defaultPath: string | undefined;
  try {
    defaultPath = await homeDir();
  } catch {
    /* pas de homeDir : on laisse le dialogue choisir */
  }
  const result = await openDialog({
    defaultPath,
    multiple: false,
    filters: [{ name: "Texte", extensions: ["txt", "md", "markdown", "json", "jsonl"] }],
  });
  return (result as string | null) ?? null;
}

/**
 * Lance un plugin sur un texte, puis charge son `.glucose` comme un NOUVEAU
 * board (le travail en cours est préservé). Renvoie l'id du board créé.
 *
 * Peut être long : le moteur fait tourner une IA locale (plusieurs minutes sur
 * une grosse conversation). L'appelant affiche un état d'attente.
 */
export async function runPluginAndImport(
  pluginId: string,
  textPath: string,
  options?: Record<string, string>,
): Promise<string> {
  // 1) Le backend lance le binaire (avec les options de la recette) et renvoie le
  //    chemin du .glucose produit.
  const glucosePath = await invoke<string>("run_plugin", { pluginId, textPath, options: options ?? null });
  // 2) On relit ce fichier (app_data_dir est dans le scope autorisé).
  const json = await invoke<string>("read_project_file", { path: glucosePath });
  // 3) Validation Zod (même garde-fou que l'ouverture manuelle d'un .glucose).
  const parsed = parseProjectFile(JSON.parse(json));
  if (!parsed.ok) {
    throw new Error(`Le plugin a produit un .glucose invalide : ${parsed.error}`);
  }
  const board = parsed.project.boards[0] as Board | undefined;
  if (!board) {
    throw new Error("Le plugin n'a produit aucun board.");
  }
  // 4) On l'ajoute comme nouveau board et on bascule dessus.
  return useGlucoseStore.getState().importBoard(board);
}
