#!/usr/bin/env node
// ─────────────────────────────────────────────────────────────────────────────
// glucose-mcp — Serveur MCP (Model Context Protocol) pour Glucose.
// ─────────────────────────────────────────────────────────────────────────────
//
// BUT : donner à Claude (Claude Code, Claude Desktop, ou tout client MCP) la
// capacité de LIRE et EXPLORER tes fichiers `.glucose` — pour t'aider à
// organiser tes projets — sans qu'il ait à comprendre le format binaire.
//
// Un `.glucose` est un document Automerge (CRDT) binaire. Ce serveur réutilise
// l'Automerge DÉJÀ installé dans ce repo (`@automerge/automerge`, v3) pour le
// décoder, puis en extrait un digest lisible (texte, flèches sémantiques,
// membranes, domaines). Il ne SORT rien sur le réseau : 100 % local.
//
// TRANSPORT : stdio, JSON-RPC 2.0 en messages délimités par des sauts de ligne
// (le transport MCP « stdio »). AUCUNE dépendance en dehors d'Automerge : le
// protocole est implémenté à la main (≈ handshake + tools/list + tools/call).
//
// ⚠️  RÈGLE D'OR stdio : stdout est RÉSERVÉ au protocole. Tout log de debug va
//     sur stderr, jamais sur stdout (sinon on corrompt le flux JSON-RPC).
//
// LECTURE **ET ÉCRITURE** : 6 outils lisent (list, read, search, analyze, lint,
// detect) et 5 écrivent (create, add_note, connect_notes, apply_layout,
// optimize_layout). Les outils d'écriture sauvegardent un `.bak` avant de
// toucher un fichier existant, et acceptent `outPath` pour travailler sur copie.
// Un `.glucose` v1 (JSON legacy) écrit est converti en v2 binaire, comme le fait
// l'app à son prochain save.
// ─────────────────────────────────────────────────────────────────────────────

import { readFileSync, statSync, readdirSync, writeFileSync, existsSync, copyFileSync } from "node:fs";
import { join, extname, basename, resolve } from "node:path";
import { homedir } from "node:os";
import { randomUUID } from "node:crypto";
import { next as A } from "@automerge/automerge";

const SERVER_NAME = "glucose";
const SERVER_VERSION = "0.1.0";
const PROTOCOL_VERSION = "2024-11-05";

// Racine par défaut des projets : le dossier Documents de l'utilisateur.
const DEFAULT_ROOT = join(homedir(), "Documents");
// Dossiers qu'on ne descend jamais (bruit / volumineux).
const SKIP_DIRS = new Set(["node_modules", ".git", "target", "dist", "build", ".cache"]);
const MAX_DEPTH = 6;

// ── Décodage d'un .glucose ──────────────────────────────────────────────────

/**
 * Décode les octets d'un `.glucose` : binaire Automerge (v2) OU JSON UTF-8 (v1
 * legacy). Même ordre de tentatives que l'app (src/utils/project.ts) — sans le
 * fallback JSON, les projets v1 sont rejetés alors qu'ils sont parfaitement
 * lisibles, et l'app les migre au prochain save.
 * @returns {{plain: object, doc: object|null, legacy: boolean}} doc=null si v1.
 */
function decodeGlucose(bytes, path) {
  const u8 = new Uint8Array(bytes);
  try {
    const doc = A.load(u8);
    const plain = typeof A.toJS === "function" ? A.toJS(doc) : JSON.parse(JSON.stringify(doc));
    if (plain && Array.isArray(plain.boards)) return { plain, doc, legacy: false };
  } catch { /* pas du binaire Automerge → on tente le JSON legacy */ }
  let plain;
  try {
    plain = JSON.parse(new TextDecoder().decode(u8));
  } catch (e) {
    throw new Error(`Fichier .glucose illisible (ni binaire Automerge, ni JSON) : ${path}\n${e.message}`);
  }
  if (!plain || !Array.isArray(plain.boards))
    throw new Error(`Fichier .glucose invalide (aucun tableau \`boards\`) : ${path}`);
  return { plain, doc: null, legacy: true };
}

/** Charge un `.glucose` → objet Project plain JS. Lève si format illisible. */
function loadProject(path) {
  return decodeGlucose(readFileSync(path), path).plain;
}

/** Compte les annotations d'un projet par type + images, sans tout extraire. */
function summarize(p) {
  let text = 0, sticky = 0, arrow = 0, membrane = 0, images = 0;
  for (const b of p.boards ?? []) {
    images += (b.images ?? []).length;
    for (const a of b.annotations ?? []) {
      if (a.type === "text") text++;
      else if (a.type === "sticky") sticky++;
      else if (a.type === "arrow") arrow++;
      else if (a.type === "membrane") membrane++;
    }
  }
  return { boards: (p.boards ?? []).length, text, sticky, arrow, membrane, images };
}

/** Ordre de lecture : haut→bas puis gauche→droite (comme on lit un canvas). */
function readingOrder(a, b) {
  const dy = (a.y ?? 0) - (b.y ?? 0);
  if (Math.abs(dy) > 40) return dy;
  return (a.x ?? 0) - (b.x ?? 0);
}

/** Étiquette courte d'un nœud (pour nommer les extrémités d'une flèche). */
function nodeLabel(a) {
  const t = (a.text ?? "").replace(/\s+/g, " ").trim();
  if (t) return t.slice(0, 50);
  if (a.type === "arrow") return "(flèche)";
  return `(${a.type})`;
}

/** Digest lisible complet d'un projet : texte, flèches sémantiques, zones. */
function digest(p, { includeText = true, includeIds = false } = {}) {
  const lines = [];
  lines.push(`# ${p.name ?? "(sans nom)"}`);
  const meta = [];
  if (p.version) meta.push(`format ${p.version}`);
  if (p.createdAt) meta.push(`créé ${new Date(p.createdAt).toISOString().slice(0, 10)}`);
  if (p.updatedAt) meta.push(`modifié ${new Date(p.updatedAt).toISOString().slice(0, 10)}`);
  if (meta.length) lines.push(`_${meta.join(" · ")}_`);
  if (p.collabUrl) lines.push(`_collab: oui_`);
  if ((p.domains ?? []).length) {
    lines.push(`\n**Domaines :** ${p.domains.map((d) => `${d.icon ?? ""}${d.name}`).join(", ")}`);
  }
  const s = summarize(p);
  lines.push(
    `\n**Contenu :** ${s.boards} board(s) · ${s.text} texte · ${s.sticky} sticky · ` +
      `${s.arrow} flèche(s) · ${s.membrane} membrane(s) · ${s.images} image(s)`
  );

  for (const b of p.boards ?? []) {
    const anns = b.annotations ?? [];
    // Index id → annotation pour résoudre les extrémités de flèches.
    const byId = new Map(anns.map((a) => [a.id, a]));
    lines.push(`\n## Board « ${b.name ?? b.id} »`);

    if (includeText) {
      const blocks = anns
        .filter((a) => (a.type === "text" || a.type === "sticky") && (a.text ?? "").trim())
        .sort(readingOrder);
      if (blocks.length) {
        lines.push(`\n### Notes (ordre de lecture)`);
        for (const a of blocks) {
          const tag = a.type === "sticky" ? "📌 " : "";
          const idTag = includeIds ? `\`[${a.id}]\` ` : "";
          lines.push(`\n${idTag}${tag}${a.text.trim()}`);
        }
      }
    }

    const arrows = anns.filter((a) => a.type === "arrow");
    if (arrows.length) {
      lines.push(`\n### Relations (flèches)`);
      for (const ar of arrows) {
        const src = ar.sourceId ? byId.get(ar.sourceId) : null;
        const dst = ar.targetId ? byId.get(ar.targetId) : null;
        const from = src ? nodeLabel(src) : ar.sourceTextSel || "?";
        const to = dst ? nodeLabel(dst) : ar.targetTextSel || "?";
        const rel = ar.predicate ? ` —${ar.predicate}→ ` : ar.text ? ` —${ar.text}→ ` : " → ";
        const idTag = includeIds ? ` \`[${ar.sourceId ?? "?"}→${ar.targetId ?? "?"}]\`` : "";
        lines.push(`- ${from}${rel}${to}${idTag}`);
      }
    }

    const membranes = anns.filter((a) => a.type === "membrane" && (a.text ?? "").trim());
    if (membranes.length) {
      lines.push(`\n### Zones (membranes)`);
      for (const m of membranes) lines.push(`- ${m.text.trim()}`);
    }
  }
  return lines.join("\n");
}

/** Concatène tout le texte lisible d'un projet (pour la recherche). */
function allText(p) {
  const out = [];
  for (const b of p.boards ?? []) {
    for (const a of b.annotations ?? []) {
      if ((a.type === "text" || a.type === "sticky") && a.text) out.push(a.text);
      if (a.type === "arrow" && a.longText) out.push(a.longText);
    }
  }
  return out.join("\n");
}

// ── Écriture (création / annotation) ────────────────────────────────────────
//
// ⚠️  Ces outils MODIFIENT le disque. Garde-fous : extension .glucose obligatoire,
//     pas d'écrasement sans `overwrite`, et une sauvegarde `<fichier>.bak` avant
//     toute réécriture. On produit un doc Automerge complet (A.save) — format que
//     l'app relit sans souci. NB : édite un projet FERMÉ dans Glucose (l'app ne
//     surveille pas le fichier ; rouvre-le après écriture pour voir le résultat).

// Reproduit `nanoid()` de l'app : UUID sans tirets, tronqué à 16 (ids compatibles).
const nid = () => randomUUID().replace(/-/g, "").slice(0, 16);

// Enlève le markdown de surface (l'app cherche le textSel dans le DOM rendu, sans
// les `**`, `###`, etc.). Sert à VALIDER qu'un `sourceTextSel` existe bien.
const stripMd = (s) => (s || "").replace(/[*#`_>~]/g, "");
const noteHasSel = (noteText, sel) =>
  stripMd(noteText).toLowerCase().includes((sel || "").trim().toLowerCase());

// Constantes de mise en page reprises de glucose-notes (layout éprouvé, chargé
// sans erreur par l'app) : colonnes de 380px, saut à la 3ᵉ section.
const COL_WIDTH = 380, COL_GAP = 60, Y_GAP = 28, LINE_H = 22, SECTIONS_PER_COL = 3;

function estimateLines(text, width) {
  const cpl = Math.max(Math.floor(width / 8), 20);
  return text.split("\n").reduce((n, ln) => n + Math.max(1, Math.ceil(ln.length / cpl)), 0);
}

/** Dispose une liste de notes en colonnes → annotations {id,type,x,y,text,width,fontSize}. */
function layoutNotes(notes) {
  const anns = [];
  let col = 0, y = 0;
  notes.forEach((n, i) => {
    if (i > 0 && i % SECTIONS_PER_COL === 0) { col++; y = 0; }
    const x = col * (COL_WIDTH + COL_GAP);
    const type = n.type === "sticky" ? "sticky" : "text";
    anns.push({
      id: nid(), type, x, y, text: n.text,
      width: COL_WIDTH, fontSize: type === "sticky" ? 13 : 14,
    });
    y += estimateLines(n.text, COL_WIDTH) * LINE_H + Y_GAP;
  });
  return anns;
}

/** Board vide (mêmes champs que `newBoard` de l'app). */
function newBoard(name) {
  const now = Date.now();
  return {
    id: nid(), name, images: [], annotations: [], panels: [], zones: [], folders: [],
    viewport: { x: 0, y: 0, scale: 1 }, createdAt: now, updatedAt: now,
  };
}

/** Construit un doc Automerge Project (même pattern init+change que l'app). */
function buildProjectDoc(name, annotations) {
  const now = Date.now();
  const board = newBoard("Board principal");
  board.annotations = annotations;
  const project = {
    version: "2.0.0", name, boards: [board], activeBoardId: board.id,
    presets: [], domains: [], createdAt: now, updatedAt: now,
  };
  let doc = A.init();
  doc = A.change(doc, "create via mcp", (d) => Object.assign(d, project));
  return doc;
}

/** Écrit un doc en `.glucose` (save complet). Sauvegarde `.bak` si le fichier existe. */
function writeDoc(path, doc) {
  if (existsSync(path)) {
    try { copyFileSync(path, path + ".bak"); } catch { /* backup best-effort */ }
  }
  writeFileSync(path, Buffer.from(A.save(doc)));
}

/** Centre approximatif d'une annotation (pour ancrer une flèche). */
function annCenter(a) {
  const w = a.width ?? COL_WIDTH;
  const h = a.height ?? 60;
  return { x: (a.x ?? 0) + w / 2, y: (a.y ?? 0) + h / 2 };
}

const PREDICATES = ["est_precurseur", "contredit", "herite_de", "inspire", "depend_de", "illustre"];

// ── Analyse d'architecture (STRUCTURE — géométrie + graphe, zéro vision) ─────
//
// On reconstruit « comment le projet est bâti » à partir de la DONNÉE, pas d'une
// image : quelle note tombe dans quelle zone (inclusion x/y), quels blocs sont
// spatialement regroupés, et la forme du graphe de flèches (chaîne / étoile /
// réseau). Indépendant de tout modèle vision. Plus complet qu'une capture : on
// voit les prédicats typés, l'appartenance aux zones, la hiérarchie — invisibles
// à l'œil sur un rendu.

/** Boîte englobante approximative d'une annotation (pour la géométrie). */
function annBox(a) {
  const w = a.width ?? COL_WIDTH;
  let h = a.height;
  if (typeof h !== "number") {
    h = a.type === "membrane" ? 200 : estimateLines(a.text ?? "", w) * LINE_H + 16;
  }
  const x = a.x ?? 0, y = a.y ?? 0;
  return { x, y, w, h, cx: x + w / 2, cy: y + h / 2 };
}

const pointInRect = (px, py, r) =>
  px >= r.x && px <= r.x + r.w && py >= r.y && py <= r.y + r.h;

/** Union-Find minimal (clustering spatial + composantes du graphe). */
function makeDSU(ids) {
  const parent = new Map(ids.map((id) => [id, id]));
  const find = (x) => {
    while (parent.get(x) !== x) { parent.set(x, parent.get(parent.get(x))); x = parent.get(x); }
    return x;
  };
  const union = (a, b) => { const ra = find(a), rb = find(b); if (ra !== rb) parent.set(ra, rb); };
  return { find, union };
}

/** Analyse structurelle d'UN board. Renvoie zones↔notes, clusters, graphe, forme. */
function analyzeBoard(board) {
  const anns = board.annotations ?? [];
  const notes = anns.filter((a) => a.type === "text" || a.type === "sticky");
  const membranes = anns.filter((a) => a.type === "membrane");
  const arrows = anns.filter((a) => a.type === "arrow");
  const byId = new Map(anns.map((a) => [a.id, a]));

  // 1) Inclusion : chaque note → la zone (membrane) la plus PETITE qui contient
  //    son centre (la plus spécifique en cas de zones imbriquées).
  const zoneMembers = new Map(membranes.map((m) => [m.id, []]));
  const noteZone = new Map();
  for (const n of notes) {
    const nb = annBox(n);
    let best = null, bestArea = Infinity;
    for (const m of membranes) {
      const mb = annBox(m);
      if (pointInRect(nb.cx, nb.cy, mb) && mb.w * mb.h < bestArea) { bestArea = mb.w * mb.h; best = m; }
    }
    if (best) { zoneMembers.get(best.id).push(n); noteZone.set(n.id, best.id); }
  }
  const looseNotes = notes.filter((n) => !noteZone.has(n.id));

  // 2) Regroupements spatiaux parmi les notes SANS zone (proximité des centres).
  const CLUSTER_DIST = 480;
  const dsu = makeDSU(looseNotes.map((n) => n.id));
  for (let i = 0; i < looseNotes.length; i++) {
    for (let j = i + 1; j < looseNotes.length; j++) {
      const a = annBox(looseNotes[i]), b = annBox(looseNotes[j]);
      if (Math.hypot(a.cx - b.cx, a.cy - b.cy) < CLUSTER_DIST) dsu.union(looseNotes[i].id, looseNotes[j].id);
    }
  }
  const clusterMap = new Map();
  for (const n of looseNotes) {
    const r = dsu.find(n.id);
    (clusterMap.get(r) ?? clusterMap.set(r, []).get(r)).push(n);
  }
  const clusters = [...clusterMap.values()].filter((c) => c.length >= 2);

  // 3) Graphe : flèches qui relient deux nœuds identifiés (par id).
  const deg = new Map(notes.map((n) => [n.id, { in: 0, out: 0 }]));
  const edges = [];
  for (const ar of arrows) {
    if (ar.sourceId && ar.targetId && deg.has(ar.sourceId) && deg.has(ar.targetId)) {
      deg.get(ar.sourceId).out++;
      deg.get(ar.targetId).in++;
      edges.push([ar.sourceId, ar.targetId]);
    }
  }
  const hubs = [...deg.entries()]
    .map(([id, d]) => ({ id, total: d.in + d.out, ...d }))
    .filter((x) => x.total >= 3)
    .sort((a, b) => b.total - a.total);
  const roots = notes.filter((n) => deg.get(n.id).out > 0 && deg.get(n.id).in === 0);
  const leaves = notes.filter((n) => deg.get(n.id).in > 0 && deg.get(n.id).out === 0);
  const isolated = notes.filter((n) => deg.get(n.id).in === 0 && deg.get(n.id).out === 0);

  // Composantes connexes (non orientées) parmi les nœuds reliés.
  const linked = notes.filter((n) => deg.get(n.id).in + deg.get(n.id).out > 0);
  const gdsu = makeDSU(linked.map((n) => n.id));
  for (const [s, t] of edges) gdsu.union(s, t);
  const comps = new Set(linked.map((n) => gdsu.find(n.id))).size;

  // Verdict de forme.
  let shape;
  const chainish = linked.filter((n) => deg.get(n.id).out <= 1 && deg.get(n.id).in <= 1).length;
  if (edges.length === 0) shape = notes.length ? "tas non structuré (aucune relation)" : "board vide";
  else if (hubs.length && hubs[0].total >= 4 && hubs[0].total >= edges.length * 0.5)
    shape = `étoile — carte centrée sur « ${nodeLabel(byId.get(hubs[0].id))} »`;
  else if (linked.length && chainish / linked.length >= 0.8)
    shape = "chaîne linéaire — logique de parcours / cours";
  else shape = "réseau / graphe mixte";

  return {
    counts: { notes: notes.length, arrows: arrows.length, membranes: membranes.length, images: (board.images ?? []).length },
    zoneMembers, membranes, looseNotes, clusters, hubs, roots, leaves, isolated, comps, byId, deg, shape,
  };
}

// ── Lint géométrique (QA visuel SANS vision) ────────────────────────────────
// On calcule les défauts qu'on VERRAIT à l'écran : flèches qui traversent une
// note, flèches qui se croisent, notes qui se chevauchent, flèches trop longues.
// Déterministe, aucun modèle, aucun coût token.

/** Deux segments [a,b] et [c,d] se croisent-ils vraiment ? */
function segCross(a, b, c, d) {
  const o = (p, q, r) => Math.sign((q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x));
  return o(a, b, c) !== o(a, b, d) && o(c, d, a) !== o(c, d, b);
}

/** Le segment [p1,p2] entre-t-il dans le rectangle r ? */
function segHitsRect(p1, p2, r) {
  const inside = (p) => p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h;
  if (inside(p1) || inside(p2)) return true;
  const c1 = { x: r.x, y: r.y }, c2 = { x: r.x + r.w, y: r.y };
  const c3 = { x: r.x + r.w, y: r.y + r.h }, c4 = { x: r.x, y: r.y + r.h };
  return segCross(p1, p2, c1, c2) || segCross(p1, p2, c2, c3) ||
    segCross(p1, p2, c3, c4) || segCross(p1, p2, c4, c1);
}

/** Endpoints d'une flèche : centres des notes reliées (comme l'app), sinon x/y. */
function arrowSeg(ar, boxById) {
  const p1 = ar.sourceId && boxById.has(ar.sourceId)
    ? { x: boxById.get(ar.sourceId).cx, y: boxById.get(ar.sourceId).cy }
    : { x: ar.x ?? 0, y: ar.y ?? 0 };
  const p2 = ar.targetId && boxById.has(ar.targetId)
    ? { x: boxById.get(ar.targetId).cx, y: boxById.get(ar.targetId).cy }
    : { x: ar.x2 ?? 0, y: ar.y2 ?? 0 };
  return [p1, p2];
}

// ── Détection d'INTENTION d'organisation (déterministe, sans vision) ────────
// Au lieu d'imposer un layout, on DÉTECTE la logique déjà présente et on la
// respecte : chronologie (dates dans les notes/zones, axe temporel), hub (nœud
// carrefour), thématique (zones), chaîne (parcours), ou non-structuré.

const yearOf = (s) => { const m = (s || "").match(/(\d{4})/); return m ? +m[1] : null; };

function detectMode(board) {
  const anns = board.annotations ?? [];
  const notes = anns.filter((a) => a.type === "text" || a.type === "sticky");
  const membranes = anns.filter((a) => a.type === "membrane");
  const arrows = anns.filter((a) => a.type === "arrow");

  const notesTA = notes.filter((n) => n.temporalAnchor && typeof n.temporalAnchor.start === "number").length;
  const datedZones = membranes.filter((m) => yearOf(m.text) !== null);
  const chronoAxis = arrows.some((a) => /chronolog|timeline|frise|le temps|ann[ée]e/i.test(a.text || ""));

  const deg = new Map(notes.map((n) => [n.id, 0]));
  let edgeCount = 0;
  for (const ar of arrows)
    if (ar.sourceId && ar.targetId && deg.has(ar.sourceId) && deg.has(ar.targetId)) {
      deg.set(ar.sourceId, deg.get(ar.sourceId) + 1);
      deg.set(ar.targetId, deg.get(ar.targetId) + 1);
      edgeCount++;
    }
  const ds = [...deg.values()].sort((a, b) => b - a);
  const maxDeg = ds[0] || 0, second = ds[1] || 0;

  const ev = [];
  if (notesTA >= notes.length * 0.4 || datedZones.length >= 2 || chronoAxis) {
    if (datedZones.length) ev.push(`${datedZones.length} zones datées (${datedZones.map((z) => yearOf(z.text)).sort().join(", ")})`);
    if (notesTA) ev.push(`${notesTA} notes avec date`);
    if (chronoAxis) ev.push("axe chronologique présent");
    return { mode: "chronological", evidence: ev };
  }
  if (membranes.length >= 3) return { mode: "thematic", evidence: [`${membranes.length} zones thématiques, pas de dates`] };
  if (maxDeg >= 5 && maxDeg >= 2 * second) return { mode: "hub", evidence: [`hub dominant degré ${maxDeg} (2ᵉ : ${second})`] };
  if (edgeCount && edgeCount >= notes.length - 2 && maxDeg <= 3) return { mode: "linear", evidence: ["graphe en chaîne (parcours)"] };
  if (membranes.length >= 1) return { mode: "thematic", evidence: [`${membranes.length} zone(s)`] };
  return { mode: "unstructured", evidence: ["ni dates, ni hub dominant, ni zones"] };
}

/** Reconstruit le parent (folder) de chaque board : childBoardId → {parentId, folderName}. */
function boardParents(project) {
  const parent = new Map();
  for (const b of project.boards ?? []) {
    for (const f of b.folders ?? []) {
      parent.set(f.childBoardId, {
        parent: b.id, name: f.name,
        mirror: !!f.mirrorSource, root: f.mirrorSource?.rootPath,
      });
    }
  }
  return parent;
}

// ── Parcours du disque ──────────────────────────────────────────────────────

/** Liste récursive des `.glucose` sous `root` (profondeur limitée). */
function findGlucoseFiles(root, depth = 0, acc = []) {
  let entries;
  try {
    entries = readdirSync(root, { withFileTypes: true });
  } catch {
    return acc; // dossier illisible → ignoré
  }
  for (const e of entries) {
    const full = join(root, e.name);
    if (e.isDirectory()) {
      if (depth < MAX_DEPTH && !SKIP_DIRS.has(e.name) && !e.name.startsWith(".")) {
        findGlucoseFiles(full, depth + 1, acc);
      }
    } else if (e.isFile() && extname(e.name).toLowerCase() === ".glucose") {
      acc.push(full);
    }
  }
  return acc;
}

// ── Implémentation des outils ───────────────────────────────────────────────

const TOOLS = [
  {
    name: "list_glucose_projects",
    description:
      "Liste tous les fichiers .glucose sous un dossier (défaut : ~/Documents). " +
      "Pour chacun : chemin, taille, date de modif, et un aperçu décodé (nom du " +
      "projet, nombre de boards/notes/flèches). Point d'entrée pour organiser les projets.",
    inputSchema: {
      type: "object",
      properties: {
        root: { type: "string", description: "Dossier racine à scanner (absolu). Défaut : ~/Documents." },
      },
    },
  },
  {
    name: "read_glucose",
    description:
      "Lit un fichier .glucose et renvoie un digest lisible en Markdown : nom, " +
      "domaines, toutes les notes dans l'ordre de lecture, les relations (flèches " +
      "sémantiques source→prédicat→cible) et les zones. À utiliser pour comprendre " +
      "ou résumer un projet Glucose.",
    inputSchema: {
      type: "object",
      properties: {
        path: { type: "string", description: "Chemin absolu du .glucose à lire." },
        includeText: {
          type: "boolean",
          description: "Inclure le texte intégral des notes (défaut : true). false = structure seule.",
        },
        includeIds: {
          type: "boolean",
          description: "Préfixer chaque note de son id `[id]` (à passer avant connect_notes). Défaut: false.",
        },
      },
      required: ["path"],
    },
  },
  {
    name: "search_glucose",
    description:
      "Cherche une expression (insensible à la casse) dans le texte de TOUS les " +
      ".glucose sous un dossier. Renvoie les projets qui matchent avec un extrait. " +
      "Utile pour retrouver « où ai-je parlé de X ? » à travers tous les projets.",
    inputSchema: {
      type: "object",
      properties: {
        query: { type: "string", description: "Texte à chercher." },
        root: { type: "string", description: "Dossier racine à scanner (absolu). Défaut : ~/Documents." },
      },
      required: ["query"],
    },
  },
  {
    name: "create_glucose_project",
    description:
      "CRÉE un nouveau fichier .glucose à partir d'un titre et d'une liste de notes. " +
      "Les notes sont disposées automatiquement en colonnes lisibles. À ouvrir " +
      "ensuite dans Glucose. Refuse d'écraser un fichier existant sauf overwrite=true.",
    inputSchema: {
      type: "object",
      properties: {
        path: { type: "string", description: "Chemin absolu du .glucose à créer." },
        name: { type: "string", description: "Nom du projet." },
        notes: {
          type: "array",
          description: "Les blocs de note (Markdown/LaTeX supporté).",
          items: {
            type: "object",
            properties: {
              text: { type: "string" },
              type: { type: "string", enum: ["text", "sticky"], description: "Défaut: text." },
            },
            required: ["text"],
          },
        },
        overwrite: { type: "boolean", description: "Autoriser l'écrasement (défaut: false)." },
      },
      required: ["path", "name", "notes"],
    },
  },
  {
    name: "add_note",
    description:
      "Ajoute une note (text ou sticky) à un .glucose EXISTANT. Par défaut sur le board " +
      "le plus structuré — le même que celui qu'analysent read/analyze/lint. " +
      "Position auto sous le contenu existant si x/y non fournis. Fait une sauvegarde .bak.",
    inputSchema: {
      type: "object",
      properties: {
        path: { type: "string", description: "Chemin absolu du .glucose." },
        text: { type: "string", description: "Contenu de la note (Markdown/LaTeX)." },
        type: { type: "string", enum: ["text", "sticky"], description: "Défaut: text." },
        x: { type: "number", description: "Position X (optionnel)." },
        y: { type: "number", description: "Position Y (optionnel)." },
        boardId: { type: "string", description: "Id du board ciblé — désignateur fiable (read_glucose includeIds=true)." },
        board: { type: "string", description: "Nom du board ciblé. Lève si plusieurs boards portent ce nom : utilise boardId." },
      },
      required: ["path", "text"],
    },
  },
  {
    name: "connect_notes",
    description:
      "Relie deux notes par une flèche sémantique dans un .glucose existant. Désigne " +
      "les extrémités par sourceId/targetId (voir read_glucose avec includeIds=true) OU " +
      "par sourceText/targetText (sous-chaîne du texte de la note). Prédicat optionnel.",
    inputSchema: {
      type: "object",
      properties: {
        path: { type: "string", description: "Chemin absolu du .glucose." },
        sourceId: { type: "string", description: "Id de la note de départ." },
        targetId: { type: "string", description: "Id de la note d'arrivée." },
        sourceText: { type: "string", description: "Sous-chaîne identifiant la note de départ." },
        targetText: { type: "string", description: "Sous-chaîne identifiant la note d'arrivée." },
        predicate: {
          type: "string",
          enum: PREDICATES,
          description: "Relation sémantique (optionnel).",
        },
        label: { type: "string", description: "Étiquette libre sur la flèche (optionnel)." },
        sourceSel: { type: "string", description: "Phrase EXACTE à SOULIGNER côté source (la flèche pointe ce texte précis, pas le bloc entier). Doit exister dans la note. Mots contigus." },
        targetSel: { type: "string", description: "Phrase EXACTE à souligner côté cible. Idem." },
        boardId: { type: "string", description: "Id du board ciblé — désignateur fiable (read_glucose includeIds=true)." },
        board: { type: "string", description: "Nom du board ciblé. Lève si plusieurs boards portent ce nom : utilise boardId." },
      },
      required: ["path"],
    },
  },
  {
    name: "analyze_architecture",
    description:
      "Analyse la STRUCTURE d'un projet .glucose SANS vision : quelles notes tombent " +
      "dans quelle zone (inclusion géométrique), regroupements spatiaux hors zones, " +
      "forme du graphe de flèches (chaîne/étoile/réseau), hubs, racines/feuilles, et " +
      "hiérarchie des boards. C'est la base pour comprendre comment le projet est bâti " +
      "et proposer une meilleure organisation. Indépendant de tout modèle vision.",
    inputSchema: {
      type: "object",
      properties: {
        path: { type: "string", description: "Chemin absolu du .glucose à analyser." },
        board: { type: "string", description: "Nom d'un board précis à détailler (défaut : board actif)." },
      },
      required: ["path"],
    },
  },
  {
    name: "apply_layout",
    description:
      "APPLIQUE une réorganisation à un .glucose en UN SEUL coup, atomiquement : retire " +
      "des annotations (removeIds), déplace des notes (moves), crée des zones (zones) et " +
      "ajoute des flèches (arrows). Écrit de préférence dans outPath (une COPIE) pour " +
      "préserver l'original. C'est l'outil qui matérialise une proposition d'organisation.",
    inputSchema: {
      type: "object",
      properties: {
        path: { type: "string", description: "Source .glucose (lu, jamais modifié si outPath diffère)." },
        outPath: { type: "string", description: "Destination (copie recommandée). Défaut : écrase path (avec .bak)." },
        overwrite: { type: "boolean", description: "Autoriser l'écrasement de outPath s'il existe déjà." },
        boardId: { type: "string", description: "Id du board ciblé — désignateur fiable, à préférer (les noms de boards ne sont pas uniques)." },
        board: { type: "string", description: "Nom du board ciblé. Lève si ambigu. Défaut : le board le plus structuré." },
        removeIds: { type: "array", items: { type: "string" }, description: "Ids d'annotations à supprimer (ex. anciennes zones)." },
        moves: {
          type: "array", description: "Notes à déplacer.",
          items: { type: "object", properties: { id: { type: "string" }, x: { type: "number" }, y: { type: "number" } }, required: ["id"] },
        },
        zones: {
          type: "array", description: "Zones (membranes) à créer.",
          items: { type: "object", properties: { x: { type: "number" }, y: { type: "number" }, width: { type: "number" }, height: { type: "number" }, text: { type: "string" }, color: { type: "string" } }, required: ["x", "y", "width", "height"] },
        },
        arrows: {
          type: "array", description: "Flèches à ajouter (par id de note).",
          items: { type: "object", properties: { sourceId: { type: "string" }, targetId: { type: "string" }, predicate: { type: "string", enum: PREDICATES }, label: { type: "string" }, sourceTextSel: { type: "string", description: "Phrase exacte à souligner côté source." }, targetTextSel: { type: "string", description: "Phrase exacte à souligner côté cible." } }, required: ["sourceId", "targetId"] },
        },
        patches: {
          type: "array", description: "Met à jour des propriétés d'annotations EXISTANTES (par id) : arrowType, strokeWidth, color, bgColor, text, fontSize…",
          items: { type: "object", properties: { id: { type: "string" }, arrowType: { type: "string", enum: ["straight", "curved"] }, strokeWidth: { type: "number" }, color: { type: "string" }, bgColor: { type: "string" }, text: { type: "string" }, fontSize: { type: "number" } }, required: ["id"] },
        },
      },
      required: ["path"],
    },
  },
  {
    name: "detect_organization",
    description:
      "Détecte la LOGIQUE d'organisation déjà présente dans un projet — chronologie / " +
      "thématique / hub / chaîne / non-structuré — sans vision, par la structure (dates " +
      "dans les notes ou zones, axe temporel, forme du graphe). Sert à RESPECTER " +
      "l'intention de l'auteur au lieu d'imposer un layout arbitraire.",
    inputSchema: {
      type: "object",
      properties: {
        path: { type: "string", description: "Chemin absolu du .glucose." },
        board: { type: "string", description: "Board ciblé (défaut : le plus structuré)." },
      },
      required: ["path"],
    },
  },
  {
    name: "lint_layout",
    description:
      "QA VISUEL sans vision : calcule les défauts qu'on VERRAIT à l'écran — flèches " +
      "qui traversent une note, flèches qui se croisent, notes qui se chevauchent, " +
      "flèches trop longues. Déterministe, 0 token, 0 modèle. À lancer après une " +
      "réorganisation pour contrôler le rendu et savoir quelles flèches nettoyer.",
    inputSchema: {
      type: "object",
      properties: {
        path: { type: "string", description: "Chemin absolu du .glucose à contrôler." },
        board: { type: "string", description: "Board ciblé (défaut : le plus rempli)." },
      },
      required: ["path"],
    },
  },
  {
    name: "optimize_layout",
    description:
      "Calcule une CARTE 2D lisible par simulation de forces (répulsion de boîtes + " +
      "arêtes-ressorts + attraction intra-cluster + gravité) qui MINIMISE l'énergie que " +
      "mesure lint_layout : arêtes courtes, aucun chevauchement, notes regroupées par " +
      "zone en territoires 2D. L'anti-parpaing : pour l'humain, pas la colonne. Applique " +
      "le résultat dans outPath (copie recommandée). Déterministe (graine).",
    inputSchema: {
      type: "object",
      properties: {
        path: { type: "string", description: "Source .glucose." },
        outPath: { type: "string", description: "Destination (copie recommandée). Défaut : écrase path (.bak)." },
        overwrite: { type: "boolean", description: "Autoriser l'écrasement de outPath." },
        board: { type: "string", description: "Board ciblé (défaut : le plus riche en flèches)." },
        mode: { type: "string", enum: ["auto", "chronological", "thematic", "hub", "linear"], description: "auto (défaut) = détecte et respecte l'intention. chronological = frise. thematic = territoires 2D. hub = radial. linear = parcours." },
        iterations: { type: "number", description: "Itérations de la simulation (défaut 500)." },
        seed: { type: "number", description: "Graine pour un layout reproductible (défaut 7)." },
      },
      required: ["path"],
    },
  },
];

function toolListProjects(args) {
  const root = args.root ? resolve(args.root) : DEFAULT_ROOT;
  const files = findGlucoseFiles(root);
  if (!files.length) return `Aucun .glucose trouvé sous ${root}.`;
  const rows = [];
  for (const f of files) {
    let info;
    try {
      const st = statSync(f);
      const kb = (st.size / 1024).toFixed(1);
      const mod = st.mtime.toISOString().slice(0, 10);
      try {
        const p = loadProject(f);
        const s = summarize(p);
        info =
          `- **${basename(f)}** — « ${p.name ?? "?"} » · ${kb} Ko · modifié ${mod}\n` +
          `  ${s.boards} board · ${s.text} texte · ${s.arrow} flèche · ${s.membrane} zone · ${s.images} img\n` +
          `  \`${f}\``;
      } catch (e) {
        info = `- **${basename(f)}** — ⚠️ format illisible (${e.message.split("\n")[0]}) · ${kb} Ko\n  \`${f}\``;
      }
    } catch (e) {
      info = `- ${f} — erreur: ${e.message}`;
    }
    rows.push(info);
  }
  return `# ${files.length} projet(s) Glucose sous ${root}\n\n${rows.join("\n")}`;
}

function toolReadGlucose(args) {
  if (!args.path) throw new Error("`path` requis.");
  const path = resolve(args.path);
  if (extname(path).toLowerCase() !== ".glucose")
    throw new Error("Le fichier n'a pas l'extension .glucose.");
  const p = loadProject(path);
  return digest(p, { includeText: args.includeText !== false, includeIds: args.includeIds === true });
}

// ── Outils d'écriture ───────────────────────────────────────────────────────

/** Valide un chemin cible d'écriture : extension .glucose obligatoire. */
function requireGlucosePath(pathArg) {
  if (!pathArg) throw new Error("`path` requis.");
  const path = resolve(pathArg);
  if (extname(path).toLowerCase() !== ".glucose")
    throw new Error("Le chemin doit se terminer par .glucose.");
  return path;
}

function toolCreate(args) {
  const path = requireGlucosePath(args.path);
  if (!args.name) throw new Error("`name` requis.");
  if (!Array.isArray(args.notes) || args.notes.length === 0)
    throw new Error("`notes` doit être une liste non vide.");
  if (existsSync(path) && args.overwrite !== true)
    throw new Error(`Le fichier existe déjà : ${path}. Passe overwrite=true pour l'écraser.`);
  const notes = args.notes.map((n) => (typeof n === "string" ? { text: n } : n))
    .filter((n) => n && typeof n.text === "string" && n.text.trim());
  if (!notes.length) throw new Error("Aucune note valide (chaque note doit avoir un `text`).");
  const anns = layoutNotes(notes);
  const doc = buildProjectDoc(args.name, anns);
  writeDoc(path, doc);
  return `✅ Projet « ${args.name} » créé : ${anns.length} note(s).\n\`${path}\`\n\nOuvre-le dans Glucose pour le voir.`;
}

/**
 * Charge un doc Automerge ÉDITABLE depuis un .glucose existant.
 * v1 legacy (JSON) → doc Automerge neuf, comme l'app qui migre au prochain save.
 * Écrire dans un v1 le convertit donc en v2 (l'original reste dans le .bak).
 */
function loadDoc(path) {
  const { doc, plain } = decodeGlucose(readFileSync(path), path);
  return doc ?? A.from(plain);
}

function toolAddNote(args) {
  const path = requireGlucosePath(args.path);
  if (!existsSync(path)) throw new Error(`Fichier introuvable : ${path}`);
  if (!args.text || !args.text.trim()) throw new Error("`text` requis.");
  const type = args.type === "sticky" ? "sticky" : "text";
  let doc = loadDoc(path);
  const plain = typeof A.toJS === "function" ? A.toJS(doc) : JSON.parse(JSON.stringify(doc));
  const board = resolveBoard(plain, args);
  // Position auto : sous l'annotation la plus basse (ou 0,0 si board vide).
  let x = args.x, y = args.y;
  if (typeof x !== "number" || typeof y !== "number") {
    let maxY = 0, atX = 0;
    for (const a of board.annotations ?? []) {
      const bottom = (a.y ?? 0) + (a.height ?? 60);
      if (bottom >= maxY) { maxY = bottom; atX = a.x ?? 0; }
    }
    x = typeof x === "number" ? x : atX;
    y = typeof y === "number" ? y : maxY + Y_GAP;
  }
  const note = {
    id: nid(), type, x, y, text: args.text,
    width: COL_WIDTH, fontSize: type === "sticky" ? 13 : 14,
  };
  const boardId = board.id;
  doc = A.change(doc, `add ${type} via mcp`, (d) => {
    const b = d.boards.find((bb) => bb.id === boardId);
    b.annotations.push(note);
    b.updatedAt = Date.now();
    d.updatedAt = Date.now();
  });
  writeDoc(path, doc);
  return `✅ Note ${type} ajoutée (id \`${note.id}\`) au board « ${board.name} » en (${Math.round(x)}, ${Math.round(y)}).\nRouvre le projet dans Glucose pour la voir.`;
}

function toolConnect(args) {
  const path = requireGlucosePath(args.path);
  if (!existsSync(path)) throw new Error(`Fichier introuvable : ${path}`);
  if (args.predicate && !PREDICATES.includes(args.predicate))
    throw new Error(`Prédicat inconnu. Choix : ${PREDICATES.join(", ")}.`);
  let doc = loadDoc(path);
  const plain = typeof A.toJS === "function" ? A.toJS(doc) : JSON.parse(JSON.stringify(doc));
  const board = resolveBoard(plain, args);
  const anns = board.annotations ?? [];

  // Résout une extrémité par id, sinon par sous-chaîne de texte (1 seul match).
  const resolveEnd = (id, text, which) => {
    if (id) {
      const a = anns.find((x) => x.id === id);
      if (!a) throw new Error(`${which}: aucune note avec l'id ${id}.`);
      return a;
    }
    if (text) {
      const needle = text.toLowerCase();
      const hits = anns.filter((x) => (x.text ?? "").toLowerCase().includes(needle) && x.type !== "arrow");
      if (hits.length === 0) throw new Error(`${which}: aucune note ne contient « ${text} ».`);
      if (hits.length > 1)
        throw new Error(`${which}: « ${text} » est ambigu (${hits.length} notes). Précise avec un id (read_glucose includeIds=true).`);
      return hits[0];
    }
    throw new Error(`${which}: fournis un id ou un texte.`);
  };
  const src = resolveEnd(args.sourceId, args.sourceText, "source");
  const dst = resolveEnd(args.targetId, args.targetText, "cible");
  if (src.id === dst.id) throw new Error("La source et la cible sont la même note.");

  // Ancrage sur texte (« soulignement ») : la flèche pointe une phrase précise.
  const warns = [];
  if (args.sourceSel && !noteHasSel(src.text ?? "", args.sourceSel))
    warns.push(`sourceSel « ${args.sourceSel} » introuvable dans la note source`);
  if (args.targetSel && !noteHasSel(dst.text ?? "", args.targetSel))
    warns.push(`targetSel « ${args.targetSel} » introuvable dans la note cible`);

  const c1 = annCenter(src), c2 = annCenter(dst);
  const arrow = {
    id: nid(), type: "arrow",
    x: c1.x, y: c1.y, x2: c2.x, y2: c2.y,
    sourceId: src.id, targetId: dst.id, strokeWidth: 2,
    ...(args.predicate ? { predicate: args.predicate } : {}),
    ...(args.label ? { text: args.label } : {}),
    ...(args.sourceSel ? { sourceTextSel: args.sourceSel } : {}),
    ...(args.targetSel ? { targetTextSel: args.targetSel } : {}),
  };
  const boardId = board.id;
  doc = A.change(doc, "connect notes via mcp", (d) => {
    const b = d.boards.find((bb) => bb.id === boardId);
    b.annotations.push(arrow);
    b.updatedAt = Date.now();
    d.updatedAt = Date.now();
  });
  writeDoc(path, doc);
  const rel = args.predicate ? ` —${args.predicate}→ ` : " → ";
  const sel = (args.sourceSel || args.targetSel)
    ? `\n✍️ Ancrée sur : « ${args.sourceSel ?? "?"} » → « ${args.targetSel ?? "?"} »` : "";
  const warn = warns.length ? `\n⚠️ ${warns.join(" ; ")}` : "";
  return `✅ Flèche ajoutée : « ${nodeLabel(src)} »${rel}« ${nodeLabel(dst)} » (id \`${arrow.id}\`).${sel}${warn}\nRouvre le projet dans Glucose.`;
}

function toolSearch(args) {
  if (!args.query) throw new Error("`query` requis.");
  const root = args.root ? resolve(args.root) : DEFAULT_ROOT;
  const needle = args.query.toLowerCase();
  const files = findGlucoseFiles(root);
  const hits = [];
  for (const f of files) {
    let p;
    try {
      p = loadProject(f);
    } catch {
      continue; // illisible → ignoré silencieusement dans la recherche
    }
    const text = allText(p);
    const idx = text.toLowerCase().indexOf(needle);
    if (idx >= 0) {
      const start = Math.max(0, idx - 60);
      const snippet = text.slice(start, idx + needle.length + 60).replace(/\s+/g, " ").trim();
      hits.push(`- **« ${p.name ?? "?"} »** (${basename(f)})\n  …${snippet}…\n  \`${f}\``);
    }
  }
  if (!hits.length) return `Aucun projet ne contient « ${args.query} » sous ${root}.`;
  return `# ${hits.length} projet(s) contiennent « ${args.query} »\n\n${hits.join("\n")}`;
}

/** Formate une liste de notes en labels courts, plafonnée. */
function fmtNotes(arr, cap = 8) {
  if (!arr.length) return "—";
  const labels = arr.slice(0, cap).map((n) => `« ${nodeLabel(n).slice(0, 40)} »`);
  const extra = arr.length > cap ? ` +${arr.length - cap} autre(s)` : "";
  return labels.join(", ") + extra;
}

/** Rend l'analyse d'un board en Markdown. */
function renderBoardArch(board) {
  const a = analyzeBoard(board);
  const L = [];
  L.push(`## Architecture — Board « ${board.name ?? board.id} »`);
  L.push(`**Forme :** ${a.shape}`);
  L.push(
    `**Contenu :** ${a.counts.notes} notes · ${a.counts.arrows} flèches · ` +
      `${a.counts.membranes} zones · ${a.counts.images} images · ${a.comps} composante(s) de graphe`
  );

  if (a.membranes.length) {
    L.push(`\n### Zones et leur contenu (inclusion géométrique)`);
    for (const m of a.membranes) {
      const members = a.zoneMembers.get(m.id) ?? [];
      const label = (m.text ?? "").trim() || "(zone sans titre)";
      L.push(`- **${label}** (${members.length}) : ${fmtNotes(members)}`);
    }
    L.push(`- _Hors zone :_ ${a.looseNotes.length} note(s)`);
  }

  if (a.clusters.length) {
    L.push(`\n### Regroupements spatiaux (hors zones dessinées)`);
    a.clusters
      .sort((x, y) => y.length - x.length)
      .forEach((c, i) => L.push(`- Groupe ${i + 1} (${c.length}) : ${fmtNotes(c, 6)}`));
  }

  if (a.counts.arrows) {
    L.push(`\n### Graphe des flèches`);
    if (a.hubs.length)
      L.push(`- **Hubs :** ${a.hubs.slice(0, 5).map((h) => `« ${nodeLabel(a.byId.get(h.id)).slice(0, 36)} » (deg ${h.total})`).join(", ")}`);
    L.push(`- **Racines (départs) :** ${fmtNotes(a.roots, 6)}`);
    L.push(`- **Feuilles (fins) :** ${fmtNotes(a.leaves, 6)}`);
    if (a.isolated.length) L.push(`- **Isolées (aucune flèche) :** ${a.isolated.length} — ${fmtNotes(a.isolated, 6)}`);
  } else if (a.counts.notes) {
    L.push(`\n_Aucune flèche : les idées ne sont pas encore reliées entre elles._`);
  }
  return L.join("\n");
}

function toolAnalyze(args) {
  const path = requireGlucosePath(args.path);
  if (!existsSync(path)) throw new Error(`Fichier introuvable : ${path}`);
  const p = loadProject(path);
  const boards = p.boards ?? [];
  if (!boards.length) return "Le projet n'a aucun board.";

  // Board à détailler : celui demandé, sinon le plus STRUCTURÉ (flèches/zones
  // priment sur le simple nombre de notes — un dossier-miroir a 250 notes mais
  // aucune structure). Le board « actif » est un curseur d'UI, pas le cœur.
  const parents0 = boardParents(p);
  const structScore = (b) => {
    const anns = b.annotations ?? [];
    if (parents0.get(b.id)?.mirror) return -1; // les miroirs de disque ne sont pas le sujet
    return anns.filter((a) => a.type === "arrow").length * 1000 +
      anns.filter((a) => a.type === "membrane").length * 100 + anns.length;
  };
  let main;
  if (args.board) {
    main = boards.find((b) => (b.name ?? "").toLowerCase() === args.board.toLowerCase());
    if (!main) throw new Error(`Board « ${args.board} » introuvable.`);
  } else {
    main = [...boards].sort((x, y) => structScore(y) - structScore(x))[0];
    if (structScore(main) <= 0) main = boards.find((b) => b.id === p.activeBoardId) ?? boards[0];
  }

  const out = [`# Architecture — « ${p.name ?? "(sans nom)"} »`];
  if ((p.domains ?? []).length)
    out.push(`**Domaines :** ${p.domains.map((d) => `${d.icon ?? ""}${d.name}`).join(", ")}`);
  out.push("");
  out.push(renderBoardArch(main));

  // Hiérarchie / autres boards non vides.
  const others = boards.filter((b) => b.id !== main.id);
  const nonEmpty = others.filter((b) => (b.annotations ?? []).length + (b.images ?? []).length > 0);
  const parents = boardParents(p);
  if (boards.length > 1) {
    out.push(`\n## Hiérarchie des boards`);
    out.push(`Le projet a **${boards.length} boards** ; ${nonEmpty.length + (( (main.annotations??[]).length+(main.images??[]).length)>0?1:0)} avec du contenu, ${boards.length - nonEmpty.length - 1} vide(s) (dossiers).`);
    if (nonEmpty.length) {
      out.push(`\n**Autres boards avec du contenu :**`);
      for (const b of nonEmpty.slice(0, 12)) {
        const ab = analyzeBoard(b);
        const par = parents.get(b.id);
        const loc = par ? ` _(dans dossier « ${par.name} »)_` : "";
        // Un board miroir de disque n'est pas un « tas non structuré » : c'est un
        // listing de fichiers. On le dit clairement au lieu de le juger.
        const shape = par?.mirror
          ? `🗂 miroir de dossier disque${par.root ? ` (${par.root})` : ""}`
          : ab.shape;
        out.push(`- **${b.name ?? b.id}**${loc} — ${ab.counts.notes} notes, ${ab.counts.arrows} flèches, ${ab.counts.images} img · ${shape}`);
      }
      if (nonEmpty.length > 12) out.push(`- … +${nonEmpty.length - 12} autre(s)`);
    }
  }
  return out.join("\n");
}

function toolApplyLayout(args) {
  const path = requireGlucosePath(args.path);
  if (!existsSync(path)) throw new Error(`Fichier introuvable : ${path}`);
  const outPath = args.outPath ? requireGlucosePath(args.outPath) : path;
  if (args.outPath && existsSync(outPath) && args.overwrite !== true)
    throw new Error(`La sortie existe déjà : ${outPath}. Passe overwrite=true.`);

  let doc = loadDoc(path);
  const plain = typeof A.toJS === "function" ? A.toJS(doc) : JSON.parse(JSON.stringify(doc));
  const boardId = resolveBoard(plain, args).id;

  const removeIds = new Set(Array.isArray(args.removeIds) ? args.removeIds : []);
  const moves = Array.isArray(args.moves) ? args.moves : [];
  const zones = Array.isArray(args.zones) ? args.zones : [];
  const arrows = Array.isArray(args.arrows) ? args.arrows : [];
  const patches = Array.isArray(args.patches) ? args.patches : [];
  let removed = 0, moved = 0, addedZones = 0, addedArrows = 0, patched = 0;
  const missing = [];

  doc = A.change(doc, "apply organization via mcp", (d) => {
    const b = d.boards.find((bb) => bb.id === boardId);
    // 1) Suppressions (ex. anciennes zones) — splice en place.
    if (removeIds.size) {
      for (let i = b.annotations.length - 1; i >= 0; i--) {
        if (removeIds.has(b.annotations[i].id)) { b.annotations.splice(i, 1); removed++; }
      }
    }
    // 2) Déplacements.
    const byId = new Map(b.annotations.map((a) => [a.id, a]));
    for (const m of moves) {
      const a = byId.get(m.id);
      if (!a) { missing.push(m.id); continue; }
      if (typeof m.x === "number") a.x = m.x;
      if (typeof m.y === "number") a.y = m.y;
      if (typeof m.x2 === "number") a.x2 = m.x2; // extrémité d'une flèche (ex. axe)
      if (typeof m.y2 === "number") a.y2 = m.y2;
      moved++;
    }
    // 3) Zones (membranes).
    for (const z of zones) {
      b.annotations.push({
        id: nid(), type: "membrane",
        x: z.x ?? 0, y: z.y ?? 0, width: z.width ?? COL_WIDTH, height: z.height ?? 200,
        ...(z.text ? { text: z.text } : {}),
        ...(z.color ? { color: z.color } : {}),
      });
      addedZones++;
    }
    // 4) Flèches (après déplacements → ancrage sur les positions finales).
    const byId2 = new Map(b.annotations.map((a) => [a.id, a]));
    for (const ar of arrows) {
      const s = byId2.get(ar.sourceId), t = byId2.get(ar.targetId);
      if (!s || !t) { missing.push(`${ar.sourceId}→${ar.targetId}`); continue; }
      const cs = annCenter(s), ct = annCenter(t);
      b.annotations.push({
        id: nid(), type: "arrow",
        x: cs.x, y: cs.y, x2: ct.x, y2: ct.y,
        sourceId: s.id, targetId: t.id, strokeWidth: 2,
        ...(ar.predicate && PREDICATES.includes(ar.predicate) ? { predicate: ar.predicate } : {}),
        ...(ar.label ? { text: ar.label } : {}),
        ...(ar.sourceTextSel ? { sourceTextSel: ar.sourceTextSel } : {}),
        ...(ar.targetTextSel ? { targetTextSel: ar.targetTextSel } : {}),
      });
      addedArrows++;
    }
    // 5) Patches : met à jour des propriétés d'annotations existantes (style de flèche…).
    const PATCHABLE = new Set(["arrowType", "strokeWidth", "color", "bgColor", "text", "fontSize", "predicate", "arrowBidirectional"]);
    const byIdP = new Map(b.annotations.map((a) => [a.id, a]));
    for (const pt of patches) {
      const a = byIdP.get(pt.id);
      if (!a) { missing.push(pt.id); continue; }
      for (const k of Object.keys(pt)) if (k !== "id" && PATCHABLE.has(k)) a[k] = pt[k];
      patched++;
    }
    b.updatedAt = Date.now();
    d.updatedAt = Date.now();
  });

  writeDoc(outPath, doc);
  let msg = `✅ Organisation appliquée → ${removed} retirée(s), ${moved} déplacée(s), ${addedZones} zone(s), ${addedArrows} flèche(s)${patched ? `, ${patched} patchée(s)` : ""}.\n\`${outPath}\``;
  if (missing.length) msg += `\n⚠️ ${missing.length} id(s) introuvable(s) : ${missing.slice(0, 5).join(", ")}${missing.length > 5 ? "…" : ""}`;
  return msg;
}

/**
 * Choisit le board le plus STRUCTURÉ (le plus de flèches, puis d'annotations).
 * On ne suit PAS `activeBoardId` : dans les vrais projets il pointe souvent sur
 * un board vide (dernier board consulté), ce qui ferait analyser — ou pire,
 * écrire dans — le vide.
 * Les noms de boards ne sont PAS uniques (dossiers-miroirs) → un nom ambigu lève
 * plutôt que de désigner silencieusement le mauvais board.
 */
function pickStructuredBoard(p, name) {
  const boards = p.boards ?? [];
  if (name) {
    const hits = boards.filter((x) => (x.name ?? "").toLowerCase() === name.toLowerCase());
    if (!hits.length) throw new Error(`Board « ${name} » introuvable.`);
    if (hits.length > 1)
      throw new Error(
        `Board « ${name} » ambigu : ${hits.length} boards portent ce nom (ids : ${hits.map((b) => b.id).join(", ")}). ` +
        `Passe boardId pour lever le doute.`);
    return hits[0];
  }
  const arrowsOf = (b) => (b.annotations ?? []).filter((a) => a.type === "arrow").length;
  return [...boards].sort((a, b) =>
    arrowsOf(b) - arrowsOf(a) || (b.annotations?.length ?? 0) - (a.annotations?.length ?? 0))[0];
}

/**
 * Résolveur de board UNIQUE, partagé lecture ET écriture — c'est ce qui garantit
 * qu'on écrit dans le board qu'on vient d'analyser. `boardId` est le désignateur
 * fiable (les noms sont ambigus, `activeBoardId` est périmé).
 */
function resolveBoard(p, args = {}) {
  const boards = p.boards ?? [];
  if (!boards.length) throw new Error("Le projet n'a aucun board.");
  if (args.boardId) {
    const b = boards.find((x) => x.id === args.boardId);
    if (!b) throw new Error(`Board id « ${args.boardId} » introuvable.`);
    return b;
  }
  return pickStructuredBoard(p, args.board);
}

const MODE_LABEL = {
  chronological: "⏳ Chronologique — une frise temporelle",
  thematic: "🗂 Thématique — des territoires par sujet",
  hub: "🕸 Hub — une carte radiale autour d'un nœud central",
  linear: "➡️ Chaîne — un parcours / cours linéaire",
  unstructured: "🌫 Non structuré — un tas à organiser",
};
const MODE_LAYOUT = {
  chronological: "frise : zones datées ordonnées gauche→droite par année",
  thematic: "territoires 2D (force + clusters)",
  hub: "radial : hub au centre, satellites autour",
  linear: "colonne / serpentin de lecture",
  unstructured: "clusters par proximité, ou proposer une structure",
};

function toolDetect(args) {
  const path = requireGlucosePath(args.path);
  if (!existsSync(path)) throw new Error(`Fichier introuvable : ${path}`);
  const p = loadProject(path);
  const board = pickStructuredBoard(p, args.board);
  if (!board) return "Le projet n'a aucun board.";
  const { mode, evidence } = detectMode(board);
  return [
    `# Intention détectée — « ${p.name} » · board « ${board.name} »`,
    `\n**${MODE_LABEL[mode]}**`,
    `\nIndices : ${evidence.join(" · ") || "—"}`,
    `\nLayout qui la respecterait : _${MODE_LAYOUT[mode]}_`,
  ].join("\n");
}

function toolLint(args) {
  const path = requireGlucosePath(args.path);
  if (!existsSync(path)) throw new Error(`Fichier introuvable : ${path}`);
  const p = loadProject(path);
  const boards = p.boards ?? [];
  let board;
  if (args.board) {
    board = boards.find((b) => (b.name ?? "").toLowerCase() === args.board.toLowerCase());
    if (!board) throw new Error(`Board « ${args.board} » introuvable.`);
  } else {
    // Même critère qu'optimize_layout : le board le plus STRUCTURÉ (par les
    // flèches), pas le plus peuplé — sinon un dossier-miroir (250 fichiers, 0
    // flèche) rafle la mise et on lint le mauvais board.
    const arrowsOf = (b) => (b.annotations ?? []).filter((a) => a.type === "arrow").length;
    board = [...boards].sort((a, b) =>
      arrowsOf(b) - arrowsOf(a) || (b.annotations?.length ?? 0) - (a.annotations?.length ?? 0))[0];
  }
  if (!board) return "Le projet n'a aucun board.";
  const anns = board.annotations ?? [];
  const notes = anns.filter((a) => a.type === "text" || a.type === "sticky");
  const arrows = anns.filter((a) => a.type === "arrow");
  const boxById = new Map(notes.map((n) => [n.id, annBox(n)]));
  const LONG = (COL_WIDTH + COL_GAP) * 1.6;

  // Notes qui se chevauchent (>20px dans les deux axes).
  const overlaps = [];
  for (let i = 0; i < notes.length; i++)
    for (let j = i + 1; j < notes.length; j++) {
      const a = annBox(notes[i]), b = annBox(notes[j]);
      const ox = Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x);
      const oy = Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y);
      if (ox > 20 && oy > 20) overlaps.push([notes[i], notes[j]]);
    }

  // Flèches : collisions avec des notes tierces + longueur.
  const segs = arrows.map((ar) => ({ ar, seg: arrowSeg(ar, boxById) }));
  const collisions = [], longs = [];
  for (const { ar, seg } of segs) {
    const [p1, p2] = seg;
    const dist = Math.hypot(p2.x - p1.x, p2.y - p1.y);
    if (dist > LONG) longs.push({ ar, dist });
    for (const n of notes) {
      if (n.id === ar.sourceId || n.id === ar.targetId) continue;
      if (segHitsRect(p1, p2, annBox(n))) collisions.push({ ar, note: n });
    }
  }

  // Croisements de flèches.
  let crossings = 0;
  for (let i = 0; i < segs.length; i++)
    for (let j = i + 1; j < segs.length; j++)
      if (segCross(segs[i].seg[0], segs[i].seg[1], segs[j].seg[0], segs[j].seg[1])) crossings++;

  const score = overlaps.length * 2 + collisions.length * 2 + longs.length + crossings;
  const L = [`# Lint visuel — « ${p.name} » · board « ${board.name} »`];
  L.push(`${notes.length} notes · ${arrows.length} flèches`);
  L.push(`\n**Score de désordre : ${score}** (0 = impeccable)`);
  L.push(`- **Notes qui se chevauchent :** ${overlaps.length}` +
    (overlaps.length ? ` — ex. « ${nodeLabel(overlaps[0][0]).slice(0, 28)} » ∩ « ${nodeLabel(overlaps[0][1]).slice(0, 28)} »` : ""));
  L.push(`- **Flèches qui traversent une note :** ${collisions.length}` +
    (collisions.length ? ` — ex. sur « ${nodeLabel(collisions[0].note).slice(0, 32)} »` : ""));
  L.push(`- **Flèches trop longues (> ${Math.round(LONG)}px) :** ${longs.length}`);
  L.push(`- **Croisements de flèches :** ${crossings}`);
  if (longs.length) {
    const top = longs.sort((a, b) => b.dist - a.dist).slice(0, 6);
    L.push(`\n**Flèches les plus longues (candidates à retirer ou re-router) :**`);
    for (const { ar, dist } of top) {
      const lbl = ar.text ? `« ${ar.text} »` : ar.predicate || "(sans label)";
      L.push(`- ${lbl} — ${Math.round(dist)}px  \`[${ar.sourceId ?? "?"}→${ar.targetId ?? "?"}]\``);
    }
  }
  L.push(`\n${score === 0 ? "✅ Rien à signaler." : score < 8 ? "🟡 Défauts mineurs." : "🔴 Rendu encombré — à nettoyer."}`);
  return L.join("\n");
}

// PRNG déterministe (mulberry32) → layout reproductible à graine fixe.
function mulberry32(a) {
  return function () {
    a |= 0; a = (a + 0x6D2B79F5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Layout CHRONOLOGIQUE : respecte la frise — zones datées en colonnes ordonnées
 *  gauche→droite par année, zones thématiques (non datées) sous la frise, axe
 *  temporel repositionné pour couvrir la période. NE piétine PAS la chronologie. */
function chronoLayout(args, board, path) {
  const anns = board.annotations ?? [];
  const notes = anns.filter((a) => a.type === "text" || a.type === "sticky");
  const membranes = anns.filter((a) => a.type === "membrane");
  const axis = anns.find((a) => a.type === "arrow" && /chronolog|timeline|frise|le temps|ann[ée]e/i.test(a.text || ""));

  // Chaque note → plus petite membrane qui la contient.
  const zoneOf = new Map();
  for (const n of notes) {
    const nb = annBox(n); let best = null, area = Infinity;
    for (const m of membranes) { const mb = annBox(m); if (pointInRect(nb.cx, nb.cy, mb) && mb.w * mb.h < area) { area = mb.w * mb.h; best = m; } }
    zoneOf.set(n.id, best ? best.id : null);
  }
  const notesOfZone = (mid) => notes.filter((n) => zoneOf.get(n.id) === mid);
  const noteH = (a) => Math.max(120, estimateLines(a.text || "", a.width ?? COL_WIDTH) * 28 + 100);
  const COLW = COL_WIDTH, GAP = 90, ROWGAP = 150;

  const dated = membranes.filter((m) => yearOf(m.text) !== null).sort((a, b) => yearOf(a.text) - yearOf(b.text));
  const undated = membranes.filter((m) => yearOf(m.text) === null);
  const moves = [], zones = [];
  let timelineBottom = 0, timelineRight = 0;

  // Rangée 1 — zones DATÉES en colonnes, gauche→droite par année.
  dated.forEach((m, c) => {
    const x = c * (COLW + GAP); let y = 0;
    for (const n of notesOfZone(m.id)) { moves.push({ id: n.id, x, y }); y += noteH(n) + 40; }
    zones.push({ x: x - 30, y: -70, width: COLW + 60, height: (y || 120) + 100, text: m.text, color: "#e9e9e9" });
    timelineBottom = Math.max(timelineBottom, y); timelineRight = x + COLW;
  });

  // Axe temporel repositionné au-dessus, couvrant toute la frise.
  if (axis) moves.push({ id: axis.id, x: -40, y: -120, x2: timelineRight + 40, y2: -120 });

  // Rangée 2 — zones THÉMATIQUES (non datées) sous la frise.
  const row2y = timelineBottom + ROWGAP;
  undated.forEach((m, c) => {
    const x = c * (COLW + GAP); let y = row2y;
    for (const n of notesOfZone(m.id)) { moves.push({ id: n.id, x, y }); y += noteH(n) + 40; }
    zones.push({ x: x - 30, y: row2y - 70, width: COLW + 60, height: (y - row2y || 120) + 100, text: m.text, color: "#efefef" });
  });

  // Labels d'axe (« ← 2000 », « 2012 → ») → aux EXTRÉMITÉS de l'axe, pas en « divers ».
  const isAxisLabel = (t) => { const s = (t || "").trim(); return s.length <= 15 && (/[←→]/.test(s) || (/\d{4}/.test(s) && s.replace(/[\s~←→]/g, "").length <= 5)); };
  const allLoose = notes.filter((n) => zoneOf.get(n.id) === null);
  const axisLabels = allLoose.filter((n) => isAxisLabel(n.text));
  const loose = allLoose.filter((n) => !isAxisLabel(n.text));
  const years = dated.map((m) => yearOf(m.text));
  const minYr = Math.min(...years), maxYr = Math.max(...years);
  for (const n of axisLabels) {
    const left = /←/.test(n.text || "") || (yearOf(n.text) != null && yearOf(n.text) <= minYr && !/→/.test(n.text || ""));
    moves.push({ id: n.id, x: left ? -280 : timelineRight + 60, y: -150 });
  }

  // Vraies notes hors zone → colonne « divers » au bout de la rangée 2.
  if (loose.length) {
    const x = undated.length * (COLW + GAP); let y = row2y;
    for (const n of loose) { moves.push({ id: n.id, x, y }); y += noteH(n) + 40; }
  }

  // Flèches TRANSVERSES (entre deux zones datées différentes = liens à travers le
  // temps) → secondaires : courbes + fines, pour ne pas concurrencer la frise.
  const datedIds = new Set(dated.map((m) => m.id));
  const patches = [];
  for (const ar of anns.filter((a) => a.type === "arrow")) {
    if (!ar.sourceId || !ar.targetId) continue;
    const zs = zoneOf.get(ar.sourceId), zt = zoneOf.get(ar.targetId);
    const isPartagent = /partagent/i.test(ar.text || "");
    const crossesTime = zs !== zt && (datedIds.has(zs) || datedIds.has(zt));
    if (isPartagent || crossesTime)
      patches.push({ id: ar.id, arrowType: "curved", strokeWidth: 1 });
  }

  const removeIds = membranes.map((m) => m.id);
  const res = toolApplyLayout({ path, outPath: args.outPath, overwrite: args.overwrite, boardId: board.id, removeIds, moves, zones, arrows: [], patches });
  return `${res}\n\n(layout ⏳ CHRONOLOGIQUE : ${dated.length} zones datées ${dated.map((m) => yearOf(m.text)).join(" → ")}, ${undated.length} thématiques sous la frise${axis ? ", axe repositionné" : ""}${axisLabels.length ? `, ${axisLabels.length} labels d'axe recasés` : ""}${patches.length ? `, ${patches.length} flèches transverses adoucies` : ""})`;
}

/** Layout LINÉAIRE : respecte un parcours/chaîne — notes dans l'ordre du fil,
 *  en colonnes remplies par hauteur (moins « mur » qu'une colonne unique). */
function linearLayout(args, board, path) {
  const anns = board.annotations ?? [];
  const notes = anns.filter((a) => a.type === "text" || a.type === "sticky");
  const arrows = anns.filter((a) => a.type === "arrow");
  const byId = new Map(notes.map((n) => [n.id, n]));
  const idset = new Set(notes.map((n) => n.id));
  const nextOf = new Map(); const indeg = new Map(notes.map((n) => [n.id, 0]));
  for (const ar of arrows)
    if (ar.sourceId && ar.targetId && idset.has(ar.sourceId) && idset.has(ar.targetId)) {
      if (!nextOf.has(ar.sourceId)) nextOf.set(ar.sourceId, []);
      nextOf.get(ar.sourceId).push(ar.targetId);
      indeg.set(ar.targetId, (indeg.get(ar.targetId) || 0) + 1);
    }
  const order = [], seen = new Set();
  const dfs = (id) => { if (seen.has(id)) return; seen.add(id); order.push(id); for (const t of (nextOf.get(id) || [])) dfs(t); };
  for (const n of notes) if ((indeg.get(n.id) || 0) === 0) dfs(n.id);
  for (const n of notes) if (!seen.has(n.id)) dfs(n.id);

  const noteH = (a) => Math.max(110, estimateLines(a.text || "", a.width ?? COL_WIDTH) * 24 + 80);
  const CAP = 2600, COLW = COL_WIDTH, GAP = 90;
  const moves = []; let col = 0, y = 0;
  for (const id of order) {
    const h = noteH(byId.get(id));
    if (y > 0 && y + h > CAP) { col++; y = 0; }
    moves.push({ id, x: col * (COLW + GAP), y }); y += h + 40;
  }
  const removeIds = anns.filter((a) => a.type === "membrane").map((m) => m.id);
  const res = toolApplyLayout({ path, outPath: args.outPath, overwrite: args.overwrite, boardId: board.id, removeIds, moves, zones: [], arrows: [] });
  return `${res}\n\n(layout ➡️ LINÉAIRE : ${order.length} notes suivant le fil, ${col + 1} colonne(s))`;
}

/** Layout HUB : respecte une carte radiale — le nœud carrefour au centre, ses
 *  voisins directs en anneau, le reste en périphérie, séparation anti-collision. */
function hubLayout(args, board, path) {
  const anns = board.annotations ?? [];
  const notes = anns.filter((a) => a.type === "text" || a.type === "sticky");
  const arrows = anns.filter((a) => a.type === "arrow");
  if (!notes.length) throw new Error("Aucune note.");
  const deg = new Map(notes.map((n) => [n.id, 0]));
  for (const ar of arrows)
    if (ar.sourceId && ar.targetId && deg.has(ar.sourceId) && deg.has(ar.targetId)) {
      deg.set(ar.sourceId, deg.get(ar.sourceId) + 1); deg.set(ar.targetId, deg.get(ar.targetId) + 1);
    }
  let hub = notes[0];
  for (const n of notes) if ((deg.get(n.id) || 0) > (deg.get(hub.id) || 0)) hub = n;
  const neigh = new Set();
  for (const ar of arrows) {
    if (ar.sourceId === hub.id && deg.has(ar.targetId)) neigh.add(ar.targetId);
    if (ar.targetId === hub.id && deg.has(ar.sourceId)) neigh.add(ar.sourceId);
  }
  const nodes = notes.map((n) => { const b = annBox(n); return { id: n.id, w: b.w, h: b.h, cx: 0, cy: 0 }; });
  const idx = new Map(nodes.map((n, i) => [n.id, i]));
  const place = (id, x, y) => { const nn = nodes[idx.get(id)]; nn.cx = x; nn.cy = y; };
  place(hub.id, 0, 0);
  const ring1 = notes.filter((n) => neigh.has(n.id));
  const rest = notes.filter((n) => n.id !== hub.id && !neigh.has(n.id));
  const ringPlace = (arr, R, ph) => arr.forEach((n, i) => { const a = ph + (i / Math.max(1, arr.length)) * 2 * Math.PI; place(n.id, Math.cos(a) * R, Math.sin(a) * R); });
  ringPlace(ring1, 900, 0);
  ringPlace(rest, 1750, 0.35);

  const MARGIN = 44;
  for (let pass = 0; pass < 140; pass++) {
    let moved = false;
    for (let i = 0; i < nodes.length; i++)
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i], b = nodes[j];
        const dx = b.cx - a.cx, dy = b.cy - a.cy;
        const mx = (a.w + b.w) / 2 + MARGIN, my = (a.h + b.h) / 2 + MARGIN;
        const ox = mx - Math.abs(dx), oy = my - Math.abs(dy);
        if (ox > 0 && oy > 0) {
          moved = true;
          const hubA = a.id === hub.id, hubB = b.id === hub.id;
          if (ox < oy) { const s = ox * (dx < 0 ? -1 : 1); if (hubA) b.cx += s; else if (hubB) a.cx -= s; else { a.cx -= s / 2; b.cx += s / 2; } }
          else { const s = oy * (dy < 0 ? -1 : 1); if (hubA) b.cy += s; else if (hubB) a.cy -= s; else { a.cy -= s / 2; b.cy += s / 2; } }
        }
      }
    if (!moved) break;
  }
  let minx = Infinity, miny = Infinity;
  for (const n of nodes) { minx = Math.min(minx, n.cx - n.w / 2); miny = Math.min(miny, n.cy - n.h / 2); }
  const offx = 120 - minx, offy = 120 - miny;
  const moves = nodes.map((n) => ({ id: n.id, x: Math.round(n.cx - n.w / 2 + offx), y: Math.round(n.cy - n.h / 2 + offy) }));
  const removeIds = anns.filter((a) => a.type === "membrane").map((m) => m.id);
  const res = toolApplyLayout({ path, outPath: args.outPath, overwrite: args.overwrite, boardId: board.id, removeIds, moves, zones: [], arrows: [] });
  return `${res}\n\n(layout 🕸 HUB : « ${nodeLabel(hub).slice(0, 36)} » au centre, ${ring1.length} voisins en anneau, ${rest.length} en périphérie)`;
}

/** Carte 2D par simulation de forces. Minimise l'énergie que lint_layout mesure :
 *  boîtes qui se repoussent (anti-chevauchement), arêtes-ressorts (arêtes courtes),
 *  attraction vers le centroïde de cluster (territoires), gravité (garde le tout groupé). */
function toolOptimizeLayout(args) {
  const path = requireGlucosePath(args.path);
  if (!existsSync(path)) throw new Error(`Fichier introuvable : ${path}`);
  const p = loadProject(path);
  const boards = p.boards ?? [];
  let board;
  if (args.board) {
    board = boards.find((b) => (b.name ?? "").toLowerCase() === args.board.toLowerCase());
    if (!board) throw new Error(`Board « ${args.board} » introuvable.`);
  } else {
    board = [...boards].sort((a, b) =>
      (b.annotations?.filter((x) => x.type === "arrow").length ?? 0) -
      (a.annotations?.filter((x) => x.type === "arrow").length ?? 0))[0];
  }
  if (!board) throw new Error("Le projet n'a aucun board.");

  // INTENTION : par défaut on DÉTECTE la logique et on la respecte. `mode` force.
  const mode = (args.mode && args.mode !== "auto") ? args.mode : detectMode(board).mode;
  if (mode === "chronological") return chronoLayout(args, board, path);
  if (mode === "linear") return linearLayout(args, board, path);
  if (mode === "hub") return hubLayout(args, board, path);
  // thematic / unstructured → carte 2D par forces (ci-dessous)

  const anns = board.annotations ?? [];
  const noteAnns = anns.filter((a) => a.type === "text" || a.type === "sticky");
  const membranes = anns.filter((a) => a.type === "membrane");
  const arrowAnns = anns.filter((a) => a.type === "arrow");
  if (!noteAnns.length) throw new Error("Aucune note à disposer.");

  // Cluster de chaque note = plus petite membrane contenant son centre.
  const clusterOf = new Map(), clusterLabel = new Map();
  for (const n of noteAnns) {
    const nb = annBox(n);
    let best = null, bestArea = Infinity;
    for (const m of membranes) {
      const mb = annBox(m);
      if (pointInRect(nb.cx, nb.cy, mb) && mb.w * mb.h < bestArea) { bestArea = mb.w * mb.h; best = m; }
    }
    const key = best ? best.id : "__loose";
    clusterOf.set(n.id, key);
    if (best && !clusterLabel.has(key)) clusterLabel.set(key, (best.text ?? "").trim() || "(zone)");
  }

  const nodes = noteAnns.map((n) => { const b = annBox(n); return { id: n.id, w: b.w, h: b.h, cx: b.cx, cy: b.cy, cluster: clusterOf.get(n.id) }; });
  const idx = new Map(nodes.map((n, i) => [n.id, i]));
  const edges = [];
  for (const ar of arrowAnns)
    if (ar.sourceId && ar.targetId && idx.has(ar.sourceId) && idx.has(ar.targetId))
      edges.push([idx.get(ar.sourceId), idx.get(ar.targetId)]);

  // Init : chaque cluster sur une grille + jitter déterministe.
  const clusterKeys = [...new Set(nodes.map((n) => n.cluster))];
  const cols = Math.max(1, Math.ceil(Math.sqrt(clusterKeys.length)));
  const SPACING = 1500;
  const rnd = mulberry32(args.seed ?? 7);
  const cinit = new Map();
  clusterKeys.forEach((k, i) => cinit.set(k, { x: (i % cols) * SPACING, y: Math.floor(i / cols) * SPACING }));
  for (const n of nodes) { const c = cinit.get(n.cluster); n.cx = c.x + (rnd() - 0.5) * 400; n.cy = c.y + (rnd() - 0.5) * 400; }

  const iters = Math.max(50, Math.min(args.iterations ?? 500, 3000));
  const MARGIN = 40, KREP = 120000, KSPRING = 0.03, KCLUST = 0.04, KGRAV = 0.003;
  const centroids = () => {
    const acc = new Map();
    for (const n of nodes) { const a = acc.get(n.cluster) ?? { x: 0, y: 0, c: 0 }; a.x += n.cx; a.y += n.cy; a.c++; acc.set(n.cluster, a); }
    for (const a of acc.values()) { a.x /= a.c; a.y /= a.c; }
    return acc;
  };
  let temp = 500;
  for (let it = 0; it < iters; it++) {
    const fx = new Array(nodes.length).fill(0), fy = new Array(nodes.length).fill(0);
    // Répulsion / anti-chevauchement (AABB) entre toutes les paires.
    for (let i = 0; i < nodes.length; i++)
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i], b = nodes[j];
        const dx = b.cx - a.cx, dy = b.cy - a.cy;
        const mx = (a.w + b.w) / 2 + MARGIN, my = (a.h + b.h) / 2 + MARGIN;
        const ox = mx - Math.abs(dx), oy = my - Math.abs(dy);
        if (ox > 0 && oy > 0) { // boîtes qui se chevauchent → séparer sur l'axe de moindre pénétration
          if (ox < oy) { const s = ox * 0.5 * (dx < 0 ? -1 : 1); fx[i] -= s; fx[j] += s; }
          else { const s = oy * 0.5 * (dy < 0 ? -1 : 1); fy[i] -= s; fy[j] += s; }
        } else { // répulsion longue portée pour étaler
          const d2 = dx * dx + dy * dy + 1, d = Math.sqrt(d2), f = KREP / d2;
          fx[i] -= f * dx / d; fy[i] -= f * dy / d; fx[j] += f * dx / d; fy[j] += f * dy / d;
        }
      }
    // Arêtes-ressorts : rapprocher les notes reliées jusqu'à « adjacentes ».
    for (const [i, j] of edges) {
      const a = nodes[i], b = nodes[j];
      const dx = b.cx - a.cx, dy = b.cy - a.cy, d = Math.hypot(dx, dy) + 0.01;
      const rest = (a.w + b.w) / 2 + MARGIN + 20, f = KSPRING * (d - rest);
      fx[i] += f * dx / d; fy[i] += f * dy / d; fx[j] -= f * dx / d; fy[j] -= f * dy / d;
    }
    // Attraction vers le cluster + gravité globale.
    const cent = centroids();
    for (let i = 0; i < nodes.length; i++) {
      const c = cent.get(nodes[i].cluster);
      if (c) { fx[i] += (c.x - nodes[i].cx) * KCLUST; fy[i] += (c.y - nodes[i].cy) * KCLUST; }
      fx[i] -= nodes[i].cx * KGRAV; fy[i] -= nodes[i].cy * KGRAV;
    }
    // Intégration avec plafond de déplacement (refroidissement).
    for (let i = 0; i < nodes.length; i++) {
      const disp = Math.hypot(fx[i], fy[i]) + 0.001, cap = Math.min(disp, temp);
      nodes[i].cx += fx[i] / disp * cap; nodes[i].cy += fy[i] / disp * cap;
    }
    temp *= 0.99;
  }
  // Passe finale de séparation pure → garantit ~0 chevauchement.
  for (let pass = 0; pass < 60; pass++) {
    let moved = false;
    for (let i = 0; i < nodes.length; i++)
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i], b = nodes[j];
        const dx = b.cx - a.cx, dy = b.cy - a.cy;
        const mx = (a.w + b.w) / 2 + MARGIN, my = (a.h + b.h) / 2 + MARGIN;
        const ox = mx - Math.abs(dx), oy = my - Math.abs(dy);
        if (ox > 0 && oy > 0) {
          moved = true;
          if (ox < oy) { const s = ox * 0.5 * (dx < 0 ? -1 : 1); a.cx -= s; b.cx += s; }
          else { const s = oy * 0.5 * (dy < 0 ? -1 : 1); a.cy -= s; b.cy += s; }
        }
      }
    if (!moved) break;
  }

  // Passe finale bis : séparer les TERRITOIRES entiers → les membranes (boîtes
  // englobantes) ne se chevauchent plus. On déplace chaque cluster EN BLOC.
  const clusterBox = () => {
    const m = new Map();
    for (const n of nodes) {
      if (n.cluster === "__loose") continue;
      const b = m.get(n.cluster) ?? { minx: Infinity, miny: Infinity, maxx: -Infinity, maxy: -Infinity };
      b.minx = Math.min(b.minx, n.cx - n.w / 2); b.miny = Math.min(b.miny, n.cy - n.h / 2);
      b.maxx = Math.max(b.maxx, n.cx + n.w / 2); b.maxy = Math.max(b.maxy, n.cy + n.h / 2);
      m.set(n.cluster, b);
    }
    return m;
  };
  const CMARGIN = 90;
  for (let pass = 0; pass < 160; pass++) {
    const bb = clusterBox();
    const keys = [...bb.keys()];
    let moved = false;
    for (let i = 0; i < keys.length; i++)
      for (let j = i + 1; j < keys.length; j++) {
        const A = bb.get(keys[i]), B = bb.get(keys[j]);
        const dx = (B.minx + B.maxx) / 2 - (A.minx + A.maxx) / 2;
        const dy = (B.miny + B.maxy) / 2 - (A.miny + A.maxy) / 2;
        const mx = ((A.maxx - A.minx) + (B.maxx - B.minx)) / 2 + CMARGIN;
        const my = ((A.maxy - A.miny) + (B.maxy - B.miny)) / 2 + CMARGIN;
        const ox = mx - Math.abs(dx), oy = my - Math.abs(dy);
        if (ox > 0 && oy > 0) {
          moved = true;
          let sx = 0, sy = 0;
          if (ox < oy) sx = ox * 0.5 * (dx < 0 ? -1 : 1); else sy = oy * 0.5 * (dy < 0 ? -1 : 1);
          for (const n of nodes) {
            if (n.cluster === keys[i]) { n.cx -= sx; n.cy -= sy; }
            else if (n.cluster === keys[j]) { n.cx += sx; n.cy += sy; }
          }
        }
      }
    if (!moved) break;
  }

  // Normaliser en coordonnées positives.
  let minx = Infinity, miny = Infinity;
  for (const n of nodes) { minx = Math.min(minx, n.cx - n.w / 2); miny = Math.min(miny, n.cy - n.h / 2); }
  const offx = 120 - minx, offy = 120 - miny;
  const moves = nodes.map((n) => ({ id: n.id, x: Math.round(n.cx - n.w / 2 + offx), y: Math.round(n.cy - n.h / 2 + offy) }));

  // Membranes des clusters = boîte englobante finale de chaque territoire.
  const bounds = new Map();
  for (const n of nodes) {
    if (n.cluster === "__loose") continue;
    const b = bounds.get(n.cluster) ?? { minx: Infinity, miny: Infinity, maxx: -Infinity, maxy: -Infinity };
    b.minx = Math.min(b.minx, n.cx - n.w / 2 + offx); b.miny = Math.min(b.miny, n.cy - n.h / 2 + offy);
    b.maxx = Math.max(b.maxx, n.cx + n.w / 2 + offx); b.maxy = Math.max(b.maxy, n.cy + n.h / 2 + offy);
    bounds.set(n.cluster, b);
  }
  const PAD = 46;
  const zones = [...bounds.entries()].map(([k, b]) => ({
    x: Math.round(b.minx - PAD), y: Math.round(b.miny - PAD - 30),
    width: Math.round((b.maxx - b.minx) + 2 * PAD), height: Math.round((b.maxy - b.miny) + 2 * PAD + 30),
    text: clusterLabel.get(k) || "(zone)", color: "#ededed",
  }));

  const removeIds = membranes.map((m) => m.id);
  const res = toolApplyLayout({
    path, outPath: args.outPath, overwrite: args.overwrite, boardId: board.id,
    removeIds, moves, zones, arrows: [],
  });
  return `${res}\n\n(force layout déterministe : ${nodes.length} nœuds · ${edges.length} arêtes · ${zones.length} territoires · ${iters} itérations · graine ${args.seed ?? 7})`;
}

function callTool(name, args) {
  switch (name) {
    case "list_glucose_projects":
      return toolListProjects(args ?? {});
    case "read_glucose":
      return toolReadGlucose(args ?? {});
    case "search_glucose":
      return toolSearch(args ?? {});
    case "create_glucose_project":
      return toolCreate(args ?? {});
    case "add_note":
      return toolAddNote(args ?? {});
    case "connect_notes":
      return toolConnect(args ?? {});
    case "analyze_architecture":
      return toolAnalyze(args ?? {});
    case "apply_layout":
      return toolApplyLayout(args ?? {});
    case "lint_layout":
      return toolLint(args ?? {});
    case "detect_organization":
      return toolDetect(args ?? {});
    case "optimize_layout":
      return toolOptimizeLayout(args ?? {});
    default:
      throw new Error(`Outil inconnu : ${name}`);
  }
}

// ── Boucle JSON-RPC sur stdio ───────────────────────────────────────────────

function send(msg) {
  process.stdout.write(JSON.stringify(msg) + "\n");
}
function reply(id, result) {
  send({ jsonrpc: "2.0", id, result });
}
function replyError(id, code, message) {
  send({ jsonrpc: "2.0", id, error: { code, message } });
}
function log(...a) {
  process.stderr.write("[glucose-mcp] " + a.join(" ") + "\n");
}

function handle(msg) {
  const { id, method, params } = msg;
  // Notifications (pas d'id) → aucune réponse.
  if (id === undefined || id === null) {
    return;
  }
  switch (method) {
    case "initialize":
      reply(id, {
        protocolVersion: params?.protocolVersion || PROTOCOL_VERSION,
        capabilities: { tools: {} },
        serverInfo: { name: SERVER_NAME, version: SERVER_VERSION },
      });
      return;
    case "ping":
      reply(id, {});
      return;
    case "tools/list":
      reply(id, { tools: TOOLS });
      return;
    case "tools/call": {
      const name = params?.name;
      const args = params?.arguments ?? {};
      try {
        const text = callTool(name, args);
        reply(id, { content: [{ type: "text", text }] });
      } catch (e) {
        // Erreur « métier » → on la renvoie DANS le résultat (isError) pour que
        // le modèle la voie, plutôt qu'en erreur protocole.
        reply(id, { content: [{ type: "text", text: `Erreur: ${e.message}` }], isError: true });
      }
      return;
    }
    default:
      replyError(id, -32601, `Méthode inconnue : ${method}`);
  }
}

function main() {
  log(`démarré (Automerge, lecture + écriture) — racine défaut : ${DEFAULT_ROOT}`);
  let buf = "";
  process.stdin.setEncoding("utf8");
  process.stdin.on("data", (chunk) => {
    buf += chunk;
    let nl;
    while ((nl = buf.indexOf("\n")) >= 0) {
      const line = buf.slice(0, nl).trim();
      buf = buf.slice(nl + 1);
      if (!line) continue;
      let msg;
      try {
        msg = JSON.parse(line);
      } catch {
        log("ligne JSON invalide ignorée");
        continue;
      }
      try {
        handle(msg);
      } catch (e) {
        log("handler a levé:", e.message);
      }
    }
  });
  process.stdin.on("end", () => {
    // Quand stdin se ferme, laisse stdout FINIR de se vider avant de quitter.
    // Sinon, si stdout est un pipe, les dernières réponses (volumineuses, ex.
    // read_glucose) peuvent être tronquées par une sortie trop brutale.
    process.stdout.end(() => process.exit(0));
  });
}

main();
