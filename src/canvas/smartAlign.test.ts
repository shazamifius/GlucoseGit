import { describe, expect, it } from "vitest";
import {
  collectAlignTargets,
  rectOfImage,
  sameGuides,
  snapMove,
  snapPoint,
  snapResize,
  unionRect,
  type AlignTarget,
} from "./smartAlign";
import type { Annotation, Board, BoardImage, CanvasFolder } from "../types";

// Cible de référence : boîte 100×100 posée en (0,0).
//   bords X : 0 / 50 (centre) / 100 — bords Y : idem.
const REF: AlignTarget = { id: "ref", kind: "text", rect: { left: 0, top: 0, width: 100, height: 100 } };

function box(left: number, top: number, width = 40, height = 40) {
  return { left, top, width, height };
}

describe("snapMove — déplacement", () => {
  it("accroche le bord gauche au bord gauche d'une cible", () => {
    const r = snapMove(box(3, 500), [REF]);
    expect(r.dx).toBeCloseTo(-3);
    expect(r.guides.x).toEqual([0]);
  });

  it("accroche les centres", () => {
    // centre de la boîte = 50 + 20 = ... on la place pour que son centre tombe à 47
    const r = snapMove(box(27, 500), [REF]); // centre = 47, cible = 50
    expect(r.dx).toBeCloseTo(3);
    expect(r.guides.x).toEqual([50]);
  });

  it("n'accroche pas au-delà du seuil", () => {
    const r = snapMove(box(40, 500), [REF]);
    expect(r.dx).toBe(0);
    expect(r.guides.x).toBeUndefined();
  });

  it("les deux axes s'accrochent indépendamment", () => {
    const r = snapMove(box(2, 103), [REF]);
    expect(r.dx).toBeCloseTo(-2);   // gauche → 0
    expect(r.dy).toBeCloseTo(-3);   // haut  → 100 (bord bas de la cible)
    expect(r.guides.x).toEqual([0]);
    expect(r.guides.y).toEqual([100]);
  });

  it("le seuil est constant à l'ÉCRAN : dézoomé, il couvre plus de monde", () => {
    // 20 unités monde d'écart = 2 px écran à scale 0.1 → dans le seuil.
    expect(snapPoint(20, 500, [REF], { scale: 0.1 }).x).toBeCloseTo(0);
    // Les mêmes 20 unités à scale 4 valent 80 px écran → hors seuil.
    expect(snapPoint(20, 500, [REF], { scale: 4 }).x).toBe(20);
  });

  it("un axe désactivé ne produit ni correction ni guide", () => {
    const r = snapMove(box(3, 3), [REF], { axes: { x: false } });
    expect(r.dx).toBe(0);
    expect(r.guides.x).toBeUndefined();
    expect(r.dy).toBeCloseTo(-3);
  });

  it("retient la cible la PLUS PROCHE quand plusieurs sont dans le seuil", () => {
    const other: AlignTarget = { id: "o", kind: "image", rect: { left: 6, top: 0, width: 10, height: 10 } };
    const r = snapMove(box(5, 500), [REF, other]);
    expect(r.guides.x).toEqual([6]);
  });
});

describe("snapResize — mise à l'échelle", () => {
  it("accroche le bord tiré et lui seul", () => {
    // Boîte (200,200) 100×100, poignée bas-droite amenée près de x=... aucune
    // cible proche : rien ne bouge.
    const far = snapResize({ left: 200, top: 200, width: 100, height: 100 }, "br", [REF]);
    expect(far.rect).toEqual({ left: 200, top: 200, width: 100, height: 100 });
    expect(far.guides.x).toBeUndefined();
  });

  it("bord droit tiré → accroche sur un bord de cible, le gauche ne bouge pas", () => {
    // gauche = -60, droite = 97 → accroche à 100 (bord droit de REF)
    const r = snapResize({ left: -60, top: 500, width: 157, height: 40 }, "br", [REF]);
    expect(r.rect.left).toBe(-60);
    expect(r.rect.width).toBeCloseTo(160);
    expect(r.guides.x).toEqual([100]);
  });

  it("bord gauche tiré → déplace le bord gauche ET compense la largeur", () => {
    // gauche = 3 → accroche à 0 ; droite (203) inchangée.
    const r = snapResize({ left: 3, top: 500, width: 200, height: 40 }, "tl", [REF]);
    expect(r.rect.left).toBeCloseTo(0);
    expect(r.rect.width).toBeCloseTo(203);
    expect(r.guides.x).toEqual([0]);
  });

  it("un bord NON tiré n'accroche jamais (poignée bas-droite, bord haut à 3)", () => {
    // bord bas = 303 : hors de portée des lignes de REF (0/50/100) → seul le
    // bord haut est proche d'une cible, et il ne bouge pas avec cette poignée.
    const r = snapResize({ left: 500, top: 3, width: 40, height: 300 }, "br", [REF]);
    expect(r.rect.top).toBe(3);
    expect(r.rect.height).toBe(300);
    expect(r.guides.y).toBeUndefined();
  });

  it("abandonne l'accroche plutôt que de violer la taille minimale", () => {
    // droite = 97 → accrocherait à 100, mais width max autorisée ici est... on
    // impose minWidth 200 alors que l'accroche donnerait 160.
    const r = snapResize({ left: -60, top: 500, width: 157, height: 40 }, "br", [REF], { minWidth: 200 });
    expect(r.rect.width).toBe(157);      // taille proposée conservée telle quelle
    expect(r.guides.x).toBeUndefined();  // …et AUCUN guide mensonger
  });
});

describe("snapPoint — placement avant création", () => {
  it("accroche un point isolé sur les deux axes", () => {
    const p = snapPoint(4, 96, [REF]);
    expect(p.x).toBeCloseTo(0);
    expect(p.y).toBeCloseTo(100);
    expect(p.guides).toEqual({ x: [0], y: [100] });
  });

  it("laisse le point intact hors seuil", () => {
    const p = snapPoint(400, 400, [REF]);
    expect(p).toMatchObject({ x: 400, y: 400 });
    expect(p.guides.x).toBeUndefined();
  });
});

describe("collectAlignTargets — toutes les familles d'éléments", () => {
  const img = {
    id: "i1", x: 100, y: 100, width: 40, height: 20,
    rotation: 0, locked: false, tags: [], originalWidth: 40, originalHeight: 20,
  } as BoardImage;
  const text: Annotation = { id: "t1", type: "text", x: 0, y: 0, text: "", width: 50, height: 20 };
  const memb: Annotation = { id: "m1", type: "membrane", x: 10, y: 10, width: 300, height: 200, color: "#fff", text: "" };
  const arrow: Annotation = { id: "a1", type: "arrow", x: 0, y: 0, x2: 5, y2: 5 };
  const folder = { id: "f1", name: "D", color: "#fff", x: 400, y: 0, width: 200, height: 150, childBoardId: "b2" } as CanvasFolder;
  const board = { images: [img], annotations: [text, memb, arrow], folders: [folder] } as unknown as Board;

  it("inclut images, textes, membranes et dossiers — jamais les flèches", () => {
    const kinds = collectAlignTargets(board).map((t) => t.kind).sort();
    expect(kinds).toEqual(["folder", "image", "membrane", "text"]);
  });

  it("convertit les images (ancre CENTRE) en boîte coin haut-gauche", () => {
    const t = collectAlignTargets(board).find((x) => x.id === "i1")!;
    expect(t.rect).toEqual({ left: 80, top: 90, width: 40, height: 20 });
  });

  it("exclut les éléments manipulés (pas d'auto-alignement)", () => {
    const ids = collectAlignTargets(board, ["i1", "f1"]).map((t) => t.id);
    expect(ids).toEqual(["t1", "m1"]);
  });

  it("un dossier sert de cible aux autres éléments", () => {
    const targets = collectAlignTargets(board, ["t1", "m1", "i1"]);
    // bord gauche du dossier = 400
    expect(snapMove(box(403, 900), targets).guides.x).toEqual([400]);
  });
});

describe("unionRect / rectOfImage", () => {
  it("englobe toute la sélection", () => {
    expect(unionRect([box(0, 0, 10, 10), box(90, 40, 10, 10)]))
      .toEqual({ left: 0, top: 0, width: 100, height: 50 });
  });

  it("null sur une sélection vide", () => {
    expect(unionRect([])).toBeNull();
  });

  it("rectOfImage recentre l'ancre 0.5", () => {
    expect(rectOfImage({ x: 0, y: 0, width: 10, height: 4 }))
      .toEqual({ left: -5, top: -2, width: 10, height: 4 });
  });
});

describe("sameGuides — anti re-render", () => {
  it("deux jeux vides sont identiques quelle que soit leur forme", () => {
    expect(sameGuides(null, {})).toBe(true);
    expect(sameGuides({ x: [] }, null)).toBe(true);
  });

  it("distingue des lignes différentes", () => {
    expect(sameGuides({ x: [10] }, { x: [11] })).toBe(false);
    expect(sameGuides({ x: [10] }, { x: [10] })).toBe(true);
    expect(sameGuides({ x: [10] }, { y: [10] })).toBe(false);
  });
});
