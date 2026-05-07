// ────────────────────────────────────────────────────────────────────────────
// Phase 6 — Réglette Temporelle Sémantique
// ────────────────────────────────────────────────────────────────────────────
// Module pur (testable, sans dépendance React/PixiJS).
//
// Convention : une "année" est un nombre entier signé.
//   • 1789  = an 1789 ap. J.-C.
//   • -3000 = 3000 av. J.-C.
//   • 0     = an 0 conventionnel (pas d'année 0 historique, mais on simplifie)
//   • -100000000 (-1e8) borne basse pratique (~Crétacé)
//   • 3000  borne haute pratique (futur proche)
//
// On évite Date / ms Unix : trop limité pour la préhistoire et négligeable
// pour la précision sub-annuelle dont on n'a pas besoin ici.

import type { TemporalAnchor } from "../types";

export const YEAR_MIN = -100_000_000; // -100 Ma (Crétacé moyen)
export const YEAR_MAX = 3_000;        // an 3000

// ── Époques nommées (extensibles par l'utilisateur plus tard) ─────────────
// Tableau ordonné chronologiquement. La première époque englobant l'année
// requise est retournée (les longues sont en premier, les précises ensuite).
export interface NamedEra {
  name: string;
  start: number;
  end: number;
  description?: string;
}

export const DEFAULT_ERAS: NamedEra[] = [
  // Préhistoire & paléontologie (large)
  { name: "Crétacé",            start: -145_000_000, end: -66_000_000, description: "Dinosaures, premiers oiseaux" },
  { name: "Paléogène",          start: -66_000_000,  end: -23_000_000, description: "Émergence des mammifères" },
  { name: "Néogène",            start: -23_000_000,  end: -2_580_000,  description: "Hominidés primitifs" },
  { name: "Pléistocène",        start: -2_580_000,   end: -11_700,     description: "Glaciations, Homo sapiens" },
  { name: "Holocène",           start: -11_700,      end: 2_026,       description: "Notre époque géologique" },

  // Préhistoire humaine
  { name: "Paléolithique",      start: -3_300_000,   end: -10_000,     description: "Pierre taillée" },
  { name: "Néolithique",        start: -10_000,      end: -3_300,      description: "Agriculture, sédentarité" },

  // Antiquité
  { name: "Antiquité",          start: -3_300,       end: 476,         description: "De l'écriture à la chute de Rome" },
  { name: "Égypte ancienne",    start: -3_150,       end: -30,         description: "Des premières dynasties à Cléopâtre" },
  { name: "Grèce antique",      start: -800,         end: -146,        description: "Cités-États, Alexandre" },
  { name: "République romaine", start: -509,         end: -27 },
  { name: "Empire romain",      start: -27,          end: 476 },

  // Moyen Âge
  { name: "Moyen Âge",          start: 476,          end: 1453,        description: "De la chute de Rome à celle de Constantinople" },
  { name: "Haut Moyen Âge",     start: 476,          end: 1000 },
  { name: "Moyen Âge central",  start: 1000,         end: 1300 },
  { name: "Bas Moyen Âge",      start: 1300,         end: 1453 },

  // Modernité
  { name: "Renaissance",        start: 1400,         end: 1600,        description: "Humanisme, redécouverte de l'antique" },
  { name: "Lumières",           start: 1715,         end: 1789,        description: "Raison, encyclopédie, droits naturels" },
  { name: "Révolution française", start: 1789,       end: 1799 },
  { name: "Empire napoléonien", start: 1804,         end: 1815 },
  { name: "Révolution industrielle", start: 1760,    end: 1840 },

  // XXe et après
  { name: "Belle Époque",       start: 1871,         end: 1914 },
  { name: "Première Guerre mondiale", start: 1914,   end: 1918 },
  { name: "Entre-deux-guerres", start: 1918,         end: 1939 },
  { name: "Seconde Guerre mondiale", start: 1939,    end: 1945 },
  { name: "Guerre froide",      start: 1947,         end: 1991 },
  { name: "Ère numérique",      start: 1990,         end: 2026,        description: "Web, mobile, IA générative" },
];

// ── Format d'une année ────────────────────────────────────────────────────
/**
 * Formate une année en string lisible selon son ordre de grandeur.
 *   1789      → "1789"
 *   -500      → "500 av. J.-C."
 *   -50000    → "50 ka"
 *   -1500000  → "1,5 Ma"
 */
export function formatYear(year: number): string {
  if (!Number.isFinite(year)) return "?";
  const y = Math.round(year);
  if (y >= 0) return String(y);
  const abs = -y;
  if (abs < 10_000) return `${abs} av. J.-C.`;
  if (abs < 1_000_000) return `${(abs / 1_000).toFixed(abs >= 100_000 ? 0 : 1)} ka`;
  return `${(abs / 1_000_000).toFixed(abs >= 100_000_000 ? 0 : 1)} Ma`;
}

/**
 * Formate une plage [start, end] de façon compacte.
 *   {1789, 1789} → "1789"
 *   {1789, 1799} → "1789–1799"
 */
export function formatAnchor(a: TemporalAnchor): string {
  if (a.label) return a.label;
  if (a.start === a.end) return formatYear(a.start);
  return `${formatYear(a.start)} – ${formatYear(a.end)}`;
}

// ── Parsing d'une saisie utilisateur ──────────────────────────────────────
/**
 * Convertit une saisie utilisateur en TemporalAnchor.
 * Formats acceptés :
 *   "1789"             → { start: 1789, end: 1789 }
 *   "1789-1799"        → { start: 1789, end: 1799 }
 *   "1789 - 1799"      → idem
 *   "-500"             → { start: -500, end: -500 }   (500 av. J.-C.)
 *   "500 av JC" / "500 BC" → { start: -500, end: -500 }
 *   "Renaissance"      → { start: 1400, end: 1600, label: "Renaissance" }
 *   "10 ka"            → { start: -10000, end: -10000 }
 *   "1,5 Ma"           → { start: -1500000, end: -1500000 }
 *
 * Renvoie null si non parsable.
 */
export function parseAnchor(input: string, eras: NamedEra[] = DEFAULT_ERAS): TemporalAnchor | null {
  const trimmed = input.trim();
  if (!trimmed) return null;

  // 1) Match exact d'une époque nommée (insensible à la casse)
  const lc = trimmed.toLowerCase();
  const era = eras.find((e) => e.name.toLowerCase() === lc);
  if (era) {
    return { start: era.start, end: era.end, label: era.name };
  }

  // 2) Plage "AAAA-BBBB" / "AAAA – BBBB" / "AAAA..BBBB"
  const range = trimmed.match(/^(-?\d[\d\s.,]*\s*(?:ka|ma|av\.?\s*j\.?-?\s*c\.?|bc)?)\s*(?:[–\-‒—]|\.\.)\s*(-?\d[\d\s.,]*\s*(?:ka|ma|av\.?\s*j\.?-?\s*c\.?|bc)?)$/i);
  if (range) {
    const a = parseSingleYear(range[1]);
    const b = parseSingleYear(range[2]);
    if (a !== null && b !== null) {
      return { start: Math.min(a, b), end: Math.max(a, b) };
    }
  }

  // 3) Année unique
  const single = parseSingleYear(trimmed);
  if (single !== null) {
    return { start: single, end: single };
  }

  return null;
}

/** Parse une année unique ; renvoie null si non reconnu. */
function parseSingleYear(input: string): number | null {
  const s = input.trim().toLowerCase().replace(/\s+/g, " ");

  // "1,5 ma", "1.5 Ma" → millions d'années avant maintenant (toujours négatif)
  const ma = s.match(/^(-?\d+(?:[.,]\d+)?)\s*ma$/);
  if (ma) {
    const n = Number(ma[1].replace(",", "."));
    if (Number.isFinite(n)) return -Math.round(n * 1_000_000);
  }

  // "10 ka", "12,5 ka" → milliers d'années avant maintenant
  const ka = s.match(/^(-?\d+(?:[.,]\d+)?)\s*ka$/);
  if (ka) {
    const n = Number(ka[1].replace(",", "."));
    if (Number.isFinite(n)) return -Math.round(n * 1_000);
  }

  // "500 av JC", "500 BC", "500 av. J.-C." → négatif
  const bc = s.match(/^(\d+)\s*(?:av\.?\s*j\.?-?\s*c\.?|bc)$/);
  if (bc) return -Number(bc[1]);

  // "1789 ap JC" / "1789 AD" → positif explicite
  const ad = s.match(/^(\d+)\s*(?:ap\.?\s*j\.?-?\s*c\.?|ad)$/);
  if (ad) return Number(ad[1]);

  // Entier signé brut "1789", "-500"
  const raw = s.match(/^(-?\d+)$/);
  if (raw) return Number(raw[1]);

  return null;
}

// ── Helper : un nœud passe-t-il le filtre temporel ? ──────────────────────
/**
 * Renvoie `true` si :
 *   - le nœud n'a pas de `temporalAnchor` (atemporel — toujours visible)
 *   - OU si l'intervalle du nœud chevauche celui du filtre (intersection non vide)
 */
export function nodeMatchesTemporalFilter(
  anchor: TemporalAnchor | undefined,
  filter: { start: number; end: number } | null
): boolean {
  if (!filter) return true;
  if (!anchor) return true;
  return anchor.start <= filter.end && anchor.end >= filter.start;
}

// ── Échelles de zoom de la réglette ───────────────────────────────────────
/** Retourne le pas de graduation adapté à la largeur visible. */
export function tickStep(spanYears: number): number {
  if (spanYears > 50_000_000) return 10_000_000;
  if (spanYears > 5_000_000)  return 1_000_000;
  if (spanYears > 500_000)    return 100_000;
  if (spanYears > 50_000)     return 10_000;
  if (spanYears > 5_000)      return 1_000;
  if (spanYears > 500)        return 100;
  if (spanYears > 50)         return 10;
  if (spanYears > 5)          return 1;
  return 1;
}
