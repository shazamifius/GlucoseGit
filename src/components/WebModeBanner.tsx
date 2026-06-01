import { useState } from "react";
import { isWeb } from "../utils/platform";

/**
 * Bandeau discret affiché UNIQUEMENT en version web/PWA (pas dans l'app desktop
 * Tauri). Il prévient honnêtement que c'est expérimental et que l'accès aux
 * fichiers PC n'existe pas côté web. Masquable.
 */
export function WebModeBanner() {
  const [dismissed, setDismissed] = useState(false);
  if (!isWeb() || dismissed) return null;
  return (
    <div
      style={{
        position: "fixed", bottom: 10, left: "50%", transform: "translateX(-50%)",
        zIndex: 1000, display: "flex", alignItems: "center", gap: 10,
        background: "#1a1a2e", border: "1px solid #33335a", borderRadius: 8,
        padding: "7px 12px", fontSize: 12, color: "#b9b9d6",
        boxShadow: "0 4px 16px rgba(0,0,0,0.5)", fontFamily: "system-ui, sans-serif",
        maxWidth: "92vw",
      }}
    >
      <span>
        🧪 <strong>Version web expérimentale.</strong> Le canvas fonctionne ;
        l'accès à tes fichiers PC n'est pas disponible ici (pour ça, l'app desktop).
      </span>
      <button
        type="button"
        onClick={() => setDismissed(true)}
        title="Masquer"
        style={{
          background: "transparent", border: "none", color: "#7a7a9a",
          cursor: "pointer", fontSize: 14, lineHeight: 1, padding: 0,
        }}
      >
        ✕
      </button>
    </div>
  );
}
