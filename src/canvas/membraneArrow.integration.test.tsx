// ────────────────────────────────────────────────────────────────────────────
// MEMB-5 — Les flèches dans une membrane minimisée, de bout en bout.
//
// LE TROU QU'ON BOUCHE. `projectBoard` rendait les flèches telles quelles, et
// la couche qui les dessine lit la géométrie des nœuds DIRECTEMENT dans le
// board qu'on lui passe. On lui passait le board naturel : une flèche entre
// deux images rangées dans une membrane minimisée était donc tracée à la
// position que ces images auraient SANS la membrane — c'est-à-dire loin
// d'elles, en travers du canvas. C'est le seul endroit où la fonctionnalité
// mentait.
//
// LE CORRECTIF NE TOUCHE PAS LA COUCHE. Elle n'a aucune notion de membrane à
// apprendre : on lui donne le board projeté, et ses calculs d'ancrage tombent
// justes tout seuls. C'est la promesse de `membraneSpace` — une seule couture,
// pas une rustine par site d'appel.
//
// On monte donc la VRAIE couche et on lit le tracé produit, parce que c'est la
// seule façon de prouver que la projection arrive jusqu'au pixel.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import type { Annotation, Board, BoardImage } from "../types";
import { hasScaling, itemsOfBoard, projectBoard, resolveItems } from "./membraneSpace";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: (s: string) => s,
}));

import ArrowSvgLayer from "./ArrowSvgLayer";

afterEach(cleanup);

const vpRef = () => ({ current: { x: 0, y: 0, scale: 1 } });

function img(id: string, x: number, y: number, membraneId?: string): BoardImage {
  return {
    id, x, y, width: 100, height: 100, rotation: 0, locked: false, tags: [],
    originalWidth: 100, originalHeight: 100,
    ...(membraneId ? { membraneId } : {}),
  } as BoardImage;
}

/** Membrane 200×200 dont le contenu s'étend jusqu'à 800 → échelle 0,25. */
const MINI = {
  id: "M", type: "membrane", x: 0, y: 0, width: 200, height: 200, mode: "minimized",
} as Annotation;

const CLASSIQUE = {
  id: "M", type: "membrane", x: 0, y: 0, width: 200, height: 200,
} as Annotation;

const FLECHE = {
  id: "F", type: "arrow", x: 0, y: 0, x2: 0, y2: 0, sourceId: "A", targetId: "B",
} as Annotation;

/** Exactement la dérivation que fait le canvas avant de rendre la couche. */
function boardVisible(annotations: Annotation[], images: BoardImage[]): Board {
  const base = { images, annotations };
  const items = itemsOfBoard(base);
  const geom = hasScaling(items) ? resolveItems(items) : null;
  const p = projectBoard(base, geom);
  return {
    id: "main", name: "B", images: p.images, annotations: p.annotations,
    panels: [], zones: [], folders: [],
    viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0,
  } as Board;
}

function tracer(annotations: Annotation[], images: BoardImage[]) {
  const { container } = render(
    <ArrowSvgLayer
      board={boardVisible(annotations, images)}
      vpRef={vpRef()} editingId={null} selectedIds={[]} onSelect={vi.fn()}
    />,
  );
  const d = container.querySelector("path")?.getAttribute("d") ?? "";
  const nombres = d.match(/-?\d+(?:\.\d+)?/g)?.map(Number) ?? [];
  return { d, xs: nombres.filter((_, i) => i % 2 === 0), ys: nombres.filter((_, i) => i % 2 === 1) };
}

// ── Le trou ─────────────────────────────────────────────────────────────────

describe("une flèche entre deux nœuds d'une membrane minimisée", () => {
  // A et B sont rangés dans la membrane ; ETALON fixe l'étendue à 800, donc
  // l'échelle à 0,25. Sans projection, le tracé partirait vers 150 et 550.
  const images = [
    img("A", 150, 150, "M"),
    img("B", 550, 550, "M"),
    img("ETALON", 750, 750, "M"),
  ];

  it("est tracée DANS la membrane, pas là où les nœuds seraient sans elle", () => {
    const { xs, ys } = tracer([MINI, FLECHE], images);
    expect(xs.length).toBeGreaterThan(0);
    // La membrane fait 200×200 depuis l'origine. Tout le tracé tient dedans,
    // avec la marge de la sortie de périmètre des ancres.
    for (const x of xs) expect(x).toBeLessThanOrEqual(220);
    for (const y of ys) expect(y).toBeLessThanOrEqual(220);
  });

  it("avant le correctif, le même tracé sortait à plus du double", () => {
    // On refait le calcul SANS projeter : c'est ce que la couche recevait, et
    // ce que l'utilisateur voyait — une flèche en travers du canvas.
    const { container } = render(
      <ArrowSvgLayer
        board={{
          id: "main", name: "B", images, annotations: [MINI, FLECHE],
          panels: [], zones: [], folders: [],
          viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0,
        } as Board}
        vpRef={vpRef()} editingId={null} selectedIds={[]} onSelect={vi.fn()}
      />,
    );
    const d = container.querySelector("path")?.getAttribute("d") ?? "";
    const xs = (d.match(/-?\d+(?:\.\d+)?/g) ?? []).map(Number).filter((_, i) => i % 2 === 0);
    expect(Math.max(...xs)).toBeGreaterThan(220);
  });
});

// ── Le garde-fou ────────────────────────────────────────────────────────────

describe("GARDE-FOU — sans membrane minimisée, le tracé est celui d'avant", () => {
  const images = [img("A", 150, 150, "M"), img("B", 550, 550, "M")];

  it("une membrane CLASSIQUE ne déplace aucune flèche", () => {
    const avec = tracer([CLASSIQUE, FLECHE], images);
    const sans = tracer([FLECHE], images.map((i) => img(i.id, i.x, i.y)));
    expect(avec.d).toBe(sans.d);
  });

  it("et le board n'est même pas recopié", () => {
    const base = { images: images.map((i) => img(i.id, i.x, i.y)), annotations: [FLECHE] };
    const items = itemsOfBoard(base);
    expect(hasScaling(items)).toBe(false);
    expect(projectBoard(base, null)).toBe(base);
  });
});
