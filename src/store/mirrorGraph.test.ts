// CLEANUP T-01 — Tests du cycle detector des miroirs de dossiers (Phase 4).
// Critique pour la sécurité : un échec ici = crash Inception en runtime.

import { describe, it, expect } from "vitest";
import { wouldCreateMirrorCycle, findBoardContainingFolder } from "./mirrorGraph";
import type { Board } from "../types";

function makeBoard(id: string, folders: { id: string; childBoardId: string }[] = []): Board {
  return {
    id,
    name: id,
    images: [],
    annotations: [],
    panels: [],
    viewport: { x: 0, y: 0, scale: 1 },
    zones: [],
    folders: folders.map((f) => ({
      id: f.id, name: f.id, color: "#fff",
      x: 0, y: 0, width: 100, height: 100,
      childBoardId: f.childBoardId,
    })),
    createdAt: 0,
    updatedAt: 0,
  };
}

describe("wouldCreateMirrorCycle", () => {
  it("retourne false quand le dossier original n'existe pas", () => {
    const boards: Board[] = [makeBoard("main")];
    expect(wouldCreateMirrorCycle(boards, "ghost-folder", "main")).toBe(false);
  });

  it("retourne true si on tente de placer un miroir d'un dossier dans son propre child", () => {
    // main contient folderA (child=childA)
    // childA est vide
    // Placer un miroir de folderA dans childA → cycle (folderA contiendrait lui-même)
    const boards: Board[] = [
      makeBoard("main", [{ id: "folderA", childBoardId: "childA" }]),
      makeBoard("childA"),
    ];
    expect(wouldCreateMirrorCycle(boards, "folderA", "childA")).toBe(true);
  });

  it("retourne true sur cycle indirect A→B→A", () => {
    // main contient folderA (child=childA)
    // childA contient folderB (child=childB)
    // childB est vide.
    // Placer un miroir de folderA dans childB → cycle (childB ⊂ childA, et y mettre A
    // ferme la boucle)
    const boards: Board[] = [
      makeBoard("main", [{ id: "folderA", childBoardId: "childA" }]),
      makeBoard("childA", [{ id: "folderB", childBoardId: "childB" }]),
      makeBoard("childB"),
    ];
    expect(wouldCreateMirrorCycle(boards, "folderA", "childB")).toBe(true);
  });

  it("retourne false quand les dossiers sont des siblings indépendants", () => {
    // main contient folderA et folderB côte à côte
    // Placer un miroir de folderA dans childB → pas de cycle
    const boards: Board[] = [
      makeBoard("main", [
        { id: "folderA", childBoardId: "childA" },
        { id: "folderB", childBoardId: "childB" },
      ]),
      makeBoard("childA"),
      makeBoard("childB"),
    ];
    expect(wouldCreateMirrorCycle(boards, "folderA", "childB")).toBe(false);
  });

  it("retourne true avec un cycle profond A→B→C→A", () => {
    const boards: Board[] = [
      makeBoard("main", [{ id: "fA", childBoardId: "childA" }]),
      makeBoard("childA", [{ id: "fB", childBoardId: "childB" }]),
      makeBoard("childB", [{ id: "fC", childBoardId: "childC" }]),
      makeBoard("childC"),
    ];
    expect(wouldCreateMirrorCycle(boards, "fA", "childC")).toBe(true);
  });

  it("ne boucle pas indéfiniment sur un graphe déjà cyclique (visited set)", () => {
    // Graphe pré-cyclique : A→B et B→A simultanément (situation théoriquement
    // déjà invalide, mais notre helper doit terminer quand même)
    const boards: Board[] = [
      makeBoard("main", [
        { id: "fA", childBoardId: "childA" },
        { id: "fB", childBoardId: "childB" },
      ]),
      makeBoard("childA", [{ id: "fB-mirror", childBoardId: "childB" }]),
      makeBoard("childB", [{ id: "fA-mirror", childBoardId: "childA" }]),
    ];
    // Termine sans timeout grâce au `visited`
    const result = wouldCreateMirrorCycle(boards, "fA", "childA");
    expect(typeof result).toBe("boolean");
  });
});

describe("findBoardContainingFolder", () => {
  it("trouve le board parent d'un dossier", () => {
    const boards: Board[] = [
      makeBoard("main", [{ id: "f1", childBoardId: "child" }]),
      makeBoard("child"),
    ];
    expect(findBoardContainingFolder(boards, "f1")?.id).toBe("main");
  });

  it("retourne undefined si le folder est introuvable", () => {
    const boards: Board[] = [makeBoard("main")];
    expect(findBoardContainingFolder(boards, "ghost")).toBeUndefined();
  });
});
