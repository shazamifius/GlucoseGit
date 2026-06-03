// Export SVG — rend la scène en SVG NATIF (rect/path/text/tspan). Texte
// sélectionnable, zoom infini net, ouvrable dans tout navigateur. Sert aussi
// d'intermédiaire rasterisé pour le PNG (pas de <foreignObject> → rasterisation
// fiable via canvas).
import {
  ExportScene, SceneCard, SceneArrow, SceneMembrane, SceneSticky, SceneImage,
  makeMeasure, MeasureFn,
} from "./scene";
import { splitBlocks, wrapRuns, TextRun } from "./markdownText";

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

const PAD_X = 24, PAD_Y = 16, LINE_H = 1.45;
const HEADING_SCALE: Record<string, number> = { h1: 1.5, h2: 1.3, h3: 1.15 };

// ── Cartes ──────────────────────────────────────────────────────────────────
function renderCard(c: SceneCard, measure: MeasureFn): string {
  const rx = 24;
  const parts: string[] = [];
  parts.push(`<g class="card" data-id="${esc(c.id)}">`);
  // halo
  parts.push(`<rect x="${c.x}" y="${c.y}" width="${c.w}" height="${c.h}" rx="${rx}" fill="${c.auraColor}" opacity="0.13" filter="url(#cardGlow)"/>`);
  // fond + bord
  parts.push(`<rect x="${c.x}" y="${c.y}" width="${c.w}" height="${c.h}" rx="${rx}" fill="${c.auraColor}" fill-opacity="0.05" stroke="${c.auraColor}" stroke-opacity="0.22" stroke-width="1.5"/>`);

  // texte
  const contentW = c.w - PAD_X * 2;
  let cursorY = c.y + PAD_Y;
  const blocks = splitBlocks(c.text);
  for (const b of blocks) {
    const scale = HEADING_SCALE[b.kind] ?? 1;
    const fs = c.fontSize * scale;
    const isHeading = b.kind.startsWith("h");
    const bulletIndent = b.kind === "bullet" ? fs * 1.2 : 0;
    const color = isHeading ? c.auraColor : c.textColor;
    const weight = isHeading ? 700 : 400;
    const lines = wrapRuns(b.runs, fs, contentW - bulletIndent, measure);

    lines.forEach((line, li) => {
      cursorY += fs * LINE_H;
      const baseX = c.x + PAD_X + bulletIndent;
      const prefix = (b.kind === "bullet" && li === 0)
        ? `<tspan x="${c.x + PAD_X}" fill="${c.auraColor}">• </tspan>`
        : "";
      const spans = line.runs.map((r: TextRun) => {
        const fw = r.bold || isHeading ? 700 : weight;
        const fsAttr = r.italic ? ` font-style="italic"` : "";
        return `<tspan font-weight="${fw}"${fsAttr}>${esc(r.text)}</tspan>`;
      }).join("");
      parts.push(
        `<text x="${baseX}" y="${cursorY.toFixed(1)}" font-family="system-ui, -apple-system, Segoe UI, sans-serif" font-size="${fs.toFixed(1)}" fill="${color}">${li === 0 ? prefix : ""}${spans}</text>`
      );
    });
    cursorY += isHeading ? 6 : 2;
  }
  parts.push(`</g>`);
  return parts.join("");
}

// ── Membranes (zones) ────────────────────────────────────────────────────────
function renderMembrane(m: SceneMembrane): string {
  const rx = 28;
  const parts = [
    `<rect x="${m.x}" y="${m.y}" width="${m.w}" height="${m.h}" rx="${rx}" fill="${m.color}" fill-opacity="0.06" stroke="${m.color}" stroke-opacity="0.4" stroke-width="2" stroke-dasharray="10 8"/>`,
  ];
  if (m.text) {
    parts.push(`<text x="${m.x + 18}" y="${m.y + 30}" font-family="system-ui, sans-serif" font-size="20" font-weight="700" fill="${m.color}" opacity="0.85">${esc(m.text)}</text>`);
  }
  return parts.join("");
}

// ── Stickies ─────────────────────────────────────────────────────────────────
function renderSticky(s: SceneSticky, measure: MeasureFn): string {
  const parts = [
    `<rect x="${s.x}" y="${s.y}" width="${s.w}" height="${s.h}" rx="6" fill="${s.bgColor}" stroke="rgba(0,0,0,0.15)"/>`,
  ];
  const contentW = s.w - 16;
  let cy = s.y + 18;
  const blocks = splitBlocks(s.text);
  for (const b of blocks) {
    const lines = wrapRuns(b.runs, 13, contentW, measure);
    for (const line of lines) {
      const spans = line.runs.map((r) => `<tspan${r.bold ? ' font-weight="700"' : ""}${r.italic ? ' font-style="italic"' : ""}>${esc(r.text)}</tspan>`).join("");
      parts.push(`<text x="${s.x + 8}" y="${cy}" font-family="system-ui, sans-serif" font-size="13" fill="${s.color}">${spans}</text>`);
      cy += 13 * 1.4;
      if (cy > s.y + s.h - 4) break;
    }
  }
  if (s.operator) {
    parts.push(`<text x="${s.x + s.w - 8}" y="${s.y + 14}" text-anchor="end" font-family="system-ui, sans-serif" font-size="10" font-weight="700" fill="${s.color}" opacity="0.6">${esc(s.operator)}</text>`);
  }
  return parts.join("");
}

// ── Images ───────────────────────────────────────────────────────────────────
function renderImage(im: SceneImage): string {
  const cx = im.x + im.w / 2, cy = im.y + im.h / 2;
  const t = im.rotation ? ` transform="rotate(${im.rotation} ${cx} ${cy})"` : "";
  return `<image x="${im.x}" y="${im.y}" width="${im.w}" height="${im.h}" href="${esc(im.href)}" preserveAspectRatio="xMidYMid slice"${t}/>`;
}

// ── Flèches ──────────────────────────────────────────────────────────────────
function buildPath(pts: { x: number; y: number }[], curved: boolean): string {
  const n = pts.length;
  if (n < 2) return "";
  let d = `M ${pts[0].x.toFixed(1)} ${pts[0].y.toFixed(1)}`;
  if (!curved || n === 2) {
    for (let i = 1; i < n; i++) d += ` L ${pts[i].x.toFixed(1)} ${pts[i].y.toFixed(1)}`;
    return d;
  }
  const p = [
    { x: 2 * pts[0].x - pts[1].x, y: 2 * pts[0].y - pts[1].y },
    ...pts,
    { x: 2 * pts[n - 1].x - pts[n - 2].x, y: 2 * pts[n - 1].y - pts[n - 2].y },
  ];
  for (let i = 1; i < p.length - 2; i++) {
    const p0 = p[i - 1], p1 = p[i], p2 = p[i + 1], p3 = p[i + 2];
    const cp1x = p1.x + (p2.x - p0.x) / 6, cp1y = p1.y + (p2.y - p0.y) / 6;
    const cp2x = p2.x - (p3.x - p1.x) / 6, cp2y = p2.y - (p3.y - p1.y) / 6;
    d += ` C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${p2.x.toFixed(1)} ${p2.y.toFixed(1)}`;
  }
  return d;
}

function arrowHead(tip: { x: number; y: number }, from: { x: number; y: number }, size: number, color: string): string {
  const ang = Math.atan2(tip.y - from.y, tip.x - from.x);
  const a1 = ang + Math.PI - 0.5, a2 = ang + Math.PI + 0.5;
  const x1 = tip.x + Math.cos(a1) * size, y1 = tip.y + Math.sin(a1) * size;
  const x2 = tip.x + Math.cos(a2) * size, y2 = tip.y + Math.sin(a2) * size;
  return `<path d="M ${tip.x.toFixed(1)} ${tip.y.toFixed(1)} L ${x1.toFixed(1)} ${y1.toFixed(1)} L ${x2.toFixed(1)} ${y2.toFixed(1)} Z" fill="${color}"/>`;
}

function renderArrow(a: SceneArrow): { defs: string; body: string } {
  const pts = a.points;
  const n = pts.length;
  if (n < 2) return { defs: "", body: "" };
  const gradId = `grad-${a.id}`;
  const defs = `<linearGradient id="${gradId}" gradientUnits="userSpaceOnUse" x1="${pts[0].x.toFixed(1)}" y1="${pts[0].y.toFixed(1)}" x2="${pts[n - 1].x.toFixed(1)}" y2="${pts[n - 1].y.toFixed(1)}"><stop offset="0" stop-color="${a.colStart}"/><stop offset="1" stop-color="${a.colEnd}"/></linearGradient>`;

  const d = buildPath(pts, a.curved);
  const headSize = 5 + a.strokeWidth * 1.4;
  const parts: string[] = [`<g class="arrow" data-id="${esc(a.id)}">`];
  parts.push(`<path d="${d}" fill="none" stroke="url(#${gradId})" stroke-width="${a.strokeWidth}" stroke-linecap="round" stroke-linejoin="round" opacity="0.92"/>`);
  parts.push(arrowHead(pts[n - 1], pts[n - 2], headSize, a.colEnd));
  if (a.bidirectional) parts.push(arrowHead(pts[0], pts[1], headSize, a.colStart));

  // badge prédicat
  if (a.predicate && a.predicateColor) {
    const r = 11;
    parts.push(`<circle cx="${a.mid.x.toFixed(1)}" cy="${a.mid.y.toFixed(1)}" r="${r}" fill="#15151b" stroke="${a.predicateColor}" stroke-width="2"/>`);
    parts.push(`<text x="${a.mid.x.toFixed(1)}" y="${(a.mid.y + 4).toFixed(1)}" text-anchor="middle" font-family="system-ui, sans-serif" font-size="13" fill="${a.predicateColor}">${esc(a.predicateLabel || "")}</text>`);
  }
  // libellé court
  if (a.label) {
    const ly = a.mid.y - (a.predicate ? 18 : 6);
    parts.push(`<text x="${a.mid.x.toFixed(1)}" y="${ly.toFixed(1)}" text-anchor="middle" font-family="system-ui, sans-serif" font-size="12" font-weight="600" fill="#e8e8f0" paint-order="stroke" stroke="#0d0d0d" stroke-width="3" stroke-linejoin="round">${esc(a.label)}</text>`);
  }
  // marqueur description (statique)
  if (a.longText) {
    const ix = a.mid.x + (a.predicate ? 18 : 10);
    parts.push(`<circle cx="${ix.toFixed(1)}" cy="${a.mid.y.toFixed(1)}" r="7" fill="#15151b" stroke="#4aa3ff" stroke-width="1.5"/>`);
    parts.push(`<text x="${ix.toFixed(1)}" y="${(a.mid.y + 3.5).toFixed(1)}" text-anchor="middle" font-family="Georgia, serif" font-style="italic" font-size="10" fill="#4aa3ff">i</text>`);
  }
  parts.push(`</g>`);
  return { defs, body: parts.join("") };
}

// ── Assemblage ───────────────────────────────────────────────────────────────
export interface SvgOptions {
  /** Fond transparent (sinon fond sombre du board). Défaut: false. */
  transparent?: boolean;
}

export function sceneToSvg(scene: ExportScene, opts: SvgOptions = {}): string {
  const measure = makeMeasure();
  const { bbox, width, height } = scene;

  const arrowDefs: string[] = [];
  const arrowBodies: string[] = [];
  for (const a of scene.arrows) {
    const r = renderArrow(a);
    if (r.defs) arrowDefs.push(r.defs);
    if (r.body) arrowBodies.push(r.body);
  }

  const defs = [
    `<filter id="cardGlow" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="16"/></filter>`,
    ...arrowDefs,
  ].join("");

  const bg = opts.transparent ? "" :
    `<rect x="${bbox.left}" y="${bbox.top}" width="${width}" height="${height}" fill="${scene.background}"/>`;

  // Ordre de peinture : fond → membranes → images → cartes/stickies → flèches.
  const layers = [
    bg,
    scene.membranes.map(renderMembrane).join(""),
    scene.images.map(renderImage).join(""),
    scene.stickies.map((s) => renderSticky(s, measure)).join(""),
    scene.cards.map((c) => renderCard(c, measure)).join(""),
    arrowBodies.join(""),
  ].join("\n");

  return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${Math.round(width)}" height="${Math.round(height)}" viewBox="${bbox.left.toFixed(1)} ${bbox.top.toFixed(1)} ${width.toFixed(1)} ${height.toFixed(1)}">
<defs>${defs}</defs>
${layers}
</svg>`;
}
