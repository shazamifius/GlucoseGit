import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  MAX_NAME_LENGTH,
  USER_CHANGED_EVENT,
  USER_COLORS,
  _resetLocalUserCache,
  getLocalUser,
  randomColor,
  randomName,
  sanitizeColor,
  sanitizeName,
  setLocalUser,
} from "./localUser";

const KEY = "glucose:local-user";

beforeEach(() => {
  localStorage.clear();
  sessionStorage.clear();
  _resetLocalUserCache();
});

afterEach(() => {
  vi.restoreAllMocks();
});

// ── Nettoyage des entrées ───────────────────────────────────────────────────

describe("nom", () => {
  it("supprime les espaces superflus et les retours à la ligne", () => {
    expect(sanitizeName("  Marie   Curie \n")).toBe("Marie Curie");
    expect(sanitizeName("a\r\nb")).toBe("a b");
  });

  it("borne la longueur — un nom fleuve masquerait le canvas des autres", () => {
    expect(sanitizeName("x".repeat(200))).toHaveLength(MAX_NAME_LENGTH);
  });

  it("un nom vide retombe sur un tirage, jamais sur un curseur anonyme", () => {
    expect(sanitizeName("   ")).not.toBe("");
    expect(sanitizeName("")).toMatch(/\S/);
  });

  it("garde les accents et la ponctuation", () => {
    expect(sanitizeName("Zoé-Ana")).toBe("Zoé-Ana");
  });
});

describe("couleur", () => {
  it("accepte une couleur hexadécimale et la normalise", () => {
    expect(sanitizeColor("#AABBCC")).toBe("#aabbcc");
    expect(sanitizeColor("  #38bdf8 ")).toBe("#38bdf8");
  });

  it("développe la forme courte", () => {
    expect(sanitizeColor("#abc")).toBe("#aabbcc");
  });

  it("toute couleur illisible retombe sur la palette", () => {
    for (const mauvais of ["", "rouge", "#12", "rgb(1,2,3)", "#gggggg"]) {
      expect(USER_COLORS).toContain(sanitizeColor(mauvais) as (typeof USER_COLORS)[number]);
    }
  });

  it("la palette est distincte et bien formée", () => {
    expect(new Set(USER_COLORS).size).toBe(USER_COLORS.length);
    for (const c of USER_COLORS) expect(c).toMatch(/^#[0-9a-f]{6}$/);
  });

  it("les tirages restent dans le domaine valide", () => {
    for (let i = 0; i < 50; i++) {
      expect(USER_COLORS).toContain(randomColor() as (typeof USER_COLORS)[number]);
      expect(randomName()).toMatch(/\S/);
    }
  });
});

// ── Persistance ─────────────────────────────────────────────────────────────

describe("persistance — un nom choisi ne s'évapore pas", () => {
  it("crée une identité au premier appel et l'écrit sur le disque", () => {
    const u = getLocalUser();
    expect(u.name).toMatch(/\S/);
    expect(JSON.parse(localStorage.getItem(KEY)!)).toEqual(u);
  });

  it("la relit d'une session à l'autre", () => {
    const u = setLocalUser({ name: "Ada", color: "#34d399" });
    _resetLocalUserCache(); // simule une réouverture de l'application
    expect(getLocalUser()).toEqual(u);
  });

  it("REGRESSION : reprend l'ancien emplacement de session", () => {
    // Avant, l'identité vivait dans sessionStorage. Mettre à jour l'application
    // en pleine session ne doit pas renommer l'utilisateur sous ses yeux.
    sessionStorage.setItem(KEY, JSON.stringify({ name: "Historique", color: "#f87171" }));
    expect(getLocalUser()).toEqual({ name: "Historique", color: "#f87171" });
  });

  it("un stockage abîmé ne casse pas la collaboration", () => {
    localStorage.setItem(KEY, "{ pas du json");
    expect(getLocalUser().name).toMatch(/\S/);
  });

  it("un stockage incomplet est complété, pas propagé", () => {
    localStorage.setItem(KEY, JSON.stringify({ name: "Sans couleur" }));
    const u = getLocalUser();
    expect(u.name).toMatch(/\S/);
    expect(u.color).toMatch(/^#[0-9a-f]{6}$/);
  });

  it("un stockage indisponible (navigation privée) ne fait pas échouer l'app", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("refusé"); });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("refusé"); });
    expect(() => getLocalUser()).not.toThrow();
    expect(getLocalUser().name).toMatch(/\S/);
  });
});

// ── Modification ────────────────────────────────────────────────────────────

describe("modification", () => {
  it("change le nom sans toucher à la couleur, et l'inverse", () => {
    const base = setLocalUser({ name: "Ada", color: "#34d399" });
    expect(setLocalUser({ name: "Grace" })).toEqual({ name: "Grace", color: base.color });
    expect(setLocalUser({ color: "#fb923c" })).toEqual({ name: "Grace", color: "#fb923c" });
  });

  it("nettoie ce qu'on lui donne", () => {
    expect(setLocalUser({ name: "  Ada  Lovelace ", color: "#ABC" }))
      .toEqual({ name: "Ada Lovelace", color: "#aabbcc" });
  });

  it("PRÉVIENT l'application — sinon les pairs gardent l'ancien nom affiché", () => {
    const vu: unknown[] = [];
    const h = (e: Event) => vu.push((e as CustomEvent).detail);
    window.addEventListener(USER_CHANGED_EVENT, h);
    setLocalUser({ name: "Ada" });
    window.removeEventListener(USER_CHANGED_EVENT, h);
    expect(vu).toEqual([{ name: "Ada", color: expect.any(String) }]);
  });

  it("ne prévient PAS quand rien ne change — pas de rediffusion inutile", () => {
    const u = setLocalUser({ name: "Ada", color: "#34d399" });
    let appels = 0;
    const h = () => { appels++; };
    window.addEventListener(USER_CHANGED_EVENT, h);
    expect(setLocalUser({ name: u.name, color: u.color })).toBe(u);
    expect(setLocalUser({})).toBe(u);
    window.removeEventListener(USER_CHANGED_EVENT, h);
    expect(appels).toBe(0);
  });
});
