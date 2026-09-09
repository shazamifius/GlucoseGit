// ────────────────────────────────────────────────────────────────────────────
// MEMB-6 — Le passage fluide : une image qui entre dans une membrane
// minimisée rétrécit, elle ne saute pas.
//
// PROBLÈME. L'échelle est DÉDUITE (cf. `membraneSpace`), donc à l'instant où
// l'appartenance est écrite, la géométrie effective change d'un coup : une image
// de 800 px devient 200 px entre deux images à l'écran. Fonctionnellement juste,
// visuellement brutal — on ne voit pas ce qui s'est passé, l'objet a disparu et
// un autre est apparu ailleurs.
//
// SOLUTION, ET SURTOUT OÙ ELLE SE BRANCHE. On n'anime ni les sprites, ni les
// textes, ni les membranes séparément : on interpole `geom`, la carte de
// géométrie effective, et TOUT ce qui en dépend suit — sprites Pixi, couche
// HTML, couche SVG, ancrage des flèches, test de collision au clic, culling.
// Une seule couture, comme pour le reste du module. En particulier le clic
// tombe pendant l'animation là où l'élément est VU, pas là où il finira.
//
// ① ON N'ANIME QUE CE QUI CHANGE D'ÉCHELLE. Un élément dont l'échelle est
// identique des deux côtés est repris TEL QUEL — au même objet près. C'est ce
// qui garantit qu'un glisser reste collé au curseur : déplacer ne change aucune
// échelle, donc n'anime rien, donc n'introduit aucun retard.
//
// ② LE DÉCLENCHEUR EST UN FAIT DISCRET, PAS UNE DIFFÉRENCE DE GÉOMÉTRIE.
// C'est le point non évident. Étirer une membrane minimisée à la poignée change
// son échelle À CHAQUE IMAGE : animer sur « l'échelle a changé » ferait traîner
// le contenu 200 ms derrière la poignée, et l'animation courrait après une
// cible qui bouge encore. On ne démarre donc que sur un changement
// d'APPARTENANCE ou de MODE — les deux seuls faits qui arrivent d'un coup, à
// un instant précis, sur décision de l'utilisateur. Le redimensionnement et le
// déplacement restent instantanés, et c'est ce qu'on veut.
//
// ③ ON N'ÉCRIT RIEN. L'animation vit entre les données et le rendu ; aucun
// champ n'est ajouté, rien n'est persisté, rien n'est envoyé aux pairs. Une
// animation interrompue (fermeture, undo, arrivée d'un pair) ne laisse donc
// aucune trace — au pire une image saute, ce qu'elle faisait déjà avant.
//
// ④ LE GARDE-FOU RESTE INTACT. Tant qu'aucune animation ne court, ce module
// n'est pas appelé : `geom` reste `null` quand rien n'est minimisé et le rendu
// reprend le chemin d'avant, au même objet près.
// ────────────────────────────────────────────────────────────────────────────

import {
  contentExtent, contentScale, parentMap,
  type ResolvedItem, type SpaceItem,
} from "./membraneSpace";

export const MEMBRANE_TWEEN = {
  /** Durée du passage. Assez long pour qu'on suive l'objet des yeux, assez
   *  court pour ne jamais donner l'impression d'attendre. */
  MS: 200,
} as const;

/** Adoucissement — le même que le cadrage du focus, pour une seule sensation. */
export function ease(p: number): number {
  const c = Math.min(1, Math.max(0, p));
  return 1 - (1 - c) ** 3;
}

/**
 * Empreinte des faits DISCRETS d'un board : qui appartient à qui, et dans quel
 * mode est chaque membrane.
 *
 * Elle ne bouge NI quand un élément se déplace, NI quand une membrane est
 * redimensionnée — ce qui est précisément ce qui rend le déclencheur sûr
 * (cf. ② dans l'en-tête). Les éléments libres n'y figurent pas : en créer un
 * ailleurs sur le board ne doit rien déclencher.
 */
export function membershipSignature(items: SpaceItem[]): string {
  const parts: string[] = [];
  for (const it of items) {
    if (it.membraneId) parts.push(`${it.id}>${it.membraneId}`);
    if (it.kind === "membrane") parts.push(`${it.id}=${it.mode ?? "classic"}`);
  }
  return parts.sort().join("|");
}

/**
 * Géométrie effective à l'échelle 1 — celle d'un board dont rien n'est réduit.
 *
 * Sert de point de départ (ou d'arrivée) quand l'autre côté de l'animation est
 * `null`, c'est-à-dire quand on vient de créer la PREMIÈRE membrane minimisée du
 * board ou de supprimer la dernière. Sans elle, ces deux transitions — les plus
 * spectaculaires — seraient les seules à sauter.
 *
 * Construite uniquement au démarrage d'une animation : le chemin rapide ne la
 * paie jamais.
 */
export function naturalGeom(items: SpaceItem[]): Map<string, ResolvedItem> {
  const members = parentMap(items);
  const out = new Map<string, ResolvedItem>();
  for (const it of items) {
    const r: ResolvedItem = {
      id: it.id, x: it.x, y: it.y, width: it.width, height: it.height,
      scale: 1, membraneId: members.get(it.id) ?? null,
    };
    if (it.kind === "membrane") r.contentScale = 1;
    out.set(it.id, r);
  }
  return out;
}

/**
 * Géométrie effective d'un board, JAMAIS `null`.
 *
 * `geom` vaut `null` quand rien n'est réduit (le garde-fou du chemin rapide) ;
 * une animation, elle, a besoin des deux bouts.
 */
export function geomOrNatural(
  items: SpaceItem[],
  geom: Map<string, ResolvedItem> | null,
): Map<string, ResolvedItem> {
  return geom ?? naturalGeom(items);
}

/**
 * Éléments dont l'ÉCHELLE change entre deux géométries — les seuls à animer.
 *
 * Un élément absent du départ (créé, ou rendu visible entre-temps) n'y figure
 * pas : on ne peut pas venir de nulle part, et le faire apparaître à sa place
 * finale est le comportement juste.
 */
export function tweenedIds(
  from: Map<string, ResolvedItem>,
  to: Map<string, ResolvedItem>,
): Set<string> {
  const out = new Set<string>();
  for (const [id, t] of to) {
    const f = from.get(id);
    if (f && f.scale !== t.scale) out.add(id);
  }
  return out;
}

/** Ce qu'il faut retenir pour jouer une animation : d'où l'on part, et sur qui. */
export interface TweenPlan {
  /** Géométrie de DÉPART, figée. L'arrivée, elle, reste lue en direct : si le
   *  board bouge pendant les 200 ms, l'animation converge vers l'état réel. */
  from: Map<string, ResolvedItem>;
  ids: Set<string>;
}

/**
 * Faut-il démarrer une animation, et laquelle ? `null` = non.
 *
 * TOUTE la décision est ici, et pas dans le composant, pour qu'elle soit
 * démontrable. Trois refus, chacun pour une raison distincte :
 *
 *   • `prev.sig === null` — c'est le premier rendu. Il n'y a pas d'« avant »,
 *     et animer l'ouverture d'un projet ferait ramper tout le board à l'écran.
 *   • les empreintes sont égales — rien de discret n'a bougé. C'est le cas de
 *     l'immense majorité des rendus : déplacement, redimensionnement, zoom.
 *   • aucune échelle ne change — l'appartenance a bien bougé, mais vers ou
 *     depuis une membrane classique, qui ne réduit rien. Il n'y a rien à voir.
 */
export function planTween(
  prev: { sig: string | null; items: SpaceItem[]; geom: Map<string, ResolvedItem> | null },
  next: { sig: string; items: SpaceItem[]; geom: Map<string, ResolvedItem> | null },
): TweenPlan | null {
  if (prev.sig === null || prev.sig === next.sig) return null;
  const from = geomOrNatural(prev.items, prev.geom);
  const ids = tweenedIds(from, geomOrNatural(next.items, next.geom));
  return ids.size === 0 ? null : { from, ids };
}

function lerp(a: number, b: number, p: number): number {
  return a + (b - a) * p;
}

/**
 * Géométrie intermédiaire à l'avancement `p` (déjà adouci), entre 0 et 1.
 *
 * Tout ce qui n'est pas dans `ids` est repris depuis `to` PAR RÉFÉRENCE : le
 * reste du board n'est ni recopié ni recalculé, et une identité d'objet
 * préservée évite des rendus inutiles en aval.
 *
 * L'appartenance et le mode ne s'interpolent pas — ce sont des faits, pas des
 * quantités : on prend ceux d'arrivée dès la première image.
 */
export function tweenGeom(
  from: Map<string, ResolvedItem>,
  to: Map<string, ResolvedItem>,
  ids: Set<string>,
  p: number,
): Map<string, ResolvedItem> {
  if (ids.size === 0 || p >= 1) return to;
  const out = new Map(to);
  for (const id of ids) {
    const f = from.get(id);
    const t = to.get(id);
    if (!f || !t) continue;
    const mid: ResolvedItem = {
      ...t,
      x: lerp(f.x, t.x, p),
      y: lerp(f.y, t.y, p),
      width: lerp(f.width, t.width, p),
      height: lerp(f.height, t.height, p),
      scale: lerp(f.scale, t.scale, p),
    };
    if (t.contentScale !== undefined || f.contentScale !== undefined) {
      mid.contentScale = lerp(f.contentScale ?? 1, t.contentScale ?? 1, p);
    }
    out.set(id, mid);
  }
  return out;
}

/**
 * Une membrane minimisée verra-t-elle son échelle changer si `count` éléments
 * de plus y entrent ? Utilitaire de raisonnement, employé par les tests pour
 * démontrer l'effet de bord voulu : déposer un élément dans une membrane
 * minimisée fait AUSSI rétrécir ce qui s'y trouvait déjà, et ce rétrécissement
 * doit s'animer comme le reste — sinon la moitié de la scène saute.
 */
export function scaleOfMembrane(membrane: SpaceItem, children: SpaceItem[]): number {
  return contentScale(membrane.mode ?? "classic", membrane, contentExtent(membrane, children));
}
