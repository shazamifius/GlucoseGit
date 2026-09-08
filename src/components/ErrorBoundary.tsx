import { Component, type ReactNode } from "react";
import { reportError } from "../telemetry/telemetry";

/** Isole un crash de rendu React à la sous-arborescence fautive au lieu de faire
 *  tomber toute l'application. Extrait de `App.tsx` pour que les autres couches
 *  (le dock de panneaux, notamment) puissent isoler chacun de leurs enfants —
 *  un panneau qui plante ne doit pas emporter les panneaux voisins. */
export default class ErrorBoundary extends Component<{ children: ReactNode }, { error: Error | null }> {
  state = { error: null };
  static getDerivedStateFromError(e: Error) { return { error: e }; }
  componentDidCatch(error: Error) {
    // Remonte le crash à la télémétrie (no-op si l'utilisateur n'a pas consenti).
    reportError("react-boundary", error.message, error.stack);
  }
  render() {
    if (this.state.error) {
      const err = this.state.error as Error;
      return (
        <div style={{ padding: 32, color: "#f87171", background: "#0d0d0d", height: "100%", fontFamily: "monospace" }}>
          <b>Erreur :</b> {err.message}
          <pre style={{ fontSize: 11, marginTop: 12, color: "#666", whiteSpace: "pre-wrap" }}>{err.stack}</pre>
        </div>
      );
    }
    return this.props.children;
  }
}
