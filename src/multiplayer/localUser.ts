// ────────────────────────────────────────────────────────────────────────────
// COLLAB-1 — Identité locale : ton nom et ta couleur.
//
// Elles existaient déjà, mais enfouies dans `PeerCursorsLayer` : tirées au sort,
// impossibles à modifier, et rangées dans `sessionStorage` — donc reperdues à
// chaque fermeture de l'application. On pouvait travailler une semaine à
// plusieurs sans jamais garder le même nom deux jours de suite.
//
// Trois changements, dans cet ordre d'importance :
//
//   • elles se MODIFIENT (c'est le but) ;
//   • elles PERSISTENT (`localStorage`) — un nom qu'on a choisi ne doit pas
//     s'évaporer en fermant la fenêtre ;
//   • elles PRÉVIENNENT quand elles changent, pour que les pairs voient le
//     nouveau nom tout de suite au lieu du prochain mouvement de souris.
//
// Le tirage au sort reste, mais seulement comme point de départ : arriver dans
// une session sans nom est pire que d'arriver avec « Renard Bleu ».
//
// La couleur sert AUSSI de code visuel ailleurs qu'aux curseurs : les languettes
// des coulisses reprennent la couleur de leur propriétaire (cf. MEMB-3). D'où
// une palette exposée, assez contrastée pour rester distinguable en petit.
// ────────────────────────────────────────────────────────────────────────────

const STORAGE_KEY = "glucose:local-user";

/** Palette proposée. Choisie pour rester lisible en pastille de quelques pixels
 *  sur fond sombre, et pour que deux voisines ne se confondent pas. */
export const USER_COLORS = [
  "#38bdf8", // ciel
  "#34d399", // émeraude
  "#fb923c", // orange
  "#c084fc", // violet
  "#f472b6", // rose
  "#fbbf24", // ambre
  "#2dd4bf", // turquoise
  "#818cf8", // indigo
  "#f87171", // rouge
  "#a3e635", // citron
] as const;

const ADJECTIVES = ["Bleu", "Rouge", "Vert", "Jaune", "Orange", "Violet", "Rose", "Cyan", "Indigo", "Émeraude"];
const ANIMALS = ["Renard", "Ours", "Aigle", "Chat", "Chien", "Lapin", "Loup", "Cerf", "Hibou", "Écureuil"];

/** Longueur maximale d'un nom : au-delà, l'étiquette de curseur devient une
 *  banderole qui masque le canvas des autres. */
export const MAX_NAME_LENGTH = 24;

export interface LocalUser {
  name: string;
  color: string;
}

export const USER_CHANGED_EVENT = "glucose:local-user-changed";

// ════════════════════════════════════════════════════════════════════════════
// Validation
// ════════════════════════════════════════════════════════════════════════════

/**
 * Nom utilisable : sans espaces superflus, sur une seule ligne, borné.
 *
 * Un nom vide retombe sur un tirage plutôt que d'afficher un curseur anonyme —
 * en collaboration, un curseur sans nom est un curseur qu'on ne peut pas
 * interpeller.
 */
export function sanitizeName(raw: string): string {
  const flat = raw.replace(/[\r\n\t]+/g, " ").replace(/\s+/g, " ").trim();
  if (flat.length === 0) return randomName();
  return flat.slice(0, MAX_NAME_LENGTH);
}

/** Couleur utilisable, ou une couleur tirée au sort si elle est illisible. */
export function sanitizeColor(raw: string): string {
  const m = raw.trim();
  if (/^#[0-9a-fA-F]{6}$/.test(m)) return m.toLowerCase();
  if (/^#[0-9a-fA-F]{3}$/.test(m)) {
    const [r, g, b] = [m[1], m[2], m[3]];
    return `#${r}${r}${g}${g}${b}${b}`.toLowerCase();
  }
  return randomColor();
}

export function randomName(): string {
  const a = ANIMALS[Math.floor(Math.random() * ANIMALS.length)];
  const j = ADJECTIVES[Math.floor(Math.random() * ADJECTIVES.length)];
  return `${a} ${j}`;
}

export function randomColor(): string {
  return USER_COLORS[Math.floor(Math.random() * USER_COLORS.length)];
}

// ════════════════════════════════════════════════════════════════════════════
// Lecture / écriture
// ════════════════════════════════════════════════════════════════════════════

let cached: LocalUser | null = null;

function readStored(): LocalUser | null {
  try {
    // Reprise de l'ancien emplacement : quelqu'un qui met à jour l'application
    // en pleine session garde le nom qu'il avait à l'écran.
    const raw = localStorage.getItem(STORAGE_KEY) ?? sessionStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<LocalUser>;
    if (typeof parsed?.name !== "string" || typeof parsed?.color !== "string") return null;
    return { name: sanitizeName(parsed.name), color: sanitizeColor(parsed.color) };
  } catch {
    // Stockage indisponible (navigation privée, quota) ou JSON abîmé : on
    // repart d'une identité fraîche plutôt que de faire échouer la collaboration.
    return null;
  }
}

function writeStored(user: LocalUser) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(user));
  } catch {
    /* stockage indisponible : l'identité vivra le temps de la session */
  }
}

/** L'identité locale, créée au premier appel. */
export function getLocalUser(): LocalUser {
  if (cached) return cached;
  cached = readStored() ?? { name: randomName(), color: randomColor() };
  writeStored(cached);
  return cached;
}

/**
 * Modifie l'identité et prévient l'application.
 *
 * L'événement est ce qui fait qu'un changement de nom se voit TOUT DE SUITE
 * chez les autres : sans lui, la présence n'est renvoyée qu'au prochain
 * mouvement de souris, et on reste affiché sous son ancien nom en attendant.
 */
export function setLocalUser(patch: Partial<LocalUser>): LocalUser {
  const current = getLocalUser();
  const next: LocalUser = {
    name: patch.name !== undefined ? sanitizeName(patch.name) : current.name,
    color: patch.color !== undefined ? sanitizeColor(patch.color) : current.color,
  };
  if (next.name === current.name && next.color === current.color) return current;

  cached = next;
  writeStored(next);
  if (typeof window !== "undefined") {
    window.dispatchEvent(new CustomEvent(USER_CHANGED_EVENT, { detail: next }));
  }
  return next;
}

/** Réinitialise le cache mémoire — tests uniquement. */
export function _resetLocalUserCache() {
  cached = null;
}
