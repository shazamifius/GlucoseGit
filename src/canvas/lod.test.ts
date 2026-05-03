// CLEANUP T-01 — Tests des fonctions pures du système LOD (Phase 2).
// Le LOD pilote le rendu de toutes les couches : régression silencieuse = projet
// inutilisable.

import { describe, it, expect } from "vitest";
import { computeLOD, shouldRenderArrow, LOD_THRESHOLDS } from "./lod";

describe("computeLOD", () => {
  // Note : computeLOD est temporairement forcé à "micro" (lod.ts:15).
  // Les tests reflètent le comportement ACTUEL.
  it("retourne toujours 'micro' (LOD désactivé temporairement)", () => {
    expect(computeLOD(0.01)).toBe("micro");
    expect(computeLOD(0.5)).toBe("micro");
    expect(computeLOD(1.0)).toBe("micro");
    expect(computeLOD(10)).toBe("micro");
  });

  it("expose des seuils cohérents (micro > meso > macro)", () => {
    expect(LOD_THRESHOLDS.macroToMeso).toBeLessThan(LOD_THRESHOLDS.mesoToMicro);
  });
});

describe("shouldRenderArrow — règle anti-spaghetti", () => {
  const baseProbe = { arrowId: "a1" };
  const noSelection = new Set<string>();

  it("rend toujours une flèche épinglée (pinned)", () => {
    expect(shouldRenderArrow(
      { ...baseProbe, pinned: true },
      { lod: "macro", selectedNodeIds: noSelection, hoveredNodeId: null, transDomainVisible: false },
    )).toBe(true);
  });

  it("rend une flèche trans-domaine si le toggle est actif", () => {
    expect(shouldRenderArrow(
      { ...baseProbe, isTransDomain: true },
      { lod: "macro", selectedNodeIds: noSelection, hoveredNodeId: null, transDomainVisible: true },
    )).toBe(true);
  });

  it("ne rend PAS une flèche trans-domaine si le toggle est désactivé", () => {
    expect(shouldRenderArrow(
      { ...baseProbe, isTransDomain: true },
      { lod: "macro", selectedNodeIds: noSelection, hoveredNodeId: null, transDomainVisible: false },
    )).toBe(false);
  });

  it("en macro, ignore la sélection (pas de flèche normale)", () => {
    expect(shouldRenderArrow(
      { ...baseProbe, sourceId: "n1" },
      { lod: "macro", selectedNodeIds: new Set(["n1"]), hoveredNodeId: null, transDomainVisible: false },
    )).toBe(false);
  });

  it("en meso, rend si la source est sélectionnée", () => {
    expect(shouldRenderArrow(
      { ...baseProbe, sourceId: "n1" },
      { lod: "meso", selectedNodeIds: new Set(["n1"]), hoveredNodeId: null, transDomainVisible: false },
    )).toBe(true);
  });

  it("en meso, rend si la cible est sélectionnée", () => {
    expect(shouldRenderArrow(
      { ...baseProbe, targetId: "n2" },
      { lod: "meso", selectedNodeIds: new Set(["n2"]), hoveredNodeId: null, transDomainVisible: false },
    )).toBe(true);
  });

  it("en meso, ne rend PAS sur hover (réservé au micro)", () => {
    expect(shouldRenderArrow(
      { ...baseProbe, sourceId: "n1" },
      { lod: "meso", selectedNodeIds: noSelection, hoveredNodeId: "n1", transDomainVisible: false },
    )).toBe(false);
  });

  it("en micro, rend sur hover", () => {
    expect(shouldRenderArrow(
      { ...baseProbe, sourceId: "n1" },
      { lod: "micro", selectedNodeIds: noSelection, hoveredNodeId: "n1", transDomainVisible: false },
    )).toBe(true);
  });

  it("en micro, ne rend pas si aucune condition", () => {
    expect(shouldRenderArrow(
      { ...baseProbe, sourceId: "n1", targetId: "n2" },
      { lod: "micro", selectedNodeIds: noSelection, hoveredNodeId: null, transDomainVisible: false },
    )).toBe(false);
  });

  it("rend si la flèche elle-même est sélectionnée (clic direct)", () => {
    expect(shouldRenderArrow(
      { ...baseProbe, sourceId: "n1", targetId: "n2" },
      { lod: "meso", selectedNodeIds: new Set(["a1"]), hoveredNodeId: null, transDomainVisible: false },
    )).toBe(true);
  });
});
