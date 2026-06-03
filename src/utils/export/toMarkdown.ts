// Export Markdown — linéarise le board en un document texte structuré et
// lisible n'importe où (éditeur, GitHub). On regroupe les cartes par ZONE
// (membrane) puis on liste les LIENS (flèches) en fin de document.
//
// L'objectif « retenir le savoir » : ce format se ré-édite à la main et boucle
// avec le plugin glucose-notes.
import { Project } from "../../types";
import { buildScene, ExportScene, SceneCard, stripInlineMarkdown } from "./scene";

/** Titre d'affichage d'une carte : 1er titre `#`/`##`/`###`, sinon 1re ligne. */
export function cardTitle(text: string): string {
  const lines = (text || "").replace(/\r\n/g, "\n").split("\n");
  for (const l of lines) {
    const t = l.trim();
    if (!t) continue;
    const h = /^#{1,3}\s+(.*)$/.exec(t);
    if (h) return stripInlineMarkdown(h[1]).slice(0, 80);
    return stripInlineMarkdown(t).slice(0, 80);
  }
  return "(sans titre)";
}

/** Corps d'une carte sans son 1er titre (évite la double-titraille sous la zone). */
function cardBodyWithoutTitle(text: string): string {
  const lines = (text || "").replace(/\r\n/g, "\n").split("\n");
  let dropped = false;
  const out: string[] = [];
  for (const l of lines) {
    if (!dropped && l.trim()) {
      // on saute la 1re ligne non vide (devient le titre de section)
      dropped = true;
      if (/^#{1,3}\s+/.test(l.trim())) continue; // c'était un titre markdown → retiré
      // 1re ligne en texte simple : on la garde aussi dans le corps n'aurait pas
      // de sens (déjà dans le titre) → on la retire également.
      continue;
    }
    out.push(l);
  }
  return out.join("\n").trim();
}

const PREDICATE_TEXT: Record<string, string> = {
  est_precurseur: "est précurseur de",
  contredit: "contredit",
  herite_de: "hérite de",
  inspire: "inspire",
  depend_de: "dépend de",
  illustre: "illustre",
};

function cardInMembrane(c: SceneCard, m: { x: number; y: number; w: number; h: number }): boolean {
  const cx = c.x + c.w / 2, cy = c.y + c.h / 2;
  return cx >= m.x && cx <= m.x + m.w && cy >= m.y && cy <= m.y + m.h;
}

/** Ordre de lecture : par bandes horizontales (haut→bas), puis gauche→droite. */
function readingOrder<T extends { x: number; y: number }>(items: T[]): T[] {
  const ROW = 200;
  return [...items].sort((a, b) => {
    const ra = Math.round(a.y / ROW), rb = Math.round(b.y / ROW);
    if (ra !== rb) return ra - rb;
    return a.x - b.x;
  });
}

export function sceneToMarkdown(scene: ExportScene): string {
  const out: string[] = [];
  out.push(`# ${scene.projectName} — ${scene.boardName}`);
  out.push("");
  out.push(`> Export Markdown du board Glucose (${scene.cards.length} cartes, ${scene.arrows.length} liens). Mise en page spatiale aplatie en lecture linéaire.`);
  out.push("");

  // Index titre par id (pour la section liens)
  const titleById = new Map<string, string>();
  for (const c of scene.cards) titleById.set(c.id, cardTitle(c.text));

  // Regroupement par zone (membrane)
  const used = new Set<string>();
  const membranes = readingOrder(scene.membranes);
  for (const m of membranes) {
    const inside = readingOrder(scene.cards.filter((c) => !used.has(c.id) && cardInMembrane(c, m)));
    if (!inside.length) continue;
    const zoneTitle = m.text ? stripInlineMarkdown(m.text) : "Zone";
    out.push(`## ${zoneTitle}`);
    out.push("");
    for (const c of inside) {
      used.add(c.id);
      out.push(`### ${cardTitle(c.text)}`);
      out.push("");
      const body = cardBodyWithoutTitle(c.text);
      if (body) { out.push(body); out.push(""); }
    }
  }

  // Cartes hors zone
  const orphans = readingOrder(scene.cards.filter((c) => !used.has(c.id)));
  if (orphans.length) {
    if (membranes.length) { out.push("## Autres"); out.push(""); }
    for (const c of orphans) {
      out.push(`### ${cardTitle(c.text)}`);
      out.push("");
      const body = cardBodyWithoutTitle(c.text);
      if (body) { out.push(body); out.push(""); }
    }
  }

  // Notes (stickies)
  if (scene.stickies.length) {
    out.push("## Notes");
    out.push("");
    for (const s of readingOrder(scene.stickies)) {
      const op = s.operator ? `_(${s.operator})_ ` : "";
      out.push(`- ${op}${stripInlineMarkdown(s.text)}`);
    }
    out.push("");
  }

  // Liens (flèches)
  const links = scene.arrows.filter((a) => a.sourceId && a.targetId);
  if (links.length) {
    out.push("## Liens");
    out.push("");
    for (const a of links) {
      const src = titleById.get(a.sourceId!) ?? "?";
      const tgt = titleById.get(a.targetId!) ?? "?";
      const rel = a.predicate ? PREDICATE_TEXT[a.predicate] ?? a.predicate : (a.label ? `« ${a.label} »` : "→");
      let line = `- **${src}** ${rel} **${tgt}**`;
      if (a.longText) line += ` — ${stripInlineMarkdown(a.longText)}`;
      out.push(line);
    }
    out.push("");
  }

  return out.join("\n").replace(/\n{3,}/g, "\n\n").trimEnd() + "\n";
}

export function projectToMarkdown(project: Project): string {
  return sceneToMarkdown(buildScene(project, { includeImages: false }));
}
