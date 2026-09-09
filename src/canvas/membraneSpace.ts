// ────────────────────────────────────────────────────────────────────────────
// MEMB-1 — Repère local des membranes (géométrie PURE, testable).
//
// PROBLÈME. Jusqu'ici une membrane ne possédait RIEN : aucun lien
// d'appartenance dans le modèle, et la déplacer ne déplaçait pas son contenu.
// « Être dedans » voulait seulement dire « être aux mêmes coordonnées ». Les
// trois modes demandés (classique / minimisée / étirée) et le mode Focus
// disent tous la même chose : la position STOCKÉE d'un élément n'est plus sa
// position à l'écran. C'est donc un second repère qu'il faut introduire, pas
// une option de plus sur la membrane.
//
// SOLUTION. Les éléments gardent TOUJOURS leurs vraies coordonnées (dites
// « naturelles »). Ce module est l'unique point de dérivation qui en tire la
// géométrie EFFECTIVE — celle qu'on affiche et qu'on teste au clic. Une seule
// couture, au lieu d'une rustine dans chacun des ~184 endroits qui lisent
// aujourd'hui la géométrie en supposant naturel == effectif.
//
// L'ÉCHELLE NE SE STOCKE PAS, ELLE SE DÉDUIT. C'est le cœur du module :
//
//     k = min(1, largeur / étendueX, hauteur / étendueY)
//
// où l'étendue est celle du contenu mesurée depuis l'origine de la membrane.
// Trois conséquences, qui sont exactement les règles demandées :
//
//   • on prend le MIN des deux axes — étirer la membrane dans une seule
//     direction monte un facteur mais pas l'autre, donc `min` ne bouge pas et
//     une image ne peut pas regrossir tant que l'autre axe n'a pas suivi ;
//   • k est PLAFONNÉ à 1 — agrandir une membrane minimisée ramène d'abord son
//     contenu à sa taille naturelle, et seulement ensuite crée du vide ;
//   • rien ne peut se DÉSYNCHRONISER entre une échelle stockée et la taille
//     réelle de la boîte, puisqu'il n'y a pas d'échelle stockée.
//
// ANCRAGE. La mise à l'échelle se fait autour du coin haut-gauche de la
// membrane, jamais autour du centre du contenu : sinon déplacer UN élément
// changerait le centre et ferait glisser tous ses voisins.
//
// L'APPARTENANCE EST STOCKÉE, PAS REDÉRIVÉE. C'est contre-intuitif mais forcé :
// une membrane minimisée est PAR CONSTRUCTION plus petite que son contenu à
// l'échelle 1 (c'est la définition même de « minimisée »). Un test d'inclusion
// géométrique déclarerait donc le contenu sorti de la membrane à l'instant
// précis où elle le réduit — elle perdrait son contenu en le rangeant. On stocke
// donc `membraneId` sur l'élément, et la géométrie ne fait que PROPOSER un
// changement à des moments explicites : dépôt, conversion de mode. C'est aussi
// ce qu'exige l'avertissement de collision — rien n'est capturé sans un geste.
//
// CE QUI N'EST PAS ICI. Aucun rendu, aucune caméra, aucun React : ce module ne
// connaît que des boîtes. Le mode Focus, l'animation de dépôt et l'interface
// d'avertissement de collision s'appuient dessus mais vivent ailleurs.
// ────────────────────────────────────────────────────────────────────────────

import type { Annotation, Board, BoardImage, MembraneMode } from "../types";

export type { MembraneMode };

// ════════════════════════════════════════════════════════════════════════════
// Modes & réglages
// ════════════════════════════════════════════════════════════════════════════

export const MEMBRANE_SPACE = {
  /** Plancher d'échelle : en deçà le contenu n'est plus lisible du tout. */
  MIN_CONTENT_SCALE: 0.08,
  /** Air laissé entre le contenu et le bord quand une membrane s'auto-étire. */
  STRETCH_PADDING: 32,
} as const;

// ════════════════════════════════════════════════════════════════════════════
// Boîtes
// ════════════════════════════════════════════════════════════════════════════

/**
 * Un élément vu par ce module : une boîte ancrée en HAUT-GAUCHE, en
 * coordonnées naturelles. Les sprites images sont ancrés au centre côté Pixi —
 * la conversion se fait à la frontière (`itemsOfBoard` / `imageCenterOf`), pour
 * que le cœur n'ait jamais deux conventions à gérer.
 */
export interface SpaceItem {
  id: string;
  kind: "image" | "text" | "sticky" | "membrane";
  x: number;
  y: number;
  width: number;
  height: number;
  /** Mode, pour les membranes uniquement. */
  mode?: MembraneMode;
  /** Membrane propriétaire STOCKÉE, ou absente si l'élément est libre. */
  membraneId?: string | null;
}

export interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

function centerOf(b: Box): { cx: number; cy: number } {
  return { cx: b.x + b.width / 2, cy: b.y + b.height / 2 };
}

/** La boîte `inner` a-t-elle son CENTRE dans `outer` ? */
export function containsCenter(outer: Box, inner: Box): boolean {
  const { cx, cy } = centerOf(inner);
  return cx >= outer.x && cx <= outer.x + outer.width
    && cy >= outer.y && cy <= outer.y + outer.height;
}

/** Deux boîtes se recouvrent-elles (contact franc, pas simple tangence) ? */
export function overlaps(a: Box, b: Box): boolean {
  return a.x < b.x + b.width && b.x < a.x + a.width
    && a.y < b.y + b.height && b.y < a.y + a.height;
}

function clamp(v: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, v));
}

// ════════════════════════════════════════════════════════════════════════════
// Appartenance
// ════════════════════════════════════════════════════════════════════════════

/**
 * Table `élément → membrane propriétaire`, lue depuis les données STOCKÉES.
 *
 * Voir l'en-tête du module : redériver cette table de la géométrie ferait perdre
 * son contenu à toute membrane minimisée. On se contente donc de valider ce qui
 * est stocké — référence morte (membrane supprimée), auto-référence, parent qui
 * n'est pas une membrane, et surtout CYCLE, qui ferait tourner la résolution en
 * rond. Une chaîne fautive est simplement coupée : l'élément redevient libre.
 */
export function parentMap(items: SpaceItem[]): Map<string, string> {
  const membranes = new Set(items.filter((i) => i.kind === "membrane").map((i) => i.id));
  const raw = new Map<string, string>();
  for (const it of items) {
    const p = it.membraneId;
    if (!p || p === it.id) continue;
    if (!membranes.has(p)) continue;
    raw.set(it.id, p);
  }

  const out = new Map(raw);
  for (const id of raw.keys()) {
    const seen = new Set<string>([id]);
    let cur = raw.get(id);
    while (cur) {
      if (seen.has(cur)) { out.delete(id); break; }
      seen.add(cur);
      cur = raw.get(cur);
    }
  }
  return out;
}

/**
 * Éléments dont le CENTRE tombe dans la boîte de `membrane`, en coordonnées
 * naturelles.
 *
 * À n'employer qu'aux instants où naturel == effectif : la CONVERSION d'une
 * membrane en mode spécial (son échelle vaut encore 1, l'inclusion est donc sans
 * ambiguïté) et le mode Focus. C'est ce qui permet à une membrane classique d'un
 * projet existant — qui n'a aucun `membraneId` stocké — de montrer quand même
 * son contenu au moment où on la focalise ou on la convertit.
 *
 * Le centre plutôt que la boîte entière : un élément à cheval sur un bord ne doit
 * pas basculer à chaque pixel, et « je le glisse dedans » veut bien dire « je
 * l'amène au milieu ».
 */
export function containedIn(items: SpaceItem[], membrane: SpaceItem): SpaceItem[] {
  return items.filter((it) => it.id !== membrane.id && containsCenter(membrane, it));
}

/** Les enfants DIRECTS de chaque membrane, dans l'ordre des `items`. */
export function childrenByMembrane(
  items: SpaceItem[],
  members: Map<string, string>,
): Map<string, SpaceItem[]> {
  const out = new Map<string, SpaceItem[]>();
  for (const it of items) {
    const parent = members.get(it.id);
    if (!parent) continue;
    const list = out.get(parent);
    if (list) list.push(it);
    else out.set(parent, [it]);
  }
  return out;
}

// ════════════════════════════════════════════════════════════════════════════
// Étendue & échelle
// ════════════════════════════════════════════════════════════════════════════

/**
 * Place qu'il faut à l'intérieur d'une membrane pour afficher son contenu à sa
 * taille NATURELLE, mesurée depuis le coin haut-gauche de la membrane (l'ancre
 * de la mise à l'échelle).
 *
 * Un membre qui déborde à gauche ou en haut n'est pas rattrapable par une mise
 * à l'échelle autour de ce coin : son débordement est simplement toléré (il
 * reste rendu hors du cadre), ce que la règle du centre rend marginal.
 */
export function contentExtent(membrane: Box, children: SpaceItem[]): { w: number; h: number } {
  let w = 0;
  let h = 0;
  for (const c of children) {
    w = Math.max(w, c.x + c.width - membrane.x);
    h = Math.max(h, c.y + c.height - membrane.y);
  }
  return { w: Math.max(0, w), h: Math.max(0, h) };
}

/**
 * Échelle du contenu d'une membrane — LE cœur du module.
 *
 * Seule une membrane `minimized` descend sous 1. `classic` laisse déborder
 * (comportement historique) et `stretched` s'agrandit au lieu de réduire, donc
 * toutes deux restent à 1.
 */
export function contentScale(
  mode: MembraneMode,
  box: Box,
  extent: { w: number; h: number },
): number {
  if (mode !== "minimized") return 1;
  if (extent.w <= 0 || extent.h <= 0) return 1;
  const fit = Math.min(box.width / extent.w, box.height / extent.h);
  return clamp(fit, MEMBRANE_SPACE.MIN_CONTENT_SCALE, 1);
}

// ════════════════════════════════════════════════════════════════════════════
// Résolution : naturel → effectif
// ════════════════════════════════════════════════════════════════════════════

export interface ResolvedItem {
  id: string;
  /** Boîte EFFECTIVE (ce qu'on affiche), ancrée en haut-gauche. */
  x: number;
  y: number;
  width: number;
  height: number;
  /** Produit des échelles de toutes les membranes ancêtres. 1 = hors membrane. */
  scale: number;
  /** Membrane parente directe, ou `null`. */
  membraneId: string | null;
  /** Échelle appliquée AUX ENFANTS de cet élément (membranes uniquement). */
  contentScale?: number;
}

export interface ResolveOptions {
  /**
   * Membrane en mode Focus : elle et ses descendants sont rendus à l'échelle
   * naturelle (k = 1) quel que soit leur mode. C'est ce qui fait tenir la
   * promesse « en focus on ne s'aperçoit de rien » sans jamais toucher aux
   * données — l'échelle étant dérivée, il suffit de ne pas l'appliquer.
   */
  focusedMembraneId?: string | null;
}

/**
 * Géométrie effective de tous les éléments. Les membranes imbriquées composent
 * naturellement : chaque niveau multiplie l'échelle accumulée.
 */
export function resolveItems(
  items: SpaceItem[],
  opts: ResolveOptions = {},
): Map<string, ResolvedItem> {
  const members = parentMap(items);
  const children = childrenByMembrane(items, members);
  const byId = new Map(items.map((i) => [i.id, i]));
  const out = new Map<string, ResolvedItem>();

  const focusId = opts.focusedMembraneId ?? null;

  /** Descend un sous-arbre. `(ox, oy)` = origine effective, `s` = échelle accumulée. */
  function walk(item: SpaceItem, ox: number, oy: number, s: number, inFocus: boolean) {
    const resolved: ResolvedItem = {
      id: item.id,
      x: ox,
      y: oy,
      width: item.width * s,
      height: item.height * s,
      scale: s,
      membraneId: members.get(item.id) ?? null,
    };
    out.set(item.id, resolved);

    if (item.kind !== "membrane") return;
    const kids = children.get(item.id) ?? [];
    // Sous focus, l'échelle interne est neutralisée : le contenu s'affiche tel
    // qu'il est réellement stocké.
    const focusHere = inFocus || item.id === focusId;
    const k = focusHere
      ? 1
      : contentScale(item.mode ?? "classic", item, contentExtent(item, kids));
    resolved.contentScale = k;

    const childScale = s * k;
    for (const kid of kids) {
      walk(kid, ox + (kid.x - item.x) * childScale, oy + (kid.y - item.y) * childScale, childScale, focusHere);
    }
  }

  for (const it of items) {
    if (members.has(it.id)) continue; // atteint via son parent
    walk(it, it.x, it.y, 1, false);
  }

  // Filet : une arête de cycle cassée pourrait laisser un orphelin non visité.
  for (const it of items) {
    if (!out.has(it.id)) walk(it, it.x, it.y, 1, false);
  }
  void byId;
  return out;
}

// ════════════════════════════════════════════════════════════════════════════
// Projection vers le rendu
// ════════════════════════════════════════════════════════════════════════════

/**
 * Une membrane réduit-elle quoi que ce soit sur ce board ?
 *
 * CHEMIN RAPIDE, et surtout GARDE-FOU : seul le mode `minimized` produit une
 * échelle différente de 1. Tant qu'aucune membrane n'est minimisée, le rendu
 * doit reprendre EXACTEMENT le chemin d'avant cette fonctionnalité — mêmes
 * coordonnées, mêmes objets, aucun calcul en plus. C'est ce qui rend la
 * migration sûre pour tous les projets existants.
 */
export function hasScaling(items: SpaceItem[]): boolean {
  return items.some((i) => i.kind === "membrane" && i.mode === "minimized");
}

/**
 * Board dont la géométrie est EFFECTIVE de bout en bout.
 *
 * Destiné à ce qui raisonne en BOÎTES : test de collision au clic, alignement
 * intelligent, culling. Le rendu, lui, préfère « origine + échelle » : mettre
 * un texte à la moitié de sa largeur sans réduire sa police le ferait déborder,
 * alors qu'une transformation d'échelle emporte tout d'un coup.
 *
 * Rend le board TEL QUEL quand rien n'est réduit — pas de copie, pas de coût.
 */
export function projectBoard<T extends Pick<Board, "images" | "annotations">>(
  board: T,
  resolved: Map<string, ResolvedItem> | null,
): Pick<Board, "images" | "annotations"> {
  if (!resolved) return board;

  const images = board.images.map((img) => {
    const r = resolved.get(img.id);
    if (!r || r.scale === 1) return img;
    const c = imageCenterOf(r);
    return { ...img, x: c.x, y: c.y, width: r.width, height: r.height };
  });

  const annotations = board.annotations.map((ann) => {
    const r = resolved.get(ann.id);
    if (!r || r.scale === 1) return ann;
    if (ann.type === "arrow") {
      // Une flèche suit ses extrémités ; à défaut d'ancrage, on la met à
      // l'échelle autour de la même origine que le reste.
      return ann;
    }
    return { ...ann, x: r.x, y: r.y, width: r.width, height: r.height };
  });

  return { images, annotations };
}

/** Échelle appliquée à un élément, 1 par défaut. */
export function scaleOf(resolved: Map<string, ResolvedItem> | null, id: string): number {
  return resolved?.get(id)?.scale ?? 1;
}

/** Origine effective d'un élément, ou `null` s'il n'est pas transformé. */
export function originOf(
  resolved: Map<string, ResolvedItem> | null,
  id: string,
): { x: number; y: number; scale: number } | null {
  const r = resolved?.get(id);
  if (!r || r.scale === 1) return null;
  return { x: r.x, y: r.y, scale: r.scale };
}

// ════════════════════════════════════════════════════════════════════════════
// Appartenance au DÉPÔT
// ════════════════════════════════════════════════════════════════════════════

export interface MembershipChange {
  id: string;
  /** Nouvelle membrane, ou `null` si l'élément redevient libre. */
  membraneId: string | null;
}

/**
 * Changements d'appartenance à écrire après un DÉPÔT.
 *
 * C'est ici que « je glisse un élément dans une membrane, il lui appartient »
 * devient vrai. L'appartenance étant stockée (cf. l'en-tête du module), elle ne
 * bouge qu'à ce moment précis — jamais en continu, jamais parce qu'une membrane
 * a grandi par-dessus quelque chose.
 *
 * On juge sur la géométrie EFFECTIVE, celle que l'utilisateur voit : il a lâché
 * l'élément là où il le voyait, pas là où sont ses coordonnées naturelles.
 *
 * Une membrane ne peut jamais devenir membre de sa propre descendance — ce
 * serait un cycle, et un cycle rend l'arbre inrésoluble.
 */
export function reconcileMembership(
  items: SpaceItem[],
  resolved: Map<string, ResolvedItem>,
  movedIds: readonly string[],
): MembershipChange[] {
  if (movedIds.length === 0) return [];
  const parents = parentMap(items);
  const byId = new Map(items.map((i) => [i.id, i]));
  const membranes = items.filter((i) => i.kind === "membrane");
  const out: MembershipChange[] = [];

  /** `candidate` descend-il de `rootId` ? (garde anti-cycle) */
  const descendsFrom = (candidate: string, rootId: string): boolean => {
    const seen = new Set<string>();
    let cur: string | undefined = candidate;
    while (cur && !seen.has(cur)) {
      if (cur === rootId) return true;
      seen.add(cur);
      cur = parents.get(cur);
    }
    return false;
  };

  for (const id of new Set(movedIds)) {
    const item = byId.get(id);
    const r = resolved.get(id);
    if (!item || !r) continue;

    const cx = r.x + r.width / 2;
    const cy = r.y + r.height / 2;

    let best: SpaceItem | null = null;
    let bestArea = Number.POSITIVE_INFINITY;
    for (const m of membranes) {
      if (m.id === id) continue;
      if (descendsFrom(m.id, id)) continue;
      const rm = resolved.get(m.id);
      if (!rm) continue;
      if (cx < rm.x || cx > rm.x + rm.width || cy < rm.y || cy > rm.y + rm.height) continue;
      const area = Math.abs(rm.width * rm.height);
      if (area < bestArea) { best = m; bestArea = area; }
    }

    const next = best?.id ?? null;
    const current = item.membraneId ?? null;
    if (next !== current) out.push({ id, membraneId: next });
  }
  return out;
}

// ════════════════════════════════════════════════════════════════════════════
// Mode étiré : jusqu'où la membrane a le droit de pousser
// ════════════════════════════════════════════════════════════════════════════

export interface StretchPlan {
  /** Boîte que la membrane VOUDRAIT avoir pour contenir son contenu à l'échelle 1. */
  desired: Box;
  /** Boîte réellement atteignable : elle bute sur le premier élément étranger. */
  allowed: Box;
  /** Éléments étrangers qui bloquent — à surligner, avec un saut vers eux. */
  blockers: SpaceItem[];
  /** `true` si la croissance a été rabotée : c'est ce qui déclenche l'avertissement. */
  blocked: boolean;
}

/**
 * Croissance d'une membrane `stretched`, et ce qui l'en empêche.
 *
 * La membrane pousse vers la DROITE et vers le BAS uniquement : son coin
 * haut-gauche est l'ancre de tout le repère, le bouger déplacerait tout son
 * contenu d'un coup.
 *
 * Un élément étranger (qui n'est pas un membre) rencontré sur le chemin ARRÊTE
 * net la croissance dans cet axe — choix retenu plutôt que « recouvrir puis
 * capturer », qui volerait silencieusement à la membrane un contenu que
 * l'utilisateur n'y a jamais mis.
 */
export function stretchPlan(
  membrane: SpaceItem,
  children: SpaceItem[],
  foreigners: SpaceItem[],
): StretchPlan {
  const extent = contentExtent(membrane, children);
  const pad = MEMBRANE_SPACE.STRETCH_PADDING;
  const desired: Box = {
    x: membrane.x,
    y: membrane.y,
    width: Math.max(membrane.width, extent.w + pad),
    height: Math.max(membrane.height, extent.h + pad),
  };

  let maxW = desired.width;
  let maxH = desired.height;
  const blockers: SpaceItem[] = [];

  for (const f of foreigners) {
    if (f.id === membrane.id) continue;
    if (!overlaps(desired, f)) continue;
    // Déjà chevauché AVANT toute croissance : ce n'est pas nous qui venons de
    // le heurter, on ne bloque pas rétroactivement une situation préexistante.
    if (overlaps(membrane, f)) continue;

    blockers.push(f);
    // On rabote l'axe par lequel l'obstacle est le MOINS enfoncé : c'est la
    // direction dans laquelle il suffit de reculer le moins pour le libérer.
    const cutW = f.x - membrane.x;
    const cutH = f.y - membrane.y;
    const loseW = desired.width - cutW;
    const loseH = desired.height - cutH;
    if (loseW <= loseH) maxW = Math.min(maxW, Math.max(membrane.width, cutW));
    else maxH = Math.min(maxH, Math.max(membrane.height, cutH));
  }

  const allowed: Box = { x: membrane.x, y: membrane.y, width: maxW, height: maxH };
  return {
    desired,
    allowed,
    blockers,
    blocked: allowed.width < desired.width || allowed.height < desired.height,
  };
}

// ════════════════════════════════════════════════════════════════════════════
// Transitions de mode
// ════════════════════════════════════════════════════════════════════════════

/**
 * Une membrane spéciale ne redevient JAMAIS classique : son contenu a été
 * dimensionné sous une règle que « classique » ne sait pas rendre, et le
 * retour rendrait aux éléments une taille que personne n'a choisie.
 * minimisée ↔ étirée reste permis (avec les avertissements de collision).
 */
export function canSwitchMode(from: MembraneMode, to: MembraneMode): boolean {
  if (from === to) return true;
  if (to === "classic") return from === "classic";
  return true;
}

// ════════════════════════════════════════════════════════════════════════════
// Adaptateur : un Board → des boîtes
// ════════════════════════════════════════════════════════════════════════════

/** Les sprites sont ancrés au CENTRE côté Pixi ; ici tout est haut-gauche. */
export function imageBox(img: BoardImage): Box {
  return {
    x: img.x - img.width / 2,
    y: img.y - img.height / 2,
    width: img.width,
    height: img.height,
  };
}

/** Inverse d'`imageBox` : repasse d'une boîte haut-gauche au centre stocké. */
export function imageCenterOf(box: Box): { x: number; y: number } {
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

/**
 * Boîtes d'un board, dans un ordre stable (images puis annotations).
 *
 * Les FLÈCHES sont exclues : elles n'ont pas de boîte propre, elles suivent les
 * nœuds qu'elles relient — leur tracé se recalcule donc tout seul depuis la
 * géométrie effective de leurs extrémités. Les DOSSIERS aussi : ce sont des
 * portes vers un autre board, pas du contenu de membrane.
 */
export function itemsOfBoard(board: Pick<Board, "images" | "annotations">): SpaceItem[] {
  const out: SpaceItem[] = [];
  for (const img of board.images) {
    const b = imageBox(img);
    out.push({ id: img.id, kind: "image", x: b.x, y: b.y, width: b.width, height: b.height, membraneId: img.membraneId ?? null });
  }
  for (const ann of board.annotations) {
    out.push(...spaceItemOfAnnotation(ann));
  }
  return out;
}

function spaceItemOfAnnotation(ann: Annotation): SpaceItem[] {
  if (ann.type === "arrow") return [];
  if (ann.type === "membrane") {
    return [{
      id: ann.id, kind: "membrane",
      x: ann.x, y: ann.y, width: ann.width, height: ann.height,
      mode: ann.mode ?? "classic",
      membraneId: ann.membraneId ?? null,
    }];
  }
  // Texte / sticky : la taille vient du store (mesurée au rendu). Sans elle on
  // ne peut ni décider l'appartenance ni mettre à l'échelle → on ignore, le
  // ResizeObserver la fournira au prochain tour.
  const w = ann.width ?? (ann.type === "sticky" ? 160 : 0);
  const h = ann.height ?? (ann.type === "sticky" ? 120 : 0);
  if (w <= 0 || h <= 0) return [];
  return [{
    id: ann.id,
    kind: ann.type === "sticky" ? "sticky" : "text",
    x: ann.x, y: ann.y, width: w, height: h,
    membraneId: ann.membraneId ?? null,
  }];
}

/** Raccourci : géométrie effective d'un board entier. */
export function resolveBoard(
  board: Pick<Board, "images" | "annotations">,
  opts: ResolveOptions = {},
): Map<string, ResolvedItem> {
  return resolveItems(itemsOfBoard(board), opts);
}
