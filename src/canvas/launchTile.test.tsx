// ────────────────────────────────────────────────────────────────────────────
// R-FIL — Régression du LANCEMENT de fichier depuis une tuile.
//
// Bug historique : double-clic sur une tuile launcher (sourceFile) passait en
// mode édition de nom (postit) au lieu de lancer l'app, et le lancement ne
// marchait qu'« une fois sur 10 » (il dépendait de l'event dblclick natif).
// On vérifie ici le contrat :
//   - tuile sourceFile : double-clic → invoke("open_in_app"), JAMAIS onEdit.
//   - note normale : double-clic → onEdit, JAMAIS open_in_app.
// La détection se fait sur 2 pointerdown rapprochés (fiable), pas sur dblclick.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
import type { Annotation } from "../types";
import { useGlucoseStore } from "../store";

const invokeMock = vi.fn().mockResolvedValue(undefined);
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  convertFileSrc: (s: string) => s,
}));

import HtmlAnnotationLayer from "./HtmlAnnotationLayer";

function vpRef() {
  return { current: { x: 0, y: 0, scale: 1 } };
}

afterEach(() => {
  cleanup();
  invokeMock.mockClear();
});

describe("Tuile fichier — double-clic lance, jamais d'édition de nom", () => {
  it("launcher (sourceFile) : double-clic → open_in_app, et JAMAIS onEdit", () => {
    useGlucoseStore.setState({ activeTool: "select" });
    const onEdit = vi.fn();
    const ann = {
      id: "L1", type: "sticky", x: 0, y: 0, width: 150, height: 140,
      text: "scene.blend", sourceFile: "C:/w/scene.blend", bgColor: "#e87d0d",
    } as Annotation;

    const { container } = render(
      <HtmlAnnotationLayer
        annotations={[ann]} selectedIds={[]} editingId={null}
        vpRef={vpRef()} onSelect={vi.fn()} onEdit={onEdit} onResize={vi.fn()}
      />,
    );
    const tile = container.querySelector('[data-id="L1"]');
    expect(tile).toBeTruthy();

    // 2 pointerdown rapprochés = double-clic fiable.
    fireEvent.pointerDown(tile!, { button: 0 });
    fireEvent.pointerDown(tile!, { button: 0 });

    expect(invokeMock).toHaveBeenCalledWith("open_in_app", { path: "C:/w/scene.blend" });
    expect(onEdit).not.toHaveBeenCalled();
  });

  it("note normale (sans sourceFile) : double-clic → onEdit, et JAMAIS open_in_app", () => {
    useGlucoseStore.setState({ activeTool: "select" });
    const onEdit = vi.fn();
    const ann = {
      id: "N1", type: "sticky", x: 0, y: 0, width: 160, height: 120, text: "note",
    } as Annotation;

    const { container } = render(
      <HtmlAnnotationLayer
        annotations={[ann]} selectedIds={[]} editingId={null}
        vpRef={vpRef()} onSelect={vi.fn()} onEdit={onEdit} onResize={vi.fn()}
      />,
    );
    const tile = container.querySelector('[data-id="N1"]');
    fireEvent.pointerDown(tile!, { button: 0 });
    fireEvent.pointerDown(tile!, { button: 0 });

    expect(onEdit).toHaveBeenCalledWith("N1");
    expect(invokeMock).not.toHaveBeenCalledWith("open_in_app", expect.anything());
  });
});
