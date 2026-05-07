// CLEANUP C-01 — Composants extraits pour les badges des annotations.
// Dédupliquait 3x dans HtmlAnnotationLayer (text, sticky standard, sticky-opérateur).

import type { Domain, DomainAssignment, TemporalAnchor } from "../types";
import { formatAnchor } from "../utils/timeline";

// ════════════════════════════════════════════════════════════════════════════
// Mirror badge (Phase 4)
// ════════════════════════════════════════════════════════════════════════════

export function MirrorBadge({ mirrorOf }: { mirrorOf?: string }) {
  if (!mirrorOf) return null;
  return (
    <button
      onPointerDown={(e) => e.stopPropagation()}
      onClick={(e) => {
        e.stopPropagation();
        window.dispatchEvent(
          new CustomEvent("glucose:teleport-to-mirror-original", {
            detail: { mirrorOf, type: "annotation" },
          }),
        );
      }}
      title="Miroir → cliquer pour aller à l'original"
      style={{
        position: "absolute",
        top: -10,
        left: -10,
        width: 22,
        height: 22,
        borderRadius: "50%",
        background: "rgba(15,15,25,0.9)",
        color: "#93c5fd",
        border: "1.5px solid #93c5fd99",
        cursor: "pointer",
        padding: 0,
        fontSize: 13,
        lineHeight: 1,
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        pointerEvents: "all",
        boxShadow: "0 0 8px rgba(147,197,253,0.4)",
      }}
    >
      ↻
    </button>
  );
}

// ════════════════════════════════════════════════════════════════════════════
// Domain badges (Phase 3)
// ════════════════════════════════════════════════════════════════════════════

export interface DomainBadgesProps {
  /** Liste filtrée des domaines (poids > 0.4) résolus avec leur définition. */
  badges: Array<DomainAssignment & { def: Domain }>;
}

export function DomainBadges({ badges }: DomainBadgesProps) {
  if (badges.length === 0) return null;
  return (
    <div
      style={{
        position: "absolute",
        top: -10,
        right: -10,
        display: "flex",
        gap: 4,
        pointerEvents: "none",
      }}
    >
      {badges.map((b) => (
        <span
          key={b.domainId}
          title={`${b.def.name} (${Math.round(b.weight * 100)}%)`}
          style={{
            display: "inline-flex",
            alignItems: "center",
            justifyContent: "center",
            width: 22,
            height: 22,
            borderRadius: "50%",
            background: `${b.def.color}cc`,
            color: "#fff",
            fontSize: 12,
            lineHeight: 1,
            border: "1.5px solid #111",
            boxShadow: `0 0 8px ${b.def.color}66`,
          }}
        >
          {b.def.icon}
        </span>
      ))}
    </div>
  );
}

// ════════════════════════════════════════════════════════════════════════════
// Temporal anchor badge (Phase 6)
// ════════════════════════════════════════════════════════════════════════════

export function TemporalBadge({ anchor }: { anchor?: TemporalAnchor }) {
  if (!anchor) return null;
  return (
    <span
      title={`Date : ${formatAnchor(anchor)}`}
      style={{
        position: "absolute",
        bottom: -10,
        right: -10,
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        height: 18,
        minWidth: 18,
        padding: "0 6px",
        borderRadius: 9,
        background: "rgba(15,15,25,0.9)",
        color: "#fde68a",
        border: "1.5px solid #fbbf2499",
        fontSize: 10,
        lineHeight: 1,
        pointerEvents: "none",
        boxShadow: "0 0 6px rgba(251,191,36,0.35)",
        whiteSpace: "nowrap",
        fontFamily: "system-ui, sans-serif",
      }}
    >
      📅 {formatAnchor(anchor)}
    </span>
  );
}

/**
 * Calcule les badges domaine à afficher pour une annotation/image.
 * Retourne uniquement les domaines avec poids > 0.4 ET dont la définition existe.
 */
export function resolveDomainBadges(
  nodeDomains: DomainAssignment[] | undefined,
  domains: Domain[],
): Array<DomainAssignment & { def: Domain }> {
  if (!nodeDomains?.length) return [];
  const byId = new Map(domains.map((d) => [d.id, d]));
  const out: Array<DomainAssignment & { def: Domain }> = [];
  for (const da of nodeDomains) {
    if (da.weight <= 0.4) continue;
    const def = byId.get(da.domainId);
    if (def) out.push({ ...da, def });
  }
  return out;
}
