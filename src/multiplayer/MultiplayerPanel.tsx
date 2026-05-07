// ────────────────────────────────────────────────────────────────────────────
// Phase 7.5bis — Panel multijoueur LAN
// ────────────────────────────────────────────────────────────────────────────
//
// Statut : OFF / EN ÉCOUTE / CONNECTÉ.
// Quand l'utilisateur active : on appelle `mp_start` côté Rust qui :
//   - Démarre un serveur WebSocket
//   - Annonce le service via mDNS-SD (`_glucose._tcp.local`)
//   - Browse pour découvrir les autres instances
//
// Le panel affiche :
//   - Toggle ON/OFF
//   - Nom de l'instance + port
//   - Liste des peers découverts (cliquable pour s'y connecter)
//   - Liste des connexions actives
//
// Sync auto : useMultiplayerSync(true) gère la diffusion + réception des patches.

import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

interface Props {
  enabled: boolean;
  onToggle: (next: boolean) => void;
  onClose: () => void;
}

interface PeerFound { name: string; addr: string; port: number; }
type Status =
  | { kind: "off" }
  | { kind: "starting" }
  | { kind: "listening"; port: number; name: string }
  | { kind: "error"; message: string };

const DEFAULT_PORT = 7777;

export default function MultiplayerPanel({ enabled, onToggle, onClose }: Props) {
  const [status, setStatus] = useState<Status>({ kind: "off" });
  const [peers, setPeers] = useState<PeerFound[]>([]);
  const [connected, setConnected] = useState<Array<{ key: string; name: string }>>([]);
  const [manualAddr, setManualAddr] = useState("");
  const subsRef = useRef<UnlistenFn[]>([]);

  // Bind aux events Tauri tant que le panel est ouvert
  useEffect(() => {
    let alive = true;
    const setup = async () => {
      const a = await listen<Status>("mp:status", (e) => {
        const payload = e.payload as unknown as Record<string, unknown>;
        // Backend envoie `{ "type": "listening", port, name }` etc.
        const t = (payload.type as string) ?? "off";
        if (t === "listening") {
          setStatus({ kind: "listening", port: payload.port as number, name: payload.name as string });
        } else if (t === "starting") {
          setStatus({ kind: "starting" });
        } else if (t === "error") {
          setStatus({ kind: "error", message: (payload.message as string) ?? "?" });
        } else {
          setStatus({ kind: "off" });
        }
      });
      const b = await listen<PeerFound>("mp:peer-found", (e) => {
        setPeers((prev) => {
          const exists = prev.some((p) => p.name === e.payload.name);
          return exists ? prev : [...prev, e.payload];
        });
      });
      const c = await listen<{ name: string }>("mp:peer-lost", (e) => {
        setPeers((prev) => prev.filter((p) => p.name !== e.payload.name));
      });
      const d = await listen<{ key: string; name: string }>("mp:peer-connected", (e) => {
        setConnected((prev) => {
          const exists = prev.some((p) => p.key === e.payload.key);
          return exists ? prev : [...prev, e.payload];
        });
      });
      const f = await listen<{ key: string }>("mp:peer-disconnected", (e) => {
        setConnected((prev) => prev.filter((p) => p.key !== e.payload.key));
      });
      if (alive) subsRef.current = [a, b, c, d, f];
    };
    setup();
    return () => {
      alive = false;
      for (const u of subsRef.current) u();
      subsRef.current = [];
    };
  }, []);

  async function handleToggle() {
    if (enabled) {
      await invoke("mp_stop").catch(console.warn);
      setStatus({ kind: "off" });
      setPeers([]);
      setConnected([]);
      onToggle(false);
    } else {
      try {
        await invoke<Status>("mp_start", { port: DEFAULT_PORT });
        onToggle(true);
      } catch (e) {
        setStatus({ kind: "error", message: String(e) });
      }
    }
  }

  async function connectPeer(p: PeerFound) {
    try {
      await invoke("mp_connect", { addr: p.addr, port: p.port, name: p.name });
    } catch (e) {
      console.error("[mp] connect failed:", e);
    }
  }

  async function connectManual() {
    if (!manualAddr.trim()) return;
    const m = manualAddr.match(/^([\w.-]+)(?::(\d+))?$/);
    if (!m) return;
    const addr = m[1];
    const port = m[2] ? Number(m[2]) : DEFAULT_PORT;
    await invoke("mp_connect", { addr, port, name: `${addr}:${port}` }).catch(console.error);
    setManualAddr("");
  }

  const statusColor =
    status.kind === "listening" ? "#10b981" :
    status.kind === "starting" ? "#fbbf24" :
    status.kind === "error" ? "#ef4444" : "#6b7280";

  const statusLabel =
    status.kind === "listening" ? `EN ÉCOUTE · port ${status.port}` :
    status.kind === "starting" ? "DÉMARRAGE…" :
    status.kind === "error" ? `ERREUR : ${status.message}` :
    "DÉSACTIVÉ";

  return (
    <div
      style={{
        position: "absolute", top: 60, right: 16, width: 360,
        background: "rgba(20,20,24,0.96)",
        border: "1px solid #2a2a2a", borderRadius: 8,
        boxShadow: "0 12px 32px rgba(0,0,0,0.55)",
        backdropFilter: "blur(8px)",
        WebkitBackdropFilter: "blur(8px)",
        zIndex: 60,
        fontFamily: "system-ui, sans-serif",
        color: "#cbd5e1",
        padding: 14,
        userSelect: "none",
      }}
    >
      {/* Header */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 10 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <span
            style={{
              width: 8, height: 8, borderRadius: "50%",
              background: statusColor,
              boxShadow: `0 0 8px ${statusColor}`,
            }}
          />
          <span style={{ fontSize: 11, letterSpacing: 0.6, color: "#9ca3af" }}>
            🛰️ MULTIJOUEUR LAN · {statusLabel}
          </span>
        </div>
        <button onClick={onClose} title="Fermer" style={btnIcon()}>⛌</button>
      </div>

      {/* Toggle */}
      <button
        onClick={handleToggle}
        style={{
          width: "100%",
          background: enabled ? "#10b981" : "#374151",
          color: enabled ? "#0d0d0d" : "#cbd5e1",
          border: "none", borderRadius: 6,
          padding: "8px 12px",
          fontWeight: 600, fontSize: 13,
          cursor: "pointer",
          marginBottom: 12,
        }}
      >
        {enabled ? "⏹ Arrêter" : "▶ Activer le multijoueur"}
      </button>

      {/* Nom local */}
      {status.kind === "listening" && (
        <div style={{ fontSize: 11, color: "#666", marginBottom: 10 }}>
          Visible aux autres comme : <span style={{ color: "#cbd5e1" }}>{status.name}</span>
        </div>
      )}

      {/* Liste peers découverts */}
      {enabled && (
        <>
          <div style={{ fontSize: 10, letterSpacing: 0.5, color: "#6b7280", marginBottom: 4 }}>
            INSTANCES SUR LE LAN
          </div>
          {peers.length === 0 ? (
            <div style={{ fontSize: 11, color: "#4b5563", padding: "8px 0" }}>
              Aucune autre instance détectée. Démarre Glucose sur un autre PC du même réseau.
            </div>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: 4, marginBottom: 10 }}>
              {peers.map((p) => {
                const isConnected = connected.some((c) => c.name === p.name);
                return (
                  <button
                    key={p.name}
                    onClick={() => connectPeer(p)}
                    disabled={isConnected}
                    style={{
                      display: "flex", justifyContent: "space-between", alignItems: "center",
                      background: isConnected ? "rgba(16,185,129,0.15)" : "#262626",
                      border: `1px solid ${isConnected ? "#10b98155" : "#333"}`,
                      borderRadius: 4,
                      padding: "6px 10px",
                      color: "#cbd5e1",
                      cursor: isConnected ? "default" : "pointer",
                      fontSize: 12,
                      textAlign: "left",
                    }}
                  >
                    <span>
                      {isConnected ? "🟢 " : "📡 "}
                      {p.name}
                      <span style={{ color: "#6b7280", fontSize: 10, marginLeft: 6 }}>
                        {p.addr}:{p.port}
                      </span>
                    </span>
                    {!isConnected && <span style={{ color: "#9ca3af", fontSize: 10 }}>connecter →</span>}
                  </button>
                );
              })}
            </div>
          )}

          {/* Connexion manuelle */}
          <div style={{ fontSize: 10, letterSpacing: 0.5, color: "#6b7280", marginTop: 8, marginBottom: 4 }}>
            CONNEXION MANUELLE (IP:PORT)
          </div>
          <div style={{ display: "flex", gap: 6 }}>
            <input
              value={manualAddr}
              onChange={(e) => setManualAddr(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") connectManual(); }}
              placeholder={`192.168.1.42:${DEFAULT_PORT}`}
              style={{
                flex: 1,
                background: "#0d0d0d", color: "#f3f4f6",
                border: "1px solid #444", borderRadius: 4,
                padding: "5px 8px", fontSize: 11,
                outline: "none", fontFamily: "monospace",
              }}
            />
            <button onClick={connectManual} style={btnSecondary()}>OK</button>
          </div>
        </>
      )}

      <div style={{ marginTop: 12, fontSize: 10, color: "#4b5563", lineHeight: 1.4 }}>
        ⚠ MVP : sync des données uniquement (pas de curseurs flottants). LAN privé non chiffré.
      </div>
    </div>
  );
}

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
