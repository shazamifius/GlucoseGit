import { describe, it, expect } from "vitest";
import {
  addAnchor,
  createAnchor,
  domPointToOffset,
  highlightDomRanges,
  indexDomText,
  normalizeTextSel,
  resolveAnchors,
  resolveTextSel,
  hasTextSelection,
  type TextAnchor,
} from "./textAnchors";

/** Le texte exact du bug rapporté : « bonjours » y apparaît deux fois. */
const PLAIN = "bonjourstesttestbonjours";
const FIRST = 0;
const SECOND = PLAIN.lastIndexOf("bonjours");

describe("createAnchor", () => {
  it("mémorise la position, la citation et le contexte", () => {
    const a = createAnchor(PLAIN, SECOND, SECOND + 8);
    expect(a).toEqual({
      start: SECOND,
      end: SECOND + 8,
      quote: "bonjours",
      prefix: "bonjourstesttest",
      suffix: "",
    });
  });

  it("rogne les blancs de bord et rejette une sélection vide", () => {
    const text = "a  mot  b";
    expect(createAnchor(text, 1, 7)).toMatchObject({ start: 3, end: 6, quote: "mot" });
    expect(createAnchor(text, 1, 3)).toBeNull();
  });
});

describe("resolveAnchors — le bug d'origine", () => {
  it("ne surligne QUE l'occurrence sélectionnée", () => {
    const anchor = createAnchor(PLAIN, SECOND, SECOND + 8) as TextAnchor;
    expect(resolveAnchors(PLAIN, [anchor])).toEqual([{ start: SECOND, end: SECOND + 8 }]);
  });

  it("distingue la première de la seconde occurrence", () => {
    const first = createAnchor(PLAIN, FIRST, FIRST + 8) as TextAnchor;
    expect(resolveAnchors(PLAIN, [first])).toEqual([{ start: FIRST, end: FIRST + 8 }]);
  });

  it("accepte deux fois le même mot comme deux ancres distinctes", () => {
    let anchors: TextAnchor[] = [];
    anchors = addAnchor(anchors, createAnchor(PLAIN, FIRST, FIRST + 8) as TextAnchor);
    anchors = addAnchor(anchors, createAnchor(PLAIN, SECOND, SECOND + 8) as TextAnchor);
    expect(anchors).toHaveLength(2);
    expect(resolveAnchors(PLAIN, anchors)).toEqual([
      { start: FIRST, end: FIRST + 8 },
      { start: SECOND, end: SECOND + 8 },
    ]);
  });

  it("ignore un ajout qui chevauche une ancre existante", () => {
    const anchors = addAnchor([createAnchor(PLAIN, FIRST, FIRST + 8) as TextAnchor], {
      start: FIRST + 3,
      end: FIRST + 6,
      quote: "jou",
    });
    expect(anchors).toHaveLength(1);
  });

  it("abandonne une ancre dont la citation a disparu", () => {
    expect(resolveAnchors(PLAIN, [{ start: 0, end: 5, quote: "absent" }])).toEqual([]);
  });

  it("fusionne les plages qui se recouvrent", () => {
    const ranges = resolveAnchors("abcdef", [
      { start: 0, end: 3, quote: "abc" },
      { start: 2, end: 5, quote: "cde" },
    ]);
    expect(ranges).toEqual([{ start: 0, end: 5 }]);
  });
});

describe("resolveAnchors — ré-ancrage après édition", () => {
  it("suit la citation quand les offsets ont glissé", () => {
    const anchor = createAnchor(PLAIN, SECOND, SECOND + 8) as TextAnchor;
    const edited = `préface ${PLAIN}`;
    const shifted = SECOND + "préface ".length;
    expect(resolveAnchors(edited, [anchor])).toEqual([{ start: shifted, end: shifted + 8 }]);
  });

  it("tranche entre occurrences homonymes grâce au contexte", () => {
    // Ancre sur le SECOND « bonjours » : son contexte gauche est « …testtest ».
    const anchor = createAnchor(PLAIN, SECOND, SECOND + 8) as TextAnchor;
    const edited = `XX${PLAIN}`;
    // Les deux « bonjours » sont candidats ; seul le préfixe départage.
    expect(resolveAnchors(edited, [anchor])).toEqual([{ start: SECOND + 2, end: SECOND + 10 }]);
  });

  it("privilégie la position tant qu'elle reste valide", () => {
    // Le premier « bonjours » n'a aucun contexte gauche : sans la priorité
    // donnée aux offsets, le ré-ancrage pourrait dériver vers son homonyme.
    const anchor = createAnchor(PLAIN, FIRST, FIRST + 8) as TextAnchor;
    expect(resolveAnchors(PLAIN, [anchor])).toEqual([{ start: FIRST, end: FIRST + 8 }]);
  });
});

describe("normalizeTextSel — compat des anciens projets", () => {
  it("convertit l'ancienne chaîne « a ‖ b » en ancres sans position", () => {
    expect(normalizeTextSel("bonjours ‖ test")).toEqual([
      { start: -1, end: -1, quote: "bonjours" },
      { start: -1, end: -1, quote: "test" },
    ]);
  });

  it("résout une ancienne sélection sur la première occurrence", () => {
    expect(resolveTextSel(PLAIN, "bonjours")).toEqual([{ start: FIRST, end: FIRST + 8 }]);
  });

  it("distingue absence de sélection et tableau vide", () => {
    expect(hasTextSelection(undefined)).toBe(false);
    expect(hasTextSelection([])).toBe(false);
    expect(hasTextSelection("bonjours")).toBe(true);
  });
});

describe("pont DOM", () => {
  function mount(html: string): HTMLElement {
    const root = document.createElement("div");
    root.innerHTML = html;
    document.body.appendChild(root);
    return root;
  }

  it("indexe le texte rendu à travers les éléments", () => {
    const root = mount("<p>bonjours</p><p>test</p>");
    expect(indexDomText(root).plain).toBe("bonjourstest");
  });

  it("convertit un point DOM en offset", () => {
    const root = mount("<p>bonjours</p><p>test</p>");
    const second = root.querySelectorAll("p")[1].firstChild as Text;
    expect(domPointToOffset(root, second, 2)).toBe(10);
  });

  it("surligne une plage qui traverse plusieurs nœuds — cas que l'ancien indexOf ratait", () => {
    const root = mount("<p><strong>bon</strong>jours</p>");
    const undo = highlightDomRanges(root, [{ start: 0, end: 8 }], () => {});
    const marks = root.querySelectorAll("mark[data-glucose-hl]");
    expect(marks).toHaveLength(2);
    expect(Array.from(marks, m => m.textContent).join("")).toBe("bonjours");
    undo();
    expect(root.querySelectorAll("mark")).toHaveLength(0);
    expect(indexDomText(root).plain).toBe("bonjours");
  });

  it("ne surligne que l'occurrence ancrée et restitue le DOM au retrait", () => {
    const root = mount("<p>bonjours</p><p>test</p><p>bonjours</p>");
    const plain = indexDomText(root).plain;
    const anchor = createAnchor(plain, plain.lastIndexOf("bonjours"), plain.length) as TextAnchor;
    const undo = highlightDomRanges(root, resolveAnchors(plain, [anchor]), () => {});
    const marks = root.querySelectorAll("mark[data-glucose-hl]");
    expect(marks).toHaveLength(1);
    expect(marks[0].parentElement).toBe(root.querySelectorAll("p")[2]);
    undo();
    expect(root.innerHTML).toBe("<p>bonjours</p><p>test</p><p>bonjours</p>");
  });
});
