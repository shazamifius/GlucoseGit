// ────────────────────────────────────────────────────────────────────────────
// Smoke tests composants : monte chaque composant principal avec des props
// réalistes, déclenche un re-render, vérifie qu'aucune erreur React n'est
// jetée (#310 hooks count mismatch, "setState during render", etc.).
//
// Si un composant viole les règles des Hooks ou setState pendant render,
// React jette une erreur qu'on attrape ici.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { type RefObject } from "react";
import { useGlucoseStore } from "./store";
import type {
  Annotation, ArrowAnnotation, MembraneAnnotation, StickyAnnotation, TextAnnotation,
} from "./types";
import { nanoid } from "./utils/nanoid";

// Console.error intercept : faire péter le test sur toute erreur React.
// Sans ça, les `console.error` de React (incluant #310) ne font PAS échouer.
const realError = console.error;
let recordedErrors: string[] = [];
function startCaptureReactErrors() {
  recordedErrors = [];
  console.error = (...args: unknown[]) => {
    recordedErrors.push(args.map(String).join(" "));
    // On garde aussi la sortie pour debug
    realError(...args);
  };
}
function endCaptureReactErrors() {
  console.error = realError;
  return recordedErrors;
}

afterEach(() => {
  cleanup();
  console.error = realError;
});

// ─────────── Factories ──────────────────────────────────────────────
const mkText = (o: Partial<TextAnnotation> = {}): TextAnnotation =>
  ({ id: nanoid(), type: "text", x: 0, y: 0, text: "hello", ...o });
const mkSticky = (o: Partial<StickyAnnotation> = {}): StickyAnnotation =>
  ({ id: nanoid(), type: "sticky", x: 0, y: 0, text: "note", width: 160, height: 120, ...o });
const mkArrow = (o: Partial<ArrowAnnotation> = {}): ArrowAnnotation =>
  ({ id: nanoid(), type: "arrow", x: 0, y: 0, x2: 100, y2: 0, ...o });
const mkMembrane = (o: Partial<MembraneAnnotation> = {}): MembraneAnnotation =>
  ({ id: nanoid(), type: "membrane", x: 0, y: 0, width: 200, height: 160, color: "#60a5fa", ...o });

function makeViewportRef(): RefObject<{ x: number; y: number; scale: number }> {
  return { current: { x: 0, y: 0, scale: 1 } };
}

// ─────────── HtmlAnnotationLayer ────────────────────────────────────
describe("HtmlAnnotationLayer", () => {
  // Lazy-load pour ne pas crash si les imports indirects (Pixi, etc.) sont lourds.
  it("monte avec annotations mixtes des 4 types et re-render sans crash", async () => {
    const HtmlAnnotationLayer = (await import("./canvas/HtmlAnnotationLayer")).default;

    const annotations: Annotation[] = [
      mkText({ id: "t1", text: "First text" }),
      mkSticky({ id: "s1", text: "First sticky" }),
      mkSticky({ id: "s2", text: "Op sticky", operator: "AND" }),
      mkArrow({ id: "a1" }),
      mkMembrane({ id: "m1" }),
    ];

    startCaptureReactErrors();
    const vpRef = makeViewportRef();
    const { rerender } = render(
      <HtmlAnnotationLayer
        annotations={annotations}
        selectedIds={[]}
        editingId={null}
        vpRef={vpRef}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onResize={vi.fn()}
      />
    );

    // Re-render avec un sous-ensemble (simule entrée dans un folder)
    rerender(
      <HtmlAnnotationLayer
        annotations={[mkText({ id: "t2", text: "Folder content" })]}
        selectedIds={[]}
        editingId={null}
        vpRef={vpRef}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onResize={vi.fn()}
      />
    );

    // Re-render à nouveau avec sticky qui change de type (text → sticky)
    rerender(
      <HtmlAnnotationLayer
        annotations={[mkSticky({ id: "t2", text: "Now sticky" })]}
        selectedIds={["t2"]}
        editingId={null}
        vpRef={vpRef}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onResize={vi.fn()}
      />
    );

    const errs = endCaptureReactErrors();
    const realErrs = errs.filter(e => !e.includes("not implemented") && !e.includes("Not implemented"));
    expect(realErrs, `Erreurs React détectées:\n${realErrs.join("\n---\n")}`).toEqual([]);
  });

  it("texte avec LaTeX ne crash pas", async () => {
    const HtmlAnnotationLayer = (await import("./canvas/HtmlAnnotationLayer")).default;
    startCaptureReactErrors();
    const vpRef = makeViewportRef();
    render(
      <HtmlAnnotationLayer
        annotations={[mkText({ text: "Inline $E = mc^2$ and block $$\\int_0^\\infty$$" })]}
        selectedIds={[]}
        editingId={null}
        vpRef={vpRef}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onResize={vi.fn()}
      />
    );
    const errs = endCaptureReactErrors().filter(e => !e.includes("not implemented"));
    expect(errs).toEqual([]);
  });

  it("sticky avec sourceFile (App Bridge) rendu sans crash", async () => {
    const HtmlAnnotationLayer = (await import("./canvas/HtmlAnnotationLayer")).default;
    startCaptureReactErrors();
    const vpRef = makeViewportRef();
    render(
      <HtmlAnnotationLayer
        annotations={[mkSticky({ sourceFile: "C:/path/file.blend", text: "file content preview" })]}
        selectedIds={[]}
        editingId={null}
        vpRef={vpRef}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onResize={vi.fn()}
      />
    );
    const errs = endCaptureReactErrors().filter(e => !e.includes("not implemented"));
    expect(errs).toEqual([]);
  });
});

// ─────────── ArrowOptions ───────────────────────────────────────────
describe("ArrowOptions", () => {
  it("monte avec une flèche minimale", async () => {
    const ArrowOptions = (await import("./components/ArrowOptions")).default;
    useGlucoseStore.getState().loadProject({
      version: "2.0.0", name: "test",
      boards: [{ id: "main", name: "main", images: [], annotations: [],
        panels: [], zones: [], folders: [],
        viewport: { x: 0, y: 0, scale: 1 },
        createdAt: 0, updatedAt: 0 }],
      activeBoardId: "main", presets: [], domains: [],
      createdAt: 0, updatedAt: 0,
    });
    startCaptureReactErrors();
    render(<ArrowOptions arrow={mkArrow({ text: "label", predicate: "inspire" })} />);
    const errs = endCaptureReactErrors().filter(e => !e.includes("not implemented"));
    expect(errs).toEqual([]);
  });
});

// ─────────── Toast ─────────────────────────────────────────────────
describe("Toast", () => {
  it("monte sans toast actif", async () => {
    const Toast = (await import("./components/Toast")).default;
    startCaptureReactErrors();
    render(<Toast />);
    const errs = endCaptureReactErrors().filter(e => !e.includes("not implemented"));
    expect(errs).toEqual([]);
  });
});

// ─────────── AnnotationBadges ──────────────────────────────────────
describe("AnnotationBadges", () => {
  it("rend mirror + temporal + domain badges sans crash", async () => {
    const { MirrorBadge, TemporalBadge, DomainBadges } = await import("./canvas/AnnotationBadges");
    startCaptureReactErrors();
    render(<><MirrorBadge mirrorOf="abc" /></>);
    render(<><TemporalBadge anchor={{ start: 1789, end: 1789, label: "Révolution" }} /></>);
    render(<><DomainBadges badges={[{
      domainId: "d1", weight: 0.8,
      def: { id: "d1", name: "Sci", color: "#60a5fa", icon: "🔬", createdAt: 0 },
    }]} /></>);
    const errs = endCaptureReactErrors().filter(e => !e.includes("not implemented"));
    expect(errs).toEqual([]);
  });
});

// ─────────── PRESERVATION : test #310-like scenarios ─────────────
describe("régression hooks", () => {
  it("HtmlAnnotationLayer survit à 10 re-renders avec annotations qui change de type", async () => {
    const HtmlAnnotationLayer = (await import("./canvas/HtmlAnnotationLayer")).default;
    startCaptureReactErrors();
    const vpRef = makeViewportRef();
    const baseProps = {
      selectedIds: [],
      editingId: null,
      vpRef,
      onSelect: vi.fn(),
      onEdit: vi.fn(),
      onResize: vi.fn(),
    };
    const { rerender } = render(
      <HtmlAnnotationLayer annotations={[mkText({ id: "x", text: "v1" })]} {...baseProps} />
    );
    // 10 cycles : alternance text/sticky/membrane/arrow avec même id "x"
    const cycles = [
      mkText({ id: "x", text: "v2" }),
      mkSticky({ id: "x", text: "v3" }),
      mkText({ id: "x", text: "v4 with $math$" }),
      mkSticky({ id: "x", text: "v5", operator: "AND" }),
      mkMembrane({ id: "x" }),
      mkArrow({ id: "x" }),
      mkText({ id: "x", text: "v8" }),
      mkSticky({ id: "x", text: "v9", sourceFile: "C:/test.blend" }),
      mkText({ id: "x", text: "v10" }),
      mkSticky({ id: "x", text: "v11" }),
    ];
    for (const a of cycles) {
      rerender(<HtmlAnnotationLayer annotations={[a]} {...baseProps} />);
    }
    const errs = endCaptureReactErrors().filter(e =>
      !e.includes("not implemented") && !e.includes("Not implemented"));
    expect(errs, `Erreurs détectées:\n${errs.join("\n---\n")}`).toEqual([]);
  });
});
