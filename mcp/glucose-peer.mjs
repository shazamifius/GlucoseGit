#!/usr/bin/env node
// ─────────────────────────────────────────────────────────────────────────────
// glucose-peer — le « pair IA » de #3 (pont VIVANT, work in progress).
//
// Rejoint une session collab Glucose EN DIRECT via le même serveur public que
// l'app (`wss://sync.automerge.org`) et le même code `automerge:…`. Une fois
// connecté, lire = le doc courant, écrire = `handle.change()` → ça apparaît chez
// l'utilisateur en temps réel.
//
//   node mcp/glucose-peer.mjs <automerge:url>   → lit le doc vivant (Stage 1)
//   node mcp/glucose-peer.mjs                   → self-test host↔pair (connectivité)
// ─────────────────────────────────────────────────────────────────────────────
import { Repo } from "@automerge/automerge-repo";
import { WebSocketClientAdapter } from "@automerge/automerge-repo-network-websocket";
import { randomUUID } from "node:crypto";

const SERVER = "wss://sync.automerge.org";
const mkRepo = () => new Repo({ network: [new WebSocketClientAdapter(SERVER)] });
const nid = () => randomUUID().replace(/-/g, "").slice(0, 16);

// Garde-fou : jamais bloqué plus de 25 s.
const HARD = setTimeout(() => { console.error("⏱️  TIMEOUT (25 s)"); process.exit(2); }, 25000);

async function readLive(url, say) {
  const repo = mkRepo();
  const handle = await repo.find(url);
  const doc = await handle.doc();
  const notes = (doc?.boards ?? []).reduce((n, b) => n + (b.annotations?.length ?? 0), 0);
  console.log(JSON.stringify({ name: doc?.name, boards: (doc?.boards ?? []).length, annotations: notes }));

  if (say) {
    // Placement DANS la vue actuelle de l'utilisateur : le point-monde qui tombe
    // près du coin haut-gauche de son écran (transform = translate(vp) scale(vp)).
    const board = (doc.boards ?? []).find((b) => b.id === doc.activeBoardId) ?? doc.boards?.[0];
    const vp = board?.viewport ?? { x: 0, y: 0, scale: 1 };
    const wx = Math.round((120 - vp.x) / (vp.scale || 1));
    const wy = Math.round((120 - vp.y) / (vp.scale || 1));
    const note = {
      id: nid(), type: "sticky", x: wx, y: wy,
      text: say, width: 360, height: 130, fontSize: 14,
      bgColor: "#fde68a", color: "#1a1a1a",
    };
    const boardId = board.id;
    handle.change((d) => {
      const b = d.boards.find((bb) => bb.id === boardId) ?? d.boards[0];
      b.annotations.push(note);
      d.updatedAt = Date.now();
    });
    console.log(`✍️  note écrite EN DIRECT à (${wx}, ${wy}) — regarde ton canvas Glucose !`);
    await new Promise((r) => setTimeout(r, 4500)); // laisser la synchro partir vers le serveur
  }
  clearTimeout(HARD); process.exit(0);
}

async function selfTest() {
  const host = mkRepo();
  const h = host.create({
    name: "peer-selftest",
    boards: [{ id: "b1", annotations: [{ id: "n1", type: "text", x: 0, y: 0, text: "bonjour depuis l'hôte" }] }],
  });
  const url = h.url;
  console.error("hôte a publié :", url);
  await new Promise((r) => setTimeout(r, 3500)); // laisser le serveur propager

  const peer = mkRepo();
  const ph = await peer.find(url);
  const doc = await ph.doc();
  console.log("✅ le PAIR a lu le doc de l'hôte via wss://sync.automerge.org :",
    JSON.stringify({ name: doc?.name, texte: doc?.boards?.[0]?.annotations?.[0]?.text }));

  // Écriture croisée : le pair modifie → l'hôte doit voir le changement.
  ph.change((d) => { d.boards[0].annotations.push({ id: "n2", type: "text", x: 50, y: 50, text: "ajouté par le pair" }); });
  await new Promise((r) => setTimeout(r, 2500));
  const hostDoc = await h.doc();
  const seen = hostDoc?.boards?.[0]?.annotations?.length;
  console.log(seen === 2
    ? "✅ l'HÔTE a vu l'ajout du pair EN TEMPS RÉEL (co-présence prouvée)"
    : `⚠️ l'hôte voit ${seen} note(s) (sync incomplète ?)`);
  clearTimeout(HARD); process.exit(0);
}

const url = process.argv[2];
const sayIdx = process.argv.indexOf("--say");
const say = sayIdx >= 0 ? process.argv[sayIdx + 1] : null;
(url ? readLive(url, say) : selfTest()).catch((e) => { console.error("ERREUR:", e.message); process.exit(1); });
