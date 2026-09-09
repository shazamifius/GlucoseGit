// ────────────────────────────────────────────────────────────────────────────
// MEMB-4 — Le mode étiré, de bout en bout.
//
// `membraneStretch` démontre le calcul sur des boîtes. Ici on monte la VRAIE
// barre de modes, on clique, et on relit le STORE — parce que c'est là que les
// choses non démontrables se cassent : la taille n'est pas écrite, elle l'est
// hors de la transaction d'annulation, ou l'avertissement ne part jamais.
//
// Trois choses ne peuvent se vérifier QU'ICI :
//   ① la conversion écrit vraiment la nouvelle taille dans le document ;
//   ② conversion + croissance forment UNE SEULE entrée d'annulation ;
//   ③ l'avertissement se lève, montre l'obstacle, et sait y emmener.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { getActiveBoard, useGlucoseStore } from "../store";
import type { Annotation, MembraneAnnotation } from "../types";
import { MEMBRANE_SPACE } from "./membraneSpace";
import MembraneOptions from "../components/MembraneOptions";
import MembraneStretchAlert, { type StretchAlertData } from "./MembraneStretchAlert";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: (s: string) => s,
}));

const PAD = MEMBRANE_SPACE.STRETCH_PADDING;
const MEMB_ID = "M1";

function membrane(over: Partial<MembraneAnnotation> = {}): Annotation {
  return {
    id: MEMB_ID, type: "membrane", x: 0, y: 0, width: 200, height: 200,
    color: "#60a5fa", ...over,
  } as Annotation;
}

function texte(id: string, x: number, y: number, w: number, h: number): Annotation {
  return { id, type: "text", x, y, text: id, width: w, height: h } as Annotation;
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

const board = () => getActiveBoard(useGlucoseStore.getState().project);
const memb = () => board().annotations.find((a) => a.id === MEMB_ID) as MembraneAnnotation;

/** Écoute l'avertissement émis par la conversion. */
function ecouterAlerte() {
  const recu: (StretchAlertData | null)[] = [];
  const h = (e: Event) => recu.push((e as CustomEvent<StretchAlertData | null>).detail);
  window.addEventListener("glucose:stretch-blocked", h);
  return { recu, stop: () => window.removeEventListener("glucose:stretch-blocked", h) };
}

afterEach(cleanup);

// ── ① La croissance atteint vraiment le document ────────────────────────────

describe("passer en étirée fait grandir la membrane pour de vrai", () => {
  beforeEach(() => {
    // Contenu qui déborde de 200 à droite. Membre AVANT la conversion : ici on
    // teste la croissance, pas l'instantané d'appartenance (déjà couvert).
    seed([membrane(), { ...texte("I", 0, 0, 300, 100), membraneId: MEMB_ID } as Annotation]);
  });

  it("la nouvelle largeur est écrite dans le store", () => {
    render(<MembraneOptions membrane={memb()} />);
    fireEvent.click(screen.getByText("Étirée"));
    expect(memb().mode).toBe("stretched");
    expect(memb().width).toBe(300 + PAD);
  });

  it("l'axe qui tenait déjà ne bouge pas", () => {
    render(<MembraneOptions membrane={memb()} />);
    fireEvent.click(screen.getByText("Étirée"));
    expect(memb().height).toBe(200);
  });

  it("et rien ne s'annonce bloqué quand la voie est libre", () => {
    const { recu, stop } = ecouterAlerte();
    render(<MembraneOptions membrane={memb()} />);
    fireEvent.click(screen.getByText("Étirée"));
    stop();
    expect(recu).toEqual([null]);
  });
});

// ── ② Une seule entrée d'annulation ─────────────────────────────────────────

describe("conversion et croissance s'annulent ENSEMBLE", () => {
  it("un seul Ctrl+Z rend le mode ET la taille d'avant", () => {
    // Séparées, l'annulation laisserait une membrane classique restée grande —
    // une taille que personne n'a choisie.
    seed([membrane(), { ...texte("I", 0, 0, 300, 100), membraneId: MEMB_ID } as Annotation]);
    render(<MembraneOptions membrane={memb()} />);
    fireEvent.click(screen.getByText("Étirée"));
    expect(memb().width).toBe(300 + PAD);

    expect(useGlucoseStore.getState().undo()).toBe(true);
    expect(memb().mode ?? "classic").toBe("classic");
    expect(memb().width).toBe(200);
  });
});

// ── ③ L'obstacle, et le chemin vers lui ─────────────────────────────────────

describe("un élément étranger sur le chemin arrête la croissance et se signale", () => {
  beforeEach(() => {
    seed([
      membrane(),
      { ...texte("I", 0, 0, 300, 100), membraneId: MEMB_ID } as Annotation,
      texte("ETR", 250, 0, 50, 50), // libre : il n'appartient à personne
    ]);
  });

  it("la membrane s'arrête AU CONTACT, sans recouvrir ni capturer", () => {
    render(<MembraneOptions membrane={memb()} />);
    fireEvent.click(screen.getByText("Étirée"));
    expect(memb().width).toBe(250);
    // L'intrus n'a pas été aspiré au passage.
    expect(board().annotations.find((a) => a.id === "ETR")!.membraneId).toBeUndefined();
  });

  it("l'avertissement nomme l'élément fautif", () => {
    const { recu, stop } = ecouterAlerte();
    render(<MembraneOptions membrane={memb()} />);
    fireEvent.click(screen.getByText("Étirée"));
    stop();
    expect(recu).toHaveLength(1);
    expect(recu[0]).toEqual({ membraneIds: [MEMB_ID], blockerIds: ["ETR"] });
  });
});

// ── L'avertissement à l'écran ───────────────────────────────────────────────

describe("l'avertissement montre l'obstacle et sait y emmener", () => {
  const alerte: StretchAlertData = { membraneIds: [MEMB_ID], blockerIds: ["ETR"] };
  const vpRef = () => ({ current: { x: 0, y: 0, scale: 1 } });

  beforeEach(() => {
    seed([membrane({ mode: "stretched", width: 250 }), texte("ETR", 250, 0, 50, 50)]);
  });

  function monter(a: StretchAlertData | null, onDismiss = vi.fn()) {
    return render(
      <MembraneStretchAlert alert={a} boardId="main" vpRef={vpRef()} onDismiss={onDismiss} />,
    );
  }

  it("il entoure le bloqueur, à sa position vue", () => {
    const { container } = monter(alerte);
    const rect = container.querySelector("rect")!;
    // Contour = boîte du bloqueur, élargie de la marge de part et d'autre.
    expect(rect.getAttribute("x")).toBe("244");
    expect(rect.getAttribute("width")).toBe("62");
  });

  it("« Y aller » centre la vue sur le bloqueur", () => {
    const jumps: { wx: number; wy: number }[] = [];
    const h = (e: Event) => jumps.push((e as CustomEvent<{ wx: number; wy: number }>).detail);
    window.addEventListener("glucose:jump-viewport", h);
    monter(alerte);
    fireEvent.click(screen.getByText("Y aller"));
    window.removeEventListener("glucose:jump-viewport", h);
    expect(jumps).toEqual([{ wx: 275, wy: 25 }]);
  });

  it("il se ferme à la demande", () => {
    const onDismiss = vi.fn();
    monter(alerte, onDismiss);
    fireEvent.click(screen.getByLabelText("Fermer l'avertissement"));
    expect(onDismiss).toHaveBeenCalled();
  });

  it("un bloqueur disparu efface l'avertissement de lui-même", () => {
    // On ne fige jamais la géométrie du moment : elle mentirait dès le premier
    // déplacement, et survivrait à la suppression de l'élément.
    const { container } = monter({ membraneIds: [MEMB_ID], blockerIds: ["FANTOME"] });
    expect(container.querySelector("rect")).toBeNull();
    expect(screen.queryByText("Y aller")).toBeNull();
  });

  it("sans alerte, la couche ne rend rien du tout", () => {
    const { container } = monter(null);
    expect(container.innerHTML).toBe("");
  });
});
