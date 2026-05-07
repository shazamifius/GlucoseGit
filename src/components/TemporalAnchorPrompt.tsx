// Phase 6 — Modal léger pour assigner une date à la sélection courante.
// Déclenché par Shift+T quand au moins un nœud est sélectionné.
//
// Saisies acceptées (cf. timeline.parseAnchor) :
//   1789 · 1789-1799 · -500 · 500 av JC · Renaissance · 10 ka · 1,5 Ma
//
// Suggestions auto-complétées tirées de DEFAULT_ERAS.

import { useEffect, useMemo, useRef, useState } from "react";
import { useGlucoseStore } from "../store";
import { parseAnchor, formatAnchor, DEFAULT_ERAS } from "../utils/timeline";
import type { TemporalAnchor } from "../types";

interface Props {
  onClose: () => void;
}

export default function TemporalAnchorPrompt({ onClose }: Props) {
  const [value, setValue] = useState("");
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const project = useGlucoseStore((s) => s.project);
  const selectedAnnotationIds = useGlucoseStore((s) => s.selectedAnnotationIds);
  const selectedImageIds = useGlucoseStore((s) => s.selectedImageIds);
  const updateAnnotation = useGlucoseStore((s) => s.updateAnnotation);
  const updateImage = useGlucoseStore((s) => s.updateImage);
  const board = project.boards.find((b) => b.id === project.activeBoardId);

  // Pré-remplit avec la valeur déjà ancrée si une seule sélection commune
  useEffect(() => {
    if (!board) return;
    const anchors: TemporalAnchor[] = [];
    for (const id of selectedAnnotationIds) {
      const a = board.annotations.find((x) => x.id === id)?.temporalAnchor;
      if (a) anchors.push(a);
    }
    for (const id of selectedImageIds) {
      const a = board.images.find((x) => x.id === id)?.temporalAnchor;
      if (a) anchors.push(a);
    }
    if (anchors.length > 0) setValue(formatAnchor(anchors[0]));
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  const suggestions = useMemo(() => {
    const q = value.trim().toLowerCase();
    if (!q) return DEFAULT_ERAS.slice(0, 6);
    return DEFAULT_ERAS
      .filter((e) => e.name.toLowerCase().includes(q))
      .slice(0, 8);
  }, [value]);

  function apply(text: string) {
    const parsed = parseAnchor(text);
    if (!parsed) {
      setError("Date non reconnue. Essaye « 1789 », « -500 », « Renaissance », « 10 ka »…");
      return;
    }
    if (!board) return;
    for (const id of selectedAnnotationIds) {
      updateAnnotation(board.id, id, { temporalAnchor: parsed });
    }
    for (const id of selectedImageIds) {
      updateImage(board.id, id, { temporalAnchor: parsed });
    }
    onClose();
  }

  function clear() {
    if (!board) return;
    for (const id of selectedAnnotationIds) {
      updateAnnotation(board.id, id, { temporalAnchor: undefined });
    }
    for (const id of selectedImageIds) {
      updateImage(board.id, id, { temporalAnchor: undefined });
    }
    onClose();
  }

  const total = selectedAnnotationIds.length + selectedImageIds.length;

  return (
    <div
      onClick={onClose}
      style={{
        position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)",
        display: "flex", alignItems: "center", justifyContent: "center", zIndex: 200,
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          background: "#1a1a1a", border: "1px solid #333", borderRadius: 8,
          padding: 20, width: 420,
          boxShadow: "0 24px 48px rgba(0,0,0,0.6)",
          fontFamily: "system-ui, sans-serif",
        }}
      >
        <div style={{ color: "#fde68a", fontSize: 12, letterSpacing: 0.6, marginBottom: 6 }}>
          📅 ANCRAGE TEMPOREL · {total} nœud{total > 1 ? "s" : ""}
        </div>
        <input
          ref={inputRef}
          value={value}
          onChange={(e) => { setValue(e.target.value); setError(null); }}
          onKeyDown={(e) => {
            if (e.key === "Enter") apply(value);
            if (e.key === "Escape") onClose();
          }}
          placeholder="1789 · -500 · Renaissance · 10 ka …"
          style={{
            width: "100%", padding: "10px 12px",
            background: "#0d0d0d", color: "#f3f4f6",
            border: `1px solid ${error ? "#ef4444" : "#444"}`,
            borderRadius: 6, fontSize: 14, outline: "none", boxSizing: "border-box",
          }}
        />
        {error && (
          <div style={{ color: "#fca5a5", fontSize: 11, marginTop: 6 }}>{error}</div>
        )}

        {suggestions.length > 0 && (
          <div style={{ marginTop: 12, display: "flex", flexWrap: "wrap", gap: 6 }}>
            {suggestions.map((s) => (
              <button
                key={s.name}
                onClick={() => apply(s.name)}
                title={s.description ?? `${s.start}..${s.end}`}
                style={{
                  background: "#222", color: "#cbd5e1",
                  border: "1px solid #333", borderRadius: 4,
                  padding: "4px 8px", fontSize: 11, cursor: "pointer",
                }}
              >
                {s.name}
              </button>
            ))}
          </div>
        )}

        <div style={{ display: "flex", justifyContent: "space-between", marginTop: 16 }}>
          <button
            onClick={clear}
            style={{
              background: "transparent", color: "#aaa",
              border: "1px solid #444", borderRadius: 4,
              padding: "6px 12px", fontSize: 12, cursor: "pointer",
            }}
          >
            Retirer l'ancrage
          </button>
          <div style={{ display: "flex", gap: 8 }}>
            <button
              onClick={onClose}
              style={{
                background: "transparent", color: "#aaa",
                border: "1px solid #444", borderRadius: 4,
                padding: "6px 12px", fontSize: 12, cursor: "pointer",
              }}
            >
              Annuler
            </button>
            <button
              onClick={() => apply(value)}
              style={{
                background: "#fbbf24", color: "#0d0d0d",
                border: "none", borderRadius: 4,
                padding: "6px 14px", fontSize: 12, cursor: "pointer", fontWeight: 600,
              }}
            >
              Ancrer
            </button>
          </div>
        </div>

        <div style={{ marginTop: 12, fontSize: 10, color: "#666" }}>
          Entrée pour valider · Échap pour fermer · les nœuds sans ancrage restent toujours visibles.
        </div>
      </div>
    </div>
  );
}
