// ────────────────────────────────────────────────────────────────────────────
// PICK-1 — Ordre de priorité de sélection au clic (géométrie PURE, testable).
//
// PROBLÈME. Chaque type d'élément vit dans sa propre couche DOM (Pixi pour les
// images, SVG pour membranes/dossiers/flèches, HTML pour textes/stickies). Le
// z-order du DOM décidait donc seul de QUI reçoit le clic : une membrane, qui
// couvre une grande surface et se trouve au-dessus du canvas Pixi, avalait tous
// les clics sur les images qu'elle contient. Idem pour les poignées de resize,
// inatteignables dès qu'une couche passait par-dessus.
//
// SOLUTION. Un seul test de collision, en coordonnées MONDE, qui produit la
// liste ORDONNÉE de tout ce qui se trouve sous le curseur. Le rang ne vient plus
// de l'empilement graphique mais de l'INTENTION : plus une cible est petite,
// précise et « voulue », plus elle est prioritaire.
//
//   0  poignée de redimensionnement (sélection courante) ← absolue
//   10 bord de membrane  (la bande autour du pointillé)
//   11 bord / en-tête de dossier
//   20 flèche            (tracé fin, difficile à viser)
//   30 image
//   40 sticky
//   50 texte             ← toujours dernier parmi les contenus
//   60 corps de dossier
//   70 corps de membrane ← un conteneur ne gagne jamais sur son contenu
//
// LE TEXTE EST TOUJOURS DERNIER, et c'est structurel : un double-clic sur un
// bloc texte ouvre l'édition. Il ne peut donc pas être une étape INTERMÉDIAIRE
// du cycle (le 2e clic serait mangé par l'éditeur). D'où `terminal: true` : le
// cycle s'arrête quand il atteint un bloc éditable et le double-clic reprend la
// main. Même raison pour les stickies.
//
// CYCLE. Re-cliquer sans bouger passe à la cible suivante (« switch priority »),
// façon sélection par profondeur des logiciels CAO. Voir `pickWithCycle`.
// ────────────────────────────────────────────────────────────────────────────

import type { Annotation, BoardImage, CanvasFolder } from "../types";

// ════════════════════════════════════════════════════════════════════════════
// Rangs & réglages
// ════════════════════════════════════════════════════════════════════════════

export const PICK_RANK = {
  /** Poignée de resize de la sélection courante — gagne toujours. */
  HANDLE: 0,
  /** Bande autour du pointillé d'une membrane (dedans ET dehors). */
  MEMBRANE_EDGE: 10,
  /** Bordure + bandeau d'en-tête d'un dossier. */
  FOLDER_EDGE: 11,
  /** Tracé d'une flèche (cible naturelle du DOM, cf. ArrowSvgLayer). */
  ARROW: 20,
  IMAGE: 30,
  STICKY: 40,
  TEXT: 50,
  FOLDER_BODY: 60,
  MEMBRANE_BODY: 70,
} as const;

export const PICK = {
  /** Rayon de préhension d'une poignée, en PIXELS ÉCRAN (constant au zoom). */
  HANDLE_SLOP_PX: 18,
  /** Plafond : une poignée ne mange jamais plus que ce ratio du petit côté de
   *  l'élément — sinon un bloc minuscule à faible zoom deviendrait indéplaçable
   *  (ses 4 poignées couvriraient toute la boîte). */
  HANDLE_SLOP_MAX_RATIO: 0.35,
  /** Plancher de préhension quand le plafond ci-dessus mord. */
  HANDLE_SLOP_MIN_PX: 6,
  /** Demi-largeur de la bande « bord » d'un conteneur, en pixels écran. */
  EDGE_BAND_PX: 14,
  /** Hauteur du bandeau d'en-tête d'un dossier (monde) — cf. FolderSvgLayer. */
  FOLDER_HEADER: 38,
  /** Bande au-dessus d'une membrane où s'affiche son étiquette (monde). */
  MEMBRANE_LABEL_BAND: 30,
  /** Tolérance de « je n'ai pas bougé la souris » entre 2 clics (pixels écran). */
  CYCLE_RADIUS_PX: 8,
  /** Au-delà, le cycle est oublié : le clic repart au rang le plus prioritaire. */
  CYCLE_TTL_MS: 2500,
} as const;

export type PickOwner = "image" | "annotation" | "membrane" | "folder" | "arrow";

export type PickKind =
  | "handle"
  | "membrane-edge"
  | "membrane-body"
  | "folder-edge"
  | "folder-body"
  | "arrow"
  | "image"
  | "sticky"
  | "text";

export interface PickCandidate {
  /** Couche propriétaire — sert à router le clic vers le bon handler. */
  owner: PickOwner;
  id: string;
  kind: PickKind;
  rank: number;
  /** Ordre de peinture dans sa liste source ; à rang égal, le dessus gagne. */
  z: number;
  /** Poignée visée ("tl" | "tr" | "bl" | "br") — absent pour un corps. */
  corner?: string;
  /** Distance monde au point cliqué (poignées uniquement, 0 sinon). */
  dist: number;
  /** Un 2e clic dessus ouvre un éditeur → terminus du cycle. */
  terminal?: boolean;
}

// ════════════════════════════════════════════════════════════════════════════
// Géométrie
// ════════════════════════════════════════════════════════════════════════════

/** Point dans un rectangle ancré en haut-gauche. */
export function inRect(px: number, py: number, x: number, y: number, w: number, h: number): boolean {
  return px >= x && px <= x + w && py >= y && py <= y + h;
}

/** Point dans un rectangle ancré au CENTRE et pivoté de `rot` radians. */
export function inRotatedBox(
  px: number, py: number,
  cx: number, cy: number, w: number, h: number, rot: number,
): boolean {
  let dx = px - cx;
  let dy = py - cy;
  if (rot) {
    const c = Math.cos(-rot);
    const s = Math.sin(-rot);
    const rx = dx * c - dy * s;
    const ry = dx * s + dy * c;
    dx = rx;
    dy = ry;
  }
  return Math.abs(dx) <= w / 2 && Math.abs(dy) <= h / 2;
}

/**
 * Point dans la BANDE de bord d'un rectangle : entre le contour élargi de
 * `band` vers l'extérieur et le contour rétréci de `band` vers l'intérieur.
 * Si le rectangle est plus petit que la bande, tout son intérieur compte comme
 * bord (une membrane minuscule à l'écran se saisit n'importe où).
 */
export function onRectEdge(
  px: number, py: number,
  x: number, y: number, w: number, h: number,
  band: number,
): boolean {
  const outer = px >= x - band && px <= x + w + band && py >= y - band && py <= y + h + band;
  if (!outer) return false;
  if (w <= band * 2 || h <= band * 2) return true;
  const inner = px >= x + band && px <= x + w - band && py >= y + band && py <= y + h - band;
  return !inner;
}

/** Les 4 coins d'un rectangle ancré en haut-gauche, avec leur identifiant. */
function cornersOfRect(x: number, y: number, w: number, h: number): Array<[string, number, number]> {
  return [
    ["tl", x, y],
    ["tr", x + w, y],
    ["bl", x, y + h],
    ["br", x + w, y + h],
  ];
}

/**
 * Rayon de préhension d'une poignée en unités MONDE. Constant à l'écran
 * (`HANDLE_SLOP_PX / scale`) mais borné à une fraction du petit côté de
 * l'élément : sur un bloc de 30 px à l'écran, quatre poignées de 36 px
 * couvriraient toute la boîte et on ne pourrait plus la déplacer.
 */
export function handleSlopWorld(scale: number, boxW: number, boxH: number): number {
  const s = Math.max(1e-6, scale);
  const wanted = PICK.HANDLE_SLOP_PX / s;
  const cap = Math.max(
    PICK.HANDLE_SLOP_MIN_PX / s,
    Math.min(Math.abs(boxW), Math.abs(boxH)) * PICK.HANDLE_SLOP_MAX_RATIO,
  );
  return Math.min(wanted, cap);
}

// ════════════════════════════════════════════════════════════════════════════
// Collecte des candidats
// ════════════════════════════════════════════════════════════════════════════

export interface PickInput {
  /** Point cliqué, en coordonnées monde. */
  wx: number;
  wy: number;
  /** Échelle du viewport (world.scale.x) — convertit les tolérances écran↔monde. */
  scale: number;
  images: BoardImage[];
  annotations: Annotation[];
  folders: CanvasFolder[];
  selectedImageIds: string[];
  selectedAnnotationIds: string[];
  selectedFolderId: string | null;
  /** Flèche sous le curseur, résolue par le DOM (son tracé exact vit dans le SVG). */
  arrowId?: string | null;
}

/** Ajoute les poignées de coin d'une boîte sélectionnée, si le clic tombe dessus. */
function pushHandles(
  out: PickCandidate[],
  owner: PickOwner, id: string, z: number,
  corners: Array<[string, number, number]>,
  wx: number, wy: number, slop: number,
) {
  for (const [corner, cx, cy] of corners) {
    const d = Math.hypot(wx - cx, wy - cy);
    if (d <= slop) {
      out.push({ owner, id, kind: "handle", rank: PICK_RANK.HANDLE, z, corner, dist: d });
    }
  }
}

/**
 * Tout ce qui se trouve sous `(wx, wy)`, du plus prioritaire au moins
 * prioritaire. Chaque élément produit AU PLUS un candidat (bord OU corps), plus
 * ses poignées de resize qui sont des candidats à part entière.
 */
export function collectCandidates(input: PickInput): PickCandidate[] {
  const { wx, wy, scale, images, annotations, folders } = input;
  const selImg = new Set(input.selectedImageIds);
  const selAnn = new Set(input.selectedAnnotationIds);
  const band = PICK.EDGE_BAND_PX / Math.max(1e-6, scale);
  const out: PickCandidate[] = [];

  // ── Images ───────────────────────────────────────────────────────────────
  // Une image verrouillée n'est pas cliquable (cf. attachSpriteEvents) : elle ne
  // doit donc pas occuper une case du cycle.
  images.forEach((img, z) => {
    if (img.locked) return;
    if (selImg.has(img.id)) {
      const hw = img.width / 2;
      const hh = img.height / 2;
      const rot = img.rotation || 0;
      const c = Math.cos(rot);
      const s = Math.sin(rot);
      const offsets: Array<[string, number, number]> = [
        ["tl", -hw, -hh], ["tr", hw, -hh], ["bl", -hw, hh], ["br", hw, hh],
      ];
      const corners = offsets.map(([k, ox, oy]) =>
        [k, img.x + ox * c - oy * s, img.y + ox * s + oy * c] as [string, number, number]);
      pushHandles(out, "image", img.id, z, corners, wx, wy,
        handleSlopWorld(scale, img.width, img.height));
    }
    if (inRotatedBox(wx, wy, img.x, img.y, img.width, img.height, img.rotation || 0)) {
      out.push({ owner: "image", id: img.id, kind: "image", rank: PICK_RANK.IMAGE, z, dist: 0 });
    }
  });

  // ── Annotations (texte / sticky / membrane) ──────────────────────────────
  annotations.forEach((ann, z) => {
    if (ann.type === "arrow") return; // le tracé exact appartient à ArrowSvgLayer

    if (ann.type === "membrane") {
      const w = ann.width;
      const h = ann.height;
      if (selAnn.has(ann.id)) {
        pushHandles(out, "membrane", ann.id, z, cornersOfRect(ann.x, ann.y, w, h),
          wx, wy, handleSlopWorld(scale, w, h));
      }
      // L'étiquette est peinte AU-DESSUS du bord haut : elle appartient au bord.
      const onLabel = !!ann.text
        && wx >= ann.x && wx <= ann.x + w
        && wy >= ann.y - PICK.MEMBRANE_LABEL_BAND && wy <= ann.y;
      if (onLabel || onRectEdge(wx, wy, ann.x, ann.y, w, h, band)) {
        out.push({ owner: "membrane", id: ann.id, kind: "membrane-edge", rank: PICK_RANK.MEMBRANE_EDGE, z, dist: 0 });
      } else if (inRect(wx, wy, ann.x, ann.y, w, h)) {
        out.push({ owner: "membrane", id: ann.id, kind: "membrane-body", rank: PICK_RANK.MEMBRANE_BODY, z, dist: 0 });
      }
      return;
    }

    // Texte / sticky — boîte ancrée en haut-gauche. La taille vient du store
    // (mesurée par le ResizeObserver de HtmlAnnotationLayer) ; sans elle, pas de
    // test de collision possible → on saute.
    const w = ann.width ?? (ann.type === "sticky" ? 160 : 0);
    const h = ann.height ?? (ann.type === "sticky" ? 120 : 0);
    if (w <= 0 || h <= 0) return;
    if (selAnn.has(ann.id)) {
      pushHandles(out, "annotation", ann.id, z, cornersOfRect(ann.x, ann.y, w, h),
        wx, wy, handleSlopWorld(scale, w, h));
    }
    if (inRect(wx, wy, ann.x, ann.y, w, h)) {
      out.push({
        owner: "annotation", id: ann.id,
        kind: ann.type === "sticky" ? "sticky" : "text",
        rank: ann.type === "sticky" ? PICK_RANK.STICKY : PICK_RANK.TEXT,
        z, dist: 0,
        // Double-clic = édition en place → terminus du cycle (cf. en-tête).
        terminal: true,
      });
    }
  });

  // ── Dossiers ─────────────────────────────────────────────────────────────
  folders.forEach((f, z) => {
    if (input.selectedFolderId === f.id) {
      // FolderSvgLayer n'expose que la poignée bas-droite.
      const br: Array<[string, number, number]> = [["br", f.x + f.width, f.y + f.height]];
      pushHandles(out, "folder", f.id, z, br, wx, wy, handleSlopWorld(scale, f.width, f.height));
    }
    const onHeader = wx >= f.x && wx <= f.x + f.width
      && wy >= f.y && wy <= f.y + PICK.FOLDER_HEADER;
    if (onHeader || onRectEdge(wx, wy, f.x, f.y, f.width, f.height, band)) {
      out.push({ owner: "folder", id: f.id, kind: "folder-edge", rank: PICK_RANK.FOLDER_EDGE, z, dist: 0 });
    } else if (inRect(wx, wy, f.x, f.y, f.width, f.height)) {
      out.push({ owner: "folder", id: f.id, kind: "folder-body", rank: PICK_RANK.FOLDER_BODY, z, dist: 0 });
    }
  });

  // ── Flèche ───────────────────────────────────────────────────────────────
  // Son tracé (courbes, pathfinding, waypoints) est calculé par ArrowSvgLayer ;
  // on ne le refait pas ici. Si le DOM dit que le curseur est dessus, elle entre
  // dans la liste à son rang.
  if (input.arrowId) {
    out.push({ owner: "arrow", id: input.arrowId, kind: "arrow", rank: PICK_RANK.ARROW, z: 0, dist: 0 });
  }

  // Rang croissant ; à rang égal, la poignée la plus proche puis l'élément peint
  // le plus haut (dernier de sa liste). L'ordre doit être TOTAL et STABLE, sinon
  // la signature du cycle changerait d'un clic à l'autre.
  out.sort((a, b) =>
    a.rank - b.rank
    || a.dist - b.dist
    || b.z - a.z
    || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0)
    || (a.corner ?? "").localeCompare(b.corner ?? ""),
  );
  return out;
}

// ════════════════════════════════════════════════════════════════════════════
// Cycle « re-clic = cible suivante »
// ════════════════════════════════════════════════════════════════════════════

export interface CycleState {
  /** Position écran du clic précédent. */
  sx: number;
  sy: number;
  /** Empreinte des candidats cyclables — si elle change, le cycle est caduc. */
  sig: string;
  index: number;
  t: number;
}

export interface PickOptions {
  /** Alt : ignore la priorité absolue des poignées et force le pas suivant. */
  alt?: boolean;
  /** Ctrl/Shift/Cmd (multi-sélection) : jamais de cycle, on prend le 1er rang. */
  multi?: boolean;
}

function signatureOf(cands: PickCandidate[]): string {
  return cands.map((c) => `${c.owner}:${c.id}`).join("|");
}

/**
 * Choisit la cible d'un clic, en tenant compte du clic précédent.
 *
 *   • Clic « frais » (souris déplacée, ou trop de temps écoulé) → rang le plus
 *     prioritaire. Les POIGNÉES gagnent alors toujours : c'est le geste « je
 *     veux redimensionner », il ne doit jamais rater.
 *   • Re-clic au même endroit → cible suivante dans l'ordre de priorité, en
 *     boucle. Les poignées sortent du cycle (sinon un 2e clic sur une poignée
 *     lâcherait la sélection qu'on vient tout juste de saisir) ; Alt sert
 *     d'échappatoire pour atteindre ce qui se cache dessous.
 *   • Le cycle s'ARRÊTE sur un bloc éditable (texte, sticky) : le 2e clic doit
 *     rester un double-clic → mode édition.
 */
export function pickWithCycle(
  candidates: PickCandidate[],
  prev: CycleState | null,
  sx: number,
  sy: number,
  now: number,
  opts: PickOptions = {},
): { picked: PickCandidate | null; cycle: CycleState | null } {
  if (candidates.length === 0) return { picked: null, cycle: null };

  const handles = candidates.filter((c) => c.rank === PICK_RANK.HANDLE);
  const cyclable = candidates.filter((c) => c.rank !== PICK_RANK.HANDLE);
  const sig = signatureOf(cyclable);

  const continuing = !!prev
    && prev.sig === sig
    && now - prev.t <= PICK.CYCLE_TTL_MS
    && Math.hypot(sx - prev.sx, sy - prev.sy) <= PICK.CYCLE_RADIUS_PX;

  // Poignée : priorité absolue, et HORS cycle — on renvoie `cycle: null` pour
  // que le clic suivant reparte à zéro et retombe encore sur la poignée.
  if (handles.length > 0 && !opts.alt && !continuing) {
    return { picked: handles[0], cycle: null };
  }
  if (cyclable.length === 0) {
    return handles.length > 0 ? { picked: handles[0], cycle: null } : { picked: null, cycle: null };
  }

  let index = 0;
  if (continuing && !opts.multi && prev) {
    const at = Math.min(prev.index, cyclable.length - 1);
    const stay = cyclable[at]?.terminal && !opts.alt;
    index = stay ? at : (at + 1) % cyclable.length;
  }
  return { picked: cyclable[index], cycle: { sx, sy, sig, index, t: now } };
}
