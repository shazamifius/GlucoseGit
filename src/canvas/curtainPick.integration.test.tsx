// ────────────────────────────────────────────────────────────────────────────
// MEMB-8 — Le rideau est un monde de clic autonome.
//
// Ce que ces tests verrouillent est exactement ce qui manquait après MEMB-7 :
// le rideau montait bien les couches de Glucose, mais il n'avait pas d'arbitre,
// donc pas de cycle « re-clic = cible suivante », pas de préhension d'image, et
// son déplacement partait sur la sélection de la SCÈNE.
//
// On monte le VRAI canvas de rideau sur un VRAI board du store, on envoie de
// vrais évènements pointeur, et on relit le store. Rien n'est simulé sauf le
// temps (le cycle a des seuils en millisecondes qu'on ne va pas attendre).
//
// PixiJS n'entre pas ici : dans un rideau les images sont en DOM, précisément
// pour ça. C'est le seul endroit de Glucose où la préhension d'image est
// couverte par des tests.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render } from "@testing-library/react";
import type { Annotation, BoardImage } from "../types";
import { useGlucoseStore } from "../store";
import CurtainCanvas from "./CurtainCanvas";
import { PICK } from "./hitPriority";
import { beginPick, curtainScope, registerPickHandler } from "./pickArbiter";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: (s: string) => s,
}));

const BOARD = "rideau";

// La caméra du rideau démarre à (40, 40) à l'échelle 1 : un point MONDE (wx, wy)
// se clique donc en (wx + 40, wy + 40) écran. jsdom rend des rectangles nuls,
// l'origine du panneau est (0, 0).
const OFF = 40;
const at = (wx: number, wy: number) => ({ clientX: wx + OFF, clientY: wy + OFF, button: 0 });

/**
 * Une pile délibérée, au même point (200, 150) :
 *   • l'IMAGE couvre 150..250 × 110..190   → rang 30
 *   • le TEXTE couvre 170..230 × 130..170  → rang 50, et il est PEINT DESSUS
 *   • la MEMBRANE couvre tout              → rang 60 (corps)
 *
 * C'est le cas que l'utilisateur décrit : une image SOUS un texte, dans une
 * membrane. L'image doit gagner malgré le texte au-dessus.
 */
function seed() {
  useGlucoseStore.getState().loadProject({
    version: "2.0.0", name: "test",
    boards: [{
      id: BOARD, name: "Rideau",
      images: [], annotations: [], panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 },
      createdAt: 0, updatedAt: 0,
    }],
    activeBoardId: BOARD, presets: [], domains: [],
    createdAt: 0, updatedAt: 0,
  });
  const st = useGlucoseStore.getState();
  st.addAnnotation(BOARD, {
    id: "MEMB", type: "membrane", x: 0, y: 0, width: 400, height: 300, color: "#60a5fa",
  } as Annotation);
  st.addImage(BOARD, {
    id: "IMG", src: "data:,", x: 200, y: 150, width: 100, height: 80, rotation: 0,
  } as BoardImage);
  st.addAnnotation(BOARD, {
    id: "TXT", type: "text", x: 170, y: 130, width: 60, height: 40, text: "coucou",
  } as Annotation);
}

function board() {
  return useGlucoseStore.getState().project.boards.find((b) => b.id === BOARD)!;
}
function image(id = "IMG") {
  return board().images.find((i) => i.id === id)!;
}
function annotation(id: string) {
  return board().annotations.find((a) => a.id === id)!;
}

/** Un clic complet, immobile. */
function click(el: Element, wx: number, wy: number) {
  fireEvent.pointerDown(el, at(wx, wy));
  fireEvent.pointerUp(window, at(wx, wy));
}

/** Un glisser : appui, deux mouvements (le 1er franchit le seuil), relâcher. */
function drag(el: Element, wx: number, wy: number, dx: number, dy: number) {
  fireEvent.pointerDown(el, at(wx, wy));
  fireEvent.pointerMove(window, at(wx + dx, wy + dy));
  fireEvent.pointerMove(window, at(wx + dx, wy + dy));
  fireEvent.pointerUp(window, at(wx + dx, wy + dy));
}

/** Avance l'horloge au-delà du seuil de double-clic : le re-clic suivant est un
 *  vrai re-clic, pas la 2e moitié d'un double-clic. */
function apresDoubleClic() {
  vi.setSystemTime(Date.now() + PICK.DBLCLICK_MS + 50);
}

function monter(editable = true) {
  seed();
  const vue = render(<CurtainCanvas boardId={BOARD} editable={editable} />);
  return { ...vue, root: vue.container.firstElementChild as HTMLElement };
}

beforeEach(() => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  vi.setSystemTime(new Date("2026-01-01T00:00:00Z"));
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe("MEMB-8 — l'ordre de priorité s'applique DANS le rideau", () => {
  it("une image sous un texte gagne le premier clic", () => {
    const { root } = monter();
    // Un seul clic, puis on tire : c'est l'image qui doit suivre le curseur.
    drag(root, 200, 150, 30, 0);
    expect(image().x).toBeCloseTo(230, 1);
    expect(annotation("TXT").x).toBeCloseTo(170, 1); // le texte n'a pas bougé
  });

  it("le corps de la membrane ne gagne jamais sur son contenu", () => {
    const { root } = monter();
    // (60, 60) : dans la membrane, loin de son bord et de tout contenu.
    drag(root, 60, 60, 25, 0);
    expect(annotation("MEMB").x).toBeCloseTo(25, 1);
    expect(image().x).toBeCloseTo(200, 1);
  });
});

describe("MEMB-8 — le cycle « re-clic = cible suivante » vit aussi dans le rideau", () => {
  it("re-cliquer sans bouger descend d'un cran, et le glisser suivant prend la nouvelle cible", () => {
    const { root } = monter();

    click(root, 200, 150);           // 1er clic → image (rang le plus prioritaire)
    apresDoubleClic();
    click(root, 200, 150);           // re-clic → au relâchement, le cran passe au texte

    // Le glisser suivant doit donc emmener le TEXTE, pas l'image.
    drag(root, 200, 150, 40, 0);
    expect(annotation("TXT").x).toBeCloseTo(210, 1);
    expect(image().x).toBeCloseTo(200, 1);
  });

  it("le texte est un TERMINUS : le cycle ne va pas au-delà", () => {
    const { root } = monter();

    click(root, 200, 150);           // image
    apresDoubleClic();
    click(root, 200, 150);           // texte
    apresDoubleClic();
    click(root, 200, 150);           // ne doit PAS passer à la membrane

    // Le glisser doit rester assez espacé du clic précédent pour ne pas être lu
    // comme la 2e moitié d'un double-clic — sinon c'est l'ÉDITEUR qui s'ouvre,
    // et c'est justement la raison pour laquelle le texte est un terminus.
    apresDoubleClic();
    drag(root, 200, 150, 15, 0);
    expect(annotation("TXT").x).toBeCloseTo(185, 1);
    expect(annotation("MEMB").x).toBeCloseTo(0, 1);
  });

  it("un GLISSER n'avance aucun cran — c'est ce qui rend « je clique, puis je tire » sûr", () => {
    const { root } = monter();

    click(root, 200, 150);           // image
    apresDoubleClic();
    drag(root, 200, 150, 20, 0);     // on tire : le cycle est oublié, pas avancé
    expect(image().x).toBeCloseTo(220, 1);

    // Comme l'image a bougé sous le curseur, elle n'est plus au même endroit :
    // on rejoue depuis son nouveau centre. Le cycle doit repartir de zéro.
    apresDoubleClic();
    drag(root, 220, 150, 10, 0);
    expect(image().x).toBeCloseTo(230, 1);
    expect(annotation("TXT").x).toBeCloseTo(170, 1);
  });
});

describe("MEMB-8 — la poignée gagne toujours, et sa préhension est généreuse", () => {
  it("une image sélectionnée se redimensionne depuis LOIN de son coin dessiné", () => {
    const { root } = monter();
    click(root, 200, 150);           // sélectionne l'image (rang 30)

    // Coin bas-droit de l'image : (250, 190). On vise 12 px à côté — bien au-delà
    // du carré dessiné (9 px), bien en deçà de la préhension (24 px écran).
    const w0 = image().width;
    drag(root, 262, 202, 20, 16);
    expect(image().width).toBeGreaterThan(w0);
  });

  it("hors de la préhension, le clic retombe sur le contenu et déplace au lieu de redimensionner", () => {
    const { root } = monter();
    click(root, 200, 150);
    const w0 = image().width;

    // 40 px au-delà du coin : au-dessus du plafond de préhension. Là il n'y a
    // plus rien que la membrane — c'est elle qui doit bouger.
    drag(root, 290, 230, 20, 0);
    expect(image().width).toBeCloseTo(w0, 1);
    expect(annotation("MEMB").x).toBeCloseTo(20, 1);
  });
});

describe("MEMB-8 — le rideau ne vole pas le routage de la scène", () => {
  it("deux portées coexistent : chacune reçoit ses propres clics", () => {
    const recu: string[] = [];
    const off = registerPickHandler("image", (id) => recu.push(`scene:${id}`));
    monter();

    const ev = new PointerEvent("pointerdown") as PointerEvent;
    // La portée du rideau route vers le rideau…
    beginPick("image", "IMG", ev, undefined, curtainScope(BOARD));
    expect(recu).toEqual([]);        // la scène n'a rien reçu
    // …et la portée de la scène route toujours vers la scène.
    beginPick("image", "IMG", ev);
    expect(recu).toEqual(["scene:IMG"]);
    off();
  });

  it("démonter le rideau libère sa portée", () => {
    const { unmount } = monter();
    const ev = new PointerEvent("pointerdown") as PointerEvent;
    expect(beginPick("image", "IMG", ev, undefined, curtainScope(BOARD))).toBe(true);
    unmount();
    expect(beginPick("image", "IMG", ev, undefined, curtainScope(BOARD))).toBe(false);
  });
});

describe("MEMB-8 — un rideau en lecture seule se regarde sans s'écrire", () => {
  it("aucun glisser n'écrit dans le document", () => {
    const { root } = monter(false);
    const avantImg = image().x;
    const avantTxt = annotation("TXT").x;

    drag(root, 200, 150, 40, 0);
    apresDoubleClic();
    click(root, 200, 150);
    drag(root, 200, 150, 40, 0);

    expect(image().x).toBeCloseTo(avantImg, 1);
    expect(annotation("TXT").x).toBeCloseTo(avantTxt, 1);
  });
});

describe("MEMB-8 — l'appartenance se règle au dépôt, dans le rideau comme dehors", () => {
  it("une image tirée hors de sa membrane cesse d'en être membre", () => {
    const { root } = monter();
    act(() => {
      useGlucoseStore.getState().updateImage(BOARD, "IMG", { membraneId: "MEMB" });
    });
    expect(image().membraneId).toBe("MEMB");

    // La membrane s'arrête à x = 400 : on emmène l'image bien au-delà.
    drag(root, 200, 150, 400, 0);
    expect(image().x).toBeCloseTo(600, 1);
    expect(image().membraneId).toBeUndefined();
  });
});
