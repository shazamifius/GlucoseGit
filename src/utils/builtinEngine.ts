// Moteur INTÉGRÉ « Cours magistral ».
//
// Pourquoi il existe : jusqu'ici, transformer un texte en carte exigeait
// d'installer un binaire compagnon (Phase 8, cf. `plugins.ts`). Tant qu'aucun
// plugin n'était installé, « Lancer » restait grisé pour toujours — et le
// binaire n'est pas distribué avec l'application. Ce moteur fait le même
// travail EN INTERNE, en parlant directement à l'IA locale via `ollama_generate`
// (API HTTP du démon). Aucun binaire, aucun PATH, aucun redémarrage.
//
// Un plugin externe reste prioritaire quand l'utilisateur en installe un : ce
// moteur est simplement le choix par défaut, toujours disponible.

import { invoke } from "@tauri-apps/api/core";
import { nanoid } from "./nanoid";
import { useGlucoseStore } from "../store";
import type { Annotation, Board } from "../types";
import type { PluginManifest } from "./plugins";

/** Identifiant réservé du moteur intégré (jamais un dossier sur le disque). */
export const BUILTIN_ENGINE_ID = "builtin-cours";

/** Le moteur se présente comme un plugin : l'UI n'a qu'une seule liste à rendre. */
export const BUILTIN_ENGINE: PluginManifest = {
  id: BUILTIN_ENGINE_ID,
  name: "Cours magistral (intégré)",
  description: "Transforme un texte en carte de concepts avec l'IA locale. Aucun binaire à installer.",
  version: "1.0",
  options: [
    {
      id: "densite",
      label: "Densité",
      type: "enum",
      default: "normal",
      choices: [
        { value: "concis", label: "Concis — les idées maîtresses" },
        { value: "normal", label: "Normal — équilibré" },
        { value: "detaille", label: "Détaillé — chaque nuance" },
      ],
      description: "Plus c'est détaillé, plus il y a de cartes et plus le traitement est long.",
    },
    {
      id: "disposition",
      label: "Disposition",
      type: "enum",
      default: "grille",
      choices: [
        { value: "grille", label: "Grille — lecture en blocs" },
        { value: "fil", label: "Fil — une section par ligne" },
      ],
    },
  ],
};

// ── Géométrie de la carte ────────────────────────────────────────────────────
const CARD_W = 320;
const GAP_X = 48;
const GAP_Y = 56;
const CARDS_PER_ROW = 4;
const SECTION_GAP_Y = 150;
const HEADER_H = 70;

/** Palette de sections : une teinte par section, pour les distinguer d'un coup d'œil. */
const SECTION_COLORS = ["#1e3a34", "#2a2340", "#34281e", "#1e2b3a", "#33203a", "#1f331f"];

/** Une carte produite par le modèle (déjà normalisée). */
export interface EngineCard {
  titre: string;
  resume: string;
  points: string[];
}

/** Une section du cours = un morceau du texte source + ses cartes. */
export interface EngineSection {
  titre: string;
  cartes: EngineCard[];
}

// ────────────────────────────────────────────────────────────────────────────
// Fonctions PURES (testées unitairement — c'est là que vivent les bugs)
// ────────────────────────────────────────────────────────────────────────────

/**
 * Découpe le texte source en morceaux traitables par le modèle, sans couper au
 * milieu d'une idée : on préfère les titres Markdown, puis les paragraphes.
 * Un morceau trop gros pour un seul appel produit une réponse tronquée — d'où
 * le plafond `maxChars`.
 */
export function splitSource(text: string, maxChars = 4500): string[] {
  const normalized = text.replace(/\r\n?/g, "\n").trim();
  if (!normalized) return [];

  // 1) Blocs naturels : une section commence à un titre Markdown, sinon à une
  //    ligne vide.
  const blocks: string[] = [];
  let current: string[] = [];
  for (const line of normalized.split("\n")) {
    const isHeading = /^#{1,6}\s+\S/.test(line);
    if (isHeading && current.some((l) => l.trim())) {
      blocks.push(current.join("\n").trim());
      current = [];
    }
    current.push(line);
  }
  if (current.some((l) => l.trim())) blocks.push(current.join("\n").trim());

  // 2) Un bloc plus gros que le plafond est recoupé sur les paragraphes, puis
  //    en dernier recours à la brute (un pavé sans respiration existe).
  const pieces: string[] = [];
  for (const block of blocks) {
    if (block.length <= maxChars) {
      pieces.push(block);
      continue;
    }
    let buffer = "";
    for (const para of block.split(/\n{2,}/)) {
      for (const chunk of hardSplit(para, maxChars)) {
        if (buffer && buffer.length + chunk.length + 2 > maxChars) {
          pieces.push(buffer.trim());
          buffer = "";
        }
        buffer += (buffer ? "\n\n" : "") + chunk;
      }
    }
    if (buffer.trim()) pieces.push(buffer.trim());
  }

  // 3) Regroupe les miettes : un appel par phrase serait absurde (et très lent).
  const out: string[] = [];
  for (const piece of pieces) {
    const last = out[out.length - 1];
    if (last && last.length + piece.length + 2 <= maxChars) {
      out[out.length - 1] = `${last}\n\n${piece}`;
    } else {
      out.push(piece);
    }
  }
  return out.filter((p) => p.trim().length > 0);
}

/** Coupe une chaîne trop longue en tranches de `max` (dernier recours). */
function hardSplit(s: string, max: number): string[] {
  if (s.length <= max) return [s];
  const out: string[] = [];
  for (let i = 0; i < s.length; i += max) out.push(s.slice(i, i + max));
  return out;
}

/** Titre lisible d'un morceau : son titre Markdown, sinon ses premiers mots. */
export function sectionTitle(chunk: string, index: number): string {
  const heading = chunk.split("\n").find((l) => /^#{1,6}\s+\S/.test(l));
  if (heading) return heading.replace(/^#{1,6}\s+/, "").trim().slice(0, 80);
  const firstWords = chunk.replace(/\s+/g, " ").trim().slice(0, 60).trim();
  return firstWords ? `${firstWords}…` : `Partie ${index + 1}`;
}

/**
 * Extrait les cartes de la réponse du modèle. Ollama en mode `format: json`
 * renvoie du JSON valide, mais PAS toujours la forme demandée (tantôt
 * `{cartes:[…]}`, tantôt un tableau nu, tantôt des clés anglaises) — et un
 * modèle bavard peut encadrer sa réponse de texte. On accepte tout ça plutôt
 * que de perdre un traitement de plusieurs minutes sur une virgule.
 */
export function parseCards(raw: string): EngineCard[] {
  const value = extractJson(raw);
  if (value === null) return [];
  const list = Array.isArray(value)
    ? value
    : (firstArray(value as Record<string, unknown>) ?? []);
  const cards: EngineCard[] = [];
  for (const item of list) {
    if (typeof item === "string") {
      const titre = item.trim();
      if (titre) cards.push({ titre: titre.slice(0, 120), resume: "", points: [] });
      continue;
    }
    if (!item || typeof item !== "object") continue;
    const o = item as Record<string, unknown>;
    const titre = str(o.titre ?? o.title ?? o.nom ?? o.concept ?? o.name);
    const resume = str(o.resume ?? o.résumé ?? o.summary ?? o.description ?? o.contenu);
    const points = arr(o.points ?? o.details ?? o.key_points ?? o.puces ?? o.bullets);
    if (!titre && !resume && points.length === 0) continue;
    cards.push({
      titre: (titre || resume.slice(0, 60) || "Sans titre").slice(0, 120),
      resume: resume.slice(0, 600),
      points: points.slice(0, 8).map((p) => p.slice(0, 220)),
    });
  }
  return cards;
}

/** Premier tableau trouvé dans un objet (la clé exacte varie selon le modèle). */
function firstArray(o: Record<string, unknown>): unknown[] | null {
  for (const key of ["cartes", "cards", "concepts", "items", "resultats", "results"]) {
    if (Array.isArray(o[key])) return o[key] as unknown[];
  }
  for (const v of Object.values(o)) if (Array.isArray(v)) return v as unknown[];
  return null;
}

/** JSON.parse tolérant : accepte les fences ```json et le texte autour. */
function extractJson(raw: string): unknown {
  const cleaned = raw.replace(/```(?:json)?/gi, "").trim();
  try {
    return JSON.parse(cleaned);
  } catch {
    /* on tente une extraction ciblée ci-dessous */
  }
  const start = cleaned.search(/[[{]/);
  if (start < 0) return null;
  const openChar = cleaned[start];
  const closeChar = openChar === "[" ? "]" : "}";
  const end = cleaned.lastIndexOf(closeChar);
  if (end <= start) return null;
  try {
    return JSON.parse(cleaned.slice(start, end + 1));
  } catch {
    return null;
  }
}

function str(v: unknown): string {
  if (typeof v === "string") return v.trim();
  if (typeof v === "number" || typeof v === "boolean") return String(v);
  return "";
}
function arr(v: unknown): string[] {
  if (!Array.isArray(v)) return typeof v === "string" && v.trim() ? [v.trim()] : [];
  return v.map(str).filter(Boolean);
}

/**
 * Choisit le modèle à utiliser : le modèle conseillé s'il est installé, sinon
 * une variante de la même famille, sinon n'importe quel modèle présent. Sans ce
 * repli, un utilisateur ayant déjà un autre modèle se voyait refuser le
 * lancement pour rien.
 */
export function chooseModel(recommended: string | null, installed: string[]): string | null {
  const present = installed.filter((m) => m.trim().length > 0);
  if (present.length === 0) return null;
  if (recommended) {
    const exact = present.find((m) => m === recommended);
    if (exact) return exact;
    const family = recommended.split(":")[0];
    const sameFamily = present.find((m) => m.split(":")[0] === family);
    if (sameFamily) return sameFamily;
  }
  return present[0];
}

/**
 * Taille (en milliards de paramètres) lue dans le nom du modèle : « qwen2.5:32b »
 * → 32. Sert uniquement à AVERTIR : un modèle qui dépasse la VRAM tourne sur le
 * CPU, à quelques mots par minute. null si le nom ne dit rien.
 */
export function modelSizeB(name: string): number | null {
  const m = /(\d+(?:\.\d+)?)\s*b\b/i.exec(name.split(":").pop() ?? name);
  if (!m) return null;
  const n = Number.parseFloat(m[1]);
  return Number.isFinite(n) && n > 0 ? n : null;
}

/** Hauteur d'une carte : assez haute pour son contenu, jamais démesurée. */
export function cardHeight(card: EngineCard): number {
  const body = card.resume.length + card.points.reduce((n, p) => n + p.length + 2, 0);
  const lines = Math.ceil(body / 38) + card.points.length + 2;
  return Math.min(420, Math.max(140, 40 + lines * 19));
}

/** Corps Markdown d'une carte (le rendu Markdown est natif côté canvas). */
export function cardMarkdown(card: EngineCard): string {
  const parts = [`**${card.titre}**`];
  if (card.resume) parts.push(card.resume);
  if (card.points.length) parts.push(card.points.map((p) => `- ${p}`).join("\n"));
  return parts.join("\n\n");
}

/**
 * Construit le board à partir des sections. Chaque section = un titre + ses
 * cartes, reliées entre elles dans l'ordre de lecture. Fonction pure : c'est
 * elle qu'on teste, pas l'appel réseau.
 */
export function buildBoard(
  name: string,
  sections: EngineSection[],
  disposition: string = "grille",
): Board {
  const annotations: Annotation[] = [];
  const perRow = disposition === "fil" ? Number.POSITIVE_INFINITY : CARDS_PER_ROW;
  let y = 0;

  sections.forEach((section, si) => {
    const color = SECTION_COLORS[si % SECTION_COLORS.length];
    annotations.push({
      id: nanoid(),
      type: "text",
      x: 0,
      y,
      text: `## ${section.titre}`,
      fontSize: 26,
      color: "#e8efe9",
      width: 620,
    });
    y += HEADER_H;

    let rowHeight = 0;
    const ids: string[] = [];
    const boxes: { x: number; y: number; w: number; h: number }[] = [];

    section.cartes.forEach((card, ci) => {
      const col = perRow === Number.POSITIVE_INFINITY ? ci : ci % perRow;
      const h = cardHeight(card);
      // Une nouvelle rangée démarre sous la plus haute carte de la précédente.
      if (col === 0 && ci > 0) {
        y += rowHeight + GAP_Y;
        rowHeight = 0;
      }
      rowHeight = Math.max(rowHeight, h);
      const x = col * (CARD_W + GAP_X);
      const cy = y;
      const id = nanoid();
      ids.push(id);
      boxes.push({ x, y: cy, w: CARD_W, h });
      annotations.push({
        id,
        type: "sticky",
        x,
        y: cy,
        text: cardMarkdown(card),
        bgColor: color,
        color: "#dfe7e2",
        width: CARD_W,
        height: h,
        fontSize: 13,
      });
    });

    // Fil de lecture : carte n → carte n+1 (attachées, donc elles suivent si on
    // déplace une carte).
    for (let i = 0; i < ids.length - 1; i++) {
      const a = boxes[i];
      const b = boxes[i + 1];
      annotations.push({
        id: nanoid(),
        type: "arrow",
        x: a.x + a.w,
        y: a.y + a.h / 2,
        x2: b.x,
        y2: b.y + b.h / 2,
        arrowType: "curved",
        color: "#3f5c52",
        strokeWidth: 2,
        sourceId: ids[i],
        targetId: ids[i + 1],
      });
    }

    y += rowHeight + SECTION_GAP_Y;
  });

  const now = Date.now();
  return {
    id: nanoid(),
    name,
    images: [],
    annotations,
    panels: [],
    zones: [],
    folders: [],
    viewport: { x: 0, y: 0, scale: 1 },
    createdAt: now,
    updatedAt: now,
  };
}

// ────────────────────────────────────────────────────────────────────────────
// Exécution (IA locale)
// ────────────────────────────────────────────────────────────────────────────

const SYSTEM_PROMPT =
  "Tu es un professeur qui structure un cours. Tu réponds UNIQUEMENT en JSON valide, " +
  "sans commentaire ni texte autour. Tu écris dans la langue du texte fourni.";

/** Nombre de cartes visé par morceau, selon la densité demandée. */
function targetCards(densite: string): string {
  if (densite === "concis") return "2 à 3";
  if (densite === "detaille") return "6 à 9";
  return "4 à 6";
}

function buildPrompt(chunk: string, densite: string): string {
  return [
    `Découpe le passage ci-dessous en ${targetCards(densite)} cartes de cours.`,
    "Chaque carte porte UNE idée : un titre court, un résumé de 1 à 3 phrases,",
    "et 0 à 5 points clés. N'invente rien qui ne soit pas dans le passage.",
    "",
    'Réponds exactement dans cette forme : {"cartes":[{"titre":"…","resume":"…","points":["…"]}]}',
    "",
    "PASSAGE :",
    chunk,
  ].join("\n");
}

export interface RunOptions {
  textPath: string;
  model: string;
  options?: Record<string, string>;
  /** Progression 0-100 + libellé lisible (aucun event Tauri : appel direct). */
  onProgress?: (pct: number, label: string) => void;
  /** Annulation coopérative : vrai → on s'arrête entre deux morceaux. */
  shouldStop?: () => boolean;
}

/** Lit le texte source EN ENTIER (le plafond inline de 100 Ko ne suffit pas ici). */
async function readSource(path: string): Promise<{ content: string; truncated: boolean }> {
  return await invoke<{ content: string; truncated: boolean }>("read_text_file_inline", {
    path,
    maxBytes: 4_000_000,
  });
}

/** Une requête au modèle local, JSON contraint. */
async function ask(model: string, prompt: string): Promise<string> {
  return await invoke<string>("ollama_generate", {
    model,
    prompt,
    system: SYSTEM_PROMPT,
    json: true,
    timeoutSecs: 900,
  });
}

/**
 * Exécute le moteur intégré et importe le résultat comme un NOUVEAU board (le
 * travail en cours n'est jamais écrasé). Renvoie l'id du board créé.
 */
export async function runBuiltinEngine(opts: RunOptions): Promise<string> {
  const { textPath, model, options = {}, onProgress, shouldStop } = opts;
  const densite = options.densite ?? "normal";
  const disposition = options.disposition ?? "grille";

  onProgress?.(4, "Lecture du texte…");
  const { content, truncated } = await readSource(textPath);
  if (!content.trim()) throw new Error("Le fichier source est vide.");

  const chunks = splitSource(content);
  if (chunks.length === 0) throw new Error("Rien à traiter dans ce fichier.");

  const sections: EngineSection[] = [];
  const failures: string[] = [];
  for (let i = 0; i < chunks.length; i++) {
    if (shouldStop?.()) break;
    const titre = sectionTitle(chunks[i], i);
    onProgress?.(
      6 + Math.round((88 * i) / chunks.length),
      `Partie ${i + 1}/${chunks.length} — ${titre}`,
    );
    try {
      const cartes = parseCards(await ask(model, buildPrompt(chunks[i], densite)));
      if (cartes.length > 0) sections.push({ titre, cartes });
      else failures.push(titre);
    } catch (e) {
      // Un morceau raté ne doit pas jeter les vingt autres : on note et on
      // continue. L'échec total est signalé plus bas.
      failures.push(titre);
      if (/ne répond pas|n'est pas installé/i.test(String((e as Error)?.message ?? e))) {
        throw e; // panne franche de l'IA : inutile d'insister 20 fois.
      }
    }
  }

  if (sections.length === 0) {
    throw new Error(
      "L'IA locale n'a produit aucune carte exploitable. Essaie un texte plus court, " +
        "ou un modèle plus grand si ta machine le permet.",
    );
  }

  onProgress?.(96, "Construction de la carte…");
  const name = fileTitle(textPath);
  const board = buildBoard(name, sections, disposition);
  if (truncated) {
    board.annotations.unshift({
      id: nanoid(),
      type: "sticky",
      x: -CARD_W - GAP_X,
      y: 0,
      text: "**Texte tronqué**\n\nLe fichier dépasse 4 Mo : seule la première partie a été traitée.",
      bgColor: "#3a2a1e",
      color: "#e8d9c8",
      width: CARD_W,
      height: 140,
      fontSize: 13,
    });
  }
  const boardId = useGlucoseStore.getState().importBoard(board);
  onProgress?.(100, failures.length ? `Terminé (${failures.length} passage(s) ignoré(s))` : "Terminé");
  return boardId;
}

/** Nom du board : le nom du fichier, sans extension ni chemin. */
export function fileTitle(path: string): string {
  const base = path.split(/[\\/]/).pop() ?? path;
  return base.replace(/\.[^.]+$/, "") || "Cours";
}
