import { useGlucoseStore, getActiveBoard } from "../store";
import { Annotation, ArrowPredicate } from "../types";

const PREDICATE_OPTIONS: { value: ArrowPredicate; label: string; color: string }[] = [
  { value: "est_precurseur", label: "→ précurseur", color: "#f59e0b" },
  { value: "contredit",      label: "✗ contredit",  color: "#ef4444" },
  { value: "herite_de",      label: "⊂ hérite",     color: "#8b5cf6" },
  { value: "inspire",        label: "✦ inspire",    color: "#10b981" },
  { value: "depend_de",      label: "⊕ dépend",     color: "#3b82f6" },
  { value: "illustre",       label: "◎ illustre",   color: "#f472b6" },
];

interface Props {
  arrow: Annotation;
  onEditText?: () => void;
}

export default function ArrowOptions({ arrow, onEditText }: Props) {
  const { project, updateAnnotation } = useGlucoseStore();
  const boardId = getActiveBoard(project).id;

  function patch(p: Partial<Annotation>) {
    updateAnnotation(boardId, arrow.id, p);
  }

  const sw      = arrow.strokeWidth ?? 2;
  const curved  = arrow.arrowType === "curved";
  const bi      = arrow.arrowBidirectional ?? false;
  const hasTextSel = !!(arrow.sourceTextSel || arrow.targetTextSel);

  const btnBase: React.CSSProperties = {
    padding: "3px 8px", fontSize: 11, borderRadius: 3,
    border: "1px solid #333", cursor: "pointer", background: "#1a1a1a", color: "#888",
  };
  const btnActive: React.CSSProperties = { ...btnBase, background: "#2d2d2d", color: "#ccc", borderColor: "#555" };

  return (
    <div style={{
      position: "absolute", bottom: 48, left: "50%", transform: "translateX(-50%)",
      background: "#111", border: "1px solid #2a2a2a", borderRadius: 6,
      padding: "8px 12px", display: "flex", alignItems: "center", gap: 10,
      zIndex: 300, boxShadow: "0 4px 20px rgba(0,0,0,0.7)",
      pointerEvents: "all", flexWrap: "wrap", maxWidth: 700,
    }}>

      <span style={{ fontSize: 10, color: "#444", textTransform: "uppercase", letterSpacing: 1 }}>Flèche</span>
      <div style={{ width: 1, height: 16, background: "#2a2a2a" }} />

      <button style={curved ? btnBase : btnActive} onClick={() => patch({ arrowType: "straight" })}>Droite</button>
      <button style={curved ? btnActive : btnBase} onClick={() => patch({ arrowType: "curved" })}>Courbe</button>
      <div style={{ width: 1, height: 16, background: "#2a2a2a" }} />

      <button style={bi ? btnActive : btnBase} onClick={() => patch({ arrowBidirectional: !bi })} title="Double sens">⇄</button>
      <div style={{ width: 1, height: 16, background: "#2a2a2a" }} />

      <span style={{ fontSize: 10, color: "#555" }}>Épais.</span>
      {[1, 2, 3, 5].map((w) => (
        <button key={w} onClick={() => patch({ strokeWidth: w })} style={sw === w ? btnActive : btnBase}>{w}</button>
      ))}
      <div style={{ width: 1, height: 16, background: "#2a2a2a" }} />

      {/* Bouton d'édition de texte précis */}
      {onEditText && (arrow.sourceId || arrow.targetId) && (
        <button
          onClick={onEditText}
          style={{
            ...btnBase,
            color: hasTextSel ? "#ccc" : "#888",
            borderColor: hasTextSel ? "#555" : "#333",
            background: hasTextSel ? "#222" : "#1a1a1a",
            display: "flex", alignItems: "center", gap: 4,
          }}
          title={hasTextSel ? "Modifier le texte lié" : "Sélectionner le texte exact"}
        >
          <svg width="10" height="10" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" style={{ flexShrink: 0 }}>
            <path d="M11.5 1.5l3 3L5 14H2v-3z" /><path d="M9.5 3.5l3 3" />
          </svg>
          {hasTextSel ? "Édité" : "Éditer"}
        </button>
      )}
      <div style={{ width: 1, height: 16, background: "#2a2a2a" }} />

      <select
        value={arrow.predicate ?? ""}
        onChange={(e) => patch({ predicate: (e.target.value as ArrowPredicate) || undefined })}
        onPointerDown={(e) => e.stopPropagation()}
        style={{
          background: "#1a1a1a", border: "1px solid #333", borderRadius: 3,
          color: arrow.predicate ? (PREDICATE_OPTIONS.find(p => p.value === arrow.predicate)?.color ?? "#888") : "#555",
          fontSize: 11, padding: "2px 4px", cursor: "pointer", outline: "none",
        }}
      >
        <option value="">Prédicat…</option>
        {PREDICATE_OPTIONS.map((p) => (
          <option key={p.value} value={p.value}>{p.label}</option>
        ))}
      </select>

      <input
        value={arrow.text ?? ""}
        onChange={(e) => patch({ text: e.target.value || undefined })}
        onPointerDown={(e) => e.stopPropagation()}
        onKeyDown={(e) => e.stopPropagation()}
        placeholder="Label…"
        style={{
          background: "#1a1a1a", border: "1px solid #333", borderRadius: 3,
          color: "#888", fontSize: 11, padding: "2px 6px", width: 80, outline: "none",
        }}
      />
    </div>
  );
}
