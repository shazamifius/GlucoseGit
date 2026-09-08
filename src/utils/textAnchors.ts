/**
 * textAnchors — ancrage d'une sélection de texte par POSITION, pas par contenu.
 *
 * Le bug d'origine : `sourceTextSel` / `targetTextSel` stockaient la chaîne
 * sélectionnée (« bonjours »). Une chaîne ne désigne pas *quelle* occurrence :
 * un texte contenant deux fois « bonjours » devenait ambigu, et chacun des trois
 * consommateurs (panneau d'édition, glow au survol, ancrage Y de la flèche)
 * tranchait l'ambiguïté à sa façon — d'où deux comportements contradictoires
 * pour une même donnée.
 *
 * Le modèle retenu est celui du W3C Web Annotation, réduit à l'utile :
 *   - TextPositionSelector : `start` / `end`, offsets dans le texte RENDU
 *     (concaténation des nœuds texte du conteneur markdown) → identité exacte ;
 *   - TextQuoteSelector : `quote` + `prefix` / `suffix` → ré-ancrage quand le
 *     texte du bloc a été édité et que les offsets ont glissé.
 *
 * Rien n'est injecté dans le markdown de l'utilisateur : l'ancre vit à côté du
 * texte, elle survit donc à l'édition manuelle et au copier/coller.
 *
 * ⚠️ Les offsets portent sur le texte RENDU, pas sur le markdown brut. Tous les
 * rendus d'un même bloc doivent donc passer par le même pipeline
 * (`preprocessText` + react-markdown + mêmes plugins), sans quoi les offsets
 * divergent — le ré-ancrage par citation reste le filet de sécurité.
 */

export interface TextAnchor {
  /** Offset de début dans le texte rendu du bloc. -1 = ancre héritée sans position. */
  start: number;
  /** Offset de fin (exclu). */
  end: number;
  /** Texte exact sélectionné — sert au ré-ancrage si les offsets ont glissé. */
  quote: string;
  /** Jusqu'à CONTEXT_LEN caractères précédant la citation. */
  prefix?: string;
  /** Jusqu'à CONTEXT_LEN caractères suivant la citation. */
  suffix?: string;
}

/** Forme persistée : tableau d'ancres, ou l'ancienne chaîne « a ‖ b » (compat). */
export type TextSelection = string | TextAnchor[];

export interface ResolvedRange {
  start: number;
  end: number;
}

/** Séparateur de l'ancien encodage multi-sélection dans une seule chaîne. */
const LEGACY_SEP = " ‖ ";
/** Longueur du contexte mémorisé de chaque côté de la citation. */
const CONTEXT_LEN = 32;

/* ────────────────────────── Construction ────────────────────────── */

/**
 * Fabrique une ancre à partir d'une plage d'offsets dans le texte rendu.
 * Rogne les blancs de bord (une sélection souris déborde souvent d'un espace).
 * Renvoie `null` si la plage est vide après rognage.
 */
export function createAnchor(plain: string, start: number, end: number): TextAnchor | null {
  let s = Math.max(0, Math.min(start, plain.length));
  let e = Math.max(0, Math.min(end, plain.length));
  if (e < s) [s, e] = [e, s];
  while (s < e && /\s/.test(plain[s])) s++;
  while (e > s && /\s/.test(plain[e - 1])) e--;
  if (e <= s) return null;
  return {
    start: s,
    end: e,
    quote: plain.slice(s, e),
    prefix: plain.slice(Math.max(0, s - CONTEXT_LEN), s),
    suffix: plain.slice(e, Math.min(plain.length, e + CONTEXT_LEN)),
  };
}

/**
 * Ajoute une ancre à une liste, en ignorant les doublons **positionnels**.
 * Contrairement à l'ancien code, deux occurrences du même mot sont bien deux
 * ancres distinctes : c'est exactement le cas que le bug empêchait.
 */
export function addAnchor(anchors: TextAnchor[], anchor: TextAnchor): TextAnchor[] {
  const overlaps = anchors.some(a => a.start < anchor.end && anchor.start < a.end);
  if (overlaps) return anchors;
  return [...anchors, anchor].sort((a, b) => a.start - b.start);
}

/* ────────────────────────── Normalisation ────────────────────────── */

function isAnchor(v: unknown): v is TextAnchor {
  if (!v || typeof v !== "object") return false;
  const a = v as Partial<TextAnchor>;
  return typeof a.start === "number" && typeof a.end === "number" && typeof a.quote === "string";
}

/**
 * Ramène n'importe quelle forme persistée à un tableau d'ancres.
 * Les projets d'avant la refonte stockaient « a ‖ b » : on en fait des ancres
 * sans position (`start: -1`), que `resolveAnchors` retrouvera par citation —
 * soit le comportement historique (première occurrence), sans rien casser.
 */
export function normalizeTextSel(value: TextSelection | undefined | null): TextAnchor[] {
  if (!value) return [];
  if (Array.isArray(value)) return value.filter(isAnchor);
  return value
    .split(LEGACY_SEP)
    .map(s => s.trim())
    .filter(Boolean)
    .map(quote => ({ start: -1, end: -1, quote }));
}

/** `true` si la sélection persistée contient au moins une ancre exploitable. */
export function hasTextSelection(value: TextSelection | undefined | null): boolean {
  return normalizeTextSel(value).length > 0;
}

/* ────────────────────────── Résolution ────────────────────────── */

function indexesOf(hay: string, needle: string): number[] {
  if (!needle) return [];
  const out: number[] = [];
  let i = hay.indexOf(needle);
  while (i !== -1) {
    out.push(i);
    i = hay.indexOf(needle, i + 1);
  }
  return out;
}

function commonSuffixLen(a: string, b: string): number {
  let n = 0;
  while (n < a.length && n < b.length && a[a.length - 1 - n] === b[b.length - 1 - n]) n++;
  return n;
}

function commonPrefixLen(a: string, b: string): number {
  let n = 0;
  while (n < a.length && n < b.length && a[n] === b[n]) n++;
  return n;
}

/**
 * Retrouve l'occurrence visée quand les offsets ne collent plus (texte édité).
 * On note chaque candidat sur la ressemblance de son contexte gauche/droit,
 * et on départage à la distance à la position d'origine.
 */
function reanchor(plain: string, anchor: TextAnchor): ResolvedRange | null {
  const { quote } = anchor;
  if (!quote) return null;

  let candidates = indexesOf(plain, quote);
  const len = quote.length;
  if (candidates.length === 0) {
    // Repli insensible à la casse (l'ancien résolveur l'était partout).
    candidates = indexesOf(plain.toLowerCase(), quote.toLowerCase());
    if (candidates.length === 0) return null;
  }

  const prefix = anchor.prefix ?? "";
  const suffix = anchor.suffix ?? "";
  const origin = anchor.start >= 0 ? anchor.start : 0;

  let best = candidates[0];
  let bestScore = -1;
  let bestDist = Number.POSITIVE_INFINITY;
  for (const i of candidates) {
    const left = plain.slice(Math.max(0, i - CONTEXT_LEN), i);
    const right = plain.slice(i + len, i + len + CONTEXT_LEN);
    const score = commonSuffixLen(left, prefix) + commonPrefixLen(right, suffix);
    const dist = Math.abs(i - origin);
    if (score > bestScore || (score === bestScore && dist < bestDist)) {
      best = i;
      bestScore = score;
      bestDist = dist;
    }
  }
  return { start: best, end: best + len };
}

/**
 * Résout des ancres contre le texte rendu d'un bloc.
 * Une ancre dont la citation a disparu est simplement abandonnée (pas de
 * surlignage fantôme). Les plages renvoyées sont triées et fusionnées.
 */
export function resolveAnchors(plain: string, anchors: TextAnchor[]): ResolvedRange[] {
  const ranges: ResolvedRange[] = [];
  for (const anchor of anchors) {
    if (
      anchor.start >= 0 &&
      anchor.end <= plain.length &&
      anchor.end > anchor.start &&
      plain.slice(anchor.start, anchor.end) === anchor.quote
    ) {
      ranges.push({ start: anchor.start, end: anchor.end });
      continue;
    }
    const found = reanchor(plain, anchor);
    if (found) ranges.push(found);
  }

  ranges.sort((a, b) => a.start - b.start || a.end - b.end);
  const merged: ResolvedRange[] = [];
  for (const r of ranges) {
    const last = merged[merged.length - 1];
    if (last && r.start <= last.end) last.end = Math.max(last.end, r.end);
    else merged.push({ ...r });
  }
  return merged;
}

/** Résout directement depuis la forme persistée. */
export function resolveTextSel(plain: string, value: TextSelection | undefined | null): ResolvedRange[] {
  return resolveAnchors(plain, normalizeTextSel(value));
}

/* ────────────────────────── Pont DOM ────────────────────────── */

interface TextNodeEntry {
  node: Text;
  start: number;
  end: number;
}

export interface DomTextIndex {
  entries: TextNodeEntry[];
  /** Concaténation des nœuds texte — l'espace d'offsets des ancres. */
  plain: string;
}

/**
 * Indexe les nœuds texte d'un conteneur rendu.
 * L'espace d'offsets est la concaténation brute des nœuds texte, exactement ce
 * que renvoie `Range.toString()` : `domPointToOffset` et cet index parlent donc
 * le même langage.
 */
export function indexDomText(root: Node): DomTextIndex {
  const entries: TextNodeEntry[] = [];
  let plain = "";
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, null);
  let nd: Node | null;
  while ((nd = walker.nextNode())) {
    const node = nd as Text;
    const text = node.data;
    if (!text) continue;
    entries.push({ node, start: plain.length, end: plain.length + text.length });
    plain += text;
  }
  return { entries, plain };
}

/**
 * Convertit un point DOM (nœud + offset, tel que le donne une Selection) en
 * offset dans le texte rendu. Passe par `Range.toString()` : c'est la seule
 * mesure qui reste exacte quand la sélection démarre sur un élément et non sur
 * un nœud texte.
 */
export function domPointToOffset(root: Node, node: Node, offset: number): number {
  const range = document.createRange();
  range.selectNodeContents(root);
  try {
    range.setEnd(node, offset);
  } catch {
    return 0;
  }
  return range.toString().length;
}

function locate(index: DomTextIndex, offset: number, side: "start" | "end") {
  for (const entry of index.entries) {
    if (side === "start" ? offset < entry.end : offset <= entry.end) {
      if (offset >= entry.start) return { node: entry.node, offset: offset - entry.start };
    }
  }
  const last = index.entries[index.entries.length - 1];
  if (!last) return null;
  return { node: last.node, offset: last.node.data.length };
}

/** Construit des `Range` DOM pour des plages d'offsets — mesure géométrique. */
export function domRangesFor(root: Node, ranges: ResolvedRange[]): Range[] {
  if (ranges.length === 0) return [];
  const index = indexDomText(root);
  const out: Range[] = [];
  for (const r of ranges) {
    const s = locate(index, r.start, "start");
    const e = locate(index, r.end, "end");
    if (!s || !e) continue;
    try {
      const range = document.createRange();
      range.setStart(s.node, s.offset);
      range.setEnd(e.node, e.offset);
      out.push(range);
    } catch {}
  }
  return out;
}

/** Marqueur posé sur les `<mark>` injectés, pour les distinguer du contenu. */
export const HL_ATTR = "data-glucose-hl";

/**
 * Enveloppe les plages données dans des `<mark>` et renvoie la fonction de
 * retrait. Une plage qui traverse plusieurs nœuds texte (gras, code, lien…)
 * produit un `<mark>` par fragment — l'ancien `indexOf` sur un seul nœud, lui,
 * n'y trouvait rien du tout.
 */
export function highlightDomRanges(
  root: HTMLElement,
  ranges: ResolvedRange[],
  decorate: (mark: HTMLElement) => void,
): () => void {
  if (ranges.length === 0) return () => {};
  const index = indexDomText(root);
  const marks: HTMLElement[] = [];

  for (const entry of index.entries) {
    // Fragments de cette entrée couverts par une plage, en ordre DÉCROISSANT :
    // `splitText` conserve le début dans le nœud d'origine, donc découper par la
    // fin laisse les offsets des fragments précédents valides.
    const local: ResolvedRange[] = [];
    for (const r of ranges) {
      const a = Math.max(r.start, entry.start);
      const b = Math.min(r.end, entry.end);
      if (b > a) local.push({ start: a - entry.start, end: b - entry.start });
    }
    if (local.length === 0) continue;
    local.sort((x, y) => y.start - x.start);

    for (const { start, end } of local) {
      const node = entry.node;
      try {
        if (end < node.data.length) node.splitText(end);
        const mid = start > 0 ? node.splitText(start) : node;
        const parent = mid.parentNode;
        if (!parent) continue;
        const mark = document.createElement("mark");
        mark.setAttribute(HL_ATTR, "true");
        decorate(mark);
        parent.replaceChild(mark, mid);
        mark.appendChild(mid);
        marks.push(mark);
      } catch {}
    }
  }

  return () => {
    for (const mark of marks) {
      const parent = mark.parentNode;
      if (!parent) continue;
      while (mark.firstChild) parent.insertBefore(mark.firstChild, mark);
      parent.removeChild(mark);
      // Refusionne les fragments : le nœud texte d'origine (premier du groupe)
      // survit à `normalize` et retrouve son contenu complet — les refs que
      // React garde dessus restent donc valides.
      parent.normalize();
    }
  };
}
