// ────────────────────────────────────────────────────────────────────────────
// MEMB-2 — Vérification de bout en bout de la mise à l'échelle au rendu.
//
// `membraneSpace` démontre le calcul. Ici on monte les VRAIES couches et on
// regarde le style produit : un bloc rangé dans une membrane minimisée doit
// arriver à l'écran réduit et déplacé, sans que ses coordonnées stockées ne
// bougent d'un pixel.
//
// Et surtout le GARDE-FOU de la migration : sans membrane minimisée, le rendu
// doit être exactement celui d'avant — aucune transformation, aucune coordonnée
// déplacée. C'est ce qui protège tous les projets existants.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import type { Annotation } from "../types";
import { useGlucoseStore } from "../store";
import { hasScaling, itemsOfBoard, resolveItems } from "./membraneSpace";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: (s: string) => s,
}));

import HtmlAnnotationLayer from "./HtmlAnnotationLayer";
import SvgAnnotationLayer from "./SvgAnnotationLayer";

afterEach(cleanup);

const vpRef = () => ({ current: { x: 0, y: 0, scale: 1 } });

/** Membrane 200×200 dont le contenu s'étend sur 800 → échelle 0,25. */
const MINI: Annotation = {
  id: "M", type: "membrane", x: 0, y: 0, width: 200, height: 200, mode: "minimized",
} as Annotation;

const CLASSIQUE: Annotation = {
  id: "M", type: "membrane", x: 0, y: 0, width: 200, height: 200,
} as Annotation;

const texte = (membraneId?: string): Annotation => ({
  id: "T", type: "text", x: 400, y: 400, text: "bonjour",
  width: 400, height: 400, membraneId,
} as Annotation);

function geomOf(annotations: Annotation[]) {
  const board = { images: [], annotations };
  const items = itemsOfBoard(board);
  return hasScaling(items) ? resolveItems(items) : null;
}

function monterTextes(annotations: Annotation[]) {
  useGlucoseStore.setState({ activeTool: "select" });
  const geom = geomOf(annotations);
  return render(
    <HtmlAnnotationLayer
      annotations={annotations.filter((a) => a.type === "text" || a.type === "sticky")}
      selectedIds={[]} editingId={null} vpRef={vpRef()} geom={geom}
      onSelect={vi.fn()} onEdit={vi.fn()} onResize={vi.fn()}
    />,
  );
}

function monterMembranes(annotations: Annotation[]) {
  useGlucoseStore.setState({ activeTool: "select" });
  const geom = geomOf(annotations);
  return render(
    <SvgAnnotationLayer
      annotations={annotations.filter((a) => a.type === "membrane")}
      selectedIds={[]} editingId={null} vpRef={vpRef()} geom={geom}
      onSelect={vi.fn()} onEdit={vi.fn()} onResize={vi.fn()}
    />,
  );
}

// ── Le garde-fou ────────────────────────────────────────────────────────────

describe("GARDE-FOU — sans membrane minimisée, rien ne change", () => {
  it("un bloc libre est posé à ses coordonnées, sans transformation", () => {
    const { container } = monterTextes([texte()]);
    const el = container.querySelector('[data-id="T"]') as HTMLElement;
    expect(el.style.left).toBe("400px");
    expect(el.style.top).toBe("400px");
    expect(el.style.transform).toBe("");
  });

  it("un bloc dans une membrane CLASSIQUE n'est pas touché non plus", () => {
    // Classique laisse déborder : c'est tout son intérêt, et le comportement
    // historique de l'application.
    const { container } = monterTextes([CLASSIQUE, texte("M")]);
    const el = container.querySelector('[data-id="T"]') as HTMLElement;
    expect(el.style.left).toBe("400px");
    expect(el.style.transform).toBe("");
  });

  it("une membrane classique garde son translate nu", () => {
    const { container } = monterMembranes([CLASSIQUE]);
    const g = container.querySelector('[data-pick-id="M"]') as SVGGElement;
    expect(g.getAttribute("transform")).toBe("translate(0,0)");
  });
});

// ── La mise à l'échelle ─────────────────────────────────────────────────────

describe("membrane minimisée — le contenu arrive réduit à l'écran", () => {
  it("le bloc est déplacé ET mis à l'échelle", () => {
    const stocke = texte("M");
    const { container } = monterTextes([MINI, stocke]);
    const el = container.querySelector('[data-id="T"]') as HTMLElement;
    // Origine effective = 400 × 0,25 = 100 ; échelle 0,25.
    expect(el.style.left).toBe("100px");
    expect(el.style.top).toBe("100px");
    expect(el.style.transform).toBe("scale(0.25)");
    expect(el.style.transformOrigin).toBe("top left");
  });

  it("les DONNÉES ne bougent pas — c'est un rendu, pas une modification", () => {
    const stocke = texte("M") as Extract<Annotation, { type: "text" }>;
    monterTextes([MINI, stocke]);
    expect(stocke.x).toBe(400);
    expect(stocke.y).toBe(400);
    expect(stocke.width).toBe(400);
  });

  it("l'échelle plutôt qu'une largeur divisée — la police suit", () => {
    // Réduire la boîte sans réduire la police ferait déborder le texte.
    // `scale` emporte cadre, police, marges et badges d'un seul coup.
    const { container } = monterTextes([MINI, texte("M")]);
    const el = container.querySelector('[data-id="T"]') as HTMLElement;
    expect(el.style.width).toBe("400px"); // largeur NATURELLE conservée
    expect(el.style.transform).toBe("scale(0.25)");
  });

  it("une membrane rangée dans une membrane minimisée se dessine réduite", () => {
    const interne: Annotation = {
      id: "IN", type: "membrane", x: 0, y: 0, width: 800, height: 800, membraneId: "M",
    } as Annotation;
    const { container } = monterMembranes([MINI, interne]);
    const g = container.querySelector('[data-pick-id="IN"]') as SVGGElement;
    expect(g.getAttribute("transform")).toBe("translate(0,0) scale(0.25)");
  });

  it("un voisin resté libre n'est pas entraîné", () => {
    const libre: Annotation = {
      id: "L", type: "text", x: 5000, y: 5000, text: "loin", width: 100, height: 100,
    } as Annotation;
    const { container } = monterTextes([MINI, texte("M"), libre]);
    const el = container.querySelector('[data-id="L"]') as HTMLElement;
    expect(el.style.left).toBe("5000px");
    expect(el.style.transform).toBe("");
  });
});
