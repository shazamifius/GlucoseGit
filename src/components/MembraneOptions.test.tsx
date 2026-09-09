// ────────────────────────────────────────────────────────────────────────────
// MEMB-3 — Barre de modes : vérification de bout en bout.
//
// On monte le VRAI panneau, on clique un mode, et on relit le STORE. Ce qui
// compte ici n'est pas l'affichage mais deux règles qui doivent être
// inatteignables autrement : l'aller sans retour vers classique, et
// l'instantané d'appartenance pris au moment exact de la conversion.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { getActiveBoard, useGlucoseStore } from "../store";
import type { Annotation, BoardImage, MembraneAnnotation } from "../types";
import { _resetLocalUserCache, getLocalUser } from "../multiplayer/localUser";
import MembraneOptions from "./MembraneOptions";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: (s: string) => s,
}));

const MEMB_ID = "M1";

function membrane(over: Partial<MembraneAnnotation> = {}): Annotation {
  return {
    id: MEMB_ID, type: "membrane", x: 0, y: 0, width: 400, height: 400,
    color: "#60a5fa", ...over,
  } as Annotation;
}

function texte(id: string, x: number, y: number, membraneId?: string): Annotation {
  return { id, type: "text", x, y, text: id, width: 40, height: 40, membraneId } as Annotation;
}

function image(id: string, x: number, y: number): BoardImage {
  return {
    id, x, y, width: 40, height: 40, rotation: 0, locked: false, tags: [],
    originalWidth: 40, originalHeight: 40,
  };
}

function seed(annotations: Annotation[], images: BoardImage[] = []) {
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
  for (const i of images) useGlucoseStore.getState().addImage("main", i);
}

function board() {
  return getActiveBoard(useGlucoseStore.getState().project);
}

function memb(): MembraneAnnotation {
  return board().annotations.find((a) => a.id === MEMB_ID) as MembraneAnnotation;
}

function monter() {
  return render(<MembraneOptions membrane={memb()} />);
}

beforeEach(() => {
  localStorage.clear();
  _resetLocalUserCache();
  seed([membrane()]);
});
afterEach(cleanup);

// ── Les trois modes ─────────────────────────────────────────────────────────

describe("choix du mode", () => {
  it("classique est l'état de départ", () => {
    monter();
    expect(screen.getByText("Classique")).toHaveAttribute("aria-pressed", "true");
  });

  it("passer en minimisée écrit le mode", () => {
    monter();
    fireEvent.click(screen.getByText("Minimisée"));
    expect(memb().mode).toBe("minimized");
  });

  it("passer en étirée aussi", () => {
    monter();
    fireEvent.click(screen.getByText("Étirée"));
    expect(memb().mode).toBe("stretched");
  });

  it("minimisée ↔ étirée reste permis dans les deux sens", () => {
    seed([membrane({ mode: "minimized" })]);
    monter();
    fireEvent.click(screen.getByText("Étirée"));
    expect(memb().mode).toBe("stretched");
  });
});

// ── L'aller sans retour ─────────────────────────────────────────────────────

describe("une membrane spéciale ne redevient jamais classique", () => {
  it("le bouton est désactivé, pas caché", () => {
    // Le cacher laisserait croire à un oubli ; désactivé, c'est visiblement
    // une décision.
    seed([membrane({ mode: "minimized" })]);
    monter();
    expect(screen.getByText("Classique")).toBeDisabled();
  });

  it("et cliquer dessus ne fait rien", () => {
    seed([membrane({ mode: "stretched" })]);
    monter();
    fireEvent.click(screen.getByText("Classique"));
    expect(memb().mode).toBe("stretched");
  });
});

// ── L'instantané d'appartenance ─────────────────────────────────────────────

describe("la conversion prend un instantané de ce qu'elle contient", () => {
  it("les éléments géométriquement dedans deviennent membres", () => {
    // Au moment de la conversion l'échelle vaut encore 1 : c'est la seule
    // occasion de déduire l'appartenance de la géométrie sans se tromper.
    seed([membrane(), texte("dedans", 100, 100)], [image("img", 200, 200)]);
    monter();
    fireEvent.click(screen.getByText("Minimisée"));

    const b = board();
    expect(b.annotations.find((a) => a.id === "dedans")!.membraneId).toBe(MEMB_ID);
    expect(b.images.find((i) => i.id === "img")!.membraneId).toBe(MEMB_ID);
  });

  it("ce qui est dehors reste libre", () => {
    seed([membrane(), texte("dehors", 5000, 5000)]);
    monter();
    fireEvent.click(screen.getByText("Minimisée"));
    expect(board().annotations.find((a) => a.id === "dehors")!.membraneId).toBeUndefined();
  });

  it("on ne VOLE pas le contenu d'une autre membrane", () => {
    const autre = { ...membrane({ id: "AUTRE" }) } as Annotation;
    seed([membrane(), autre, texte("aAutrui", 100, 100, "AUTRE")]);
    monter();
    fireEvent.click(screen.getByText("Minimisée"));
    expect(board().annotations.find((a) => a.id === "aAutrui")!.membraneId).toBe("AUTRE");
  });

  it("l'instantané n'est pris qu'en QUITTANT classique, pas à chaque bascule", () => {
    // Déjà spéciale : son contenu est fixé, une bascule minimisée→étirée ne doit
    // pas ratisser ce qui se trouve géométriquement dessous entre-temps.
    seed([membrane({ mode: "minimized" }), texte("passant", 100, 100)]);
    monter();
    fireEvent.click(screen.getByText("Étirée"));
    expect(board().annotations.find((a) => a.id === "passant")!.membraneId).toBeUndefined();
  });

  it("la membrane ne se prend jamais elle-même", () => {
    seed([membrane()]);
    monter();
    fireEvent.click(screen.getByText("Minimisée"));
    expect(memb().membraneId).toBeUndefined();
  });
});

describe("compte des membres", () => {
  it("annonce le vide", () => {
    monter();
    expect(screen.getByText("vide")).toBeTruthy();
  });

  it("compte textes et images ensemble", () => {
    seed([membrane(), texte("a", 10, 10, MEMB_ID)], [{ ...image("b", 20, 20), membraneId: MEMB_ID }]);
    monter();
    expect(screen.getByText("2 éléments")).toBeTruthy();
  });
});

// ── Le rideau se crée ICI ───────────────────────────────────────────────────
//
// MEMB-9 — Le bouton vivait sur la languette, au bord droit, donc atteignable
// uniquement en mode focus — alors que créer un rideau n'a rien à voir avec le
// fait d'en regarder un. Les trois choses qu'on décide d'une membrane sont
// désormais au même endroit.

describe("création d'un rideau", () => {
  it("le bouton écrit un rideau PRIVÉ dans le store, à mon nom", () => {
    const moi = getLocalUser();
    monter();
    fireEvent.click(screen.getByText("Rideau"));

    const cs = memb().curtains ?? [];
    expect(cs).toHaveLength(1);
    expect(cs[0].ownerId).toBe(moi.id);
    expect(cs[0].ownerName).toBe(moi.name);
    expect(cs[0].visibility).toBe("private");
  });

  it("le rideau naît avec son board — séparés, un Ctrl+Z laisserait une coquille", () => {
    monter();
    fireEvent.click(screen.getByText("Rideau"));

    const c = (memb().curtains ?? [])[0];
    expect(c.boardId).toBeTruthy();
    expect(useGlucoseStore.getState().project.boards.some((b) => b.id === c.boardId)).toBe(true);
  });

  it("un seul rideau par personne : le bouton se DÉSACTIVE, il ne disparaît pas", () => {
    // Comme pour les modes : une impossibilité se montre, elle ne s'escamote
    // pas — un bouton disparu se lit comme un oubli, pas comme une décision.
    monter();
    fireEvent.click(screen.getByText("Rideau"));
    cleanup();
    monter();

    const b = screen.getByText("Rideau");
    expect(b).toBeTruthy();
    expect(b).toBeDisabled();

    fireEvent.click(b);
    expect(memb().curtains ?? []).toHaveLength(1);
  });

  it("le rideau d'un autre ne m'empêche pas de créer le mien", () => {
    useGlucoseStore.getState().updateAnnotation("main", MEMB_ID, {
      curtains: [{
        id: "c-autre", ownerId: "u-autre", ownerName: "Grace", ownerColor: "#34d399",
        visibility: "private", editable: "owner", notes: [], createdAt: 1,
      }],
    } as Partial<Annotation>);
    monter();

    expect(screen.getByText("Rideau")).not.toBeDisabled();
    fireEvent.click(screen.getByText("Rideau"));
    expect(memb().curtains ?? []).toHaveLength(2);
  });
});
