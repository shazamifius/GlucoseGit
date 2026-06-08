import { describe, expect, it } from "vitest";
import { computeResize, MIN_IMAGE_SIZE, type ResizeState } from "./imageResize";

// Image au grab : centre (100,100), 100×50 (aspect 2). Coins :
//   NW(50,75) NE(150,75) SE(150,125) SW(50,125).
function stateGrabbingSE(): ResizeState {
  // poignée SE → ancre = coin opposé NW (50,75)
  return { cx: 100, cy: 100, aspect: 2, ax: 50, ay: 75 };
}

describe("computeResize — coin opposé (défaut)", () => {
  it("garde le coin opposé (ancre) FIXE", () => {
    const s = stateGrabbingSE();
    const r = computeResize(s, 200, 130, false);
    // coin NW de la nouvelle boîte = (x - w/2, y - h/2) doit rester sur l'ancre
    expect(r.x - r.width / 2).toBeCloseTo(s.ax);
    expect(r.y - r.height / 2).toBeCloseTo(s.ay);
  });

  it("verrouille le ratio (jamais déformé)", () => {
    const s = stateGrabbingSE();
    const r = computeResize(s, 200, 130, false);
    expect(r.width / r.height).toBeCloseTo(2);
  });

  it("le centre se DÉPLACE (≠ centre d'origine)", () => {
    const s = stateGrabbingSE();
    const r = computeResize(s, 200, 130, false);
    // agrandissement vers le bas-droite → centre pousse en bas-droite
    expect(r.x).toBeGreaterThan(100);
    expect(r.y).toBeGreaterThan(100);
  });

  it("exemple numérique exact", () => {
    const s = stateGrabbingSE();
    const r = computeResize(s, 200, 130, false);
    // |200-50|=150 → h=75 (ratio 2) ; centre = ancre + w/2,h/2
    expect(r).toEqual({ x: 50 + 75, y: 75 + 37.5, width: 150, height: 75 });
  });
});

describe("computeResize — centre (Ctrl)", () => {
  it("garde le CENTRE fixe", () => {
    const s = stateGrabbingSE();
    const r = computeResize(s, 170, 130, true);
    expect(r.x).toBe(s.cx);
    expect(r.y).toBe(s.cy);
  });

  it("taille = 2× la distance centre→curseur, ratio verrouillé", () => {
    const s = stateGrabbingSE();
    const r = computeResize(s, 170, 100, true); // 70px à droite du centre
    expect(r.width).toBeCloseTo(140);
    expect(r.height).toBeCloseTo(70);
  });
});

describe("computeResize — plancher de taille", () => {
  it("ne descend pas sous MIN_IMAGE_SIZE en gardant le ratio", () => {
    const s = stateGrabbingSE();
    const r = computeResize(s, s.ax, s.ay, false); // curseur sur l'ancre
    expect(r.width).toBe(MIN_IMAGE_SIZE);
    expect(r.height).toBeCloseTo(MIN_IMAGE_SIZE / s.aspect);
  });
});
