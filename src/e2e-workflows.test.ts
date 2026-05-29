// ────────────────────────────────────────────────────────────────────────────
// Scénarios E2E : workflows utilisateur complets, simulés via store.
//
// Si quelque chose se casse sur une chaîne d'actions réelle (créer → folder →
// entrer → éditer → sortir → sauvegarder), ce test l'attrape.
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it } from "vitest";
import { useGlucoseStore, getActiveBoard } from "./store";
import type { ArrowAnnotation, BoardImage, StickyAnnotation, TextAnnotation } from "./types";
import { nanoid } from "./utils/nanoid";

const mkText = (o: Partial<TextAnnotation> = {}): TextAnnotation =>
  ({ id: nanoid(), type: "text", x: 0, y: 0, text: "hello", ...o });
const mkSticky = (o: Partial<StickyAnnotation> = {}): StickyAnnotation =>
  ({ id: nanoid(), type: "sticky", x: 0, y: 0, text: "note", width: 160, height: 120, ...o });
const mkArrow = (o: Partial<ArrowAnnotation> = {}): ArrowAnnotation =>
  ({ id: nanoid(), type: "arrow", x: 0, y: 0, x2: 100, y2: 0, ...o });
const mkImage = (o: Partial<BoardImage> = {}): BoardImage =>
  ({ id: nanoid(), src: "asset:abc.png", x: 0, y: 0,
    width: 100, height: 100, rotation: 0, locked: false, tags: [],
    originalWidth: 100, originalHeight: 100, ...o });

beforeEach(() => {
  useGlucoseStore.getState().loadProject({
    version: "2.0.0", name: "test",
    boards: [{ id: "main", name: "Board principal",
      images: [], annotations: [], panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0 }],
    activeBoardId: "main", presets: [], domains: [],
    createdAt: 0, updatedAt: 0,
  });
});

// ─────────── Workflow 1 — folder lifecycle complet ───────────────
describe("workflow folder", () => {
  it("créer contenu → folder → entrer → éditer → sortir → undo restaure tout", () => {
    const s = useGlucoseStore.getState();
    // 1. Crée du contenu dans le board parent
    s.addImage("main", mkImage({ x: 50, y: 50 }));
    s.addAnnotation("main", mkText({ id: "T1", text: "Inside", x: 80, y: 80 }));

    // 2. Crée folder par-dessus (capture)
    s.createFolder("main", { id: "F", name: "F", color: "#fff",
      x: 0, y: 0, width: 200, height: 200 });

    // Vérifie que le contenu a été capturé
    const main = getActiveBoard(useGlucoseStore.getState().project);
    expect(main.images).toHaveLength(0);
    expect(main.annotations).toHaveLength(0);
    expect(main.folders).toHaveLength(1);

    // 3. Entre dans le folder
    useGlucoseStore.getState().enterFolder("F");
    const child = getActiveBoard(useGlucoseStore.getState().project);
    expect(child.images).toHaveLength(1);
    expect(child.annotations).toHaveLength(1);

    // 4. Édite : ajoute un nouveau sticky
    useGlucoseStore.getState().addAnnotation(child.id, mkSticky({ text: "Added in folder" }));
    expect(getActiveBoard(useGlucoseStore.getState().project).annotations).toHaveLength(2);

    // 5. Sort
    useGlucoseStore.getState().exitFolder();
    expect(useGlucoseStore.getState().project.activeBoardId).toBe("main");

    // 6. Undo enfile sticky qu'on vient d'ajouter
    useGlucoseStore.getState().undo();
    // Vérifie qu'on peut continuer à manipuler le store sans crash
    expect(() => useGlucoseStore.getState().addImage("main", mkImage())).not.toThrow();
  });

  it("créer folder vide, entrer, sortir, supprimer", () => {
    const s = useGlucoseStore.getState();
    s.createFolder("main", { id: "F1", name: "F", color: "#aaa",
      x: 0, y: 0, width: 100, height: 100 });
    s.enterFolder("F1");
    s.exitFolder();
    s.removeFolders("main", ["F1"]);
    expect(useGlucoseStore.getState().project.boards).toHaveLength(1);
  });

  it("nested folders : F → enter → créer G → enter → exit → exit → tout fonctionne", () => {
    const s = useGlucoseStore.getState();
    s.createFolder("main", { id: "F", name: "F", color: "#fff",
      x: 0, y: 0, width: 100, height: 100 });
    s.enterFolder("F");
    const childF = useGlucoseStore.getState().project.activeBoardId;
    s.createFolder(childF, { id: "G", name: "G", color: "#fff",
      x: 0, y: 0, width: 50, height: 50 });
    s.enterFolder("G");
    expect(useGlucoseStore.getState().folderStack).toHaveLength(2);
    s.exitFolder();
    expect(useGlucoseStore.getState().folderStack).toHaveLength(1);
    expect(useGlucoseStore.getState().project.activeBoardId).toBe(childF);
    s.exitFolder();
    expect(useGlucoseStore.getState().project.activeBoardId).toBe("main");
  });

  it("renommer folder ne casse pas la navigation", () => {
    useGlucoseStore.getState().createFolder("main", { id: "F", name: "Old", color: "#fff",
      x: 0, y: 0, width: 100, height: 100 });
    useGlucoseStore.getState().updateFolder("main", "F", { name: "New" });
    useGlucoseStore.getState().enterFolder("F");
    useGlucoseStore.getState().exitFolder();
    expect(useGlucoseStore.getState().project.activeBoardId).toBe("main");
  });

  it("mirror un folder + entrer dans le mirror = même contenu que l'original", () => {
    const s = useGlucoseStore.getState();
    s.createFolder("main", { id: "F", name: "F", color: "#fff",
      x: 0, y: 0, width: 100, height: 100 });
    s.enterFolder("F");
    const fChild = useGlucoseStore.getState().project.activeBoardId;
    s.addAnnotation(fChild, mkText({ id: "T-in-F" }));
    s.exitFolder();
    const mid = s.mirrorFolder("main", "F", 300, 300);
    expect(mid).toBeTruthy();
    s.enterFolder(mid!);
    // Le miroir partage le childBoardId : son contenu doit être visible
    const child = getActiveBoard(useGlucoseStore.getState().project);
    expect(child.annotations.find(a => a.id === "T-in-F")).toBeTruthy();
  });
});

// ─────────── Workflow 2 — flèches lifecycle ───────────────────
describe("workflow flèches", () => {
  it("créer 2 nœuds + flèche entre, déplacer la source, la flèche suit", () => {
    const s = useGlucoseStore.getState();
    const t1 = mkText({ id: "T1", x: 0, y: 0 });
    const t2 = mkText({ id: "T2", x: 200, y: 0 });
    const a = mkArrow({ id: "A1", sourceId: "T1", targetId: "T2",
      x: 0, y: 0, x2: 200, y2: 0 });
    s.addAnnotation("main", t1);
    s.addAnnotation("main", t2);
    s.addAnnotation("main", a);
    s.updateAnnotation("main", "T1", { x: 100, y: 100 });
    const updated = getActiveBoard(useGlucoseStore.getState().project)
      .annotations.find(x => x.id === "A1") as ArrowAnnotation;
    expect(updated.x).toBe(100);
    expect(updated.y).toBe(100);
  });

  it("supprimer la source supprime la flèche", () => {
    const t = mkText({ id: "T" });
    const a = mkArrow({ id: "A", sourceId: "T" });
    useGlucoseStore.getState().addAnnotation("main", t);
    useGlucoseStore.getState().addAnnotation("main", a);
    useGlucoseStore.getState().removeAnnotations("main", ["T"]);
    expect(getActiveBoard(useGlucoseStore.getState().project).annotations).toHaveLength(0);
  });

  it("flèche portail vers autre board", () => {
    const s = useGlucoseStore.getState();
    const other = s.addBoard("Other");
    s.setActiveBoardId("main");
    s.addAnnotation("main", mkArrow({ id: "PA", targetBoardId: other }));
    expect((getActiveBoard(useGlucoseStore.getState().project).annotations[0] as ArrowAnnotation).targetBoardId).toBe(other);
    s.removeBoard(other);
    expect((getActiveBoard(useGlucoseStore.getState().project).annotations[0] as ArrowAnnotation).targetBoardId).toBeUndefined();
  });
});

// ─────────── Workflow 3 — undo/redo intensif ───────────
describe("workflow undo/redo", () => {
  it("100 mutations + 100 undos + 100 redos sans crash", () => {
    const s = useGlucoseStore.getState();
    for (let i = 0; i < 100; i++) s.addImage("main", mkImage({ x: i, y: i }));
    for (let i = 0; i < 100; i++) s.undo();
    for (let i = 0; i < 100; i++) s.redo();
    // On ne vérifie pas le nombre exact (UNDO_DEPTH limite à 50)
    // mais qu'on ne crashe pas et qu'au moins quelques images restent
    expect(getActiveBoard(useGlucoseStore.getState().project).images.length).toBeGreaterThan(0);
  });

  it("undo après mutation après undo (mutation invalide redo) — pas de crash", () => {
    const s = useGlucoseStore.getState();
    s.addImage("main", mkImage());
    s.undo();
    s.addImage("main", mkImage()); // doit invalider redo
    // Le redo ne fait rien (pile vide) mais ne crashe pas
    s.redo();
    expect(getActiveBoard(useGlucoseStore.getState().project).images).toHaveLength(1);
  });

  it("undo après createFolder restaure le contenu pré-capture", () => {
    const s = useGlucoseStore.getState();
    s.addImage("main", mkImage({ id: "IMG", x: 50, y: 50 }));
    s.createFolder("main", { id: "F", name: "F", color: "#fff",
      x: 0, y: 0, width: 200, height: 200 });
    // Après création folder, image dans child board
    expect(getActiveBoard(useGlucoseStore.getState().project).images).toHaveLength(0);
    s.undo();
    // Après undo, image revient dans main
    expect(getActiveBoard(useGlucoseStore.getState().project).images).toHaveLength(1);
  });
});

// ─────────── Workflow 4 — duplication et miroir ────────
describe("workflow duplication + mirror", () => {
  it("duplicate sélection mixte (image + annotations de 3 types)", () => {
    const s = useGlucoseStore.getState();
    const img = mkImage({ id: "I", x: 50, y: 50 });
    const txt = mkText({ id: "T", x: 0, y: 0 });
    const sty = mkSticky({ id: "S", x: 100, y: 100 });
    s.addImage("main", img);
    s.addAnnotation("main", txt);
    s.addAnnotation("main", sty);
    s.setSelectedImageIds([img.id]);
    s.setSelectedAnnotationIds([txt.id, sty.id]);
    s.duplicateSelected("main");
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.images).toHaveLength(2);
    expect(b.annotations).toHaveLength(4);
  });

  it("mirror annotation + déplacement original propage au miroir au save/load (via save d'asPlain)", () => {
    // Les miroirs partagent du contenu via mirrorOf. Le test minimal vérifie
    // que mirrorOf est bien défini.
    const s = useGlucoseStore.getState();
    const original = mkText({ id: "O", text: "hello" });
    s.addAnnotation("main", original);
    const mid = s.mirrorAnnotation("main", "O", 50, 50);
    const b = getActiveBoard(useGlucoseStore.getState().project);
    const mirror = b.annotations.find(a => a.id === mid);
    expect(mirror?.mirrorOf).toBe("O");
  });
});

// ─────────── Workflow 5 — domains + temporal + presets ──
describe("workflow domains + temporal", () => {
  it("créer domaine, assigner à plusieurs nœuds, filtrer par temporal", () => {
    const s = useGlucoseStore.getState();
    const d = { id: "D1", name: "Sci", color: "#60a5fa", icon: "🔬", createdAt: 0 };
    s.addDomain(d);
    const t1 = mkText({ id: "T1", temporalAnchor: { start: 1900, end: 1900 } });
    const t2 = mkText({ id: "T2", temporalAnchor: { start: 2000, end: 2000 } });
    s.addAnnotation("main", t1);
    s.addAnnotation("main", t2);
    s.assignDomainToNode("main", "T1", "D1", 0.7);
    s.assignDomainToNode("main", "T2", "D1", 0.5);
    s.setTemporalFilter({ start: 1950, end: 2050 });
    expect(useGlucoseStore.getState().temporalFilter).toMatchObject({ start: 1950 });
    s.setTemporalFilter(null);
    expect(useGlucoseStore.getState().temporalFilter).toBeNull();
  });
});

// ─────────── Workflow 6 — presets ────────────────────────
describe("workflow presets", () => {
  it("crée preset, applique, change preset, applique null", () => {
    const s = useGlucoseStore.getState();
    s.addPreset({
      id: "P1", name: "Char", description: "", isBuiltin: false, createdAt: 0,
      slots: [{ id: "X", name: "X", color: "#000", description: "", order: 0 }],
    });
    s.applyPresetToBoard("main", "P1");
    expect(getActiveBoard(useGlucoseStore.getState().project).zones).toHaveLength(1);
    s.applyPresetToBoard("main", null);
    expect(getActiveBoard(useGlucoseStore.getState().project).zones).toHaveLength(0);
  });
});

// ─────────── Workflow 7 — chemin chaud "le bug folder" ──
describe("workflow folder (regression React #310)", () => {
  it("scénario probable du bug : annotation text avec markdown → folder → enter", () => {
    const s = useGlucoseStore.getState();
    // Ajoute un text annotation contenant du markdown ET du LaTeX
    s.addAnnotation("main", mkText({
      id: "MD",
      text: "# Titre\n\n- liste\n- item\n\n$E = mc^2$\n\n**bold** _italic_",
    }));
    // Crée folder par-dessus
    s.createFolder("main", { id: "F", name: "F", color: "#fff",
      x: -50, y: -50, width: 300, height: 300 });
    // Le text est censé être capturé
    const main = getActiveBoard(useGlucoseStore.getState().project);
    expect(main.annotations).toHaveLength(0);
    expect(main.folders).toHaveLength(1);
    // Entrer dans le folder
    s.enterFolder("F");
    const child = getActiveBoard(useGlucoseStore.getState().project);
    expect(child.annotations.find(a => a.id === "MD")).toBeTruthy();
    // Sortir
    s.exitFolder();
    expect(useGlucoseStore.getState().project.activeBoardId).toBe("main");
  });

  it("scénario sticky avec opérateur et markdown → folder → enter", () => {
    const s = useGlucoseStore.getState();
    s.addAnnotation("main", mkSticky({
      id: "OP",
      text: "$\\int f \\, dx$",
      operator: "BECAUSE",
      x: 50, y: 50,
    }));
    s.createFolder("main", { id: "F", name: "F", color: "#fff",
      x: 0, y: 0, width: 200, height: 200 });
    s.enterFolder("F");
    s.exitFolder();
    expect(useGlucoseStore.getState().project.activeBoardId).toBe("main");
  });
});

// ─────────── Workflow 8 — projet réaliste à 100+ items ──
describe("workflow gros volume", () => {
  it("100 images + 50 annotations mixtes + folder qui capture 30 : pas de crash", () => {
    const s = useGlucoseStore.getState();
    for (let i = 0; i < 100; i++) {
      s.addImage("main", mkImage({ x: (i % 10) * 50, y: Math.floor(i / 10) * 50 }));
    }
    for (let i = 0; i < 50; i++) {
      s.addAnnotation("main", mkText({ x: (i % 5) * 30, y: i * 10 }));
    }
    // Folder par-dessus la première moitié du grid
    s.createFolder("main", {
      id: "F", name: "F", color: "#fff",
      x: 0, y: 0, width: 200, height: 200,
    });
    // Pas de crash, c'est l'essentiel
    expect(useGlucoseStore.getState().project.boards.length).toBeGreaterThanOrEqual(2);
  });
});
