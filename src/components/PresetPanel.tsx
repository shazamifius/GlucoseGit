import { useState } from "react";
import { useGlucoseStore, getActiveBoard } from "../store";
import { PresetSlot } from "../types";

function PresetThumb({ slots, width = 220, height = 56 }: { slots: PresetSlot[]; width?: number; height?: number }) {
  const n    = slots.length;
  const gap  = 4;
  const slotW = (width - gap * (n - 1)) / n;
  return (
    <svg width={width} height={height} style={{ display: "block", borderRadius: 3, overflow: "hidden" }}>
      {slots.map((slot, i) => (
        <g key={slot.id}>
          <rect
            x={i * (slotW + gap)} y={0}
            width={slotW} height={height}
            rx={2} fill={slot.color} fillOpacity={0.18}
            stroke={slot.color} strokeOpacity={0.45} strokeWidth={1}
          />
          <text
            x={i * (slotW + gap) + slotW / 2} y={height / 2}
            textAnchor="middle" dominantBaseline="middle"
            fill={slot.color} fillOpacity={0.8}
            fontSize={9} fontFamily="system-ui, sans-serif"
          >
            {slot.name}
          </text>
        </g>
      ))}
    </svg>
  );
}

interface Props {
  onClose: () => void;
}

export default function PresetPanel({ onClose }: Props) {
  const { project, applyPresetToBoard, getAllPresets } = useGlucoseStore();
  const board = getActiveBoard(project);
  const allPresets = getAllPresets();
  const activePreset = board.presetId ? allPresets.find((p) => p.id === board.presetId) : null;
  const [hovered, setHovered] = useState<string | null>(null);

  function apply(presetId: string | null) {
    if (presetId === null) {
      applyPresetToBoard(board.id, null);
      window.dispatchEvent(new CustomEvent("glucose:layout-preview", { detail: null }));
      onClose();
      return;
    }
    // Enter placement mode — user clicks on canvas to place
    const preset = allPresets.find((p) => p.id === presetId);
    if (!preset) return;
    window.dispatchEvent(new CustomEvent("glucose:layout-preview", {
      detail: { type: "preset", slots: preset.slots, presetId, locked: true },
    }));
    onClose();
  }

  return (
    <div style={{
      position: "absolute", top: 0, right: 0, bottom: 0,
      width: 280, background: "#111", borderLeft: "1px solid #222",
      display: "flex", flexDirection: "column", zIndex: 100,
    }}>
      {/* Header */}
      <div style={{
        display: "flex", alignItems: "center", justifyContent: "space-between",
        padding: "12px 16px", borderBottom: "1px solid #1e1e1e",
      }}>
        <span style={{ color: "#ccc", fontSize: 13, fontWeight: 600, letterSpacing: 1, textTransform: "uppercase" }}>
          Presets
        </span>
        <button onClick={onClose} style={{ background: "none", border: "none", color: "#555", cursor: "pointer", fontSize: 18, padding: 0 }}>×</button>
      </div>

      {/* Board actuel */}
      {activePreset && (
        <div style={{ padding: "12px 16px", borderBottom: "1px solid #1a1a1a", background: "#161616" }}>
          <div style={{ fontSize: 10, color: "#555", textTransform: "uppercase", letterSpacing: 1, marginBottom: 6 }}>
            Board actif : {board.name}
          </div>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
            <span style={{ color: "#aaa", fontSize: 12 }}>{activePreset.name}</span>
            <button
              onClick={() => apply(null)}
              style={{
                fontSize: 11, padding: "2px 8px", background: "transparent",
                border: "1px solid #333", borderRadius: 3, color: "#666", cursor: "pointer",
              }}
            >
              Retirer
            </button>
          </div>
          {/* Slots progress */}
          <div style={{ marginTop: 10, display: "flex", flexWrap: "wrap", gap: 4 }}>
            {activePreset.slots.map((slot) => {
              const filled = board.images.some((img) => img.slotId === slot.id);
              return (
                <div key={slot.id} style={{
                  fontSize: 10, padding: "2px 7px", borderRadius: 10,
                  background: filled ? slot.color + "22" : "#1a1a1a",
                  border: `1px solid ${filled ? slot.color + "66" : "#2a2a2a"}`,
                  color: filled ? slot.color : "#444",
                }}>
                  {filled ? "✓" : "○"} {slot.name}
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Liste des presets */}
      <div style={{ flex: 1, overflowY: "auto", padding: "8px 0" }}>
        {!activePreset && (
          <div style={{ padding: "8px 16px 4px", fontSize: 10, color: "#444", textTransform: "uppercase", letterSpacing: 1 }}>
            Choisir un preset pour "{board.name}"
          </div>
        )}
        {activePreset && (
          <div style={{ padding: "8px 16px 4px", fontSize: 10, color: "#444", textTransform: "uppercase", letterSpacing: 1 }}>
            Changer de preset
          </div>
        )}

        {allPresets.map((preset) => {
          const isActive = board.presetId === preset.id;
          const isHov = hovered === preset.id;
          return (
            <div
              key={preset.id}
              onClick={() => apply(preset.id)}
              onMouseEnter={() => {
                setHovered(preset.id);
                window.dispatchEvent(new CustomEvent("glucose:layout-preview", {
                  detail: { type: "preset", slots: preset.slots },
                }));
              }}
              onMouseLeave={() => {
                setHovered(null);
                window.dispatchEvent(new CustomEvent("glucose:layout-preview", { detail: null }));
              }}
              style={{
                padding: "10px 16px", cursor: "pointer",
                background: isActive ? "#1e1e1e" : isHov ? "#161616" : "transparent",
                borderLeft: isActive ? `2px solid ${preset.slots[0]?.color ?? "#fff"}` : "2px solid transparent",
                transition: "all 0.1s",
              }}
            >
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 6 }}>
                <span style={{ color: isActive ? "#fff" : "#aaa", fontSize: 13, fontWeight: isActive ? 600 : 400 }}>
                  {preset.name}
                </span>
                {isActive && <span style={{ fontSize: 10, color: "#555" }}>actif</span>}
              </div>

              {/* Miniature visuelle du layout */}
              <PresetThumb slots={preset.slots} />

              <div style={{ fontSize: 10, color: "#3a3a3a", marginTop: 5 }}>{preset.description}</div>
            </div>
          );
        })}
      </div>

      {/* Footer - créer preset custom */}
      <div style={{ padding: "12px 16px", borderTop: "1px solid #1a1a1a" }}>
        <button
          style={{
            width: "100%", padding: "7px", fontSize: 12,
            background: "#1a1a1a", color: "#666",
            border: "1px dashed #2a2a2a", borderRadius: 4, cursor: "pointer",
          }}
          onMouseOver={(e) => { e.currentTarget.style.color = "#aaa"; e.currentTarget.style.borderColor = "#444"; }}
          onMouseOut={(e) => { e.currentTarget.style.color = "#666"; e.currentTarget.style.borderColor = "#2a2a2a"; }}
          onClick={() => alert("Création de preset custom — bientôt !")}
        >
          + Créer un preset custom
        </button>
      </div>
    </div>
  );
}
