// ────────────────────────────────────────────────────────────────────────────
// MEMB-3 — Vérification de bout en bout du rideau.
//
// `curtainPanel` et `curtainModel` sont testés à part. Ici on monte la VRAIE
// couche, on crée un rideau, on change les permissions, et on regarde le DOM et
// le store. Ce que le modèle interdit doit être inatteignable à l'écran, pas
// seulement faux en mémoire.
//
// MEMB-7 — le contenu d'un rideau n'est plus une liste de notes mais un BOARD.
// Les tests de saisie de note ont donc été remplacés par ce qui les rend
// caduques : le board arrive à l'ouverture, et les notes d'avant y atterrissent
// sous forme de blocs. Le reste — existence, permissions, languettes — est
// inchangé et doit le rester.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { Annotation, MembraneAnnotation, MembraneCurtain } from "../types";
import { useGlucoseStore } from "../store";
import { _resetLocalUserCache, getLocalUser, setLocalUser } from "../multiplayer/localUser";
import MembraneCurtainLayer from "./MembraneCurtainLayer";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: (s: string) => s,
}));

const BOARD = "main";

function membrane(curtains?: MembraneCurtain[]): MembraneAnnotation {
  return {
    id: "M1", type: "membrane", x: 0, y: 0, width: 800, height: 600,
    color: "#60a5fa", curtains,
  } as MembraneAnnotation;
}

/** Amorçage par les actions du store — `setState` seul serait écrasé par la
 *  prochaine mutation, qui repart du document et non de l'état React. */
function seed(curtains?: MembraneCurtain[]) {
  const st = useGlucoseStore.getState();
  st.loadProject({
    version: "2.0.0", name: "test",
    boards: [{
      id: BOARD, name: "Board principal",
      images: [], annotations: [], panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 },
      createdAt: 0, updatedAt: 0,
    }],
    activeBoardId: BOARD, presets: [], domains: [],
    createdAt: 0, updatedAt: 0,
  });
  useGlucoseStore.getState().addAnnotation(BOARD, membrane(curtains) as Annotation);
}

/** Les rideaux tels que le store les a réellement enregistrés. */
function stored(): MembraneCurtain[] {
  const b = useGlucoseStore.getState().project.boards.find((x) => x.id === BOARD)!;
  const m = b.annotations.find((a) => a.id === "M1") as MembraneAnnotation;
  return m.curtains ?? [];
}

function current(): MembraneAnnotation {
  const b = useGlucoseStore.getState().project.boards.find((x) => x.id === BOARD)!;
  return b.annotations.find((a) => a.id === "M1") as MembraneAnnotation;
}

function monter(curtains?: MembraneCurtain[]) {
  seed(curtains);
  const vue = render(<MembraneCurtainLayer membrane={current()} boardId={BOARD} />);
  return {
    ...vue,
    /**
     * Rejoue le rendu avec la membrane À JOUR.
     *
     * La couche reçoit sa membrane en PROP : dans l'application c'est
     * `GlucoseCanvas` qui la relit à chaque mutation, mais ici la prop est
     * figée au montage. Dès qu'un effet écrit dans le store — la création du
     * board du rideau, par exemple — il faut donc redonner la prop nous-mêmes
     * pour voir ce que l'utilisateur verrait.
     */
    refresh: () => vue.rerender(<MembraneCurtainLayer membrane={current()} boardId={BOARD} />),
  };
}

beforeEach(() => {
  localStorage.clear();
  sessionStorage.clear();
  _resetLocalUserCache();
});
afterEach(cleanup);

// ── Existence ───────────────────────────────────────────────────────────────

describe("le rideau n'existe que si on le crée", () => {
  it("hors focus, la couche ne rend rien du tout", () => {
    const { container } = render(<MembraneCurtainLayer membrane={null} boardId={BOARD} />);
    expect(container.firstChild).toBeNull();
  });

  it("sans rideau, une seule languette : celle qui en crée un", () => {
    monter();
    expect(screen.getByLabelText("Créer mon rideau personnel")).toBeTruthy();
    expect(screen.queryByLabelText(/^Rideau de /)).toBeNull();
  });

  it("créer écrit un rideau PRIVÉ dans le store, à mon nom", () => {
    const moi = getLocalUser();
    monter();
    fireEvent.click(screen.getByLabelText("Créer mon rideau personnel"));

    const cs = stored();
    expect(cs).toHaveLength(1);
    expect(cs[0].ownerId).toBe(moi.id);
    expect(cs[0].ownerName).toBe(moi.name);
    expect(cs[0].visibility).toBe("private");
  });
});

// ── Contenu : le rideau est un canvas ─────────────────────────────

describe("le contenu du rideau est un board", () => {
  function mienAvecNote(text = "Relire le chapitre II") {
    const moi = getLocalUser();
    const c: MembraneCurtain = {
      id: "c1", ownerId: moi.id, ownerName: moi.name, ownerColor: moi.color,
      visibility: "private", editable: "owner",
      notes: [{ id: "n1", text, createdAt: 1 }], createdAt: 1,
    };
    return c;
  }

  /** Le board du rideau, tel que le store l'a enregistré. */
  function boardDuRideau() {
    const id = stored()[0]?.boardId;
    return useGlucoseStore.getState().project.boards.find((b) => b.id === id);
  }

  it("le board arrive À L'OUVERTURE, pas au chargement du projet", () => {
    // Un rideau qu'on ne regarde jamais ne doit rien coûter.
    monter([mienAvecNote()]);
    expect(stored()[0].boardId).toBeTruthy();
    expect(boardDuRideau()).toBeDefined();
  });

  it("mes notes d'avant ne sont pas perdues : elles deviennent des blocs", () => {
    // C'est le passage de l'ancien modèle au nouveau, vérifié de bout en bout.
    monter([mienAvecNote()]);
    const b = boardDuRideau()!;
    expect(b.annotations).toHaveLength(1);
    expect((b.annotations[0] as { text: string }).text).toBe("Relire le chapitre II");
    // Et elles ne sont pas comptées deux fois.
    expect(stored()[0].notes).toEqual([]);
  });

  it("le texte repris s'affiche vraiment dans le panneau", () => {
    // Le board naît dans un effet ; l'application re-rend alors la couche avec
    // la membrane à jour, et c'est ce second rendu qui montre le canvas.
    const { refresh } = monter([mienAvecNote("Relire le chapitre II")]);
    refresh();
    expect(screen.getByText("Relire le chapitre II")).toBeTruthy();
  });

  it("c'est un board ORDINAIRE : les outils de Glucose y travaillent", () => {
    // Tout l'intérêt de la décision — aucun outil n'a été adapté.
    monter([mienAvecNote()]);
    const id = stored()[0].boardId!;
    useGlucoseStore.getState().addAnnotation(id, {
      id: "m-dedans", type: "membrane", x: 0, y: 0, width: 120, height: 120,
    } as Annotation);
    expect(boardDuRideau()!.annotations.map((a) => a.type)).toContain("membrane");
  });

  it("supprimer le rideau emporte son board — rien ne fuit", () => {
    monter([mienAvecNote()]);
    const id = stored()[0].boardId!;
    fireEvent.click(screen.getByTitle("Supprimer ce rideau"));
    expect(stored()).toHaveLength(0);
    expect(useGlucoseStore.getState().project.boards.find((b) => b.id === id)).toBeUndefined();
  });
});

// ── Permissions, vues depuis l'écran ────────────────────────────────────────

describe("permissions — ce que le modèle interdit est inatteignable", () => {
  const AUTRUI = { id: "u-autre", name: "Grace", color: "#34d399" };

  function curtainDe(owner: typeof AUTRUI, over: Partial<MembraneCurtain> = {}): MembraneCurtain {
    return {
      id: "c-autre", ownerId: owner.id, ownerName: owner.name, ownerColor: owner.color,
      visibility: "private", editable: "owner",
      notes: [{ id: "n", text: "secret", createdAt: 1 }], createdAt: 1, ...over,
    };
  }

  it("le CARNET d'un autre n'apparaît nulle part", () => {
    monter([curtainDe(AUTRUI)]);
    expect(screen.queryByLabelText("Rideau de Grace")).toBeNull();
    expect(screen.queryByText("secret")).toBeNull();
    // Et je peux quand même créer le mien.
    expect(screen.getByLabelText("Créer mon rideau personnel")).toBeTruthy();
  });

  it("la VITRINE d'un autre se voit mais ne s'édite pas", () => {
    const { refresh } = monter([curtainDe(AUTRUI, { visibility: "shared", editable: "owner" })]);
    refresh();
    expect(screen.getByLabelText("Rideau de Grace")).toBeTruthy();
    // Son contenu s'affiche — c'est une vitrine, elle se regarde.
    expect(screen.getByText("secret")).toBeTruthy();
    // Mais le canvas est monté en lecture seule, et le dit.
    expect(screen.getByText("Lecture seule")).toBeTruthy();
  });

  it("l'ATELIER d'un autre s'édite", () => {
    const { refresh } = monter([curtainDe(AUTRUI, { visibility: "shared", editable: "everyone" })]);
    refresh();
    expect(screen.getByText("secret")).toBeTruthy();
    expect(screen.queryByText("Lecture seule")).toBeNull();
  });

  it("les réglages d'un rideau ne s'offrent qu'à son propriétaire", () => {
    monter([curtainDe(AUTRUI, { visibility: "shared", editable: "everyone" })]);
    expect(screen.queryByTitle("Supprimer ce rideau")).toBeNull();
    expect(screen.queryByTitle("Élargir le rideau")).toBeNull();
  });
});

describe("mes réglages", () => {
  function mien(over: Partial<MembraneCurtain> = {}): MembraneCurtain {
    const moi = getLocalUser();
    return {
      id: "c1", ownerId: moi.id, ownerName: moi.name, ownerColor: moi.color,
      visibility: "private", editable: "owner", notes: [], createdAt: 1, ...over,
    };
  }

  it("basculer privé ↔ partagé", () => {
    monter([mien()]);
    fireEvent.click(screen.getByText("Privé"));
    expect(stored()[0].visibility).toBe("shared");
  });

  it("« qui peut modifier » est inopérant tant que le rideau est privé", () => {
    // Un rideau privé est réservé par construction : proposer le réglage sans
    // le désactiver laisserait croire à un état qui n'existe pas.
    monter([mien({ visibility: "private" })]);
    expect(screen.getByText("Moi seul")).toBeDisabled();
  });

  it("une fois partagé, il s'ouvre à tout le monde", () => {
    monter([mien({ visibility: "shared" })]);
    fireEvent.click(screen.getByText("Moi seul"));
    expect(stored()[0].editable).toBe("everyone");
  });

  it("élargir enregistre une proportion, bornée", () => {
    monter([mien()]);
    fireEvent.click(screen.getByTitle("Élargir le rideau"));
    const r = stored()[0].expandedRatio!;
    expect(r).toBeGreaterThan(0);
    expect(r).toBeLessThan(1);
  });

  it("supprimer retire le rideau", () => {
    monter([mien()]);
    fireEvent.click(screen.getByTitle("Supprimer ce rideau"));
    expect(stored()).toHaveLength(0);
  });
});

// ── Languettes ──────────────────────────────────────────────────────────────

describe("languettes — variante A", () => {
  it("une languette par rideau visible, à la couleur de son propriétaire", () => {
    const moi = getLocalUser();
    monter([
      { id: "a", ownerId: moi.id, ownerName: moi.name, ownerColor: moi.color,
        visibility: "private", editable: "owner", notes: [], createdAt: 1 },
      { id: "b", ownerId: "u2", ownerName: "Grace", ownerColor: "#34d399",
        visibility: "shared", editable: "owner", notes: [], createdAt: 2 },
    ]);
    expect(screen.getByLabelText(`Rideau de ${moi.name}`)).toBeTruthy();
    expect(screen.getByLabelText("Rideau de Grace")).toBeTruthy();
  });

  it("survoler une languette change de rideau, sans clic", () => {
    const moi = getLocalUser();
    monter([
      { id: "a", ownerId: moi.id, ownerName: moi.name, ownerColor: moi.color,
        visibility: "private", editable: "owner", notes: [], createdAt: 1 },
      { id: "b", ownerId: "u2", ownerName: "Grace", ownerColor: "#34d399",
        visibility: "shared", editable: "owner",
        notes: [{ id: "n", text: "chez Grace", createdAt: 1 }], createdAt: 2 },
    ]);
    // On juge sur le nom affiché en tête du panneau : c'est lui qui dit quel
    // rideau est ouvert, indépendamment de ce qu'il contient.
    expect(screen.queryByText("Grace")).toBeNull();
    fireEvent.pointerEnter(screen.getByLabelText("Rideau de Grace"));
    expect(screen.getByText("Grace")).toBeTruthy();
  });

  it("la languette « + » disparaît quand j'ai déjà le mien", () => {
    const moi = getLocalUser();
    monter([{ id: "a", ownerId: moi.id, ownerName: moi.name, ownerColor: moi.color,
      visibility: "private", editable: "owner", notes: [], createdAt: 1 }]);
    expect(screen.queryByLabelText("Créer mon rideau personnel")).toBeNull();
  });

  it("renommer l'identité ne me fait pas perdre mon rideau", () => {
    // L'appartenance tient à l'identifiant stable, jamais au nom.
    const moi = getLocalUser();
    monter([{ id: "a", ownerId: moi.id, ownerName: moi.name, ownerColor: moi.color,
      visibility: "private", editable: "owner", notes: [], createdAt: 1 }]);
    setLocalUser({ name: "Tout autre chose" });
    expect(screen.queryByLabelText("Créer mon rideau personnel")).toBeNull();
  });
});
