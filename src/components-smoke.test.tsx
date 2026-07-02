// ────────────────────────────────────────────────────────────────────────────
// Smoke tests composants : monte chaque composant principal avec des props
// réalistes, déclenche un re-render, vérifie qu'aucune erreur React n'est
// jetée (#310 hooks count mismatch, "setState during render", etc.).
//
// Si un composant viole les règles des Hooks ou setState pendant render,
// React jette une erreur qu'on attrape ici.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render } from "@testing-library/react";
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

// ─────────── Helper : projet baseline pour tous les smoke tests ────────────
function loadBaselineProject(opts: { withFolder?: boolean; withAnnotations?: boolean } = {}) {
  const folders = opts.withFolder
    ? [{ id: "f1", name: "Dossier A", x: 0, y: 0, width: 200, height: 200,
         color: "#60a5fa", childBoardId: "child1" }]
    : [];
  const childBoards = opts.withFolder
    ? [{ id: "child1", name: "Dossier A", images: [], annotations: [],
         panels: [], zones: [], folders: [],
         viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0 }]
    : [];
  const annotations: Annotation[] = opts.withAnnotations
    ? [
        mkText({ id: "t1", text: "Premier texte" }),
        mkSticky({ id: "s1", text: "Premier sticky" }),
        mkArrow({ id: "a1", sourceId: "t1", targetId: "s1" }),
      ]
    : [];
  useGlucoseStore.getState().loadProject({
    version: "2.0.0", name: "test",
    boards: [
      { id: "root", name: "Racine", images: [], annotations,
        panels: [], zones: [], folders,
        viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0 },
      ...childBoards,
    ],
    activeBoardId: "root", presets: [], domains: [],
    createdAt: 0, updatedAt: 0,
  });
}

function expectNoReactErrors() {
  const errs = endCaptureReactErrors().filter(e =>
    !e.includes("not implemented") && !e.includes("Not implemented"));
  expect(errs, `Erreurs React détectées:\n${errs.join("\n---\n")}`).toEqual([]);
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

// ─────────── FolderBreadcrumb — transition de state critique ──────────────
// Le bug "folder cassé" venait de FolderBreadcrumb : un `useEffect` placé
// APRÈS un `if (folderStack.length === 0) return null;`. Côté React, le
// composant appelait N hooks hors-folder vs N+1 hooks dans-folder → React #310
// (« rendered more hooks than during the previous render ») au moment précis
// où on entrait dans le premier folder.
//
// Ce test reproduit la transition exacte qui déclenchait le crash : on monte
// FolderBreadcrumb avec folderStack vide, puis on bascule à length>0 via le
// store. Avant le fix, ce rerender lève React #310 ; après le fix, il passe.
describe("FolderBreadcrumb — transition folderStack 0→1", () => {
  it("survit au passage de folderStack vide à non-vide sans erreur React", async () => {
    const FolderBreadcrumb = (await import("./components/FolderBreadcrumb")).default;

    // Setup : projet avec un parent board + un folder pointant vers un child board
    useGlucoseStore.getState().loadProject({
      version: "2.0.0", name: "test",
      boards: [
        { id: "root", name: "Racine", images: [], annotations: [],
          panels: [], zones: [], folders: [
            { id: "f1", name: "Dossier A", x: 0, y: 0, width: 200, height: 200,
              color: "#60a5fa", childBoardId: "child1" },
          ],
          viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0 },
        { id: "child1", name: "Dossier A", images: [], annotations: [],
          panels: [], zones: [], folders: [],
          viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0 },
      ],
      activeBoardId: "root", presets: [], domains: [],
      createdAt: 0, updatedAt: 0,
    });

    startCaptureReactErrors();
    // Render initial : folderStack vide → composant retourne null
    const { rerender } = render(<FolderBreadcrumb />);

    // ⚡ TRANSITION CRITIQUE : on entre dans le folder
    // C'est précisément ici que React #310 explosait avant le fix
    useGlucoseStore.getState().enterFolder("f1");
    rerender(<FolderBreadcrumb />);

    // Re-sortie pour vérifier le sens inverse aussi
    useGlucoseStore.getState().exitFolder();
    rerender(<FolderBreadcrumb />);

    // Re-rentrée
    useGlucoseStore.getState().enterFolder("f1");
    rerender(<FolderBreadcrumb />);

    const errs = endCaptureReactErrors().filter(e =>
      !e.includes("not implemented") && !e.includes("Not implemented"));
    expect(errs, `Erreurs React détectées:\n${errs.join("\n---\n")}`).toEqual([]);
  });
});

// ─────────── COUVERTURE LARGE : mount + transition sur tous les composants ──
// Chaque composant en dessous doit au moins être monté une fois par un test.
// Un re-render après une transition d'état du store est inclus pour exposer
// les hook-order violations cachées (du même genre que FolderBreadcrumb).

describe("FolderViewportIndicator — transition folderStack 0→1", () => {
  it("monte sans crash quand folderStack vide puis non-vide", async () => {
    const FolderViewportIndicator = (await import("./components/FolderViewportIndicator")).default;
    loadBaselineProject({ withFolder: true });
    startCaptureReactErrors();
    const { rerender } = render(<FolderViewportIndicator />);
    useGlucoseStore.getState().enterFolder("f1");
    rerender(<FolderViewportIndicator />);
    useGlucoseStore.getState().exitFolder();
    rerender(<FolderViewportIndicator />);
    expectNoReactErrors();
  });
});

describe("BoardTabs", () => {
  it("monte + re-render après ajout/suppression de board", async () => {
    const BoardTabs = (await import("./components/BoardTabs")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    const { rerender } = render(<BoardTabs />);
    useGlucoseStore.getState().addBoard("Second board");
    rerender(<BoardTabs />);
    expectNoReactErrors();
  });
});

describe("Toolbar", () => {
  it("monte avec tous les panneaux fermés puis re-render avec ouverts", async () => {
    const Toolbar = (await import("./components/Toolbar")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    const props = {
      onTogglePreset: vi.fn(), presetPanelOpen: false,
      onToggleDomains: vi.fn(), domainsPanelOpen: false,
      onToggleOrganize: vi.fn(), organizePanelOpen: false,
      onToggleStoryboard: vi.fn(), storyboardPanelOpen: false,
      onTogglePomodoro: vi.fn(), pomodoroOpen: false,
      onToggleMultiplayer: vi.fn(), multiplayerPanelOpen: false, collabActive: false,
      onTogglePlugins: vi.fn(), pluginsPanelOpen: false,
    };
    const { rerender } = render(<Toolbar {...props} />);
    rerender(<Toolbar {...props} presetPanelOpen={true} domainsPanelOpen={true} />);
    expectNoReactErrors();
  });
});

describe("TemporalAnchorPrompt", () => {
  it("monte le modal d'ancrage temporel sans crash", async () => {
    const TemporalAnchorPrompt = (await import("./components/TemporalAnchorPrompt")).default;
    loadBaselineProject({ withAnnotations: true });
    startCaptureReactErrors();
    render(<TemporalAnchorPrompt onClose={vi.fn()} />);
    expectNoReactErrors();
  });
});

describe("TemporalRuler", () => {
  it("monte la réglette temporelle + change le filtre", async () => {
    const TemporalRuler = (await import("./components/TemporalRuler")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    const { rerender } = render(<TemporalRuler onClose={vi.fn()} />);
    useGlucoseStore.getState().setTemporalFilter({ start: 1500, end: 1800 });
    rerender(<TemporalRuler onClose={vi.fn()} />);
    expectNoReactErrors();
  });
});

describe("ArrowDescriptionPanel", () => {
  it("monte le panel description d'une flèche existante", async () => {
    const ArrowDescriptionPanel = (await import("./components/ArrowDescriptionPanel")).default;
    loadBaselineProject({ withAnnotations: true });
    startCaptureReactErrors();
    render(<ArrowDescriptionPanel arrowId="a1" midX={400} midY={300} onClose={vi.fn()} />);
    expectNoReactErrors();
  });
  it("se monte gracieusement même si l'arrowId n'existe pas (early return)", async () => {
    const ArrowDescriptionPanel = (await import("./components/ArrowDescriptionPanel")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    render(<ArrowDescriptionPanel arrowId="nope" midX={0} midY={0} onClose={vi.fn()} />);
    expectNoReactErrors();
  });
});

describe("ArrowTextEditor", () => {
  it("monte l'éditeur de texte de flèche avec arrow valide", async () => {
    const ArrowTextEditor = (await import("./components/ArrowTextEditor")).default;
    loadBaselineProject({ withAnnotations: true });
    startCaptureReactErrors();
    render(<ArrowTextEditor arrowId="a1" onClose={vi.fn()} />);
    expectNoReactErrors();
  });
});

describe("ColorPicker", () => {
  it("monte la palette de couleur sans crash", async () => {
    const ColorPicker = (await import("./components/ColorPicker")).default;
    startCaptureReactErrors();
    render(
      <ColorPicker color="#60a5fa" onChange={vi.fn()} />
    );
    expectNoReactErrors();
  });
});

describe("Minimap", () => {
  it("monte la minimap avec un projet vide puis avec contenu", async () => {
    const Minimap = (await import("./components/Minimap")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    const { rerender } = render(<Minimap />);
    loadBaselineProject({ withAnnotations: true });
    rerender(<Minimap />);
    expectNoReactErrors();
  });

  it("se met à jour visuellement quand on entre dans un folder vide (overlay 'Canvas vide' apparaît)", async () => {
    const Minimap = (await import("./components/Minimap")).default;
    // Setup : root REMPLI + child board VIDE
    // Si la subscription Zustand fonctionne, entrer dans le folder doit
    // afficher l'overlay "Canvas vide" car le child board n'a rien dedans.
    useGlucoseStore.getState().loadProject({
      version: "2.0.0", name: "test",
      boards: [
        { id: "root", name: "Racine",
          images: [],
          annotations: [
            mkText({ id: "root-t1", text: "Du contenu dans la racine" }),
            mkSticky({ id: "root-s1", text: "Sticky racine" }),
          ],
          panels: [], zones: [],
          folders: [
            { id: "f1", name: "Dossier", x: 0, y: 0, width: 200, height: 200,
              color: "#60a5fa", childBoardId: "child1" },
          ],
          viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0 },
        { id: "child1", name: "Dossier",
          images: [], annotations: [],
          panels: [], zones: [], folders: [],
          viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0 },
      ],
      activeBoardId: "root", presets: [], domains: [],
      createdAt: 0, updatedAt: 0,
    });
    startCaptureReactErrors();
    // ⚠️ PAS de rerender() manuel : on vérifie que Zustand propage réellement.
    const { queryByText } = render(<Minimap />);

    // Avant : root a 2 annotations → pas d'overlay
    expect(queryByText("Canvas vide"), "Le root contient des annotations, overlay ne doit pas apparaître").toBeNull();

    // Action store réelle : entre dans le folder. La Minimap doit voir
    // que activeBoardId = child1 (vide) et afficher l'overlay.
    act(() => {
      useGlucoseStore.getState().enterFolder("f1");
    });

    // Si Zustand propage correctement : "Canvas vide" doit apparaître.
    expect(queryByText("Canvas vide"),
      "La Minimap n'a pas suivi le changement d'activeBoardId — Zustand ne propage pas le re-render.")
      .not.toBeNull();

    expectNoReactErrors();
  });
});

describe("PomodoroTimer", () => {
  it("monte le timer Pomodoro sans crash", async () => {
    const PomodoroTimer = (await import("./components/PomodoroTimer")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    render(<PomodoroTimer />);
    expectNoReactErrors();
  });
});

describe("PomodoroOverlay", () => {
  it("monte l'overlay Pomodoro sans crash", async () => {
    const PomodoroOverlay = (await import("./components/PomodoroOverlay")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    render(<PomodoroOverlay onOpen={vi.fn()} />);
    expectNoReactErrors();
  });
});

describe("PresetPanel", () => {
  it("monte le panneau presets sans crash", async () => {
    const PresetPanel = (await import("./components/PresetPanel")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    render(<PresetPanel onClose={vi.fn()} />);
    expectNoReactErrors();
  });
});

describe("DomainsPanel", () => {
  it("monte le panneau domaines sans crash", async () => {
    const DomainsPanel = (await import("./components/DomainsPanel")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    render(<DomainsPanel onClose={vi.fn()} />);
    expectNoReactErrors();
  });
});

describe("SearchPanel", () => {
  it("monte le panneau de recherche sans crash", async () => {
    const SearchPanel = (await import("./components/SearchPanel")).default;
    loadBaselineProject({ withAnnotations: true });
    startCaptureReactErrors();
    render(<SearchPanel onClose={vi.fn()} />);
    expectNoReactErrors();
  });
});

describe("OrganizePanel", () => {
  it("monte le panneau d'organisation sans crash", async () => {
    const OrganizePanel = (await import("./components/OrganizePanel")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    render(<OrganizePanel docked />);
    expectNoReactErrors();
  });

  it("convertit correctement RGB en HSL", async () => {
    const { rgbToHsl } = await import("./components/OrganizePanel");
    
    // Rouge pur
    const [h1, s1, l1] = rgbToHsl(255, 0, 0);
    expect(h1).toBeCloseTo(0, 1);
    expect(s1).toBeCloseTo(1, 2);
    expect(l1).toBeCloseTo(0.5, 2);

    // Vert pur
    const [h2, s2, l2] = rgbToHsl(0, 255, 0);
    expect(h2).toBeCloseTo(120, 1);
    expect(s2).toBeCloseTo(1, 2);
    expect(l2).toBeCloseTo(0.5, 2);

    // Bleu pur
    const [h3, s3, l3] = rgbToHsl(0, 0, 255);
    expect(h3).toBeCloseTo(240, 1);
    expect(s3).toBeCloseTo(1, 2);
    expect(l3).toBeCloseTo(0.5, 2);

    // Blanc
    const [, s4, l4] = rgbToHsl(255, 255, 255);
    expect(s4).toBeCloseTo(0, 2);
    expect(l4).toBeCloseTo(1, 2);

    // Noir
    const [, s5, l5] = rgbToHsl(0, 0, 0);
    expect(s5).toBeCloseTo(0, 2);
    expect(l5).toBeCloseTo(0, 2);
  });
});

describe("StoryboardControls", () => {
  it("monte les contrôles storyboard sans crash", async () => {
    const StoryboardControls = (await import("./components/StoryboardControls")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    render(<StoryboardControls docked />);
    expectNoReactErrors();
  });
});

describe("TimelinePanel", () => {
  it("monte le panneau timeline sans crash", async () => {
    const TimelinePanel = (await import("./components/TimelinePanel")).default;
    loadBaselineProject({ withAnnotations: true });
    startCaptureReactErrors();
    render(<TimelinePanel onClose={vi.fn()} />);
    expectNoReactErrors();
  });
});

describe("PanelDock", () => {
  it("monte le dock avec différents tabs ouverts", async () => {
    const PanelDock = (await import("./components/PanelDock")).default;
    loadBaselineProject();
    startCaptureReactErrors();
    const { rerender } = render(
      <PanelDock openTabs={[]} dismissingTabs={[]} onDismiss={vi.fn()} />
    );
    rerender(
      <PanelDock openTabs={["organize"]} dismissingTabs={[]} onDismiss={vi.fn()} />
    );
    rerender(
      <PanelDock openTabs={["organize", "storyboard", "pomodoro"]} dismissingTabs={[]} onDismiss={vi.fn()} />
    );
    rerender(
      <PanelDock openTabs={["pomodoro"]} dismissingTabs={["organize"]} onDismiss={vi.fn()} />
    );
    expectNoReactErrors();
  });
});

describe("AppBridgeIcon", () => {
  it("monte l'icône App Bridge", async () => {
    const AppBridgeIcon = (await import("./components/AppBridgeIcon")).default;
    startCaptureReactErrors();
    render(<AppBridgeIcon filePath="C:/test.blend" size={18} />);
    expectNoReactErrors();
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
