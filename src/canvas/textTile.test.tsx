// ────────────────────────────────────────────────────────────────────────────
// TXT-1 / TXT-2 — Tests des helpers de tuile texte de folder mirror :
//   - clipTextForTile : retire l'en-tête « ### 📄 », clippe, referme les fences.
//   - highlightCode   : colore mots-clés / chaînes / nombres / commentaires.
// ────────────────────────────────────────────────────────────────────────────

import { isValidElement, type ReactNode } from "react";
import { describe, expect, it } from "vitest";
import { clipTextForTile, highlightCode } from "./HtmlAnnotationLayer";

/** Couleurs des spans produits (ignore les bouts de texte bruts). */
function spanColors(nodes: ReactNode[]): string[] {
  const colors: string[] = [];
  for (const n of nodes) {
    if (isValidElement(n)) {
      const c = (n.props as { style?: { color?: string } }).style?.color;
      if (c) colors.push(c);
    }
  }
  return colors;
}

describe("clipTextForTile — TXT-1", () => {
  it("retire l'en-tête « ### 📄 nom » (redondant avec l'en-tête de la tuile)", () => {
    const out = clipTextForTile("### 📄 a.md\n\nBonjour le monde");
    expect(out).toBe("Bonjour le monde");
  });

  it("clippe au-delà de maxLines et ajoute une ellipse", () => {
    const raw = Array.from({ length: 200 }, (_, i) => `ligne ${i}`).join("\n");
    const out = clipTextForTile(raw, 20);
    expect(out.split("\n").length).toBeLessThanOrEqual(21); // 20 + l'ellipse
    expect(out.endsWith("…")).toBe(true);
  });

  it("referme un bloc de code laissé ouvert par la coupe", () => {
    const raw = "```python\n" + Array.from({ length: 50 }, (_, i) => `x = ${i}`).join("\n");
    const out = clipTextForTile(raw, 10);
    // Nb de ``` pair → le fence est refermé.
    expect((out.match(/```/g) || []).length % 2).toBe(0);
  });
});

describe("highlightCode — TXT-2", () => {
  it("python : mots-clés bleus, commentaire vert, nombre, chaîne orange", () => {
    const nodes = highlightCode(`def foo():\n    return "hi"  # note\n    n = 42`, "python");
    const colors = spanColors(nodes);
    expect(colors).toContain("#569cd6"); // mot-clé (def / return)
    expect(colors).toContain("#ce9178"); // chaîne "hi"
    expect(colors).toContain("#b5cea8"); // nombre 42
    expect(colors).toContain("#6a9955"); // commentaire # note
  });

  it("le commentaire python utilise # (pas //)", () => {
    const nodes = highlightCode("x = 1  # commentaire", "python");
    // Le « # commentaire » entier est un seul span vert.
    const comment = nodes.find(
      (n) => isValidElement(n) && (n.props as { style?: { color?: string } }).style?.color === "#6a9955",
    );
    expect(comment).toBeDefined();
    expect(isValidElement(comment) && (comment.props as { children?: string }).children).toContain("# commentaire");
  });

  it("langage C-like : commentaire // reconnu", () => {
    const nodes = highlightCode("const x = 1; // inline", "javascript");
    const colors = spanColors(nodes);
    expect(colors).toContain("#569cd6"); // const
    expect(colors).toContain("#6a9955"); // // inline
  });

  it("une chaîne contenant un # n'est pas coupée en commentaire", () => {
    const nodes = highlightCode(`s = "a # b"`, "python");
    const str = nodes.find(
      (n) => isValidElement(n) && (n.props as { style?: { color?: string } }).style?.color === "#ce9178",
    );
    expect(isValidElement(str) && (str.props as { children?: string }).children).toBe('"a # b"');
  });
});
