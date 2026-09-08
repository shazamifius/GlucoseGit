// ────────────────────────────────────────────────────────────────────────────
// MEMB-2 — Vérification de BOUT EN BOUT du masquage en mode Focus.
//
// Les tests de `membraneFocus.test.ts` démontrent la décision et le filtre.
// Ceux-ci vont plus loin : ils font tourner la dérivation RÉELLE (`focusView`,
// celle qu'exécute GlucoseCanvas), passent son résultat aux VRAIES couches, et
// regardent le DOM produit. Un élément masqué n'est donc pas seulement absent
// d'un Set : il est absent de l'écran.
//
// C'est la garantie la plus forte atteignable sans lancer l'application — le
// rendu React est déterministe, et les couches sont du DOM pur (le seul étage
// non couvert ici est PixiJS, qui ne dessine que les images).
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import type { Annotation, Board, BoardImage } from "../types";
import { focusView } from "./membraneFocus";
import { itemsOfBoard } from "./membraneSpace";
import { useGlucoseStore } from "../store";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: (s: string) => s,
}));

import HtmlAnnotationLayer from "./HtmlAnnotationLayer";
import SvgAnnotationLayer from "./SvgAnnotationLayer";

afterEach(cleanup);

function vpRef() {
  return { current: { x: 0, y: 0, scale: 1 } };
}

const membrane = (id: string, x: number, y: number, w: number, h: number): Annotation =>
  ({ id, type: "membrane", x, y, width: w, height: h, mode: "minimized", color: "#60a5fa" }) as Annotation;

const texte = (id: string, x: number, y: number, membraneId?: string): Annotation =>
  ({ id, type: "text", x, y, text: id, width: 120, height: 40, membraneId }) as Annotation;

const image = (id: string, x: number, y: number, membraneId?: string): BoardImage => ({
  id, x, y, width: 100, height: 100, rotation: 0, locked: false, tags: [],
  originalWidth: 100, originalHeight: 100, membraneId,
});

/** Board minimal + la dérivation exacte du canvas. */
function view(board: Pick<Board, "images" | "annotations">, focusedId: string | null) {
  return focusView(board, itemsOfBoard(board), focusedId);
}

function renderTexts(annotations: Annotation[]) {
  useGlucoseStore.setState({ activeTool: "select" });
  return render(
    <HtmlAnnotationLayer
      annotations={annotations.filter((a) => a.type === "text" || a.type === "sticky")}
      selectedIds={[]} editingId={null} vpRef={vpRef()}
      onSelect={vi.fn()} onEdit={vi.fn()} onResize={vi.fn()}
    />,
  );
}

function renderMembranes(annotations: Annotation[]) {
  useGlucoseStore.setState({ activeTool: "select" });
  return render(
    <SvgAnnotationLayer
      annotations={annotations.filter((a) => a.type === "membrane")}
      selectedIds={[]} editingId={null} vpRef={vpRef()}
      onSelect={vi.fn()} onEdit={vi.fn()} onResize={vi.fn()}
    />,
  );
}

describe("Focus — ce qui est masqué est réellement absent du DOM", () => {
  const board = {
    images: [image("img-dedans", 100, 100, "M"), image("img-dehors", 9000, 9000)],
    annotations: [
      membrane("M", 0, 0, 400, 400),
      membrane("AUTRE", 8000, 8000, 400, 400),
      texte("txt-dedans", 50, 50, "M"),
      texte("txt-dehors", 9000, 9000),
    ],
  };

  it("hors focus, absolument tout est rendu", () => {
    const v = view(board, null);
    expect(v.visible).toBeNull();
    const { container } = renderTexts(v.annotations);
    expect(container.querySelector('[data-id="txt-dedans"]')).toBeTruthy();
    expect(container.querySelector('[data-id="txt-dehors"]')).toBeTruthy();
  });

  it("sous focus, le texte du dehors DISPARAÎT et celui du dedans reste", () => {
    const v = view(board, "M");
    const { container } = renderTexts(v.annotations);
    expect(container.querySelector('[data-id="txt-dedans"]')).toBeTruthy();
    expect(container.querySelector('[data-id="txt-dehors"]')).toBeNull();
  });

  it("sous focus, l'autre membrane disparaît et la focalisée reste", () => {
    const v = view(board, "M");
    const { container } = renderMembranes(v.annotations);
    expect(container.querySelector('[data-pick-id="M"]')).toBeTruthy();
    expect(container.querySelector('[data-pick-id="AUTRE"]')).toBeNull();
  });

  it("les dossiers sont retirés sous focus, rendus hors focus", () => {
    expect(view(board, "M").folders).toEqual([]);
    const withFolders = { ...board, folders: [{ id: "F" }] } as never;
    expect(view(withFolders, null).folders).toHaveLength(1);
  });

  it("le contenu d'une AUTRE membrane n'est pas récupéré au passage", () => {
    const b = {
      images: [],
      annotations: [
        membrane("M", 0, 0, 400, 400),
        membrane("AUTRE", 0, 0, 400, 400),
        texte("a-autrui", 50, 50, "AUTRE"),
      ],
    };
    const { container } = renderTexts(view(b, "M").annotations);
    expect(container.querySelector('[data-id="a-autrui"]')).toBeNull();
  });
});

describe("Focus — un projet d'AVANT la fonctionnalité reste utilisable", () => {
  // Aucun membraneId stocké : sans le filet géométrique, focaliser une membrane
  // historique donnerait un écran vide.
  const legacy = {
    images: [],
    annotations: [
      { id: "M", type: "membrane", x: 0, y: 0, width: 400, height: 400 } as Annotation,
      texte("historique", 100, 100),
      texte("loin", 9000, 9000),
    ],
  };

  it("le contenu géométriquement dedans s'affiche quand même", () => {
    const { container } = renderTexts(view(legacy, "M").annotations);
    expect(container.querySelector('[data-id="historique"]')).toBeTruthy();
    expect(container.querySelector('[data-id="loin"]')).toBeNull();
  });
});

describe("Focus — les flèches suivent leurs extrémités", () => {
  const fleche = (id: string, over: Partial<Annotation> = {}): Annotation =>
    ({ id, type: "arrow", x: 10, y: 10, x2: 100, y2: 100, ...over }) as Annotation;

  const b = {
    images: [],
    annotations: [
      membrane("M", 0, 0, 400, 400),
      texte("A", 10, 10, "M"),
      texte("B", 100, 100, "M"),
      texte("Z", 9000, 9000),
      fleche("interne", { sourceId: "A", targetId: "B" }),
      fleche("sortante", { sourceId: "A", targetId: "Z" }),
      fleche("libre"),
      fleche("libre-dehors", { x: 9000, y: 9000, x2: 9100, y2: 9100 }),
    ],
  };

  it("une flèche entre deux membres reste, une flèche qui sort disparaît", () => {
    const ids = view(b, "M").annotations.map((a) => a.id);
    expect(ids).toContain("interne");
    expect(ids).not.toContain("sortante");
  });

  it("une flèche libre suit ses deux extrémités", () => {
    const ids = view(b, "M").annotations.map((a) => a.id);
    expect(ids).toContain("libre");
    expect(ids).not.toContain("libre-dehors");
  });
});
