// ────────────────────────────────────────────────────────────────────────────
// Auto-update — popup au démarrage + PASTILLE d'état visible (débogage updater)
// ────────────────────────────────────────────────────────────────────────────
//
// Au lancement : affiche la VERSION installée + interroge GitHub. Si une version
// signée plus récente existe → popup « Installer et redémarrer ». La pastille
// d'état (version + résultat du check) est TEMPORAIRE : elle rend l'updater
// observable pour comprendre pourquoi le popup n'apparaît pas (mauvaise version
// installée ? erreur réseau/signature ?). À simplifier une fois validé.

import { useEffect, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";

type Status =
  | { kind: "checking" }
  | { kind: "uptodate" }
  | { kind: "available"; version: string }
  | { kind: "error"; message: string };

export default function UpdatePrompt() {
  const [version, setVersion] = useState<string>("?");
  const [status, setStatus] = useState<Status>({ kind: "checking" });
  const [update, setUpdate] = useState<Update | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [pct, setPct] = useState<number | null>(null);

  useEffect(() => {
    let cancelled = false;
    getVersion().then((v) => { if (!cancelled) setVersion(v); }).catch(() => {});
    (async () => {
      try {
        const u = await check();
        if (cancelled) return;
        if (u) {
          setUpdate(u);
          setStatus({ kind: "available", version: u.version });
        } else {
          setStatus({ kind: "uptodate" });
        }
      } catch (e) {
        if (!cancelled) setStatus({ kind: "error", message: String(e) });
        console.warn("[updater] check échec :", String(e));
      }
    })();
    return () => { cancelled = true; };
  }, []);

  async function install() {
    if (!update) return;
    setBusy(true);
    let total = 0;
    let got = 0;
    try {
      await update.downloadAndInstall((ev) => {
        if (ev.event === "Started") { total = ev.data.contentLength ?? 0; setPct(0); }
        else if (ev.event === "Progress") { got += ev.data.chunkLength; if (total > 0) setPct(Math.min(100, Math.round((got / total) * 100))); }
        else if (ev.event === "Finished") { setPct(100); }
      });
      await relaunch();
    } catch (e) {
      setStatus({ kind: "error", message: String(e) });
      setBusy(false);
    }
  }

  // ── Pastille d'état (toujours visible, temporaire) ──────────────────────────
  const statusText =
    status.kind === "checking" ? "vérification…"
    : status.kind === "uptodate" ? "à jour ✓"
    : status.kind === "available" ? `MAJ trouvée : v${status.version}`
    : `erreur : ${status.message.slice(0, 80)}`;
  const statusColor =
    status.kind === "error" ? "#f87171"
    : status.kind === "available" ? "#4ade80"
    : "#7d7d8c";

  return (
    <>
      <div
        style={{
          position: "absolute", top: 60, right: 12, zIndex: 2500,
          background: "#0d0d0dee", border: "1px solid #26262e", borderRadius: 6,
          padding: "5px 9px", maxWidth: 320,
          font: "11px/1.4 ui-monospace, SFMono-Regular, Menlo, monospace",
          color: "#9a9aa0", pointerEvents: "none", userSelect: "none",
        }}
      >
        <span style={{ color: "#d4d4dd" }}>Glucose v{version}</span>
        {"  ·  updater: "}
        <span style={{ color: statusColor }}>{statusText}</span>
      </div>

      {update && !dismissed && (
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
            <span style={{ display: "inline-block", width: 8, height: 8, borderRadius: 8, background: "#4ade80" }} />
            <strong style={{ color: "#fff", fontWeight: 600 }}>Mise à jour disponible</strong>
            <span style={{ marginLeft: "auto", color: "#7d7d8c", fontSize: 12 }}>v{update.version}</span>
          </div>

          {busy ? (
            <div>
              <div style={{ color: "#9a9aa0", fontSize: 12, marginBottom: 6 }}>
                {pct === null ? "Préparation…" : pct < 100 ? `Téléchargement… ${pct}%` : "Installation…"}
              </div>
              <div style={{ height: 6, background: "#23232b", borderRadius: 6, overflow: "hidden" }}>
                <div style={{ height: "100%", width: `${pct ?? 0}%`, background: "#4ade80", transition: "width 0.15s linear" }} />
              </div>
            </div>
          ) : (
            <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
              <button
                type="button"
                onClick={() => setDismissed(true)}
                style={{ background: "transparent", color: "#9a9aa0", border: "1px solid #34343e", borderRadius: 6, padding: "6px 12px", cursor: "pointer", fontSize: 12 }}
              >
                Plus tard
              </button>
              <button
                type="button"
                onClick={install}
                style={{ background: "#4ade80", color: "#0d0d0d", border: "none", borderRadius: 6, padding: "6px 14px", cursor: "pointer", fontSize: 12, fontWeight: 600 }}
              >
                Installer et redémarrer
              </button>
            </div>
          )}
        </div>
      )}
    </>
  );
}
