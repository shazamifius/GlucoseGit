// Setup commun pour tous les tests Vitest.
//
// - matchMedia : jsdom ne l'implémente pas, certains composants Tailwind / PixiJS
//   en ont besoin.
// - ResizeObserver : idem.
// - URL.createObjectURL : utilisé par certains imports d'image.
// - crypto.randomUUID : fallback pour nanoid si nécessaire (déjà importé).
// - console.error : on intercepte pour faire échouer le test en cas d'erreur
//   React (incluant les hooks violations) — c'est PRÉCISÉMENT ce qu'on
//   cherche à détecter avec les smoke tests composants.

import "@testing-library/jest-dom/vitest";

if (typeof window !== "undefined") {
  if (!window.matchMedia) {
    window.matchMedia = (query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    });
  }
  if (!window.ResizeObserver) {
    // Stub minimal pour jsdom (qui n'implémente pas ResizeObserver).
    class StubResizeObserver {
      observe() {}
      unobserve() {}
      disconnect() {}
    }
    window.ResizeObserver = StubResizeObserver as unknown as typeof ResizeObserver;
  }
  if (!URL.createObjectURL) {
    URL.createObjectURL = () => "blob:test";
  }
}
