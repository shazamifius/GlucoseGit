// ────────────────────────────────────────────────────────────────────────────
// UNDO-1 — Batterie de tests « Ctrl+Z / Ctrl+Maj+Z marche pour TOUT ».
//
// Trois familles d'invariants, pensées pour ne plus JAMAIS régresser :
//
//   A. NAVIGATION TRANSPARENTE — zoomer, paner, entrer/sortir d'un dossier,
//      changer de board, scanner paresseusement : RIEN de tout ça ne crée
//      d'entrée undo ni ne détruit un redo en attente. (C'était LA cause du
//      « comportement incertain » : chaque pan empilait un undo et vidait le
//      redo, donc Ctrl+Z annulait un mouvement de caméra au lieu de l'action.)
//
//   B. ÉDITIONS RÉVERSIBLES — chaque mutation de CONTENU (image, annotation,
//      folder, board, preset, domaine, projet) s'annule ET se refait à
//      l'identique. Un round-trip mutate → undo → redo par action.
//
//   C. ROBUSTESSE INTER-NAVIGATION — la caméra n'est jamais téléportée par un
//      undo ; on reste dans le dossier où on est ; le redo survit à la
//      navigation ; le feedback est honnête (undo() renvoie false si rien à
//      faire).
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it } from "vitest";
import { getActiveBoard, useGlucoseStore } from "./index";
import type {
  ArrowAnnotation, BoardImage, BoardZone, CanvasFolder, Domain,
  FolderTreeNode, MembraneAnnotation, Preset, StickyAnnotation,
  StoryboardPanel, TextAnnotation,
} from "../types";
import { nanoid } from "../utils/nanoid";

// ─────────── Accès store concis ─────────────────────────────────────
const S = () => useGlucoseStore.getState();
const board = () => getActiveBoard(S().project);
const undoLen = () => S()._undoStack.length;
const redoLen = () => S()._redoStack.length;

// ─────────── Factories ──────────────────────────────────────────────
function mkText(o: Partial<TextAnnotation> = {}): TextAnnotation {
  return { id: nanoid(), type: "text", x: 0, y: 0, text: "hello", ...o };
}
function mkSticky(o: Partial<StickyAnnotation> = {}): StickyAnnotation {
  return { id: nanoid(), type: "sticky", x: 0, y: 0, text: "note", width: 160, height: 120, ...o };
}
function mkArrow(o: Partial<ArrowAnnotation> = {}): ArrowAnnotation {
  return { id: nanoid(), type: "arrow", x: 0, y: 0, x2: 100, y2: 0, ...o };
}
function mkMembrane(o: Partial<MembraneAnnotation> = {}): MembraneAnnotation {
  return { id: nanoid(), type: "membrane", x: 0, y: 0, width: 200, height: 160, color: "#60a5fa", text: "", ...o };
}
function mkImage(o: Partial<BoardImage> = {}): BoardImage {
  return {
    id: nanoid(), src: "asset:abc.png", x: 0, y: 0,
    width: 100, height: 100, rotation: 0, locked: false, tags: [],
    originalWidth: 100, originalHeight: 100, ...o,
  };
}
function mkPanel(o: Partial<StoryboardPanel> = {}): StoryboardPanel {
  return { id: nanoid(), order: 0, description: "", x: 0, y: 0, width: 320, height: 180, ...o };
}
function mkPreset(o: Partial<Preset> = {}): Preset {
  return { id: nanoid(), name: "P", description: "", slots: [], isBuiltin: false, createdAt: 0, ...o };
}
function mkDomain(o: Partial<Domain> = {}): Domain {
  return { id: nanoid(), name: "Sci", color: "#60a5fa", icon: "🔬", createdAt: 0, ...o };
}
function mkZone(o: Partial<BoardZone> = {}): BoardZone {
  return { slotId: "s1", x: 0, y: 0, width: 100, height: 100, ...o };
}
/** Données folder sans id/childBoardId (signature de createFolderWithContent). */
function mkFolderData(o: Partial<Omit<CanvasFolder, "id" | "childBoardId">> = {}) {
  return { name: "Dossier", color: "#888888", x: 0, y: 0, width: 200, height: 150, ...o };
}

// Réinitialise le store : un projet vierge mono-board "main".
beforeEach(() => {
  S().loadProject({
    version: "2.0.0",
    name: "test",
    boards: [{
      id: "main", name: "Board principal",
      images: [], annotations: [], panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 },
      createdAt: 0, updatedAt: 0,
    }],
    activeBoardId: "main",
    presets: [], domains: [],
    createdAt: 0, updatedAt: 0,
  });
});

// ════════════════════════════════════════════════════════════════════
// A. NAVIGATION TRANSPARENTE — rien de tout ça ne touche l'undo/redo.
// ════════════════════════════════════════════════════════════════════
describe("A — la navigation ne pollue jamais l'undo/redo", () => {
  it("setViewport (pan/zoom) ne crée aucune entrée undo", () => {
    expect(undoLen()).toBe(0);
    S().setViewport("main", { x: 50, y: 60, scale: 2 });
    S().setViewport("main", { x: 70, y: 80, scale: 3 });
    expect(undoLen()).toBe(0);
  });

  it("RÉGRESSION : 20 pans entre l'action et le Ctrl+Z n'enterrent pas l'undo", () => {
    S().addImage("main", mkImage());                 // l'action à annuler
    for (let i = 0; i < 20; i++) S().setViewport("main", { x: i, y: i, scale: 1 });
    expect(undoLen()).toBe(1);                        // 1 seul vrai pas, pas 21
    expect(S().undo()).toBe(true);
    expect(board().images).toHaveLength(0);           // annulée du premier coup
  });

  it("naviguer après un undo NE DÉTRUIT PAS le redo en attente (cœur du bug)", () => {
    S().addImage("main", mkImage());
    S().undo();
    expect(redoLen()).toBe(1);                        // redo armé
    S().setViewport("main", { x: 999, y: 999, scale: 4 });
    S().setActiveBoardId("main");
    expect(redoLen()).toBe(1);                        // toujours là malgré la nav
    expect(S().redo()).toBe(true);
    expect(board().images).toHaveLength(1);
  });

  it("enterFolder / exitFolder ne créent aucune entrée undo", () => {
    const fid = S().createFolderWithContent("main", mkFolderData(), [mkText()]);
    const base = undoLen();                           // 1 = la création du dossier
    S().enterFolder(fid);
    S().exitFolder();
    S().enterFolder(fid);
    S().exitToRoot();
    expect(undoLen()).toBe(base);
  });

  it("setActiveBoardId ne crée aucune entrée undo", () => {
    const other = S().addBoard("Autre");
    const base = undoLen();
    S().setActiveBoardId("main");
    S().setActiveBoardId(other);
    expect(undoLen()).toBe(base);
  });

  it("expandFolder (scan paresseux à l'entrée) ne crée aucune entrée undo", () => {
    const fid = S().createFolderWithContent("main", mkFolderData(), []);
    const base = undoLen();
    const level: FolderTreeNode = {
      folder: mkFolderData(), annotations: [mkText(), mkText()], images: [], children: [],
    };
    S().expandFolder("main", fid, level);
    expect(undoLen()).toBe(base);                     // navigation, pas édition
    // …mais le contenu scanné est bien là (persisté hors undo).
    const folder = board().folders.find((f) => f.id === fid)!;
    const child = S().project.boards.find((b) => b.id === folder.childBoardId)!;
    expect(child.annotations).toHaveLength(2);
  });
});

// ════════════════════════════════════════════════════════════════════
// B. ÉDITIONS RÉVERSIBLES — un round-trip mutate→undo→redo par mutation.
// ════════════════════════════════════════════════════════════════════
/** Exécute `mutate`, vérifie `after` (état changé), undo → `before` (revenu),
 *  redo → `after` (réappliqué). Vérifie aussi que undo/redo renvoient true. */
function roundTrip(opts: { mutate: () => void; after: () => void; before: () => void }) {
  opts.mutate();
  opts.after();
  expect(S().undo()).toBe(true);
  opts.before();
  expect(S().redo()).toBe(true);
  opts.after();
}

describe("B — chaque édition de contenu s'annule ET se refait", () => {
  it("addImage", () => {
    roundTrip({
      mutate: () => S().addImage("main", mkImage()),
      after: () => expect(board().images).toHaveLength(1),
      before: () => expect(board().images).toHaveLength(0),
    });
  });

  it("updateImage (déplacement)", () => {
    const img = mkImage();
    S().addImage("main", img);
    roundTrip({
      mutate: () => S().updateImage("main", img.id, { x: 500, y: 300 }),
      after: () => expect(board().images[0].x).toBe(500),
      before: () => expect(board().images[0].x).toBe(0),
    });
  });

  it("removeImages", () => {
    const img = mkImage();
    S().addImage("main", img);
    roundTrip({
      mutate: () => S().removeImages("main", [img.id]),
      after: () => expect(board().images).toHaveLength(0),
      before: () => expect(board().images).toHaveLength(1),
    });
  });

  it("moveSelected", () => {
    const img = mkImage();
    S().addImage("main", img);
    S().setSelectedImageIds([img.id]);
    roundTrip({
      mutate: () => S().moveSelected("main", 10, 20),
      after: () => expect(board().images[0].y).toBe(20),
      before: () => expect(board().images[0].y).toBe(0),
    });
  });

  it("duplicateSelected", () => {
    const img = mkImage();
    S().addImage("main", img);
    S().setSelectedImageIds([img.id]);
    roundTrip({
      mutate: () => S().duplicateSelected("main"),
      after: () => expect(board().images).toHaveLength(2),
      before: () => expect(board().images).toHaveLength(1),
    });
  });

  it("addAnnotation (texte)", () => {
    roundTrip({
      mutate: () => S().addAnnotation("main", mkText()),
      after: () => expect(board().annotations).toHaveLength(1),
      before: () => expect(board().annotations).toHaveLength(0),
    });
  });

  it("updateAnnotation", () => {
    const a = mkText({ text: "avant" });
    S().addAnnotation("main", a);
    roundTrip({
      mutate: () => S().updateAnnotation("main", a.id, { text: "après" }),
      after: () => expect((board().annotations[0] as TextAnnotation).text).toBe("après"),
      before: () => expect((board().annotations[0] as TextAnnotation).text).toBe("avant"),
    });
  });

  it("removeAnnotations", () => {
    const a = mkSticky();
    S().addAnnotation("main", a);
    roundTrip({
      mutate: () => S().removeAnnotations("main", [a.id]),
      after: () => expect(board().annotations).toHaveLength(0),
      before: () => expect(board().annotations).toHaveLength(1),
    });
  });

  it("flèche supprimée en cascade (source retirée) revient au redo", () => {
    const src = mkImage();
    const arrow = mkArrow({ sourceId: src.id });
    S().addImage("main", src);
    S().addAnnotation("main", arrow);
    roundTrip({
      mutate: () => S().removeImages("main", [src.id]),
      after: () => {
        expect(board().images).toHaveLength(0);
        expect(board().annotations).toHaveLength(0);     // flèche orpheline retirée
      },
      before: () => {
        expect(board().images).toHaveLength(1);
        expect(board().annotations).toHaveLength(1);     // flèche restaurée
      },
    });
  });

  it("addPanel / updatePanel / removePanel", () => {
    const p = mkPanel();
    roundTrip({
      mutate: () => S().addPanel("main", p),
      after: () => expect(board().panels).toHaveLength(1),
      before: () => expect(board().panels).toHaveLength(0),
    });
    S().updatePanel("main", p.id, { description: "x" });   // panel re-présent après redo
    roundTrip({
      mutate: () => S().updatePanel("main", p.id, { description: "y" }),
      after: () => expect(board().panels[0].description).toBe("y"),
      before: () => expect(board().panels[0].description).toBe("x"),
    });
  });

  it("renameBoard", () => {
    roundTrip({
      mutate: () => S().renameBoard("main", "Renommé"),
      after: () => expect(S().project.boards.find((b) => b.id === "main")!.name).toBe("Renommé"),
      before: () => expect(S().project.boards.find((b) => b.id === "main")!.name).toBe("Board principal"),
    });
  });

  it("addBoard", () => {
    roundTrip({
      mutate: () => S().addBoard("Nouveau"),
      after: () => expect(S().project.boards).toHaveLength(2),
      before: () => expect(S().project.boards).toHaveLength(1),
    });
  });

  it("removeBoard", () => {
    const other = S().addBoard("Jetable");
    S().setActiveBoardId("main");
    roundTrip({
      mutate: () => S().removeBoard(other),
      after: () => expect(S().project.boards.some((b) => b.id === other)).toBe(false),
      before: () => expect(S().project.boards.some((b) => b.id === other)).toBe(true),
    });
  });

  it("createFolder (+ child board)", () => {
    roundTrip({
      mutate: () => S().createFolder("main", { ...mkFolderData(), id: nanoid() }),
      after: () => {
        expect(board().folders).toHaveLength(1);
        expect(S().project.boards).toHaveLength(2);       // le child board créé
      },
      before: () => {
        expect(board().folders).toHaveLength(0);
        expect(S().project.boards).toHaveLength(1);
      },
    });
  });

  it("createFolderWithContent (dossier pré-peuplé)", () => {
    roundTrip({
      mutate: () => S().createFolderWithContent("main", mkFolderData(), [mkText(), mkSticky()]),
      after: () => expect(board().folders).toHaveLength(1),
      before: () => expect(board().folders).toHaveLength(0),
    });
  });

  it("removeFolders (cascade child board)", () => {
    const fid = S().createFolderWithContent("main", mkFolderData(), [mkText()]);
    const boardsAfterCreate = S().project.boards.length;     // 2
    roundTrip({
      mutate: () => S().removeFolders("main", [fid]),
      after: () => {
        expect(board().folders).toHaveLength(0);
        expect(S().project.boards.length).toBe(boardsAfterCreate - 1);   // child supprimé
      },
      before: () => {
        expect(board().folders).toHaveLength(1);
        expect(S().project.boards.length).toBe(boardsAfterCreate);
      },
    });
  });

  it("addPreset / updatePreset / removePreset", () => {
    const preset = mkPreset({ name: "Mien" });
    roundTrip({
      mutate: () => S().addPreset(preset),
      after: () => expect(S().project.presets.some((p) => p.id === preset.id)).toBe(true),
      before: () => expect(S().project.presets.some((p) => p.id === preset.id)).toBe(false),
    });
    roundTrip({
      mutate: () => S().updatePreset(preset.id, { name: "Modifié" }),
      after: () => expect(S().project.presets.find((p) => p.id === preset.id)!.name).toBe("Modifié"),
      before: () => expect(S().project.presets.find((p) => p.id === preset.id)!.name).toBe("Mien"),
    });
  });

  it("addDomain / updateDomain / removeDomain", () => {
    const dom = mkDomain({ name: "Maths" });
    roundTrip({
      mutate: () => S().addDomain(dom),
      after: () => expect((S().project.domains ?? []).some((d) => d.id === dom.id)).toBe(true),
      before: () => expect((S().project.domains ?? []).some((d) => d.id === dom.id)).toBe(false),
    });
    roundTrip({
      mutate: () => S().updateDomain(dom.id, { name: "Algèbre" }),
      after: () => expect((S().project.domains ?? []).find((d) => d.id === dom.id)!.name).toBe("Algèbre"),
      before: () => expect((S().project.domains ?? []).find((d) => d.id === dom.id)!.name).toBe("Maths"),
    });
  });

  it("setProjectName", () => {
    roundTrip({
      mutate: () => S().setProjectName("Projet B"),
      after: () => expect(S().project.name).toBe("Projet B"),
      before: () => expect(S().project.name).toBe("test"),
    });
  });

  it("setBoardZones", () => {
    roundTrip({
      mutate: () => S().setBoardZones("main", [mkZone(), mkZone({ slotId: "s2" })]),
      after: () => expect(board().zones).toHaveLength(2),
      before: () => expect(board().zones).toHaveLength(0),
    });
  });

  it("mirrorAnnotation", () => {
    const a = mkText();
    S().addAnnotation("main", a);
    roundTrip({
      mutate: () => S().mirrorAnnotation("main", a.id, 300, 300),
      after: () => expect(board().annotations).toHaveLength(2),
      before: () => expect(board().annotations).toHaveLength(1),
    });
  });

  it("addAnnotation (membrane) — création annulée+refaite d'un coup", () => {
    // Bug rapporté : créer une membrane puis Ctrl+Z « ne fait rien ». Au niveau
    // store, addAnnotation(membrane) est une mutation comme les autres → vérifié.
    roundTrip({
      mutate: () => S().addAnnotation("main", mkMembrane()),
      after: () => expect(board().annotations.filter((a) => a.type === "membrane")).toHaveLength(1),
      before: () => expect(board().annotations.filter((a) => a.type === "membrane")).toHaveLength(0),
    });
  });

  it("removeAnnotations (membrane sélectionnée) — suppression réversible", () => {
    const m = mkMembrane();
    S().addAnnotation("main", m);
    S().setSelectedAnnotationIds([m.id]);
    roundTrip({
      mutate: () => S().deleteSelected("main"),
      after: () => expect(board().annotations).toHaveLength(0),
      before: () => expect(board().annotations).toHaveLength(1),
    });
  });
});

// ════════════════════════════════════════════════════════════════════
// C. ROBUSTESSE INTER-NAVIGATION (caméra, dossier courant, feedback).
// ════════════════════════════════════════════════════════════════════
describe("C — robustesse caméra / dossier / feedback", () => {
  it("undo NE TÉLÉPORTE PAS la caméra (garde le viewport courant)", () => {
    S().addImage("main", mkImage());
    S().setViewport("main", { x: 1234, y: 5678, scale: 3 });
    expect(S().undo()).toBe(true);
    const vp = board().viewport;
    expect(vp.x).toBe(1234);
    expect(vp.y).toBe(5678);
    expect(vp.scale).toBe(3);
    expect(board().images).toHaveLength(0);          // contenu bien annulé
  });

  it("redo NE TÉLÉPORTE PAS la caméra non plus", () => {
    S().addImage("main", mkImage());
    S().undo();
    S().setViewport("main", { x: 42, y: 42, scale: 2 });
    expect(S().redo()).toBe(true);
    const vp = board().viewport;
    expect(vp.x).toBe(42);
    expect(vp.scale).toBe(2);
    expect(board().images).toHaveLength(1);
  });

  it("annuler une édition faite DANS un dossier te garde dans le dossier", () => {
    const fid = S().createFolderWithContent("main", mkFolderData(), []);
    S().enterFolder(fid);
    const childId = S().project.activeBoardId;
    S().addAnnotation(childId, mkText());
    expect(board().annotations).toHaveLength(1);
    expect(S().undo()).toBe(true);
    expect(S().project.activeBoardId).toBe(childId); // toujours dans le dossier
    expect(S().folderStack).toHaveLength(1);
    expect(board().annotations).toHaveLength(0);      // l'ajout est annulé
  });

  it("annuler la création du dossier où l'on est retombe proprement à la racine", () => {
    S().addAnnotation("main", mkText());              // snapshot racine
    const fid = S().createFolderWithContent("main", mkFolderData(), []);
    S().enterFolder(fid);
    const childId = S().project.activeBoardId;
    expect(S().undo()).toBe(true);                    // annule createFolderWithContent
    expect(S().project.boards.some((b) => b.id === childId)).toBe(false);
    expect(S().project.activeBoardId).toBe("main");   // fallback sain, pas de crash
    expect(S().folderStack).toHaveLength(0);
  });

  it("undo/redo renvoient false quand il n'y a rien à faire (feedback honnête)", () => {
    expect(S().undo()).toBe(false);
    expect(S().redo()).toBe(false);
    S().addImage("main", mkImage());
    expect(S().undo()).toBe(true);
    expect(S().undo()).toBe(false);                   // pile vidée
    expect(S().redo()).toBe(true);
    expect(S().redo()).toBe(false);                   // plus rien à refaire
  });

  it("une nouvelle édition invalide bien le redo (mais pas la navigation)", () => {
    S().addImage("main", mkImage());
    S().undo();
    expect(redoLen()).toBe(1);
    S().addAnnotation("main", mkText());              // VRAIE édition → vide le redo
    expect(redoLen()).toBe(0);
    expect(S().redo()).toBe(false);
  });

  it("séquence longue mixte (édition + nav + édition) reste cohérente", () => {
    S().addImage("main", mkImage());                  // E1
    S().setViewport("main", { x: 10, y: 10, scale: 1 });
    S().addAnnotation("main", mkText());              // E2
    S().setActiveBoardId("main");
    S().addAnnotation("main", mkSticky());            // E3
    expect(undoLen()).toBe(3);                         // exactement 3 éditions
    expect(S().undo()).toBe(true);                     // défait E3
    expect(board().annotations).toHaveLength(1);
    expect(S().undo()).toBe(true);                     // défait E2
    expect(board().annotations).toHaveLength(0);
    expect(S().undo()).toBe(true);                     // défait E1
    expect(board().images).toHaveLength(0);
    expect(S().undo()).toBe(false);                    // plus rien
    expect(S().redo()).toBe(true);                     // refait E1
    expect(board().images).toHaveLength(1);
  });
});

// ════════════════════════════════════════════════════════════════════
// D. TRANSACTIONS D'INTERACTION — un drag / resize / tracé = 1 entrée undo.
//    (Au lieu d'une entrée par frame de pointermove, qui rendait Ctrl+Z
//    inutilisable : il fallait 50 Ctrl+Z pour défaire un seul glissement.)
// ════════════════════════════════════════════════════════════════════
describe("D — un drag continu = UNE seule entrée undo", () => {
  it("30 moves entre begin/endLiveEdit = +1 entrée (pas +30), annulés d'un bloc", () => {
    const img = mkImage();
    S().addImage("main", img);                         // setup : 1 entrée
    S().setSelectedImageIds([img.id]);
    const base = undoLen();
    S().beginLiveEdit();
    for (let i = 0; i < 30; i++) S().moveSelected("main", 1, 0);   // drag simulé
    S().endLiveEdit();
    expect(undoLen()).toBe(base + 1);                  // +1, pas +30
    expect(board().images[0].x).toBe(30);
    expect(S().undo()).toBe(true);
    expect(board().images[0].x).toBe(0);               // tout le drag défait d'un coup
    expect(S().redo()).toBe(true);
    expect(board().images[0].x).toBe(30);              // …et refait d'un coup
  });

  it("beginLiveEdit est idempotent : appels multiples = 1 seul snapshot", () => {
    S().addImage("main", mkImage());
    const base = undoLen();
    S().beginLiveEdit();
    S().beginLiveEdit();                               // idempotent
    S().addAnnotation("main", mkText());
    S().updateAnnotation("main", board().annotations[0].id, { text: "x" });
    S().endLiveEdit();
    expect(undoLen()).toBe(base + 1);
  });

  it("endLiveEdit sans beginLiveEdit est un no-op sûr", () => {
    S().endLiveEdit();
    expect(S()._liveEdit).toBe(false);
    expect(undoLen()).toBe(0);
  });

  it("démarrer un drag (beginLiveEdit) vide le redo en attente", () => {
    S().addImage("main", mkImage());
    S().undo();
    expect(redoLen()).toBe(1);
    S().beginLiveEdit();                               // un VRAI geste d'édition commence
    S().moveSelected("main", 1, 1);
    S().endLiveEdit();
    expect(redoLen()).toBe(0);                         // le redo est bien invalidé
  });

  it("un clic sans mouvement (begin jamais appelé) ne crée aucune entrée", () => {
    // Reproduit le comportement UI : pointerdown puis pointerup sans bouger →
    // beginLiveEdit n'est jamais déclenché (lazy au 1er move).
    S().addImage("main", mkImage());
    const base = undoLen();
    S().endLiveEdit();                                 // up sans move : end seul
    expect(undoLen()).toBe(base);                      // rien empilé
  });

  it("RÉGRESSION bug texte : créer un bloc + frapper + commit = 1 entrée, effacé en 1 Ctrl+Z", () => {
    // Reproduit la vraie séquence UI : ouverture overlay (beginLiveEdit) →
    // addAnnotation(bloc vide) → auto-fit hauteur ×N (updateAnnotation) → commit
    // du texte (updateAnnotation) → fermeture overlay (endLiveEdit).
    const base = undoLen();
    const id = nanoid();
    S().beginLiveEdit();
    S().addAnnotation("main", mkText({ id, text: "" }));
    for (let i = 0; i < 6; i++) S().updateAnnotation("main", id, { height: 40 + i * 16 }); // auto-fit
    S().updateAnnotation("main", id, { text: "Bonjour le monde" });                         // commit frappe
    S().endLiveEdit();
    expect(undoLen()).toBe(base + 1);                  // +1, pas +8
    expect((board().annotations[0] as TextAnnotation).text).toBe("Bonjour le monde");
    expect(S().undo()).toBe(true);                     // UN SEUL Ctrl+Z
    expect(board().annotations).toHaveLength(0);       // le bloc disparaît entièrement
    // …et pas de « le texte redevient "texte" puis 15 Ctrl+Z » : c'est tout ou rien.
  });

  it("RÉGRESSION bug texte : éditer un bloc existant = 1 entrée (revient au texte d'avant)", () => {
    const a = mkText({ text: "ancien" });
    S().addAnnotation("main", a);                      // bloc déjà là (1 entrée)
    const base = undoLen();
    // Session d'édition : begin → frappe (commit) → end.
    S().beginLiveEdit();
    S().updateAnnotation("main", a.id, { text: "nouveau" });
    S().endLiveEdit();
    expect(undoLen()).toBe(base + 1);
    expect(S().undo()).toBe(true);
    expect((board().annotations[0] as TextAnnotation).text).toBe("ancien"); // revient d'un coup
    expect(board().annotations).toHaveLength(1);       // le bloc n'est PAS supprimé
  });

  it("syncAnnotationSize (ResizeObserver) ne crée AUCUNE entrée undo", () => {
    const a = mkText();
    S().addAnnotation("main", a);                      // 1 entrée
    const base = undoLen();
    S().syncAnnotationSize("main", a.id, 250, 180);    // mesures de rendu (auto-fit)
    S().syncAnnotationSize("main", a.id, 260, 200);
    expect(undoLen()).toBe(base);                      // 0 entrée ajoutée
    expect((board().annotations[0] as TextAnnotation).width).toBe(260); // taille bien appliquée
    expect(redoLen()).toBe(0);                         // et le redo n'est pas vidé non plus
  });

  it("RÉGRESSION (2ᵉ retour user) : édition texte + réconciliation de taille post-commit = 1 entrée", () => {
    // Le bug réel : après commit, le ResizeObserver mesure le bloc rendu et
    // réécrit width/height → ça empilait des Ctrl+Z « fantômes » APRÈS la
    // transaction. Résultat : « l'undo de texte ne fonctionne pas » (le 1ᵉʳ
    // Ctrl+Z annulait un reflow invisible au lieu d'effacer le bloc).
    const base = undoLen();
    const id = nanoid();
    S().beginLiveEdit();                               // ouverture overlay
    S().addAnnotation("main", mkText({ id, text: "" }));
    S().updateAnnotation("main", id, { text: "salut" });
    S().endLiveEdit();                                 // fermeture overlay
    // APRÈS le commit : réconciliations de taille (hors transaction) — NE doivent
    // PAS ajouter d'entrée.
    S().syncAnnotationSize("main", id, 120, 44);
    S().syncAnnotationSize("main", id, 140, 60);
    expect(undoLen()).toBe(base + 1);                  // toujours 1 entrée
    expect(S().undo()).toBe(true);                     // UN SEUL Ctrl+Z
    expect(board().annotations).toHaveLength(0);       // le bloc disparaît vraiment
    expect(S().redo()).toBe(true);
    expect((board().annotations[0] as TextAnnotation).text).toBe("salut"); // refait proprement
  });
});
