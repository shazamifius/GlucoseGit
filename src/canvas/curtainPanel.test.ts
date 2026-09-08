import { describe, expect, it } from "vitest";
import {
  CURTAIN,
  advance,
  canvasStrip,
  decide,
  initialState,
  isOverPanel,
  normalizeConfig,
  panelRect,
  step,
  type CurtainConfig,
  type CurtainState,
} from "./curtainPanel";

const SCREEN = { width: 1000, height: 800 };
const CFG = normalizeConfig();

/** Position en x correspondant à une fraction de l'écran. */
const at = (frac: number) => SCREEN.width * frac;

/** Fait tourner la boucle jusqu'à stabilisation, en relevant tout ce qui bouge. */
function run(
  state: CurtainState,
  cursorX: number | null,
  frames: number,
  cfg: CurtainConfig = CFG,
  dt = 16,
) {
  const ratios: number[] = [state.ratio];
  const phases: string[] = [state.phase];
  let s = state;
  for (let i = 1; i <= frames; i++) {
    s = step(s, cursorX, SCREEN, cfg, i * dt, dt);
    ratios.push(s.ratio);
    phases.push(s.phase);
  }
  return { state: s, ratios, phases, flips: phases.filter((p, i) => i > 0 && p !== phases[i - 1]).length };
}

// ── Réglages ────────────────────────────────────────────────────────────────

describe("réglages — les deux surfaces restent toujours visibles", () => {
  it("valeurs par défaut : une languette à droite, le canvas à gauche", () => {
    expect(CFG.collapsed).toBe(0.1);
    expect(CFG.expanded).toBe(0.9);
  });

  it("INVARIANT : 0 < replié < déployé < 1, quoi qu'on donne", () => {
    const cas: Array<Partial<CurtainConfig> | null | undefined> = [
      null, undefined, {}, { collapsed: 0 }, { collapsed: -5 }, { expanded: 1 },
      { expanded: 42 }, { collapsed: 0.9, expanded: 0.1 }, { collapsed: 0.5, expanded: 0.5 },
      { collapsed: Number.NaN, expanded: Number.POSITIVE_INFINITY },
    ];
    for (const c of cas) {
      const n = normalizeConfig(c);
      expect(n.collapsed).toBeGreaterThan(0);
      expect(n.expanded).toBeLessThan(1);
      expect(n.expanded).toBeGreaterThan(n.collapsed);
    }
  });

  it("un réglage personnalisé (5/10) est respecté", () => {
    const n = normalizeConfig({ collapsed: 0.15, expanded: 0.5 });
    expect(n).toEqual({ collapsed: 0.15, expanded: 0.5 });
  });

  it("un déployé plus petit que le replié est corrigé, pas accepté", () => {
    const n = normalizeConfig({ collapsed: 0.3, expanded: 0.1 });
    expect(n.expanded).toBeGreaterThanOrEqual(n.collapsed + CURTAIN.MIN_SPAN);
  });
});

describe("géométrie", () => {
  it("le panneau est ancré au bord droit", () => {
    expect(panelRect(0.1, SCREEN)).toEqual({ x: 900, y: 0, width: 100, height: 800 });
    expect(panelRect(0.9, SCREEN)).toEqual({ x: 100, y: 0, width: 900, height: 800 });
  });

  it("la bande de canvas est le complément EXACT du panneau — aucun liseré", () => {
    // En virgule flottante, width × (1 − ratio) ne rejoint pas toujours le bord
    // du panneau : il resterait une raie d'un pixel entre les deux surfaces.
    for (const r of [0.1, 0.9, 0.333, CFG.expanded, CFG.collapsed]) {
      expect(canvasStrip(r, SCREEN).width + panelRect(r, SCREEN).width).toBe(SCREEN.width);
      expect(canvasStrip(r, SCREEN).width).toBe(panelRect(r, SCREEN).x);
    }
    expect(canvasStrip(0.9, SCREEN).width).toBe(100);
    expect(canvasStrip(CFG.expanded, SCREEN).width).toBeGreaterThan(0);
  });

  it("le survol se décide sur la frontière courante", () => {
    expect(isOverPanel(at(0.95), 0.1, SCREEN)).toBe(true);
    expect(isOverPanel(at(0.5), 0.1, SCREEN)).toBe(false);
    expect(isOverPanel(at(0.5), 0.9, SCREEN)).toBe(true);
  });

  it("souris hors fenêtre = jamais sur le panneau", () => {
    expect(isOverPanel(null, 0.9, SCREEN)).toBe(false);
  });
});

// ── L'auto-stabilisation, démontrée ─────────────────────────────────────────

describe("le survol ne peut pas battre — c'est géométrique", () => {
  it("PREUVE : en se déployant, la frontière FUIT un curseur posé sur le panneau", () => {
    // Le curseur qui déclenche le déploiement est à droite de la frontière.
    // Le déploiement pousse la frontière vers la gauche : il y reste, à chaque
    // position intermédiaire. La condition qui a déclenché le geste est donc
    // renforcée par le geste lui-même.
    const cursor = at(0.95);
    for (let r = CFG.collapsed; r <= CFG.expanded; r += 0.01) {
      expect(isOverPanel(cursor, r, SCREEN)).toBe(true);
    }
  });

  it("PREUVE : en se repliant, la frontière FUIT un curseur posé sur le canvas", () => {
    const cursor = at(0.05);
    for (let r = CFG.expanded; r >= CFG.collapsed; r -= 0.01) {
      expect(isOverPanel(cursor, r, SCREEN)).toBe(false);
    }
  });

  it("SIMULATION : curseur immobile sur la languette → une seule bascule", () => {
    const r = run(initialState(CFG), at(0.95), 200);
    expect(r.flips).toBe(1);
    expect(r.state.phase).toBe("expanded");
    expect(r.state.ratio).toBeCloseTo(CFG.expanded, 6);
  });

  it("SIMULATION : curseur immobile sur le canvas, rideau ouvert → une seule bascule", () => {
    const ouvert: CurtainState = { ratio: CFG.expanded, phase: "expanded", pending: null, pendingSince: 0 };
    const r = run(ouvert, at(0.05), 200);
    expect(r.flips).toBe(1);
    expect(r.state.phase).toBe("collapsed");
    expect(r.state.ratio).toBeCloseTo(CFG.collapsed, 6);
  });

  it("le glissement est MONOTONE : jamais de retour en arrière ni de dépassement", () => {
    const r = run(initialState(CFG), at(0.95), 200);
    for (let i = 1; i < r.ratios.length; i++) {
      expect(r.ratios[i]).toBeGreaterThanOrEqual(r.ratios[i - 1]);
      expect(r.ratios[i]).toBeLessThanOrEqual(CFG.expanded + 1e-9);
    }
  });

  it("SIMULATION : curseur pile sur la frontière de repos → il commit, il ne vibre pas", () => {
    // Cas limite le plus hostile : le curseur est exactement sur le bord.
    const r = run(initialState(CFG), at(1 - CFG.collapsed), 200);
    expect(r.flips).toBe(1);
    expect(r.state.phase).toBe("expanded");
  });
});

// ── Temporisation ───────────────────────────────────────────────────────────

describe("temporisation — frôler le bord droit n'ouvre pas le rideau", () => {
  it("un passage plus court que la temporisation ne déploie rien", () => {
    let s = initialState(CFG);
    // 80 ms sur la languette (< 90), puis on repart vers le canvas.
    s = decide(s, at(0.95), SCREEN, 0);
    s = decide(s, at(0.95), SCREEN, 80);
    expect(s.phase).toBe("collapsed");
    s = decide(s, at(0.2), SCREEN, 90);
    expect(s.phase).toBe("collapsed");
    expect(s.pending).toBeNull();
  });

  it("rester assez longtemps déploie", () => {
    let s = initialState(CFG);
    s = decide(s, at(0.95), SCREEN, 0);
    s = decide(s, at(0.95), SCREEN, CURTAIN.EXPAND_DWELL_MS + 1);
    expect(s.phase).toBe("expanded");
  });

  it("le repli est plus vif que le déploiement", () => {
    expect(CURTAIN.COLLAPSE_DWELL_MS).toBeLessThan(CURTAIN.EXPAND_DWELL_MS);
  });

  it("revenir sur ses pas annule la temporisation en cours", () => {
    let s = initialState(CFG);
    s = decide(s, at(0.95), SCREEN, 0);
    expect(s.pending).toBe("expanded");
    s = decide(s, at(0.2), SCREEN, 10);
    expect(s.pending).toBeNull();
    expect(s.phase).toBe("collapsed");
  });
});

// ── Sortie de fenêtre ───────────────────────────────────────────────────────

describe("souris hors fenêtre", () => {
  it("le rideau se replie plutôt que de rester ouvert dans le vide", () => {
    const ouvert: CurtainState = { ratio: CFG.expanded, phase: "expanded", pending: null, pendingSince: 0 };
    const r = run(ouvert, null, 200);
    expect(r.state.phase).toBe("collapsed");
    expect(r.state.ratio).toBeCloseTo(CFG.collapsed, 6);
  });
});

// ── Animation ───────────────────────────────────────────────────────────────

describe("glissement", () => {
  it("indépendant de la cadence : 1 pas de 160 ms ≈ 10 pas de 16 ms", () => {
    const base: CurtainState = { ratio: CFG.collapsed, phase: "expanded", pending: null, pendingSince: 0 };
    const gros = advance(base, CFG, 160);
    let fin = base;
    for (let i = 0; i < 10; i++) fin = advance(fin, CFG, 16);
    expect(gros.ratio).toBeCloseTo(fin.ratio, 6);
  });

  it("atteint sa cible et s'y colle", () => {
    let s: CurtainState = { ratio: CFG.collapsed, phase: "expanded", pending: null, pendingSince: 0 };
    for (let i = 0; i < 100; i++) s = advance(s, CFG, 16);
    expect(s.ratio).toBe(CFG.expanded);
    // Une fois collé, on n'alloue plus d'état inutile.
    expect(advance(s, CFG, 16)).toBe(s);
  });

  it("un dt nul ou négatif ne fait rien bouger", () => {
    const s: CurtainState = { ratio: 0.5, phase: "expanded", pending: null, pendingSince: 0 };
    expect(advance(s, CFG, 0).ratio).toBe(0.5);
    expect(advance(s, CFG, -100).ratio).toBe(0.5);
  });
});
