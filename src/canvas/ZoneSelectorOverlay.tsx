import type { RefObject } from "react";

const ACTION_LABELS: Record<string, string> = {
  folder: "dossier",
  membrane: "membrane",
};

interface Props {
  active: boolean;
  pendingAction: "folder" | "membrane" | null;
  zoneLabelRef: RefObject<HTMLDivElement | null>;
}

export default function ZoneSelectorOverlay({ active, pendingAction, zoneLabelRef }: Props) {
  if (!active) return null;

  const actionName = pendingAction ? ACTION_LABELS[pendingAction] : "zone";

  return (
    <div style={{ position: "absolute", inset: 0, zIndex: 10, pointerEvents: "none", cursor: "crosshair" }}>
      {/* Instruction banner */}
      <div style={{
        position: "absolute", top: 16, left: "50%", transform: "translateX(-50%)",
        background: "rgba(15, 15, 25, 0.92)", color: "#93c5fd",
        border: "1px solid rgba(59, 130, 246, 0.6)", borderRadius: 6,
        padding: "6px 18px", fontSize: 11, letterSpacing: 0.5,
        backdropFilter: "blur(4px)", whiteSpace: "nowrap",
      }}>
        ✦ Glisse pour définir la zone de ton {actionName} · Échap pour annuler
      </div>

      {/* Live dimensions label — mis à jour impérativement via ref, sans re-render */}
      <div
        ref={zoneLabelRef}
        style={{
          position: "absolute",
          display: "none",
          background: "rgba(15, 15, 25, 0.9)",
          color: "#93c5fd",
          border: "1px solid rgba(59, 130, 246, 0.45)",
          borderRadius: 4,
          padding: "2px 8px",
          fontSize: 11,
          fontVariantNumeric: "tabular-nums",
          letterSpacing: 0.3,
          pointerEvents: "none",
        }}
      />
    </div>
  );
}
