// Phase 7.5.B4 — Indicateur visuel quand on est dans un folder.
//
// Affiche un cadre subtil tout autour du viewport, de la couleur du dossier
// courant. L'opacité augmente quand l'utilisateur dézoome vers le seuil de
// sortie (EXIT_SCALE) → signal clair que continuer à dézoomer = sortir.
//
// Aucune interaction (pointer-events: none).

import { useEffect, useState } from "react";
import { useGlucoseStore } from "../store";

const EXIT_SCALE = 0.4;       // doit matcher GlucoseCanvas
const FADE_START = 1.0;       // au-dessus → invisible

export default function FolderViewportIndicator() {
  const folderStack = useGlucoseStore((s) => s.folderStack);
  const project = useGlucoseStore((s) => s.project);
  const [scale, setScale] = useState(1);

  useEffect(() => {
    const onVp = (e: Event) => {
      const detail = (e as CustomEvent).detail as { scale: number };
      if (typeof detail?.scale === "number") setScale(detail.scale);
    };
    window.addEventListener("glucose:viewport-changed", onVp);
    return () => window.removeEventListener("glucose:viewport-changed", onVp);
  }, []);

  if (folderStack.length === 0) return null;

  // Récupère le folder courant (dernier de la pile)
  const top = folderStack[folderStack.length - 1];
  const parent = project.boards.find((b) => b.id === top.boardId);
  const folder = parent?.folders.find((f) => f.id === top.folderId);
  if (!folder) return null;

  // Opacité interpolée :
  //   scale ≥ FADE_START      → 0   (rien à signaler, on est bien dedans)
  //   scale ≤ EXIT_SCALE      → 1   (au seuil, sortie imminente)
  //   entre les deux          → interpolation linéaire
  const range = FADE_START - EXIT_SCALE;
  const t = Math.min(1, Math.max(0, (FADE_START - scale) / range));
  // Toujours un peu visible quand on est dans un folder, même à scale=1, pour
  // rappeler le contexte (15% mini).
  const baseOpacity = 0.15;
  const opacity = baseOpacity + (1 - baseOpacity) * t;

  // Largeur du cadre : épaisse quand t est grand, fine sinon
  const borderW = 4 + 12 * t;
  const glow = 30 + 60 * t;

  return (
    <>
      {/* Cadre principal */}
      <div
        style={{
          position: "fixed",
          inset: 0,
          pointerEvents: "none",
          border: `${borderW}px solid ${folder.color}`,
          borderRadius: 12,
          opacity,
          boxShadow: `inset 0 0 ${glow}px ${folder.color}`,
          transition: "opacity 120ms linear, border-width 120ms linear",
          zIndex: 40,
        }}
      />
      {/* Hint de sortie quand on est proche du seuil */}
      {t > 0.55 && (
        <div
          style={{
            position: "fixed",
            top: 24,
            left: "50%",
            transform: "translateX(-50%)",
            pointerEvents: "none",
            background: "rgba(15,15,18,0.85)",
            color: folder.color,
            padding: "6px 14px",
            borderRadius: 20,
            fontSize: 11,
            letterSpacing: 1,
            fontFamily: "system-ui, sans-serif",
            border: `1px solid ${folder.color}80`,
            boxShadow: `0 0 18px ${folder.color}55`,
            zIndex: 41,
            opacity: (t - 0.55) / 0.45,
            transition: "opacity 120ms",
          }}
        >
          ⤴ continue à dézoomer pour sortir de « {folder.name || "Dossier"} »
        </div>
      )}
    </>
  );
}
