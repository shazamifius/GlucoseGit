// ────────────────────────────────────────────────────────────────────────────
// MEMB-4 — Le mode étiré, à l'échelle d'un board.
//
// PROBLÈME. `stretchPlan` (dans `membraneSpace`) sait déjà, pour UNE membrane,
// jusqu'où elle a le droit de pousser et qui l'en empêche. Il lui manquait un
// appelant, et cet appelant doit trancher deux questions que la géométrie seule
// ne pose pas :
//
//   ① QUI compte comme obstacle ?
//   ② DANS QUEL ORDRE traiter des membranes imbriquées ?
//
// ① LES OBSTACLES SONT LES FRÈRES, ET RIEN D'AUTRE. `stretchPlan` compare des
// boîtes NATURELLES. Or deux éléments n'ont de coordonnées comparables que
// s'ils vivent dans le même repère — c'est-à-dire s'ils ont la même membrane
// parente. Un élément posé au fond d'une AUTRE membrane a des coordonnées
// naturelles qui ne disent rien de sa place à l'écran (sa membrane peut le
// réduire de moitié) : le confronter à notre boîte comparerait deux repères
// différents. Ce n'est pas une perte : cette autre membrane est elle-même un
// frère, et c'est SA boîte qui fait obstacle — au bon niveau, dans le bon
// repère. Cette règle exclut gratuitement les trois cas qu'il aurait fallu
// écarter à la main : la membrane elle-même, sa descendance (qui grandit avec
// elle) et ses ancêtres (qui la contiennent, donc la chevauchent toujours).
//
// ② ON TRAITE LES PLUS PROFONDES D'ABORD. Une membrane étirée à l'intérieur
// d'une autre grandit, et sa nouvelle boîte fait partie du contenu que la
// parente doit contenir. Mesurer la parente avant que l'enfant ait poussé la
// ferait grandir d'un cran de retard — visible à l'usage comme un « il faut
// bouger deux fois pour que ça s'ajuste ». On descend donc par profondeur
// décroissante, en reportant chaque croissance dans la copie de travail.
//
// CE QUI RESTE VRAI D'AILLEURS. On applique `allowed`, JAMAIS `desired` : la
// membrane bute sur l'obstacle, elle ne le recouvre pas et ne le capture pas
// (décision produit). Et une membrane ne RÉTRÉCIT jamais toute seule — un
// utilisateur qui l'a agrandie à la main a exprimé une intention que retirer du
// contenu ne doit pas effacer.
//
// QUAND C'EST APPELÉ. À des instants DISCRETS — fin d'un glisser, conversion de
// mode — jamais en continu. Même raison que pour l'appartenance (cf. l'en-tête
// de `membraneSpace`) : une géométrie qui se réécrit à chaque frame de drag
// noierait l'annulation et ferait fuir la cible sous le curseur.
// ────────────────────────────────────────────────────────────────────────────

import { MEMBRANE_SPACE, parentMap, stretchPlan, type SpaceItem } from "./membraneSpace";

/** Ce qu'une membrane étirée doit devenir, et ce qui l'a empêchée d'aller plus loin. */
export interface StretchOutcome {
  membraneId: string;
  /** Taille à écrire — c'est `allowed`, jamais `desired`. */
  width: number;
  height: number;
  /** La boîte a-t-elle effectivement changé ? */
  grew: boolean;
  /** La croissance a-t-elle été rabotée par un obstacle ? */
  blocked: boolean;
  /** Ids des éléments à entourer, dans l'ordre de rencontre. */
  blockerIds: string[];
}

/** Profondeur d'imbrication d'un élément (0 = à la racine du board). */
function depthOf(id: string, parents: Map<string, string>): number {
  const seen = new Set<string>([id]);
  let d = 0;
  let cur = parents.get(id);
  while (cur && !seen.has(cur)) {
    seen.add(cur);
    d += 1;
    cur = parents.get(cur);
  }
  return d;
}

/**
 * Croissance de TOUTES les membranes étirées d'un board.
 *
 * Fonction pure : `items` n'est pas modifié (on travaille sur une copie), et
 * seules les membranes qui bougent ou qui butent sont rendues — une membrane
 * étirée déjà à la bonne taille et sans obstacle ne produit rien, donc
 * n'entraîne aucune écriture dans le document.
 */
export function planBoardStretch(items: SpaceItem[]): StretchOutcome[] {
  const parents = parentMap(items);
  const work: SpaceItem[] = items.map((i) => ({ ...i }));
  const byId = new Map(work.map((i) => [i.id, i]));

  const targets = work
    .filter((i) => i.kind === "membrane" && i.mode === "stretched")
    .sort((a, b) => depthOf(b.id, parents) - depthOf(a.id, parents));

  const out: StretchOutcome[] = [];
  for (const m of targets) {
    const frame = parents.get(m.id) ?? null;
    const children: SpaceItem[] = [];
    const foreigners: SpaceItem[] = [];
    for (const it of work) {
      if (it.id === m.id) continue;
      const p = parents.get(it.id) ?? null;
      if (p === m.id) children.push(it);
      else if (p === frame) foreigners.push(it);
    }

    const plan = stretchPlan(m, children, foreigners);
    const width = plan.allowed.width;
    const height = plan.allowed.height;
    const grew = width !== m.width || height !== m.height;
    if (!grew && !plan.blocked) continue;

    // Reporté dans la copie de travail : la membrane parente, traitée juste
    // après, doit mesurer la boîte que celle-ci vient de prendre.
    const live = byId.get(m.id);
    if (live) { live.width = width; live.height = height; }

    out.push({
      membraneId: m.id,
      width, height, grew,
      blocked: plan.blocked,
      blockerIds: plan.blockers.map((b) => b.id),
    });
  }
  return out;
}

/** Marge laissée entre le contenu et le bord — réexportée pour l'interface. */
export const STRETCH_PADDING = MEMBRANE_SPACE.STRETCH_PADDING;
