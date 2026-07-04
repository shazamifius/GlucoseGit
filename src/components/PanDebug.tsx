// ────────────────────────────────────────────────────────────────────────────
// Diagnostic PAN — TEMPORAIRE (débogage du déplacement cassé sous Linux/WebKitGTK)
// ────────────────────────────────────────────────────────────────────────────
//
// Visible UNIQUEMENT sous Linux. Montre la vérité terrain quand un utilisateur
// signale « impossible de se déplacer » :
//   • `movementX/Y` = ce que WebKitGTK renvoie (souvent 0/faux = LE bug) ;
//   • `clientΔ` = le delta qu'on utilise désormais (doit bouger quand on pane).
// Pane et screenshotte cette boîte : si movementX/Y restent à 0 mais clientΔ bouge,
// le fix est bon. À retirer une fois le pan Linux validé.

import { useEffect, useState } from "react";
import { getPanDebug } from "../utils/cursorWrap";

export default function PanDebug() {
  const [, setTick] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setTick((t) => t + 1), 350);
    return () => clearInterval(id);
  }, []);

  const d = getPanDebug();
  if (!d.linux) return null; // seulement sous Linux

  return (
    <div
      style={{
        position: "absolute", bottom: 12, left: 12, zIndex: 2000,
        background: "#0d0d0dee", border: "1px solid #26262e", borderRadius: 6,
        padding: "6px 9px", minWidth: 170,
        font: "11px/1.5 ui-monospace, SFMono-Regular, Menlo, monospace",
        color: "#9a9aa0", pointerEvents: "none", userSelect: "none",
      }}
    >
      <div style={{ color: "#d4d4dd", letterSpacing: "0.04em" }}>
        PAN · {d.mode}{d.wayland ? " · wayland" : ""}
      </div>
      <div>movementX/Y : <span style={{ color: "#d4d4dd" }}>{d.mvX}, {d.mvY}</span></div>
      <div>clientΔ : <span style={{ color: "#d4d4dd" }}>{d.cdx}, {d.cdy}</span></div>
    </div>
  );
}
