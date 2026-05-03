import { useEffect, useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { useGlucoseStore, getActiveBoard } from "../store";
// CLEANUP B-03 : CSS KaTeX chargé à la demande quand le panel s'ouvre.
import { ensureKatexCss } from "../utils/loadKatexCss";

/**
 * Phase 5 — Flèche déroulante (point D)
 *
 * Panneau coulissant attaché à une flèche pour porter une description longue
 * (Markdown). Ouvert via badge "i" sur la flèche, fermé via Échap ou ×.
 *
 * Position : à droite du milieu de la flèche, en coordonnées écran.
 */

interface Props {
  arrowId: string;
  midX: number;       // position écran (déjà calculée par GlucoseCanvas)
  midY: number;
  onClose: () => void;
}

export default function ArrowDescriptionPanel({ arrowId, midX, midY, onClose }: Props) {
  const { project, updateAnnotation } = useGlucoseStore();
  const board = getActiveBoard(project);
  const arrow = board.annotations.find(a => a.id === arrowId);
  const initial = arrow?.longText ?? "";
  const [text, setText] = useState(initial);
  const [editing, setEditing] = useState(initial.length === 0); // si vide, on ouvre direct en édition
  const taRef = useRef<HTMLTextAreaElement>(null);

  // Précharge le CSS KaTeX dès l'ouverture du panel (utilisateur peut écrire du LaTeX)
  useEffect(() => { void ensureKatexCss(); }, []);

  // Échap → fermer
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") { e.stopPropagation(); commit(); onClose(); }
    }
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, []);

  useEffect(() => {
    if (editing && taRef.current) taRef.current.focus();
  }, [editing]);

  function commit() {
    if (text === initial) return;
    updateAnnotation(board.id, arrowId, { longText: text || undefined });
  }

  if (!arrow) return null;

  // Style positionné en absolute sur le wrapper canvas, ajusté pour ne pas sortir de l'écran
  const PANEL_W = 360;
  const PANEL_H_MAX = 480;
  const margin = 16;
  let left = midX + 24;
  let top = midY - 60;
  // Clamp à l'écran (approximatif, on s'appuie sur viewport)
  if (typeof window !== "undefined") {
    if (left + PANEL_W + margin > window.innerWidth) left = midX - PANEL_W - 24;
    if (top + PANEL_H_MAX + margin > window.innerHeight) top = window.innerHeight - PANEL_H_MAX - margin;
    if (top < margin) top = margin;
  }

  return (
    <div
      style={{
        position: "fixed", left, top,
        width: PANEL_W, maxHeight: PANEL_H_MAX,
        zIndex: 400, background: "#141414",
        border: "1px solid #2a2a2a", borderRadius: 10,
        boxShadow: "0 12px 48px rgba(0,0,0,0.7), 0 0 24px rgba(147,197,253,0.08)",
        display: "flex", flexDirection: "column",
        animation: "glucose-arrow-desc-in 180ms ease-out",
      }}
      onPointerDown={(e) => e.stopPropagation()}
      onClick={(e) => e.stopPropagation()}
      onWheel={(e) => e.stopPropagation()}
    >
      {/* Header */}
      <div style={{
        display: "flex", alignItems: "center", justifyContent: "space-between",
        padding: "10px 14px", borderBottom: "1px solid #1e1e1e",
      }}>
        <span style={{ fontSize: 11, color: "#93c5fd", letterSpacing: 1.5, textTransform: "uppercase", fontWeight: 600 }}>
          Description de la flèche
        </span>
        <div style={{ display: "flex", gap: 4 }}>
          <button
            onClick={() => setEditing((v) => !v)}
            title={editing ? "Aperçu" : "Éditer"}
            style={{
              background: "transparent", border: "none", color: "#666",
              cursor: "pointer", fontSize: 13, padding: "2px 8px",
            }}
          >{editing ? "👁" : "✎"}</button>
          <button
            onClick={() => { commit(); onClose(); }}
            title="Fermer (Échap)"
            style={{ background: "transparent", border: "none", color: "#666", cursor: "pointer", fontSize: 16, padding: "0 6px" }}
          >×</button>
        </div>
      </div>

      {/* Body */}
      <div style={{ flex: 1, overflowY: "auto", padding: "12px 14px" }}>
        {editing ? (
          <textarea
            ref={taRef}
            value={text}
            onChange={(e) => setText(e.target.value)}
            onBlur={commit}
            placeholder="Markdown : titres, listes, gras, **liens**, $LaTeX$, `code`..."
            style={{
              width: "100%", minHeight: 200, height: 320,
              background: "#0d0d0d", border: "1px solid #1e1e1e",
              borderRadius: 6, padding: "10px 12px",
              color: "#d4d4d4", fontSize: 13, lineHeight: 1.55,
              fontFamily: "system-ui, -apple-system, sans-serif",
              resize: "vertical", outline: "none",
            }}
          />
        ) : (
          <div className="prose prose-invert prose-sm max-w-none" style={{ fontSize: 13, lineHeight: 1.6, color: "#d4d4d4" }}>
            {text.trim()
              ? <ReactMarkdown remarkPlugins={[remarkGfm, remarkMath]} rehypePlugins={[rehypeKatex]}>{text}</ReactMarkdown>
              : <span style={{ color: "#555", fontStyle: "italic" }}>Aucune description. Clique ✎ pour en ajouter une.</span>}
          </div>
        )}
      </div>

      {/* Footer */}
      <div style={{
        padding: "8px 14px", borderTop: "1px solid #1e1e1e",
        fontSize: 10, color: "#444", display: "flex", justifyContent: "space-between",
      }}>
        <span>Markdown + LaTeX supportés</span>
        <span>Échap pour fermer</span>
      </div>

      <style>{`
        @keyframes glucose-arrow-desc-in {
          from { opacity: 0; transform: translateX(-8px) scale(0.96); }
          to   { opacity: 1; transform: translateX(0) scale(1); }
        }
      `}</style>
    </div>
  );
}
