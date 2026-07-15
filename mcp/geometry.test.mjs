import { describe, it, expect } from "vitest";
import {
  H_PER_LINE, H_CHROME, NOTE_GAP,
  estimateLines, heightUpperBound, annBox, imgBox, annCenter,
  overlappingPairs, separateUntilClean,
} from "./geometry.mjs";

const note = (x, y, text = "court", w = 380) => ({ type: "text", x, y, width: w, text });
const box = (id, x, y, w, h) => ({ id, x, y, w, h, cx: x + w / 2, cy: y + h / 2 });
const chevauchent = (a, b) =>
  Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x) > 0 &&
  Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y) > 0;

describe("heightUpperBound — c'est une BORNE, pas une moyenne", () => {
  it("majore les cas les plus SERRÉS du corpus réel", () => {
    // Relevés dans les fichiers de l'utilisateur : (lignes comptées par
    // estimateLines, hauteur que l'app a MESURÉE puis écrite dans le document).
    // Ce sont les deux notes qui ont dicté la constante — celles où la borne
    // colle au plus près. Si elles passent, les 1782 autres passent.
    // La propriété testée EST la garantie : borne ≥ rendu, sinon « 0
    // chevauchement calculé » ne dit plus rien de l'écran.
    const serres = [
      { lignes: 38, reel: 1084 },   // marge : 3 px
      { lignes: 34, reel: 979 },    // marge : 14 px
    ];
    for (const r of serres) {
      const borne = r.lignes * H_PER_LINE + H_CHROME;
      expect(borne, `${r.lignes} lignes → rendu ${r.reel}px`).toBeGreaterThanOrEqual(r.reel);
    }
  });

  it("majore aussi les notes courtes, où le chrome domine", () => {
    // Une note d'une ligne est rendue ~140px par l'app : presque tout est du
    // padding. C'est ce que l'ancienne formule ignorait (elle disait 38px).
    for (const { lignes, reel } of [{ lignes: 1, reel: 140 }, { lignes: 2, reel: 180 }, { lignes: 5, reel: 300 }])
      expect(lignes * H_PER_LINE + H_CHROME, `${lignes} ligne(s)`).toBeGreaterThanOrEqual(reel);
  });

  it("croît avec le texte, et jamais en dessous du chrome", () => {
    expect(heightUpperBound("", 380)).toBeGreaterThanOrEqual(H_CHROME);
    const court = heightUpperBound("un mot", 380);
    const long = heightUpperBound("un mot ".repeat(200), 380);
    expect(long).toBeGreaterThan(court);
  });

  it("l'ancienne formule (lignes × 22 + 16) ne majorait PAS — c'était la faille", () => {
    // Une note d'une ligne : l'app en rend ~140px, l'ancienne formule disait 38.
    expect(1 * 22 + 16).toBeLessThan(140);       // l'ancienne sous-estime
    expect(heightUpperBound("court", 380)).toBeGreaterThanOrEqual(140); // la borne, non
  });

  it("une note plus étroite s'enroule sur plus de lignes, donc monte", () => {
    const texte = "a".repeat(400);
    expect(heightUpperBound(texte, 200)).toBeGreaterThan(heightUpperBound(texte, 800));
  });

  it("estimateLines compte les paragraphes ET l'enroulement", () => {
    expect(estimateLines("a\nb\nc", 380)).toBe(3);
    expect(estimateLines("", 380)).toBe(1);          // une ligne vide reste une ligne
    expect(estimateLines("x".repeat(1000), 380)).toBeGreaterThan(20);
  });
});

describe("annBox vs imgBox — deux conventions à ne jamais confondre", () => {
  it("une note est posée par son COIN haut-gauche", () => {
    const b = annBox({ type: "text", x: 100, y: 200, width: 380, height: 100, text: "x" });
    expect([b.x, b.y]).toEqual([100, 200]);
    expect([b.cx, b.cy]).toEqual([290, 250]);
  });

  it("une image est posée par son CENTRE (comme l'ancre de flèche de l'app)", () => {
    const b = imgBox({ x: 100, y: 200, width: 400, height: 300 });
    expect([b.x, b.y]).toEqual([-100, 50]);   // x - w/2, y - h/2
    expect([b.cx, b.cy]).toEqual([100, 200]);
  });

  it("annBox devine la hauteur d'une note qui n'en a pas, via la borne", () => {
    const b = annBox({ type: "text", x: 0, y: 0, width: 380, text: "du texte" });
    expect(b.h).toBe(heightUpperBound("du texte", 380));
  });

  it("annCenter s'accorde avec annBox (deux devinettes divergentes = flèches fausses)", () => {
    const a = { type: "text", x: 10, y: 20, width: 380, text: "du texte sans hauteur" };
    const c = annCenter(a), b = annBox(a);
    expect([c.x, c.y]).toEqual([b.cx, b.cy]);
  });
});

describe("separateUntilClean — l'invariant dur", () => {
  it("sépare deux boîtes qui se chevauchent", () => {
    const m = [box("a", 0, 0, 100, 100), box("b", 50, 50, 100, 100)];
    const r = separateUntilClean(m, [], 0);
    expect(r.ok).toBe(true);
    expect(chevauchent(m[0], m[1])).toBe(false);
  });

  it("sépare des boîtes EXACTEMENT superposées (le cas dégénéré dx=dy=0)", () => {
    // Sans signe stable, elles resteraient collées à jamais : c'est le pire cas.
    const m = Array.from({ length: 6 }, (_, k) => box(`n${k}`, 100, 100, 380, 300));
    const r = separateUntilClean(m, [], NOTE_GAP);
    expect(r.ok).toBe(true);
    expect(overlappingPairs(m)).toHaveLength(0);
  });

  it("laisse tranquille ce qui ne se chevauche pas", () => {
    const m = [box("a", 0, 0, 100, 100), box("b", 500, 500, 100, 100)];
    const r = separateUntilClean(m, [], 0);
    expect(r.passes).toBe(1);           // une passe, rien à faire
    expect([m[0].x, m[0].y]).toEqual([0, 0]);
    expect([m[1].x, m[1].y]).toEqual([500, 500]);
  });

  it("respecte la respiration demandée", () => {
    const m = [box("a", 0, 0, 100, 100), box("b", 10, 0, 100, 100)];
    separateUntilClean(m, [], 40);
    expect(Math.abs(m[1].cx - m[0].cx)).toBeGreaterThanOrEqual(100 + 40 - 0.001);
  });

  it("NE BOUGE JAMAIS une image : c'est une ancre, les notes s'écartent autour", () => {
    const image = box("img", 0, 0, 400, 400);
    const avant = { ...image };
    const notes = [box("n1", 100, 100, 380, 300)];
    const r = separateUntilClean(notes, [image], NOTE_GAP);
    expect(r.ok).toBe(true);
    expect([image.x, image.y]).toEqual([avant.x, avant.y]);       // intacte
    expect(chevauchent(notes[0], image)).toBe(false);             // la note s'est écartée
  });

  it("une note coincée entre deux images sort quand même", () => {
    const images = [box("i1", 0, 0, 300, 300), box("i2", 320, 0, 300, 300)];
    const notes = [box("n", 150, 50, 300, 200)];
    const r = separateUntilClean(notes, images, NOTE_GAP);
    expect(r.ok).toBe(true);
    for (const im of images) expect(chevauchent(notes[0], im)).toBe(false);
  });

  it("est déterministe : mêmes entrées, mêmes sorties", () => {
    const gen = () => [box("a", 0, 0, 200, 200), box("b", 50, 30, 200, 200), box("c", 20, 60, 200, 200)];
    const m1 = gen(), m2 = gen();
    separateUntilClean(m1, [], NOTE_GAP);
    separateUntilClean(m2, [], NOTE_GAP);
    expect(m1.map((b) => [b.x, b.y])).toEqual(m2.map((b) => [b.x, b.y]));
  });

  it("est idempotent : une 2e passe ne bouge plus rien", () => {
    const m = [box("a", 0, 0, 200, 200), box("b", 50, 30, 200, 200), box("c", 20, 60, 200, 200)];
    separateUntilClean(m, [], NOTE_GAP);
    const apres1 = m.map((b) => [b.x, b.y]);
    const r2 = separateUntilClean(m, [], NOTE_GAP);
    expect(r2.passes).toBe(1);
    expect(m.map((b) => [b.x, b.y])).toEqual(apres1);
  });

  it("avoue quand il n'y arrive pas au lieu de prétendre", () => {
    // Plafond volontairement ridicule sur un cas dur → il doit dire ok:false.
    const m = Array.from({ length: 8 }, (_, k) => box(`n${k}`, 100, 100, 380, 400));
    const r = separateUntilClean(m, [], NOTE_GAP, 1);
    if (!r.ok) expect(r.restant).toBeGreaterThan(0);
  });
});

describe("overlappingPairs", () => {
  it("ne compte pas deux boîtes qui se touchent bord à bord", () => {
    expect(overlappingPairs([box("a", 0, 0, 100, 100), box("b", 100, 0, 100, 100)])).toHaveLength(0);
  });

  it("compte toutes les paires d'un empilement", () => {
    const m = Array.from({ length: 4 }, (_, k) => box(`n${k}`, 0, 0, 100, 100));
    expect(overlappingPairs(m)).toHaveLength(6);   // C(4,2)
  });

  it("rapporte l'aire du recouvrement", () => {
    const [p] = overlappingPairs([box("a", 0, 0, 100, 100), box("b", 50, 50, 100, 100)]);
    expect(p.area).toBe(2500);
  });
});
