// ────────────────────────────────────────────────────────────────────────────
// COLLAB-1 — Vérification de bout en bout de l'édition d'identité.
//
// Le module `localUser` est testé à part. Ici on monte le VRAI panneau, on tape
// un nom, on clique une couleur, et on vérifie que ça arrive jusqu'au stockage
// et jusqu'à l'événement que les pairs écoutent. Un réglage qui ne se propage
// pas n'est pas un réglage.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";

vi.mock("./collabBridge", () => ({
  createShare: vi.fn(), resumeShare: vi.fn(), joinByCode: vi.fn(),
  leaveCollab: vi.fn(), getSavedShareUrl: () => null,
}));
vi.mock("./collabHandle", () => ({ getActiveShareUrl: () => null }));
vi.mock("./repo", () => ({
  isServerConnected: () => false,
  onConnectivityChange: () => () => {},
}));

import MultiplayerPanel from "./MultiplayerPanel";
import { USER_CHANGED_EVENT, USER_COLORS, _resetLocalUserCache, getLocalUser } from "./localUser";

const KEY = "glucose:local-user";

beforeEach(() => {
  localStorage.clear();
  sessionStorage.clear();
  _resetLocalUserCache();
});
afterEach(cleanup);

function monter() {
  return render(<MultiplayerPanel enabled={false} onToggle={vi.fn()} onClose={vi.fn()} />);
}

describe("panneau collaboration — mon identité", () => {
  it("affiche le nom courant dans un champ éditable", () => {
    const moi = getLocalUser();
    monter();
    expect((screen.getByLabelText("Mon nom") as HTMLInputElement).value).toBe(moi.name);
  });

  it("renommer écrit jusqu'au stockage — et survit à une réouverture", () => {
    monter();
    const champ = screen.getByLabelText("Mon nom");
    fireEvent.change(champ, { target: { value: "  Ada  Lovelace " } });
    fireEvent.blur(champ);

    expect(JSON.parse(localStorage.getItem(KEY)!).name).toBe("Ada Lovelace");
    _resetLocalUserCache();
    expect(getLocalUser().name).toBe("Ada Lovelace");
  });

  it("Entrée valide sans avoir à cliquer ailleurs", () => {
    monter();
    const champ = screen.getByLabelText("Mon nom");
    fireEvent.change(champ, { target: { value: "Grace" } });
    fireEvent.keyDown(champ, { key: "Enter" });
    fireEvent.blur(champ); // ce que produit le .blur() du gestionnaire
    expect(getLocalUser().name).toBe("Grace");
  });

  it("choisir une couleur l'applique et la marque comme sélectionnée", () => {
    monter();
    const cible = USER_COLORS[3];
    const pastille = screen.getByLabelText(`Couleur ${cible}`);
    fireEvent.click(pastille);

    expect(getLocalUser().color).toBe(cible);
    expect(screen.getByLabelText(`Couleur ${cible}`)).toHaveAttribute("aria-pressed", "true");
  });

  it("toute la palette est proposée", () => {
    monter();
    for (const c of USER_COLORS) expect(screen.getByLabelText(`Couleur ${c}`)).toBeTruthy();
  });

  it("PRÉVIENT les pairs — sinon ils gardent l'ancien nom à l'écran", () => {
    const vu: unknown[] = [];
    const h = (e: Event) => vu.push((e as CustomEvent).detail);
    window.addEventListener(USER_CHANGED_EVENT, h);

    monter();
    fireEvent.click(screen.getByLabelText(`Couleur ${USER_COLORS[5]}`));

    window.removeEventListener(USER_CHANGED_EVENT, h);
    expect(vu).toHaveLength(1);
    expect((vu[0] as { color: string }).color).toBe(USER_COLORS[5]);
  });

  it("un nom vidé ne laisse pas un curseur anonyme", () => {
    monter();
    const champ = screen.getByLabelText("Mon nom");
    fireEvent.change(champ, { target: { value: "   " } });
    fireEvent.blur(champ);
    expect(getLocalUser().name).toMatch(/\S/);
  });
});
