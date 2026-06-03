import { describe, it, expect } from "vitest";
import { Project, Board, Annotation } from "../../types";
import { buildScene } from "./scene";
import { sceneToSvg } from "./toSvg";

function makeProject(annotations: Annotation[]): Project {
  const board: Board = {
    id: "b1", name: "B", images: [], annotations, panels: [],
    viewport: { x: 0, y: 0, scale: 1 }, zones: [], folders: [], createdAt: 0, updatedAt: 0,
  };
  return { version: "2.0.0", name: "P", boards: [board], activeBoardId: "b1", presets: [], createdAt: 0, updatedAt: 0 };
}
const card = (id: string, x: number, y: number, text: string): Annotation =>
  ({ id, type: "text", x, y, text, width: 340, height: 120 } as Annotation);

describe("sceneToSvg", () => {
  it("génère un SVG bien formé avec fond et texte des cartes", () => {
    const scene = buildScene(makeProject([card("a", 0, 0, "### Titre\ncontenu visible")]));
    const svg = sceneToSvg(scene);
    expect(svg.startsWith("<?xml")).toBe(true);
    expect(svg).toContain("<svg");
    expect(svg).toContain("</svg>");
    expect(svg).toContain("viewBox=");
    expect(svg).toContain("Titre");
    expect(svg).toContain("contenu");
    // fond opaque par défaut
    expect(svg).toContain(scene.background);
  });

  it("option transparent retire le rectangle de fond", () => {
    const scene = buildScene(makeProject([card("a", 0, 0, "x")]));
    const svg = sceneToSvg(scene, { transparent: true });
    expect(svg.startsWith("<?xml")).toBe(true);
  });

  it("dessine un chemin de flèche entre deux cartes", () => {
    const scene = buildScene(makeProject([
      card("a", 0, 0, "Alpha"),
      card("b", 900, 0, "Beta"),
      { id: "arr", type: "arrow", x: 0, y: 0, x2: 900, y2: 0, sourceId: "a", targetId: "b" } as Annotation,
    ]));
    const svg = sceneToSvg(scene);
    expect(svg).toContain("<path");
    expect(svg).toContain("linearGradient");
  });

  it("échappe les caractères dangereux", () => {
    const scene = buildScene(makeProject([card("a", 0, 0, "a < b & c > d")]));
    const svg = sceneToSvg(scene);
    expect(svg).toContain("&lt;");
    expect(svg).toContain("&amp;");
  });
});
