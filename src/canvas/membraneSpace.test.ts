import { describe, expect, it } from "vitest";
import {
  MEMBRANE_SPACE,
  canSwitchMode,
  containedIn,
  contentExtent,
  contentScale,
  imageBox,
  itemsOfBoard,
  membraneAtPoint,
  naturalDelta,
  parentMap,
  reconcileMembership,
  resolveItems,
  stretchPlan,
  type MembraneMode,
  type SpaceItem,
} from "./membraneSpace";
import type { Annotation, BoardImage } from "../types";

// ── Fabriques ───────────────────────────────────────────────────────────────

function memb(
  id: string, x: number, y: number, w: number, h: number,
  mode: MembraneMode = "classic", membraneId: string | null = null,
): SpaceItem {
  return { id, kind: "membrane", x, y, width: w, height: h, mode, membraneId };
}

function box(
  id: string, x: number, y: number, w: number, h: number,
  membraneId: string | null = null,
): SpaceItem {
  return { id, kind: "image", x, y, width: w, height: h, membraneId };
}

/** Géométrie effective d'un élément, en lecture directe. */
function eff(items: SpaceItem[], id: string, focusedMembraneId: string | null = null) {
  const r = resolveItems(items, { focusedMembraneId });
  const it = r.get(id);
  if (!it) throw new Error(`${id} absent de la résolution`);
  return it;
}

// ── Appartenance ────────────────────────────────────────────────────────────

describe("appartenance — stockée, jamais redérivée", () => {
  it("INVARIANT : une membrane ne perd pas son contenu en le réduisant", () => {
    // C'est LA raison d'être du choix « stockée ». Le centre naturel de l'image
    // est à (500,500), très loin des 100×100 de la membrane : une appartenance
    // géométrique l'aurait déclarée dehors à l'instant même de la réduction, et
    // la membrane se serait vidée en rangeant.
    const M = memb("M", 0, 0, 100, 100, "minimized");
    const I = box("I", 0, 0, 1000, 1000, "M");
    expect(parentMap([M, I]).get("I")).toBe("M");
    const r = eff([M, I], "I");
    expect(r.membraneId).toBe("M");
    expect(r.width).toBe(100); // réduite à 0,1 : elle tient pile dans la membrane
  });

  it("une référence vers une membrane supprimée libère l'élément", () => {
    expect(parentMap([box("I", 0, 0, 10, 10, "DISPARUE")]).has("I")).toBe(false);
  });

  it("un parent qui n'est pas une membrane est ignoré", () => {
    const items = [box("A", 0, 0, 10, 10), box("B", 0, 0, 10, 10, "A")];
    expect(parentMap(items).has("B")).toBe(false);
  });

  it("l'auto-référence est ignorée", () => {
    expect(parentMap([memb("M", 0, 0, 10, 10, "classic", "M")]).has("M")).toBe(false);
  });

  it("un cycle est coupé et la résolution termine quand même", () => {
    const A = memb("A", 0, 0, 100, 100, "minimized", "B");
    const B = memb("B", 0, 0, 100, 100, "minimized", "A");
    expect(parentMap([A, B]).size).toBe(0);
    expect(resolveItems([A, B]).size).toBe(2);
  });
});

describe("containedIn — l'instantané pris à la conversion", () => {
  const M = memb("M", 0, 0, 400, 400);

  it("retient les éléments dont le CENTRE est dans la membrane", () => {
    const dedans = box("dedans", 100, 100, 50, 50);
    const dehors = box("dehors", 380, 100, 100, 50); // centre en x = 430
    expect(containedIn([M, dedans, dehors], M).map((i) => i.id)).toEqual(["dedans"]);
  });

  it("ne se retient jamais elle-même", () => {
    expect(containedIn([M], M)).toEqual([]);
  });
});

describe("membraneAtPoint — la règle du dépôt, en géométrie vue", () => {
  const M = memb("M", 0, 0, 400, 300, "minimized");
  const I = box("I", 0, 0, 800, 600, "M");

  it("capture la membrane sous le curseur", () => {
    const items = [M, I];
    const r = resolveItems(items);
    expect(membraneAtPoint(items, r, 200, 150)?.id).toBe("M");
    expect(membraneAtPoint(items, r, 600, 150)).toBeNull();
  });

  it("entre membranes imbriquées, la plus petite à l'écran gagne", () => {
    const OUT = memb("OUT", 0, 0, 1000, 1000);
    const IN = memb("IN", 100, 100, 200, 200, "classic", "OUT");
    const items = [OUT, IN];
    const r = resolveItems(items);
    expect(membraneAtPoint(items, r, 150, 150)?.id).toBe("IN");
    expect(membraneAtPoint(items, r, 800, 800)?.id).toBe("OUT");
  });
});

// ── Échelle du contenu — le cœur ────────────────────────────────────────────

describe("échelle du contenu — déduite, jamais stockée", () => {
  const CONTENU = box("I", 0, 0, 800, 600, "M"); // étendue 800 × 600

  it("une membrane minimisée réduit son contenu pour le faire tenir", () => {
    const m = memb("M", 0, 0, 400, 300, "minimized");
    expect(contentScale("minimized", m, contentExtent(m, [CONTENU]))).toBe(0.5);
    expect(eff([m, CONTENU], "I").width).toBe(400);
  });

  it("étirer un SEUL axe ne fait pas regrossir le contenu (le min des deux)", () => {
    // Largeur doublée (400 → 800), hauteur inchangée : le facteur X passe à 1
    // mais le facteur Y reste à 0,5, donc min() ne bouge pas. C'est l'image
    // rectangle qui ne peut pas regrossir tant que l'autre côté n'a pas suivi.
    const large = memb("M", 0, 0, 800, 300, "minimized");
    expect(contentScale("minimized", large, contentExtent(large, [CONTENU]))).toBe(0.5);
    expect(eff([large, CONTENU], "I").width).toBe(400);
  });

  it("étirer le SECOND axe libère enfin la croissance", () => {
    const carre = memb("M", 0, 0, 800, 600, "minimized");
    expect(contentScale("minimized", carre, contentExtent(carre, [CONTENU]))).toBe(1);
    expect(eff([carre, CONTENU], "I").width).toBe(800);
  });

  it("l'échelle PLAFONNE à 1 : agrandir encore ne fait que créer du vide", () => {
    const vaste = memb("M", 0, 0, 1000, 800, "minimized");
    // min(1000/800, 800/600) = 1,25 → ramené à 1, jamais 1,25.
    expect(contentScale("minimized", vaste, contentExtent(vaste, [CONTENU]))).toBe(1);
  });

  it("un plancher empêche le contenu de disparaître complètement", () => {
    const minuscule = memb("M", 0, 0, 1, 1, "minimized");
    expect(contentScale("minimized", minuscule, { w: 10000, h: 10000 }))
      .toBe(MEMBRANE_SPACE.MIN_CONTENT_SCALE);
  });

  it("classique et étirée ne réduisent JAMAIS — elles laissent déborder ou poussent", () => {
    const m = memb("M", 0, 0, 400, 300);
    const extent = contentExtent(m, [CONTENU]);
    expect(contentScale("classic", m, extent)).toBe(1);
    expect(contentScale("stretched", m, extent)).toBe(1);
  });

  it("une membrane vide reste à l'échelle 1 (pas de division par zéro)", () => {
    const m = memb("M", 0, 0, 400, 300, "minimized");
    expect(contentScale("minimized", m, contentExtent(m, []))).toBe(1);
  });
});

// ── Mode Focus ──────────────────────────────────────────────────────────────

describe("mode Focus — « on ne s'aperçoit de rien »", () => {
  const M = memb("M", 0, 0, 400, 300, "minimized");
  const I = box("I", 0, 0, 800, 600, "M");

  it("sous focus, le contenu retrouve sa taille naturelle sans toucher aux données", () => {
    expect(eff([M, I], "I").width).toBe(400);          // vue normale : réduit
    expect(eff([M, I], "I", "M").width).toBe(800);     // focus : taille réelle
    expect(eff([M, I], "I", "M").scale).toBe(1);
    expect(I.width).toBe(800);                          // la donnée n'a pas bougé
  });

  it("le focus se propage aux membranes imbriquées", () => {
    const inner = memb("IN", 0, 0, 100, 100, "minimized", "M");
    const deep = box("D", 0, 0, 400, 400, "IN");
    expect(eff([M, inner, deep], "D", "M").scale).toBe(1);
  });

  it("focaliser une AUTRE membrane ne change rien ici", () => {
    const autre = memb("AUTRE", 5000, 5000, 100, 100);
    expect(eff([M, I, autre], "I", "AUTRE").width).toBe(400);
  });
});

// ── Composition (membranes imbriquées) ──────────────────────────────────────

describe("membranes imbriquées — les échelles se composent", () => {
  it("deux niveaux de réduction se multiplient", () => {
    const outer = memb("O", 0, 0, 500, 500, "minimized");
    const inner = memb("IN", 0, 0, 1000, 1000, "minimized", "O");
    const leaf = box("L", 0, 0, 2000, 2000, "IN");
    const items = [outer, inner, leaf];
    // O : son contenu est la boîte de IN (1000) → 500/1000 = 0,5.
    // IN : son contenu est L (2000) → 1000/2000 = 0,5. Cumul : 0,25.
    expect(eff(items, "IN").scale).toBe(0.5);
    expect(eff(items, "L").scale).toBe(0.25);
    expect(eff(items, "L").width).toBe(500);
  });

  it("la mise à l'échelle s'ancre sur le coin haut-gauche de la membrane", () => {
    // Un voisin ne doit pas bouger quand on déplace un autre élément : l'ancre
    // est un point FIXE de la membrane, pas le centre du contenu.
    const m = memb("M", 100, 100, 200, 200, "minimized");
    const a = box("A", 100, 100, 400, 400, "M");
    expect(eff([m, a], "A").x).toBe(100); // l'élément posé sur l'ancre n'y bouge pas
    expect(eff([m, a], "A").width).toBe(200);
  });

  it("un élément libre n'est pas touché par les membranes alentour", () => {
    const m = memb("M", 0, 0, 100, 100, "minimized");
    const dedans = box("D", 0, 0, 400, 400, "M");
    const libre = box("L", 0, 0, 400, 400);
    const items = [m, dedans, libre];
    expect(eff(items, "L").width).toBe(400);
    expect(eff(items, "L").scale).toBe(1);
    expect(eff(items, "D").width).toBe(100);
  });
});

// ── Mode étiré & collisions ─────────────────────────────────────────────────

describe("mode étiré — la membrane pousse, mais bute sur ce qui n'est pas à elle", () => {
  const M = memb("M", 0, 0, 200, 200, "stretched");
  const DEDANS = box("I", 0, 0, 300, 100, "M"); // déborde à droite

  it("sans obstacle, la membrane grandit pour contenir son contenu", () => {
    const plan = stretchPlan(M, [DEDANS], []);
    expect(plan.desired.width).toBe(300 + MEMBRANE_SPACE.STRETCH_PADDING);
    expect(plan.allowed).toEqual(plan.desired);
    expect(plan.blocked).toBe(false);
    expect(plan.blockers).toEqual([]);
  });

  it("un élément étranger sur le chemin ARRÊTE la croissance et se signale", () => {
    const etranger = box("ETR", 250, 0, 50, 50);
    const plan = stretchPlan(M, [DEDANS], [etranger]);
    expect(plan.blocked).toBe(true);
    expect(plan.blockers.map((b) => b.id)).toEqual(["ETR"]);
    // On s'arrête AU CONTACT, sans jamais le recouvrir ni le capturer.
    expect(plan.allowed.width).toBe(250);
    expect(plan.allowed.width).toBeLessThan(plan.desired.width);
  });

  it("la membrane ne rétrécit jamais en dessous de sa taille actuelle", () => {
    const colle = box("COLLE", 100, 0, 20, 20);
    const plan = stretchPlan(M, [DEDANS], [colle]);
    expect(plan.allowed.width).toBeGreaterThanOrEqual(M.width);
  });

  it("un chevauchement PRÉEXISTANT ne bloque pas rétroactivement", () => {
    // Il était déjà sous la membrane avant toute croissance : ce n'est pas ce
    // mouvement-ci qui vient de le heurter.
    const deja = box("DEJA", 50, 50, 20, 20);
    const plan = stretchPlan(M, [DEDANS], [deja]);
    expect(plan.blockers).toEqual([]);
    expect(plan.blocked).toBe(false);
  });

  it("l'obstacle est raboté sur l'axe où il coûte le moins de reculer", () => {
    // Obstacle bas et large : c'est la hauteur qu'on sacrifie, pas la largeur.
    const grand = memb("M2", 0, 0, 100, 100, "stretched");
    const contenu = box("C", 0, 0, 400, 400, "M2");
    const bas = box("BAS", 0, 150, 500, 50);
    const plan = stretchPlan(grand, [contenu], [bas]);
    expect(plan.allowed.height).toBe(150);
    expect(plan.allowed.width).toBe(plan.desired.width);
  });
});

// ── Transitions de mode ─────────────────────────────────────────────────────

describe("transitions de mode — un aller sans retour", () => {
  it("minimisée ↔ étirée est permis", () => {
    expect(canSwitchMode("minimized", "stretched")).toBe(true);
    expect(canSwitchMode("stretched", "minimized")).toBe(true);
  });

  it("classique peut devenir spéciale", () => {
    expect(canSwitchMode("classic", "minimized")).toBe(true);
    expect(canSwitchMode("classic", "stretched")).toBe(true);
  });

  it("mais une membrane spéciale ne redevient JAMAIS classique", () => {
    expect(canSwitchMode("minimized", "classic")).toBe(false);
    expect(canSwitchMode("stretched", "classic")).toBe(false);
  });

  it("rester sur place est toujours permis", () => {
    expect(canSwitchMode("minimized", "minimized")).toBe(true);
    expect(canSwitchMode("classic", "classic")).toBe(true);
  });
});

// ── Écriture inverse (glisser dans une membrane réduite) ────────────────────

describe("retour vers les coordonnées naturelles", () => {
  it("un déplacement écran se divise par l'échelle avant d'être écrit", () => {
    // Sans ça, glisser dans une membrane à 0,4 ferait filer l'élément 2,5× trop vite.
    expect(naturalDelta(10, 20, 0.5)).toEqual({ dx: 20, dy: 40 });
    expect(naturalDelta(10, 20, 1)).toEqual({ dx: 10, dy: 20 });
  });

  it("le plancher d'échelle protège d'une division explosive", () => {
    expect(Number.isFinite(naturalDelta(1, 0, 0).dx)).toBe(true);
  });
});

// ── Adaptateur Board ────────────────────────────────────────────────────────

describe("adaptateur — un board devient des boîtes", () => {
  const img = (id: string, x: number, y: number, w: number, h: number): BoardImage => ({
    id, x, y, width: w, height: h, rotation: 0, locked: false, tags: [],
    originalWidth: w, originalHeight: h,
  });

  it("les images passent du centre (Pixi) au coin haut-gauche", () => {
    expect(imageBox(img("I", 100, 100, 200, 150)))
      .toEqual({ x: 0, y: 25, width: 200, height: 150 });
  });

  it("les flèches sont exclues — elles suivent les nœuds qu'elles relient", () => {
    const arrow: Annotation = { id: "A", type: "arrow", x: 0, y: 0, x2: 10, y2: 10 };
    expect(itemsOfBoard({ images: [], annotations: [arrow] })).toEqual([]);
  });

  it("un texte pas encore mesuré est ignoré plutôt que placé au hasard", () => {
    const t: Annotation = { id: "T", type: "text", x: 0, y: 0, text: "salut" };
    expect(itemsOfBoard({ images: [], annotations: [t] })).toEqual([]);
  });

  it("une membrane sans champ mode se comporte comme classique", () => {
    const m: Annotation = { id: "M", type: "membrane", x: 0, y: 0, width: 100, height: 100 };
    expect(itemsOfBoard({ images: [], annotations: [m] })[0].mode).toBe("classic");
  });

  it("un board complet se résout d'un bloc, appartenance comprise", () => {
    const m: Annotation = {
      id: "M", type: "membrane", x: 0, y: 0, width: 400, height: 300, mode: "minimized",
    };
    const image = { ...img("I", 400, 300, 800, 600), membraneId: "M" };
    const resolved = resolveItems(itemsOfBoard({ images: [image], annotations: [m] }));
    expect(resolved.get("I")?.width).toBe(400);
    expect(resolved.get("I")?.membraneId).toBe("M");
  });

  it("un projet d'avant la fonctionnalité se résout à l'identique", () => {
    // Aucun `mode`, aucun `membraneId` : tout reste à l'échelle 1, exactement
    // comme avant l'introduction du repère.
    const m: Annotation = { id: "M", type: "membrane", x: 0, y: 0, width: 400, height: 300 };
    const resolved = resolveItems(itemsOfBoard({
      images: [img("I", 100, 100, 200, 150)],
      annotations: [m],
    }));
    expect(resolved.get("I")).toMatchObject({ x: 0, y: 25, width: 200, height: 150, scale: 1 });
    expect(resolved.get("M")).toMatchObject({ x: 0, y: 0, width: 400, height: 300, scale: 1 });
  });
});

// ── Appartenance au dépôt ───────────────────────────────────────────────────

describe("reconcileMembership — « je le glisse dedans, il lui appartient »", () => {
  const M = memb("M", 0, 0, 400, 400);
  const AUTRE = memb("AUTRE", 1000, 0, 400, 400);

  /** Raccourci : les changements produits pour un dépôt de `ids`. */
  function apres(items: SpaceItem[], ids: string[]) {
    return reconcileMembership(items, resolveItems(items), ids);
  }

  it("un élément lâché dans une membrane la rejoint", () => {
    const libre = box("I", 100, 100, 50, 50);
    expect(apres([M, libre], ["I"])).toEqual([{ id: "I", membraneId: "M" }]);
  });

  it("un élément sorti de la membrane redevient libre", () => {
    const parti = box("I", 5000, 5000, 50, 50, "M");
    expect(apres([M, parti], ["I"])).toEqual([{ id: "I", membraneId: null }]);
  });

  it("rien à écrire quand rien n'a changé — pas de mutation inutile", () => {
    expect(apres([M, box("I", 100, 100, 50, 50, "M")], ["I"])).toEqual([]);
    expect(apres([M, box("I", 5000, 5000, 50, 50)], ["I"])).toEqual([]);
  });

  it("seuls les éléments déplacés sont examinés", () => {
    const bouge = box("A", 100, 100, 50, 50);
    const pasBouge = box("B", 120, 120, 50, 50);
    expect(apres([M, bouge, pasBouge], ["A"]).map((c) => c.id)).toEqual(["A"]);
  });

  it("on change de membrane d'un seul geste", () => {
    const migre = box("I", 1100, 100, 50, 50, "M");
    expect(apres([M, AUTRE, migre], ["I"])).toEqual([{ id: "I", membraneId: "AUTRE" }]);
  });

  it("entre membranes imbriquées, la plus petite l'emporte", () => {
    const petite = memb("PETITE", 50, 50, 100, 100);
    const it0 = box("I", 80, 80, 20, 20);
    expect(apres([M, petite, it0], ["I"])).toEqual([{ id: "I", membraneId: "PETITE" }]);
  });

  it("une membrane peut elle-même rejoindre une autre membrane", () => {
    const petite = memb("PETITE", 50, 50, 100, 100);
    expect(apres([M, petite], ["PETITE"])).toEqual([{ id: "PETITE", membraneId: "M" }]);
  });

  it("mais JAMAIS sa propre descendance — un cycle rend l'arbre inrésoluble", () => {
    // La grande englobe la petite, qui lui appartient déjà. Déplacer la GRANDE
    // ne doit pas la faire entrer dans sa propre enfant.
    const grande = memb("GRANDE", 0, 0, 400, 400);
    const petite = memb("PETITE", 0, 0, 380, 380, "classic", "GRANDE");
    const changes = apres([grande, petite], ["GRANDE"]);
    expect(changes).toEqual([]);
  });

  it("le centre décide, pas le recouvrement", () => {
    // À cheval sur le bord, centre dehors : il reste libre.
    const cheval = box("I", 380, 100, 100, 50);
    expect(apres([M, cheval], ["I"])).toEqual([]);
  });

  it("un identifiant inconnu est ignoré sans broncher", () => {
    expect(apres([M], ["fantome"])).toEqual([]);
  });

  it("aucun déplacement, aucun travail", () => {
    expect(reconcileMembership([M], new Map(), [])).toEqual([]);
  });

  it("juge sur la géométrie VUE, pas sur les coordonnées naturelles", () => {
    // Dans une membrane minimisée, le contenu est rendu bien plus près de
    // l'origine que ne le disent ses coordonnées. Un élément dont le centre
    // naturel est hors du cadre peut donc être visiblement DEDANS.
    const mini = memb("MINI", 0, 0, 200, 200, "minimized");
    const dedans = box("D", 0, 0, 800, 800, "MINI");   // rendu à l'échelle 0,25
    const nouveau = box("N", 100, 100, 40, 40);        // centre vu à (120,120)
    const items = [mini, dedans, nouveau];
    expect(reconcileMembership(items, resolveItems(items), ["N"]))
      .toEqual([{ id: "N", membraneId: "MINI" }]);
  });
});
