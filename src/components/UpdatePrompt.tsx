// ────────────────────────────────────────────────────────────────────────────
// Auto-update — popup au démarrage « Mise à jour dispo — Installer ? »
// ────────────────────────────────────────────────────────────────────────────
//
// Au lancement, on demande à GitHub s'il existe une version plus récente
// (signée). Si oui, on propose de l'installer. Téléchargement + install + relance
// automatiques. L'utilisateur garde le contrôle (peut reporter) → on ne relance
// jamais en plein travail sans son accord.
//
// Hors Tauri (web/PWA) ou si l'endpoint est injoignable : `check()` échoue et on
// n'affiche rien (silencieux).

import { useEffect, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export default function UpdatePrompt() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [pct, setPct] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const u = await check();
        if (!cancelled && u) setUpdate(u);
      } catch (e) {
        // Pas grave : web/PWA, hors-ligne, ou endpoint absent → pas de popup.
        console.warn("[updater] vérification impossible :", String(e));
      }
    })();
    return () => { cancelled = true; };
  }, []);

  if (!update || dismissed) return null;

  async function install() {
    if (!update) return;
    setBusy(true);
    setError(null);
    let total = 0;
    let got = 0;
    try {
      await update.downloadAndInstall((ev) => {
        if (ev.event === "Started") {
          total = ev.data.contentLength ?? 0;
          setPct(0);
        } else if (ev.event === "Progress") {
          got += ev.data.chunkLength;
          if (total > 0) setPct(Math.min(100, Math.round((got / total) * 100)));
        } else if (ev.event === "Finished") {
          setPct(100);
        }
      });
      // Redémarre sur la nouvelle version.
      await relaunch();
    } catch (e) {
      setError(String(e));
      setBusy(false);
    }
  }

  return (
    <div
      style={{
        position: "absolute", top: 16, left: "50%", transform: "translateX(-50%)",
        zIndex: 3000, width: 340, maxWidth: "calc(100vw - 24px)",
        background: "#16161a", border: "1px solid #34343e", borderRadius: 8,
        padding: "14px 16px", color: "#d4d4dd",
        font: "13px/1.5 system-ui, sans-serif",
        boxShadow: "0 8px 30px rgba(0,0,0,0.55)",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 6 }}>
        <span style={{
          display: "inline-block", width: 8, height: 8, borderRadius: 8, background: "#4ade80",
        }} />
        <strong style={{ color: "#fff", fontWeight: 600 }}>
          Mise à jour disponible
        </strong>
        <span style={{ marginLeft: "auto", color: "#7d7d8c", fontSize: 12 }}>
          v{update.version}
        </span>
      </div>

      {update.body && !busy && (
        <div style={{
          color: "#9a9aa0", fontSize: 12, maxHeight: 90, overflowY: "auto",
          margin: "0 0 10px", whiteSpace: "pre-wrap",
        }}>
          {update.body}
        </div>
      )}

      {busy ? (
        <div>
          <div style={{ color: "#9a9aa0", fontSize: 12, marginBottom: 6 }}>
            {pct === null ? "Préparation…" : pct < 100 ? `Téléchargement… ${pct}%` : "Installation…"}
          </div>
          <div style={{ height: 6, background: "#23232b", borderRadius: 6, overflow: "hidden" }}>
            <div style={{
              height: "100%", width: `${pct ?? 0}%`, background: "#4ade80",
              transition: "width 0.15s linear",
            }} />
          </div>
        </div>
      ) : (
        <div style={{ display: "flex", gap: 8, justifyContent: "flex-end", alignItems: "center" }}>
          {error && (
            <span style={{ color: "#f87171", fontSize: 11, marginRight: "auto" }}>
              Échec — réessaie plus tard
            </span>
          )}
          <button
            type="button"
            onClick={() => setDismissed(true)}
            style={{
              background: "transparent", color: "#9a9aa0", border: "1px solid #34343e",
              borderRadius: 6, padding: "6px 12px", cursor: "pointer", fontSize: 12,
            }}
          >
            Plus tard
          </button>
          <button
            type="button"
            onClick={install}
            style={{
              background: "#4ade80", color: "#0d0d0d", border: "none",
              borderRadius: 6, padding: "6px 14px", cursor: "pointer",
              fontSize: 12, fontWeight: 600,
            }}
          >
            Installer et redémarrer
          </button>
        </div>
      )}
    </div>
  );
}
