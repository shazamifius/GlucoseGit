// ────────────────────────────────────────────────────────────────────────────
// MEMB-6 — Le passage fluide, jusqu'au style produit.
//
// `membraneTween` démontre l'interpolation sur des cartes. Ce qu'on vérifie ici
// est la seule chose qu'elle ne peut pas prouver : que la carte intermédiaire
// traverse bien les couches et ressort en géométrie à l'écran.
//
// C'est tout l'enjeu du choix d'implantation. L'animation n'a été écrite NULLE
// PART dans les couches — ni dans les sprites, ni dans les textes, ni dans les
// membranes : on interpole `geom`, et elles suivent sans rien savoir. Si ce
// test passe, c'est que la couture unique tient.
//
// RÉSERVE HONNÊTE : PixiJS n'est pas couvert (jsdom n'a pas de WebGL). Les
// images passent par le même `geom` et par `projectBoard`, donc le raisonnement
// vaut — mais il n'est pas vérifié ici, et la boucle `requestAnimationFrame` du
// canvas non plus.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import type { Annotation } from "../types";
import { itemsOfBoard, resolveItems } from "./membraneSpace";
import { geomOrNatural, planTween, tweenGeom } from "./membraneTween";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: (s: string) => s,
}));

import HtmlAnnotationLayer from "./HtmlAnnotationLayer";
import SvgAnnotationLayer from "./SvgAnnotationLayer";

afterEach(cleanup);

const vpRef = () => ({ current: { x: 0, y: 0, scale: 1 } });

/** Membrane 200×200 dont le contenu s'étend sur 800 → échelle finale 0,25. */
const MINI = {
  id: "M", type: "membrane", x: 0, y: 0, width: 200, height: 200, mode: "minimized",
} as Annotation;

const texte = (membraneId?: string): Annotation => ({
  id: "T", type: "text", x: 400, y: 400, text: "bonjour",
  width: 400, height: 400, membraneId,
} as Annotation);

/**
 * Géométrie à mi-course du dépôt de `T` dans la membrane — exactement ce que
 * le canvas fabrique à l'image du milieu.
 */
function miCourse(p = 0.5) {
  const avant = { images: [], annotations: [MINI, texte()] };
  const apres = { images: [], annotations: [MINI, texte("M")] };
  const itemsAvant = itemsOfBoard(avant);
  const itemsApres = itemsOfBoard(apres);
  const cible = resolveItems(itemsApres);

  const plan = planTween(
    { sig: "avant", items: itemsAvant, geom: null },
    { sig: "apres", items: itemsApres, geom: cible },
  );
  if (!plan) throw new Error("le dépôt aurait dû déclencher une animation");
  return { geom: tweenGeom(plan.from, cible, plan.ids, p), cible, plan };
}

function monterTexte(annotations: Annotation[], geom: ReturnType<typeof resolveItems> | null) {
  return render(
    <HtmlAnnotationLayer
      annotations={annotations.filter((a) => a.type === "text" || a.type === "sticky")}
      selectedIds={[]} editingId={null} vpRef={vpRef()} geom={geom}
      onSelect={vi.fn()} onEdit={vi.fn()} onResize={vi.fn()}
    />,
  );
}

// ── La carte intermédiaire atteint le style ─────────────────────────────────

describe("à mi-course, l'élément est à mi-chemin — À L'ÉCRAN", () => {
  it("le texte est rendu à une échelle intermédiaire, pas à sa taille finale", () => {
    const { geom } = miCourse(0.5);
    const { container } = monterTexte([texte("M")], geom);
    const el = container.querySelector('[data-id="T"]') as HTMLElement;
    // 1 → 0,25 : la moitié du trajet vaut 0,625. Ni 1 (départ), ni 0,25 (arrivée).
    expect(el.style.transform).toBe("scale(0.625)");
    expect(el.style.transformOrigin).toBe("top left");
  });

  it("et il a aussi PARCOURU la moitié du chemin, il ne fait pas que rétrécir", () => {
    // Le mouvement compte autant que la taille : l'objet doit se voir traverser,
    // sinon il disparaît d'un endroit et réapparaît à un autre.
    const { geom, cible } = miCourse(0.5);
    const { container } = monterTexte([texte("M")], geom);
    const el = container.querySelector('[data-id="T"]') as HTMLElement;
    const gauche = Number.parseFloat(el.style.left);
    expect(gauche).toBeLessThan(400);                    // parti de 400 (naturel)
    expect(gauche).toBeGreaterThan(cible.get("T")!.x);   // pas encore arrivé
  });

  it("au bout, le style est EXACTEMENT celui de la géométrie finale", () => {
    // L'animation ne doit rien laisser derrière elle : arrivée == pas d'animation.
    const { cible, plan } = miCourse();
    const fin = tweenGeom(plan.from, cible, plan.ids, 1);
    expect(fin).toBe(cible); // la carte cible elle-même, pas une copie

    const anime = monterTexte([texte("M")], fin);
    const el1 = anime.container.querySelector('[data-id="T"]') as HTMLElement;
    const style1 = { left: el1.style.left, top: el1.style.top, transform: el1.style.transform };
    cleanup();

    const direct = monterTexte([texte("M")], cible);
    const el2 = direct.container.querySelector('[data-id="T"]') as HTMLElement;
    expect({ left: el2.style.left, top: el2.style.top, transform: el2.style.transform })
      .toEqual(style1);
  });
});

// ── Les membranes imbriquées ────────────────────────────────────────────────

describe("une membrane imbriquée s'anime comme le reste", () => {
  const INTERNE = {
    id: "N", type: "membrane", x: 0, y: 0, width: 400, height: 400,
    mode: "minimized", membraneId: "M",
  } as Annotation;
  const CONTENU = {
    id: "T", type: "text", x: 0, y: 0, text: "x", width: 800, height: 800, membraneId: "N",
  } as Annotation;

  it("son transform SVG est intermédiaire, pas final", () => {
    const avant = { images: [], annotations: [MINI, { ...INTERNE, membraneId: undefined }, CONTENU] };
    const apres = { images: [], annotations: [MINI, INTERNE, CONTENU] };
    const cible = resolveItems(itemsOfBoard(apres));
    const plan = planTween(
      { sig: "a", items: itemsOfBoard(avant), geom: null },
      { sig: "b", items: itemsOfBoard(apres), geom: cible },
    );
    expect(plan).not.toBeNull();

    const geom = tweenGeom(plan!.from, cible, plan!.ids, 0.5);
    const { container } = render(
      <SvgAnnotationLayer
        annotations={[MINI, INTERNE]}
        selectedIds={[]} editingId={null} vpRef={vpRef()} geom={geom}
        onSelect={vi.fn()} onEdit={vi.fn()} onResize={vi.fn()}
      />,
    );
    const g = container.querySelector('[data-pick-id="N"]');
    const t = g?.getAttribute("transform") ?? "";
    const echelle = Number.parseFloat(t.match(/scale\(([\d.]+)\)/)?.[1] ?? "1");
    const finale = geomOrNatural(itemsOfBoard(apres), cible).get("N")!.scale;
    expect(echelle).toBeGreaterThan(finale);
    expect(echelle).toBeLessThan(1);
  });
});

// ── Le garde-fou ────────────────────────────────────────────────────────────

describe("GARDE-FOU — hors animation, rien n'est touché", () => {
  it("sans membrane minimisée, le texte est posé à ses coordonnées, sans transform", () => {
    const { container } = monterTexte([texte()], null);
    const el = container.querySelector('[data-id="T"]') as HTMLElement;
    expect(el.style.left).toBe("400px");
    expect(el.style.transform).toBe("");
  });
});
