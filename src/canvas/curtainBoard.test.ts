// ────────────────────────────────────────────────────────────────────────────
// MEMB-7 — Le rideau porte un board.
//
// C'est la décision structurante de l'étape : le contenu d'un rideau n'est pas
// une liste de notes, c'est un BOARD Glucose ordinaire — sur le modèle du
// `childBoardId` d'un dossier. Tout ce que Glucose sait faire, il le fait sur
// un board : membranes, flèches, images, alignement, undo, synchro. En donner
// un au rideau lui offre donc tous les outils sans en réimplémenter un seul.
//
// Ce qui se vérifie ici, et qui ne se voit pas à la relecture :
//   ① les notes d'avant ne sont pas perdues, elles redeviennent des blocs ;
//   ② un rideau neuf et un rideau d'avant empruntent le MÊME chemin ;
//   ③ supprimer un rideau supprime son board — sinon il fuit dans le projet ;
//   ④ tout tient dans UNE entrée d'annulation.
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it } from "vitest";
import { getActiveBoard, useGlucoseStore } from "../store";
import type { Annotation, MembraneAnnotation, MembraneCurtain } from "../types";
import { CURTAIN_BOARD, createCurtain, createNote, notesToAnnotations } from "./curtainModel";

const MEMB_ID = "M1";
const ME = { id: "u-moi", name: "Moi", color: "#38bdf8" };

function membrane(curtains: MembraneCurtain[] = []): Annotation {
  return {
    id: MEMB_ID, type: "membrane", x: 0, y: 0, width: 400, height: 400,
    color: "#60a5fa", curtains,
  } as Annotation;
}

function seed(annotations: Annotation[]) {
  useGlucoseStore.getState().loadProject({
    version: "2.0.0", name: "test",
    boards: [{
      id: "main", name: "Board", images: [], annotations: [],
      panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0,
    }],
    activeBoardId: "main", presets: [], domains: [], createdAt: 0, updatedAt: 0,
  });
  for (const a of annotations) useGlucoseStore.getState().addAnnotation("main", a);
}

const st = () => useGlucoseStore.getState();
const memb = () =>
  getActiveBoard(st().project).annotations.find((a) => a.id === MEMB_ID) as MembraneAnnotation;
const curtainOf = (id: string) => (memb().curtains ?? []).find((c) => c.id === id);
const boardOf = (id: string | undefined) => st().project.boards.find((b) => b.id === id);

// ── ① Les notes ne sont pas perdues ─────────────────────────────────────────

describe("notesToAnnotations — les notes redeviennent des blocs ordinaires", () => {
  it("chaque note devient un bloc de texte, empilé en colonne", () => {
    const out = notesToAnnotations([createNote("premier"), createNote("second")]);
    expect(out).toHaveLength(2);
    expect(out[0]).toMatchObject({ type: "text", text: "premier", x: CURTAIN_BOARD.MARGIN });
    expect(out[1].y - out[0].y).toBe(CURTAIN_BOARD.BLOCK_STEP);
  });

  it("une note vide est écartée — c'était un champ de saisie, pas un contenu", () => {
    expect(notesToAnnotations([createNote("   "), createNote("vrai")])).toHaveLength(1);
  });

  it("aucune note du tout ne produit rien, et ne casse rien", () => {
    expect(notesToAnnotations([])).toEqual([]);
  });

  it("les identifiants restent traçables jusqu'au bloc produit", () => {
    const n = createNote("garde-moi");
    expect(notesToAnnotations([n])[0].id).toContain(n.id);
  });
});

// ── ② Un seul chemin, pour le neuf comme pour l'ancien ──────────────────────

describe("ensureCurtainBoard — le rideau reçoit son board", () => {
  beforeEach(() => { seed([membrane()]); });

  function poser(c: MembraneCurtain) {
    st().updateAnnotation("main", MEMB_ID, { curtains: [c] } as Partial<Annotation>);
  }

  it("un rideau neuf obtient un board, vide et nommé à son propriétaire", () => {
    const c = createCurtain(ME);
    poser(c);
    const id = st().ensureCurtainBoard("main", MEMB_ID, c.id);
    expect(id).toBeTruthy();
    expect(curtainOf(c.id)!.boardId).toBe(id);
    const b = boardOf(id!);
    expect(b).toBeDefined();
    expect(b!.annotations).toEqual([]);
    expect(b!.name).toContain(ME.name);
  });

  it("un rideau D'AVANT emprunte le MÊME chemin, et ses notes le suivent", () => {
    // Le point : il n'y a pas de « code de migration » à faire vivre à côté du
    // code normal. C'est le même appel, exercé à chaque création.
    const c: MembraneCurtain = { ...createCurtain(ME), notes: [createNote("ma vieille note")] };
    poser(c);
    const id = st().ensureCurtainBoard("main", MEMB_ID, c.id);

    const b = boardOf(id!)!;
    expect(b.annotations).toHaveLength(1);
    expect((b.annotations[0] as { text: string }).text).toBe("ma vieille note");
    // Et elles ne sont pas comptées deux fois : le rideau ne les porte plus.
    expect(curtainOf(c.id)!.notes).toEqual([]);
  });

  it("est idempotent : rappeler ne crée pas un second board", () => {
    const c = createCurtain(ME);
    poser(c);
    const premier = st().ensureCurtainBoard("main", MEMB_ID, c.id);
    const avant = st().project.boards.length;
    const second = st().ensureCurtainBoard("main", MEMB_ID, c.id);
    expect(second).toBe(premier);
    expect(st().project.boards.length).toBe(avant);
  });

  it("un board disparu est refabriqué plutôt que de laisser un lien mort", () => {
    const c = createCurtain(ME);
    poser(c);
    const premier = st().ensureCurtainBoard("main", MEMB_ID, c.id)!;
    // Quelqu'un a supprimé le board (undo d'un pair, document réparé…).
    st().mutate("test", (d) => {
      const i = d.boards.findIndex((b) => b.id === premier);
      if (i >= 0) d.boards.splice(i, 1);
    });
    const second = st().ensureCurtainBoard("main", MEMB_ID, c.id);
    expect(second).not.toBe(premier);
    expect(boardOf(second!)).toBeDefined();
  });

  it("ne touche pas aux AUTRES rideaux de la membrane", () => {
    // Le piège Automerge : réécrire le tableau réinsère les éléments non
    // touchés, et le document refuse. `detachCurtains` recopie champ par champ.
    const a = createCurtain(ME);
    const b = { ...createCurtain({ id: "u-2", name: "Léo", color: "#34d399" }), id: "c-2" };
    st().updateAnnotation("main", MEMB_ID, { curtains: [a, b] } as Partial<Annotation>);
    expect(() => st().ensureCurtainBoard("main", MEMB_ID, a.id)).not.toThrow();
    expect(memb().curtains).toHaveLength(2);
    expect(curtainOf("c-2")!.ownerName).toBe("Léo");
    expect(curtainOf("c-2")!.boardId).toBeUndefined();
  });

  it("un rideau inconnu ne fabrique rien", () => {
    expect(st().ensureCurtainBoard("main", MEMB_ID, "fantome")).toBeNull();
    expect(st().project.boards).toHaveLength(1);
  });

  it("④ création du board et rattachement s'annulent ENSEMBLE", () => {
    const c = createCurtain(ME);
    poser(c);
    const id = st().ensureCurtainBoard("main", MEMB_ID, c.id)!;
    expect(st().project.boards).toHaveLength(2);

    expect(st().undo()).toBe(true);
    expect(boardOf(id)).toBeUndefined();
    expect(curtainOf(c.id)!.boardId).toBeUndefined();
  });
});

// ── ③ Supprimer un rideau supprime son board ────────────────────────────────

describe("removeCurtain — rien ne fuit", () => {
  beforeEach(() => { seed([membrane()]); });

  it("le rideau part, et son board avec lui", () => {
    const c = createCurtain(ME);
    st().updateAnnotation("main", MEMB_ID, { curtains: [c] } as Partial<Annotation>);
    const id = st().ensureCurtainBoard("main", MEMB_ID, c.id)!;

    st().removeCurtain("main", MEMB_ID, c.id);
    expect(memb().curtains ?? []).toHaveLength(0);
    expect(boardOf(id)).toBeUndefined();
  });

  it("les autres rideaux et leurs boards restent intacts", () => {
    const a = createCurtain(ME);
    const b = { ...createCurtain({ id: "u-2", name: "Léo", color: "#34d399" }), id: "c-2" };
    st().updateAnnotation("main", MEMB_ID, { curtains: [a, b] } as Partial<Annotation>);
    const idA = st().ensureCurtainBoard("main", MEMB_ID, a.id)!;
    const idB = st().ensureCurtainBoard("main", MEMB_ID, "c-2")!;

    st().removeCurtain("main", MEMB_ID, a.id);
    expect(boardOf(idA)).toBeUndefined();
    expect(boardOf(idB)).toBeDefined();
    expect(memb().curtains).toHaveLength(1);
  });

  it("si on regardait le board du rideau, on retombe sur celui de la membrane", () => {
    const c = createCurtain(ME);
    st().updateAnnotation("main", MEMB_ID, { curtains: [c] } as Partial<Annotation>);
    const id = st().ensureCurtainBoard("main", MEMB_ID, c.id)!;
    st().mutate("test", (d) => { d.activeBoardId = id; });

    st().removeCurtain("main", MEMB_ID, c.id);
    expect(st().project.activeBoardId).toBe("main");
  });

  it("un rideau sans board se supprime aussi, sans rien casser", () => {
    const c = createCurtain(ME);
    st().updateAnnotation("main", MEMB_ID, { curtains: [c] } as Partial<Annotation>);
    st().removeCurtain("main", MEMB_ID, c.id);
    expect(memb().curtains ?? []).toHaveLength(0);
    expect(st().project.boards).toHaveLength(1);
  });
});

// ── Le board du rideau est un board comme les autres ────────────────────────

describe("une fois créé, c'est un board ORDINAIRE", () => {
  it("on y pose une membrane, une flèche, une image — sans rien de spécial", () => {
    // C'est tout l'intérêt de la décision : aucun outil n'a été adapté.
    seed([membrane()]);
    const c = createCurtain(ME);
    st().updateAnnotation("main", MEMB_ID, { curtains: [c] } as Partial<Annotation>);
    const id = st().ensureCurtainBoard("main", MEMB_ID, c.id)!;

    st().addAnnotation(id, {
      id: "m-dedans", type: "membrane", x: 0, y: 0, width: 200, height: 200,
    } as Annotation);
    st().addAnnotation(id, {
      id: "f-dedans", type: "arrow", x: 0, y: 0, x2: 50, y2: 50,
    } as Annotation);
    st().addImage(id, {
      id: "i-dedans", x: 10, y: 10, width: 40, height: 40, rotation: 0,
      locked: false, tags: [], originalWidth: 40, originalHeight: 40,
    });

    const b = boardOf(id)!;
    expect(b.annotations.map((a) => a.type).sort()).toEqual(["arrow", "membrane"]);
    expect(b.images).toHaveLength(1);
  });
});
