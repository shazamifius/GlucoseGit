import { describe, it, expect } from "vitest";
import { parseInlineRuns, splitBlocks, stripInlineMarkdown, wrapRuns } from "./markdownText";

describe("parseInlineRuns", () => {
  it("détecte gras et italique", () => {
    const runs = parseInlineRuns("normal **gras** et *ital*");
    const bold = runs.find((r) => r.bold);
    const ital = runs.find((r) => r.italic);
    expect(bold?.text).toContain("gras");
    expect(ital?.text).toContain("ital");
  });

  it("retire les liens en gardant le libellé", () => {
    const runs = parseInlineRuns("voir [Kant](https://x) ici");
    const joined = runs.map((r) => r.text).join("");
    expect(joined).toContain("Kant");
    expect(joined).not.toContain("https");
  });

  it("neutralise les délimiteurs LaTeX", () => {
    const runs = parseInlineRuns("formule $E=mc^2$ fin");
    const joined = runs.map((r) => r.text).join("");
    expect(joined).toContain("E=mc^2");
    expect(joined).not.toContain("$");
  });
});

describe("splitBlocks", () => {
  it("classe titres, puces et paragraphes", () => {
    const blocks = splitBlocks("### Titre\n- point\ntexte normal");
    expect(blocks[0].kind).toBe("h3");
    expect(blocks[1].kind).toBe("bullet");
    expect(blocks[2].kind).toBe("para");
  });

  it("ignore les lignes vides", () => {
    const blocks = splitBlocks("a\n\n\nb");
    expect(blocks.length).toBe(2);
  });
});

describe("stripInlineMarkdown", () => {
  it("aplati le markdown en texte nu", () => {
    expect(stripInlineMarkdown("**Newton** et *Leibniz*")).toBe("Newton et Leibniz");
  });
});

describe("wrapRuns", () => {
  const measure = (t: string, fs: number) => t.length * fs * 0.5;
  it("coupe en plusieurs lignes quand ça dépasse", () => {
    const runs = [{ text: "un deux trois quatre cinq six", bold: false, italic: false }];
    const lines = wrapRuns(runs, 10, 40, measure); // ~8 chars max par ligne
    expect(lines.length).toBeGreaterThan(1);
  });
  it("garde une seule ligne si ça rentre", () => {
    const runs = [{ text: "court", bold: false, italic: false }];
    const lines = wrapRuns(runs, 10, 1000, measure);
    expect(lines.length).toBe(1);
  });
});
