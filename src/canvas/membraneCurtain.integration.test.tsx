// ────────────────────────────────────────────────────────────────────────────
// MEMB-3 — Vérification de bout en bout du rideau.
//
// `curtainPanel` et `curtainModel` sont testés à part. Ici on monte la VRAIE
// couche, on crée un rideau, on écrit une note, on change les permissions, et
// on regarde le DOM et le store. Ce que le modèle interdit doit être
// inatteignable à l'écran, pas seulement faux en mémoire.
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
  return render(<MembraneCurtainLayer membrane={current()} boardId={BOARD} />);
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

// ── Notes ───────────────────────────────────────────────────────────────────

describe("notes", () => {
  function mienAvecNote(text = "Relire le chapitre II") {
    const moi = getLocalUser();
    const c: MembraneCurtain = {
      id: "c1", ownerId: moi.id, ownerName: moi.name, ownerColor: moi.color,
      visibility: "private", editable: "owner",
      notes: [{ id: "n1", text, createdAt: 1 }], createdAt: 1,
    };
    return c;
  }

  it("affiche mes notes", () => {
    monter([mienAvecNote()]);
    expect((screen.getByLabelText("Note du rideau") as HTMLTextAreaElement).value)
      .toBe("Relire le chapitre II");
  });

  it("écrire puis quitter le champ enregistre", () => {
    monter([mienAvecNote()]);
    const champ = screen.getByLabelText("Note du rideau");
    fireEvent.change(champ, { target: { value: "  Version corrigée  " } });
    fireEvent.blur(champ);
    expect(stored()[0].notes[0].text).toBe("Version corrigée");
  });

  it("vider une note la supprime — pas de bloc fantôme", () => {
    monter([mienAvecNote()]);
    const champ = screen.getByLabelText("Note du rideau");
    fireEvent.change(champ, { target: { value: "   " } });
    fireEvent.blur(champ);
    expect(stored()[0].notes).toHaveLength(0);
  });

  it("Échap annule la saisie en cours", () => {
    monter([mienAvecNote()]);
    const champ = screen.getByLabelText("Note du rideau") as HTMLTextAreaElement;
    fireEvent.change(champ, { target: { value: "n'importe quoi" } });
    fireEvent.keyDown(champ, { key: "Escape" });
    fireEvent.blur(champ);
    expect(stored()[0].notes[0].text).toBe("Relire le chapitre II");
  });

  it("ajouter une note", () => {
    monter([mienAvecNote()]);
    fireEvent.click(screen.getByText("+ note"));
    expect(stored()[0].notes).toHaveLength(2);
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
    monter([curtainDe(AUTRUI, { visibility: "shared", editable: "owner" })]);
    expect(screen.getByLabelText("Rideau de Grace")).toBeTruthy();
    expect(screen.getByText("secret")).toBeTruthy();
    expect(screen.queryByLabelText("Note du rideau")).toBeNull(); // pas de champ
    expect(screen.queryByText("+ note")).toBeNull();
    expect(screen.getByText("Lecture seule")).toBeTruthy();
  });

  it("l'ATELIER d'un autre s'édite", () => {
    monter([curtainDe(AUTRUI, { visibility: "shared", editable: "everyone" })]);
    expect(screen.getByLabelText("Note du rideau")).toBeTruthy();
    expect(screen.getByText("+ note")).toBeTruthy();
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
    expect(screen.queryByText("chez Grace")).toBeNull();
    fireEvent.pointerEnter(screen.getByLabelText("Rideau de Grace"));
    expect(screen.getByText("chez Grace")).toBeTruthy();
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
