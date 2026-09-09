import { describe, expect, it } from "vitest";
import { itemsOfBoard, resolveItems, type MembraneMode, type SpaceItem } from "./membraneSpace";
import {
  ease, geomOrNatural, membershipSignature, naturalGeom, planTween, scaleOfMembrane,
  tweenGeom, tweenedIds, MEMBRANE_TWEEN,
} from "./membraneTween";
import type { Annotation, BoardImage } from "../types";

function memb(
  id: string, x: number, y: number, w: number, h: number,
  mode: MembraneMode = "minimized", membraneId: string | null = null,
): SpaceItem {
  return { id, kind: "membrane", x, y, width: w, height: h, mode, membraneId };
}

function box(
  id: string, x: number, y: number, w: number, h: number,
  membraneId: string | null = null,
): SpaceItem {
  return { id, kind: "image", x, y, width: w, height: h, membraneId };
}

/** Membrane 200×200, contenu jusqu'à 800 → échelle 0,25. */
const M = memb("M", 0, 0, 200, 200);
const DEDANS = box("I", 0, 0, 800, 800, "M");
const LIBRE = box("L", 0, 0, 800, 800);

// ── Le déclencheur ──────────────────────────────────────────────────────────

describe("le déclencheur est un fait DISCRET, pas une différence de géométrie", () => {
  it("l'appartenance change → l'empreinte change", () => {
    expect(membershipSignature([M, LIBRE])).not.toBe(membershipSignature([M, DEDANS]));
  });

  it("le mode change → l'empreinte change", () => {
    const etiree = { ...M, mode: "stretched" as MembraneMode };
    expect(membershipSignature([M, DEDANS])).not.toBe(membershipSignature([etiree, DEDANS]));
  });

  it("INVARIANT : redimensionner la membrane ne déclenche RIEN", () => {
    // Le point de tout le module. L'échelle change à chaque image quand on tire
    // la poignée ; animer là-dessus ferait traîner le contenu derrière elle.
    const tiree = { ...M, width: 400, height: 400 };
    expect(membershipSignature([tiree, DEDANS])).toBe(membershipSignature([M, DEDANS]));
  });

  it("INVARIANT : déplacer un élément ne déclenche rien non plus", () => {
    const bouge = { ...DEDANS, x: 300, y: 120 };
    expect(membershipSignature([M, bouge])).toBe(membershipSignature([M, DEDANS]));
  });

  it("créer un élément LIBRE ailleurs sur le board ne déclenche rien", () => {
    const autre = box("AUTRE", 9000, 9000, 50, 50);
    expect(membershipSignature([M, DEDANS, autre])).toBe(membershipSignature([M, DEDANS]));
  });

  it("l'empreinte ne dépend pas de l'ordre des éléments", () => {
    expect(membershipSignature([M, DEDANS])).toBe(membershipSignature([DEDANS, M]));
  });
});

// ── Ce qu'on anime ──────────────────────────────────────────────────────────

describe("on n'anime QUE ce qui change d'échelle", () => {
  const avant = naturalGeom([M, LIBRE]);
  const apres = resolveItems([M, DEDANS]);

  it("l'élément qui entre dans la membrane est animé", () => {
    expect(tweenedIds(avant, apres).has("I")).toBe(false); // il s'appelle L avant
    expect(tweenedIds(naturalGeom([M, DEDANS]), apres).has("I")).toBe(true);
  });

  it("la membrane elle-même ne l'est pas : elle n'est pas réduite, elle est le cadre", () => {
    expect(tweenedIds(naturalGeom([M, DEDANS]), apres).has("M")).toBe(false);
  });

  it("un élément absent du départ n'est pas animé — on ne vient pas de nulle part", () => {
    const nouveau = box("NEUF", 0, 0, 100, 100, "M");
    const to = resolveItems([M, DEDANS, nouveau]);
    expect(tweenedIds(naturalGeom([M, DEDANS]), to).has("NEUF")).toBe(false);
  });

  it("les VOISINS déjà dans la membrane sont animés aussi", () => {
    // Déposer un élément agrandit l'étendue du contenu, donc réduit l'échelle
    // de tout le monde. Ne pas les animer ferait sauter la moitié de la scène.
    const voisin = box("V", 0, 0, 100, 100, "M");
    const avantDepot = resolveItems([M, voisin]);
    const apresDepot = resolveItems([M, voisin, DEDANS]);
    expect(scaleOfMembrane(M, [voisin])).toBe(1);
    expect(scaleOfMembrane(M, [voisin, DEDANS])).toBe(0.25);
    expect(tweenedIds(avantDepot, apresDepot).has("V")).toBe(true);
  });
});

// ── L'interpolation ─────────────────────────────────────────────────────────

describe("la géométrie intermédiaire", () => {
  const from = naturalGeom([M, DEDANS]);
  const to = resolveItems([M, DEDANS]);
  const ids = tweenedIds(from, to);

  it("à mi-course, l'élément est à mi-chemin de sa taille", () => {
    const mid = tweenGeom(from, to, ids, 0.5);
    // 800 → 200 : la moitié du trajet vaut 500.
    expect(mid.get("I")!.width).toBe(500);
    expect(mid.get("I")!.scale).toBe(0.625); // (1 + 0,25) / 2
  });

  it("à p = 0 on est exactement au départ, à p = 1 exactement à l'arrivée", () => {
    expect(tweenGeom(from, to, ids, 0).get("I")!.width).toBe(from.get("I")!.width);
    expect(tweenGeom(from, to, ids, 1).get("I")!.width).toBe(to.get("I")!.width);
  });

  it("l'arrivée rend la carte cible ELLE-MÊME — l'animation ne laisse rien derrière", () => {
    expect(tweenGeom(from, to, ids, 1)).toBe(to);
    expect(tweenGeom(from, to, new Set(), 0.5)).toBe(to);
  });

  it("ce qui n'est pas animé est repris au MÊME objet près", () => {
    const mid = tweenGeom(from, to, ids, 0.5);
    expect(mid.get("M")).toBe(to.get("M"));
  });

  it("l'appartenance ne s'interpole pas : c'est un fait, pris dès la 1re image", () => {
    const mid = tweenGeom(from, to, ids, 0.01);
    expect(mid.get("I")!.membraneId).toBe("M");
  });

  it("l'échelle du CONTENU d'une membrane s'interpole aussi", () => {
    // Sans elle, les enfants d'une membrane imbriquée sauteraient alors que
    // leur parente glisse — la composition des échelles serait incohérente.
    const interne = memb("N", 0, 0, 400, 400, "minimized", "M");
    const dedansN = box("J", 0, 0, 800, 800, "N");
    const f = naturalGeom([M, interne, dedansN]);
    const t = resolveItems([M, interne, dedansN]);
    const mid = tweenGeom(f, t, tweenedIds(f, t), 0.5);
    const cs = mid.get("N")!.contentScale!;
    expect(cs).toBeGreaterThan(t.get("N")!.contentScale!);
    expect(cs).toBeLessThan(1);
  });
});

// ── Les deux bouts nuls ─────────────────────────────────────────────────────

describe("quand un côté de l'animation est `null` (chemin rapide)", () => {
  it("la PREMIÈRE membrane minimisée du board a quand même son animation", () => {
    // Avant elle, `geom` vaut `null` : sans point de départ, la transition la
    // plus spectaculaire serait la seule à sauter.
    const items = [M, DEDANS];
    const depart = geomOrNatural(items, null);
    expect(depart.get("I")!.scale).toBe(1);
    expect(depart.get("I")!.width).toBe(800);
    expect(tweenedIds(depart, resolveItems(items)).has("I")).toBe(true);
  });

  it("`geomOrNatural` rend la carte réelle quand elle existe", () => {
    const g = resolveItems([M, DEDANS]);
    expect(geomOrNatural([M, DEDANS], g)).toBe(g);
  });

  it("la géométrie naturelle sait qui est membre de qui", () => {
    expect(naturalGeom([M, DEDANS]).get("I")!.membraneId).toBe("M");
    expect(naturalGeom([M, LIBRE]).get("L")!.membraneId).toBeNull();
  });
});

// ── Adoucissement & réglages ────────────────────────────────────────────────

describe("adoucissement", () => {
  it("part de 0, arrive à 1, et démarre vite (easeOut)", () => {
    expect(ease(0)).toBe(0);
    expect(ease(1)).toBe(1);
    expect(ease(0.5)).toBeGreaterThan(0.5);
  });

  it("un avancement hors bornes ne casse rien", () => {
    expect(ease(-1)).toBe(0);
    expect(ease(2)).toBe(1);
  });

  it("la durée reste courte — on suit l'objet, on n'attend jamais", () => {
    expect(MEMBRANE_TWEEN.MS).toBeLessThanOrEqual(250);
  });
});

// ── Depuis un vrai board ────────────────────────────────────────────────────

describe("depuis un board", () => {
  const image = (id: string, x: number, y: number, membraneId?: string): BoardImage => ({
    id, x, y, width: 800, height: 800, rotation: 0, locked: false, tags: [],
    originalWidth: 800, originalHeight: 800,
    ...(membraneId ? { membraneId } : {}),
  } as BoardImage);

  const membrane = {
    id: "M", type: "membrane", x: 0, y: 0, width: 200, height: 200, mode: "minimized",
  } as Annotation;

  it("le dépôt d'une image change l'empreinte, et elle seule est animée", () => {
    const avantBoard = { images: [image("I", 400, 400)], annotations: [membrane] };
    const apresBoard = { images: [image("I", 400, 400, "M")], annotations: [membrane] };
    const avantItems = itemsOfBoard(avantBoard);
    const apresItems = itemsOfBoard(apresBoard);

    expect(membershipSignature(avantItems)).not.toBe(membershipSignature(apresItems));

    const from = geomOrNatural(avantItems, null);
    const to = resolveItems(apresItems);
    expect([...tweenedIds(from, to)]).toEqual(["I"]);
  });
});

// ── La décision de démarrer ─────────────────────────────────────────────────

describe("planTween — faut-il animer, et sur qui", () => {
  const avantItems = [M, LIBRE];
  const apresItems = [M, box("L", 0, 0, 800, 800, "M")];
  const sigAvant = membershipSignature(avantItems);
  const sigApres = membershipSignature(apresItems);

  const avant = { sig: sigAvant, items: avantItems, geom: null };
  const apres = { sig: sigApres, items: apresItems, geom: resolveItems(apresItems) };

  it("un dépôt dans une membrane minimisée déclenche l'animation", () => {
    const plan = planTween(avant, apres);
    expect(plan).not.toBeNull();
    expect([...plan!.ids]).toEqual(["L"]);
    expect(plan!.from.get("L")!.scale).toBe(1);
  });

  it("le PREMIER rendu n'anime rien — sinon tout le board ramperait à l'ouverture", () => {
    expect(planTween({ ...avant, sig: null }, apres)).toBeNull();
  });

  it("une empreinte inchangée n'anime rien — c'est l'immense majorité des rendus", () => {
    expect(planTween({ ...avant, sig: sigApres }, apres)).toBeNull();
  });

  it("un dépôt dans une membrane CLASSIQUE n'anime rien : elle ne réduit rien", () => {
    const classiqueAvant = [memb("C", 0, 0, 200, 200, "classic"), LIBRE];
    const classiqueApres = [
      memb("C", 0, 0, 200, 200, "classic"), box("L", 0, 0, 800, 800, "C"),
    ];
    const plan = planTween(
      { sig: membershipSignature(classiqueAvant), items: classiqueAvant, geom: null },
      { sig: membershipSignature(classiqueApres), items: classiqueApres, geom: null },
    );
    expect(plan).toBeNull();
  });

  it("SORTIR d'une membrane minimisée s'anime aussi — l'image regrossit", () => {
    const plan = planTween(
      { sig: sigApres, items: apresItems, geom: resolveItems(apresItems) },
      { sig: sigAvant, items: avantItems, geom: null },
    );
    expect(plan).not.toBeNull();
    expect(plan!.from.get("L")!.scale).toBe(0.25); // il part réduit…
    expect([...plan!.ids]).toEqual(["L"]);
  });

  it("passer de minimisée à étirée s'anime : le contenu reprend sa taille", () => {
    const mini = [M, DEDANS];
    const etiree = [memb("M", 0, 0, 200, 200, "stretched"), DEDANS];
    const plan = planTween(
      { sig: membershipSignature(mini), items: mini, geom: resolveItems(mini) },
      { sig: membershipSignature(etiree), items: etiree, geom: null },
    );
    expect(plan).not.toBeNull();
    expect([...plan!.ids]).toEqual(["I"]);
  });
});
