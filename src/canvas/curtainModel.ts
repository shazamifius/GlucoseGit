// ────────────────────────────────────────────────────────────────────────────
// MEMB-3 — Rideaux : données, propriété et permissions.
//
// `curtainPanel` sait comment le panneau se déploie ; ce module sait ce qu'il
// contient et qui a le droit d'y toucher.
//
// Un rideau appartient à UNE membrane et à UNE personne. Il n'en existe jamais
// par défaut : on en crée un, explicitement, et c'est toujours le sien.
//
// LES TROIS AXES tiennent en deux champs, parce que le troisième se déduit :
//
//   visibility  private  — les autres ne le voient même pas
//               shared   — ils le voient
//   editable    owner    — seul le propriétaire y touche
//               everyone — tout le monde peut y déposer
//
// Ce qui donne les trois usages nommés dans la maquette :
//
//   private            → un CARNET  (invisible, réservé — `editable` est alors
//                                    sans objet, personne d'autre n'y accède)
//   shared + owner     → une VITRINE (on regarde, on ne touche pas)
//   shared + everyone  → un ATELIER  (chacun y dépose)
//
// Un rideau privé est réservé par construction : c'est pour ça qu'il n'y a pas
// de troisième champ. `canEdit` le vérifie plutôt que de faire confiance à la
// combinaison stockée — un document partagé peut arriver dans n'importe quel
// état, y compris incohérent.
// ────────────────────────────────────────────────────────────────────────────

import { nanoid } from "../utils/nanoid";
import { CURTAIN, normalizeConfig, type CurtainConfig } from "./curtainPanel";

import type {
  Annotation, CurtainEditable, CurtainNote, CurtainVisibility, MembraneCurtain,
} from "../types";

export type { CurtainEditable, CurtainNote, CurtainVisibility, MembraneCurtain };

/** Longueur d'une note. Au-delà, ce n'est plus une note, c'est un document. */
export const MAX_NOTE_LENGTH = 2000;

// ════════════════════════════════════════════════════════════════════════════
// Création
// ════════════════════════════════════════════════════════════════════════════

export interface CurtainOwner {
  id: string;
  name: string;
  color: string;
}

/**
 * Un rideau neuf, privé par défaut.
 *
 * Privé et pas partagé : « rideau PERSONNEL » est le mot d'origine, et il vaut
 * mieux avoir à ouvrir volontairement le sien que découvrir après coup que tout
 * le monde lisait ses brouillons.
 */
export function createCurtain(owner: CurtainOwner, now: number = Date.now()): MembraneCurtain {
  return {
    id: nanoid(),
    ownerId: owner.id,
    ownerName: owner.name,
    ownerColor: owner.color,
    visibility: "private",
    editable: "owner",
    notes: [],
    createdAt: now,
  };
}

export function createNote(text: string, now: number = Date.now()): CurtainNote {
  return { id: nanoid(), text: sanitizeNoteText(text), createdAt: now };
}

/** Texte de note utilisable : borné, sans blancs de bord. */
/**
 * MEMB-7 — Contenu de départ du board d'un rideau, à partir de ses notes.
 *
 * Le rideau ne contenait que des notes texte ; il devient un board. Ces notes
 * ne sont pas jetées : chacune redevient un bloc de texte ordinaire, empilé en
 * colonne. Une fois posées sur le board ce sont des blocs comme les autres —
 * déplaçables, reliables, supprimables, annulables.
 *
 * Les notes vides sont écartées : elles ne décrivaient qu'un champ de saisie en
 * attente, pas un contenu.
 */
export function notesToAnnotations(
  notes: readonly CurtainNote[],
  now: number = Date.now(),
): Annotation[] {
  const out: Annotation[] = [];
  let y = CURTAIN_BOARD.MARGIN;
  for (const n of notes) {
    const text = sanitizeNoteText(n.text);
    if (!text) continue;
    out.push({
      id: `${n.id}-b`,
      type: "text",
      x: CURTAIN_BOARD.MARGIN,
      y,
      text,
      width: CURTAIN_BOARD.BLOCK_W,
      fontSize: 14,
    } as Annotation);
    y += CURTAIN_BOARD.BLOCK_STEP;
  }
  void now;
  return out;
}

/** Mise en page du contenu repris des notes. */
export const CURTAIN_BOARD = {
  MARGIN: 40,
  BLOCK_W: 320,
  BLOCK_STEP: 88,
} as const;

export function sanitizeNoteText(raw: string): string {
  return raw.replace(/\r\n/g, "\n").trim().slice(0, MAX_NOTE_LENGTH);
}

// ════════════════════════════════════════════════════════════════════════════
// Détachement (obligatoire avant toute écriture)
// ════════════════════════════════════════════════════════════════════════════

/**
 * Copie DÉTACHÉE d'un rideau — à passer systématiquement avant d'écrire.
 *
 * Automerge refuse qu'un objet déjà présent dans le document y soit réinséré :
 * « Cannot create a reference to an existing document object ». Or réécrire la
 * liste des rideaux réinsère fatalement ceux qu'on n'a PAS touchés, et la liste
 * des notes celles qu'on n'a pas modifiées. Sans cette recopie, l'application
 * lève dès qu'il y a deux rideaux ou deux notes — c'est-à-dire tout de suite,
 * et seulement en collaboration, donc au pire moment.
 *
 * Recopie champ par champ plutôt que `structuredClone` : la forme reste lisible,
 * et un champ ajouté un jour sans être recopié se voit à la relecture.
 */
export function detachCurtain(c: MembraneCurtain): MembraneCurtain {
  const out: MembraneCurtain = {
    id: c.id,
    ownerId: c.ownerId,
    ownerName: c.ownerName,
    ownerColor: c.ownerColor,
    visibility: c.visibility,
    editable: c.editable,
    notes: (c.notes ?? []).map((n) => ({ id: n.id, text: n.text, createdAt: n.createdAt })),
    createdAt: c.createdAt,
  };
  // Champs optionnels : présents seulement s'ils le sont, pour ne pas écrire
  // d'`undefined` dans le document.
  //
  // ⚠️ CETTE LISTE EST EXHAUSTIVE PAR CONSTRUCTION. Recopier champ par champ
  // est ce qui rend l'écriture détachable (Automerge refuse qu'un objet déjà
  // présent dans le document y soit réinséré), mais c'est aussi un piège :
  // un champ ajouté au type et oublié ici est SILENCIEUSEMENT effacé à chaque
  // écriture. C'est arrivé avec `boardId`. Tout ajout à `MembraneCurtain` doit
  // passer par ici, et un test le vérifie.
  if (c.collapsedRatio !== undefined) out.collapsedRatio = c.collapsedRatio;
  if (c.expandedRatio !== undefined) out.expandedRatio = c.expandedRatio;
  if (c.boardId !== undefined) out.boardId = c.boardId;
  return out;
}

/** Copie détachée d'une liste entière. */
export function detachCurtains(list: readonly MembraneCurtain[]): MembraneCurtain[] {
  return list.map(detachCurtain);
}

// ════════════════════════════════════════════════════════════════════════════
// Permissions
// ════════════════════════════════════════════════════════════════════════════

/** Le propriétaire voit toujours le sien ; les autres, seulement s'il est partagé. */
export function canSee(curtain: MembraneCurtain, userId: string): boolean {
  return curtain.ownerId === userId || curtain.visibility === "shared";
}

/**
 * Qui peut modifier. Le propriétaire, toujours. Les autres, seulement si le
 * rideau est À LA FOIS partagé et ouvert à tous — on ne se fie pas à `editable`
 * seul, un document collaboratif peut arriver dans un état incohérent
 * (« privé mais modifiable par tous » ne doit rien vouloir dire).
 */
export function canEdit(curtain: MembraneCurtain, userId: string): boolean {
  if (curtain.ownerId === userId) return true;
  return curtain.visibility === "shared" && curtain.editable === "everyone";
}

/** Étiquette de l'usage courant, telle qu'elle s'affiche dans le panneau. */
export function curtainKind(curtain: MembraneCurtain): "carnet" | "vitrine" | "atelier" {
  if (curtain.visibility === "private") return "carnet";
  return curtain.editable === "everyone" ? "atelier" : "vitrine";
}

/** Les rideaux qu'une personne a le droit de voir, propriétaire d'abord.
 *  Le sien en tête : c'est celui qu'on cherche en ouvrant le panneau. */
export function visibleCurtains(curtains: MembraneCurtain[], userId: string): MembraneCurtain[] {
  return curtains
    .filter((c) => canSee(c, userId))
    .sort((a, b) => {
      const mine = Number(b.ownerId === userId) - Number(a.ownerId === userId);
      return mine !== 0 ? mine : a.createdAt - b.createdAt;
    });
}

// ════════════════════════════════════════════════════════════════════════════
// Proportions
// ════════════════════════════════════════════════════════════════════════════

/** Réglages du panneau pour ce rideau, bornés (cf. `curtainPanel`). */
export function configOf(curtain: MembraneCurtain | null | undefined): CurtainConfig {
  return normalizeConfig({
    collapsed: curtain?.collapsedRatio,
    expanded: curtain?.expandedRatio,
  });
}

/** Élargit ou rétrécit le panneau déployé d'un cran, en restant dans les bornes. */
export function stepExpanded(current: number | undefined, delta: number): number {
  const from = normalizeConfig({ expanded: current }).expanded;
  return normalizeConfig({ expanded: from + delta }).expanded;
}

export const EXPAND_STEP = 0.1;
export { CURTAIN };
