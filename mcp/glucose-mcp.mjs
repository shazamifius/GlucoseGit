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
// optimize_layout). Les outils d'écriture sauvegardent l'existant avant de
// toucher un fichier existant, et acceptent `outPath` pour travailler sur copie.
// Un `.glucose` v1 (JSON legacy) écrit est converti en v2 binaire, comme le fait
// l'app à son prochain save.
// ─────────────────────────────────────────────────────────────────────────────

import { readFileSync, statSync, readdirSync, writeFileSync, existsSync, copyFileSync, unlinkSync } from "node:fs";
import { join, extname, basename, dirname, resolve } from "node:path";
import { homedir } from "node:os";
import { randomUUID, createHash } from "node:crypto";
import { next as A } from "@automerge/automerge";
import {
  COL_WIDTH, LINE_H, NOTE_GAP, estimateLines, heightUpperBound,
  annBox, imgBox, annCenter, overlappingPairs, separateUntilClean,
} from "./geometry.mjs";

const SERVER_NAME = "glucose";
const SERVER_VERSION = "0.1.0";
const PROTOCOL_VERSION = "2024-11-05";

// Racine par défaut des projets, quand un outil ne reçoit pas de `root` explicite.
// `GLUCOSE_ROOT` la surcharge : indispensable hors Windows, où ~/Documents n'existe
// pas sur beaucoup d'installations (NixOS et les distros sans profil XDG complet)
// et où les projets vivent ailleurs. Sans ce repli, list/search renverraient
// « aucun projet » sur un disque qui en est plein — un échec muet, le pire genre.
const DEFAULT_ROOT = (() => {
  const fromEnv = (process.env.GLUCOSE_ROOT ?? "").trim();
  if (fromEnv) return resolve(fromEnv);
  const docs = join(homedir(), "Documents");
  return existsSync(docs) ? docs : homedir();
})();
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
//     pas d'écrasement sans `overwrite`, et une sauvegarde `<fichier>.orig.bak` avant
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
// COL_WIDTH, LINE_H, estimateLines, heightUpperBound et la séparation viennent de
// geometry.mjs : une seule source pour tout ce qui décide d'une position.
const COL_GAP = 60, Y_GAP = 28, SECTIONS_PER_COL = 3;

/**
 * Dispose une liste de notes en colonnes.
 *
 * On ÉCRIT `height` (la borne supérieure) au lieu de la laisser absente : sans
 * elle, le pont range selon sa devinette et l'app dessine autre chose, si bien
 * que les notes ne se chevauchent que là où personne ne mesure — à l'écran.
 * Écrite, elle est la même des deux côtés dès la première seconde. L'app la
 * corrigera à la baisse quand elle rendra vraiment la note (syncAnnotationSize) :
 * une borne ne peut que se resserrer, donc l'espace ne peut que s'ouvrir, jamais
 * se refermer sur un chevauchement.
 */
function layoutNotes(notes) {
  const anns = [];
  let col = 0, y = 0;
  notes.forEach((n, i) => {
    if (i > 0 && i % SECTIONS_PER_COL === 0) { col++; y = 0; }
    const x = col * (COL_WIDTH + COL_GAP);
    const type = n.type === "sticky" ? "sticky" : "text";
    const height = heightUpperBound(n.text, COL_WIDTH);
    anns.push({
      id: nid(), type, x, y, text: n.text,
      width: COL_WIDTH, height, fontSize: type === "sticky" ? 13 : 14,
    });
    y += height + Y_GAP;
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

// Nombre de sauvegardes horodatées conservées, `.orig.bak` non compris. Volontairement
// bas : un .glucose à images embarquées pèse >100 Mo, chaque copie coûte ce prix.
const MAX_ROLLING_BAK = 3;

/**
 * Sauvegarde avant écrasement. DEUX filets, de rôles distincts :
 *
 *  - `<f>.orig.bak` — l'état d'avant la TOUTE PREMIÈRE écriture du pont. Écrit
 *    une seule fois, jamais réécrit, jamais élagué. C'est lui qui rend le retour
 *    en arrière possible quoi qu'il arrive : l'ancien `.bak` à chemin fixe était
 *    écrasé à chaque run, donc deux runs suffisaient à perdre l'original.
 *  - `<f>.<horodatage>.bak` — l'état d'avant CE run. On ne garde que les
 *    MAX_ROLLING_BAK plus récents ; l'élagage retire les plus anciens.
 *
 * L'élagage ne touche JAMAIS `.orig.bak` (le plus ancien est le plus précieux :
 * élaguer par ancienneté détruirait exactement ce qu'on protège) ni un éventuel
 * `<f>.bak` hérité de l'ancien schéma.
 */
function backupBeforeWrite(path) {
  if (!existsSync(path)) return;
  const orig = `${path}.orig.bak`;
  try { if (!existsSync(orig)) copyFileSync(path, orig); } catch { /* best-effort */ }

  // Horodatage en UTC (suffixe Z) et non en heure locale : le tri lexicographique
  // ci-dessous décide quelle sauvegarde élaguer, et l'heure locale se répète au
  // passage à l'heure d'hiver — un tri faux élaguerait la mauvaise.
  const stamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19) + "Z";
  try { copyFileSync(path, `${path}.${stamp}.bak`); } catch { /* best-effort */ }

  try {
    const dir = dirname(path), base = basename(path);
    const rolling = readdirSync(dir)
      .filter((f) => f.startsWith(`${base}.`) && f.endsWith(".bak")
        && f !== `${base}.orig.bak` && f !== `${base}.bak`)
      .sort(); // horodatage ISO ⇒ ordre lexicographique = ordre chronologique
    for (const f of rolling.slice(0, -MAX_ROLLING_BAK)) {
      try { unlinkSync(join(dir, f)); } catch { /* best-effort */ }
    }
  } catch { /* best-effort */ }
}

/** Écrit un doc en `.glucose` (save complet), après sauvegarde de l'existant. */
function writeDoc(path, doc) {
  backupBeforeWrite(path);
  writeFileSync(path, Buffer.from(A.save(doc)));
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

const pointInRect = (px, py, r) =>
  px >= r.x && px <= r.x + r.w && py >= r.y && py <= r.y + r.h;

/**
 * Appartenance note → zone : plus PETITE membrane contenant le CENTRE de la note
 * (la plus petite gagne pour que des zones imbriquées désignent le sous-thème).
 *
 * UNE SEULE définition dans tout le fichier : le lint et les layouts la partagent.
 * Deux définitions divergentes feraient produire un placement selon une règle et
 * le juger selon une autre — le genre d'écart qui rend un score incompréhensible.
 */
function zonesOfNotes(notes, membranes) {
  const zoneOf = new Map();
  for (const n of notes) {
    const nb = annBox(n);
    let best = null, area = Infinity;
    for (const m of membranes) {
      const mb = annBox(m);
      if (pointInRect(nb.cx, nb.cy, mb) && mb.w * mb.h < area) { area = mb.w * mb.h; best = m; }
    }
    zoneOf.set(n.id, best ? best.id : null);
  }
  return zoneOf;
}

/** Médiane (statistique d'ordre : robuste aux extrêmes, contrairement à la moyenne). */
const median = (xs) => {
  if (!xs.length) return 0;
  const s = [...xs].sort((a, b) => a - b), m = s.length >> 1;
  return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2;
};

// Décomposition d'une boîte AXIS-ALIGNED sur un repère sémantique (u, u⊥) : les
// deux extents sont exacts (pas une approximation) pour tout u unitaire.
const perpOf = (u) => ({ x: -u.y, y: u.x });
const spanPerp = (b, u) => b.w * Math.abs(u.y) + b.h * Math.abs(u.x);
const spanPara = (b, u) => b.w * Math.abs(u.x) + b.h * Math.abs(u.y);

/** Recouvrement > 20px sur les DEUX axes (sous ce seuil, deux boîtes se frôlent). */
const overlap20 = (a, b) =>
  Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x) > 20 &&
  Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y) > 20;

/** a contient-elle b ? (zones imbriquées = taxonomie légitime, pas un défaut) */
const boxContains = (a, b) =>
  b.x >= a.x && b.y >= a.y && b.x + b.w <= a.x + a.w && b.y + b.h <= a.y + a.h;

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

/**
 * Croisement STRICT : deux segments qui se TOUCHENT ne se croisent pas.
 *
 * POURQUOI un second test alors que segCross existe : segCross s'appuie sur
 * Math.sign, qui rend 0 pour un point colinéaire ; le test `o(a,b,c) !== o(a,b,d)`
 * compare alors 0 à ±1 et déclare un croisement. Or deux flèches incidentes à la
 * MÊME note partagent exactement un point (les extrémités sont des centres) : tout
 * éventail sortant d'un nœud était donc facturé — 16 des 17 « croisements » du
 * board de référence. Une incidence n'est pas un croisement : c'est la structure
 * du graphe qui se voit, pas un défaut.
 *
 * segHitsRect, lui, GARDE segCross : pour un rectangle, un segment tangent à une
 * arête touche bel et bien la note. Les deux tests ont des rôles distincts — les
 * avoir confondus était le bug.
 */
function segCrossStrict(a, b, c, d) {
  const o = (p, q, r) => Math.sign((q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x));
  const o1 = o(a, b, c), o2 = o(a, b, d), o3 = o(c, d, a), o4 = o(c, d, b);
  if (o1 === 0 || o2 === 0 || o3 === 0 || o4 === 0) return false;
  return o1 !== o2 && o3 !== o4;
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

// (arrowSeg a disparu avec l'ancien lint : sa table d'ancrage ne contenait que
// les notes, donc une flèche pointant une zone ou une image retombait en silence
// sur des coordonnées périmées. toolLint construit désormais une table fidèle à
// getAnchor — toute annotation non-arrow + les images.)

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

// ── Le mode d'emploi envoyé à l'IA au handshake (champ `instructions` du MCP) ─
//
// Une liste d'outils n'est pas un mode d'emploi. Une IA qui découvre Glucose lit
// les noms, prend le plus évident, et rend un graphe brut : elle a « fini » après
// create + connect, sans jamais soupçonner que les zones et les couleurs
// existaient. Ce n'est pas sa faute — c'est un défaut de découvrabilité. La doc
// fait partie de l'outil.
//
// Le test d'un bon outil n'est pas ce qu'un expert en tire, c'est ce qu'un novice
// produit PAR DÉFAUT. Ce texte est le seul endroit où l'on peut agir sur ça
// AVANT que l'IA ne choisisse.
const INSTRUCTIONS = `Glucose est un canevas spatial infini : des notes posées en 2D, reliées par des
flèches, regroupées dans des zones (membranes). Un .glucose est un document
Automerge (CRDT) binaire. Tu le lis et l'écris par ces outils — jamais à la main.

Tu ne vois PAS le canevas. Tu le comprends par sa STRUCTURE : géométrie, zones,
graphe des flèches. C'est déterministe et ça suffit — n'essaie pas d'imaginer les
pixels.

═══ LA RÈGLE D'OR ═══

TERMINE TOUJOURS PAR lint_layout, ET CORRIGE AVANT DE RENDRE.
Une carte que tu n'as pas lintée est une carte que tu n'as pas vue. Le lint est
ton seul œil : il calcule les défauts qu'un humain VERRAIT (notes qui se
chevauchent, flèches qui masquent une note, croisements). Si tu rends sans linter,
tu rends à l'aveugle.

═══ LES TROIS RECETTES ═══

Choisis-en une AVANT de commencer. Chacune produit une carte finie, pas un
brouillon.

┌─ PARCOURS (un cours, un fil, une démonstration : « A puis B puis C »)
│  1. create_glucose_project(notes)          → les notes
│  2. connect_notes(...) pour chaque étape   → le fil, dans l'ordre
│  3. optimize_layout(mode:"linear")         → une colonne de lecture
│  4. lint_layout                            → et tu corriges
└─ Le lecteur suit une ligne. Les flèches disent « ensuite ».

┌─ CARTE ZONÉE (des territoires par sujet : « voici les 4 familles d'idées »)
│  1. create_glucose_project(notes)
│  2. connect_notes(...)                     → les liens qui existent VRAIMENT
│  3. apply_layout(zones:[...], patches:[...]) → CRÉE les zones et teinte les notes
│  4. optimize_layout(mode:"thematic")       → force + clusters dans ces territoires
│  5. lint_layout
└─ ⚠️ L'étape 3 n'est PAS optionnelle : le mode "thematic" a besoin de zones qui
   EXISTENT DÉJÀ. Sans elle, tu obtiens un tas gris. C'est l'erreur la plus
   fréquente.

┌─ FRISE (une chronologie : « de 2000 à 2012 »)
│  1. create_glucose_project(notes)
│  2. apply_layout(zones:[...])              → une zone par époque, titre DATÉ
│                                              (ex. "Origine · ~2005" — l'année
│                                              dans le titre est ce qui fait la date)
│  3. connect_notes(...)                     → les liens entre époques
│  4. optimize_layout(mode:"chronological")  → la frise, ordonnée par année
│  5. lint_layout
└─ Une flèche longue LE LONG du temps est le message, pas un défaut : le lint le
   sait et ne la facture pas.

═══ CE QUE CHAQUE OUTIL EST VRAIMENT ═══

create_glucose_project — un STARTER MINIMAL : des notes en colonnes, rien d'autre.
  Ce n'est PAS une carte finie. Il te rend les ids : garde-les, tu en auras besoin.
apply_layout — LE CONSTRUCTEUR. Malgré son nom, il ne fait pas que réarranger : il
  CRÉE les zones, teinte les notes (patches), ajoute des flèches — le tout
  atomiquement. C'est lui qui fait la différence entre un graphe brut et une carte.
optimize_layout — réarrange une carte EXISTANTE en respectant l'intention détectée.
  Il ne rend jamais de chevauchement et te dit ce qu'il a corrigé.
analyze_architecture — comment le projet est bâti : zones, clusters, hubs, racines,
  feuilles, isolées. À lire AVANT de proposer quoi que ce soit sur un projet existant.
detect_organization — la logique que l'auteur a DÉJÀ suivie. Respecte-la : ne
  transforme pas sa frise en liste parce que tu préfères les listes.

═══ LES FLÈCHES : UTILISE-LES ENTIÈREMENT ═══

Une flèche peut pointer une PHRASE PRÉCISE, pas seulement le bloc entier :
connect_notes(sourceSel:"la phrase exacte", targetSel:"...") souligne ce texte et
la flèche le vise. C'est ce qui fait la différence entre « ces deux notes sont
liées » et « CETTE IDÉE-LÀ cause CELLE-CI ».

La phrase doit exister MOT POUR MOT dans la note (sous-chaîne exacte, mots
contigus) — lis la note avant, n'invente jamais. Ajoute un predicate quand la
relation a un nom : est_precurseur, contredit, herite_de, inspire, depend_de,
illustre.

═══ CE QUI FAIT PERDRE DU TEMPS ═══

• Le board « actif » est souvent VIDE (c'est le dernier consulté, pas celui qui
  compte). Les outils choisissent le board le plus structuré. Passe boardId si tu
  veux être sûr.
• Travaille sur une COPIE : outPath sur un chemin neuf. La source reste intacte.
• L'app Glucose enregistre toute seule ~1,5 s après une modification : si elle est
  ouverte sur le fichier que tu écris, elle écrasera ton travail. Demande à
  l'utilisateur de fermer.
• read_glucose(includeIds:true) avant connect_notes ou apply_layout : ces outils
  désignent les notes par id.`;

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
      "Position auto sous le contenu existant si x/y non fournis. Sauvegarde l'existant "
      + "(<f>.orig.bak, écrit une seule fois, + horodatées).",
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
        outPath: { type: "string", description: "Destination (copie recommandée). Défaut : écrase path, après sauvegarde (<f>.orig.bak + horodatée)." },
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
      "QA VISUEL sans vision : calcule les défauts qu'on VERRAIT à l'écran — notes qui " +
      "se chevauchent, flèches qui masquent une note, arêtes qui se croisent — puis les " +
      "défauts propres à l'INTENTION du board (détectée par detect_organization) : dans " +
      "une frise, une flèche longue LE LONG du temps est le message et ne coûte rien, " +
      "seul son écart EN TRAVERS est facturé. Déterministe, 0 token, 0 modèle. Le " +
      "rapport annonce le mode et l'étalon qui ont servi à juger.",
    inputSchema: {
      type: "object",
      properties: {
        path: { type: "string", description: "Chemin absolu du .glucose à contrôler." },
        board: { type: "string", description: "Board ciblé (défaut : le plus riche en flèches)." },
        mode: { type: "string", enum: ["auto", "chronological", "thematic", "hub", "linear", "unstructured"], description: "Référentiel de jugement. auto (défaut) = détecte l'intention. Deux modes = deux jeux de critères : leurs scores ne se comparent pas." },
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
        outPath: { type: "string", description: "Destination (copie recommandée). Défaut : écrase path, après sauvegarde (<f>.orig.bak + horodatée)." },
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
 * Écrire dans un v1 le convertit donc en v2 (l'original reste dans <f>.orig.bak).
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

  // ── INVARIANT : on ne rend jamais un chevauchement ────────────────────────
  //
  // Point de passage obligé de TOUS les layouts : c'est ici, une seule fois,
  // qu'on vérifie ce qu'on s'apprête à écrire. Un layout qui se croit propre
  // n'est pas une preuve — les notes se chevauchaient à l'écran précisément parce
  // que personne ne regardait après coup.
  //
  // QUI BOUGE :
  //  • les notes (annotations hors flèches/membranes) — elles s'écartent ;
  //  • les IMAGES et VIDÉOS ne bougent JAMAIS : l'utilisateur les a posées là,
  //    ce sont des ancres. Mais elles COMPTENT comme obstacles — les ignorer,
  //    c'est empiler des notes sur des vidéos en annonçant « ✅ 0 chevauchement ».
  //    Elles vivent dans `board.images`, pas dans `annotations`, et sont ancrées
  //    par leur CENTRE (d'où `imgBox`) là où une note l'est par son coin.
  //  • une membrane est un fond : elle englobe, elle ne chevauche pas.
  //  • une flèche suit ses nœuds.
  let repare = 0, subsiste = 0, zonesRecalees = 0;
  {
    const plainAfter = typeof A.toJS === "function" ? A.toJS(doc) : JSON.parse(JSON.stringify(doc));
    const bAfter = (plainAfter.boards ?? []).find((bb) => bb.id === boardId);
    const notes = (bAfter?.annotations ?? []).filter((a) => a.type !== "arrow" && a.type !== "membrane");
    const mobiles = notes.map((n) => { const bx = annBox(n); return { id: n.id, x: bx.x, y: bx.y, w: bx.w, h: bx.h, cx: bx.cx, cy: bx.cy }; });
    const fixes = (bAfter?.images ?? []).map((im) => { const bx = imgBox(im); return { id: im.id, x: bx.x, y: bx.y, w: bx.w, h: bx.h, cx: bx.cx, cy: bx.cy }; });

    const avantN = mobiles.map((m) => ({ id: m.id, x: m.x, y: m.y }));
    const avant = overlappingPairs([...mobiles, ...fixes]);
    if (avant.length) {
      const res = separateUntilClean(mobiles, fixes, NOTE_GAP);
      const bouges = new Map();
      mobiles.forEach((m, i) => {
        if (Math.round(m.x) !== Math.round(avantN[i].x) || Math.round(m.y) !== Math.round(avantN[i].y)) bouges.set(m.id, m);
      });
      if (bouges.size) {
        // Les membranes qui contenaient une note déplacée doivent la contenir
        // encore : sinon la réparation sortirait la note de son territoire en
        // silence, et casserait le sens de la carte pour sauver sa lisibilité.
        const membranes = (bAfter?.annotations ?? []).filter((a) => a.type === "membrane");
        const grandir = new Map();
        for (const m of membranes) {
          const mb = annBox(m);
          const dedansAvant = avantN.filter((n0) => {
            const w = mobiles.find((x) => x.id === n0.id);
            return n0.x >= mb.x && n0.y >= mb.y && n0.x + w.w <= mb.x + mb.w && n0.y + w.h <= mb.y + mb.h;
          });
          if (!dedansAvant.length) continue;
          let x1 = mb.x, y1 = mb.y, x2 = mb.x + mb.w, y2 = mb.y + mb.h;
          for (const n0 of dedansAvant) {
            const w = bouges.get(n0.id); if (!w) continue;
            x1 = Math.min(x1, w.x - 20); y1 = Math.min(y1, w.y - 20);
            x2 = Math.max(x2, w.x + w.w + 20); y2 = Math.max(y2, w.y + w.h + 20);
          }
          if (x1 !== mb.x || y1 !== mb.y || x2 !== mb.x + mb.w || y2 !== mb.y + mb.h)
            grandir.set(m.id, { x: Math.round(x1), y: Math.round(y1), width: Math.round(x2 - x1), height: Math.round(y2 - y1) });
        }
        zonesRecalees = grandir.size;
        doc = A.change(doc, "anti-chevauchement via mcp", (d) => {
          const b = d.boards.find((bb) => bb.id === boardId);
          for (const a of b.annotations) {
            const w = bouges.get(a.id);
            if (w) { a.x = Math.round(w.x); a.y = Math.round(w.y); continue; }
            const g = grandir.get(a.id);
            if (g) { a.x = g.x; a.y = g.y; a.width = g.width; a.height = g.height; }
          }
        });
      }
      repare = avant.length;
      subsiste = res.ok ? 0 : res.restant;
    }
  }

  writeDoc(outPath, doc);
  let msg = `✅ Organisation appliquée → ${removed} retirée(s), ${moved} déplacée(s), ${addedZones} zone(s), ${addedArrows} flèche(s)${patched ? `, ${patched} patchée(s)` : ""}.\n\`${outPath}\``;
  if (missing.length) msg += `\n⚠️ ${missing.length} id(s) introuvable(s) : ${missing.slice(0, 5).join(", ")}${missing.length > 5 ? "…" : ""}`;
  if (repare && !subsiste) {
    msg += `\n🔧 ${repare} chevauchement(s) corrigé(s) automatiquement — la carte rendue n'en contient aucun (notes ET médias).`;
    if (zonesRecalees) msg += `\n   ${zonesRecalees} zone(s) élargie(s) pour continuer à contenir leurs notes.`;
  } else if (subsiste) {
    msg += `\n❌ ${subsiste} chevauchement(s) IRRÉDUCTIBLE(S) : la carte est illisible à ces endroits. Ne fais pas comme si de rien n'était.`;
  } else {
    msg += `\n✅ 0 chevauchement (vérifié sur la géométrie rendue, notes ET médias — pas supposé).`;
  }
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

// ── Lint CONSCIENT DE L'INTENTION ───────────────────────────────────────────
//
// Un lint ne juge pas un dessin : il juge un dessin SELON SON INTENTION. Trois
// idées, dans cet ordre.
//
// 1. detectMode() sert de RÉFÉRENTIEL. Il existait depuis toujours et le lint ne
//    l'appelait jamais : on appliquait à une frise les seuils d'un layout en
//    colonnes. Le mode n'ajoute pas des règles — il dit quelle direction PORTE DU
//    SENS, et une longueur portée par le sens n'est pas un défaut.
//
// 2. On DÉCOMPOSE au lieu de mesurer la norme. Math.hypot est ISOTROPE : il traite
//    toutes les directions comme équivalentes. Dans une frise elles ne le sont
//    pas — x = le temps = le message, y = la mise en page subie. Chaque mode
//    définit un vecteur unitaire sémantique u, et toute arête v se décompose
//    exactement en along = |v·u| (jamais facturé) et across = |v·u⊥| (seul
//    facturé). Deux flèches de même longueur brute peuvent donc recevoir des
//    verdicts opposés — c'est précisément le but.
//
// 3. On partitionne les flèches par leur STATUT STRUCTUREL (des champs), pas par
//    leur géométrie (des pixels) : EDGE (dans le graphe de sens), CHROME (décor
//    structurel, ex. l'axe du temps), DÉBRIS (défaut d'intégrité).
//
// Aucun LOD, aucune dépendance au zoom, aucune vision : des produits scalaires et
// des intervalles sur la donnée.

/**
 * DÉCOR STRUCTUREL — une flèche est du décor ssi les TROIS clauses sont vraies :
 *   D1 libre aux DEUX bouts   → elle ne référence aucun nœud, elle n'est pas
 *                               dans le graphe de sens ;
 *   D2 étiquetée              → sans texte, ce n'est pas du décor mais un trait
 *                               oublié (→ DÉBRIS : sortir du graphe COÛTE, sauf à
 *                               assumer une étiquette — l'exemption n'est pas une
 *                               zone franche) ;
 *   D3 extrémités dans le vide → une règle graduée ne pose pas ses bouts sur le
 *                               contenu. Ferme le seul trou de D1+D2 : la flèche
 *                               « A cause B » dessinée à la main entre deux notes
 *                               sans les attacher est une ARÊTE, et reste jugée.
 *
 * NE FONDE RIEN sur strokeWidth ni sur le texte : strokeWidth est un attribut de
 * style réglable en deux clics (fonder une exemption dessus = offrir un
 * interrupteur « rends-moi invisible au lint ») et un regex de libellé rendrait
 * l'exemption dépendante de la LANGUE. D1+D2+D3 ne dépendent que de la structure.
 *
 * L'exemption est BORNÉE : le décor échappe à la longueur et aux croisements
 * (il n'est pas dans le graphe, il ne peut pas l'emmêler), JAMAIS à l'obstruction
 * — un décor qui cache une note est un vrai défaut, quelle que soit son intention.
 */
function classifyArrows(arrows, notes, anchorById, segFrom) {
  const edges = [], chrome = [], debris = [];
  for (const ar of arrows) {
    const entry = { ar, seg: segFrom(ar) };
    if (!ar.sourceId && !ar.targetId) {
      if (!(ar.text ?? "").trim()) { debris.push({ ar, why: "flèche libre sans étiquette" }); continue; }
      // Pour une flèche libre, x/y/x2/y2 font FOI (getAnchor : `if (!refId) return
      // {x: fallbackX, y: fallbackY}`) — contrairement à une flèche attachée, dont
      // les coordonnées stockées sont périmées et ignorées par l'app.
      const onNote = (pt) => notes.some((n) => pointInRect(pt.x, pt.y, annBox(n)));
      if (onNote(entry.seg[0]) || onNote(entry.seg[1])) edges.push(entry);
      else chrome.push(entry);
      continue;
    }
    if ((ar.sourceId && !anchorById.has(ar.sourceId)) || (ar.targetId && !anchorById.has(ar.targetId))) {
      debris.push({ ar, why: "référence morte (id introuvable)" });
      continue;
    }
    edges.push(entry);
  }
  return { edges, chrome, debris };
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

  // L'APPEL QUI MANQUAIT. `mode` force le référentiel ; par défaut on le détecte.
  const forced = !!(args.mode && args.mode !== "auto");
  const det = detectMode(board);
  const mode = forced ? args.mode : det.mode;
  const evidence = forced ? [`mode forcé par l'appelant (auto aurait dit « ${det.mode} »)`] : det.evidence;

  const anns = board.annotations ?? [];
  const notes = anns.filter((a) => a.type === "text" || a.type === "sticky");
  const membranes = anns.filter((a) => a.type === "membrane");
  const arrows = anns.filter((a) => a.type === "arrow");

  // TABLE D'ANCRAGE — fidèle à getAnchor (ArrowSvgLayer) : l'app résout un refId
  // contre TOUTE annotation non-arrow **et les images**. L'ancienne table ne
  // contenait que les notes : une flèche pointant une membrane retombait en
  // silence sur ses x/y/x2/y2 périmés, et le lint mesurait un segment FANTÔME que
  // personne ne voit à l'écran. On élargit ce qui compte comme « attachée » ; on
  // garde le centre-à-centre, qui est bien ce que l'app dessine.
  const anchorById = new Map();
  for (const a of anns) if (a.type !== "arrow") anchorById.set(a.id, annBox(a));
  for (const im of board.images ?? []) {
    const w = im.width ?? 0, h = im.height ?? 0, x = im.x ?? 0, y = im.y ?? 0;
    anchorById.set(im.id, { x: x - w / 2, y: y - h / 2, w, h, cx: x, cy: y });
  }
  const segFrom = (ar) => {
    const s = ar.sourceId ? anchorById.get(ar.sourceId) : null;
    const t = ar.targetId ? anchorById.get(ar.targetId) : null;
    return [
      s ? { x: s.cx, y: s.cy } : { x: ar.x ?? 0, y: ar.y ?? 0 },
      t ? { x: t.cx, y: t.cy } : { x: ar.x2 ?? 0, y: ar.y2 ?? 0 },
    ];
  };
  const { edges, chrome, debris } = classifyArrows(arrows, notes, anchorById, segFrom);

  // ── ÉTALON : dérivé du CONTENU, jamais des statistiques de ce qu'on mesure ──
  // Un seuil tiré de la population mesurée est une cible mobile : raccourcir la
  // pire arête ferait baisser la médiane des arêtes, donc refacturerait une arête
  // saine — le score deviendrait un thermomètre qui bouge avec la fièvre. On le
  // dérive donc de la TAILLE DES NOTES, invariante par re-layout : toolApplyLayout
  // n'écrit que x/y/x2/y2, il ne redimensionne JAMAIS une note. C'est la raison
  // exacte du choix — ni la longueur des arêtes ni la hauteur des zones n'ont
  // cette propriété (chronoLayout les recalcule à chaque passage).
  const K = 6;
  const u = (() => {
    if (mode === "chronological") {
      const ax = chrome.find((c) => /chronolog|timeline|frise|le temps|ann[ée]e/i.test(c.ar.text || ""));
      if (ax) {
        const v = { x: ax.seg[1].x - ax.seg[0].x, y: ax.seg[1].y - ax.seg[0].y };
        const n = Math.hypot(v.x, v.y);
        if (n > 0) return { x: v.x / n, y: v.y / n }; // l'axe du temps DONNE le référentiel
      }
      const dated = membranes.filter((m) => yearOf(m.text) !== null).map(annBox);
      if (dated.length >= 2) {
        const spanX = Math.max(...dated.map((b) => b.cx)) - Math.min(...dated.map((b) => b.cx));
        const spanY = Math.max(...dated.map((b) => b.cy)) - Math.min(...dated.map((b) => b.cy));
        return spanX >= spanY ? { x: 1, y: 0 } : { x: 0, y: 1 }; // extension dominante
      }
    }
    return { x: 1, y: 0 };
  })();
  const up = perpOf(u);
  const boxes = notes.map(annBox);
  const Sperp = median(boxes.map((b) => spanPerp(b, u)));
  const Sdiag = median(boxes.map((b) => Math.hypot(b.w, b.h)));
  const Tperp = K * Sperp, Tiso = K * Sdiag;
  // Empreinte : deux scores ne sont comparables QUE si les étalons sont égaux.
  // On préfère refuser une comparaison que la falsifier.
  const fp = createHash("sha1")
    .update([mode, u.x.toFixed(6), u.y.toFixed(6), Sperp.toFixed(3), Sdiag.toFixed(3), K].join("|"))
    .digest("hex").slice(0, 12);

  // ── DÉFAUTS UNIVERSELS (tous modes) ────────────────────────────────────────
  // U1 — chevauchement de notes : illisible dans tous les modes, aucune intention
  // ne le rachète.
  const overlaps = [];
  for (let i = 0; i < notes.length; i++)
    for (let j = i + 1; j < notes.length; j++)
      if (overlap20(boxes[i], boxes[j])) overlaps.push([notes[i], notes[j]]);

  // U2 — obstruction : une flèche qui passe sur une note tierce cache du contenu.
  // Compté PAR COUPLE (flèche × note) et non par flèche, à dessein : 4 notes
  // masquées = 4 dégâts, c'est la vérité. Le CHROME est facturé ici, et ici seul.
  const collisions = [];
  for (const { ar, seg } of [...edges, ...chrome])
    for (const n of notes) {
      if (n.id === ar.sourceId || n.id === ar.targetId) continue;
      if (segHitsRect(seg[0], seg[1], annBox(n))) collisions.push({ ar, note: n });
    }

  // U3 — croisement d'ARÊTES, avec les deux correctifs : (a) les paires qui
  // partagent un nœud sont exclues par ID avant tout test géométrique — deux
  // arêtes incidentes au même nœud s'y rencontrent par construction ; (b) test
  // strict, le contact colinéaire n'est pas un croisement.
  const sharesNode = (a, b) => {
    const ia = [a.sourceId, a.targetId].filter(Boolean);
    const ib = [b.sourceId, b.targetId].filter(Boolean);
    return ia.some((i) => ib.includes(i));
  };
  let crossings = 0;
  for (let i = 0; i < edges.length; i++)
    for (let j = i + 1; j < edges.length; j++) {
      if (sharesNode(edges[i].ar, edges[j].ar)) continue;
      if (segCrossStrict(edges[i].seg[0], edges[i].seg[1], edges[j].seg[0], edges[j].seg[1])) crossings++;
    }

  // U4 — intégrité : référence morte ou trait oublié. Non mesuré géométriquement
  // (il n'y a pas de géométrie fiable à mesurer) — signalé.
  const base = overlaps.length * 2 + collisions.length * 2 + crossings + debris.length;

  const L = [`# Lint visuel — « ${p.name} » · board « ${board.name} »`];
  L.push(`${notes.length} notes · ${edges.length} arêtes · ${chrome.length} décor · ${membranes.length} zones`);
  L.push(`\n**Mode retenu : ${mode}**${forced ? " (forcé)" : " (auto)"} — ${evidence.join(" · ") || "—"}`);
  L.push(`> ${MODE_LABEL[mode] ?? mode}. Jugé selon les critères de ce mode : un autre mode`);
  L.push(`> donnerait d'autres critères, et les scores de deux modes ne se comparent pas.`);
  L.push(`\n_Étalon : u=(${u.x.toFixed(3)}, ${u.y.toFixed(3)}) · S⊥=${Sperp.toFixed(1)} · ` +
    `T⊥=${Math.round(Tperp)} · S_diag=${Sdiag.toFixed(1)} · K=${K} · empreinte \`${fp}\`_`);

  const extra = []; // lignes de détail propres au mode
  let modeScore = 0;

  if (mode === "chronological") {
    const zoneOf = zonesOfNotes(notes, membranes);
    const mById = new Map(membranes.map((m) => [m.id, m]));
    const yrOf = (id) => {
      const z = zoneOf.get(id);
      return z && mById.has(z) ? yearOf(mById.get(z).text) : null;
    };
    // C1 — écart TRANSVERSAL : l'arête s'écarte de l'axe du temps de plus de K
    // hauteurs-de-note. C'est un écart SUBI, pas une distance temporelle.
    // |along| — la longueur LE LONG du temps — coûte ZÉRO, explicitement : relier
    // 2005 à 2012 est le message de la frise, pas un désordre.
    const c1 = [];
    let c2 = 0;
    for (const { ar, seg } of edges) {
      const v = { x: seg[1].x - seg[0].x, y: seg[1].y - seg[0].y };
      const along = Math.abs(v.x * u.x + v.y * u.y);
      const across = Math.abs(v.x * up.x + v.y * up.y);
      if (across > Tperp) c1.push({ ar, along, across });
      // C2 — contre-sens temporel : testé sur les ANNÉES, pas sur le signe de v·u
      // (robuste si l'axe est inversé ou oblique). C'est LE défaut propre à une
      // frise : dans un dessin qui se lit gauche→droite, une flèche à rebours est
      // la seule longueur qui coûte vraiment au lecteur.
      const ys = yrOf(ar.sourceId), yt = yrOf(ar.targetId);
      if (ys != null && yt != null && yt < ys) c2++;
    }
    // C3 — bande hors-format : le « parpaing ». Une bande sert à être balayée puis
    // comparée à ses voisines ; au-delà de 3:1 le lecteur défile le long d'une
    // bande et perd la comparaison — donc l'unique raison d'être des bandes.
    const c3 = membranes.filter((m) => {
      const b = annBox(m);
      return spanPerp(b, u) > 3 * spanPara(b, u);
    });
    modeScore = c1.length + c2 + c3.length;
    extra.push(`- **Écart transversal à l'axe (> ${Math.round(Tperp)}px) :** ${c1.length}`);
    for (const { ar, along, across } of c1.sort((a, b) => b.across - a.across).slice(0, 6))
      extra.push(`   - ${ar.text ? `« ${ar.text} »` : ar.predicate || "(sans label)"} — ` +
        `le long du temps ${Math.round(along)}px (gratuit) · en travers ${Math.round(across)}px ✗`);
    extra.push(`- **Contre-sens temporels :** ${c2}`);
    extra.push(`- **Bandes hors-format (h > 3·l) :** ${c3.length}` +
      (c3.length ? ` — ex. « ${nodeLabel(c3[0]).slice(0, 24)} » ` +
        `${Math.round(spanPara(annBox(c3[0]), u))}×${Math.round(spanPerp(annBox(c3[0]), u))} ` +
        `(1:${(spanPerp(annBox(c3[0]), u) / spanPara(annBox(c3[0]), u)).toFixed(1)})` : ""));
  } else if (mode === "hub") {
    // La LONGUEUR DES RAYONS n'est jamais facturée : un rayon long est la forme
    // même d'un hub. Les croisements entre rayons sont gratuits sans règle
    // spéciale — tous les rayons partagent H, donc U3(a) les élimine déjà tous.
    const deg = new Map();
    for (const { ar } of edges)
      for (const id of [ar.sourceId, ar.targetId])
        if (id) deg.set(id, (deg.get(id) ?? 0) + 1);
    const H = [...deg.entries()].sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : 1))[0]?.[0];
    const hb = H ? anchorById.get(H) : null;
    let h1 = 0, h2 = 0;
    if (hb) {
      const inc = (ar) => ar.sourceId === H || ar.targetId === H;
      const spokes = edges.filter((e) => inc(e.ar));
      const chords = edges.filter((e) => !inc(e.ar));
      const k = spokes.length || 1;
      // H1 — rayons confondus : un éventail de k rayons se partage 2π, pas idéal
      // 2π/k ; deux rayons plus serrés que la MOITIÉ de ce pas se recouvrent
      // visuellement. Auto-calibré : plus le hub est chargé, plus la tolérance est
      // fine — exactement comme la lisibilité réelle.
      const th = spokes.map(({ ar, seg }) => {
        const far = ar.sourceId === H ? seg[1] : seg[0];
        return Math.atan2(far.y - hb.cy, far.x - hb.cx);
      });
      for (let i = 0; i < th.length; i++)
        for (let j = i + 1; j < th.length; j++) {
          const d = Math.abs(th[i] - th[j]);
          if (Math.min(d, 2 * Math.PI - d) < Math.PI / k) h1++;
        }
      // H2 — corde traversante : elle coupe le disque au lieu de longer la
      // couronne, seul type de flèche qu'une étoile ne peut pas lire radialement.
      h2 = chords.filter(({ seg }) => Math.hypot(seg[1].x - seg[0].x, seg[1].y - seg[0].y) > Tiso).length;
      extra.push(`- **Rayons confondus (Δθ < ${(180 / k).toFixed(0)}°) :** ${h1}  _(hub degré ${k})_`);
      extra.push(`- **Cordes traversantes (> ${Math.round(Tiso)}px) :** ${h2}`);
    }
    modeScore = h1 + h2;
  } else if (mode === "linear") {
    // L1 — inversion de lecture : on réutilise readingOrder, la définition maison
    // de « lire » (haut→bas puis gauche→droite, bande d'indifférence de 40px). On
    // n'en invente pas une seconde.
    let l1 = 0, l2 = 0;
    for (const { ar, seg } of edges) {
      const a = ar.sourceId ? anchorById.get(ar.sourceId) : null;
      const b = ar.targetId ? anchorById.get(ar.targetId) : null;
      if (a && b && readingOrder(a, b) > 0) l1++;
      if (Math.hypot(seg[1].x - seg[0].x, seg[1].y - seg[0].y) > Tiso) l2++;
    }
    modeScore = l1 + l2;
    extra.push(`- **Inversions de lecture :** ${l1}`);
    extra.push(`- **Sauts de séquence (> ${Math.round(Tiso)}px) :** ${l2}`);
  } else if (mode === "thematic") {
    // AUCUNE règle de longueur, dans aucune direction : une carte thématique n'a
    // pas de direction privilégiée, donc pas de u, donc rien à décomposer. Et une
    // arête qui relie deux zones éloignées est un PONT — l'affirmation la plus
    // intéressante de la carte. La facturer, ce serait demander à l'auteur de
    // taire ses rapprochements. U2 et U3 suffisent : une arête vraiment nuisible
    // cache une note ou emmêle le graphe, et elle est comptée pour ce qu'elle
    // FAIT, pas pour sa taille.
    const zoneOf = zonesOfNotes(notes, membranes);
    const mb = membranes.map(annBox);
    let t1 = 0;
    for (let i = 0; i < mb.length; i++)
      for (let j = i + 1; j < mb.length; j++)
        if (overlap20(mb[i], mb[j]) && !boxContains(mb[i], mb[j]) && !boxContains(mb[j], mb[i])) t1++;
    let t2 = 0;
    for (const n of notes)
      for (const m of membranes)
        if (overlap20(annBox(n), annBox(m)) && zoneOf.get(n.id) !== m.id) t2++;
    const t3 = membranes.length ? notes.filter((n) => zoneOf.get(n.id) === null).length : 0;
    const t4 = membranes.filter((m) => { const b = annBox(m); return spanPerp(b, u) > 3 * spanPara(b, u); }).length;
    modeScore = 2 * t1 + t2 + t3 + t4;
    extra.push(`- **Zones qui se chevauchent :** ${t1}  _(imbriquées = sous-thème, épargnées)_`);
    extra.push(`- **Notes à cheval sur une autre zone :** ${t2}`);
    extra.push(`- **Notes orphelines (hors de tout thème) :** ${t3}`);
    extra.push(`- **Bandes hors-format (h > 3·l) :** ${t4}`);
  } else {
    // unstructured — sans intention détectée, aucune direction n'est privilégiée :
    // on retombe sur la norme isotrope, mais mesurée en UNITÉS-NOTE du board et
    // non avec le 704 hérité d'un layout en colonnes étranger au board mesuré. Un
    // board dessiné en grand n'est pas un board en désordre.
    const n1 = edges.filter(({ seg }) => Math.hypot(seg[1].x - seg[0].x, seg[1].y - seg[0].y) > Tiso).length;
    modeScore = n1;
    extra.push(`- **Flèches trop longues (> ${Math.round(Tiso)}px = K·S_diag) :** ${n1}`);
  }

  const score = base + modeScore;
  L.push(`\n**Score ${mode} : ${score}**  _(comparable à un run antérieur ssi l'empreinte d'étalon est identique)_`);
  L.push(`- **Notes qui se chevauchent :** ${overlaps.length}` +
    (overlaps.length ? ` — ex. « ${nodeLabel(overlaps[0][0]).slice(0, 28)} » ∩ « ${nodeLabel(overlaps[0][1]).slice(0, 28)} »` : ""));
  L.push(`- **Flèches qui masquent une note :** ${collisions.length}` +
    (collisions.length ? ` — ex. sur « ${nodeLabel(collisions[0].note).slice(0, 32)} »` : ""));
  L.push(`- **Croisements d'arêtes :** ${crossings}  _(incidences sur un nœud commun exclues)_`);
  L.push(`- **Intégrité :** ${debris.length}` + (debris.length ? ` — ex. ${debris[0].why}` : ""));
  L.push(...extra);

  if (chrome.length) {
    L.push(`\n**Décor structurel (non facturé en longueur ni en croisement) :** ` +
      chrome.map((c) => `« ${c.ar.text} » ${Math.round(Math.hypot(c.seg[1].x - c.seg[0].x, c.seg[1].y - c.seg[0].y))}px`).join(", "));
  }
  // Le lint ne propose JAMAIS de supprimer un élément : il dit ce qu'il mesure.
  // Suggérer de retirer l'axe du temps — le référentiel même de la frise — était
  // le vrai défaut de ce rapport.

  // Le score est EXTENSIF (une somme de comptages) : c'est ce qui le rend
  // comparable dans le temps. Le VERDICT, lui, doit être intensif, sinon un board
  // de 300 notes est rouge d'office et un board de 4 notes vert quoi qu'il arrive.
  const d = score / Math.max(1, notes.length + edges.length);
  L.push(`\nDensité de défauts : ${d.toFixed(2)} par objet.`);
  L.push(d === 0 ? "✅ Rien à signaler." : d < 0.25 ? "🟡 Défauts mineurs." : "🔴 Rendu encombré — à nettoyer.");
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

// ── Layout CHRONOLOGIQUE ────────────────────────────────────────────────────
//
// Il remplace une géométrie à DEUX RANGÉES qui fabriquait elle-même les défauts
// que lint_layout dénonçait ensuite. Deux corrections, chacune adossée à un
// théorème plutôt qu'à un réglage.
//
// 1. L'ABÎME (des flèches de 4000px). L'ancien code exilait sous la frise toute
//    zone dont le TITRE ne contenait pas 4 chiffres, et posait cette rangée à
//    `max_Z Σ hauteurs + 150` — un maximum GLOBAL, donc une fonction NON BORNÉE du
//    contenu : une note longue de plus dans une colonne creusait l'abîme pour tout
//    le monde. Les flèches datée↔non-datée devaient le traverser.
//    Le critère d'exil devient TOPOLOGIQUE au lieu de lexical : « des flèches
//    relient-elles cette zone à la frise ? ». D'où le théorème qui règle le
//    problème — une composante SANS aucune arête vers la frise peut être posée
//    n'importe où, elle n'allonge aucune flèche ; et réciproquement, une zone
//    reliée à la frise EST attirée par elle et la rejoint. Le nombre d'arêtes
//    frise↔bande est donc EXACTEMENT 0 après coup, par construction. La bande
//    devient gratuite : le défaut n'est pas corrigé après coup, il est rendu
//    impossible à fabriquer.
//
// 2. LE PARPAING (440 × 2928, ratio 1:6,6 — « impossible pour un humain de lire »).
//    L'ancien code empilait chaque zone en UNE colonne sans plafond ; linearLayout
//    possède depuis toujours le `if (y + h > CAP) { col++; y = 0 }` qui manquait
//    ici. On replie donc chaque zone en couloirs, et on borne ce que l'humain LIT
//    (la colonne) et non la boîte englobante — voir SIGMA_MAX.
//
// Aucun PRNG, aucune graine, aucune vision, aucun LOD : `iterations` et `seed` ne
// concernent pas ce chemin et ne l'ont jamais concerné.

// LANEGAP/THEMEGAP/BUCKETGAP : les trois échelles de séparation, de la plus fine
// (deux couloirs d'une même zone) à la plus large (deux époques). Elles se
// distinguent à l'œil, et toutes dépassent le seuil de 20px du lint.
const LANEGAP = 40, THEMEGAP = 90, BUCKETGAP = 190, ROWGAP = 150, PAD = 30, PAD_V = 70;

// SIGMA_MAX = « aucune colonne plus de 3× plus haute que large » : la plainte de
// l'utilisateur, chiffrée, en UNE constante à sens humain direct.
//
// ⚠ Elle borne la COLONNE RÉELLEMENT LUE (hauteur de couloir / COL_WIDTH), pas la
// boîte de la zone. La différence n'est pas cosmétique : une boîte élargie par un
// couloir à moitié vide affiche un joli ratio pendant que le couloir 0, celui que
// l'œil descend, reste un parpaing. Borner la colonne rend la contrainte
// littéralement égale à la plainte — et la rend CLOSE : le plafond de couloir vaut
// SIGMA_MAX·COL_WIDTH, sans recherche ni bissection, et il s'applique du même coup
// à TOUTES les zones (frise comme bande), là où un critère de faisabilité par
// bissection n'était vérifié que sur la frise.
const SIGMA_MAX = 3;

/**
 * Layout CHRONOLOGIQUE : x = le temps, une seule rangée.
 *
 * Les zones datées par l'auteur sont le BORD FIXÉ ; les zones non datées reliées à
 * la frise sont placées là où leurs liens les tirent (extension harmonique) ;
 * celles que le graphe ne relie à rien restent hors du temps.
 */
function chronoLayout(args, board, path) {
  const anns = board.annotations ?? [];
  const notes = anns.filter((a) => a.type === "text" || a.type === "sticky");
  const membranes = anns.filter((a) => a.type === "membrane");
  const arrows = anns.filter((a) => a.type === "arrow");

  // Un axe est LIBRE, par définition : c'est un décor, pas une relation entre deux
  // idées. Sans ce test, `find` élirait la première flèche ATTACHÉE dont le texte
  // dit « la même année » — on écraserait ses coordonnées en pure perte (l'app
  // recalcule depuis les nœuds) et le vrai axe ne serait jamais repositionné.
  const axis = anns.find((a) => a.type === "arrow" && !a.sourceId && !a.targetId &&
    /chronolog|timeline|frise|le temps|ann[ée]e/i.test(a.text || ""));

  const zoneOf = zonesOfNotes(notes, membranes);
  const mById = new Map(membranes.map((m) => [m.id, m]));
  const isAxisLabel = (t) => { const s = (t || "").trim(); return s.length <= 15 && (/[←→]/.test(s) || (/\d{4}/.test(s) && s.replace(/[\s~←→]/g, "").length <= 5)); };

  // Hauteur de packing = la hauteur RENDUE (annBox) — celle que l'app dessine et
  // que le lint mesure. L'ancienne réserve (estimateLines·28+100) était une
  // TROISIÈME métrique, en désaccord avec les deux autres : elle faisait empiler
  // les notes sur ~20% de vide (mesuré sur le board de référence), puis borner ce
  // vide comme s'il était du contenu. Packer dans l'unité qu'on prétend borner est
  // la condition pour que SIGMA_MAX veuille dire quelque chose.
  const packH = (a) => annBox(a).h;

  // ── 1. LE GRAPHE DE SENS : notes (hors labels d'axe) + arêtes attachées ──────
  // L'axe libre n'a ni sourceId ni targetId → il n'entre jamais dans E, et c'est
  // cohérent : l'axe n'est pas une relation entre idées, c'est le référentiel.
  const V = notes.filter((n) => !isAxisLabel(n.text));
  const vset = new Set(V.map((n) => n.id));
  const adj = new Map(V.map((n) => [n.id, new Set()]));
  for (const a of arrows) {
    if (!a.sourceId || !a.targetId) continue;
    if (!vset.has(a.sourceId) || !vset.has(a.targetId)) continue;
    if (a.sourceId === a.targetId) continue; // boucle
    adj.get(a.sourceId).add(a.targetId);
    adj.get(a.targetId).add(a.sourceId);
  }

  // ── 2. PROBLÈME DE DIRICHLET SUR LE GRAPHE ──────────────────────────────────
  // Bord fixé D = les notes des zones datées par l'auteur (leur millésime est une
  // CONTRAINTE, jamais une inconnue : mathématiquement impossible à déplacer).
  // On minimise ½·Σ(t_u − t_v)² à bord fixé, dont le point critique est la
  // fonction HARMONIQUE : chaque note libre = moyenne exacte de ses voisins.
  const t = new Map(), D = new Set();
  for (const n of V) {
    const z = zoneOf.get(n.id);
    const y = z && mById.has(z) ? yearOf(mById.get(z).text) : null;
    if (y !== null) { t.set(n.id, y); D.add(n.id); }
  }
  const U = V.filter((n) => !D.has(n.id)).map((n) => n.id); // ordre document = ordre canonique
  const uset = new Set(U);

  // Composantes de G[U] et leur frontière ∂C. C'est ICI que le système sait
  // reconnaître son ignorance, par DÉFAUT DE RANG et non par un seuil :
  //  • ∂C ≠ ∅ → L_UU|_C est à diagonale dominante irréductiblement → définie
  //    positive → solution UNIQUE (et Gauss-Seidel converge inconditionnellement).
  //  • ∂C = ∅ → L_UU|_C est le laplacien de C, ker = les constantes → SINGULIÈRE
  //    → une infinité de solutions. Le graphe NE SAIT PAS ; on n'invente pas.
  //    C'est un résultat calculé, pas un échec.
  const seen = new Set(), solvable = new Set();
  let freeComps = 0;
  for (const id of U) {
    if (seen.has(id)) continue;
    const stack = [id], comp = []; seen.add(id);
    while (stack.length) {
      const v = stack.pop(); comp.push(v);
      for (const w of adj.get(v)) if (uset.has(w) && !seen.has(w)) { seen.add(w); stack.push(w); }
    }
    let anchored = false;
    for (const v of comp) for (const w of adj.get(v)) if (D.has(w)) anchored = true;
    if (anchored) for (const v of comp) solvable.add(v); else freeComps++;
  }

  // Gauss-Seidel, ordre document, arrêt sur la DONNÉE (jamais sur le temps ni sur
  // un compteur d'itérations) → reproductible bit à bit : uniquement +,−,×,÷
  // IEEE-754, dans le même ordre, aucun transcendantal, aucun PRNG.
  let sweeps = 0;
  if (D.size && solvable.size) {
    const mean = [...D].reduce((s, v) => s + t.get(v), 0) / D.size;
    for (const id of U) if (solvable.has(id)) t.set(id, mean);
    for (let s = 0; s < 10000; s++) {
      let maxd = 0;
      for (const v of U) {
        if (!solvable.has(v)) continue;
        const nb = [...adj.get(v)];
        if (!nb.length) continue;
        const nv = nb.reduce((acc, w) => acc + t.get(w), 0) / nb.length;
        maxd = Math.max(maxd, Math.abs(nv - t.get(v)));
        t.set(v, nv);
      }
      sweeps = s + 1;
      if (maxd < 1e-9) break;
    }
  }

  // ── 3. QUANTIFICATION : on garde les époques de l'AUTEUR ─────────────────────
  // Il écrit « ~2012 » : le tilde EST la quantification. On respecte sa propre
  // précision, on n'en fabrique pas une plus fine. Le principe du maximum
  // (min_∂C ≤ t ≤ max_∂C) garantit que toute valeur déduite tombe dans la période
  // écrite : aucune extrapolation n'est possible, par théorème. La sortie utile
  // étant un ENTIER (l'indice d'époque), un résidu de 1e-9 est ~10⁹ fois sous le
  // seuil de bascule : elle est prouvablement insensible au dernier bit.
  const years = [...new Set(membranes.map((m) => yearOf(m.text)).filter((y) => y !== null))].sort((a, b) => a - b);
  const bucketOf = (v) => {
    let b = 0, bd = Infinity;
    for (let j = 0; j < years.length; j++) {
      const d = Math.abs(v - years[j]);
      if (d < bd - 1e-12) { bd = d; b = j; } // ex æquo → j minimal (déterminisme)
    }
    return b;
  };

  // ── 4. OÙ VA CHAQUE ZONE ────────────────────────────────────────────────────
  // Une zone non datée rejoint la frise SSI le graphe la tire vers UNE SEULE
  // époque. Critère purement discret, zéro constante magique. Sinon — aucun
  // signal, ou des liens étalés sur plusieurs époques — elle reste hors du temps :
  // le silence du graphe est respecté comme un résultat, pas comblé par une
  // heuristique.
  const themes = [];
  for (const m of membranes) {
    const kids = V.filter((n) => zoneOf.get(n.id) === m.id);
    const y = yearOf(m.text);
    if (y !== null) { themes.push({ m, kids, bucket: bucketOf(y), onFrieze: true, dated: true }); continue; }
    const dd = kids.filter((k) => solvable.has(k.id)).map((k) => t.get(k.id)).sort((a, b) => a - b);
    if (!dd.length) { themes.push({ m, kids, bucket: null, onFrieze: false, dated: false }); continue; }
    const bks = [...new Set(dd.map(bucketOf))];
    themes.push({
      m, kids, bucket: bks.length === 1 ? bks[0] : null, onFrieze: bks.length === 1, dated: false,
      pull: dd[Math.ceil(dd.length / 2) - 1], // médiane basse : tombe sur une vraie valeur
    });
  }

  // ── 5. COULOIRS SERPENTINS À COLONNE BORNÉE ─────────────────────────────────
  // Plafond CLOS : SIGMA_MAX·COL_WIDTH, relevé à la plus haute note si une note
  // dépasse à elle seule (meilleur effort, jamais d'échec).
  const maxNote = V.length ? Math.max(...V.map(packH)) : 120;
  const H_CAP = Math.max(SIGMA_MAX * COL_WIDTH, maxNote);
  const packLanes = (kids, H) => {
    const lanes = [[]]; let y = 0;
    for (const n of kids) {
      const h = packH(n);
      if (y > 0 && y + h > H) { lanes.push([]); y = 0; } // le CAP que linearLayout a toujours eu
      lanes[lanes.length - 1].push({ n, y });
      y += h + LANEGAP;
    }
    return lanes;
  };
  const lanesHeight = (lanes) =>
    Math.max(0, ...lanes.map((L) => (L.length ? L[L.length - 1].y + packH(L[L.length - 1].n) : 0)));
  const packZone = (kids) => {
    // ÉQUILIBRAGE : le next-fit brut laisse un couloir plein et un moignon (un 2ᵉ
    // couloir rempli à 15% mesuré sur le board de référence) — la boîte s'élargit,
    // le ratio s'embellit, et la colonne lue reste un parpaing. À nombre de
    // couloirs FIXÉ, on cherche donc le plus PETIT plafond qui tient encore en
    // autant de couloirs : les couloirs s'égalisent, la zone se remplit, et la
    // colonne la plus haute ne peut que RÉTRÉCIR (H' ≤ H ⟹ max_couloir ≤ H').
    const k = packLanes(kids, H_CAP).length;
    let lo = Math.max(120, kids.length ? Math.max(...kids.map(packH)) : 120), hi = H_CAP, best = H_CAP;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (packLanes(kids, mid).length <= k) { best = mid; hi = mid - 1; } else lo = mid + 1;
    }
    const lanes = packLanes(kids, best);
    const h = Math.max(120, lanesHeight(lanes));
    // SERPENTIN — non cosmétique : replier une chaîne en k couloirs crée k−1
    // flèches de repli. Couloirs alignés en haut, chaque repli est une diagonale
    // qui retraverse toute la zone par-dessus les notes (obstruction, le défaut le
    // plus cher du lint). Miroir vertical exact du couloir impair → le repli
    // devient horizontal et court. Déterministe, sans paramètre.
    lanes.forEach((L, i) => { if (i % 2 === 1) for (const e of L) e.y = h - (e.y + packH(e.n)); });
    return { lanes, k: lanes.length, h };
  };
  const zoneW = (k) => k * COL_WIDTH + (k - 1) * LANEGAP + 2 * PAD;
  const zoneH = (h) => h + 2 * PAD_V;

  // ── 6. PLACEMENT — UNE SEULE RANGÉE, x = le temps ───────────────────────────
  const moves = [], zones = [];
  const onFrieze = themes.filter((z) => z.onFrieze);
  let x = 0, friezeH = 0, firstBucket = true;
  const marks = []; // repères époque → abscisse (pour dire la vérité sur l'échelle)
  for (let j = 0; j < years.length; j++) {
    const inB = onFrieze.filter((z) => z.bucket === j).sort((a, b) =>
      (a.dated ? 0 : 1) - (b.dated ? 0 : 1) || // la zone datée par l'auteur d'abord
      (a.m.x ?? 0) - (b.m.x ?? 0));            // puis l'ordre de l'auteur → idempotent
    if (!inB.length) continue;
    if (!firstBucket) x += BUCKETGAP;
    firstBucket = false;
    const x0 = x;
    for (let i = 0; i < inB.length; i++) {
      const pk = packZone(inB[i].kids);
      inB[i].pk = pk;
      pk.lanes.forEach((L, li) => {
        const lx = x + li * (COL_WIDTH + LANEGAP);
        for (const e of L) moves.push({ id: e.n.id, x: lx, y: e.y });
      });
      zones.push({
        x: x - PAD, y: -PAD_V, width: zoneW(pk.k), height: zoneH(pk.h), text: inB[i].m.text,
        // La couleur PORTE l'information « daté par l'auteur » vs « placé par ses
        // liens ». Une zone déduite ne reçoit PAS l'autorité graphique d'un
        // millésime écrit à la main : elle est peinte comme atemporelle et son
        // TEXTE reste intact. Écrire « · déduit @2012 » dans le titre gèlerait
        // l'inférence en donnée — yearOf la relirait au run suivant comme un
        // millésime d'auteur, et plus personne ne pourrait l'en distinguer. Le
        // document ne reçoit que ce que l'auteur a dit ; la déduction se recalcule
        // à chaque passage et se lit dans le rapport.
        color: inB[i].dated ? "#e9e9e9" : "#efefef",
      });
      friezeH = Math.max(friezeH, pk.h);
      x += zoneW(pk.k) - 2 * PAD + (i < inB.length - 1 ? THEMEGAP : 0);
    }
    marks.push({ year: years[j], cx: Math.round((x0 + x) / 2) });
  }
  // Calculé APRÈS la boucle : l'ancien code réaffectait timelineRight à chaque
  // itération et ne tenait que parce que la dernière colonne était la plus à
  // droite — un tri différent cassait silencieusement l'axe ET son label.
  const timelineRight = x + PAD;

  // ── 7. BANDE ATEMPORELLE — prouvablement 0 flèche vers la frise ──────────────
  // Sa distance ne coûte rien (théorème §1), et sa profondeur est désormais bornée
  // par H_CAP au lieu d'être une fonction non bornée du contenu.
  const bandTop = friezeH + PAD_V + ROWGAP;
  const row2y = bandTop + PAD_V;
  let bx = 0;
  const band = themes.filter((z) => !z.onFrieze).sort((a, b) => (a.m.x ?? 0) - (b.m.x ?? 0));
  for (const z of band) {
    const pk = packZone(z.kids);
    pk.lanes.forEach((L, li) => {
      const lx = bx + li * (COL_WIDTH + LANEGAP);
      for (const e of L) moves.push({ id: e.n.id, x: lx, y: row2y + e.y });
    });
    zones.push({ x: bx - PAD, y: bandTop, width: zoneW(pk.k), height: zoneH(pk.h), text: z.m.text, color: "#efefef" });
    bx += zoneW(pk.k) - 2 * PAD + THEMEGAP;
  }

  // ── 8. AXE + LABELS ─────────────────────────────────────────────────────────
  if (axis) moves.push({ id: axis.id, x: -40, y: -120, x2: timelineRight + 40, y2: -120 });
  const axisLabels = notes.filter((n) => isAxisLabel(n.text));
  for (const n of axisLabels) {
    // `years[0]` est gardé par years.length : l'ancien code faisait
    // Math.min(...[]) = Infinity quand aucune zone n'était datée, ce qui envoyait
    // TOUS les labels à gauche, empilés au même pixel. (maxYr était calculé et
    // jamais utilisé — supprimé.)
    const left = /←/.test(n.text || "") ||
      (yearOf(n.text) != null && years.length > 0 && yearOf(n.text) <= years[0] && !/→/.test(n.text || ""));
    moves.push({ id: n.id, x: left ? -280 : timelineRight + 60, y: -150 });
  }

  // Notes hors de toute zone → colonne au bout de la bande. Aucune membrane n'est
  // créée pour elles : inventer une zone leur donnerait un thème que l'auteur n'a
  // pas écrit. Elles restent visiblement non classées, ce qu'elles sont.
  const loose = V.filter((n) => zoneOf.get(n.id) === null);
  if (loose.length) {
    const pk = packZone(loose);
    pk.lanes.forEach((L, li) => {
      const lx = bx + li * (COL_WIDTH + LANEGAP);
      for (const e of L) moves.push({ id: e.n.id, x: lx, y: row2y + e.y });
    });
  }

  // ── 9. Flèches transverses → secondaires (courbes + fines) ──────────────────
  const datedIds = new Set(themes.filter((z) => z.dated).map((z) => z.m.id));
  const patches = [];
  for (const ar of arrows) {
    if (!ar.sourceId || !ar.targetId) continue;
    const zs = zoneOf.get(ar.sourceId), zt = zoneOf.get(ar.targetId);
    const isPartagent = /partagent/i.test(ar.text || "");
    const crossesTime = zs !== zt && (datedIds.has(zs) || datedIds.has(zt));
    if (isPartagent || crossesTime) patches.push({ id: ar.id, arrowType: "curved", strokeWidth: 1 });
  }

  const removeIds = membranes.map((m) => m.id);
  const res = toolApplyLayout({ path, outPath: args.outPath, overwrite: args.overwrite, boardId: board.id, removeIds, moves, zones, arrows: [], patches });

  // ── RAPPORT — dire ce qui a été DÉDUIT, et à quelle échelle ─────────────────
  const deduced = onFrieze.filter((z) => !z.dated);
  const R = [`\n(layout ⏳ CHRONOLOGIQUE — une seule rangée, x = le temps)`];
  R.push(`- ${themes.filter((z) => z.dated).length} zones datées par toi : ${years.join(" → ")}`);
  if (deduced.length)
    R.push(`- ${deduced.length} zone(s) placée(s) par leurs LIENS (déduit, non écrit dans le fichier — ` +
      `titre et couleur atemporelle inchangés) : ` +
      deduced.map((z) => `« ${(z.m.text || "").trim()} » → près de ${years[z.bucket]}`).join(", "));
  if (band.length)
    R.push(`- ${band.length} zone(s) hors du temps : ${band.map((z) => `« ${(z.m.text || "").trim()} »`).join(", ")}` +
      ` — aucune flèche ne les relie à la frise, leur distance ne coûte donc rien (0 arête à traverser).`);
  if (freeComps) R.push(`- ${freeComps} composante(s) sans aucune ancre : le graphe ne les date pas, on n'invente pas de date.`);
  if (sweeps) R.push(`- extension harmonique : ${sweeps} balayages, résidu < 1e-9 (déterministe, sans graine).`);
  const sigmas = [...onFrieze, ...band].filter((z) => z.pk).map((z) => z.pk.h / COL_WIDTH);
  if (sigmas.length)
    R.push(`- colonnes repliées à ${SIGMA_MAX}:1 max — pire colonne lue : 1:${Math.max(...sigmas).toFixed(2)} ` +
      `(plafond ${Math.round(H_CAP)}px).`);
  if (marks.length >= 2) {
    // HONNÊTETÉ D'ÉCHELLE : x est monotone (l'ordre des époques est exact) mais
    // NON MÉTRIQUE — le pas encode le VOLUME des zones, jamais la durée. Une
    // flèche longue ne dit donc PAS « beaucoup de temps franchi ». On publie la
    // dispersion px/an au lieu de laisser croire à une proportionnalité.
    const rates = [];
    for (let i = 1; i < marks.length; i++) {
      const dy = marks[i].year - marks[i - 1].year;
      if (dy > 0) rates.push((marks[i].cx - marks[i - 1].cx) / dy);
    }
    if (rates.length)
      R.push(`- ⚠ échelle NON métrique : ${Math.round(Math.min(...rates))} à ${Math.round(Math.max(...rates))} px/an ` +
        `selon l'époque (le pas suit le volume des zones). L'ORDRE est exact, la DISTANCE ne mesure pas la durée.`);
  }
  if (patches.length) R.push(`- ${patches.length} flèches transverses adoucies (courbes, fines).`);
  return res + R.join("\n");
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
        instructions: INSTRUCTIONS,
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
