// ────────────────────────────────────────────────────────────────────────────
// Git #1 Phase 4 p2 — Cœur PUR de la compaction (vrai Automerge, aucune I/O).
// Le roundtrip est LA garantie béton : compacter → recharger → état identique.
// ────────────────────────────────────────────────────────────────────────────

import { describe, expect, it } from "vitest";
import * as A from "../store/automerge";
import type { Project } from "../types";
import { compactDoc, CompactionError } from "./compaction";

function mkDoc(): A.Doc<Project> {
  return A.create<Project>({
    version: "2.0.0",
    name: "projet",
    boards: [{
      id: "b0", name: "B0", images: [], annotations: [], panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0,
    }],
    activeBoardId: "b0", presets: [], domains: [], createdAt: 0, updatedAt: 0,
  });
}

/** Applique `n` petites éditions RÉPÉTÉES sur les MÊMES données (l'état reste
 *  petit, mais l'historique d'ops gonfle) — le cas où la compaction gagne le plus. */
function withHistory(doc: A.Doc<Project>, n: number): A.Doc<Project> {
  let d = doc;
  d = A.change(d, "add", (p) => {
    p.boards[0].annotations.push({
      id: "a0", type: "text", content: "x", x: 0, y: 0, width: 100, height: 40,
    } as unknown as Project["boards"][number]["annotations"][number]);
  });
  for (let i = 0; i < n; i++) {
    d = A.change(d, "move", (p) => {
      const a = p.boards[0].annotations[0] as unknown as { x: number; y: number; content: string };
      a.x = i;
      a.y = i * 2;
      a.content = `edit ${i}`;
    });
  }
  return d;
}

describe("compactDoc — cœur pur", () => {
  it("roundtrip : l'état rechargé du compacté est IDENTIQUE à l'original", () => {
    const doc = withHistory(mkDoc(), 300);
    const { compacted, bytes } = compactDoc(doc);

    const reloaded = A.loadResilient<Project>(bytes).doc;
    expect(A.asPlain(reloaded)).toEqual(A.asPlain(doc));
    expect(A.asPlain(compacted)).toEqual(A.asPlain(doc));
  });

  it("aplatit l'historique fin à un unique change", () => {
    const doc = withHistory(mkDoc(), 300);
    expect(A.history(doc).length).toBeGreaterThan(300);

    const { compacted } = compactDoc(doc);
    expect(A.history(compacted).length).toBe(1); // le seul change « init »
  });

  it("allège réellement le fichier (moins d'octets que l'original)", () => {
    const doc = withHistory(mkDoc(), 400);
    const originalSize = A.save(doc).length;
    const { bytes } = compactDoc(doc);
    expect(bytes.length).toBeLessThan(originalSize);
  });

  it("préserve les blobs (Uint8Array) octet pour octet", () => {
    const payload = new Uint8Array(512);
    for (let i = 0; i < payload.length; i++) payload[i] = (i * 7 + 13) & 0xff;

    let doc = mkDoc();
    doc = A.change(doc, "blob", (p) => {
      (p as Project).blobs = { "sha-1": payload };
    });
    doc = withHistory(doc, 50);

    const { compacted } = compactDoc(doc);
    const blobs = (A.asPlain(compacted) as Project).blobs!;
    expect(blobs["sha-1"]).toBeInstanceOf(Uint8Array);
    expect(Array.from(blobs["sha-1"])).toEqual(Array.from(payload));
  });

  it("un doc déjà compact (1 change) reste identique après compaction", () => {
    const doc = mkDoc();
    const { compacted } = compactDoc(doc);
    expect(A.asPlain(compacted)).toEqual(A.asPlain(doc));
    expect(A.history(compacted).length).toBe(1);
  });

  it("CompactionError est bien exportée (garde-fou appelant)", () => {
    // Le roundtrip d'un état sain ne jette jamais ; on vérifie juste que la classe
    // d'erreur existe pour que l'appelant puisse discriminer (instanceof).
    expect(new CompactionError("x")).toBeInstanceOf(Error);
    expect(new CompactionError("x").name).toBe("CompactionError");
  });
});
