import { describe, it, expect } from "vitest";
import { Project, Board, Annotation } from "../../types";
import { buildScene } from "./scene";

function makeProject(annotations: Annotation[]): Project {
  const board: Board = {
    id: "b1", name: "Board 1",
    images: [], annotations, panels: [],
    viewport: { x: 0, y: 0, scale: 1 },
    zones: [], folders: [],
    createdAt: 0, updatedAt: 0,
  };
  return {
    version: "2.0.0", name: "Test",
    boards: [board], activeBoardId: "b1",
    presets: [], createdAt: 0, updatedAt: 0,
  };
}

const card = (id: string, x: number, y: number, text: string): Annotation =>
  ({ id, type: "text", x, y, text, width: 340, height: 120 } as Annotation);

describe("buildScene", () => {
  it("produit des cartes positionnées et une bbox englobante avec marge", () => {
    const scene = buildScene(makeProject([
      card("a", 0, 0, "### Alpha\ncorps"),
      card("b", 1000, 800, "### Beta\ncorps"),
    ]));
    expect(scene.cards.length).toBe(2);
    // bbox déborde le contenu (marge 120)
    expect(scene.bbox.left).toBeLessThan(0);
    expect(scene.bbox.top).toBeLessThan(0);
    expect(scene.bbox.right).toBeGreaterThan(1000 + 340);
    expect(scene.width).toBeGreaterThan(0);
    expect(scene.height).toBeGreaterThan(0);
  });

  it("attribue une couleur aura HSL à chaque carte", () => {
    const scene = buildScene(makeProject([card("a", 0, 0, "x")]));
    expect(scene.cards[0].auraColor).toMatch(/^hsl\(/);
  });

  it("route une flèche entre deux cartes liées", () => {
    const scene = buildScene(makeProject([
      card("a", 0, 0, "Alpha"),
      card("b", 800, 0, "Beta"),
      { id: "arr", type: "arrow", x: 0, y: 0, x2: 800, y2: 0, sourceId: "a", targetId: "b", predicate: "inspire" } as Annotation,
    ]));
    expect(scene.arrows.length).toBe(1);
    const a = scene.arrows[0];
    expect(a.sourceId).toBe("a");
    expect(a.targetId).toBe("b");
    expect(a.points.length).toBeGreaterThanOrEqual(2);
    expect(a.colStart).toMatch(/^hsl\(/);
    expect(a.predicateLabel).toBeTruthy();
  });

  it("inclut les membranes", () => {
    const scene = buildScene(makeProject([
      { id: "m", type: "membrane", x: -50, y: -50, width: 500, height: 400, text: "Zone A" } as Annotation,
      card("a", 0, 0, "x"),
    ]));
    expect(scene.membranes.length).toBe(1);
    expect(scene.membranes[0].text).toBe("Zone A");
  });

  it("fallback bbox sur board vide", () => {
    const scene = buildScene(makeProject([]));
    expect(scene.width).toBeGreaterThan(0);
    expect(scene.height).toBeGreaterThan(0);
  });
});
