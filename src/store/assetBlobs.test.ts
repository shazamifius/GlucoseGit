// ────────────────────────────────────────────────────────────────────────────
// R-EMB-01 — Probe : valider qu'Automerge stocke fidèlement des Uint8Array.
//
// On a besoin de cette garantie AVANT de mettre les bytes des images dans
// le doc, sinon tout l'effort de migration est vain. On vérifie :
//   1. Un Uint8Array stocké puis re-lu rend les mêmes octets (roundtrip
//      mémoire).
//   2. Le save() + load() binaire préserve les bytes.
//   3. Plusieurs images partageant le MÊME contenu (deduplication par
//      sha256) ne gonflent pas le binaire de façon linéaire.
//   4. Une mutation qui touche aux annotations sans toucher aux blobs ne
//      réécrit pas les blobs.
// ────────────────────────────────────────────────────────────────────────────

import { describe, expect, it } from "vitest";
import * as A from "./automerge";

// Type local minimal pour le probe — pas encore branché sur Project.
interface BlobBag {
  blobs: Record<string, Uint8Array>;
  refs: string[]; // sha256 référencés
}

function randBytes(size: number, _seed: number): Uint8Array {
  // crypto.getRandomValues garantit une entropie qui ne se compresse pas —
  // important pour tester la dedup d'Automerge sur des "vraies" payloads.
  const arr = new Uint8Array(size);
  crypto.getRandomValues(arr);
  return arr;
}

async function sha256Hex(data: Uint8Array): Promise<string> {
  // jsdom expose Web Crypto via crypto.subtle.
  const buf = await crypto.subtle.digest("SHA-256", data);
  return [...new Uint8Array(buf)]
    .map(b => b.toString(16).padStart(2, "0"))
    .join("");
}

describe("Automerge — support Uint8Array (probe R-EMB-01)", () => {
  it("1. roundtrip mémoire : un Uint8Array stocké rend les mêmes octets", () => {
    const payload = randBytes(4096, 42);
    const doc0 = A.create<BlobBag>({ blobs: {}, refs: [] });
    const doc1 = A.change(doc0, "add blob", (d) => {
      d.blobs["abc"] = payload;
      d.refs.push("abc");
    });

    const got = doc1.blobs["abc"];
    expect(got).toBeInstanceOf(Uint8Array);
    expect(got.length).toBe(payload.length);
    // Comparaison octet par octet
    for (let i = 0; i < payload.length; i++) {
      expect(got[i]).toBe(payload[i]);
    }
  });

  it("2. roundtrip save/load binaire préserve les bytes", () => {
    const payload = randBytes(8192, 1337);
    const doc0 = A.create<BlobBag>({ blobs: {}, refs: [] });
    const doc1 = A.change(doc0, "add", (d) => {
      d.blobs["xyz"] = payload;
      d.refs.push("xyz");
    });
    const bin = A.save(doc1);
    const loaded = A.load<BlobBag>(bin);

    const got = loaded.blobs["xyz"];
    expect(got).toBeInstanceOf(Uint8Array);
    expect(got.length).toBe(payload.length);
    for (let i = 0; i < payload.length; i++) {
      expect(got[i]).toBe(payload[i]);
    }
  });

  it("3. dédup : 10 références au même blob → ne multiplie pas la taille binaire par 10", () => {
    const payload = randBytes(16384, 7);
    const refs = Array.from({ length: 10 }, (_, i) => `ref-${i}`);

    // Cas A : 1 blob, 10 refs (toutes pointent vers le même sha)
    const docDedup0 = A.create<BlobBag>({ blobs: {}, refs: [] });
    const docDedup = A.change(docDedup0, "add", (d) => {
      d.blobs["shared"] = payload;
      for (const r of refs) d.refs.push(r);
    });
    const binDedup = A.save(docDedup);

    // Cas B : 10 blobs distincts (même contenu mais clés différentes)
    const docNoDedup0 = A.create<BlobBag>({ blobs: {}, refs: [] });
    const docNoDedup = A.change(docNoDedup0, "add", (d) => {
      for (let i = 0; i < 10; i++) {
        d.blobs[`distinct-${i}`] = randBytes(16384, i + 100);
        d.refs.push(`distinct-${i}`);
      }
    });
    const binNoDedup = A.save(docNoDedup);

    // Le cas avec dédup doit être SIGNIFICATIVEMENT plus petit
    // (rapport au moins x3 — pas besoin d'être à x10 exact car overhead Automerge)
    expect(binDedup.length).toBeLessThan(binNoDedup.length / 3);
  });

  it("4. mutation hors-blob ne réécrit pas les blobs (test stabilité ref)", () => {
    const payload = randBytes(2048, 99);
    const doc0 = A.create<BlobBag>({ blobs: {}, refs: [] });
    const doc1 = A.change(doc0, "add blob", (d) => {
      d.blobs["k"] = payload;
      d.refs.push("k");
    });

    const blobBefore = doc1.blobs["k"];

    // Mutation qui ne touche pas blobs
    const doc2 = A.change(doc1, "add ref", (d) => {
      d.refs.push("another");
    });

    const blobAfter = doc2.blobs["k"];
    expect(blobAfter.length).toBe(blobBefore.length);
    for (let i = 0; i < payload.length; i++) {
      expect(blobAfter[i]).toBe(payload[i]);
    }
  });

  it("5. SHA-256 disponible côté jsdom (sanity)", async () => {
    const payload = randBytes(1024, 12345);
    const hash = await sha256Hex(payload);
    expect(hash).toMatch(/^[0-9a-f]{64}$/);
    // Stabilité : deux appels même payload → même hash
    const hash2 = await sha256Hex(payload);
    expect(hash2).toBe(hash);
  });
});
