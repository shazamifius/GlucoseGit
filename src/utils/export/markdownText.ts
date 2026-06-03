// Helpers Markdown → texte, partagés par les exports SVG/PNG.
//
// On ne vise PAS un moteur Markdown complet (l'export HTML, lui, rend le vrai
// Markdown via react-markdown). Ici on traite l'INLINE (**gras**, *italique*,
// `code`, liens, délimiteurs LaTeX) et le découpage en blocs (titres #/##/###,
// puces -, paragraphes) pour produire du <text>/<tspan> SVG sélectionnable.

export interface TextRun {
  text: string;
  bold: boolean;
  italic: boolean;
}

export type BlockKind = "h1" | "h2" | "h3" | "bullet" | "para";

export interface TextBlock {
  kind: BlockKind;
  runs: TextRun[];
}

/** Retire les délimiteurs LaTeX ($...$, $$...$$, \( \), \[ \]) en gardant le contenu. */
function stripLatexDelimiters(s: string): string {
  return s
    .replace(/\$\$([\s\S]*?)\$\$/g, "$1")
    .replace(/\$([^$\n]+?)\$/g, "$1")
    .replace(/\\\(([\s\S]*?)\\\)/g, "$1")
    .replace(/\\\[([\s\S]*?)\\\]/g, "$1");
}

/** Aplati l'inline Markdown en texte nu (pour mesure / fallback). */
export function stripInlineMarkdown(s: string): string {
  return stripLatexDelimiters(s)
    .replace(/!\[[^\]]*\]\([^)]*\)/g, "")            // images
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")          // liens → libellé
    .replace(/`([^`]+)`/g, "$1")                       // code inline
    .replace(/\*\*([^*]+)\*\*/g, "$1")                // gras
    .replace(/__([^_]+)__/g, "$1")
    .replace(/(^|[^*])\*([^*\n]+)\*/g, "$1$2")        // italique *...*
    .replace(/(^|[^_])_([^_\n]+)_/g, "$1$2")          // italique _..._
    .replace(/~~([^~]+)~~/g, "$1")                     // barré
    .trim();
}

/**
 * Parse une ligne inline en runs stylés (gras/italique). On gère `**`, `*`,
 * `_`, `` ` ``, les liens `[txt](url)` (→ txt), et on neutralise le LaTeX.
 * Volontairement simple et robuste : pas d'imbrication gras+italique profonde.
 */
export function parseInlineRuns(input: string): TextRun[] {
  const s = stripLatexDelimiters(input)
    .replace(/!\[[^\]]*\]\([^)]*\)/g, "")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/`([^`]+)`/g, "$1");

  const runs: TextRun[] = [];
  let i = 0;
  let buf = "";
  let bold = false;
  let italic = false;

  const flush = () => {
    if (buf) {
      runs.push({ text: buf, bold, italic });
      buf = "";
    }
  };

  while (i < s.length) {
    const two = s.slice(i, i + 2);
    if (two === "**" || two === "__") {
      flush();
      bold = !bold;
      i += 2;
      continue;
    }
    const ch = s[i];
    if ((ch === "*" || ch === "_")) {
      // n'ouvre/ferme l'italique que si entouré de non-espace côté contenu
      flush();
      italic = !italic;
      i += 1;
      continue;
    }
    buf += ch;
    i += 1;
  }
  flush();
  return runs.length ? runs : [{ text: "", bold: false, italic: false }];
}

/** Découpe le texte d'une carte en blocs (titres, puces, paragraphes). */
export function splitBlocks(text: string): TextBlock[] {
  const blocks: TextBlock[] = [];
  const lines = (text || "").replace(/\r\n/g, "\n").split("\n");
  for (const raw of lines) {
    const line = raw.trim();
    if (!line) continue;
    let kind: BlockKind = "para";
    let content = line;
    if (/^###\s+/.test(line)) { kind = "h3"; content = line.replace(/^###\s+/, ""); }
    else if (/^##\s+/.test(line)) { kind = "h2"; content = line.replace(/^##\s+/, ""); }
    else if (/^#\s+/.test(line)) { kind = "h1"; content = line.replace(/^#\s+/, ""); }
    else if (/^[-*+]\s+/.test(line)) { kind = "bullet"; content = line.replace(/^[-*+]\s+/, ""); }
    else if (/^>\s+/.test(line)) { kind = "para"; content = line.replace(/^>\s+/, ""); }
    blocks.push({ kind, runs: parseInlineRuns(content) });
  }
  return blocks;
}

export type MeasureFn = (text: string, fontSize: number, bold: boolean, italic: boolean) => number;

export interface WrappedLine {
  runs: TextRun[];
}

/**
 * Enroule une liste de runs (une ligne logique) en lignes physiques bornées à
 * `maxWidth`. Coupe au mot ; les runs adjacents de même style restent fusionnés
 * dans le rendu (on émet 1 tspan par run conservé).
 */
export function wrapRuns(runs: TextRun[], fontSize: number, maxWidth: number, measure: MeasureFn): WrappedLine[] {
  const lines: WrappedLine[] = [];
  let current: TextRun[] = [];
  let currentWidth = 0;

  const pushLine = () => {
    if (current.length) lines.push({ runs: mergeAdjacent(current) });
    current = [];
    currentWidth = 0;
  };

  for (const run of runs) {
    // découpe le run en (mot + espace) pour pouvoir wrapper
    const tokens = run.text.match(/\S+\s*|\s+/g) || [];
    for (const tok of tokens) {
      const w = measure(tok, fontSize, run.bold, run.italic);
      if (currentWidth + w > maxWidth && currentWidth > 0) {
        pushLine();
      }
      // si un seul mot dépasse, on le pose quand même (pas de coupe intra-mot)
      current.push({ text: tok, bold: run.bold, italic: run.italic });
      currentWidth += w;
    }
  }
  pushLine();
  return lines.length ? lines : [{ runs: [{ text: "", bold: false, italic: false }] }];
}

function mergeAdjacent(runs: TextRun[]): TextRun[] {
  const out: TextRun[] = [];
  for (const r of runs) {
    const last = out[out.length - 1];
    if (last && last.bold === r.bold && last.italic === r.italic) {
      last.text += r.text;
    } else {
      out.push({ ...r });
    }
  }
  // retire l'espace de fin de ligne (cosmétique)
  if (out.length) out[out.length - 1].text = out[out.length - 1].text.replace(/\s+$/, "");
  return out;
}
