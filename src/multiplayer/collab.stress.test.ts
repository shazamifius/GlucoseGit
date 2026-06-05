// ────────────────────────────────────────────────────────────────────────────
// Stress co-working — des CENTAINES d'opérations sur deux instances reliées
// ────────────────────────────────────────────────────────────────────────────
//
// Objectif : prouver que TOUT type de modification (images, textes, stickies,
// flèches, dossiers, domaines, déplacements, resize, suppressions…) faite d'un
// côté arrive bien de l'autre, et que les deux documents convergent à
// l'identique — y compris sous des édits concurrents et en gros volume.
//
// On relie deux `Repo` automerge-repo par un MessageChannel (le vrai transport,
// en local) et on applique les mutations via `handle.change` — EXACTEMENT comme
// le store en collaboration (jamais d'`Automerge.change` brut sur le doc d'un
// handle : ça corromprait le WASM).

import { describe, expect, it } from "vitest";
import { Repo } from "@automerge/automerge-repo";
import type { DocHandle } from "@automerge/automerge-repo";
import { MessageChannelNetworkAdapter } from "@automerge/automerge-repo-network-messagechannel";
import * as A from "../store/automerge";
import type { Annotation, Board, BoardImage, Domain, Project } from "../types";

type H = DocHandle<Project>;

// ─────────── PRNG déterministe (mulberry32) ──────────────────────
function rng(seed: number) {
  let s = seed >>> 0;
  return () => {
    s |= 0; s = (s + 0x6d2b79f5) | 0;
    let t = Math.imul(s ^ (s >>> 15), 1 | s);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

// ─────────── Construction projet ─────────────────────────────────
function mkImage(id: string): BoardImage {
  return { id, x: 0, y: 0, width: 100, height: 100, rotation: 0, locked: false, tags: [], originalWidth: 100, originalHeight: 100 };
}
function mkProject(): Project {
  const board: Board = {
    id: "b1", name: "B", images: [], annotations: [], panels: [],
    viewport: { x: 0, y: 0, scale: 1 }, zones: [], folders: [], createdAt: 0, updatedAt: 0,
  };
  return { version: "2.0.0", name: "P", boards: [board], activeBoardId: "b1", presets: [], domains: [], createdAt: 0, updatedAt: 0 };
}

// ─────────── Connexion 2 pairs + helpers de sync ─────────────────
function connectPair() {
  const { port1, port2 } = new MessageChannel();
  const repoA = new Repo({ network: [new MessageChannelNetworkAdapter(port1)] });
  const repoB = new Repo({ network: [new MessageChannelNetworkAdapter(port2)] });
  return { repoA, repoB };
}
const headsOf = (h: H) => JSON.stringify(A.getHeads(h.doc() as unknown as A.Doc<Project>));
async function settle(hA: H, hB: H, timeout = 8000) {
  const start = Date.now();
  while (Date.now() - start < timeout) {
    if (headsOf(hA) === headsOf(hB)) return;
    await new Promise((r) => setTimeout(r, 5));
  }
  throw new Error("settle: les deux docs n'ont jamais convergé (heads différents)");
}
const b0 = (d: Project) => d.boards[0];

// ─────────── Générateur d'opérations (toutes faites via handle.change) ──
let counter = 0;
const ops: Array<(h: H, r: () => number) => void> = [
  // Ajouter une image
  (h) => h.change((d) => { b0(d).images.push(mkImage(`img-${counter++}`)); }, { message: "addImage" }),
  // Déplacer une image
  (h, r) => h.change((d) => { const a = b0(d).images; if (a.length) { const i = a[Math.floor(r() * a.length)]; i.x += Math.floor(r() * 40) - 20; i.y += Math.floor(r() * 40) - 20; } }, { message: "moveImage" }),
  // Redimensionner une image
  (h, r) => h.change((d) => { const a = b0(d).images; if (a.length) { const i = a[Math.floor(r() * a.length)]; const s = 40 + Math.floor(r() * 400); i.width = s; i.height = s; } }, { message: "resizeImage" }),
  // Supprimer une image
  (h, r) => h.change((d) => { const a = b0(d).images; if (a.length) a.splice(Math.floor(r() * a.length), 1); }, { message: "removeImage" }),
  // Ajouter un texte
  (h) => h.change((d) => { const ann: Annotation = { id: `txt-${counter++}`, type: "text", x: 0, y: 0, text: "hello" }; b0(d).annotations.push(ann); }, { message: "addText" }),
  // Éditer un texte
  (h, r) => h.change((d) => { const a = b0(d).annotations.filter((x) => x.type === "text"); if (a.length) { const t = a[Math.floor(r() * a.length)] as Annotation & { text: string }; t.text = `edit-${counter++}`; } }, { message: "editText" }),
  // Ajouter un sticky
  (h) => h.change((d) => { const ann: Annotation = { id: `stk-${counter++}`, type: "sticky", x: 5, y: 5, text: "note", bgColor: "#f5c542" }; b0(d).annotations.push(ann); }, { message: "addSticky" }),
  // Ajouter une flèche
  (h) => h.change((d) => { const ann: Annotation = { id: `arr-${counter++}`, type: "arrow", x: 0, y: 0, x2: 100, y2: 100 }; b0(d).annotations.push(ann); }, { message: "addArrow" }),
  // Déplacer une annotation
  (h, r) => h.change((d) => { const a = b0(d).annotations; if (a.length) { const an = a[Math.floor(r() * a.length)]; an.x += Math.floor(r() * 30) - 15; an.y += Math.floor(r() * 30) - 15; } }, { message: "moveAnn" }),
  // Supprimer une annotation
  (h, r) => h.change((d) => { const a = b0(d).annotations; if (a.length) a.splice(Math.floor(r() * a.length), 1); }, { message: "removeAnn" }),
  // Ajouter un domaine
  (h) => h.change((d) => { if (!d.domains) d.domains = []; const dom: Domain = { id: `dom-${counter++}`, name: "D", color: "#60a5fa", icon: "★", createdAt: 0 }; d.domains.push(dom); }, { message: "addDomain" }),
  // Renommer le projet
  (h) => h.change((d) => { d.name = `proj-${counter++}`; }, { message: "rename" }),
];

describe("stress co-working — convergence sous des centaines d'opérations", () => {
  it("200 opérations aléatoires alternées entre A et B → docs strictement identiques", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();

    const r = rng(12345);
    for (let i = 0; i < 200; i++) {
      const target = r() < 0.5 ? hA : hB;
      ops[Math.floor(r() * ops.length)](target, r);
      // De temps en temps on laisse le réseau se synchroniser (simule des pauses).
      if (i % 17 === 0) await settle(hA, hB);
    }
    await settle(hA, hB);

    expect(headsOf(hA)).toBe(headsOf(hB));
    expect(A.asPlain(hA.doc() as A.Doc<Project>)).toEqual(A.asPlain(hB.doc() as A.Doc<Project>));
  });

  it("rafale rapide (100 déplacements d'affilée chez A) → aucune perte chez B", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();

    hA.change((d) => { b0(d).images.push(mkImage("hero")); }, { message: "add" });
    // 100 micro-déplacements sans pause (comme un drag de 100 frames).
    for (let i = 0; i < 100; i++) hA.change((d) => { b0(d).images[0].x += 1; }, { message: "drag" });
    await settle(hA, hB);

    expect(b0(hB.doc()).images[0].x).toBe(100); // 100 incréments tous arrivés
    expect(headsOf(hA)).toBe(headsOf(hB));
  });

  it("édits CONCURRENTS sur la MÊME image (avant sync) → fusion sans crash, convergence", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();
    hA.change((d) => { b0(d).images.push(mkImage("x")); }, { message: "add" });
    await settle(hA, hB);

    // Les deux modifient la même image "en même temps", chacun de son côté.
    hA.change((d) => { b0(d).images[0].width = 500; }, { message: "A" });
    hB.change((d) => { b0(d).images[0].x = 999; }, { message: "B" });
    await settle(hA, hB);

    // Convergence : les deux changements coexistent (axes différents), aucune perte.
    expect(b0(hA.doc()).images[0].x).toBe(999);
    expect(b0(hA.doc()).images[0].width).toBe(500);
    expect(headsOf(hA)).toBe(headsOf(hB));
  });

  it("suppression concurrente de la même image → pas de doublon ni de crash", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();
    hA.change((d) => { b0(d).images.push(mkImage("y")); }, { message: "add" });
    await settle(hA, hB);

    hA.change((d) => { const a = b0(d).images; const k = a.findIndex((i) => i.id === "y"); if (k >= 0) a.splice(k, 1); }, { message: "delA" });
    hB.change((d) => { const a = b0(d).images; const k = a.findIndex((i) => i.id === "y"); if (k >= 0) a.splice(k, 1); }, { message: "delB" });
    await settle(hA, hB);

    expect(b0(hA.doc()).images.filter((i) => i.id === "y").length).toBe(0);
    expect(headsOf(hA)).toBe(headsOf(hB));
  });

  it("trois vagues de 50 opérations avec resync entre chaque → toujours convergent", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();

    const r = rng(999);
    for (let wave = 0; wave < 3; wave++) {
      for (let i = 0; i < 50; i++) {
        const target = r() < 0.5 ? hA : hB;
        ops[Math.floor(r() * ops.length)](target, r);
      }
      await settle(hA, hB);
      expect(headsOf(hA)).toBe(headsOf(hB));
    }
    expect(A.asPlain(hA.doc() as A.Doc<Project>)).toEqual(A.asPlain(hB.doc() as A.Doc<Project>));
  });
});
