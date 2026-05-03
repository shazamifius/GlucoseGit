// CLEANUP B-03 — Charge le CSS KaTeX (~200 KB) à la demande, uniquement
// quand on détecte du LaTeX dans une annotation (présence d'un `$`).
//
// Sans ça, le CSS était chargé au démarrage même pour 100% des projets
// qui ne contiennent jamais de math.

let loadPromise: Promise<unknown> | null = null;

/** Charge `katex.min.css` une seule fois. Idempotent. */
export function ensureKatexCss(): Promise<unknown> {
  if (!loadPromise) {
    loadPromise = import("katex/dist/katex.min.css").catch((err) => {
      // En cas d'échec, on autorise un retry futur en effaçant la promesse.
      loadPromise = null;
      throw err;
    });
  }
  return loadPromise;
}

/**
 * Helper : déclenche le chargement seulement si le texte semble contenir du
 * LaTeX (présence d'un `$` ou `\\(`). Heuristique simple suffisante pour
 * couvrir la grande majorité des cas tout en restant ultra-rapide.
 */
export function ensureKatexCssIfMath(text: string | undefined): void {
  if (!text) return;
  if (text.includes("$") || text.includes("\\(") || text.includes("\\[")) {
    void ensureKatexCss();
  }
}
