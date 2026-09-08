import { describe, expect, it } from "vitest";
import { ICON_MAP } from "./Toast";

// Les toasts sont appelés avec un emoji en 2ᵉ argument, que `Toast` remplace par
// un SVG monochrome via ICON_MAP. Un emoji NON déclaré retombe silencieusement
// sur le rendu système — c'est-à-dire un pictogramme en COULEUR au milieu d'une
// interface entièrement en niveaux de gris. Rien ne casse, donc rien ne le
// signale : d'où ce test, qui relit les sources et refuse tout emoji orphelin.
//
// Les sources sont lues via `import.meta.glob` (Vite) plutôt que `node:fs` :
// le tsconfig du projet ne charge pas les types Node, et il n'y a aucune raison
// de les y ajouter pour un seul test.
const SOURCES = import.meta.glob("../**/*.{ts,tsx}", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

/** 2ᵉ argument littéral de chaque `showToast(...)` du projet. */
function collectToastIcons(): Map<string, string[]> {
  const found = new Map<string, string[]>();
  for (const [path, src] of Object.entries(SOURCES)) {
    if (/\.test\.tsx?$/.test(path)) continue;
    for (const m of src.matchAll(/showToast\((?:[^;]*?),\s*"([^"]+)"\s*\)/g)) {
      const list = found.get(m[1]) ?? [];
      list.push(path);
      found.set(m[1], list);
    }
  }
  return found;
}

describe("Toast — icônes monochromes", () => {
  const used = collectToastIcons();

  it("trouve bien les appels showToast du projet (le regex n'est pas cassé)", () => {
    expect(used.size).toBeGreaterThan(10);
  });

  it("chaque icône de toast a un glyphe SVG — aucun emoji couleur", () => {
    const orphans = [...used.entries()]
      .filter(([icon]) => !ICON_MAP[icon])
      .map(([icon, files]) => `${icon} (${[...new Set(files)].join(", ")})`);
    expect(orphans).toEqual([]);
  });

  it("la table ne garde pas de glyphe mort", () => {
    expect(Object.keys(ICON_MAP).filter((k) => !used.has(k))).toEqual([]);
  });

  it("tous les glyphes se peignent en currentColor (aucune couleur en dur)", () => {
    const src = SOURCES["../components/Toast.tsx"] ?? SOURCES["./Toast.tsx"];
    expect(src).toBeTruthy();
    const painted = [...src.matchAll(/\b(?:stroke|fill)="([^"]+)"/g)].map((m) => m[1]);
    expect(painted.length).toBeGreaterThan(0);
    for (const v of painted) expect(["currentColor", "none"]).toContain(v);
  });
});
