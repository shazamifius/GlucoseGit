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
//   0  poignée de redimensionnement (de la sélection courante) ← absolue
//   10 bord de conteneur  (pointillé d'une membrane, bordure/en-tête d'un dossier)
//   20 flèche             (tracé fin, difficile à viser)
//   30 image
//   40 sticky
//   50 texte              ← toujours dernier parmi les contenus
//   60 corps de conteneur ← un conteneur ne gagne jamais sur son contenu
//
// À rang égal entre deux CONTENEURS, le plus PETIT gagne : une membrane posée
// dans une membrane (ou dans un dossier) se saisit sans avoir à viser un bord.
// À rang égal entre deux contenus, c'est celui peint au-dessus qui gagne — pour
// des blocs opaques, l'ordre de peinture EST l'intuition de l'utilisateur.
//
// LE TEXTE EST TOUJOURS DERNIER, et c'est structurel : un double-clic sur un
// bloc texte ouvre l'édition. Il ne peut donc pas être une étape INTERMÉDIAIRE
// du cycle (le clic suivant serait mangé par l'éditeur). D'où `terminal: true` :
// le cycle s'arrête quand il atteint un bloc éditable et le double-clic reprend
// la main. Même raison pour les stickies.
//
// CYCLE (« switch priority »). Re-cliquer sans bouger passe à la cible suivante,
// façon sélection par profondeur des logiciels CAO. Le pas se joue au
// RELÂCHEMENT du re-clic, jamais à l'appui — voir `advanceOnRelease` pour la
// raison, qui est le cœur de l'ergonomie de ce module.
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
  /** Bordure + bandeau d'en-tête d'un dossier. Même rang qu'un bord de
   *  membrane : entre deux conteneurs, c'est la SURFACE qui départage, pas le
   *  type (cf. `area`). */
  FOLDER_EDGE: 10,
  /** Tracé d'une flèche (cible naturelle du DOM, cf. ArrowSvgLayer). */
  ARROW: 20,
  IMAGE: 30,
  STICKY: 40,
  TEXT: 50,
  /** Intérieur d'un conteneur — dernier, toujours. */
  MEMBRANE_BODY: 60,
  FOLDER_BODY: 60,
} as const;

export const PICK = {
  /** Rayon de préhension d'une poignée, en PIXELS ÉCRAN (constant au zoom).
   *  Très supérieur au carré dessiné (~9 px) : viser « pile » un point de 9 px
   *  était le principal irritant du redimensionnement. */
  HANDLE_SLOP_PX: 24,
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
  /** La poignée d'un dossier est dessinée EN RETRAIT du coin bas-droit
   *  (cf. FolderSvgLayer : translate(W-16, H-16), carré de 16 → centre à -8). */
  FOLDER_HANDLE_INSET: 8,
  /** Bande au-dessus d'une membrane où s'affiche son étiquette (monde). */
  MEMBRANE_LABEL_BAND: 30,
  /** Tolérance de « je n'ai pas bougé la souris » entre 2 clics (pixels écran).
   *  Sert aussi de seuil « ce geste était un glisser, pas un clic ». */
  CYCLE_RADIUS_PX: 8,
  /** Au-delà, le cycle est oublié : le clic repart au rang le plus prioritaire. */
  CYCLE_TTL_MS: 2500,
  /** En deçà, deux clics forment un DOUBLE-CLIC : le cycle ne bouge pas, sinon
   *  ouvrir un éditeur (ou entrer dans un dossier) changerait la sélection sous
   *  l'éditeur. Doit MAJORER le seuil le plus large des couches — 350 ms pour
   *  les textes/stickies/membranes, 400 ms pour les dossiers — de sorte qu'un
   *  geste interprété comme double-clic quelque part ne fasse jamais tourner le
   *  cycle ici. */
  DBLCLICK_MS: 400,
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
  /** Surface monde — départage deux CONTENEURS : le plus petit gagne. Vaut 0
   *  pour tout le reste, où l'ordre de peinture reste le bon critère. */
  area: number;
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
 * l'élément : sur un bloc de 30 px à l'écran, quatre poignées de 48 px
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

/** Curseur CSS d'une poignée de coin. */
export function handleCursor(corner: string): "nwse-resize" | "nesw-resize" {
  return corner === "tl" || corner === "br" ? "nwse-resize" : "nesw-resize";
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
  /** Élément que le DOM aurait naturellement touché (couche + id lus sur
   *  `data-pick-owner`). FILET DE SÉCURITÉ : l'arbitre ne doit JAMAIS rendre
   *  une cible MOINS cliquable que sans lui. Si notre géométrie ne la retrouve
   *  pas (taille pas encore mesurée par le ResizeObserver, forme non
   *  rectangulaire…), on la réinjecte à son rang naturel. */
  domHint?: { owner: PickOwner; id: string } | null;
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
      out.push({ owner, id, kind: "handle", rank: PICK_RANK.HANDLE, z, corner, dist: d, area: 0 });
    }
  }
}

/**
 * Poignées de resize sous le curseur, et rien d'autre. Extrait de la collecte
 * complète parce que le retour visuel de survol (curseur ↔↕) en a besoin à
 * chaque `pointermove` : ne sont testés que les éléments SÉLECTIONNÉS, donc le
 * coût reste négligeable même sur un board chargé.
 */
function collectHandles(input: PickInput, out: PickCandidate[]) {
  const { wx, wy, scale } = input;
  const selImg = new Set(input.selectedImageIds);
  const selAnn = new Set(input.selectedAnnotationIds);

  if (selImg.size > 0) {
    input.images.forEach((img, z) => {
      if (img.locked || !selImg.has(img.id)) return;
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
    });
  }

  if (selAnn.size > 0) {
    input.annotations.forEach((ann, z) => {
      if (ann.type === "arrow" || !selAnn.has(ann.id)) return;
      const w = ann.type === "membrane" ? ann.width : (ann.width ?? (ann.type === "sticky" ? 160 : 0));
      const h = ann.type === "membrane" ? ann.height : (ann.height ?? (ann.type === "sticky" ? 120 : 0));
      if (w <= 0 || h <= 0) return;
      pushHandles(out, ann.type === "membrane" ? "membrane" : "annotation", ann.id, z,
        cornersOfRect(ann.x, ann.y, w, h), wx, wy, handleSlopWorld(scale, w, h));
    });
  }

  if (input.selectedFolderId) {
    input.folders.forEach((f, z) => {
      if (input.selectedFolderId !== f.id) return;
      // FolderSvgLayer n'expose que la poignée bas-droite, dessinée en retrait.
      const k = PICK.FOLDER_HANDLE_INSET;
      const br: Array<[string, number, number]> = [["br", f.x + f.width - k, f.y + f.height - k]];
      pushHandles(out, "folder", f.id, z, br, wx, wy, handleSlopWorld(scale, f.width, f.height));
    });
  }
}

/** La poignée sous le curseur, ou `null`. Sert au curseur de survol. */
export function hitHandle(input: PickInput): PickCandidate | null {
  const out: PickCandidate[] = [];
  collectHandles(input, out);
  if (out.length === 0) return null;
  out.sort((a, b) => a.dist - b.dist);
  return out[0];
}

/** Réinjecte la cible naturelle du DOM si la géométrie ne l'a pas vue. */
function ensureDomHint(input: PickInput, out: PickCandidate[]) {
  const hint = input.domHint;
  if (!hint) return;
  if (out.some((c) => c.kind !== "handle" && c.owner === hint.owner && c.id === hint.id)) return;

  if (hint.owner === "image") {
    const z = input.images.findIndex((i) => i.id === hint.id);
    const img = z >= 0 ? input.images[z] : null;
    if (!img || img.locked) return;
    out.push({ owner: "image", id: img.id, kind: "image", rank: PICK_RANK.IMAGE, z, dist: 0, area: 0 });
    return;
  }

  if (hint.owner === "membrane" || hint.owner === "annotation") {
    const z = input.annotations.findIndex((a) => a.id === hint.id);
    const ann = z >= 0 ? input.annotations[z] : null;
    if (!ann || ann.type === "arrow") return;
    if (ann.type === "membrane") {
      out.push({
        owner: "membrane", id: ann.id, kind: "membrane-body",
        rank: PICK_RANK.MEMBRANE_BODY, z, dist: 0, area: Math.abs(ann.width * ann.height),
      });
    } else {
      out.push({
        owner: "annotation", id: ann.id,
        kind: ann.type === "sticky" ? "sticky" : "text",
        rank: ann.type === "sticky" ? PICK_RANK.STICKY : PICK_RANK.TEXT,
        z, dist: 0, area: 0, terminal: true,
      });
    }
    return;
  }

  if (hint.owner === "folder") {
    const z = input.folders.findIndex((f) => f.id === hint.id);
    const f = z >= 0 ? input.folders[z] : null;
    if (!f) return;
    out.push({
      owner: "folder", id: f.id, kind: "folder-body",
      rank: PICK_RANK.FOLDER_BODY, z, dist: 0, area: Math.abs(f.width * f.height),
    });
  }
}

/**
 * Tout ce qui se trouve sous `(wx, wy)`, du plus prioritaire au moins
 * prioritaire. Chaque élément produit AU PLUS un candidat (bord OU corps), plus
 * ses poignées de resize qui sont des candidats à part entière.
 */
export function collectCandidates(input: PickInput): PickCandidate[] {
  const { wx, wy, scale, images, annotations, folders } = input;
  const band = PICK.EDGE_BAND_PX / Math.max(1e-6, scale);
  const out: PickCandidate[] = [];

  collectHandles(input, out);

  // ── Images ───────────────────────────────────────────────────────────────
  // Une image verrouillée n'est pas cliquable (cf. attachSpriteEvents) : elle ne
  // doit donc pas occuper une case du cycle.
  images.forEach((img, z) => {
    if (img.locked) return;
    if (inRotatedBox(wx, wy, img.x, img.y, img.width, img.height, img.rotation || 0)) {
      out.push({ owner: "image", id: img.id, kind: "image", rank: PICK_RANK.IMAGE, z, dist: 0, area: 0 });
    }
  });

  // ── Annotations (texte / sticky / membrane) ──────────────────────────────
  annotations.forEach((ann, z) => {
    if (ann.type === "arrow") return; // le tracé exact appartient à ArrowSvgLayer

    if (ann.type === "membrane") {
      const w = ann.width;
      const h = ann.height;
      const area = Math.abs(w * h);
      // L'étiquette est peinte AU-DESSUS du bord haut : elle appartient au bord.
      const onLabel = !!ann.text
        && wx >= ann.x && wx <= ann.x + w
        && wy >= ann.y - PICK.MEMBRANE_LABEL_BAND && wy <= ann.y;
      if (onLabel || onRectEdge(wx, wy, ann.x, ann.y, w, h, band)) {
        out.push({ owner: "membrane", id: ann.id, kind: "membrane-edge", rank: PICK_RANK.MEMBRANE_EDGE, z, dist: 0, area });
      } else if (inRect(wx, wy, ann.x, ann.y, w, h)) {
        out.push({ owner: "membrane", id: ann.id, kind: "membrane-body", rank: PICK_RANK.MEMBRANE_BODY, z, dist: 0, area });
      }
      return;
    }

    // Texte / sticky — boîte ancrée en haut-gauche. La taille vient du store
    // (mesurée par le ResizeObserver de HtmlAnnotationLayer) ; sans elle, pas de
    // test de collision possible → on saute (le filet `domHint` rattrape).
    const w = ann.width ?? (ann.type === "sticky" ? 160 : 0);
    const h = ann.height ?? (ann.type === "sticky" ? 120 : 0);
    if (w <= 0 || h <= 0) return;
    if (inRect(wx, wy, ann.x, ann.y, w, h)) {
      out.push({
        owner: "annotation", id: ann.id,
        kind: ann.type === "sticky" ? "sticky" : "text",
        rank: ann.type === "sticky" ? PICK_RANK.STICKY : PICK_RANK.TEXT,
        z, dist: 0, area: 0,
        // Double-clic = édition en place → terminus du cycle (cf. en-tête).
        terminal: true,
      });
    }
  });

  // ── Dossiers ─────────────────────────────────────────────────────────────
  folders.forEach((f, z) => {
    const area = Math.abs(f.width * f.height);
    const onHeader = wx >= f.x && wx <= f.x + f.width
      && wy >= f.y && wy <= f.y + PICK.FOLDER_HEADER;
    if (onHeader || onRectEdge(wx, wy, f.x, f.y, f.width, f.height, band)) {
      out.push({ owner: "folder", id: f.id, kind: "folder-edge", rank: PICK_RANK.FOLDER_EDGE, z, dist: 0, area });
    } else if (inRect(wx, wy, f.x, f.y, f.width, f.height)) {
      out.push({ owner: "folder", id: f.id, kind: "folder-body", rank: PICK_RANK.FOLDER_BODY, z, dist: 0, area });
    }
  });

  // ── Flèche ───────────────────────────────────────────────────────────────
  // Son tracé (courbes, pathfinding, waypoints) est calculé par ArrowSvgLayer ;
  // on ne le refait pas ici. Si le DOM dit que le curseur est dessus, elle entre
  // dans la liste à son rang.
  if (input.arrowId) {
    out.push({ owner: "arrow", id: input.arrowId, kind: "arrow", rank: PICK_RANK.ARROW, z: 0, dist: 0, area: 0 });
  }

  ensureDomHint(input, out);

  // Rang croissant ; puis, à rang égal : la poignée la plus proche, le conteneur
  // le plus petit, l'élément peint le plus haut (dernier de sa liste). L'ordre
  // doit être TOTAL et STABLE, sinon la signature du cycle changerait d'un clic
  // à l'autre et le « switch priority » repartirait de zéro.
  out.sort((a, b) =>
    a.rank - b.rank
    || a.dist - b.dist
    || a.area - b.area
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
  /** Index, dans la liste cyclable, de la cible que ce cycle possède. */
  index: number;
  /** Horodatage du dernier événement du cycle (appui ou relâchement). */
  t: number;
  /** Ce clic-ci est un RE-clic délibéré : son relâchement, s'il est immobile,
   *  fera avancer le cycle d'un cran. */
  repeat: boolean;
}

export interface PickOptions {
  /** Alt : ignore la priorité absolue des poignées et force le pas suivant. */
  alt?: boolean;
  /** Ctrl/Shift/Cmd (multi-sélection) : jamais de cycle, on prend le 1er rang. */
  multi?: boolean;
}

function signatureOf(cands: PickCandidate[]): string {
  return cands.map((c) => `${c.owner}:${c.id}:${c.kind}`).join("|");
}

function cyclableOf(candidates: PickCandidate[]): PickCandidate[] {
  return candidates.filter((c) => c.rank !== PICK_RANK.HANDLE);
}

/**
 * Cible d'un APPUI (pointerdown).
 *
 *   • Les POIGNÉES gagnent toujours (sauf Alt) : c'est le geste « je veux
 *     redimensionner », il ne doit jamais rater, y compris juste après avoir
 *     sélectionné l'élément.
 *   • Sinon on rend la cible que le cycle possède DÉJÀ — jamais la suivante.
 *     C'est le point clé : un appui peut toujours devenir un GLISSER, et il doit
 *     alors déplacer ce que l'utilisateur voit sélectionné. Avancer ici faisait
 *     que « je clique mon image, puis je la tire » attrapait la membrane.
 *   • Le pas se joue au relâchement (`advanceOnRelease`).
 */
export function pickAtDown(
  candidates: PickCandidate[],
  prev: CycleState | null,
  sx: number,
  sy: number,
  now: number,
  opts: PickOptions = {},
): { picked: PickCandidate | null; cycle: CycleState | null } {
  if (candidates.length === 0) return { picked: null, cycle: null };

  const handles = candidates.filter((c) => c.rank === PICK_RANK.HANDLE);
  const cyclable = cyclableOf(candidates);
  const sig = signatureOf(cyclable);

  const dt = prev ? now - prev.t : Number.POSITIVE_INFINITY;
  const sameSpot = !!prev
    && prev.sig === sig
    && dt <= PICK.CYCLE_TTL_MS
    && Math.hypot(sx - prev.sx, sy - prev.sy) <= PICK.CYCLE_RADIUS_PX
    && !opts.multi;

  // Un re-clic ne compte comme « pas suivant » que s'il est assez espacé du
  // précédent pour ne PAS être la 2e moitié d'un double-clic : sinon ouvrir un
  // éditeur déplacerait la sélection sous l'éditeur. Alt ayant déjà consommé le
  // pas ci-dessous, il ne le rejoue pas au relâchement.
  const repeat = sameSpot && dt >= PICK.DBLCLICK_MS && !opts.alt;

  let index = sameSpot ? Math.min(prev!.index, Math.max(0, cyclable.length - 1)) : 0;
  // Alt = raccourci expert « descends d'un cran TOUT DE SUITE », sans attendre
  // le relâchement, et sans se faire intercepter par une poignée.
  if (opts.alt && sameSpot && cyclable.length > 1) index = (index + 1) % cyclable.length;

  const cycle: CycleState = { sx, sy, sig, index, t: now, repeat };

  if (handles.length > 0 && !opts.alt) return { picked: handles[0], cycle };
  if (cyclable.length === 0) return { picked: handles[0] ?? null, cycle };
  return { picked: cyclable[index] ?? cyclable[0], cycle };
}

/**
 * Pas du cycle, joué au RELÂCHEMENT d'un re-clic immobile.
 *
 * Pourquoi ici et pas à l'appui : à l'appui, on ne sait pas encore si le geste
 * sera un clic ou un glisser. En avançant au relâchement, les deux cohabitent
 * sans arbitrage :
 *
 *   clic 1         → image (rang le plus prioritaire)
 *   appui 2 + tiré → déplace l'IMAGE (le cycle n'a pas bougé)
 *   clic 2         → au relâcher : membrane
 *   appui 3 + tiré → déplace la MEMBRANE
 *   clic 3         → au relâcher : texte… et le cycle s'arrête là (terminal).
 *
 * L'appelant ne doit invoquer cette fonction que si le pointeur n'a pas bougé
 * de plus de `CYCLE_RADIUS_PX` entre l'appui et le relâchement.
 */
export function advanceOnRelease(
  candidates: PickCandidate[],
  cycle: CycleState | null,
  now: number,
): { picked: PickCandidate | null; cycle: CycleState | null } {
  if (!cycle) return { picked: null, cycle: null };
  // Le cycle survit au relâchement (le clic suivant doit savoir où il en est)
  // mais son droit d'avancer est consommé.
  const settled: CycleState = { ...cycle, t: now, repeat: false };
  if (!cycle.repeat) return { picked: null, cycle: settled };

  const cyclable = cyclableOf(candidates);
  // La pile a changé entre l'appui et le relâchement (suppression, undo, arrivée
  // d'un pair en collaboration…) : le cycle ne décrit plus rien, on le jette.
  if (signatureOf(cyclable) !== cycle.sig) return { picked: null, cycle: null };
  if (cyclable.length < 2) return { picked: null, cycle: settled };

  // Un bloc éditable est un TERMINUS : le clic suivant doit rester disponible
  // pour le double-clic d'édition.
  if (cyclable[cycle.index]?.terminal) return { picked: null, cycle: settled };

  const index = (cycle.index + 1) % cyclable.length;
  return { picked: cyclable[index], cycle: { ...settled, index } };
}
