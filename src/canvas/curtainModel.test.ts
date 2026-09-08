import { describe, expect, it } from "vitest";
import {
  EXPAND_STEP,
  detachCurtain,
  detachCurtains,
  MAX_NOTE_LENGTH,
  canEdit,
  canSee,
  configOf,
  createCurtain,
  createNote,
  curtainKind,
  sanitizeNoteText,
  stepExpanded,
  visibleCurtains,
  type MembraneCurtain,
} from "./curtainModel";
import { CURTAIN, normalizeConfig } from "./curtainPanel";

const MOI = { id: "u-moi", name: "Ada", color: "#38bdf8" };
const TOI = { id: "u-toi", name: "Grace", color: "#34d399" };

function curtain(over: Partial<MembraneCurtain> = {}): MembraneCurtain {
  return { ...createCurtain(MOI, 1000), ...over };
}

// ── Création ────────────────────────────────────────────────────────────────

describe("création", () => {
  it("un rideau neuf est PRIVÉ — on ouvre le sien volontairement", () => {
    // Mieux vaut avoir à partager que découvrir après coup que tout le monde
    // lisait ses brouillons.
    const c = createCurtain(MOI);
    expect(c.visibility).toBe("private");
    expect(c.editable).toBe("owner");
    expect(c.notes).toEqual([]);
  });

  it("recopie le nom et la couleur du propriétaire", () => {
    // Sans cette copie, la languette d'un pair déconnecté n'aurait ni nom ni
    // couleur : la présence ne circule que tant qu'il est là.
    const c = createCurtain(MOI);
    expect(c.ownerId).toBe(MOI.id);
    expect(c.ownerName).toBe("Ada");
    expect(c.ownerColor).toBe("#38bdf8");
  });

  it("deux rideaux ne partagent jamais d'identifiant", () => {
    expect(createCurtain(MOI).id).not.toBe(createCurtain(MOI).id);
  });
});

describe("notes", () => {
  it("borne la longueur et retire les blancs de bord", () => {
    expect(sanitizeNoteText("  salut  ")).toBe("salut");
    expect(sanitizeNoteText("x".repeat(5000))).toHaveLength(MAX_NOTE_LENGTH);
  });

  it("normalise les fins de ligne", () => {
    expect(sanitizeNoteText("a\r\nb")).toBe("a\nb");
  });

  it("garde les retours à la ligne internes — une note peut respirer", () => {
    expect(createNote("un\ndeux").text).toBe("un\ndeux");
  });
});

// ── Permissions ─────────────────────────────────────────────────────────────

describe("permissions — les trois usages", () => {
  it("CARNET : privé, invisible et intouchable pour les autres", () => {
    const c = curtain({ visibility: "private", editable: "owner" });
    expect(curtainKind(c)).toBe("carnet");
    expect(canSee(c, MOI.id)).toBe(true);
    expect(canEdit(c, MOI.id)).toBe(true);
    expect(canSee(c, TOI.id)).toBe(false);
    expect(canEdit(c, TOI.id)).toBe(false);
  });

  it("VITRINE : partagé, on regarde sans toucher", () => {
    const c = curtain({ visibility: "shared", editable: "owner" });
    expect(curtainKind(c)).toBe("vitrine");
    expect(canSee(c, TOI.id)).toBe(true);
    expect(canEdit(c, TOI.id)).toBe(false);
    expect(canEdit(c, MOI.id)).toBe(true);
  });

  it("ATELIER : partagé et ouvert, chacun y dépose", () => {
    const c = curtain({ visibility: "shared", editable: "everyone" });
    expect(curtainKind(c)).toBe("atelier");
    expect(canSee(c, TOI.id)).toBe(true);
    expect(canEdit(c, TOI.id)).toBe(true);
  });

  it("« privé mais modifiable par tous » ne veut rien dire — et ne donne rien", () => {
    // Un document collaboratif peut arriver dans un état incohérent ; on ne se
    // fie pas à `editable` seul.
    const incoherent = curtain({ visibility: "private", editable: "everyone" });
    expect(canSee(incoherent, TOI.id)).toBe(false);
    expect(canEdit(incoherent, TOI.id)).toBe(false);
  });

  it("le propriétaire garde toujours la main sur le sien", () => {
    for (const v of ["private", "shared"] as const) {
      for (const e of ["owner", "everyone"] as const) {
        expect(canEdit(curtain({ visibility: v, editable: e }), MOI.id)).toBe(true);
      }
    }
  });
});

describe("liste visible", () => {
  const mien = curtain({ id: "a", ownerId: MOI.id, createdAt: 300 });
  const sienPartage = curtain({ id: "b", ownerId: TOI.id, visibility: "shared", createdAt: 100 });
  const sienPrive = curtain({ id: "c", ownerId: TOI.id, visibility: "private", createdAt: 200 });

  it("cache les rideaux privés des autres", () => {
    const vus = visibleCurtains([mien, sienPartage, sienPrive], MOI.id).map((c) => c.id);
    expect(vus).not.toContain("c");
  });

  it("met le mien en tête — c'est celui qu'on cherche", () => {
    expect(visibleCurtains([sienPartage, mien], MOI.id)[0].id).toBe("a");
  });

  it("puis les autres par ancienneté, ordre stable", () => {
    const autre = curtain({ id: "d", ownerId: "u-x", visibility: "shared", createdAt: 50 });
    const vus = visibleCurtains([mien, sienPartage, autre], MOI.id).map((c) => c.id);
    expect(vus).toEqual(["a", "d", "b"]);
  });

  it("quelqu'un sans rideau n'en voit aucun de privé", () => {
    expect(visibleCurtains([mien, sienPrive], "u-inconnu")).toEqual([]);
  });
});

// ── Proportions ─────────────────────────────────────────────────────────────

describe("proportions par rideau", () => {
  it("sans réglage, les valeurs par défaut", () => {
    expect(configOf(curtain())).toEqual(normalizeConfig());
    expect(configOf(null)).toEqual(normalizeConfig());
  });

  it("un réglage personnel est respecté", () => {
    expect(configOf(curtain({ collapsedRatio: 0.15, expandedRatio: 0.5 })))
      .toEqual({ collapsed: 0.15, expanded: 0.5 });
  });

  it("un réglage aberrant est ramené dans les bornes, jamais accepté tel quel", () => {
    const c = configOf(curtain({ collapsedRatio: -1, expandedRatio: 99 }));
    expect(c.collapsed).toBeGreaterThan(0);
    expect(c.expanded).toBeLessThan(1);
    expect(c.expanded).toBeGreaterThan(c.collapsed);
  });

  it("élargir puis rétrécir revient au point de départ, LOIN des bornes", () => {
    const base = 0.6;
    expect(stepExpanded(stepExpanded(base, EXPAND_STEP), -EXPAND_STEP)).toBeCloseTo(base, 10);
  });

  it("mais contre une borne, le pas est rogné et le retour ne revient pas", () => {
    // Comportement correct d'un pas borné, et non un défaut : le premier pas a
    // buté sur MAX_EXPANDED, le second repart donc de la borne, pas de la valeur
    // demandée. On le note pour que personne ne s'en étonne devant l'écran.
    const haut = CURTAIN.MAX_EXPANDED;
    expect(stepExpanded(haut, EXPAND_STEP)).toBe(haut);
    expect(stepExpanded(stepExpanded(haut, EXPAND_STEP), -EXPAND_STEP))
      .toBeCloseTo(haut - EXPAND_STEP, 10);
  });

  it("on ne peut pas élargir au-delà de la borne — la bande de canvas survit", () => {
    let v = normalizeConfig().expanded;
    for (let i = 0; i < 20; i++) v = stepExpanded(v, EXPAND_STEP);
    expect(v).toBeLessThanOrEqual(CURTAIN.MAX_EXPANDED);
    expect(v).toBeLessThan(1);
  });

  it("ni rétrécir en dessous — la languette survit aussi", () => {
    let v = normalizeConfig().expanded;
    for (let i = 0; i < 20; i++) v = stepExpanded(v, -EXPAND_STEP);
    expect(v).toBeGreaterThanOrEqual(CURTAIN.MIN_EXPANDED);
  });
});

// ── Détachement ─────────────────────────────────────────────────────────────

describe("détachement — obligatoire avant toute écriture", () => {
  // Automerge refuse qu'un objet déjà dans le document y soit réinséré. Or
  // réécrire la liste réinsère fatalement les rideaux qu'on n'a pas touchés.
  // Sans recopie, l'application levait dès le deuxième rideau ou la deuxième
  // note — et seulement en collaboration, donc au pire moment.
  const source = curtain({
    notes: [{ id: "n1", text: "a", createdAt: 1 }, { id: "n2", text: "b", createdAt: 2 }],
    expandedRatio: 0.7,
  });

  it("rend une copie profonde : plus aucune référence partagée", () => {
    const copie = detachCurtain(source);
    expect(copie).toEqual(source);
    expect(copie).not.toBe(source);
    expect(copie.notes).not.toBe(source.notes);
    expect(copie.notes[0]).not.toBe(source.notes[0]);
  });

  it("n'écrit pas de champ optionnel absent", () => {
    // Poser `expandedRatio: undefined` dans le document n'est pas la même chose
    // que de ne pas l'avoir.
    const sansReglage = detachCurtain(curtain());
    expect("expandedRatio" in sansReglage).toBe(false);
    expect("collapsedRatio" in sansReglage).toBe(false);
  });

  it("préserve un champ optionnel présent", () => {
    expect(detachCurtain(source).expandedRatio).toBe(0.7);
  });

  it("détache la liste entière, élément par élément", () => {
    const liste = [source, curtain({ id: "autre" })];
    const copie = detachCurtains(liste);
    expect(copie).toEqual(liste);
    copie.forEach((c, i) => expect(c).not.toBe(liste[i]));
  });
});
