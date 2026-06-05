// ────────────────────────────────────────────────────────────────────────────
// useZoomLock — empêche le zoom accidentel de la webview
// ────────────────────────────────────────────────────────────────────────────
//
// Symptôme corrigé : « soudainement tous les boutons sont devenus énormes ».
// En réalité la webview entière avait été zoomée par un Ctrl+molette (ou
// Ctrl + / Ctrl -) déclenché HORS du canvas (au-dessus de la barre d'outils par
// ex.). Le canvas neutralise déjà sa propre molette, mais pas le reste de l'app.
//
// Ce hook verrouille l'app à 100 % : elle s'adapte toujours à la taille de
// l'écran. Le zoom du CANVAS (molette sans Ctrl, géré par GlucoseCanvas) n'est
// pas affecté — on ne bloque que le zoom de la PAGE (Ctrl+molette, Ctrl±0).

import { useEffect } from "react";

export function useZoomLock(): void {
  useEffect(() => {
    // Ctrl/Cmd + molette = pinch-zoom de la page (et trackpad pinch synthétise
    // ctrlKey). On l'annule partout, en capture, non-passif pour pouvoir
    // preventDefault.
    const onWheel = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) e.preventDefault();
    };
    // Ctrl/Cmd + (+ | - | = | 0) = zoom clavier de la webview.
    const onKeyDown = (e: KeyboardEvent) => {
      if (!(e.ctrlKey || e.metaKey)) return;
      if (e.key === "+" || e.key === "-" || e.key === "=" || e.key === "0") {
        e.preventDefault();
      }
    };
    window.addEventListener("wheel", onWheel, { passive: false, capture: true });
    window.addEventListener("keydown", onKeyDown, { capture: true });
    return () => {
      window.removeEventListener("wheel", onWheel, { capture: true } as EventListenerOptions);
      window.removeEventListener("keydown", onKeyDown, { capture: true } as EventListenerOptions);
    };
  }, []);
}
