// Phase 7.5.B4 — Indicateur visuel quand on est dans un folder.
//
// Affiche un cadre subtil tout autour du viewport, de la couleur du dossier
// courant. L'opacité augmente quand l'utilisateur dézoome vers le seuil de
// sortie RÉEL → signal clair que continuer à dézoomer = sortir.
//
// R-FIL — Le seuil de sortie est désormais ADAPTATIF (dépend de la taille du
// dossier) : un petit dossier se quitte vite, un dossier énorme se laisse
// explorer longtemps. Le cadre s'allumait AVANT en permanence (seuil fixe 0.4
// vs FADE_START 1.0) alors qu'on était loin de sortir. On consomme maintenant
// le `exitScale` diffusé par le canvas (event viewport-changed) et on ne révèle
// le cadre qu'à l'APPROCHE réelle du seuil (entre exitScale et ~2× exitScale).
//
// Aucune interaction (pointer-events: none).

import { useEffect, useState } from "react";
import { useGlucoseStore } from "../store";

export default function FolderViewportIndicator() {
  const folderStack = useGlucoseStore((s) => s.folderStack);
  const project = useGlucoseStore((s) => s.project);
  const [scale, setScale] = useState(1);
  // Seuil de sortie adaptatif diffusé par le canvas (0 = inconnu/hors dossier).
  const [exitScale, setExitScale] = useState(0.4);

  useEffect(() => {
    const onVp = (e: Event) => {
      const detail = (e as CustomEvent).detail as { scale: number; exitScale?: number };
      if (typeof detail?.scale === "number") setScale(detail.scale);
      if (typeof detail?.exitScale === "number" && detail.exitScale > 0) {
        setExitScale(detail.exitScale);
      }
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

  // Fenêtre d'apparition : on commence à révéler le cadre à ~2× le seuil réel,
  // plein à exitScale. Hors de cette fenêtre (bien dans le dossier) → 0 → rien.
  const fadeStart = exitScale * 2.0;
  const range = Math.max(0.0001, fadeStart - exitScale);
  const t = Math.min(1, Math.max(0, (fadeStart - scale) / range));

  // Plus de glow constant : on n'affiche RIEN tant qu'on n'approche pas la
  // sortie (corrige « la bande bleue est constamment active »).
  if (t <= 0.02) return null;

  const opacity = t;
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
