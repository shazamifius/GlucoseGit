// ────────────────────────────────────────────────────────────────────────────
// Chemin du .glucose courant — registre module-level (pas de cycle d'import).
// ────────────────────────────────────────────────────────────────────────────
//
// Le chemin du fichier ouvert vit dans `pathRef` (App.tsx). Du code hors-React
// (store, jalons durables) en a besoin sans pouvoir lire un ref de composant.
// Même motif que `collabHandle.ts` : un singleton minuscule, set par App à chaque
// fois que `pathRef.current` change (save / load), lu par `versions.ts`.

let _path: string | null = null;

/** Définit le chemin du .glucose actuellement ouvert (null = jamais enregistré). */
export function setCurrentPath(path: string | null): void {
  _path = path;
}

/** Chemin du .glucose courant, ou null si le projet n'a jamais été enregistré. */
export function getCurrentPath(): string | null {
  return _path;
}
