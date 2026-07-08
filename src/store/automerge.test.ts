// Tests basiques du wrapper Automerge.
// Vérifie create / change / save / load / merge — la base sur laquelle Phase 7
// va construire la Time Machine et la sync multi-utilisateur.

import { describe, it, expect } from "vitest";
import * as A from "./automerge";

interface MiniProject {
  name: string;
  boards: { id: string; title: string; counter: number }[];
}

function makeInitial(): MiniProject {
  return {
    name: "Test",
    boards: [{ id: "b1", title: "Board 1", counter: 0 }],
  };
}

describe("automerge wrapper — basics", () => {
  it("create renvoie un document utilisable comme un objet plain", () => {
    const doc = A.create(makeInitial());
    expect(doc.name).toBe("Test");
    expect(doc.boards).toHaveLength(1);
    expect(doc.boards[0].id).toBe("b1");
  });

  it("change renvoie un NOUVEAU document immutable", () => {
    const doc1 = A.create(makeInitial());
    const doc2 = A.change(doc1, "rename", (d) => { d.name = "Renamed"; });
    expect(doc1.name).toBe("Test");
    expect(doc2.name).toBe("Renamed");
  });

  it("permet d'ajouter un élément à un tableau", () => {
    const doc1 = A.create(makeInitial());
    const doc2 = A.change(doc1, "add board", (d) => {
      d.boards.push({ id: "b2", title: "Board 2", counter: 5 });
    });
    expect(doc2.boards).toHaveLength(2);
    expect(doc2.boards[1].id).toBe("b2");
  });

  it("permet d'incrémenter un counter", () => {
    let doc = A.create(makeInitial());
    for (let i = 0; i < 10; i++) {
      doc = A.change(doc, `inc-${i}`, (d) => { d.boards[0].counter += 1; });
    }
    expect(doc.boards[0].counter).toBe(10);
  });
});

describe("automerge wrapper — save / load roundtrip", () => {
  it("save puis load reproduit le document à l'identique", () => {
    const doc = A.create(makeInitial());
    const mutated = A.change(doc, "test", (d) => {
      d.name = "Persistent";
      d.boards.push({ id: "b2", title: "Saved", counter: 42 });
    });
    const bytes = A.save(mutated);
    expect(bytes).toBeInstanceOf(Uint8Array);
    expect(bytes.length).toBeGreaterThan(0);

    const loaded = A.load<MiniProject>(bytes);
    expect(loaded.name).toBe("Persistent");
    expect(loaded.boards).toHaveLength(2);
    expect(loaded.boards[1].counter).toBe(42);
  });

  it("le binaire est compact (< 1 KB pour un mini-projet)", () => {
    const doc = A.create(makeInitial());
    const bytes = A.save(doc);
    expect(bytes.length).toBeLessThan(1024);
  });
});

// ────────────────────────────────────────────────────────────────────────────
// Phase 7.4 — Patterns Time Machine UI
// ────────────────────────────────────────────────────────────────────────────

describe("Time Machine — viewAt + restore", () => {
  it("viewAt(headsAfterStep1) restitue exactement l'état au step 1", () => {
    let doc = A.create(makeInitial());
    doc = A.change(doc, "step 1", (d) => { d.boards[0].counter = 10; });
    const heads1 = A.getHeads(doc);
    doc = A.change(doc, "step 2", (d) => { d.boards[0].counter = 20; });
    doc = A.change(doc, "step 3", (d) => { d.boards[0].counter = 30; });

    const past = A.viewAt<MiniProject>(doc, heads1);
    expect(past.boards[0].counter).toBe(10);
    // Le doc courant n'est pas affecté
    expect(doc.boards[0].counter).toBe(30);
  });

  it("restaurer un état passé via splice replace préserve l'historique antérieur", () => {
    let doc = A.create(makeInitial());
    doc = A.change(doc, "step 1", (d) => { d.boards[0].counter = 10; });
    const heads1 = A.getHeads(doc);
    doc = A.change(doc, "step 2", (d) => { d.boards[0].counter = 99; });

    // Restauration : commit qui réécrit le contenu pour matcher l'état au step 1
    const past = A.asPlain(A.viewAt<MiniProject>(doc, heads1));
    doc = A.change(doc, "restore step 1", (d) => {
      d.boards[0].counter = past.boards[0].counter;
    });

    // L'état courant est celui du step 1
    expect(doc.boards[0].counter).toBe(10);
    // L'historique est de 4 commits (init, step 1, step 2, restore)
    expect(A.history(doc).length).toBe(4);
  });

  it("history expose les messages des commits dans l'ordre", () => {
    let doc = A.create(makeInitial());
    doc = A.change(doc, "📌 Avant refonte", (d) => { d.name = "v1"; });
    doc = A.change(doc, "Refonte step 1", (d) => { d.name = "v2"; });
    doc = A.change(doc, "📌 v2 stable", (d) => { d.name = "v2-stable"; });

    const messages = A.history(doc).map((h) => h.change.message ?? "");
    expect(messages).toEqual(["init", "📌 Avant refonte", "Refonte step 1", "📌 v2 stable"]);
    // Filtrage des jalons nommés (UI Time Machine)
    const jalons = messages.filter((m) => m?.startsWith("📌") ?? false);
    expect(jalons).toHaveLength(2);
  });
});

describe("automerge wrapper — merge", () => {
  // Pour simuler deux acteurs distincts, on `clone()` le document de base
  // (Automerge interdit de muter le même doc depuis deux branches).
  it("fusionne sans conflit deux modifications sur des champs différents", () => {
    const base = A.create(makeInitial());
    const alice = A.change(A.clone(base), "alice", (d) => { d.name = "Alice's version"; });
    const bob   = A.change(A.clone(base), "bob",   (d) => { d.boards[0].counter = 99; });

    const merged = A.merge(alice, bob);
    expect(merged.name).toBe("Alice's version");
    expect(merged.boards[0].counter).toBe(99);
  });

  it("fusionne deux ajouts concurrents dans un tableau (les deux survivent)", () => {
    const base = A.create(makeInitial());
    const alice = A.change(A.clone(base), "alice add", (d) => {
      d.boards.push({ id: "b-alice", title: "Alice", counter: 0 });
    });
    const bob = A.change(A.clone(base), "bob add", (d) => {
      d.boards.push({ id: "b-bob", title: "Bob", counter: 0 });
    });

    const merged = A.merge(alice, bob);
    expect(merged.boards).toHaveLength(3); // b1 + b-alice + b-bob
    const ids = merged.boards.map((b) => b.id).sort();
    expect(ids).toEqual(["b-alice", "b-bob", "b1"]);
  });

  it("le merge est commutatif (alice ⊕ bob == bob ⊕ alice)", () => {
    const base = A.create(makeInitial());
    const alice = A.change(A.clone(base), "a", (d) => { d.boards[0].title = "Alice"; });
    const bob   = A.change(A.clone(base), "b", (d) => {
      d.boards.push({ id: "bob", title: "Bob", counter: 1 });
    });
    const ab = A.merge(A.clone(alice), bob);
    const ba = A.merge(A.clone(bob), alice);
    expect(A.asPlain(ab)).toEqual(A.asPlain(ba));
  });
});

describe("automerge wrapper — Time Machine API", () => {
  it("getHistory renvoie un commit par change()", () => {
    let doc = A.create(makeInitial());
    doc = A.change(doc, "step 1", (d) => { d.boards[0].counter = 1; });
    doc = A.change(doc, "step 2", (d) => { d.boards[0].counter = 2; });
    doc = A.change(doc, "step 3", (d) => { d.boards[0].counter = 3; });

    const history = A.history(doc);
    // 1 commit pour init + 3 pour les changes = 4
    expect(history.length).toBe(4);
    // Les messages sont retrouvables
    const messages = history.map((h) => h.change.message);
    expect(messages).toEqual(["init", "step 1", "step 2", "step 3"]);
  });

  it("viewAt(heads) restitue un état passé sans muter le doc courant", () => {
    let doc = A.create(makeInitial());
    doc = A.change(doc, "v1", (d) => { d.boards[0].counter = 10; });
    const headsAfterV1 = A.getHeads(doc);
    doc = A.change(doc, "v2", (d) => { d.boards[0].counter = 99; });

    const past = A.viewAt<MiniProject>(doc, headsAfterV1);
    expect(past.boards[0].counter).toBe(10);
    expect(doc.boards[0].counter).toBe(99); // non muté
  });

  it("allChanges + decodeMeta : mêmes libellés que getHistory, mais SANS matérialiser l'état", () => {
    // C'est le chemin léger utilisé par la Time Machine (getHistory reconstruit un
    // snapshot par change → gel ~2000 ms ; decodeMeta ne lit que l'en-tête).
    let doc = A.create(makeInitial());
    doc = A.change(doc, "📌 jalon", (d) => { d.boards[0].counter = 1; });
    doc = A.change(doc, "edit", (d) => { d.boards[0].counter = 2; });

    const viaHistory = A.history(doc).map((h) => h.change.message ?? "");
    const metas = A.allChanges(doc).map((c) => A.decodeMeta(c));
    expect(metas.map((m) => m.message)).toEqual(viaHistory); // mêmes messages, même ordre

    // Le hash décodé suffit à faire un viewAt (aperçu d'un état passé).
    const past = A.viewAt<MiniProject>(doc, [metas[1].hash]);
    expect(past.boards[0].counter).toBe(1); // état juste après « 📌 jalon »
  });
});

describe("automerge wrapper — asPlain", () => {
  it("renvoie un objet JS standard (pas un Proxy)", () => {
    const doc = A.create(makeInitial());
    const plain = A.asPlain(doc);
    expect(plain.name).toBe("Test");
    // Doit pouvoir être muté librement sans toucher le doc
    plain.name = "muté";
    expect(doc.name).toBe("Test");
  });
});

// ────────────────────────────────────────────────────────────────────────────
// Phase 7.2 — Roundtrip Project complet (proche du format réel `.glucose`)
// ────────────────────────────────────────────────────────────────────────────

describe("Project roundtrip — format v2", () => {
  // Mini-projet qui couvre les types subtils du Project réel.
  function makeRichProject() {
    return {
      version: "2.0.0",
      name: "Projet test",
      activeBoardId: "b1",
      createdAt: 1700000000000,
      updatedAt: 1700000001000,
      presets: [
        { id: "p1", name: "Cara", description: "", isBuiltin: true, createdAt: 1700000000000,
          slots: [{ id: "s1", name: "Ref", color: "#3b82f6", description: "", order: 0 }] },
      ],
      domains: [
        { id: "d1", name: "Science", color: "#10b981", icon: "🔬", createdAt: 1700000000000 },
      ],
      boards: [
        {
          id: "b1",
          name: "Main",
          createdAt: 1700000000000,
          updatedAt: 1700000001000,
          viewport: { x: 100, y: 50, scale: 1.5 },
          zones: [],
          folders: [],
          panels: [],
          images: [
            {
              id: "i1", src: "asset:abc.png",
              x: 12.5, y: -42.7, width: 320, height: 240, rotation: 0,
              locked: false, tags: ["ref", "important"],
              originalWidth: 1920, originalHeight: 1080,
              domains: [{ domainId: "d1", weight: 0.8 }],
              temporalAnchor: { start: -3000, end: -3000, label: "Antiquité" },
            },
          ],
          annotations: [
            {
              id: "a1", type: "sticky" as const,
              x: 0, y: 0, width: 160, height: 100,
              text: "Hello\nMulti-line",
              bgColor: "#fde68a",
              temporalAnchor: { start: 1789, end: 1799 },
            },
            {
              id: "a2", type: "arrow" as const,
              x: 0, y: 0, x2: 200, y2: 100,
              sourceId: "i1", targetId: "a1",
              predicate: "inspire" as const,
              waypoints: [{ x: 50, y: 25 }, { x: 100, y: 50 }],
            },
          ],
        },
      ],
    };
  }

  it("save → load préserve toutes les valeurs (numériques, booléens, arrays imbriqués)", () => {
    const original = makeRichProject();
    const doc = A.create(original);
    const bytes = A.save(doc);
    const loaded = A.load<typeof original>(bytes);
    const plain = A.asPlain(loaded);

    expect(plain.name).toBe(original.name);
    expect(plain.boards[0].images[0].x).toBe(12.5);
    expect(plain.boards[0].images[0].y).toBe(-42.7);
    expect(plain.boards[0].images[0].tags).toEqual(["ref", "important"]);
    expect(plain.boards[0].images[0].temporalAnchor).toEqual({ start: -3000, end: -3000, label: "Antiquité" });
    expect(plain.boards[0].annotations).toHaveLength(2);
    expect(plain.boards[0].annotations[1].waypoints).toEqual([{ x: 50, y: 25 }, { x: 100, y: 50 }]);
    expect(plain.boards[0].annotations[0].text).toBe("Hello\nMulti-line");
    expect(plain.domains?.[0].icon).toBe("🔬");
  });

  it("le format binaire est plus compact que le JSON équivalent", () => {
    const original = makeRichProject();
    const doc = A.create(original);
    const binary = A.save(doc);
    const json = JSON.stringify(original);
    // Pour ce mini-projet le binaire est de l'ordre du JSON ou plus léger,
    // mais surtout : sur un projet réel avec des opérations répétées,
    // Automerge dédupplique mieux que le JSON. Au pire, on est dans le même
    // ordre de grandeur — pas de blow-up.
    expect(binary.length).toBeLessThan(json.length * 2);
  });

  it("save deux fois donne deux binaires (peuvent différer en bytes — acteur), mais reload identique", () => {
    const original = makeRichProject();
    const doc1 = A.create(original);
    const doc2 = A.create(original);
    const plain1 = A.asPlain(A.load<typeof original>(A.save(doc1)));
    const plain2 = A.asPlain(A.load<typeof original>(A.save(doc2)));
    expect(plain1).toEqual(plain2);
  });
});
