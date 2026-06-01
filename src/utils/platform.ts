// ────────────────────────────────────────────────────────────────────────────
// Détection de plateforme — desktop (Tauri) vs web (navigateur).
//
// Glucose tourne nativement via Tauri (backend Rust : accès disque, lancement
// d'apps, etc.). En build WEB/PWA, ce backend est absent : `isTauri()` permet
// de garder les fonctionnalités qui en dépendent et d'afficher un mode web propre.
// ────────────────────────────────────────────────────────────────────────────

/** `true` si l'app tourne dans Tauri (desktop), `false` en navigateur web/PWA. */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** `true` en build web/PWA (pas de backend natif). */
export function isWeb(): boolean {
  return !isTauri();
}
