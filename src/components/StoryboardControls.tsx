import { useState } from "react";
import { useGlucoseStore, getActiveBoard } from "../store";
import { AspectRatio, StoryboardPanel } from "../types";
import { nanoid } from "../utils/nanoid";
import { aspectRatioToFloat } from "../utils/layout";

function StoryboardThumb({ cols, ratio, count, width = 220 }: { cols: number; ratio: number; count: number; width?: number }) {
  const safeCount = Math.max(1, Math.min(count, 24));
  const safeCols  = Math.max(1, cols);
  const rows      = Math.ceil(safeCount / safeCols);
  const gap       = 3;
  const cellW     = (width - gap * (safeCols - 1)) / safeCols;
  const cellH     = cellW / Math.max(0.3, ratio);
  const height    = cellH * rows + gap * (rows - 1);
  return (
    <svg width={width} height={Math.min(height, 120)} style={{ display: "block", overflow: "hidden" }}>
      {Array.from({ length: safeCount }).map((_, idx) => {
        const col = idx % safeCols;
        const row = Math.floor(idx / safeCols);
        const x   = col * (cellW + gap);
        const y   = row * (cellH + gap);
        if (y + cellH > 122) return null;
        return (
          <g key={idx}>
            <rect x={x} y={y} width={cellW} height={cellH} rx={1}
              fill="rgba(255,255,255,0.04)" stroke="rgba(255,255,255,0.18)" strokeWidth={0.8} />
            <text x={x + cellW / 2} y={y + cellH / 2}
              textAnchor="middle" dominantBaseline="middle"
              fill="rgba(255,255,255,0.25)" fontSize={8} fontFamily="system-ui">
              {idx + 1}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

interface Props {
  docked?: boolean;
}

const ASPECT_RATIOS: { value: AspectRatio; label: string }[] = [
  { value: "16:9", label: "16:9 — Cinéma HD" },
  { value: "4:3", label: "4:3 — Classique" },
  { value: "2.35:1", label: "2.35:1 — Scope" },
  { value: "1:1", label: "1:1 — Carré" },
  { value: "9:16", label: "9:16 — Vertical" },
];

export default function StoryboardControls({ docked }: Props) {
  const {
    project, clearStoryboard, addPanel, updatePanel, removePanel, reorderPanels,
  } = useGlucoseStore();
  const board = getActiveBoard(project);
  const settings = board.storyboard;

  const [ar, setAr] = useState<AspectRatio>(settings?.aspectRatio ?? "16:9");
  const [panelW, setPanelW] = useState(settings?.panelWidth ?? 280);
  const [cols, setCols] = useState(settings?.cols ?? 4);
  const [gap, setGap] = useState(settings?.gap ?? 24);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editDesc, setEditDesc] = useState("");

  function getGridOrigin() {
    const vp = board.viewport;
    return { x: -vp.x / vp.scale, y: -vp.y / vp.scale };
  }

  function addNewPanel() {
    const s = board.storyboard;
    if (!s) return;
    const ratio = aspectRatioToFloat(s.aspectRatio);
    const w = s.panelWidth;
    const h = w / ratio;
    const descH = 40;
    const count = board.panels.length;
    const col = count % s.cols;
    const row = Math.floor(count / s.cols);

    // Compute grid origin: first panel position, or viewport top-left for the very first panel
    let originX: number;
    let originY: number;
    if (count > 0) {
      const sorted = [...board.panels].sort((a, b) => a.order - b.order);
      const first = sorted[0];
      const firstCol = first.order % s.cols;
      const firstRow = Math.floor(first.order / s.cols);
      originX = first.x - firstCol * (w + s.gap);
      originY = first.y - firstRow * (h + descH + s.gap);
    } else {
      const o = getGridOrigin();
      originX = o.x;
      originY = o.y;
    }

    const panel: StoryboardPanel = {
      id: nanoid(),
      order: count,
      description: "",
      x: originX + col * (w + s.gap),
      y: originY + row * (h + descH + s.gap),
      width: w,
      height: h,
    };
    addPanel(board.id, panel);
  }

  const inputStyle: React.CSSProperties = {
    background: "#1a1a1a", border: "1px solid #2a2a2a", borderRadius: 3,
    color: "#ccc", fontSize: 12, padding: "3px 7px", width: "100%", outline: "none",
  };
  const labelStyle: React.CSSProperties = {
    fontSize: 10, color: "#555", textTransform: "uppercase" as const, letterSpacing: 1, marginBottom: 4,
  };

  return (
    <div style={{
      ...(docked ? {} : { position: "absolute", bottom: 60, left: 310, zIndex: 200, boxShadow: "0 8px 32px rgba(0,0,0,0.6)" }),
      width: 260, background: "#111", border: "1px solid #222", borderRadius: 6,
      maxHeight: "80vh", overflowY: "auto",
    }}>
      {/* Header */}
      <div style={{
        display: "flex", alignItems: "center", justifyContent: "space-between",
        padding: "10px 14px", borderBottom: "1px solid #1e1e1e",
        position: "sticky", top: 0, background: "#111", zIndex: 1,
      }}>
        <span style={{ color: "#ccc", fontSize: 12, fontWeight: 600, letterSpacing: 1, textTransform: "uppercase" }}>
          Storyboard
        </span>
      </div>

      <div style={{ padding: "12px 14px", display: "flex", flexDirection: "column", gap: 12 }}>

        {/* Settings */}
        <div>
          <div style={labelStyle}>Format</div>
          <select
            value={ar}
            onChange={(e) => setAr(e.target.value as AspectRatio)}
            style={{ ...inputStyle, cursor: "pointer", colorScheme: "dark" }}
          >
            {ASPECT_RATIOS.map((a) => (
              <option key={a.value} value={a.value}>{a.label}</option>
            ))}
          </select>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 8 }}>
          <div>
            <div style={labelStyle}>Largeur</div>
            <input type="number" min={80} max={800} step={10} value={panelW}
              onChange={(e) => setPanelW(Number(e.target.value))} style={inputStyle} />
          </div>
          <div>
            <div style={labelStyle}>Colonnes</div>
            <input type="number" min={1} max={12} step={1} value={cols}
              onChange={(e) => setCols(Number(e.target.value))} style={inputStyle} />
          </div>
          <div>
            <div style={labelStyle}>Espacement</div>
            <input type="number" min={0} max={100} step={4} value={gap}
              onChange={(e) => setGap(Number(e.target.value))} style={inputStyle} />
          </div>
        </div>

        {/* Miniature live du layout */}
        <div style={{
          background: "#0d0d0d", borderRadius: 4, padding: 8,
          border: "1px solid #1e1e1e",
        }}>
          <StoryboardThumb
            cols={cols}
            ratio={aspectRatioToFloat(ar)}
            count={Math.max(cols * 2, board.panels.length || cols * 2)}
          />
        </div>

        <div style={{ display: "flex", gap: 6 }}>
          <button
            onClick={() => {
              const ratio = aspectRatioToFloat(ar);
              const count = Math.max(cols * 2, board.panels.length || cols * 2);
              window.dispatchEvent(new CustomEvent("glucose:layout-preview", {
                detail: {
                  type: "storyboard", locked: true,
                  cols, ratio, count,
                  panelWidth: panelW, gap, aspectRatio: ar,
                },
              }));
            }}
            onMouseEnter={() => window.dispatchEvent(new CustomEvent("glucose:layout-preview", {
              detail: { type: "storyboard", cols, ratio: aspectRatioToFloat(ar), count: Math.max(cols * 2, board.panels.length || cols * 2), panelWidth: panelW, gap },
            }))}
            onMouseLeave={() => window.dispatchEvent(new CustomEvent("glucose:layout-preview", { detail: null }))}
            style={{
              flex: 1, padding: "6px", fontSize: 12, fontWeight: 600,
              background: "#222", color: "#ccc", border: "1px solid #333",
              borderRadius: 4, cursor: "pointer",
            }}
            onMouseOver={(e) => { e.currentTarget.style.background = "#2a2a2a"; }}
            onMouseOut={(e) => { e.currentTarget.style.background = "#222"; }}
          >
            {settings ? "Mettre à jour" : "Activer"}
          </button>
          {settings && (
            <button
              onClick={() => clearStoryboard(board.id)}
              style={{
                padding: "6px 10px", fontSize: 11, background: "transparent",
                color: "#555", border: "1px solid #2a2a2a", borderRadius: 4, cursor: "pointer",
              }}
              onMouseOver={(e) => { e.currentTarget.style.color = "#f87171"; }}
              onMouseOut={(e) => { e.currentTarget.style.color = "#555"; }}
              title="Désactiver le storyboard"
            >
              ✕
            </button>
          )}
        </div>

        {/* Panels list */}
        {settings && (
          <>
            <div style={{ borderTop: "1px solid #1a1a1a", paddingTop: 12 }}>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 8 }}>
                <span style={{ ...labelStyle, margin: 0 }}>Panels ({board.panels.length})</span>
                <button
                  onClick={addNewPanel}
                  style={{
                    fontSize: 11, padding: "2px 8px", background: "#1a1a1a",
                    color: "#888", border: "1px solid #2a2a2a", borderRadius: 3, cursor: "pointer",
                  }}
                  onMouseOver={(e) => { e.currentTarget.style.color = "#ccc"; }}
                  onMouseOut={(e) => { e.currentTarget.style.color = "#888"; }}
                >
                  + Ajouter
                </button>
              </div>

              <div style={{ display: "flex", flexDirection: "column", gap: 4, maxHeight: 240, overflowY: "auto", paddingRight: 14, scrollbarWidth: "thin" as const }}>
                {[...board.panels].sort((a, b) => a.order - b.order).map((panel) => (
                  <div
                    key={panel.id}
                    style={{
                      display: "flex", alignItems: "center", gap: 6,
                      padding: "4px 8px", background: "#161616", borderRadius: 3,
                    }}
                  >
                    <span style={{ color: "#555", fontSize: 10, minWidth: 16 }}>{panel.order + 1}</span>
                    {editingId === panel.id ? (
                      <input
                        autoFocus
                        value={editDesc}
                        onChange={(e) => setEditDesc(e.target.value)}
                        onBlur={() => {
                          updatePanel(board.id, panel.id, { description: editDesc });
                          setEditingId(null);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") {
                            updatePanel(board.id, panel.id, { description: editDesc });
                            setEditingId(null);
                          }
                          if (e.key === "Escape") setEditingId(null);
                        }}
                        style={{
                          flex: 1, background: "#1a1a1a", border: "1px solid #333",
                          color: "#ccc", fontSize: 11, padding: "2px 5px", borderRadius: 2, outline: "none",
                        }}
                      />
                    ) : (
                      <span
                        onClick={() => { setEditingId(panel.id); setEditDesc(panel.description); }}
                        style={{
                          flex: 1, color: panel.description ? "#888" : "#333",
                          fontSize: 11, cursor: "text",
                          fontStyle: panel.description ? "normal" : "italic",
                        }}
                      >
                        {panel.description || "Description..."}
                      </span>
                    )}
                    <button
                      onClick={() => removePanel(board.id, panel.id)}
                      style={{ background: "none", border: "none", color: "#333", cursor: "pointer", fontSize: 13, padding: 0, lineHeight: 1 }}
                      onMouseOver={(e) => { e.currentTarget.style.color = "#f87171"; }}
                      onMouseOut={(e) => { e.currentTarget.style.color = "#333"; }}
                    >
                      ×
                    </button>
                  </div>
                ))}
              </div>

              {board.panels.length > 0 && (
                <button
                  onClick={() => reorderPanels(board.id)}
                  style={{
                    marginTop: 6, width: "100%", padding: "4px", fontSize: 10,
                    background: "transparent", color: "#444",
                    border: "1px dashed #2a2a2a", borderRadius: 3, cursor: "pointer",
                  }}
                  onMouseOver={(e) => { e.currentTarget.style.color = "#888"; }}
                  onMouseOut={(e) => { e.currentTarget.style.color = "#444"; }}
                >
                  Re-numéroter les panels
                </button>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
