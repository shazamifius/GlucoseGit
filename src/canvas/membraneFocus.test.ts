import { describe, expect, it } from "vitest";
import {
  FOCUS,
  NO_FOCUS,
  annotationVisibleUnderFocus,
  arrowVisibleUnderFocus,
  focusBackground,
  focusFrameOf,
  coverage,
  fitViewport,
  focusBox,
  focusDecision,
  screenCenterWorld,
  visibleUnderFocus,
  type FocusState,
  type Viewport,
} from "./membraneFocus";
import { resolveItems, type MembraneMode, type SpaceItem } from "./membraneSpace";
import type { Annotation } from "../types";

const SCREEN = { width: 1000, height: 800 };

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

/** Viewport qui centre `(cx, cy)` du monde au milieu de l'écran, à `scale`. */
function centeredOn(cx: number, cy: number, scale: number): Viewport {
  return { scale, x: SCREEN.width / 2 - cx * scale, y: SCREEN.height / 2 - cy * scale };
}

function decide(items: SpaceItem[], vp: Viewport, state: FocusState, now: number) {
  return focusDecision({ items, resolved: resolveItems(items), vp, screen: SCREEN, state, now });
}

// ── Cadrage ─────────────────────────────────────────────────────────────────

describe("cadrage", () => {
  it("fitViewport centre la boîte et la fait tenir avec sa marge", () => {
    const vp = fitViewport({ x: 0, y: 0, width: 500, height: 400 }, SCREEN);
    // min(1000·0,88/500, 800·0,88/400) = min(1,76 ; 1,76)
    expect(vp.scale).toBeCloseTo(1.76, 6);
    // Le centre monde (250,200) atterrit au centre écran (500,400).
    const c = screenCenterWorld(vp, SCREEN);
    expect(c.x).toBeCloseTo(250, 6);
    expect(c.y).toBeCloseTo(200, 6);
  });

  it("focusBox agrandit une membrane minimisée pour contenir son contenu réel", () => {
    // Sous focus le contenu reprend sa taille naturelle : le cadre doit suivre,
    // sinon le contenu déborderait de sa propre membrane.
    const m = memb("M", 0, 0, 100, 100, "minimized");
    const enfant = box("I", 0, 0, 1000, 1000, "M");
    expect(focusBox(m, [enfant])).toEqual({ x: 0, y: 0, width: 1000, height: 1000 });
  });

  it("focusBox ne rétrécit jamais une membrane plus grande que son contenu", () => {
    const m = memb("M", 0, 0, 900, 900, "classic");
    expect(focusBox(m, [box("I", 0, 0, 100, 100, "M")]))
      .toEqual({ x: 0, y: 0, width: 900, height: 900 });
  });

  it("coverage vaut 1 quand la boîte déborde de tous les côtés", () => {
    expect(coverage({ x: 0, y: 0, width: 500, height: 400 }, centeredOn(250, 200, 2), SCREEN))
      .toBeCloseTo(1, 6);
  });
});

// ── Entrée ──────────────────────────────────────────────────────────────────

describe("entrée en focus", () => {
  const M = memb("M", 0, 0, 500, 400);

  it("entre quand la membrane remplit l'écran, et cadre dessus", () => {
    const r = decide([M], centeredOn(250, 200, 2), NO_FOCUS, 10_000);
    expect(r.kind).toBe("enter");
    if (r.kind !== "enter") return;
    expect(r.membraneId).toBe("M");
    expect(r.fit.scale).toBeCloseTo(1.76, 6);
    expect(r.state.enterScale).toBeCloseTo(1.76, 6);
  });

  it("n'entre pas tant qu'on n'est pas assez zoomé", () => {
    expect(decide([M], centeredOn(250, 200, 1.5), NO_FOCUS, 10_000).kind).toBe("stay");
  });

  it("n'entre pas si le centre de l'écran est hors de la membrane", () => {
    // Assez zoomé pour couvrir l'écran, mais on regarde à côté.
    const vp = centeredOn(5000, 5000, 2);
    expect(decide([M], vp, NO_FOCUS, 10_000).kind).toBe("stay");
  });

  it("entre dans la PLUS PETITE membrane qui remplit l'écran", () => {
    const GRANDE = memb("GRANDE", 0, 0, 2000, 1600);
    const PETITE = memb("PETITE", 200, 100, 500, 400, "classic", "GRANDE");
    const r = decide([GRANDE, PETITE], centeredOn(450, 300, 2), NO_FOCUS, 10_000);
    expect(r.kind).toBe("enter");
    if (r.kind === "enter") expect(r.membraneId).toBe("PETITE");
  });

  it("aucune membrane sous les yeux → rien ne se passe", () => {
    expect(decide([box("I", 0, 0, 100, 100)], centeredOn(50, 50, 40), NO_FOCUS, 10_000).kind)
      .toBe("stay");
  });
});

// ── L'absence d'oscillation, démontrée ──────────────────────────────────────

describe("le cadrage ne peut pas provoquer sa propre annulation", () => {
  const M = memb("M", 0, 0, 500, 400);

  it("PREUVE : après cadrage, la couverture retombe SOUS le seuil d'entrée", () => {
    // C'est le piège que l'asymétrie entrée/sortie évite. Si la sortie était
    // jugée sur la couverture, ce test montrerait qu'on ressort aussitôt.
    const fit = fitViewport(focusBox(M, []), SCREEN);
    const cov = coverage({ x: 0, y: 0, width: 500, height: 400 }, fit, SCREEN);
    expect(cov).toBeLessThan(FOCUS.ENTER_COVERAGE);
    expect(cov).toBeCloseTo(0.88 * 0.88, 6);
  });

  it("et pourtant on RESTE en focus une fois cadré", () => {
    const enter = decide([M], centeredOn(250, 200, 2), NO_FOCUS, 10_000);
    expect(enter.kind).toBe("enter");
    if (enter.kind !== "enter") return;
    const after = decide([M], enter.fit, enter.state, 10_000 + FOCUS.COOLDOWN_MS + 1);
    expect(after.kind).toBe("stay");
  });

  it("SIMULATION : 300 images de gigue autour du cadrage ne basculent qu'une fois", () => {
    let state = NO_FOCUS;
    let vp = centeredOn(250, 200, 2);
    let transitions = 0;
    for (let i = 0; i < 300; i++) {
      const r = decide([M], vp, state, 10_000 + i * 200);
      if (r.kind !== "stay") transitions++;
      state = r.state;
      if (r.kind === "enter") vp = r.fit;
      // Gigue : micro-pan et micro-zoom, comme une main sur un trackpad.
      vp = {
        scale: vp.scale * (1 + (i % 2 === 0 ? 0.001 : -0.001)),
        x: vp.x + (i % 3 === 0 ? 0.5 : -0.5),
        y: vp.y + (i % 5 === 0 ? 0.5 : -0.5),
      };
    }
    expect(transitions).toBe(1);
    expect(state.membraneId).toBe("M");
  });
});

// ── Sortie ──────────────────────────────────────────────────────────────────

describe("sortie de focus", () => {
  const M = memb("M", 0, 0, 500, 400);

  function entered(now = 10_000) {
    const r = decide([M], centeredOn(250, 200, 2), NO_FOCUS, now);
    if (r.kind !== "enter") throw new Error("entrée attendue");
    return r;
  }

  it("sort quand on a dézoomé sous le seuil", () => {
    const e = entered();
    const trop = e.state.enterScale * FOCUS.EXIT_SCALE_RATIO - 0.01;
    const r = decide([M], centeredOn(250, 200, trop), e.state, 20_000);
    expect(r.kind).toBe("exit");
    expect(r.state.membraneId).toBeNull();
  });

  it("ne sort pas pour un dézoom léger", () => {
    const e = entered();
    const peu = e.state.enterScale * FOCUS.EXIT_SCALE_RATIO + 0.01;
    expect(decide([M], centeredOn(250, 200, peu), e.state, 20_000).kind).toBe("stay");
  });

  it("sort si on s'éloigne franchement du centre de la membrane", () => {
    const e = entered();
    const loin = centeredOn(5000, 200, e.state.enterScale);
    expect(decide([M], loin, e.state, 20_000).kind).toBe("exit");
  });

  it("ne sort pas pour un pan d'un cheveu", () => {
    const e = entered();
    const presque = centeredOn(260, 210, e.state.enterScale);
    expect(decide([M], presque, e.state, 20_000).kind).toBe("stay");
  });

  it("sort si la membrane focalisée disparaît (suppression, undo)", () => {
    const e = entered();
    expect(decide([], centeredOn(250, 200, e.state.enterScale), e.state, 20_000).kind)
      .toBe("exit");
  });

  it("PREUVE : après une sortie, aucune ré-entrée immédiate", () => {
    const e = entered();
    const trop = e.state.enterScale * FOCUS.EXIT_SCALE_RATIO - 0.01;
    const vp = centeredOn(250, 200, trop);
    const out = decide([M], vp, e.state, 20_000);
    expect(out.kind).toBe("exit");
    // Même viewport, temps mort écoulé : la couverture est trop faible pour
    // rentrer. La bascule est donc stable des DEUX côtés.
    const again = decide([M], vp, out.state, 20_000 + FOCUS.COOLDOWN_MS + 1);
    expect(again.kind).toBe("stay");
    expect(coverage({ x: 0, y: 0, width: 500, height: 400 }, vp, SCREEN))
      .toBeLessThan(FOCUS.ENTER_COVERAGE);
  });
});

describe("temps mort", () => {
  const M = memb("M", 0, 0, 500, 400);

  it("aucune transition ne s'enchaîne dans la foulée d'une autre", () => {
    const e = decide([M], centeredOn(250, 200, 2), NO_FOCUS, 10_000);
    if (e.kind !== "enter") throw new Error("entrée attendue");
    // Conditions de sortie réunies, mais trop tôt.
    const tot = decide([M], centeredOn(9000, 9000, 0.1), e.state, 10_000 + FOCUS.COOLDOWN_MS - 1);
    expect(tot.kind).toBe("stay");
    const tard = decide([M], centeredOn(9000, 9000, 0.1), e.state, 10_000 + FOCUS.COOLDOWN_MS + 1);
    expect(tard.kind).toBe("exit");
  });
});

// ── Visibilité ──────────────────────────────────────────────────────────────

describe("visibilité sous focus", () => {
  it("hors focus, aucun filtre", () => {
    expect(visibleUnderFocus([memb("M", 0, 0, 10, 10)], null)).toBeNull();
  });

  it("la membrane et TOUTE sa descendance restent visibles", () => {
    const items = [
      memb("M", 0, 0, 400, 400, "minimized"),
      box("direct", 0, 0, 50, 50, "M"),
      memb("SOUS", 0, 0, 100, 100, "minimized", "M"),
      box("profond", 0, 0, 20, 20, "SOUS"),
    ];
    const v = visibleUnderFocus(items, "M")!;
    expect([...v].sort()).toEqual(["M", "SOUS", "direct", "profond"]);
  });

  it("ce qui est dehors disparaît", () => {
    const items = [memb("M", 0, 0, 400, 400), box("dehors", 5000, 5000, 50, 50)];
    expect(visibleUnderFocus(items, "M")!.has("dehors")).toBe(false);
  });

  it("un projet d'AVANT (aucun membraneId stocké) montre quand même son contenu", () => {
    // Sans ce filet, focaliser une membrane historique donnerait un écran vide.
    const items = [memb("M", 0, 0, 400, 400), box("legacy", 100, 100, 50, 50)];
    expect(visibleUnderFocus(items, "M")!.has("legacy")).toBe(true);
  });

  it("mais un élément appartenant à une AUTRE membrane n'est pas récupéré", () => {
    const items = [
      memb("M", 0, 0, 400, 400),
      memb("AUTRE", 0, 0, 400, 400),
      box("a-autrui", 100, 100, 50, 50, "AUTRE"),
    ];
    expect(visibleUnderFocus(items, "M")!.has("a-autrui")).toBe(false);
  });

  it("focaliser une membrane inconnue ne masque rien", () => {
    expect(visibleUnderFocus([memb("M", 0, 0, 10, 10)], "FANTOME")).toBeNull();
  });
});

describe("flèches sous focus", () => {
  const FB = { x: 0, y: 0, width: 400, height: 400 };
  const visible = new Set(["A", "B"]);

  const arrow = (over: Partial<Annotation> = {}): Annotation => ({
    id: "fl", type: "arrow", x: 10, y: 10, x2: 90, y2: 90, ...over,
  } as Annotation);

  it("hors focus, toutes les flèches passent", () => {
    expect(arrowVisibleUnderFocus(arrow(), null, null)).toBe(true);
  });

  it("attachée : visible seulement si TOUS ses nœuds le sont", () => {
    expect(arrowVisibleUnderFocus(arrow({ sourceId: "A", targetId: "B" }), visible, FB)).toBe(true);
    expect(arrowVisibleUnderFocus(arrow({ sourceId: "A", targetId: "Z" }), visible, FB)).toBe(false);
  });

  it("libre : visible si ses deux extrémités sont dans le cadre", () => {
    expect(arrowVisibleUnderFocus(arrow(), visible, FB)).toBe(true);
    expect(arrowVisibleUnderFocus(arrow({ x2: 9000, y2: 9000 }), visible, FB)).toBe(false);
  });

  it("un élément qui n'est pas une flèche n'est jamais filtré ici", () => {
    const t = { id: "T", type: "text", x: 0, y: 0, text: "x" } as Annotation;
    expect(arrowVisibleUnderFocus(t, visible, FB)).toBe(true);
  });
});

describe("invariants de réglage", () => {
  it("l'animation de cadrage tient DANS le temps mort", () => {
    // Sinon une décision pourrait tomber au milieu du glissement de caméra, et
    // le cadrage provoquerait sa propre annulation.
    expect(FOCUS.FIT_ANIM_MS).toBeLessThan(FOCUS.COOLDOWN_MS);
  });

  it("le seuil de sortie est bien en dessous du seuil d'entrée", () => {
    expect(FOCUS.EXIT_SCALE_RATIO).toBeLessThan(1);
    expect(FOCUS.ENTER_COVERAGE).toBeGreaterThan(0.5);
  });
});

describe("focusFrameOf", () => {
  it("rend le cadre de la membrane focalisée, contenu compris", () => {
    const items = [memb("M", 0, 0, 100, 100, "minimized"), box("I", 0, 0, 800, 600, "M")];
    expect(focusFrameOf(items, "M")).toEqual({ x: 0, y: 0, width: 800, height: 600 });
  });

  it("null hors focus ou sur une membrane inconnue", () => {
    expect(focusFrameOf([memb("M", 0, 0, 10, 10)], null)).toBeNull();
    expect(focusFrameOf([memb("M", 0, 0, 10, 10)], "FANTOME")).toBeNull();
  });
});

describe("annotationVisibleUnderFocus", () => {
  const FB = { x: 0, y: 0, width: 400, height: 400 };
  const visible = new Set(["dedans"]);

  it("hors focus, tout passe", () => {
    const t = { id: "x", type: "text", x: 0, y: 0, text: "a", width: 10, height: 10 } as Annotation;
    expect(annotationVisibleUnderFocus(t, null, null)).toBe(true);
  });

  it("un membre passe, un étranger mesuré est masqué", () => {
    const dedans = { id: "dedans", type: "text", x: 0, y: 0, text: "a", width: 10, height: 10 } as Annotation;
    const dehors = { id: "dehors", type: "text", x: 0, y: 0, text: "a", width: 10, height: 10 } as Annotation;
    expect(annotationVisibleUnderFocus(dedans, visible, FB)).toBe(true);
    expect(annotationVisibleUnderFocus(dehors, visible, FB)).toBe(false);
  });

  it("un bloc PAS ENCORE MESURÉ ne clignote pas : on juge son ancrage", () => {
    // Sans ça, chaque création de texte le ferait disparaître le temps d'une image.
    const neuf = { id: "neuf", type: "text", x: 50, y: 50, text: "" } as Annotation;
    const loin = { id: "loin", type: "text", x: 5000, y: 50, text: "" } as Annotation;
    expect(annotationVisibleUnderFocus(neuf, visible, FB)).toBe(true);
    expect(annotationVisibleUnderFocus(loin, visible, FB)).toBe(false);
  });

  it("délègue aux règles des flèches", () => {
    const fl = { id: "f", type: "arrow", x: 10, y: 10, x2: 9000, y2: 9000 } as Annotation;
    expect(annotationVisibleUnderFocus(fl, visible, FB)).toBe(false);
  });
});

describe("fond de scène", () => {
  it("teinte le fond du canvas avec la couleur de la membrane", () => {
    const bg = focusBackground("#60a5fa");
    expect(bg).toMatch(/^#[0-9a-f]{6}$/);
    // Teinte, pas aplat : on reste beaucoup plus proche du noir que du bleu.
    expect(bg).not.toBe("#60a5fa");
    const r = parseInt(bg.slice(1, 3), 16);
    expect(r).toBeLessThan(0x60);
  });

  it("un mélange à 100 % rend exactement la couleur demandée", () => {
    expect(focusBackground("#ffffff", "#000000", 1)).toBe("#ffffff");
    expect(focusBackground("#abc", "#000000", 1)).toBe("#aabbcc");
  });

  it("hors focus, ou couleur illisible, on garde le fond habituel", () => {
    expect(focusBackground(null)).toBe("#0d0d0d");
    expect(focusBackground("rebeccapurple")).toBe("#0d0d0d");
    expect(focusBackground("")).toBe("#0d0d0d");
  });
});
