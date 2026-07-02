// Orchestrateurs d'export — relient les builders (scene/svg/png/html/markdown)
// au dialogue de sauvegarde + écriture disque (plugin-fs, scopes Documents/
// Desktop/…). Aucune commande Rust dédiée : tout passe par @tauri-apps/plugin-fs.
//
// IMPORTANT : on écrit TOUT (texte compris) via `writeFile` (octets), PAS
// `writeTextFile`. La capability accorde `fs:allow-write-file` mais PAS
// `fs:allow-write-text-file` → `writeTextFile` lèverait une erreur de permission
// (fichier vide / page blanche). `writeFile` est la commande autorisée.
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { writeFile, readFile } from "@tauri-apps/plugin-fs";
import { documentDir } from "@tauri-apps/api/path";
import { invoke } from "@tauri-apps/api/core";
import { Project } from "../../types";
import { buildScene, ExportScene } from "./scene";
import { sceneToSvg } from "./toSvg";
import { sceneToPngDataUrl } from "./toPng";
import { sceneToHtml } from "./toHtml";
import { projectToMarkdown } from "./toMarkdown";
import { mimeFromExt } from "../assetRef";

export type ExportFormat = "html" | "png" | "svg" | "markdown";

export const FORMAT_META: Record<ExportFormat, { ext: string; label: string; filterName: string }> = {
  html: { ext: "html", label: "HTML interactif", filterName: "Page web autonome" },
  png: { ext: "png", label: "Image PNG (HD)", filterName: "Image PNG" },
  svg: { ext: "svg", label: "Image vectorielle SVG", filterName: "Image vectorielle SVG" },
  markdown: { ext: "md", label: "Markdown", filterName: "Document Markdown" },
};

function sanitize(name: string): string {
  return (name || "glucose").replace(/[\\/:*?"<>|]+/g, "_").trim() || "glucose";
}

async function pickPath(baseName: string, format: ExportFormat): Promise<string | null> {
  const meta = FORMAT_META[format];
  let defaultPath = `${sanitize(baseName)}.${meta.ext}`;
  try {
    const dir = await documentDir();
    if (dir) defaultPath = `${dir}/${defaultPath}`;
  } catch { /* fallback nom seul */ }
  const path = await saveDialog({
    defaultPath,
    filters: [{ name: meta.filterName, extensions: [meta.ext] }],
  });
  if (!path) return null;
  return path.endsWith(`.${meta.ext}`) ? path : `${path}.${meta.ext}`;
}

function dataUrlToBytes(dataUrl: string): Uint8Array {
  const b64 = dataUrl.split(",")[1] ?? "";
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

const textToBytes = (s: string): Uint8Array => new TextEncoder().encode(s);

/**
 * Résout toutes les images locales ou de type 'asset:' d'une scène d'export en data URLs Base64.
 */
async function resolveSceneImages(scene: ExportScene): Promise<void> {
  const promises = scene.images.map(async (im) => {
    if (!im.href) return;
    if (im.href.startsWith("data:")) return;
    if (im.href.startsWith("http://") || im.href.startsWith("https://")) return;

    if (im.href.startsWith("asset:")) {
      const filename = im.href.slice("asset:".length);
      try {
        const dataUrl = await invoke<string>("load_asset", { filename });
        im.href = dataUrl;
      } catch (e) {
        console.warn(`[export] Échec chargement asset ${filename} :`, e);
      }
      return;
    }

    // Chemin local (file:// ou chemin absolu)
    try {
      let cleanPath = im.href;
      if (cleanPath.startsWith("file:///")) {
        cleanPath = cleanPath.slice(8);
      } else if (cleanPath.startsWith("file://")) {
        cleanPath = cleanPath.slice(7);
      }
      const bytes = await readFile(cleanPath);
      const ext = cleanPath.split(".").pop() || "png";
      const mime = mimeFromExt(ext);

      // Conversion Uint8Array en base64
      const CHUNK = 32768;
      let binary = "";
      for (let i = 0; i < bytes.length; i += CHUNK) {
        binary += String.fromCharCode.apply(null, Array.from(bytes.subarray(i, i + CHUNK)));
      }
      const b64 = btoa(binary);
      im.href = `data:${mime};base64,${b64}`;
    } catch (e) {
      console.warn(`[export] Échec chargement fichier local ${im.href} :`, e);
    }
  });

  await Promise.all(promises);
}

/**
 * Exporte le board actif du projet dans `format`. Ouvre un dialogue de
 * sauvegarde, écrit le fichier, et renvoie le chemin (ou null si annulé).
 */
export async function exportProject(project: Project, format: ExportFormat): Promise<string | null> {
  const baseName = `${project.name || "glucose"}`;

  if (format === "markdown") {
    const md = projectToMarkdown(project);
    const path = await pickPath(baseName, format);
    if (!path) return null;
    await writeFile(path, textToBytes(md));
    return path;
  }

  // Les autres formats (html/svg/png) partent d'une scène avec images.
  const scene = buildScene(project, { includeImages: true });
  await resolveSceneImages(scene);

  if (format === "svg") {
    const svg = sceneToSvg(scene);
    const path = await pickPath(baseName, format);
    if (!path) return null;
    await writeFile(path, textToBytes(svg));
    return path;
  }

  if (format === "html") {
    const html = sceneToHtml(scene);
    const path = await pickPath(baseName, format);
    if (!path) return null;
    await writeFile(path, textToBytes(html));
    return path;
  }

  if (format === "png") {
    // On rastérise AVANT le dialogue (rendu synchrone tant que le canvas est chaud).
    const dataUrl = await sceneToPngDataUrl(scene, 2);
    const path = await pickPath(baseName, format);
    if (!path) return null;
    await writeFile(path, dataUrlToBytes(dataUrl));
    return path;
  }

  return null;
}
