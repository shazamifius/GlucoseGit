// ─────────────────────────────────────────────────────────────────────────────
// Géométrie du pont Glucose — fonctions PURES, sans I/O, sans Automerge.
// ─────────────────────────────────────────────────────────────────────────────
//
// Séparé de glucose-mcp.mjs pour UNE raison : ce fichier appelle `main()` au
// chargement, donc l'importer démarrerait le serveur. C'est pour ça qu'aucun test
// n'a jamais couvert cette géométrie — elle n'était pas importable. Le même
// verrou avait caché pendant des jours un bug d'inversion des flèches côté app,
// jusqu'à ce que la fonction sorte du composant et devienne testable.
//
// Tout ce qui décide d'une POSITION vit ici, et rien d'autre.

export const COL_WIDTH = 380;
export const LINE_H = 22;

// ── Hauteur d'une note : une BORNE SUPÉRIEURE, pas une estimation ────────────
//
// L'app mesure ce qu'elle rend et réécrit la vraie hauteur dans le document
// (HtmlAnnotationLayer → syncAnnotationSize). Tant qu'elle ne l'a pas fait, le
// pont doit la deviner.
//
// L'ancienne formule (lignes × 22 + 16) SOUS-ESTIMAIT dans 93 % des cas, d'un
// facteur 3,68 en médiane : elle ignorait le « chrome » de la note (padding,
// marges), qui coûte ~200 px quel que soit le texte. Ranger avec une hauteur
// trois fois trop petite, c'est ranger un monde qui n'existe pas — les notes ne
// se chevauchent alors pas dans la donnée, seulement à l'écran, là où aucun
// contrôle ne regarde.
//
// D'où une BORNE et non une moyenne. Ces coefficients majorent les 1782 notes du
// corpus réel dont l'app avait mesuré la hauteur (0 violation, ~99 px d'excès
// moyen). La propriété qui porte toute la garantie :
//
//     borne ≥ hauteur rendue  ⟹  (0 chevauchement dans la borne ⟹ 0 à l'écran)
//
// Majorer coûte de l'espace ; sous-estimer coûte la garantie. On paie l'espace.
// Ces valeurs ne se règlent pas à l'œil : les REFITTER sur un corpus mesuré.
export const H_PER_LINE = 23.5;
export const H_CHROME = 194;

/** Respiration minimale laissée entre deux boîtes quand on les sépare. */
export const NOTE_GAP = 24;

/** Nombre de lignes rendues, approximé par le retour à la ligne à `width`. */
export function estimateLines(text, width) {
  const cpl = Math.max(Math.floor((width ?? COL_WIDTH) / 8), 20);
  return (text ?? "").split("\n").reduce((n, ln) => n + Math.max(1, Math.ceil(ln.length / cpl)), 0);
}

/** Borne SUPÉRIEURE de la hauteur rendue d'une note dont `height` est absent. */
export function heightUpperBound(text, width) {
  return Math.ceil(estimateLines(text, width) * H_PER_LINE + H_CHROME);
}

/**
 * Boîte d'une annotation. Une annotation est posée par son COIN haut-gauche.
 * (Une image, elle, est posée par son CENTRE — voir `imgBox`. Deux conventions
 * dans le même canevas : les confondre décale de la moitié de l'objet.)
 */
export function annBox(a) {
  const w = a.width ?? COL_WIDTH;
  let h = a.height;
  if (typeof h !== "number") h = a.type === "membrane" ? 200 : heightUpperBound(a.text, w);
  const x = a.x ?? 0, y = a.y ?? 0;
  return { x, y, w, h, cx: x + w / 2, cy: y + h / 2 };
}

/**
 * Boîte d'une image ou d'une vidéo. L'app l'ancre par son CENTRE :
 * `left = x - width/2` (ArrowSvgLayer, résolution d'ancre). Traiter son (x,y)
 * comme un coin la décalerait de la moitié de sa taille — et ferait croire à un
 * chevauchement là où il n'y en a pas, ou l'inverse.
 */
export function imgBox(i) {
  const w = i.width ?? 200, h = i.height ?? 200;
  const cx = i.x ?? 0, cy = i.y ?? 0;
  return { x: cx - w / 2, y: cy - h / 2, w, h, cx, cy };
}

/** Centre d'une annotation (pour ancrer une flèche). */
export function annCenter(a) {
  const b = annBox(a);
  return { x: b.cx, y: b.cy };
}

/** Paires de boîtes qui se chevauchent. `boxes` : [{id,x,y,w,h}]. */
export function overlappingPairs(boxes) {
  const out = [];
  for (let i = 0; i < boxes.length; i++)
    for (let j = i + 1; j < boxes.length; j++) {
      const a = boxes[i], b = boxes[j];
      const ox = Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x);
      const oy = Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y);
      if (ox > 0 && oy > 0) out.push({ a: a.id, b: b.id, area: Math.round(ox * oy) });
    }
  return out;
}

/**
 * Sépare `mobiles` jusqu'à zéro chevauchement — entre eux ET avec `fixes`.
 *
 * `fixes` (images, vidéos) ne bougent JAMAIS : l'utilisateur les a posées là, ce
 * sont des ancres. Ce sont les notes qui s'écartent autour.
 *
 * Déterministe : à chaque passe, une paire en conflit est écartée sur l'axe de
 * MOINDRE pénétration — le mouvement le plus court qui résout le conflit. Écarter
 * A de B peut pousser A dans C, d'où l'itération ; l'aire de recouvrement totale
 * décroît strictement, donc ça converge.
 *
 * Mute `mobiles` en place.
 * @returns {{passes:number, ok:boolean, restant:number}} `ok:false` = plafond
 *   atteint sans convergence → l'appelant DOIT le dire au lieu de prétendre.
 */
export function separateUntilClean(mobiles, fixes = [], margin = NOTE_GAP, maxPasses = 500) {
  const ecarte = (a, b, bougeB) => {
    const dx = b.cx - a.cx, dy = b.cy - a.cy;
    const ox = (a.w + b.w) / 2 + margin - Math.abs(dx);
    const oy = (a.h + b.h) / 2 + margin - Math.abs(dy);
    if (ox <= 0 || oy <= 0) return false;
    // Signe stable quand les centres coïncident : sans ça, deux boîtes exactement
    // superposées (dx = dy = 0) ne se sépareraient jamais.
    const part = bougeB ? 0.5 : 1; // si b est fixe, a encaisse tout le déplacement
    if (ox < oy) {
      const s = ox * part * (dx < 0 ? -1 : 1);
      a.x -= s; a.cx -= s;
      if (bougeB) { b.x += s; b.cx += s; }
    } else {
      const s = oy * part * (dy < 0 ? -1 : 1);
      a.y -= s; a.cy -= s;
      if (bougeB) { b.y += s; b.cy += s; }
    }
    return true;
  };

  for (let pass = 1; pass <= maxPasses; pass++) {
    let bouge = false;
    for (let i = 0; i < mobiles.length; i++) {
      for (let j = i + 1; j < mobiles.length; j++)
        if (ecarte(mobiles[i], mobiles[j], true)) bouge = true;
      for (const f of fixes)
        if (ecarte(mobiles[i], f, false)) bouge = true;
    }
    if (!bouge) return { passes: pass, ok: true, restant: 0 };
  }
  const restant = overlappingPairs([...mobiles, ...fixes]).length;
  return { passes: maxPasses, ok: restant === 0, restant };
}
