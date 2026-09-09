import { describe, expect, it } from "vitest";
import { MEMBRANE_SPACE, type MembraneMode, type SpaceItem } from "./membraneSpace";
import { planBoardStretch } from "./membraneStretch";

const PAD = MEMBRANE_SPACE.STRETCH_PADDING;

function memb(
  id: string, x: number, y: number, w: number, h: number,
  mode: MembraneMode = "stretched", membraneId: string | null = null,
): SpaceItem {
  return { id, kind: "membrane", x, y, width: w, height: h, mode, membraneId };
}

function box(
  id: string, x: number, y: number, w: number, h: number,
  membraneId: string | null = null,
): SpaceItem {
  return { id, kind: "image", x, y, width: w, height: h, membraneId };
}

const only = (out: ReturnType<typeof planBoardStretch>, id: string) =>
  out.find((o) => o.membraneId === id);

describe("planBoardStretch — qui grandit, et de combien", () => {
  it("une membrane étirée grandit jusqu'à contenir son contenu", () => {
    const items = [memb("M", 0, 0, 200, 200), box("I", 0, 0, 300, 100, "M")];
    const out = planBoardStretch(items);
    expect(out).toHaveLength(1);
    expect(out[0]).toMatchObject({ membraneId: "M", grew: true, blocked: false, blockerIds: [] });
    expect(out[0].width).toBe(300 + PAD);
    // L'autre axe tenait déjà : il ne bouge pas.
    expect(out[0].height).toBe(200);
  });

  it("une membrane déjà à la bonne taille ne produit RIEN — donc aucune écriture", () => {
    const items = [memb("M", 0, 0, 500, 500), box("I", 0, 0, 100, 100, "M")];
    expect(planBoardStretch(items)).toEqual([]);
  });

  it("les modes classique et minimisé sont ignorés", () => {
    const items = [
      memb("C", 0, 0, 50, 50, "classic"), box("A", 0, 0, 400, 400, "C"),
      memb("N", 1000, 0, 50, 50, "minimized"), box("B", 1000, 0, 400, 400, "N"),
    ];
    expect(planBoardStretch(items)).toEqual([]);
  });

  it("ne rétrécit JAMAIS : retirer du contenu ne défait pas un agrandissement manuel", () => {
    const items = [memb("M", 0, 0, 800, 800), box("I", 0, 0, 40, 40, "M")];
    expect(planBoardStretch(items)).toEqual([]);
  });

  it("une membrane étirée vide ne bouge pas", () => {
    expect(planBoardStretch([memb("M", 0, 0, 200, 200)])).toEqual([]);
  });
});

describe("planBoardStretch — les obstacles sont les FRÈRES, et rien d'autre", () => {
  it("un élément libre sur le chemin arrête la croissance et se signale", () => {
    const items = [
      memb("M", 0, 0, 200, 200), box("I", 0, 0, 300, 100, "M"),
      box("ETR", 250, 0, 50, 50),
    ];
    const out = planBoardStretch(items);
    expect(out[0]).toMatchObject({ blocked: true, blockerIds: ["ETR"] });
    // On s'arrête AU CONTACT : ni recouvrement, ni capture.
    expect(out[0].width).toBe(250);
    expect(out[0].grew).toBe(true);
  });

  it("le contenu d'une AUTRE membrane ne bloque pas — c'est la boîte de celle-ci qui compte", () => {
    // Le point : `stretchPlan` compare des coordonnées NATURELLES. Celles de
    // `CACHE` ne sont comparables qu'à l'intérieur de `AUTRE` ; hors de son
    // repère elles ne décrivent aucune position à l'écran.
    const items = [
      memb("M", 0, 0, 200, 200), box("I", 0, 0, 300, 100, "M"),
      memb("AUTRE", 900, 900, 100, 100, "minimized"),
      box("CACHE", 250, 0, 50, 50, "AUTRE"),
    ];
    const out = planBoardStretch(items);
    expect(out[0]).toMatchObject({ membraneId: "M", blocked: false, blockerIds: [] });
    expect(out[0].width).toBe(300 + PAD);
  });

  it("le contenu de la membrane elle-même ne se bloque jamais lui-même", () => {
    const items = [
      memb("M", 0, 0, 100, 100),
      box("A", 0, 0, 400, 50, "M"), box("B", 0, 200, 50, 50, "M"),
    ];
    const out = planBoardStretch(items);
    expect(out[0].blockerIds).toEqual([]);
    expect(out[0].blocked).toBe(false);
  });

  it("la membrane parente ne bloque pas sa fille — elle la contient déjà", () => {
    const items = [
      memb("P", 0, 0, 1000, 1000, "classic"),
      memb("M", 0, 0, 200, 200, "stretched", "P"),
      box("I", 0, 0, 300, 100, "M"),
    ];
    const out = planBoardStretch(items);
    expect(only(out, "M")).toMatchObject({ blocked: false, blockerIds: [] });
  });

  it("mais un FRÈRE dans la même membrane parente bloque bien", () => {
    const items = [
      memb("P", 0, 0, 1000, 1000, "classic"),
      memb("M", 0, 0, 200, 200, "stretched", "P"),
      box("I", 0, 0, 300, 100, "M"),
      box("FRERE", 260, 0, 40, 40, null),
    ];
    // `FRERE` est à la racine, pas dans P : il n'est donc pas dans le repère de M.
    expect(only(planBoardStretch(items), "M")?.blockerIds).toEqual([]);

    const items2 = items.map((i) => (i.id === "FRERE" ? { ...i, membraneId: "P" } : i));
    expect(only(planBoardStretch(items2), "M")?.blockerIds).toEqual(["FRERE"]);
  });
});

describe("planBoardStretch — les imbriquées grandissent AVANT leur parente", () => {
  it("la parente mesure la fille APRÈS sa croissance, pas avant", () => {
    // FILLE veut passer de 100 à 300+PAD de large. PARENTE doit contenir cette
    // nouvelle boîte, pas l'ancienne — sinon il faudrait deux gestes pour que
    // l'ajustement se propage.
    const items = [
      memb("PARENTE", 0, 0, 150, 150, "stretched"),
      memb("FILLE", 0, 0, 100, 100, "stretched", "PARENTE"),
      box("I", 0, 0, 300, 100, "FILLE"),
    ];
    const out = planBoardStretch(items);
    const fille = only(out, "FILLE");
    const parente = only(out, "PARENTE");
    expect(fille?.width).toBe(300 + PAD);
    expect(parente?.width).toBe(300 + PAD + PAD);
  });

  it("l'ordre de rendu suit la profondeur, la plus profonde en premier", () => {
    const items = [
      memb("PARENTE", 0, 0, 150, 150, "stretched"),
      memb("FILLE", 0, 0, 100, 100, "stretched", "PARENTE"),
      box("I", 0, 0, 300, 100, "FILLE"),
    ];
    expect(planBoardStretch(items).map((o) => o.membraneId)).toEqual(["FILLE", "PARENTE"]);
  });
});

describe("planBoardStretch — pureté", () => {
  it("n'écrit pas dans les items qu'on lui passe", () => {
    const m = memb("M", 0, 0, 200, 200);
    const items = [m, box("I", 0, 0, 300, 100, "M")];
    const avant = JSON.stringify(items);
    planBoardStretch(items);
    expect(JSON.stringify(items)).toBe(avant);
    expect(m.width).toBe(200);
  });

  it("un cycle d'appartenance ne fait pas tourner la profondeur en rond", () => {
    // `parentMap` coupe déjà les cycles ; on vérifie qu'aucune boucle infinie
    // ne subsiste dans le tri par profondeur.
    const items = [
      memb("A", 0, 0, 100, 100, "stretched", "B"),
      memb("B", 0, 0, 100, 100, "stretched", "A"),
    ];
    expect(() => planBoardStretch(items)).not.toThrow();
  });
});
