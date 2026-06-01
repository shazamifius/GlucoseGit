// ────────────────────────────────────────────────────────────────────────────
// NAV-1 — Régression : on entre dans un dossier UNIQUEMENT quand il est le seul
// visible à l'écran. Scénario du bug : deux dossiers côte à côte, on zoome vers
// l'un, l'autre est encore visible → on NE DOIT PAS entrer (avant : on entrait
// dans le mauvais).
// ────────────────────────────────────────────────────────────────────────────

import { describe, expect, it } from "vitest";
import { classifyWheel, type FolderBox, folderToEnter, visibleWorldRect } from "./navigation";

const SCREEN_W = 1920;
const SCREEN_H = 1080;
const COVERAGE = 0.6;
const MIN_SCALE = 0.25;

/** Construit un viewport qui CENTRE un point monde (cx,cy) à une échelle donnée. */
function centerOn(cx: number, cy: number, scale: number) {
  return { x: SCREEN_W / 2 - cx * scale, y: SCREEN_H / 2 - cy * scale, scale };
}

describe("folderToEnter — NAV-1", () => {
  it("deux dossiers côte à côte visibles → n'entre PAS (le bug historique)", () => {
    // Dossier A à gauche, dossier B à droite, adjacents.
    const A: FolderBox = { id: "A", x: 0, y: 0, width: 600, height: 600 };
    const B: FolderBox = { id: "B", x: 700, y: 0, width: 600, height: 600 };
    // On vise A (centre de A sous le centre écran), zoom déjà fort.
    const vp = centerOn(300, 300, 2.2);
    // Vérifie que les deux croisent bien le viewport à ce zoom…
    const { left, right } = visibleWorldRect(vp, SCREEN_W, SCREEN_H);
    expect(left).toBeLessThan(700 + 600); // B commence avant le bord droit
    expect(right).toBeGreaterThan(700); // B est (au moins partiellement) visible
    // … donc on n'entre pas : ambiguïté.
    expect(folderToEnter([A, B], vp, SCREEN_W, SCREEN_H, COVERAGE, MIN_SCALE)).toBeNull();
  });

  it("on zoome jusqu'à ce que seul A reste visible → entre dans A", () => {
    const A: FolderBox = { id: "A", x: 0, y: 0, width: 600, height: 600 };
    const B: FolderBox = { id: "B", x: 700, y: 0, width: 600, height: 600 };
    // Zoom fort centré sur A : à scale 3.5, l'écran (1920) montre 1920/3.5≈549px
    // de monde → B (qui commence à 700) est hors champ.
    const vp = centerOn(300, 300, 3.5);
    expect(folderToEnter([A, B], vp, SCREEN_W, SCREEN_H, COVERAGE, MIN_SCALE)).toBe("A");
  });

  it("un seul dossier mais trop petit à l'écran → n'entre pas (plancher couverture)", () => {
    const A: FolderBox = { id: "A", x: 0, y: 0, width: 400, height: 400 };
    // Dézoom : A occupe une petite fraction de l'écran.
    const vp = centerOn(200, 200, 0.5); // covW = 400*0.5/1920 ≈ 0.10
    expect(folderToEnter([A], vp, SCREEN_W, SCREEN_H, COVERAGE, MIN_SCALE)).toBeNull();
  });

  it("un seul dossier qui remplit l'écran → entre", () => {
    const A: FolderBox = { id: "A", x: 0, y: 0, width: 1200, height: 1200 };
    const vp = centerOn(600, 600, 1.0); // covW = 1200/1920 ≈ 0.625 ≥ 0.6
    expect(folderToEnter([A], vp, SCREEN_W, SCREEN_H, COVERAGE, MIN_SCALE)).toBe("A");
  });

  it("GROS dossier : déclenche à un zoom plus FAIBLE qu'un petit (taille prise en compte)", () => {
    const big: FolderBox = { id: "big", x: 0, y: 0, width: 4000, height: 4000 };
    // À scale 0.3 le gros remplit déjà : 4000*0.3/1920 ≈ 0.625 ≥ 0.6.
    const vp = centerOn(2000, 2000, 0.3);
    expect(folderToEnter([big], vp, SCREEN_W, SCREEN_H, COVERAGE, MIN_SCALE)).toBe("big");
  });

  it("zoom trop faible (sous le plancher anti-jitter) → n'entre pas", () => {
    const A: FolderBox = { id: "A", x: 0, y: 0, width: 100000, height: 100000 };
    const vp = centerOn(50000, 50000, 0.1); // sous MIN_SCALE
    expect(folderToEnter([A], vp, SCREEN_W, SCREEN_H, COVERAGE, MIN_SCALE)).toBeNull();
  });

  it("aucun dossier sous le viewport (zoomé dans le vide entre dossiers) → n'entre pas", () => {
    const A: FolderBox = { id: "A", x: 0, y: 0, width: 300, height: 300 };
    const B: FolderBox = { id: "B", x: 5000, y: 5000, width: 300, height: 300 };
    // Viewport centré loin des deux, zoomé fort → rien de visible.
    const vp = centerOn(2500, 2500, 3.0);
    expect(folderToEnter([A, B], vp, SCREEN_W, SCREEN_H, COVERAGE, MIN_SCALE)).toBeNull();
  });
});

describe("classifyWheel — NAV-2 (pavé tactile)", () => {
  const base = { deltaX: 0, deltaY: 0, deltaMode: 0, ctrlKey: false } as const;

  it("pincement tactile (ctrlKey synthétisé) → zoom", () => {
    expect(classifyWheel({ ...base, ctrlKey: true, deltaY: -3, wheelDeltaY: 9 })).toBe("zoom");
  });

  it("ctrl + molette souris → zoom", () => {
    expect(classifyWheel({ ...base, ctrlKey: true, deltaY: 100, wheelDeltaY: -120 })).toBe("zoom");
  });

  it("molette souris classique (cran ±120, vertical pur) → zoom", () => {
    expect(classifyWheel({ ...base, deltaY: -100, wheelDeltaY: 120 })).toBe("zoom");
    expect(classifyWheel({ ...base, deltaY: 100, wheelDeltaY: -120 })).toBe("zoom");
    expect(classifyWheel({ ...base, deltaY: 200, wheelDeltaY: -240 })).toBe("zoom");
  });

  it("glissement 2 doigts vertical (petits deltas, pas multiple de 120) → pan", () => {
    expect(classifyWheel({ ...base, deltaY: 12, wheelDeltaY: -36 })).toBe("pan");
    expect(classifyWheel({ ...base, deltaY: -7, wheelDeltaY: 21 })).toBe("pan");
  });

  it("glissement 2 doigts horizontal (deltaX ≠ 0) → pan", () => {
    expect(classifyWheel({ ...base, deltaX: 14, deltaY: 0, wheelDeltaY: 0 })).toBe("pan");
  });

  it("glissement 2 doigts diagonal → pan", () => {
    expect(classifyWheel({ ...base, deltaX: 6, deltaY: 9, wheelDeltaY: -27 })).toBe("pan");
  });

  it("composante horizontale présente → jamais classé comme molette souris", () => {
    // wheelDeltaY multiple de 120 MAIS deltaX ≠ 0 → c'est un pavé tactile → pan.
    expect(classifyWheel({ ...base, deltaX: 5, deltaY: 100, wheelDeltaY: -120 })).toBe("pan");
  });
});
