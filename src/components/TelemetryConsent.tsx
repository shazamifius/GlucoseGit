// ────────────────────────────────────────────────────────────────────────────
// Consentement télémétrie — popup opt-in au 1er lancement
// ────────────────────────────────────────────────────────────────────────────
//
// Apparaît UNE fois tant que l'utilisateur n'a pas répondu (état « unset »). Aucun
// dark pattern : le refus est aussi simple que l'acceptation, et rien n'est envoyé
// tant que « Activer » n'a pas été cliqué. Le choix est mémorisé (localStorage) et
// modifiable plus tard via Ctrl+Shift+D (le HUD rappelle l'état).

import { useState } from "react";
import { getConsentState, setTelemetryConsent } from "../telemetry/telemetry";

export default function TelemetryConsent() {
  const [visible, setVisible] = useState(() => getConsentState() === "unset");
  const [showDetail, setShowDetail] = useState(false);

  if (!visible) return null;

  function decide(granted: boolean) {
    setTelemetryConsent(granted);
    setVisible(false);
  }

  return (
    <div
      style={{
        position: "fixed",
        bottom: 16,
        left: "50%",
        transform: "translateX(-50%)",
        zIndex: 3000,
        width: 360,
        maxWidth: "calc(100vw - 24px)",
        background: "#16161a",
        border: "1px solid #34343e",
        borderRadius: 10,
        padding: "16px 18px",
        color: "#d4d4dd",
        font: "13px/1.5 system-ui, sans-serif",
        boxShadow: "0 10px 34px rgba(0,0,0,0.6)",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 6 }}>
        <strong style={{ color: "#fff", fontWeight: 600, fontSize: 14 }}>
          Aider à améliorer Glucose ?
        </strong>
      </div>

      <div style={{ color: "#9a9aa0", fontSize: 12.5, margin: "4px 0 10px" }}>
        Partager des statistiques <strong style={{ color: "#d4d4dd" }}>anonymes</strong> nous
        permet de voir les lags, gels et bugs sur ta machine — et de les corriger pour de
        vrai (le lag Linux, par exemple). C'est facultatif et modifiable à tout moment.
      </div>

      <button
        type="button"
        onClick={() => setShowDetail((v) => !v)}
        style={{
          background: "transparent",
          color: "#7d7d8c",
          border: "none",
          padding: 0,
          cursor: "pointer",
          fontSize: 11.5,
          textDecoration: "underline",
          marginBottom: showDetail ? 8 : 12,
        }}
      >
        {showDetail ? "Masquer le détail" : "Qu'est-ce qui est partagé ?"}
      </button>

      {showDetail && (
        <div
          style={{
            color: "#9a9aa0",
            fontSize: 11.5,
            lineHeight: 1.6,
            margin: "0 0 12px",
            background: "#101014",
            border: "1px solid #26262e",
            borderRadius: 6,
            padding: "8px 10px",
          }}
        >
          <div style={{ color: "#4ade80", marginBottom: 4 }}>✓ Partagé</div>
          FPS et gels de rendu · type de GPU/renderer · OS, RAM, nombre de cœurs ·
          résolution d'écran · erreurs de l'application · version de Glucose.
          <div style={{ color: "#f87171", margin: "8px 0 4px" }}>✗ Jamais partagé</div>
          Le contenu de tes documents, images, textes, noms de fichiers, ni aucune
          donnée personnelle (nom, e-mail…). Identifiant = un code aléatoire local.
        </div>
      )}

      <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
        <button
          type="button"
          onClick={() => decide(false)}
          style={{
            background: "transparent",
            color: "#9a9aa0",
            border: "1px solid #34343e",
            borderRadius: 6,
            padding: "7px 13px",
            cursor: "pointer",
            fontSize: 12.5,
          }}
        >
          Non merci
        </button>
        <button
          type="button"
          onClick={() => decide(true)}
          style={{
            background: "#4ade80",
            color: "#0d0d0d",
            border: "none",
            borderRadius: 6,
            padding: "7px 15px",
            cursor: "pointer",
            fontSize: 12.5,
            fontWeight: 600,
          }}
        >
          Activer le partage
        </button>
      </div>
    </div>
  );
}
