// ────────────────────────────────────────────────────────────────────────────
// Phase 7.4 — Time Machine UI
// ────────────────────────────────────────────────────────────────────────────
//
// Slider d'historique en bas du canvas. Le user peut :
//   • Drag du slider → preview live d'un état passé (PixiJS redraw automatique
//     car `project` est dérivé du doc preview)
//   • Cliquer sur un jalon nommé pour aller à cet état exact
//   • « Restaurer cet état » → applique l'état preview comme nouveau commit
//     (l'historique antérieur est conservé)
//   • « + Marquer un jalon » → commit nommé (apparaît dans la timeline)
//   • « Maintenant » → sortie du mode preview, retour au présent
//
// Mode interaction :
//   - Tant que `_previewHeads !== null`, toutes les mutations du store sont
//     bloquées (cf. store mutate). Un overlay visuel signale ce mode.

import { useEffect, useMemo, useRef, useState } from "react";
import { useGlucoseStore } from "../store";
import * as A from "../store/automerge";

interface Props {
  onClose: () => void;
}

interface CommitEntry {
  index: number;
  message: string;
  time: number;            // ms unix
  heads: A.Heads;
}

const PANEL_HEIGHT = 96;

export default function TimelinePanel({ onClose }: Props) {
  const _doc = useGlucoseStore((s) => s._doc);
  const _previewHeads = useGlucoseStore((s) => s._previewHeads);
  const setPreviewHeads = useGlucoseStore((s) => s.setPreviewHeads);
  const restoreToPreview = useGlucoseStore((s) => s.restoreToPreview);
  const commitNamed = useGlucoseStore((s) => s.commitNamed);

  const trackRef = useRef<HTMLDivElement>(null);
  const [draggingIdx, setDraggingIdx] = useState<number | null>(null);
  const [namedDialog, setNamedDialog] = useState(false);
  const [jalonName, setJalonName] = useState("");

  // ── Reconstruit la liste des commits depuis l'historique Automerge ────
  const commits = useMemo<CommitEntry[]>(() => {
    try {
      // Phase 7.4 : on prend chaque change Automerge comme un point de timeline.
      // Le `message` passé à `mutate(message, ...)` est conservé ici.
      const history = A.history(_doc);
      const out: CommitEntry[] = [];
      // Pour récupérer les heads à un point N de l'historique, on doit refaire
      // un viewAt sur les changes.hash de chaque commit.
      for (let i = 0; i < history.length; i++) {
        const entry = history[i];
        out.push({
          index: i,
          message: entry.change.message || "(sans message)",
          time: entry.change.time * 1000,
          heads: [entry.change.hash],
        });
      }
      return out;
    } catch (e) {
      console.error("[TimelinePanel] history error:", e);
      return [];
    }
  }, [_doc]);

  // Index du commit actuellement sélectionné (preview ou présent)
  const currentIdx = useMemo(() => {
    if (!_previewHeads) return commits.length - 1;
    // Match par hash : le head preview correspond à un seul commit
    const target = _previewHeads[0];
    const idx = commits.findIndex((c) => c.heads[0] === target);
    return idx === -1 ? commits.length - 1 : idx;
  }, [_previewHeads, commits]);

  // ── Drag du slider ────────────────────────────────────────────────────
  function pickAtX(clientX: number) {
    if (!trackRef.current || commits.length === 0) return;
    const rect = trackRef.current.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    const idx = Math.round(ratio * (commits.length - 1));
    setDraggingIdx(idx);
    if (idx === commits.length - 1) {
      // Position « maintenant » → sortir du preview
      setPreviewHeads(null);
    } else {
      setPreviewHeads(commits[idx].heads);
    }
  }

  function startDrag(e: React.PointerEvent) {
    e.preventDefault();
    pickAtX(e.clientX);
    const onMove = (m: PointerEvent) => pickAtX(m.clientX);
    const onUp = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      setDraggingIdx(null);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  }

  // ── Échap = retour au présent + fermeture ─────────────────────────────
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (_previewHeads) setPreviewHeads(null);
        else onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [_previewHeads, onClose, setPreviewHeads]);

  // ── Affiche un jalon (commit nommé : message commence par 📌) ─────────
  const namedJalons = commits.filter((c) => c.message.startsWith("📌"));

  // Position du curseur (en %) sur la piste
  const cursorPct = commits.length <= 1
    ? 100
    : (currentIdx / (commits.length - 1)) * 100;

  const inPreview = _previewHeads !== null;
  const currentCommit = commits[currentIdx];

  return (
    <>
      {/* Overlay visuel quand on est en preview historique */}
      {inPreview && (
        <div
          style={{
            position: "fixed", inset: 0,
            pointerEvents: "none",
            border: "3px solid #fbbf24",
            boxShadow: "inset 0 0 60px rgba(251, 191, 36, 0.18)",
            zIndex: 41,
            transition: "opacity 200ms",
          }}
        />
      )}

      {/* Bandeau Time Machine */}
      <div
        style={{
          position: "absolute", left: 16, right: 16, bottom: 16,
          height: PANEL_HEIGHT,
          background: "linear-gradient(180deg, rgba(20,20,24,0.94), rgba(15,15,18,0.94))",
          border: `1px solid ${inPreview ? "#fbbf2466" : "#2a2a2a"}`,
          borderRadius: 8,
          boxShadow: "0 6px 24px rgba(0,0,0,0.5)",
          backdropFilter: "blur(6px)",
          WebkitBackdropFilter: "blur(6px)",
          userSelect: "none",
          zIndex: 50,
          display: "flex", flexDirection: "column",
          padding: "8px 12px",
        }}
      >
        {/* Header */}
        <div style={{
          display: "flex", justifyContent: "space-between", alignItems: "center",
          color: "#9ca3af", fontSize: 10, letterSpacing: 0.4, marginBottom: 4,
        }}>
          <span>
            ⏳ TIME MACHINE
            {inPreview && <span style={{ color: "#fbbf24", marginLeft: 8 }}>· APERÇU HISTORIQUE</span>}
            {!inPreview && <span style={{ color: "#10b981", marginLeft: 8 }}>· EN DIRECT</span>}
          </span>
          <span style={{ display: "flex", gap: 8, alignItems: "center" }}>
            {currentCommit && (
              <span style={{ color: "#cbd5e1", fontSize: 10 }}>
                {currentIdx + 1}/{commits.length} · {currentCommit.message.slice(0, 40)}
                {currentCommit.message.length > 40 ? "…" : ""}
                {" · "}{formatTimeAgo(currentCommit.time)}
              </span>
            )}
            <button
              onClick={onClose}
              title="Fermer (Échap)"
              style={btnIcon()}
            >⛌</button>
          </span>
        </div>

        {/* Piste avec graduations + jalons nommés + curseur */}
        <div
          ref={trackRef}
          onPointerDown={startDrag}
          style={{
            position: "relative",
            height: 28, marginTop: 2,
            background: "linear-gradient(180deg, rgba(255,255,255,0.025), rgba(255,255,255,0.01))",
            borderRadius: 4, cursor: "pointer",
            border: "1px solid #1f1f23",
          }}
        >
          {/* Graduations : un tick par commit, ticks plus marqués pour les jalons nommés */}
          {commits.map((c, i) => {
            const pct = commits.length <= 1 ? 0 : (i / (commits.length - 1)) * 100;
            const isJalon = c.message.startsWith("📌");
            return (
              <div
                key={i}
                style={{
                  position: "absolute",
                  left: `${pct}%`, top: 4, bottom: 4,
                  width: isJalon ? 2 : 1,
                  marginLeft: isJalon ? -1 : 0,
                  background: isJalon ? "#fbbf24" : "#3a3a3a",
                  pointerEvents: "none",
                }}
              />
            );
          })}
          {/* Curseur courant */}
          <div
            style={{
              position: "absolute",
              left: `${cursorPct}%`,
              top: -4, bottom: -4,
              width: 3, marginLeft: -1.5,
              background: inPreview ? "#fbbf24" : "#10b981",
              borderRadius: 2,
              boxShadow: `0 0 8px ${inPreview ? "rgba(251,191,36,0.6)" : "rgba(16,185,129,0.6)"}`,
              pointerEvents: "none",
              transition: draggingIdx === null ? "left 100ms ease-out" : "none",
            }}
          />
        </div>

        {/* Jalons nommés (sous la piste, cliquables) */}
        {namedJalons.length > 0 && (
          <div style={{ display: "flex", flexWrap: "wrap", gap: 4, marginTop: 4, maxHeight: 22, overflow: "hidden" }}>
            {namedJalons.slice(-8).map((j) => (
              <button
                key={j.index}
                onClick={() => setPreviewHeads(j.heads)}
                title={`${j.message} · ${formatTimeAgo(j.time)}`}
                style={{
                  background: "rgba(251,191,36,0.12)",
                  color: "#fde68a",
                  border: "1px solid rgba(251,191,36,0.3)",
                  borderRadius: 10,
                  padding: "1px 8px",
                  fontSize: 10,
                  cursor: "pointer",
                  whiteSpace: "nowrap",
                  fontFamily: "system-ui, sans-serif",
                }}
              >
                📌 {j.message.replace(/^📌\s*/, "").slice(0, 24)}
              </button>
            ))}
          </div>
        )}

        {/* Boutons d'action */}
        <div style={{ display: "flex", gap: 6, marginTop: "auto", paddingTop: 4 }}>
          {inPreview ? (
            <>
              <button onClick={() => setPreviewHeads(null)} style={btnSecondary()}>← Maintenant</button>
              <button onClick={restoreToPreview} style={btnPrimary()}>
                ⏪ Restaurer cet état
              </button>
            </>
          ) : (
            <button onClick={() => setNamedDialog(true)} style={btnSecondary()}>
              + Marquer un jalon
            </button>
          )}
        </div>
      </div>

      {/* Dialog jalon nommé */}
      {namedDialog && (
        <div
          onClick={() => setNamedDialog(false)}
          style={{
            position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)",
            display: "flex", alignItems: "center", justifyContent: "center", zIndex: 200,
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              background: "#1a1a1a", border: "1px solid #333", borderRadius: 8,
              padding: 18, width: 380, fontFamily: "system-ui, sans-serif",
              boxShadow: "0 24px 48px rgba(0,0,0,0.6)",
            }}
          >
            <div style={{ color: "#fde68a", fontSize: 11, letterSpacing: 0.5, marginBottom: 6 }}>
              📌 NOUVEAU JALON
            </div>
            <input
              autoFocus
              value={jalonName}
              onChange={(e) => setJalonName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  commitNamed(jalonName);
                  setJalonName("");
                  setNamedDialog(false);
                }
                if (e.key === "Escape") setNamedDialog(false);
              }}
              placeholder="ex: « Première version du concept », « avant refonte »…"
              style={{
                width: "100%", padding: "10px 12px",
                background: "#0d0d0d", color: "#f3f4f6",
                border: "1px solid #444", borderRadius: 6,
                fontSize: 13, outline: "none", boxSizing: "border-box",
              }}
            />
            <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 12 }}>
              <button onClick={() => setNamedDialog(false)} style={btnSecondary()}>Annuler</button>
              <button
                onClick={() => {
                  commitNamed(jalonName);
                  setJalonName("");
                  setNamedDialog(false);
                }}
                style={btnPrimary()}
              >
                Marquer
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

// ── Helpers de style ─────────────────────────────────────────────────────────
function btnIcon(): React.CSSProperties {
  return {
    background: "transparent",
    border: "1px solid #444",
    color: "#aaa",
    borderRadius: 4,
    fontSize: 11,
    padding: "2px 8px",
    cursor: "pointer",
    fontFamily: "system-ui, sans-serif",
  };
}
function btnSecondary(): React.CSSProperties {
  return {
    background: "transparent",
    border: "1px solid #444",
    color: "#cbd5e1",
    borderRadius: 4,
    fontSize: 11,
    padding: "5px 12px",
    cursor: "pointer",
    fontFamily: "system-ui, sans-serif",
  };
}
function btnPrimary(): React.CSSProperties {
  return {
    background: "#fbbf24",
    color: "#0d0d0d",
    border: "none",
    borderRadius: 4,
    fontSize: 11,
    padding: "5px 12px",
    cursor: "pointer",
    fontWeight: 600,
    fontFamily: "system-ui, sans-serif",
  };
}

function formatTimeAgo(ts: number): string {
  const sec = Math.max(0, (Date.now() - ts) / 1000);
  if (sec < 5) return "à l'instant";
  if (sec < 60) return `il y a ${Math.round(sec)} s`;
  const min = sec / 60;
  if (min < 60) return `il y a ${Math.round(min)} min`;
  const h = min / 60;
  if (h < 24) return `il y a ${Math.round(h)} h`;
  const d = h / 24;
  return `il y a ${Math.round(d)} j`;
}
