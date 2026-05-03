import { useState, useRef, useEffect } from "react";
import { useGlucoseStore } from "../store";

interface SearchResult {
  boardId: string;
  boardName: string;
  type: "board" | "image" | "text" | "sticky";
  label: string;
  x?: number;
  y?: number;
}

interface Props {
  onClose: () => void;
}

const TYPE_ICON: Record<string, string> = { board: "☰", image: "▣", text: "T", sticky: "N" };
const TYPE_LABEL: Record<string, string> = { board: "Board", image: "Image", text: "Texte", sticky: "Note" };

export default function SearchPanel({ onClose }: Props) {
  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const { project, setActiveBoardId } = useGlucoseStore();

  useEffect(() => { inputRef.current?.focus(); }, []);

  const results: SearchResult[] = [];
  if (query.trim().length >= 1) {
    const q = query.trim().toLowerCase();
    project.boards.forEach((board) => {
      if (board.name.toLowerCase().includes(q)) {
        results.push({ boardId: board.id, boardName: board.name, type: "board", label: board.name });
      }
      board.images.forEach((img) => {
        const matched = img.tags?.filter((t) => t.toLowerCase().includes(q)) ?? [];
        if (matched.length > 0) {
          results.push({
            boardId: board.id, boardName: board.name, type: "image",
            label: `Tags: ${matched.join(", ")}`,
            x: img.x, y: img.y,
          });
        }
      });
      board.annotations.forEach((ann) => {
        if (ann.text?.toLowerCase().includes(q)) {
          results.push({
            boardId: board.id, boardName: board.name,
            type: ann.type === "sticky" ? "sticky" : "text",
            label: ann.text ?? "",
            x: ann.x, y: ann.y,
          });
        }
      });
    });
  }

  function navigate(result: SearchResult) {
    setActiveBoardId(result.boardId);
    if (result.x !== undefined && result.y !== undefined) {
      setTimeout(() => {
        window.dispatchEvent(new CustomEvent("glucose:jump-viewport", {
          detail: { wx: result.x, wy: result.y },
        }));
      }, 50);
    }
    onClose();
  }

  const shown = results.slice(0, 20);

  return (
    <>
      <div style={{ position: "fixed", inset: 0, zIndex: 9990 }} onPointerDown={onClose} />

      <div
        style={{
          position: "fixed", top: 80, left: "50%", transform: "translateX(-50%)",
          width: 480, maxWidth: "90vw",
          background: "#1a1a1a", border: "1px solid #333", borderRadius: 8,
          boxShadow: "0 8px 32px rgba(0,0,0,0.6)", zIndex: 9991, overflow: "hidden",
        }}
        onPointerDown={(e) => e.stopPropagation()}
      >
        <div style={{
          display: "flex", alignItems: "center", gap: 8,
          padding: "10px 14px", borderBottom: "1px solid #2a2a2a",
        }}>
          <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
            <circle cx="6" cy="6" r="4" stroke="#555" strokeWidth="1.3"/>
            <path d="M9.5 9.5L12 12" stroke="#555" strokeWidth="1.3" strokeLinecap="round"/>
          </svg>
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              e.stopPropagation();
              if (e.key === "Escape") onClose();
              if (e.key === "Enter" && shown.length > 0) navigate(shown[0]);
            }}
            placeholder="Textes, notes, tags, noms de boards…"
            style={{
              flex: 1, background: "transparent", border: "none", outline: "none",
              color: "#ccc", fontSize: 13, fontFamily: "inherit",
            }}
          />
          <kbd style={{
            fontSize: 10, color: "#444", border: "1px solid #2a2a2a",
            borderRadius: 3, padding: "1px 5px",
          }}>Esc</kbd>
        </div>

        <div style={{ maxHeight: 320, overflowY: "auto" }}>
          {query.trim().length < 1 ? (
            <div style={{ padding: 20, textAlign: "center", color: "#333", fontSize: 12 }}>
              Tapez pour rechercher…
            </div>
          ) : shown.length === 0 ? (
            <div style={{ padding: 20, textAlign: "center", color: "#333", fontSize: 12 }}>
              Aucun résultat pour « {query} »
            </div>
          ) : (
            shown.map((r, i) => (
              <div
                key={i}
                onClick={() => navigate(r)}
                style={{
                  display: "flex", alignItems: "center", gap: 10,
                  padding: "8px 14px", cursor: "pointer",
                  borderBottom: "1px solid #1e1e1e",
                  transition: "background 0.08s",
                }}
                onMouseOver={(e) => { e.currentTarget.style.background = "#222"; }}
                onMouseOut={(e) => { e.currentTarget.style.background = "transparent"; }}
              >
                <span style={{ fontSize: 10, color: "#555", width: 14, textAlign: "center", flexShrink: 0 }}>
                  {TYPE_ICON[r.type]}
                </span>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{
                    color: "#ccc", fontSize: 12,
                    overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
                  }}>
                    {r.label}
                  </div>
                  <div style={{ color: "#444", fontSize: 10, marginTop: 1 }}>
                    {r.boardName} · {TYPE_LABEL[r.type]}
                  </div>
                </div>
              </div>
            ))
          )}
        </div>

        {results.length > 0 && (
          <div style={{ padding: "5px 14px", borderTop: "1px solid #222", fontSize: 10, color: "#333" }}>
            {results.length} résultat{results.length > 1 ? "s" : ""}{results.length > 20 ? " (20 affichés)" : ""}
          </div>
        )}
      </div>
    </>
  );
}
