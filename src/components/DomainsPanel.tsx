import { useState } from "react";
import { useGlucoseStore, getActiveBoard } from "../store";
import { Domain } from "../types";
import { nanoid } from "../utils/nanoid";

const PRESET_COLORS = [
  "#60a5fa", "#34d399", "#f472b6", "#fbbf24",
  "#a78bfa", "#f87171", "#22d3ee", "#fb923c",
];
const PRESET_ICONS = ["🔬", "🎨", "🎮", "📚", "🌍", "⚛", "✦", "♪", "△", "○", "✿", "❀"];

interface Props {
  onClose: () => void;
}

export default function DomainsPanel({ onClose }: Props) {
  const { project, addDomain, removeDomain, updateDomain, assignDomainToNode,
    selectedAnnotationIds, selectedImageIds } = useGlucoseStore();
  const board = getActiveBoard(project);
  const domains = project.domains ?? [];
  const [editingId, setEditingId] = useState<string | null>(null);

  // Domaines déjà appliqués sur la sélection courante (avec leur poids moyen)
  const selectedNodes = [
    ...board.annotations.filter(a => selectedAnnotationIds.includes(a.id)),
    ...board.images.filter(i => selectedImageIds.includes(i.id)),
  ];
  const selectionDomainWeights = (() => {
    const sums = new Map<string, { sum: number; count: number }>();
    for (const node of selectedNodes) {
      for (const da of node.domains ?? []) {
        const cur = sums.get(da.domainId) ?? { sum: 0, count: 0 };
        sums.set(da.domainId, { sum: cur.sum + da.weight, count: cur.count + 1 });
      }
    }
    const out = new Map<string, number>();
    sums.forEach((v, k) => out.set(k, v.sum / Math.max(1, selectedNodes.length)));
    return out;
  })();

  function createDomain() {
    const idx = domains.length % PRESET_COLORS.length;
    const newDomain: Domain = {
      id: nanoid(),
      name: "Nouveau domaine",
      color: PRESET_COLORS[idx],
      icon: PRESET_ICONS[idx % PRESET_ICONS.length],
      createdAt: Date.now(),
    };
    addDomain(newDomain);
    setEditingId(newDomain.id);
  }

  function applyToSelection(domainId: string, weight: number) {
    if (selectedNodes.length === 0) return;
    for (const node of selectedNodes) {
      assignDomainToNode(board.id, node.id, domainId, weight);
    }
  }

  return (
    <div style={{
      position: "absolute", top: 0, right: 0, bottom: 0,
      width: 320, background: "#111", borderLeft: "1px solid #222",
      display: "flex", flexDirection: "column", zIndex: 100,
    }}>
      {/* Header */}
      <div style={{
        display: "flex", alignItems: "center", justifyContent: "space-between",
        padding: "12px 16px", borderBottom: "1px solid #1e1e1e",
      }}>
        <span style={{ color: "#ccc", fontSize: 13, fontWeight: 600, letterSpacing: 1, textTransform: "uppercase" }}>
          Domaines
        </span>
        <button onClick={onClose} style={{ background: "none", border: "none", color: "#555", cursor: "pointer", fontSize: 18, padding: 0 }}>×</button>
      </div>

      {/* Sélection courante */}
      {selectedNodes.length > 0 && (
        <div style={{
          padding: "10px 16px", borderBottom: "1px solid #1a1a1a",
          background: "#161616", fontSize: 11, color: "#888",
        }}>
          {selectedNodes.length} nœud{selectedNodes.length > 1 ? "s" : ""} sélectionné{selectedNodes.length > 1 ? "s" : ""} — clique sur un domaine pour l'assigner
        </div>
      )}

      {/* Liste des domaines */}
      <div style={{ flex: 1, overflowY: "auto", padding: "8px 0" }}>
        {domains.length === 0 && (
          <div style={{ padding: "20px 16px", color: "#555", fontSize: 12, textAlign: "center", lineHeight: 1.6 }}>
            Aucun domaine.<br />
            Crée-en un pour colorer les membranes selon leur sémantique.
          </div>
        )}

        {domains.map((d) => {
          const editing = editingId === d.id;
          const currentWeight = selectionDomainWeights.get(d.id) ?? 0;
          return (
            <div
              key={d.id}
              style={{
                margin: "4px 12px", padding: "10px 12px",
                background: "#171717", border: `1px solid ${d.color}33`,
                borderRadius: 6, transition: "background 0.15s",
              }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: editing ? 8 : 0 }}>
                {/* Icône */}
                {editing ? (
                  <input
                    value={d.icon}
                    onChange={(e) => updateDomain(d.id, { icon: e.target.value.slice(0, 2) })}
                    style={{
                      width: 32, height: 28, textAlign: "center", fontSize: 16,
                      background: "#222", border: `1px solid ${d.color}55`, color: "#fff",
                      borderRadius: 4, outline: "none",
                    }}
                  />
                ) : (
                  <span style={{ fontSize: 18, width: 28, textAlign: "center" }}>{d.icon}</span>
                )}
                {/* Nom */}
                {editing ? (
                  <input
                    value={d.name}
                    onChange={(e) => updateDomain(d.id, { name: e.target.value })}
                    onKeyDown={(e) => { if (e.key === "Enter" || e.key === "Escape") setEditingId(null); }}
                    autoFocus
                    style={{
                      flex: 1, fontSize: 13, padding: "4px 8px",
                      background: "#222", border: `1px solid ${d.color}55`, color: "#fff",
                      borderRadius: 4, outline: "none",
                    }}
                  />
                ) : (
                  <span style={{ flex: 1, color: d.color, fontSize: 13, fontWeight: 500 }}>{d.name}</span>
                )}
                {/* Actions */}
                <button
                  onClick={() => setEditingId(editing ? null : d.id)}
                  title={editing ? "Terminer" : "Éditer"}
                  style={{
                    background: "none", border: "none", color: "#666", cursor: "pointer",
                    fontSize: 11, padding: "2px 6px",
                  }}
                >{editing ? "✓" : "✎"}</button>
                <button
                  onClick={() => { if (confirm(`Supprimer "${d.name}" ? Tous les nœuds qui le portent seront désassignés.`)) removeDomain(d.id); }}
                  title="Supprimer"
                  style={{
                    background: "none", border: "none", color: "#666", cursor: "pointer",
                    fontSize: 14, padding: "2px 6px",
                  }}
                >×</button>
              </div>

              {/* Palette de couleurs en mode édition */}
              {editing && (
                <div style={{ display: "flex", flexWrap: "wrap", gap: 4, marginBottom: 8 }}>
                  {PRESET_COLORS.map((c) => (
                    <button
                      key={c}
                      onClick={() => updateDomain(d.id, { color: c })}
                      style={{
                        width: 22, height: 22, borderRadius: 4,
                        background: c, border: c === d.color ? "2px solid #fff" : "1px solid #333",
                        cursor: "pointer", padding: 0,
                      }}
                    />
                  ))}
                </div>
              )}

              {/* Slider de poids — visible quand sélection non vide */}
              {selectedNodes.length > 0 && (
                <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 6 }}>
                  <input
                    type="range"
                    min={0} max={1} step={0.1}
                    value={currentWeight}
                    onChange={(e) => applyToSelection(d.id, parseFloat(e.target.value))}
                    style={{ flex: 1, accentColor: d.color }}
                  />
                  <span style={{ fontSize: 10, color: "#888", width: 28, textAlign: "right" }}>
                    {currentWeight > 0 ? `${Math.round(currentWeight * 100)}%` : "—"}
                  </span>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Footer : créer */}
      <button
        onClick={createDomain}
        style={{
          margin: "8px 12px 12px", padding: "10px 12px",
          background: "#1a1a1a", border: "1px dashed #333",
          color: "#888", fontSize: 12, cursor: "pointer", borderRadius: 6,
        }}
      >+ Nouveau domaine</button>
    </div>
  );
}
