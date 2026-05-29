// ────────────────────────────────────────────────────────────────────────────
// R-FIL-01 — Tests des helpers de drop universel (text/code → TextAnnotation).
// ────────────────────────────────────────────────────────────────────────────

import { describe, expect, it } from "vitest";
import { makeTextNodeFromFile, makeSourceSticky } from "./dropHandler";
import { isTextAnnotation, isStickyAnnotation } from "../types";

describe("makeTextNodeFromFile — wrapping selon extension", () => {
  it(".md → markdown brut (pas de fenced)", () => {
    const ann = makeTextNodeFromFile("notes.md", "# Hello\n\nworld", false, 0, 0);
    expect(isTextAnnotation(ann)).toBe(true);
    if (!isTextAnnotation(ann)) return;
    expect(ann.text).toContain("📄 notes.md");
    expect(ann.text).toContain("# Hello");
    expect(ann.text).not.toContain("```markdown"); // markdown brut, pas wrappé
  });

  it(".json → fenced ```json", () => {
    const ann = makeTextNodeFromFile("data.json", '{"a": 1}', false, 0, 0);
    if (!isTextAnnotation(ann)) throw new Error("not text");
    expect(ann.text).toMatch(/```json\n\{"a": 1\}\n```/);
  });

  it(".ts → fenced ```typescript", () => {
    const ann = makeTextNodeFromFile("file.ts", "const x = 1;", false, 0, 0);
    if (!isTextAnnotation(ann)) throw new Error("not text");
    expect(ann.text).toMatch(/```typescript\n[\s\S]+\n```/);
  });

  it(".py → fenced ```python", () => {
    const ann = makeTextNodeFromFile("script.py", "print('hi')", false, 0, 0);
    if (!isTextAnnotation(ann)) throw new Error("not text");
    expect(ann.text).toMatch(/```python\n[\s\S]+\n```/);
  });

  it(".csv → fenced ```csv", () => {
    const ann = makeTextNodeFromFile("data.csv", "a,b\n1,2", false, 0, 0);
    if (!isTextAnnotation(ann)) throw new Error("not text");
    expect(ann.text).toMatch(/```csv\n[\s\S]+\n```/);
  });

  it(".yaml → fenced ```yaml", () => {
    const ann = makeTextNodeFromFile("conf.yaml", "key: value", false, 0, 0);
    if (!isTextAnnotation(ann)) throw new Error("not text");
    expect(ann.text).toMatch(/```yaml\n[\s\S]+\n```/);
  });

  it(".txt → fenced ```text", () => {
    const ann = makeTextNodeFromFile("notes.txt", "plain stuff", false, 0, 0);
    if (!isTextAnnotation(ann)) throw new Error("not text");
    expect(ann.text).toMatch(/```text\n[\s\S]+\n```/);
  });

  it("contenu tronqué → footer explicite", () => {
    const ann = makeTextNodeFromFile("huge.log", "abc", true, 0, 0);
    if (!isTextAnnotation(ann)) throw new Error("not text");
    expect(ann.text).toContain("tronqué à");
  });

  it("position respectée + width par défaut", () => {
    const ann = makeTextNodeFromFile("a.md", "x", false, 123, 456);
    expect(ann.x).toBe(123);
    expect(ann.y).toBe(456);
    if (isTextAnnotation(ann)) {
      expect(ann.width).toBe(520);
      expect(ann.fontSize).toBe(12);
    }
  });
});

describe("makeSourceSticky — launcher pour binaires", () => {
  it(".blend → sticky avec sourceFile + couleur orange Blender", () => {
    const ann = makeSourceSticky("C:/scene.blend", 10, 20);
    expect(isStickyAnnotation(ann)).toBe(true);
    if (!isStickyAnnotation(ann)) return;
    expect(ann.sourceFile).toBe("C:/scene.blend");
    expect(ann.text).toBe("scene.blend");
    expect(ann.bgColor).toBe("#e87d0d"); // EXT_COLOR.blend
  });

  it(".psd → couleur bleu Photoshop", () => {
    const ann = makeSourceSticky("C:/work.psd", 0, 0);
    if (!isStickyAnnotation(ann)) return;
    expect(ann.bgColor).toBe("#31a8ff");
  });

  it("extension inconnue → couleur par défaut", () => {
    const ann = makeSourceSticky("C:/file.xyz", 0, 0);
    if (!isStickyAnnotation(ann)) return;
    expect(ann.bgColor).toBe("#1a1a2e");
  });
});
