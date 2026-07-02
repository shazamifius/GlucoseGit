// ────────────────────────────────────────────────────────────────────────────
// SAVE-A — Tests de l'enregistrement incrémental.
//
// Le test CRITIQUE : un fichier = [save complet] ++ [deltas ajoutés] doit se
// recharger en un document IDENTIQUE via A.load(). Si ça casse, on corrompt les
// .glucose des utilisateurs → priorité absolue.
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it } from "vitest";
import * as A from "../store/automerge";
import type { Project } from "../types";
import {
  planSave, commitSave, markLoaded, resetSaveState, _peekBaseline,
} from "./saveState";

function mkDoc(): A.Doc<Project> {
  return A.create<Project>({
    version: "2.0.0",
    name: "init",
    boards: [{
      id: "b0", name: "B0",
      images: [], annotations: [], panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 },
      createdAt: 0, updatedAt: 0,
    }],
    activeBoardId: "b0",
    presets: [], domains: [],
    createdAt: 0, updatedAt: 0,
  });
}

function concat(chunks: Uint8Array[]): Uint8Array {
  let total = 0;
  for (const c of chunks) total += c.length;
  const out = new Uint8Array(total);
  let off = 0;
  for (const c of chunks) { out.set(c, off); off += c.length; }
  return out;
}

beforeEach(() => resetSaveState());

describe("saveState — enregistrement incrémental", () => {
  it("1er save (pas de baseline) → full", () => {
    const doc = mkDoc();
    const p = planSave(doc, "/x.glucose");
    expect(p.mode).toBe("full");
    expect(p.bytes.length).toBeGreaterThan(0);
  });

  it("CRITIQUE — full + 1 incrémental se recharge à l'identique", () => {
    let doc = mkDoc();
    const p1 = planSave(doc, "/x.glucose");
    expect(p1.mode).toBe("full");
    let file = p1.bytes;                       // write (truncate)
    commitSave("/x.glucose", doc, p1);

    doc = A.change(doc, "e1", (d) => { d.name = "edited"; });
    doc = A.change(doc, "e2", (d) => { d.boards[0].name = "B1"; });

    const p2 = planSave(doc, "/x.glucose");
    expect(p2.mode).toBe("incremental");
    expect(p2.bytes.length).toBeGreaterThan(0);
    file = concat([file, p2.bytes]);           // append
    commitSave("/x.glucose", doc, p2);

    const loaded = A.load<Project>(file);
    expect(A.asPlain(loaded)).toEqual(A.asPlain(doc));
  });

  it("CRITIQUE — full + plusieurs incréments successifs → identique", () => {
    let doc = mkDoc();
    const p1 = planSave(doc, "/m.glucose");
    let file = p1.bytes;
    commitSave("/m.glucose", doc, p1);

    for (let i = 0; i < 5; i++) {
      doc = A.change(doc, `step ${i}`, (d) => {
        d.boards[0].images.push({
          id: `img${i}`, src: `asset:${i}.png`, x: i, y: i,
          width: 10, height: 10, rotation: 0, locked: false, tags: [],
          originalWidth: 10, originalHeight: 10,
        });
      });
      const p = planSave(doc, "/m.glucose");
      // tant qu'on ne dépasse pas la compaction, ça reste incrémental
      file = p.mode === "full" ? p.bytes : concat([file, p.bytes]);
      commitSave("/m.glucose", doc, p);
    }

    const loaded = A.load<Project>(file);
    expect(A.asPlain(loaded)).toEqual(A.asPlain(doc));
    expect((A.asPlain(loaded) as Project).boards[0].images).toHaveLength(5);
  });

  it("aucun changement → incrémental vide (no-op disque)", () => {
    const doc = mkDoc();
    const p1 = planSave(doc, "/n.glucose");
    commitSave("/n.glucose", doc, p1);
    const p2 = planSave(doc, "/n.glucose"); // même doc, rien n'a changé
    expect(p2.mode).toBe("incremental");
    expect(p2.bytes.length).toBe(0);
  });

  it("fichier différent → full (pas d'append sur le mauvais fichier)", () => {
    const doc = mkDoc();
    const p1 = planSave(doc, "/a.glucose");
    commitSave("/a.glucose", doc, p1);
    const p2 = planSave(doc, "/b.glucose");
    expect(p2.mode).toBe("full");
  });

  it("compaction — un delta qui dépasse la taille du full force un save complet", () => {
    let doc = mkDoc();
    const p1 = planSave(doc, "/c.glucose");
    commitSave("/c.glucose", doc, p1);
    const fullSize = _peekBaseline()!.fullSize;

    // Grosse modif → delta volumineux, largement > fullSize.
    doc = A.change(doc, "big", (d) => { d.name = "x".repeat(fullSize * 4 + 5000); });
    const p2 = planSave(doc, "/c.glucose");
    expect(p2.mode).toBe("full"); // recompaction au lieu d'un append géant
  });

  it("markLoaded permet un 1er Ctrl+S incrémental (et roundtrip OK)", () => {
    let doc = mkDoc();
    const fileFull = A.save(doc);            // = fichier sur disque après ouverture
    markLoaded("/l.glucose", doc, fileFull.length);

    doc = A.change(doc, "post-load", (d) => { d.name = "after load"; });
    const p = planSave(doc, "/l.glucose");
    expect(p.mode).toBe("incremental");

    const file = concat([fileFull, p.bytes]);
    expect(A.asPlain(A.load<Project>(file))).toEqual(A.asPlain(doc));
  });

  it("resetSaveState force un full au prochain save", () => {
    const doc = mkDoc();
    commitSave("/r.glucose", doc, planSave(doc, "/r.glucose"));
    resetSaveState();
    expect(_peekBaseline()).toBeNull();
    const p = planSave(doc, "/r.glucose");
    expect(p.mode).toBe("full");
  });
});

describe("saveState — deltaBytes (ampleur pour jalons auto, Git #1 P3)", () => {
  it("full initial → deltaBytes 0 (nouvelle ligne de base)", () => {
    const doc = mkDoc();
    expect(planSave(doc, "/d.glucose").deltaBytes).toBe(0);
  });

  it("incrémental → deltaBytes = taille du delta (>0)", () => {
    let doc = mkDoc();
    commitSave("/d.glucose", doc, planSave(doc, "/d.glucose"));
    doc = A.change(doc, "e", (d) => { d.name = "édité"; });
    const p = planSave(doc, "/d.glucose");
    expect(p.mode).toBe("incremental");
    expect(p.deltaBytes).toBe(p.bytes.length);
    expect(p.deltaBytes).toBeGreaterThan(0);
  });

  it("aucun changement → deltaBytes 0", () => {
    const doc = mkDoc();
    commitSave("/d.glucose", doc, planSave(doc, "/d.glucose"));
    expect(planSave(doc, "/d.glucose").deltaBytes).toBe(0);
  });

  it("compaction full → deltaBytes = taille des SEULS nouveaux changements (pas le full)", () => {
    let doc = mkDoc();
    commitSave("/d.glucose", doc, planSave(doc, "/d.glucose"));
    const fullSize = _peekBaseline()!.fullSize;
    const before = doc; // == baseline doc committé
    doc = A.change(doc, "big", (d) => { d.name = "x".repeat(fullSize * 4 + 5000); });
    const expectedDelta = concat(A.getChanges(before, doc)).length;
    const p = planSave(doc, "/d.glucose");
    expect(p.mode).toBe("full"); // compaction forcée
    // deltaBytes = l'ampleur RÉELLE de ce save (le delta), pas la taille du full
    // réécrit → pas de faux jalon auto au moment d'une compaction.
    expect(p.deltaBytes).toBe(expectedDelta);
  });
});

describe("loadResilient — récupération d'une fin de fichier corrompue", () => {
  it("fichier complet → pas de récupération, doc correct", () => {
    let doc = mkDoc();
    const full = A.save(doc);
    const base = doc;
    doc = A.change(doc, "e", (d) => { d.name = "v2"; });
    const delta = (() => { const cs = A.getChanges(base, doc); return concat(cs); })();
    const file = concat([full, delta]);

    const res = A.loadResilient<Project>(file);
    expect(res.recovered).toBe(false);
    expect(res.droppedBytes).toBe(0);
    expect((A.asPlain(res.doc) as Project).name).toBe("v2");
  });

  it("CRITIQUE — fin tronquée : récupère tout SAUF le delta coupé (jamais tout perdre)", () => {
    let doc = mkDoc();                                  // name=init
    const full = A.save(doc);
    let base = doc;
    doc = A.change(doc, "e1", (d) => { d.name = "v2"; });
    const delta1 = concat(A.getChanges(base, doc)); base = doc;
    doc = A.change(doc, "e2", (d) => { d.boards[0].name = "B1"; });
    const delta2 = concat(A.getChanges(base, doc));

    // Simule un crash en plein milieu de l'append de delta2.
    const torn = concat([full, delta1, delta2.subarray(0, Math.floor(delta2.length / 2))]);

    const res = A.loadResilient<Project>(torn);
    expect(res.recovered).toBe(true);
    expect(res.droppedBytes).toBeGreaterThan(0);
    const p = A.asPlain(res.doc) as Project;
    expect(p.name).toBe("v2");            // delta1 préservé
    expect(p.boards[0].name).toBe("B0");  // delta2 (tronqué) ignoré, mais le reste intact
  });

  it("plusieurs deltas, le dernier tronqué → garde tous les deltas complets", () => {
    let doc = mkDoc();
    const full = A.save(doc);
    const parts: Uint8Array[] = [full];
    let base = doc;
    // 3 deltas complets
    for (let i = 0; i < 3; i++) {
      doc = A.change(doc, `e${i}`, (d) => { d.boards[0].images.push({
        id: `img${i}`, src: `asset:${i}.png`, x: i, y: i, width: 10, height: 10,
        rotation: 0, locked: false, tags: [], originalWidth: 10, originalHeight: 10,
      }); });
      parts.push(concat(A.getChanges(base, doc))); base = doc;
    }
    // 1 delta supplémentaire, TRONQUÉ
    doc = A.change(doc, "torn", (d) => { d.name = "jamais-ecrit"; });
    const tornDelta = concat(A.getChanges(base, doc));
    const file = concat([...parts, tornDelta.subarray(0, Math.floor(tornDelta.length / 2))]);

    const res = A.loadResilient<Project>(file);
    expect(res.recovered).toBe(true);
    const p = A.asPlain(res.doc) as Project;
    expect(p.boards[0].images).toHaveLength(3); // 3 deltas complets préservés
    expect(p.name).toBe("init");                // delta tronqué (rename) ignoré
  });
});
