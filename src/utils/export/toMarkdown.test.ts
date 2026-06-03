import { describe, it, expect } from "vitest";
import { Project, Board, Annotation } from "../../types";
import { projectToMarkdown, cardTitle } from "./toMarkdown";

function makeProject(annotations: Annotation[]): Project {
  const board: Board = {
    id: "b1", name: "Mon Board",
    images: [], annotations, panels: [],
    viewport: { x: 0, y: 0, scale: 1 },
    zones: [], folders: [], createdAt: 0, updatedAt: 0,
  };
  return { version: "2.0.0", name: "Projet", boards: [board], activeBoardId: "b1", presets: [], createdAt: 0, updatedAt: 0 };
}

const card = (id: string, x: number, y: number, text: string): Annotation =>
  ({ id, type: "text", x, y, text, width: 340, height: 120 } as Annotation);

describe("cardTitle", () => {
  it("prend le 1er titre markdown", () => {
    expect(cardTitle("### Newton\ncorps")).toBe("Newton");
  });
  it("retombe sur la 1re ligne nue", () => {
    expect(cardTitle("Leibniz est important")).toContain("Leibniz");
  });
});

describe("projectToMarkdown", () => {
  it("structure zones, cartes et liens", () => {
    const md = projectToMarkdown(makeProject([
      { id: "m", type: "membrane", x: -50, y: -50, width: 900, height: 600, text: "Physique" } as Annotation,
      card("a", 0, 0, "### Newton\nGravitation universelle."),
      card("b", 400, 0, "### Leibniz\nCalcul infinitésimal."),
      { id: "arr", type: "arrow", x: 0, y: 0, x2: 400, y2: 0, sourceId: "a", targetId: "b", predicate: "contredit", longText: "Querelle de la priorité." } as Annotation,
    ]));
    expect(md).toContain("# Projet — Mon Board");
    expect(md).toContain("## Physique");
    expect(md).toContain("### Newton");
    expect(md).toContain("### Leibniz");
    expect(md).toContain("## Liens");
    expect(md).toContain("**Newton**");
    expect(md).toContain("contredit");
    expect(md).toContain("Querelle de la priorité");
  });

  it("met les cartes hors-zone sous Autres", () => {
    const md = projectToMarkdown(makeProject([
      { id: "m", type: "membrane", x: 5000, y: 5000, width: 100, height: 100, text: "Loin" } as Annotation,
      card("a", 0, 0, "### Orphelin\ncorps"),
    ]));
    expect(md).toContain("### Orphelin");
  });
});
