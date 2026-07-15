import { describe, it, expect } from "vitest";
import { arrowEndpoints, type ArrowAnchor } from "./ArrowSvgLayer";

// Une note de 200x100 posée en (x, y) : l'ancre est son centre, la boîte son contour.
const note = (x: number, y: number, w = 200, h = 100): ArrowAnchor => ({
  x: x + w / 2,
  y: y + h / 2,
  box: { left: x, right: x + w, top: y, bottom: y + h },
});

const len = (a: { x: number; y: number }, b: { x: number; y: number }) => Math.hypot(b.x - a.x, b.y - a.y);

describe("arrowEndpoints — ancrage au périmètre", () => {
  it("sort par le côté qui fait face à la cible, pas par le flanc", () => {
    // B est franchement SOUS A : la flèche doit sortir par le bas de A et entrer
    // par le haut de B. C'est tout l'objet de l'ancrage au périmètre — l'ancien
    // code sortait toujours par un flanc gauche/droite à mi-hauteur.
    const a = note(0, 0);
    const b = note(0, 500);
    const { start, end } = arrowEndpoints(a, b);
    expect(start.y).toBeGreaterThan(100); // sous le bas de A (y=100)
    expect(end.y).toBeLessThan(500);      // au-dessus du haut de B (y=500)
    expect(start.x).toBeCloseTo(100);     // pile sous le centre
    expect(end.x).toBeCloseTo(100);
  });

  it("sort par la droite quand la cible est à droite", () => {
    const { start, end } = arrowEndpoints(note(0, 0), note(500, 0));
    expect(start.x).toBeGreaterThan(200); // au-delà du bord droit de A
    expect(end.x).toBeLessThan(500);      // avant le bord gauche de B
    expect(start.y).toBeCloseTo(50);
  });

  it("décolle du bloc de 12px quand la place le permet", () => {
    const { start } = arrowEndpoints(note(0, 0), note(0, 500));
    expect(start.y).toBeCloseTo(112); // bas de A (100) + marge (12)
  });
});

describe("arrowEndpoints — la flèche ne s'inverse jamais", () => {
  // LE BUG : la marge de 12px était ajoutée à chaque extrémité sans regarder la
  // place disponible. Deux blocs à moins de 24px l'un de l'autre voyaient donc
  // leurs pointes se croiser, et la flèche pointait à l'envers.
  it("ne croise pas les pointes sur des blocs quasi collés (le cas du bug)", () => {
    const a = note(0, 0);        // bas à y=100
    const b = note(0, 110);      // haut à y=110 — 10px d'écart, < 2x12
    const { start, end } = arrowEndpoints(a, b);
    expect(end.y).toBeGreaterThan(start.y); // sens conservé : A est au-dessus de B
  });

  it("garde le sens pour tout écart de 0 à 40px", () => {
    for (let gap = 0; gap <= 40; gap++) {
      const { start, end } = arrowEndpoints(note(0, 0), note(0, 100 + gap));
      expect(end.y, `écart de ${gap}px`).toBeGreaterThanOrEqual(start.y);
    }
  });

  it("garde le sens à l'horizontale aussi", () => {
    for (let gap = 0; gap <= 40; gap++) {
      const { start, end } = arrowEndpoints(note(0, 0), note(200 + gap, 0));
      expect(end.x, `écart de ${gap}px`).toBeGreaterThanOrEqual(start.x);
    }
  });

  it("ne consomme jamais plus que la place disponible", () => {
    // La flèche reste plus courte que l'écart des périmètres : les marges rognent,
    // elles n'allongent pas.
    for (const gap of [0, 5, 10, 23, 24, 25, 100]) {
      const { start, end } = arrowEndpoints(note(0, 0), note(0, 100 + gap));
      expect(len(start, end), `écart de ${gap}px`).toBeLessThanOrEqual(gap + 0.001);
    }
  });
});

describe("arrowEndpoints — points de passage et extrémités libres", () => {
  it("vise le premier/dernier point de passage, pas l'autre extrémité", () => {
    // Le point de passage est à gauche : la flèche doit sortir par la GAUCHE de A,
    // alors que la cible finale est à droite.
    const { start } = arrowEndpoints(note(0, 0), note(500, 0), [{ x: -300, y: 50 }]);
    expect(start.x).toBeLessThan(0);
  });

  it("borne aussi la marge sur un point de passage très proche", () => {
    const wp = { x: 100, y: 105 }; // 5px sous le bas de A
    const { start } = arrowEndpoints(note(0, 0), note(0, 500), [wp]);
    expect(start.y).toBeLessThanOrEqual(wp.y); // ne dépasse pas son propre point
  });

  it("laisse une extrémité libre (sans boîte) exactement où elle est", () => {
    const libre: ArrowAnchor = { x: 42, y: 7 }; // pas de box
    const { start } = arrowEndpoints(libre, note(500, 0));
    expect(start).toEqual({ x: 42, y: 7 }); // aucune marge appliquée
  });

  it("ne produit pas de NaN quand les deux ancres sont confondues", () => {
    const a = note(0, 0);
    const { start, end } = arrowEndpoints(a, note(0, 0));
    for (const v of [start.x, start.y, end.x, end.y]) expect(Number.isFinite(v)).toBe(true);
  });
});
