// ────────────────────────────────────────────────────────────────────────────
// Canal d'assets — test d'intégration DEUX instances (MessageChannel)
// ────────────────────────────────────────────────────────────────────────────
//
// On ne peut pas lancer deux fenêtres Tauri ici. On relie deux `Repo` par un
// MessageChannel (exactement le transport, en local) et on prouve empiriquement
// le cœur du fix « images invisibles en collab » :
//
//   1. l'hôte A publie les octets d'une image dans un CANAL séparé (2ᵉ doc) ;
//   2. le pair B ouvre ce canal par son URL et MATÉRIALISE les octets à l'identique ;
//   3. bytes bit-à-bit intacts (aucune corruption) ;
//   4. mémoïsation : re-matérialiser ne réécrit pas deux fois ;
//   5. symétrie : ce que B ajoute, A le reçoit ;
//   6. collect : les noms `asset:<x>` du projet sont bien extraits (asset link + src).
//
// L'I/O disque (Tauri) est remplacée par un « disque » en mémoire (Map) — c'est
// exactement ce qu'injectent les adaptateurs réels `loadLocalAsset`/`writeLocalAsset`.

import { describe, expect, it } from "vitest";
import { Repo } from "@automerge/automerge-repo";
import { MessageChannelNetworkAdapter } from "@automerge/automerge-repo-network-messagechannel";
import {
  type AssetBlob,
  type AssetChannelDoc,
  collectAssetNames,
  materializeFromChannel,
  publishToChannel,
} from "./assetChannel";

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

/** Un « disque » en mémoire = le magasin content-addressed local, façon Map. */
function memDisk(initial: Record<string, AssetBlob> = {}) {
  const files = new Map<string, AssetBlob>(Object.entries(initial));
  return {
    files,
    load: async (name: string): Promise<AssetBlob | null> => files.get(name) ?? null,
    write: async (name: string, blob: AssetBlob): Promise<void> => {
      // Copie défensive : les bytes lus depuis le doc distant sont un proxy.
      files.set(name, { mime: blob.mime, bytes: Uint8Array.from(blob.bytes) });
    },
  };
}

describe("canal d'assets — deux instances reliées", () => {
  it("A publie une image → B ouvre le canal par URL et matérialise les octets intacts", async () => {
    const { repoA, repoB } = connectPair();

    // A a l'image « photo.png » sur son disque ; B ne l'a pas.
    const bytes = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0xde, 0xad]);
    const diskA = memDisk({ "abc123.png": { mime: "image/png", bytes } });
    const diskB = memDisk();

    // Hôte : crée le canal et y publie ce qu'il a.
    const chA = repoA.create<AssetChannelDoc>({ blobs: {} });
    const publishedCount = await publishToChannel(chA, ["abc123.png"], diskA.load);
    expect(publishedCount).toBe(1);

    // Pair : ouvre le MÊME canal par son URL (catch-up réseau) puis matérialise.
    const chB = await repoB.find<AssetChannelDoc>(chA.url);
    await chB.whenReady();
    await waitUntil(() => !!chB.doc()?.blobs?.["abc123.png"]);

    const seenB = new Set<string>();
    const wrote = await materializeFromChannel(chB, seenB, diskB.write);
    expect(wrote).toBe(1);

    // Les octets sont arrivés chez B, bit-à-bit.
    const got = diskB.files.get("abc123.png");
    expect(got).toBeDefined();
    expect(got!.mime).toBe("image/png");
    expect(Array.from(got!.bytes)).toEqual(Array.from(bytes));
  });

  it("mémoïsation : re-matérialiser le même canal ne réécrit pas deux fois", async () => {
    const { repoA, repoB } = connectPair();
    const diskA = memDisk({ "x.jpg": { mime: "image/jpeg", bytes: new Uint8Array([1, 2, 3]) } });
    const diskB = memDisk();

    const chA = repoA.create<AssetChannelDoc>({ blobs: {} });
    await publishToChannel(chA, ["x.jpg"], diskA.load);
    const chB = await repoB.find<AssetChannelDoc>(chA.url);
    await chB.whenReady();
    await waitUntil(() => !!chB.doc()?.blobs?.["x.jpg"]);

    const seen = new Set<string>();
    expect(await materializeFromChannel(chB, seen, diskB.write)).toBe(1);
    // Deuxième passe : rien de neuf (déjà vu) → 0 écriture.
    expect(await materializeFromChannel(chB, seen, diskB.write)).toBe(0);
  });

  it("ne publie pas ce qu'on n'a pas sur son disque (l'autre pair s'en chargera)", async () => {
    const { repoA } = connectPair();
    const diskA = memDisk(); // A n'a AUCUN octet
    const chA = repoA.create<AssetChannelDoc>({ blobs: {} });
    const published = await publishToChannel(chA, ["manque.png"], diskA.load);
    expect(published).toBe(0);
    expect(chA.doc()?.blobs?.["manque.png"]).toBeUndefined();
  });

  it("symétrie : ce que B publie, A le matérialise", async () => {
    const { repoA, repoB } = connectPair();
    const bytesB = new Uint8Array([0xaa, 0xbb, 0xcc, 0xdd]);
    const diskB = memDisk({ "fromB.webp": { mime: "image/webp", bytes: bytesB } });
    const diskA = memDisk();

    // Le canal est créé par A, mais c'est B qui publie dedans.
    const chA = repoA.create<AssetChannelDoc>({ blobs: {} });
    const chB = await repoB.find<AssetChannelDoc>(chA.url);
    await chB.whenReady();

    await publishToChannel(chB, ["fromB.webp"], diskB.load);
    await waitUntil(() => !!chA.doc()?.blobs?.["fromB.webp"]);

    const wrote = await materializeFromChannel(chA, new Set(), diskA.write);
    expect(wrote).toBe(1);
    expect(Array.from(diskA.files.get("fromB.webp")!.bytes)).toEqual(Array.from(bytesB));
  });

  it("publier deux fois le même nom est idempotent (pas d'écrasement)", async () => {
    const { repoA } = connectPair();
    const diskA = memDisk({ "dup.png": { mime: "image/png", bytes: new Uint8Array([9, 9]) } });
    const chA = repoA.create<AssetChannelDoc>({ blobs: {} });
    expect(await publishToChannel(chA, ["dup.png"], diskA.load)).toBe(1);
    // 2ᵉ appel : déjà dans le canal → 0 publication.
    expect(await publishToChannel(chA, ["dup.png"], diskA.load)).toBe(0);
  });
});

describe("collectAssetNames — extraction des références", () => {
  it("récupère les noms depuis asset(link) ET src, dédupliqués", () => {
    const project = {
      boards: [
        {
          images: [
            { asset: { mode: "link", href: "asset:a.png" } },
            { src: "asset:b.jpg" },
            { asset: { mode: "link", href: "asset:a.png" } }, // doublon
            { src: "data:image/png;base64,xxxx" }, // ignoré (pas asset:)
            { asset: { mode: "embed", sha256: "z" } }, // ignoré (embed, pas de fichier)
          ],
        },
        { images: [{ asset: { mode: "link", href: "asset:c.webp" } }] },
      ],
    };
    const names = collectAssetNames(project).sort();
    expect(names).toEqual(["a.png", "b.jpg", "c.webp"]);
  });

  it("projet vide → aucun nom", () => {
    expect(collectAssetNames({ boards: [] })).toEqual([]);
  });
});
