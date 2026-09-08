import { describe, expect, it } from "vitest";
import {
  PICK,
  collectCandidates,
  pickWithCycle,
  handleSlopWorld,
  onRectEdge,
  type CycleState,
  type PickInput,
} from "./hitPriority";
import type { Annotation, BoardImage, CanvasFolder } from "../types";

// ── Fabriques ───────────────────────────────────────────────────────────────

function img(id: string, x: number, y: number, w: number, h: number, locked = false): BoardImage {
  // x,y = CENTRE (ancre sprite 0.5), cf. GlucoseCanvas.
  return {
    id, x, y, width: w, height: h,
    rotation: 0, locked, tags: [],
    originalWidth: w, originalHeight: h,
  };
}

function membrane(id: string, x: number, y: number, w: number, h: number, text?: string): Annotation {
  return { id, type: "membrane", x, y, width: w, height: h, text };
}

function text(id: string, x: number, y: number, w: number, h: number): Annotation {
  return { id, type: "text", x, y, text: "hello", width: w, height: h };
}

function sticky(id: string, x: number, y: number, w: number, h: number): Annotation {
  return { id, type: "sticky", x, y, text: "note", width: w, height: h };
}

function folder(id: string, x: number, y: number, w: number, h: number): CanvasFolder {
  return { id, name: id, color: "#60a5fa", x, y, width: w, height: h, childBoardId: `${id}-child` };
}

function at(wx: number, wy: number, over: Partial<PickInput> = {}): PickInput {
  return {
    wx, wy, scale: 1,
    images: [], annotations: [], folders: [],
    selectedImageIds: [], selectedAnnotationIds: [], selectedFolderId: null,
    ...over,
  };
}

/** Raccourci : la liste des « qui gagne » dans l'ordre, pour lire un test d'un coup. */
function order(input: PickInput): string[] {
  return collectCandidates(input).map((c) => `${c.kind}:${c.id}`);
}

// ── Les règles demandées ────────────────────────────────────────────────────

describe("ordre de priorité — le conteneur ne mange plus son contenu", () => {
  const MEMB = membrane("M1", 0, 0, 1000, 800);

  it("clic au CENTRE d'une image posée dans une membrane → l'image", () => {
    const cands = collectCandidates(at(500, 400, {
      annotations: [MEMB],
      images: [img("I1", 500, 400, 200, 150)],
    }));
    expect(cands[0].id).toBe("I1");
    expect(cands.map((c) => c.kind)).toEqual(["image", "membrane-body"]);
  });

  it("clic sur les POINTILLÉS de la membrane → la membrane, même noyée d'images", () => {
    const cands = collectCandidates(at(5, 400, {
      annotations: [MEMB],
      images: [img("I1", 5, 400, 200, 150), img("I2", 40, 400, 200, 150)],
    }));
    expect(cands[0].kind).toBe("membrane-edge");
    expect(cands[0].id).toBe("M1");
  });

  it("le bord se saisit aussi depuis l'EXTÉRIEUR de la membrane", () => {
    expect(order(at(-8, 400, { annotations: [MEMB] }))).toEqual(["membrane-edge:M1"]);
  });

  it("l'étiquette, peinte au-dessus du bord haut, appartient au bord", () => {
    const withLabel = membrane("M2", 0, 0, 1000, 800, "Recherche");
    expect(order(at(200, -20, { annotations: [withLabel] }))).toEqual(["membrane-edge:M2"]);
  });

  it("clic en plein dans un texte posé dans une membrane → le texte", () => {
    const cands = collectCandidates(at(500, 400, {
      annotations: [MEMB, text("T1", 400, 380, 200, 60)],
    }));
    expect(cands.map((c) => c.kind)).toEqual(["text", "membrane-body"]);
  });
});

describe("le texte est toujours le dernier des contenus", () => {
  it("une image SOUS un texte gagne, même curseur en plein sur le bloc texte", () => {
    const cands = collectCandidates(at(100, 50, {
      annotations: [text("T1", 0, 0, 200, 100)],
      images: [img("I1", 100, 50, 300, 200)],
    }));
    expect(cands[0].id).toBe("I1");
    expect(cands.map((c) => c.kind)).toEqual(["image", "text"]);
  });

  it("image > sticky > texte", () => {
    const cands = collectCandidates(at(100, 50, {
      annotations: [text("T1", 0, 0, 200, 100), sticky("S1", 0, 0, 200, 100)],
      images: [img("I1", 100, 50, 300, 200)],
    }));
    expect(cands.map((c) => c.kind)).toEqual(["image", "sticky", "text"]);
  });

  it("une flèche passe devant l'image (tracé fin, difficile à viser)", () => {
    const cands = collectCandidates(at(100, 50, {
      images: [img("I1", 100, 50, 300, 200)],
      arrowId: "A1",
    }));
    expect(cands.map((c) => c.kind)).toEqual(["arrow", "image"]);
  });
});

describe("dossiers", () => {
  it("bandeau d'en-tête et bordure = bord ; le reste = corps", () => {
    const f = [folder("F1", 0, 0, 400, 300)];
    expect(order(at(200, 20, { folders: f }))).toEqual(["folder-edge:F1"]);
    expect(order(at(2, 150, { folders: f }))).toEqual(["folder-edge:F1"]);
    expect(order(at(200, 150, { folders: f }))).toEqual(["folder-body:F1"]);
  });

  it("le corps du dossier passe derrière une image qu'il contient", () => {
    const cands = collectCandidates(at(200, 150, {
      folders: [folder("F1", 0, 0, 400, 300)],
      images: [img("I1", 200, 150, 80, 60)],
    }));
    expect(cands.map((c) => c.kind)).toEqual(["image", "folder-body"]);
  });
});

describe("images", () => {
  it("une image verrouillée n'occupe pas de case (elle n'est pas cliquable)", () => {
    expect(order(at(100, 100, { images: [img("L1", 100, 100, 200, 200, true)] }))).toEqual([]);
  });

  it("à rang égal, l'image peinte au-dessus (dernière de la liste) gagne", () => {
    const cands = collectCandidates(at(100, 100, {
      images: [img("dessous", 100, 100, 200, 200), img("dessus", 100, 100, 400, 400)],
    }));
    expect(cands.map((c) => c.id)).toEqual(["dessus", "dessous"]);
  });

  it("tient compte de la rotation du sprite", () => {
    const rotated: BoardImage = { ...img("R1", 0, 0, 200, 40), rotation: Math.PI / 2 };
    // Boîte pivotée d'un quart de tour : haute et fine au lieu de large et plate.
    expect(order(at(0, 80, { images: [rotated] }))).toEqual(["image:R1"]);
    expect(order(at(80, 0, { images: [rotated] }))).toEqual([]);
  });
});

// ── Poignées de redimensionnement ───────────────────────────────────────────

describe("poignées — priorité absolue et boîte de clic généreuse", () => {
  const MEMB = membrane("M1", 0, 0, 1000, 800);

  it("passent devant tout le reste dès que la membrane est sélectionnée", () => {
    const cands = collectCandidates(at(10, 10, {
      annotations: [MEMB],
      images: [img("I1", 10, 10, 300, 300)],
      selectedAnnotationIds: ["M1"],
    }));
    expect(cands[0].kind).toBe("handle");
    expect(cands[0].corner).toBe("tl");
  });

  it("aucune poignée tant que rien n'est sélectionné", () => {
    const cands = collectCandidates(at(10, 10, { annotations: [MEMB] }));
    expect(cands.every((c) => c.kind !== "handle")).toBe(true);
  });

  it("la préhension est constante à l'ÉCRAN (elle grandit quand on dézoome)", () => {
    const far = { annotations: [MEMB], selectedAnnotationIds: ["M1"] };
    // 50 unités monde du coin : hors de portée à zoom 1 (18 px), dans la portée
    // à zoom 0.25 (18 / 0.25 = 72 unités monde).
    expect(collectCandidates({ ...at(50, 0, far), scale: 1 })[0]?.kind).not.toBe("handle");
    expect(collectCandidates({ ...at(50, 0, far), scale: 0.25 })[0]?.kind).toBe("handle");
  });

  it("mais jamais au point de rendre un petit bloc indéplaçable", () => {
    // Bloc de 40×30 : 4 poignées de 18 px couvriraient toute la boîte. Le
    // plafond (35 % du petit côté) les ramène à 10,5 unités.
    expect(handleSlopWorld(1, 40, 30)).toBeCloseTo(10.5);
    // Le centre du bloc reste attrapable → on peut encore le déplacer.
    const cands = collectCandidates(at(20, 15, {
      annotations: [text("T1", 0, 0, 40, 30)],
      selectedAnnotationIds: ["T1"],
    }));
    expect(cands[0].kind).toBe("text");
  });

  it("la plus PROCHE des poignées gagne quand deux se chevauchent", () => {
    // Bloc de 10 de large : à x=4 les poignées tl (d=4) et tr (d=6) sont toutes
    // deux à portée (plancher de préhension = 6).
    const narrow = text("T1", 0, 0, 10, 200);
    const cands = collectCandidates(at(4, 0, {
      annotations: [narrow], selectedAnnotationIds: ["T1"],
    }));
    expect(cands.slice(0, 2).map((c) => c.corner)).toEqual(["tl", "tr"]);
  });

  it("le dossier sélectionné expose sa poignée bas-droite", () => {
    const cands = collectCandidates(at(400, 300, {
      folders: [folder("F1", 0, 0, 400, 300)], selectedFolderId: "F1",
    }));
    expect(cands[0].kind).toBe("handle");
    expect(cands[0].owner).toBe("folder");
  });
});

// ── Cycle « re-clic = cible suivante » ──────────────────────────────────────

describe("cycle de priorité", () => {
  // Un point où se superposent : bord de membrane, image, texte.
  const stack = at(8, 400, {
    annotations: [membrane("M1", 0, 0, 1000, 800), text("T1", 0, 380, 100, 40)],
    images: [img("I1", 8, 400, 200, 150)],
  });

  function clickChain(n: number, opts: { alt?: boolean } = {}) {
    const cands = collectCandidates(stack);
    let cycle: CycleState | null = null;
    const seen: string[] = [];
    let t = 1000;
    for (let i = 0; i < n; i++) {
      const r = pickWithCycle(cands, cycle, 300, 300, t, opts);
      seen.push(`${r.picked?.kind}:${r.picked?.id}`);
      cycle = r.cycle;
      t += 120; // clics rapprochés, souris immobile
    }
    return seen;
  }

  it("chaque re-clic au même endroit descend d'un cran", () => {
    expect(clickChain(3)).toEqual(["membrane-edge:M1", "image:I1", "text:T1"]);
  });

  it("s'ARRÊTE sur le texte : le clic suivant doit rester un double-clic → édition", () => {
    expect(clickChain(5)).toEqual([
      "membrane-edge:M1", "image:I1", "text:T1", "text:T1", "text:T1",
    ]);
  });

  it("Alt force le pas suivant et reboucle (échappatoire depuis un bloc éditable)", () => {
    expect(clickChain(4, { alt: true })).toEqual([
      "membrane-edge:M1", "image:I1", "text:T1", "membrane-edge:M1",
    ]);
  });

  it("bouger la souris repart du rang le plus prioritaire", () => {
    const cands = collectCandidates(stack);
    const first = pickWithCycle(cands, null, 300, 300, 1000);
    const moved = pickWithCycle(cands, first.cycle, 300 + PICK.CYCLE_RADIUS_PX + 5, 300, 1100);
    expect(moved.picked?.kind).toBe("membrane-edge");
  });

  it("après un long moment, le cycle est oublié", () => {
    const cands = collectCandidates(stack);
    const first = pickWithCycle(cands, null, 300, 300, 1000);
    const late = pickWithCycle(cands, first.cycle, 300, 300, 1000 + PICK.CYCLE_TTL_MS + 1);
    expect(late.picked?.kind).toBe("membrane-edge");
  });

  it("Ctrl/Shift (multi-sélection) ne cycle jamais", () => {
    const cands = collectCandidates(stack);
    const first = pickWithCycle(cands, null, 300, 300, 1000, { multi: true });
    const second = pickWithCycle(cands, first.cycle, 300, 300, 1100, { multi: true });
    expect(second.picked?.kind).toBe("membrane-edge");
  });

  it("une seule cible sous le curseur → pas de cycle, le double-clic reste possible", () => {
    const solo = collectCandidates(at(500, 400, { annotations: [membrane("M1", 0, 0, 1000, 800)] }));
    let cycle: CycleState | null = null;
    for (let i = 0; i < 3; i++) {
      const r = pickWithCycle(solo, cycle, 10, 10, 1000 + i * 100);
      expect(r.picked?.id).toBe("M1");
      cycle = r.cycle;
    }
  });

  it("une poignée gagne, sort du cycle, et le clic suivant retombe dessus", () => {
    const cands = collectCandidates(at(10, 10, {
      annotations: [membrane("M1", 0, 0, 1000, 800)],
      images: [img("I1", 10, 10, 300, 300)],
      selectedAnnotationIds: ["M1"],
    }));
    const first = pickWithCycle(cands, null, 300, 300, 1000);
    expect(first.picked?.kind).toBe("handle");
    expect(first.cycle).toBeNull();
    const second = pickWithCycle(cands, first.cycle, 300, 300, 1100);
    expect(second.picked?.kind).toBe("handle");
  });

  it("Alt court-circuite la priorité des poignées pour atteindre ce qu'il y a dessous", () => {
    const cands = collectCandidates(at(10, 10, {
      annotations: [membrane("M1", 0, 0, 1000, 800)],
      images: [img("I1", 10, 10, 300, 300)],
      selectedAnnotationIds: ["M1"],
    }));
    expect(pickWithCycle(cands, null, 300, 300, 1000, { alt: true }).picked?.kind)
      .toBe("membrane-edge");
  });

  it("rien sous le curseur → aucune cible (le canvas reprend la main)", () => {
    expect(pickWithCycle([], null, 0, 0, 0)).toEqual({ picked: null, cycle: null });
  });
});

// ── Géométrie ───────────────────────────────────────────────────────────────

describe("onRectEdge", () => {
  it("attrape la bande, dedans comme dehors, et laisse filer le centre", () => {
    expect(onRectEdge(0, 50, 0, 0, 100, 100, 10)).toBe(true);   // sur le trait
    expect(onRectEdge(-8, 50, 0, 0, 100, 100, 10)).toBe(true);  // juste dehors
    expect(onRectEdge(8, 50, 0, 0, 100, 100, 10)).toBe(true);   // juste dedans
    expect(onRectEdge(50, 50, 0, 0, 100, 100, 10)).toBe(false); // plein centre
    expect(onRectEdge(-20, 50, 0, 0, 100, 100, 10)).toBe(false); // trop loin
  });

  it("un rectangle plus petit que la bande se saisit n'importe où", () => {
    expect(onRectEdge(5, 5, 0, 0, 10, 10, 20)).toBe(true);
  });
});
