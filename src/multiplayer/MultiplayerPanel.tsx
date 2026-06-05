// ────────────────────────────────────────────────────────────────────────────
// Panel Collaboration (internet, via serveur de synchro Automerge)
// ────────────────────────────────────────────────────────────────────────────
//
// Remplace l'ancien panel LAN (mDNS). La collaboration passe désormais par
// automerge-repo + un serveur de synchro always-on (cf. src/multiplayer/repo.ts)
// qui stocke le document → un pair peut fermer son PC, l'autre garde tout, et le
// catch-up est automatique à la (re)connexion.
//
// Deux usages :
//   • Héberger  → on partage le projet courant, un CODE `automerge:…` est généré.
//   • Rejoindre → on colle le code d'un pair pour adopter son projet.

import { useEffect, useRef, useState } from "react";
import {
  createShare, resumeShare, joinByCode, leaveCollab, getSavedShareUrl,
} from "./collabBridge";
import { getActiveShareUrl } from "./collabHandle";
import { isServerConnected, onConnectivityChange } from "./repo";

interface Props {
  enabled: boolean;
  onToggle: (next: boolean) => void;
  onClose: () => void;
}

export default function MultiplayerPanel({ enabled, onToggle, onClose }: Props) {
  const [shareUrl, setShareUrl] = useState<string | null>(getActiveShareUrl());
  const [joinInput, setJoinInput] = useState("");
  const [connected, setConnected] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const savedUrl = useRef<string | null>(getSavedShareUrl());

  // Indicateur de connexion au serveur — actif uniquement quand la collab tourne.
  useEffect(() => {
    if (!enabled) { setConnected(false); return; }
    setConnected(isServerConnected());
    const off = onConnectivityChange(() => setConnected(isServerConnected()));
    return off;
  }, [enabled]);

  async function withBusy(fn: () => Promise<void> | void) {
    setError(null);
    setBusy(true);
    try {
      await fn();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  const handleCreate = () => withBusy(async () => {
    const url = await createShare();
    savedUrl.current = url;
    setShareUrl(url);
    onToggle(true);
  });

  const handleResume = () => withBusy(async () => {
    const url = await resumeShare();
    savedUrl.current = url;
    setShareUrl(url);
    onToggle(true);
  });

  const handleJoin = () => withBusy(async () => {
    if (!joinInput.trim()) return;
    const url = await joinByCode(joinInput);
    savedUrl.current = url;
    setShareUrl(url);
    setJoinInput("");
    onToggle(true);
  });

  const handleLeave = () => {
    leaveCollab();
    setShareUrl(null);
    onToggle(false);
  };

  const copyCode = async () => {
    if (!shareUrl) return;
    try {
      await navigator.clipboard.writeText(shareUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch { /* clipboard indispo : l'utilisateur copie à la main */ }
  };

  const dotColor = !enabled ? "#6b7280" : connected ? "#10b981" : "#fbbf24";
  const statusLabel = !enabled
    ? "DÉSACTIVÉ"
    : connected ? "CONNECTÉ AU SERVEUR" : "CONNEXION…";

  return (
    <div
      style={{
        position: "absolute", top: 60, right: 16, width: 360,
        background: "rgba(20,20,24,0.96)",
        border: "1px solid #2a2a2a", borderRadius: 8,
        boxShadow: "0 12px 32px rgba(0,0,0,0.55)",
        backdropFilter: "blur(8px)", WebkitBackdropFilter: "blur(8px)",
        zIndex: 60, fontFamily: "system-ui, sans-serif",
        color: "#cbd5e1", padding: 14, userSelect: "none",
      }}
    >
      {/* Header */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 10 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <span style={{ width: 8, height: 8, borderRadius: "50%", background: dotColor, boxShadow: `0 0 8px ${dotColor}` }} />
          <span style={{ fontSize: 11, letterSpacing: 0.6, color: "#9ca3af" }}>
            🌐 COLLABORATION · {statusLabel}
          </span>
        </div>
        <button onClick={onClose} title="Fermer" style={btnIcon()}>⛌</button>
      </div>

      {!enabled ? (
        <>
          {/* Créer une chaîne */}
          <div style={sectionTitle()}>CRÉER UNE CHAÎNE (à partir de ce projet)</div>
          <button onClick={handleCreate} disabled={busy} style={btnPrimary(busy)}>
            ▶ Créer une chaîne
          </button>
          {savedUrl.current && (
            <button onClick={handleResume} disabled={busy} style={btnSecondaryWide()}>
              ↻ Rouvrir ma chaîne
            </button>
          )}

          {/* Rejoindre une chaîne */}
          <div style={{ ...sectionTitle(), marginTop: 14 }}>REJOINDRE UNE CHAÎNE</div>
          <div style={{ display: "flex", gap: 6 }}>
            <input
              value={joinInput}
              onChange={(e) => setJoinInput(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") handleJoin(); }}
              placeholder="colle le code automerge:… ici"
              style={inputStyle()}
            />
            <button onClick={handleJoin} disabled={busy || !joinInput.trim()} style={btnSecondary()}>
              Rejoindre
            </button>
          </div>
          <div style={{ marginTop: 6, fontSize: 10, color: "#9ca3af", lineHeight: 1.4 }}>
            ⚠ Rejoindre une chaîne ouvre SON projet (ton projet local courant est mis de côté).
          </div>
        </>
      ) : (
        <>
          {/* Code de la chaîne active */}
          <div style={sectionTitle()}>CODE DE LA CHAÎNE (à envoyer à ton pote)</div>
          <div style={{ display: "flex", gap: 6 }}>
            <input readOnly value={shareUrl ?? ""} style={{ ...inputStyle(), color: "#10b981" }} onFocus={(e) => e.currentTarget.select()} />
            <button onClick={copyCode} style={btnSecondary()}>{copied ? "✓ Copié" : "Copier"}</button>
          </div>
          <div style={{ marginTop: 10, fontSize: 11, color: connected ? "#10b981" : "#fbbf24" }}>
            {connected
              ? "Chaîne active. Vous éditez à deux, à égalité : chacun peut fermer et rouvrir quand il veut, le serveur garde tout à jour."
              : "Connexion au serveur de la chaîne en cours…"}
          </div>
          <button
            onClick={handleLeave}
            style={btnLeave()}
            onMouseOver={(e) => { e.currentTarget.style.background = "rgba(239,68,68,0.16)"; e.currentTarget.style.borderColor = "rgba(239,68,68,0.5)"; }}
            onMouseOut={(e) => { e.currentTarget.style.background = "rgba(239,68,68,0.08)"; e.currentTarget.style.borderColor = "rgba(239,68,68,0.28)"; }}
          >
            Quitter la chaîne
          </button>
        </>
      )}

      {error && (
        <div style={{ marginTop: 10, fontSize: 11, color: "#ef4444" }}>Erreur : {error}</div>
      )}

      <div style={{ marginTop: 12, fontSize: 10, color: "#4b5563", lineHeight: 1.4 }}>
        Synchro via serveur public Automerge. Pour un serveur privé, modifie l'URL dans repo.ts.
      </div>
    </div>
  );
}

function sectionTitle(): React.CSSProperties {
  return { fontSize: 10, letterSpacing: 0.5, color: "#6b7280", marginBottom: 6 };
}
function inputStyle(): React.CSSProperties {
  return {
    flex: 1, background: "#0d0d0d", color: "#f3f4f6",
    border: "1px solid #444", borderRadius: 4,
    padding: "6px 8px", fontSize: 11, outline: "none", fontFamily: "monospace",
  };
}
function btnPrimary(busy: boolean): React.CSSProperties {
  return {
    width: "100%", background: busy ? "#374151" : "#10b981",
    color: busy ? "#9ca3af" : "#0d0d0d", border: "none", borderRadius: 6,
    padding: "8px 12px", fontWeight: 600, fontSize: 13,
    cursor: busy ? "default" : "pointer", marginBottom: 6,
  };
}
function btnSecondaryWide(): React.CSSProperties {
  return {
    width: "100%", background: "transparent", border: "1px solid #444",
    color: "#cbd5e1", borderRadius: 6, padding: "7px 12px",
    fontSize: 12, cursor: "pointer",
  };
}
function btnLeave(): React.CSSProperties {
  return {
    width: "100%", marginTop: 12,
    background: "rgba(239,68,68,0.08)",
    border: "1px solid rgba(239,68,68,0.28)",
    color: "#f87171", borderRadius: 6, padding: "8px 12px",
    fontSize: 12, fontWeight: 600, cursor: "pointer",
    transition: "background 120ms, border-color 120ms",
  };
}
function btnIcon(): React.CSSProperties {
  return {
    background: "transparent", border: "1px solid #444", color: "#aaa",
    borderRadius: 4, fontSize: 11, padding: "2px 8px", cursor: "pointer",
    fontFamily: "system-ui, sans-serif",
  };
}
function btnSecondary(): React.CSSProperties {
  return {
    background: "transparent", border: "1px solid #444", color: "#cbd5e1",
    borderRadius: 4, fontSize: 11, padding: "5px 12px", cursor: "pointer",
    fontFamily: "system-ui, sans-serif",
  };
}
