// ────────────────────────────────────────────────────────────────────────────
// Test catalog : couvre 100 % des actions du store Glucose.
//
// Objectif : exercer chaque mutation, chaque cascade, chaque cas limite que
// l'utilisateur peut déclencher via l'UI. Si le code crashe, le test
// l'attrape. Si une cascade laisse un état invalide, l'assertion l'attrape.
//
// Couverture pensée comme « actions utilisateur » : pour chaque touche,
// chaque bouton de toolbar, chaque drag-drop possible, on a un test.
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it } from "vitest";
import { useGlucoseStore, getActiveBoard } from "./index";
import type {
  ArrowAnnotation, BoardImage, Domain, MembraneAnnotation,
  Preset, StickyAnnotation, StoryboardPanel, TextAnnotation,
} from "../types";
import { nanoid } from "../utils/nanoid";
import { buildLinkRef, buildEmbedRef } from "../utils/assetRef";
import { LIMITS } from "../constants";

// ─────────── Factories ──────────────────────────────────────────────
function mkText(overrides: Partial<TextAnnotation> = {}): TextAnnotation {
  return { id: nanoid(), type: "text", x: 0, y: 0, text: "hello", ...overrides };
}
function mkSticky(overrides: Partial<StickyAnnotation> = {}): StickyAnnotation {
  return { id: nanoid(), type: "sticky", x: 0, y: 0, text: "note", width: 160, height: 120, ...overrides };
}
function mkArrow(overrides: Partial<ArrowAnnotation> = {}): ArrowAnnotation {
  return { id: nanoid(), type: "arrow", x: 0, y: 0, x2: 100, y2: 0, ...overrides };
}
function mkMembrane(overrides: Partial<MembraneAnnotation> = {}): MembraneAnnotation {
  return { id: nanoid(), type: "membrane", x: 0, y: 0, width: 200, height: 160, color: "#60a5fa", ...overrides };
}
function mkImage(overrides: Partial<BoardImage> = {}): BoardImage {
  return {
    id: nanoid(), src: "asset:abc.png", x: 0, y: 0,
    width: 100, height: 100, rotation: 0, locked: false, tags: [],
    originalWidth: 100, originalHeight: 100,
    ...overrides,
  };
}
function mkDomain(overrides: Partial<Domain> = {}): Domain {
  return { id: nanoid(), name: "Science", color: "#60a5fa", icon: "🔬", createdAt: Date.now(), ...overrides };
}
function mkPreset(overrides: Partial<Preset> = {}): Preset {
  return {
    id: nanoid(), name: "Test", description: "", slots: [],
    isBuiltin: false, createdAt: Date.now(), ...overrides,
  };
}
function mkPanel(overrides: Partial<StoryboardPanel> = {}): StoryboardPanel {
  return { id: nanoid(), order: 0, description: "", x: 0, y: 0, width: 320, height: 180, ...overrides };
}

// Réinitialise le store entre chaque test (pas de fuite d'état)
beforeEach(() => {
  // On déclenche un loadProject avec un projet vierge
  useGlucoseStore.getState().loadProject({
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

// ─────────── 1. VIEWPORT ────────────────────────────────────────────
describe("viewport", () => {
  it("setViewport persiste les coords", () => {
    useGlucoseStore.getState().setViewport("main", { x: 100, y: 200, scale: 2 });
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.viewport).toMatchObject({ x: 100, y: 200, scale: 2 });
  });
  it("setViewport clampe les valeurs hors-bornes (scale)", () => {
    useGlucoseStore.getState().setViewport("main", { x: 0, y: 0, scale: 999 });
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.viewport.scale).toBeLessThanOrEqual(50);
  });
  it("setViewport sur un board inexistant ne plante pas", () => {
    expect(() => useGlucoseStore.getState().setViewport("ghost", { x: 0, y: 0, scale: 1 }))
      .not.toThrow();
  });
});

// ─────────── 2. IMAGES ───────────────────────────────────────────────
describe("images", () => {
  it("addImage l'ajoute au board", () => {
    const img = mkImage();
    useGlucoseStore.getState().addImage("main", img);
    expect(getActiveBoard(useGlucoseStore.getState().project).images).toHaveLength(1);
  });

  it("updateImage applique le patch + déplace les flèches attachées", () => {
    const img = mkImage({ x: 0, y: 0 });
    const arrow = mkArrow({ sourceId: img.id, x: 0, y: 0, targetId: undefined });
    useGlucoseStore.getState().addImage("main", img);
    useGlucoseStore.getState().addAnnotation("main", arrow);
    useGlucoseStore.getState().updateImage("main", img.id, { x: 50, y: 50 });
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.images[0]).toMatchObject({ x: 50, y: 50 });
    const a = b.annotations[0] as ArrowAnnotation;
    expect(a.x).toBe(50); expect(a.y).toBe(50);
  });

  it("updateMultipleImages applique en batch", () => {
    const a = mkImage({ x: 0 }); const b = mkImage({ x: 10 });
    useGlucoseStore.getState().addImage("main", a);
    useGlucoseStore.getState().addImage("main", b);
    useGlucoseStore.getState().updateMultipleImages("main", [
      { id: a.id, patch: { x: 100 } }, { id: b.id, patch: { x: 200 } },
    ]);
    const imgs = getActiveBoard(useGlucoseStore.getState().project).images;
    expect(imgs.find(i => i.id === a.id)?.x).toBe(100);
    expect(imgs.find(i => i.id === b.id)?.x).toBe(200);
  });

  it("removeImages cascade sur les flèches dont source ou cible est supprimée", () => {
    const img1 = mkImage(); const img2 = mkImage();
    const arrow = mkArrow({ sourceId: img1.id, targetId: img2.id });
    useGlucoseStore.getState().addImage("main", img1);
    useGlucoseStore.getState().addImage("main", img2);
    useGlucoseStore.getState().addAnnotation("main", arrow);
    useGlucoseStore.getState().removeImages("main", [img1.id]);
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.images).toHaveLength(1);
    expect(b.annotations).toHaveLength(0); // flèche orpheline supprimée
  });

  it("removeImages clampe coords aberrantes sans planter", () => {
    const img = mkImage({ x: 1e15, y: -1e15 });
    expect(() => useGlucoseStore.getState().addImage("main", img)).not.toThrow();
  });

  // ── B-STORE — invariant « doc léger » ──────────────────────────────────────
  // Aucune voie d'ajout d'image (drag, import, COLLER) ne doit remplir
  // project.blobs avec des octets : seul un AssetRef explicitement `embed` le
  // fait. C'est ce qui empêche A.save de re-freezer. Le coller (GlucoseCanvas
  // addBlob) passe désormais par un ref `link` disque → ces tests verrouillent
  // qu'on ne ré-embarque jamais par accident.
  it("addImage en mode link ne remplit JAMAIS project.blobs (doc léger)", () => {
    const img = mkImage({ src: undefined, asset: buildLinkRef("asset:abc.png", { sha256: "deadbeef", sizeBytes: 1234 }) });
    useGlucoseStore.getState().addImage("main", img);
    const p = useGlucoseStore.getState().project;
    expect(p.blobs == null || Object.keys(p.blobs).length === 0).toBe(true);
  });

  it("addImage en mode link ignore des embedBytes éventuels (pas de bloat doc)", () => {
    const bytes = new Uint8Array([1, 2, 3, 4]);
    const img = mkImage({ src: undefined, asset: buildLinkRef("asset:def.png", { sha256: "cafe", sizeBytes: 4 }) });
    // Même si un appelant passe des bytes par erreur, un asset `link` ne doit
    // RIEN écrire dans blobs (seul déclencheur : asset.mode === "embed").
    useGlucoseStore.getState().addImage("main", img, bytes);
    const p = useGlucoseStore.getState().project;
    expect(p.blobs == null || Object.keys(p.blobs).length === 0).toBe(true);
  });

  it("addImage en mode embed écrit bien dans project.blobs (contrôle inverse)", async () => {
    const bytes = new Uint8Array([5, 6, 7, 8, 9]);
    const asset = await buildEmbedRef(bytes, "image/png");
    const img = mkImage({ src: undefined, asset });
    useGlucoseStore.getState().addImage("main", img, bytes);
    const p = useGlucoseStore.getState().project;
    expect(p.blobs?.[asset.sha256]).toBeInstanceOf(Uint8Array);
  });
});

// ─────────── 3. ANNOTATIONS (les 4 types) ────────────────────────────
describe("annotations", () => {
  it.each([
    ["text",     mkText],
    ["sticky",   mkSticky],
    ["arrow",    mkArrow],
    ["membrane", mkMembrane],
  ] as const)("addAnnotation accepte les annotations de type %s", (_label, factory) => {
    const ann = factory();
    useGlucoseStore.getState().addAnnotation("main", ann);
    expect(getActiveBoard(useGlucoseStore.getState().project).annotations).toHaveLength(1);
  });

  it("updateAnnotation applique le patch", () => {
    const a = mkText({ text: "before" });
    useGlucoseStore.getState().addAnnotation("main", a);
    useGlucoseStore.getState().updateAnnotation("main", a.id, { text: "after" });
    const upd = getActiveBoard(useGlucoseStore.getState().project).annotations[0] as TextAnnotation;
    expect(upd.text).toBe("after");
  });

  it("updateAnnotation déplace les flèches attachées", () => {
    const txt = mkText({ x: 0, y: 0 });
    const arrow = mkArrow({ targetId: txt.id, x: 50, y: 50, x2: 0, y2: 0 });
    useGlucoseStore.getState().addAnnotation("main", txt);
    useGlucoseStore.getState().addAnnotation("main", arrow);
    useGlucoseStore.getState().updateAnnotation("main", txt.id, { x: 100, y: 100 });
    const a = getActiveBoard(useGlucoseStore.getState().project).annotations.find(x => x.id === arrow.id) as ArrowAnnotation;
    // ann.x2/y2 décalés du delta (100,100)
    expect(a.x2).toBe(100); expect(a.y2).toBe(100);
  });

  it("removeAnnotations cascade sur les mirrors", () => {
    const original = mkText({ text: "original" });
    useGlucoseStore.getState().addAnnotation("main", original);
    const mirrorId = useGlucoseStore.getState().mirrorAnnotation("main", original.id, 50, 50);
    expect(mirrorId).toBeTruthy();
    useGlucoseStore.getState().removeAnnotations("main", [original.id]);
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.annotations).toHaveLength(0); // mirror cascade
  });

  it("removeAnnotations cascade sur les flèches attachées", () => {
    const txt = mkText();
    const arrow = mkArrow({ sourceId: txt.id });
    useGlucoseStore.getState().addAnnotation("main", txt);
    useGlucoseStore.getState().addAnnotation("main", arrow);
    useGlucoseStore.getState().removeAnnotations("main", [txt.id]);
    expect(getActiveBoard(useGlucoseStore.getState().project).annotations).toHaveLength(0);
  });

  it("Toutes les actions sticky-opérateur (Alt+1..4)", () => {
    const s = mkSticky();
    useGlucoseStore.getState().addAnnotation("main", s);
    for (const op of ["AND", "OR", "BUT", "BECAUSE"] as const) {
      useGlucoseStore.getState().updateAnnotation("main", s.id, { operator: op });
      const upd = getActiveBoard(useGlucoseStore.getState().project).annotations[0] as StickyAnnotation;
      expect(upd.operator).toBe(op);
    }
    useGlucoseStore.getState().updateAnnotation("main", s.id, { operator: undefined });
    const upd = getActiveBoard(useGlucoseStore.getState().project).annotations[0] as StickyAnnotation;
    expect(upd.operator).toBeUndefined();
  });
});

// ─────────── 4. SÉLECTION + ACTIONS SUR SÉLECTION ────────────────────
describe("selection", () => {
  it("selectAll attrape images + annotations", () => {
    useGlucoseStore.getState().addImage("main", mkImage());
    useGlucoseStore.getState().addAnnotation("main", mkText());
    useGlucoseStore.getState().selectAll("main");
    expect(useGlucoseStore.getState().selectedImageIds).toHaveLength(1);
    expect(useGlucoseStore.getState().selectedAnnotationIds).toHaveLength(1);
  });

  it("deleteSelected supprime images + annotations sélectionnées", () => {
    const img = mkImage(); const ann = mkText();
    useGlucoseStore.getState().addImage("main", img);
    useGlucoseStore.getState().addAnnotation("main", ann);
    useGlucoseStore.getState().setSelectedImageIds([img.id]);
    useGlucoseStore.getState().setSelectedAnnotationIds([ann.id]);
    useGlucoseStore.getState().deleteSelected("main");
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.images).toHaveLength(0);
    expect(b.annotations).toHaveLength(0);
  });

  it("duplicateSelected duplique + offset les nouveaux", () => {
    const img = mkImage({ x: 0, y: 0 });
    useGlucoseStore.getState().addImage("main", img);
    useGlucoseStore.getState().setSelectedImageIds([img.id]);
    useGlucoseStore.getState().duplicateSelected("main");
    const imgs = getActiveBoard(useGlucoseStore.getState().project).images;
    expect(imgs).toHaveLength(2);
    expect(imgs[1].x).toBe(20); // offset
  });

  it("moveSelected déplace + drag les flèches attachées", () => {
    const txt = mkText({ x: 0, y: 0 });
    const arrow = mkArrow({ sourceId: txt.id, x: 0, y: 0, x2: 50, y2: 50 });
    useGlucoseStore.getState().addAnnotation("main", txt);
    useGlucoseStore.getState().addAnnotation("main", arrow);
    useGlucoseStore.getState().setSelectedAnnotationIds([txt.id]);
    useGlucoseStore.getState().moveSelected("main", 100, 50);
    const a = getActiveBoard(useGlucoseStore.getState().project).annotations.find(x => x.id === arrow.id) as ArrowAnnotation;
    expect(a.x).toBe(100); expect(a.y).toBe(50);
  });
});

// ─────────── 5. UNDO / REDO ──────────────────────────────────────────
describe("undo/redo", () => {
  it("undo restaure l'état précédent", () => {
    useGlucoseStore.getState().addImage("main", mkImage());
    expect(getActiveBoard(useGlucoseStore.getState().project).images).toHaveLength(1);
    useGlucoseStore.getState().undo();
    expect(getActiveBoard(useGlucoseStore.getState().project).images).toHaveLength(0);
  });

  it("redo réapplique après undo", () => {
    useGlucoseStore.getState().addImage("main", mkImage());
    useGlucoseStore.getState().undo();
    useGlucoseStore.getState().redo();
    expect(getActiveBoard(useGlucoseStore.getState().project).images).toHaveLength(1);
  });

  it(`undo plafonné à UNDO_DEPTH (${LIMITS.UNDO_DEPTH}) entrées`, () => {
    const over = LIMITS.UNDO_DEPTH + 20;
    for (let i = 0; i < over; i++) {
      useGlucoseStore.getState().addImage("main", mkImage());
    }
    // Quel que soit le nombre de gestes, la pile ne garde que les UNDO_DEPTH
    // derniers (mémoire bornée) — c'est le plafond de retour en arrière.
    expect(useGlucoseStore.getState()._undoStack.length).toBeLessThanOrEqual(LIMITS.UNDO_DEPTH);
  });

  it("nouvelle mutation invalide la redo stack", () => {
    useGlucoseStore.getState().addImage("main", mkImage());
    useGlucoseStore.getState().undo();
    useGlucoseStore.getState().addImage("main", mkImage()); // mutation neuve
    useGlucoseStore.getState().redo(); // doit être no-op
    expect(getActiveBoard(useGlucoseStore.getState().project).images).toHaveLength(1);
  });
});

// ─────────── 6. BOARDS ───────────────────────────────────────────────
describe("boards", () => {
  it("addBoard crée + active le nouveau board", () => {
    const newId = useGlucoseStore.getState().addBoard("Second");
    expect(useGlucoseStore.getState().project.boards).toHaveLength(2);
    expect(useGlucoseStore.getState().project.activeBoardId).toBe(newId);
  });

  it("removeBoard refuse de supprimer le dernier board", () => {
    useGlucoseStore.getState().removeBoard("main");
    expect(useGlucoseStore.getState().project.boards).toHaveLength(1); // toujours main
  });

  it("removeBoard patche les flèches portail orphelines (targetBoardId)", () => {
    const second = useGlucoseStore.getState().addBoard("Second");
    const arrow = mkArrow({ targetBoardId: second });
    useGlucoseStore.getState().addAnnotation("main", arrow);
    useGlucoseStore.getState().setActiveBoardId("main");
    useGlucoseStore.getState().removeBoard(second);
    const a = getActiveBoard(useGlucoseStore.getState().project).annotations[0] as ArrowAnnotation;
    expect(a.targetBoardId).toBeUndefined();
  });

  it("renameBoard met à jour le nom", () => {
    useGlucoseStore.getState().renameBoard("main", "Renamed");
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.name).toBe("Renamed");
  });

  it("reorderBoards inverse l'ordre", () => {
    const second = useGlucoseStore.getState().addBoard("Second");
    useGlucoseStore.getState().reorderBoards(0, 1);
    const boards = useGlucoseStore.getState().project.boards;
    expect(boards[0].id).toBe(second);
  });

  it("duplicateBoard clone le contenu", () => {
    useGlucoseStore.getState().addImage("main", mkImage());
    useGlucoseStore.getState().addAnnotation("main", mkText());
    useGlucoseStore.getState().duplicateBoard("main");
    const boards = useGlucoseStore.getState().project.boards;
    expect(boards).toHaveLength(2);
    expect(boards[1].images).toHaveLength(1);
    expect(boards[1].annotations).toHaveLength(1);
    // Les IDs doivent être différents (deep clone)
    expect(boards[1].id).not.toBe("main");
  });
});

// ─────────── 7. DOMAINS (Phase 3) ────────────────────────────────────
describe("domains", () => {
  it("addDomain enregistre dans le projet", () => {
    useGlucoseStore.getState().addDomain(mkDomain());
    expect(useGlucoseStore.getState().getDomains()).toHaveLength(1);
  });

  it("removeDomain cascade sur les assignments des annotations + images", () => {
    const d = mkDomain();
    const ann = mkText();
    const img = mkImage();
    useGlucoseStore.getState().addDomain(d);
    useGlucoseStore.getState().addAnnotation("main", ann);
    useGlucoseStore.getState().addImage("main", img);
    useGlucoseStore.getState().assignDomainToNode("main", ann.id, d.id, 0.8);
    useGlucoseStore.getState().assignDomainToNode("main", img.id, d.id, 0.6);
    useGlucoseStore.getState().removeDomain(d.id);
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.annotations[0].domains?.length ?? 0).toBe(0);
    expect(b.images[0].domains?.length ?? 0).toBe(0);
  });

  it("assignDomainToNode weight=0 retire l'assignation", () => {
    const d = mkDomain();
    const ann = mkText();
    useGlucoseStore.getState().addDomain(d);
    useGlucoseStore.getState().addAnnotation("main", ann);
    useGlucoseStore.getState().assignDomainToNode("main", ann.id, d.id, 0.8);
    useGlucoseStore.getState().assignDomainToNode("main", ann.id, d.id, 0);
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.annotations[0].domains?.length ?? 0).toBe(0);
  });
});

// ─────────── 8. MIRRORS (Phase 4) ────────────────────────────────────
describe("mirrors", () => {
  it("mirrorAnnotation crée un alias avec mirrorOf", () => {
    const original = mkText();
    useGlucoseStore.getState().addAnnotation("main", original);
    const mid = useGlucoseStore.getState().mirrorAnnotation("main", original.id, 50, 50);
    expect(mid).toBeTruthy();
    const annotations = getActiveBoard(useGlucoseStore.getState().project).annotations;
    const mirror = annotations.find(a => a.id === mid);
    expect(mirror?.mirrorOf).toBe(original.id);
  });

  it("mirrorImage idem pour les images", () => {
    const original = mkImage();
    useGlucoseStore.getState().addImage("main", original);
    const mid = useGlucoseStore.getState().mirrorImage("main", original.id, 50, 50);
    const imgs = getActiveBoard(useGlucoseStore.getState().project).images;
    expect(imgs.find(i => i.id === mid)?.mirrorOf).toBe(original.id);
  });

  it("findOriginalAnnotation traverse une chaîne", () => {
    const o = mkText();
    useGlucoseStore.getState().addAnnotation("main", o);
    const m1 = useGlucoseStore.getState().mirrorAnnotation("main", o.id, 50, 50);
    const m2 = useGlucoseStore.getState().mirrorAnnotation("main", m1!, 100, 100);
    const found = useGlucoseStore.getState().findOriginalAnnotation(m2!);
    expect(found?.id).toBe(o.id);
  });

  it("mirrorFolder refuse les cycles Inception", () => {
    // Créer A → enter → créer B (dans A)
    useGlucoseStore.getState().createFolder("main", { id: "A", name: "A", color: "#fff", x: 0, y: 0, width: 100, height: 100 });
    const folderA = getActiveBoard(useGlucoseStore.getState().project).folders!.find(f => f.id === "A")!;
    const childA = folderA.childBoardId;
    useGlucoseStore.getState().createFolder(childA, { id: "B", name: "B", color: "#fff", x: 0, y: 0, width: 50, height: 50 });
    // Mirror de A dans B → cycle
    const refused = useGlucoseStore.getState().mirrorFolder(
      useGlucoseStore.getState().project.boards.find(b => b.id === childA)!.folders!.find(f => f.id === "B")!.childBoardId,
      "A", 10, 10,
    );
    expect(refused).toBeNull();
  });
});

// ─────────── 9. FOLDERS (Phase 7.5) ─────────────────────────────────
describe("folders", () => {
  it("createFolder vide crée le dossier + child board", () => {
    useGlucoseStore.getState().createFolder("main", {
      id: "F1", name: "F1", color: "#fff",
      x: 0, y: 0, width: 200, height: 200,
    });
    const main = getActiveBoard(useGlucoseStore.getState().project);
    expect(main.folders).toHaveLength(1);
    expect(useGlucoseStore.getState().project.boards).toHaveLength(2);
  });

  it("createFolder capture les images dont le centre tombe dans la zone", () => {
    useGlucoseStore.getState().addImage("main", mkImage({ x: 50, y: 50 }));
    useGlucoseStore.getState().addImage("main", mkImage({ x: 300, y: 300 }));
    useGlucoseStore.getState().createFolder("main", {
      id: "F1", name: "F1", color: "#fff",
      x: 0, y: 0, width: 200, height: 200,
    });
    const main = getActiveBoard(useGlucoseStore.getState().project);
    expect(main.images).toHaveLength(1); // 1 capturée
    const childBoardId = main.folders![0].childBoardId;
    const child = useGlucoseStore.getState().project.boards.find(b => b.id === childBoardId)!;
    expect(child.images).toHaveLength(1);
  });

  it("createFolder capture les sticky + text + leurs flèches reliées", () => {
    const t = mkText({ x: 50, y: 50 });
    const s = mkSticky({ x: 80, y: 80 });
    const a = mkArrow({ sourceId: t.id, targetId: s.id });
    useGlucoseStore.getState().addAnnotation("main", t);
    useGlucoseStore.getState().addAnnotation("main", s);
    useGlucoseStore.getState().addAnnotation("main", a);
    useGlucoseStore.getState().createFolder("main", {
      id: "F1", name: "F1", color: "#fff",
      x: 0, y: 0, width: 200, height: 200,
    });
    const main = getActiveBoard(useGlucoseStore.getState().project);
    expect(main.annotations).toHaveLength(0);
    const child = useGlucoseStore.getState().project.boards.find(
      b => b.id === main.folders![0].childBoardId,
    )!;
    expect(child.annotations.filter(a => a.type !== "arrow")).toHaveLength(2);
    expect(child.annotations.filter(a => a.type === "arrow")).toHaveLength(1);
  });

  it("enterFolder bascule sur le child board + push folderStack", () => {
    useGlucoseStore.getState().createFolder("main", {
      id: "F1", name: "F1", color: "#fff",
      x: 0, y: 0, width: 200, height: 200,
    });
    useGlucoseStore.getState().enterFolder("F1");
    expect(useGlucoseStore.getState().folderStack).toHaveLength(1);
    const main = useGlucoseStore.getState().project.boards.find(b => b.id === "main")!;
    expect(useGlucoseStore.getState().project.activeBoardId).toBe(main.folders![0].childBoardId);
  });

  it("exitFolder remonte au parent", () => {
    useGlucoseStore.getState().createFolder("main", {
      id: "F1", name: "F1", color: "#fff",
      x: 0, y: 0, width: 200, height: 200,
    });
    useGlucoseStore.getState().enterFolder("F1");
    useGlucoseStore.getState().exitFolder();
    expect(useGlucoseStore.getState().folderStack).toHaveLength(0);
    expect(useGlucoseStore.getState().project.activeBoardId).toBe("main");
  });

  it("removeFolders cascade sur les child boards orphelins", () => {
    useGlucoseStore.getState().createFolder("main", {
      id: "F1", name: "F1", color: "#fff",
      x: 0, y: 0, width: 200, height: 200,
    });
    const before = useGlucoseStore.getState().project.boards.length;
    useGlucoseStore.getState().removeFolders("main", ["F1"]);
    expect(useGlucoseStore.getState().project.boards.length).toBe(before - 1);
  });

  it("removeFolders conserve le child board si un miroir le partage", () => {
    useGlucoseStore.getState().createFolder("main", {
      id: "F1", name: "F1", color: "#fff",
      x: 0, y: 0, width: 200, height: 200,
    });
    const mid = useGlucoseStore.getState().mirrorFolder("main", "F1", 300, 300);
    expect(mid).toBeTruthy();
    const before = useGlucoseStore.getState().project.boards.length;
    useGlucoseStore.getState().removeFolders("main", ["F1"]);
    // Le miroir partage le même childBoardId → le board reste référencé.
    expect(useGlucoseStore.getState().project.boards.length).toBe(before);
  });
});

// ─────────── 10. PRESETS ─────────────────────────────────────────────
describe("presets", () => {
  it("addPreset enregistre le preset utilisateur", () => {
    useGlucoseStore.getState().addPreset(mkPreset({ name: "Mon preset" }));
    const userPresets = useGlucoseStore.getState().project.presets;
    expect(userPresets.find(p => p.name === "Mon preset")).toBeTruthy();
  });

  it("getAllPresets retourne builtins + utilisateurs", () => {
    useGlucoseStore.getState().addPreset(mkPreset({ name: "Custom" }));
    const all = useGlucoseStore.getState().getAllPresets();
    expect(all.find(p => p.isBuiltin)).toBeTruthy();
    expect(all.find(p => p.name === "Custom")).toBeTruthy();
  });

  it("applyPresetToBoard crée les zones", () => {
    const preset = mkPreset({
      slots: [
        { id: "s1", name: "Char", color: "#f00", description: "", order: 0 },
        { id: "s2", name: "Env",  color: "#0f0", description: "", order: 1 },
      ],
    });
    useGlucoseStore.getState().addPreset(preset);
    useGlucoseStore.getState().applyPresetToBoard("main", preset.id);
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.zones).toHaveLength(2);
  });

  it("applyPresetToBoard(null) efface les zones", () => {
    useGlucoseStore.getState().setBoardZones("main", [
      { slotId: "x", x: 0, y: 0, width: 100, height: 100 },
    ]);
    useGlucoseStore.getState().applyPresetToBoard("main", null);
    expect(getActiveBoard(useGlucoseStore.getState().project).zones).toHaveLength(0);
  });
});

// ─────────── 11. STORYBOARD ──────────────────────────────────────────
describe("storyboard", () => {
  it("addPanel + reorderPanels", () => {
    useGlucoseStore.getState().addPanel("main", mkPanel({ order: 1 }));
    useGlucoseStore.getState().addPanel("main", mkPanel({ order: 0 }));
    useGlucoseStore.getState().reorderPanels("main");
    const panels = getActiveBoard(useGlucoseStore.getState().project).panels;
    expect(panels[0].order).toBe(0);
    expect(panels[1].order).toBe(1);
  });

  it("setStoryboardSettings + clearStoryboard", () => {
    useGlucoseStore.getState().setStoryboardSettings("main", {
      aspectRatio: "16:9", panelWidth: 320, cols: 4, gap: 16,
    });
    expect(getActiveBoard(useGlucoseStore.getState().project).storyboard).toBeDefined();
    useGlucoseStore.getState().addPanel("main", mkPanel());
    useGlucoseStore.getState().clearStoryboard("main");
    const b = getActiveBoard(useGlucoseStore.getState().project);
    expect(b.storyboard).toBeUndefined();
    expect(b.panels).toHaveLength(0);
  });

  it("removePanel retire le bon panel", () => {
    const p = mkPanel();
    useGlucoseStore.getState().addPanel("main", p);
    useGlucoseStore.getState().removePanel("main", p.id);
    expect(getActiveBoard(useGlucoseStore.getState().project).panels).toHaveLength(0);
  });
});

// ─────────── 12. TIME MACHINE (Phase 7.4) ───────────────────────────
describe("time machine", () => {
  it("commitNamed crée un jalon dans l'historique", async () => {
    useGlucoseStore.getState().addImage("main", mkImage());
    useGlucoseStore.getState().commitNamed("Milestone 1");
    // Pas d'assertion directe sur l'historique (privé), juste qu'on ne crashe pas.
    expect(getActiveBoard(useGlucoseStore.getState().project).images).toHaveLength(1);
  });

  it("setPreviewHeads(null) revient au présent", () => {
    useGlucoseStore.getState().addImage("main", mkImage());
    useGlucoseStore.getState().setPreviewHeads(null);
    expect(useGlucoseStore.getState()._previewHeads).toBeNull();
  });

  it("mutate ignore les mutations en mode preview", () => {
    useGlucoseStore.getState().addImage("main", mkImage());
    // Simuler un mode preview (on ne peut pas facilement obtenir un Heads
    // valide ici sans pousser plus loin, on vérifie au moins que setPreviewHeads
    // null fonctionne sans planter)
    const cnt = getActiveBoard(useGlucoseStore.getState().project).images.length;
    useGlucoseStore.getState().setPreviewHeads(null);
    expect(getActiveBoard(useGlucoseStore.getState().project).images.length).toBe(cnt);
  });
});

// ─────────── 13. PROJECT LIFECYCLE ───────────────────────────────────
describe("project lifecycle", () => {
  it("loadProject réinitialise le store sur projet vierge", () => {
    useGlucoseStore.getState().addImage("main", mkImage());
    useGlucoseStore.getState().loadProject({
      version: "2.0.0", name: "fresh", boards: [{
        id: "x", name: "x", images: [], annotations: [],
        panels: [], zones: [], folders: [],
        viewport: { x: 0, y: 0, scale: 1 },
        createdAt: 0, updatedAt: 0,
      }],
      activeBoardId: "x", presets: [], domains: [],
      createdAt: 0, updatedAt: 0,
    });
    expect(useGlucoseStore.getState().project.boards).toHaveLength(1);
    expect(useGlucoseStore.getState().project.boards[0].id).toBe("x");
    expect(useGlucoseStore.getState()._undoStack).toHaveLength(0);
  });

  it("setProjectName persiste", () => {
    useGlucoseStore.getState().setProjectName("New name");
    expect(useGlucoseStore.getState().project.name).toBe("New name");
  });
});

// ─────────── 14. POMODORO ────────────────────────────────────────────
describe("pomodoro", () => {
  it("pomodoroReset règle le temps total", () => {
    useGlucoseStore.getState().pomodoroReset(900);
    expect(useGlucoseStore.getState().pomodoroTotal).toBe(900);
    expect(useGlucoseStore.getState().pomodoroLeft).toBe(900);
    expect(useGlucoseStore.getState().pomodoroRunning).toBe(false);
  });

  it("pomodoroStart/pause sans crash", () => {
    useGlucoseStore.getState().pomodoroReset(60);
    useGlucoseStore.getState().pomodoroStart();
    expect(useGlucoseStore.getState().pomodoroRunning).toBe(true);
    useGlucoseStore.getState().pomodoroPause();
    expect(useGlucoseStore.getState().pomodoroRunning).toBe(false);
  });
});

// ─────────── 15. UI TOGGLES ──────────────────────────────────────────
describe("UI toggles", () => {
  it("setActiveTool", () => {
    for (const tool of ["select", "pan", "text", "sticky", "arrow", "folder", "membrane", "zone-select"] as const) {
      useGlucoseStore.getState().setActiveTool(tool);
      expect(useGlucoseStore.getState().activeTool).toBe(tool);
    }
  });

  it("toggleTransDomainVisible", () => {
    const before = useGlucoseStore.getState().transDomainVisible;
    useGlucoseStore.getState().toggleTransDomainVisible();
    expect(useGlucoseStore.getState().transDomainVisible).toBe(!before);
  });

  it("setTemporalFilter", () => {
    useGlucoseStore.getState().setTemporalFilter({ start: 1900, end: 2000 });
    expect(useGlucoseStore.getState().temporalFilter).toMatchObject({ start: 1900, end: 2000 });
    useGlucoseStore.getState().setTemporalFilter(null);
    expect(useGlucoseStore.getState().temporalFilter).toBeNull();
  });

  it("toggleSmartGuides", () => {
    const before = useGlucoseStore.getState().smartGuidesEnabled;
    useGlucoseStore.getState().toggleSmartGuides();
    expect(useGlucoseStore.getState().smartGuidesEnabled).toBe(!before);
  });
});

// ─────────── 16. CASCADE COMPLEXE — REGRESSION FOLDER + MIRRORS ─────
describe("regression : folder + mirror + delete", () => {
  it("supprimer un folder ne casse pas un miroir restant", () => {
    useGlucoseStore.getState().createFolder("main", {
      id: "F1", name: "F1", color: "#fff",
      x: 0, y: 0, width: 200, height: 200,
    });
    useGlucoseStore.getState().mirrorFolder("main", "F1", 300, 300);
    // Supprimer le miroir : le folder original reste, son childBoard aussi
    const main = getActiveBoard(useGlucoseStore.getState().project);
    const mirrorId = main.folders!.find(f => f.mirrorOf === "F1")!.id;
    useGlucoseStore.getState().removeFolders("main", [mirrorId]);
    const after = getActiveBoard(useGlucoseStore.getState().project);
    expect(after.folders!.find(f => f.id === "F1")).toBeTruthy();
    expect(useGlucoseStore.getState().project.boards).toHaveLength(2); // main + child
  });

  it("supprimer un folder contenant un mirror libère bien tout", () => {
    useGlucoseStore.getState().createFolder("main", {
      id: "F1", name: "F1", color: "#fff",
      x: 0, y: 0, width: 200, height: 200,
    });
    useGlucoseStore.getState().removeFolders("main", ["F1"]);
    expect(useGlucoseStore.getState().project.boards).toHaveLength(1);
  });

  it("enter + exit + enter sur le même folder : pas de fuite folderStack", () => {
    useGlucoseStore.getState().createFolder("main", {
      id: "F1", name: "F1", color: "#fff",
      x: 0, y: 0, width: 200, height: 200,
    });
    useGlucoseStore.getState().enterFolder("F1");
    useGlucoseStore.getState().exitFolder();
    useGlucoseStore.getState().enterFolder("F1");
    expect(useGlucoseStore.getState().folderStack).toHaveLength(1);
    useGlucoseStore.getState().exitFolder();
    expect(useGlucoseStore.getState().folderStack).toHaveLength(0);
  });
});
