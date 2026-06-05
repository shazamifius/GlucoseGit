// ────────────────────────────────────────────────────────────────────────────
// Tests d'intégration co-working — DEUX instances reliées par un MessageChannel
// ────────────────────────────────────────────────────────────────────────────
//
// On ne peut pas lancer deux fenêtres Tauri ici, mais on peut relier deux `Repo`
// automerge-repo par un MessageChannel (exactement le transport, juste en local)
// et rejouer les opérations que fait le pont. Ça prouve empiriquement :
//   - convergence dans les deux sens,
//   - catch-up complet quand un pair rejoint après coup,
//   - un resize chez A se retrouve à l'identique chez B (et RIEN d'autre ne bouge),
//   - les édits concurrents fusionnent sans perte,
//   - le coalescing d'un geste (un seul push) converge bien vers l'état final,
//   - la CAMÉRA n'entre jamais dans le document synchronisé.

import { afterEach, describe, expect, it } from "vitest";
import { Repo } from "@automerge/automerge-repo";
import { MessageChannelNetworkAdapter } from "@automerge/automerge-repo-network-messagechannel";
import * as A from "../store/automerge";
import { useGlucoseStore, getActiveBoard } from "../store";
import { setCollabHandle } from "./collabHandle";
import type { Board, BoardImage, Project } from "../types";

// ─────────── Helpers ─────────────────────────────────────────────
function mkImage(id: string, x = 0, y = 0, w = 100, h = 100): BoardImage {
  return { id, x, y, width: w, height: h, rotation: 0, locked: false, tags: [], originalWidth: w, originalHeight: h };
}
function mkProject(): Project {
  const board: Board = {
    id: "b1", name: "B",
    images: [mkImage("img1"), mkImage("img2", 200, 0)],
    annotations: [], panels: [], viewport: { x: 0, y: 0, scale: 1 },
    zones: [], folders: [], createdAt: 0, updatedAt: 0,
  };
  return { version: "2.0.0", name: "P", boards: [board], activeBoardId: "b1", presets: [], domains: [], createdAt: 0, updatedAt: 0 };
}
function connectPair() {
  const { port1, port2 } = new MessageChannel();
  const repoA = new Repo({ network: [new MessageChannelNetworkAdapter(port1)] });
  const repoB = new Repo({ network: [new MessageChannelNetworkAdapter(port2)] });
  return { repoA, repoB };
}
async function waitUntil(fn: () => boolean, timeout = 3000): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeout) {
    if (fn()) return;
    await new Promise((r) => setTimeout(r, 10));
  }
  throw new Error("waitUntil: condition jamais satisfaite (timeout)");
}
const img = (doc: Project, id: string) => doc.boards[0].images.find((i) => i.id === id);

// ─────────── Sync deux instances ─────────────────────────────────
describe("co-working — deux instances reliées", () => {
  it("catch-up : un pair qui rejoint après coup récupère TOUT l'état", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    hA.change((d) => { img(d, "img1")!.x = 42; }); // une modif avant que B arrive
    await hA.whenReady();

    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();

    expect(hB.doc().boards[0].images.length).toBe(2);
    expect(img(hB.doc(), "img1")!.x).toBe(42);
  });

  it("convergence bidirectionnelle", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();

    hA.change((d) => { img(d, "img1")!.x = 111; });
    await waitUntil(() => img(hB.doc(), "img1")!.x === 111);

    hB.change((d) => { img(d, "img2")!.y = 222; });
    await waitUntil(() => img(hA.doc(), "img2")!.y === 222);

    expect(img(hA.doc(), "img1")!.x).toBe(111);
    expect(img(hB.doc(), "img2")!.y).toBe(222);
  });

  it("resize chez A → B voit la même taille, et RIEN d'autre ne bouge", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();

    hA.change((d) => { const i = img(d, "img1")!; i.width = 300; i.height = 300; });
    await waitUntil(() => img(hB.doc(), "img1")!.width === 300);

    expect(img(hB.doc(), "img1")!.width).toBe(300);
    expect(img(hB.doc(), "img1")!.height).toBe(300);
    // img2 ne doit PAS avoir bougé (pas de resize parasite / écho).
    expect(img(hB.doc(), "img2")!.width).toBe(100);
    expect(img(hB.doc(), "img2")!.x).toBe(200);
  });

  it("édits concurrents (avant synchro) → fusion sans perte", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();

    // Les deux éditent "en même temps" des images différentes.
    hA.change((d) => { img(d, "img1")!.x = 50; });
    hB.change((d) => { img(d, "img2")!.x = 250; });

    await waitUntil(() =>
      img(hA.doc(), "img2")!.x === 250 && img(hB.doc(), "img1")!.x === 50);

    // Convergence identique des deux côtés, aucune perte.
    expect(img(hA.doc(), "img1")!.x).toBe(50);
    expect(img(hA.doc(), "img2")!.x).toBe(250);
    expect(img(hB.doc(), "img1")!.x).toBe(50);
    expect(img(hB.doc(), "img2")!.x).toBe(250);
  });

  it("coalescing d'un geste : un seul push consolidé → B converge vers l'état final", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();

    // Simule un drag/resize : N frames appliquées LOCALEMENT, puis UN push net
    // (exactement ce que fait endLiveEdit en collab).
    const base = hA.doc();
    let local = base;
    for (let w = 110; w <= 200; w += 10) {
      local = A.change(local as unknown as A.Doc<Project>, "frame", (d) => { img(d, "img1")!.width = w; }) as never;
    }
    const changes = A.getChanges(base as unknown as A.Doc<Project>, local as unknown as A.Doc<Project>);
    hA.update((d) => A.applyChanges(A.clone(d as unknown as A.Doc<Project>), changes) as never);

    await waitUntil(() => img(hB.doc(), "img1")!.width === 200);
    expect(img(hB.doc(), "img1")!.width).toBe(200);
    // Heads identiques → états strictement convergents.
    expect(A.getHeads(hA.doc() as unknown as A.Doc<Project>))
      .toEqual(A.getHeads(hB.doc() as unknown as A.Doc<Project>));
  });
});

// ─────────── Caméra locale (jamais synchronisée) ─────────────────
describe("co-working — la caméra reste locale", () => {
  afterEach(() => setCollabHandle(null));

  it("solo : setViewport écrit bien la caméra dans le doc (sauvegarde fichier)", () => {
    setCollabHandle(null);
    const id = getActiveBoard(useGlucoseStore.getState().project).id;
    useGlucoseStore.getState().setViewport(id, { x: 10, y: 20, scale: 2 });
    expect(getActiveBoard(useGlucoseStore.getState().project).viewport).toEqual({ x: 10, y: 20, scale: 2 });
    expect(useGlucoseStore.getState().getViewport(id)).toEqual({ x: 10, y: 20, scale: 2 });
  });

  it("collab : setViewport NE touche PAS le doc partagé, mais bouge la caméra locale", () => {
    const id = getActiveBoard(useGlucoseStore.getState().project).id;
    const docViewportBefore = { ...getActiveBoard(useGlucoseStore.getState().project).viewport };
    // Handle factice : suffit à faire croire au store qu'on est en collab.
    setCollabHandle({} as never);
    useGlucoseStore.getState().setViewport(id, { x: 99, y: 88, scale: 3 });
    // Le doc (donc le réseau) n'a PAS bougé → la caméra de l'autre ne bouge pas.
    expect(getActiveBoard(useGlucoseStore.getState().project).viewport).toEqual(docViewportBefore);
    // Mais MA caméra locale, elle, a bien changé.
    expect(useGlucoseStore.getState().getViewport(id)).toEqual({ x: 99, y: 88, scale: 3 });
  });
});

// ─────────── Reconnexion : le lien survit + fusion sans perte ────
describe("co-working — reconnexion depuis un fichier (lien embarqué)", () => {
  it("fusionne les édits HORS-LIGNE de A avec les édits DISTANTS de B, aucune perte", () => {
    // Base partagée = état de la chaîne au moment où A se déconnecte (son Ctrl+S).
    let base = A.create<Project>(mkProject());
    base = A.change(base, "shared", (d) => { d.boards[0].images.push(mkImage("shared")); });
    const fileBytes = A.save(base); // ce que A a sur son disque

    // B reste 4h : son état (poussé au serveur).
    let serverDoc = A.load<Project>(fileBytes);
    serverDoc = A.change(serverDoc, "B", (d) => {
      img(d, "shared")!.x = 500;
      d.boards[0].images.push(mkImage("byB", 10, 10));
    });

    // A rouvre SON fichier et édite avant que la reconnexion aboutisse (hors-ligne).
    let localDoc = A.load<Project>(fileBytes);
    localDoc = A.change(localDoc, "A offline", (d) => {
      img(d, "shared")!.width = 777;
      d.boards[0].images.push(mkImage("byA", 20, 20));
    });

    // Reconnexion : exactement ce que fait reconnectFromDoc → merge(local → serveur).
    const merged = A.merge(A.clone(serverDoc), localDoc);

    // Les deux mondes coexistent, rien n'est perdu.
    expect(img(merged, "shared")!.x).toBe(500);     // édit distant B
    expect(img(merged, "shared")!.width).toBe(777); // édit hors-ligne A
    expect(merged.boards[0].images.some((i) => i.id === "byB")).toBe(true);
    expect(merged.boards[0].images.some((i) => i.id === "byA")).toBe(true);
  });

  it("le collabUrl survit à un save/load (Ctrl+S puis réouverture)", () => {
    let doc = A.create<Project>(mkProject());
    doc = A.change(doc, "link", (d) => { d.collabUrl = "automerge:testLink123"; });
    const reopened = A.load<Project>(A.save(doc));
    expect(reopened.collabUrl).toBe("automerge:testLink123");
  });
});

// ─────────── Blobs embed en collaboration ────────────────────────
describe("co-working — les blobs embed se synchronisent entre pairs", () => {
  it("A ajoute une image embed → B reçoit les bytes intacts", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();

    // A ajoute une image avec AssetRef embed + blob dans project.blobs
    const fakeBytes = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
    const sha = "test-sha-abc123";
    hA.change((d) => {
      d.boards[0].images.push({
        id: "embed-img", x: 0, y: 0, width: 100, height: 100,
        rotation: 0, locked: false, tags: [],
        originalWidth: 100, originalHeight: 100,
        asset: { mode: "embed", sha256: sha, mime: "image/png" },
      } as BoardImage);
      if (!d.blobs) (d as Project).blobs = {};
      d.blobs![sha] = fakeBytes;
    });

    // Attend que B voie l'image
    await waitUntil(() => hB.doc().boards[0].images.some((i) => i.id === "embed-img"));

    const bImg = hB.doc().boards[0].images.find((i) => i.id === "embed-img")!;
    expect(bImg.asset?.mode).toBe("embed");
    if (bImg.asset?.mode === "embed") {
      expect(bImg.asset.sha256).toBe(sha);
    }

    // Les bytes doivent être arrivés chez B
    const bBlobs = hB.doc().blobs;
    expect(bBlobs).toBeDefined();
    expect(bBlobs?.[sha]).toBeDefined();
    const received = bBlobs![sha];
    expect(received).toBeInstanceOf(Uint8Array);
    expect(received.length).toBe(fakeBytes.length);
    for (let i = 0; i < fakeBytes.length; i++) {
      expect(received[i]).toBe(fakeBytes[i]);
    }
  });

  it("deux pairs ajoutent des images embed concurrentes → fusion sans perte de blobs", async () => {
    const { repoA, repoB } = connectPair();
    const hA = repoA.create<Project>(mkProject());
    // Initialise blobs AVANT que B rejoigne — en vrai, la première image
    // crée la map ; ici on s'assure qu'elle existe pour les deux peers.
    hA.change((d) => { (d as Project).blobs = {}; });
    const hB = await repoB.find<Project>(hA.url);
    await hB.whenReady();

    const bytesA = new Uint8Array([0x01, 0x02, 0x03, 0x04]);
    const bytesB = new Uint8Array([0xaa, 0xbb, 0xcc, 0xdd]);

    // A et B ajoutent chacun une image embed en même temps
    hA.change((d) => {
      d.boards[0].images.push({
        id: "from-a", x: 0, y: 0, width: 50, height: 50,
        rotation: 0, locked: false, tags: [],
        originalWidth: 50, originalHeight: 50,
        asset: { mode: "embed", sha256: "sha-a", mime: "image/png" },
      } as BoardImage);
      d.blobs!["sha-a"] = bytesA;
    });
    hB.change((d) => {
      d.boards[0].images.push({
        id: "from-b", x: 100, y: 0, width: 50, height: 50,
        rotation: 0, locked: false, tags: [],
        originalWidth: 50, originalHeight: 50,
        asset: { mode: "embed", sha256: "sha-b", mime: "image/jpeg" },
      } as BoardImage);
      d.blobs!["sha-b"] = bytesB;
    });

    // Convergence : les deux doivent avoir les deux images ET les deux blobs
    await waitUntil(() =>
      hA.doc().boards[0].images.some((i) => i.id === "from-b") &&
      hB.doc().boards[0].images.some((i) => i.id === "from-a")
    );

    expect(hA.doc().blobs?.["sha-a"]).toBeDefined();
    expect(hA.doc().blobs?.["sha-b"]).toBeDefined();
    expect(hB.doc().blobs?.["sha-a"]).toBeDefined();
    expect(hB.doc().blobs?.["sha-b"]).toBeDefined();
  });
});
