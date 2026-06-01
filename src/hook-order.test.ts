// ─────────────────────────────────────────────────────────────────────────────
// Analyseur statique : détecte les violations de la "Rules of Hooks" qui
// passent à travers les linters classiques :
//
//   « Un early `return` au niveau du body d'un composant React, suivi d'un
//     appel à un hook (useState/useEffect/useMemo/useRef/etc.) plus bas
//     dans le MÊME body. »
//
// Quand la condition de l'early return varie entre deux renders (ex :
// `folderStack.length === 0`), React détecte un changement de nombre de hooks
// et lève l'erreur minifiée #310 ("Rendered more hooks than during the
// previous render") — le bug qu'on chassait via le folder.
//
// Cette détection complète les tests de rendu : elle scrute le code source
// directement, sans monter le composant, donc elle attrape même les
// composants qu'on a oublié de couvrir.
// ─────────────────────────────────────────────────────────────────────────────

import { describe, expect, it } from "vitest";

// import.meta.glob (Vite) charge les sources en eager + raw — pas besoin de
// `node:fs` ni de `@types/node`, et ça reste lisible.
const SOURCE_FILES = import.meta.glob("./**/*.{ts,tsx}", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const HOOK_NAMES = [
  "useState", "useEffect", "useLayoutEffect", "useInsertionEffect",
  "useMemo", "useCallback", "useRef", "useReducer", "useContext",
  "useImperativeHandle", "useTransition", "useDeferredValue", "useId",
  "useSyncExternalStore", "useGlucoseStore",
];
const HOOK_CALL = new RegExp(`\\b(${HOOK_NAMES.join("|")})\\s*\\(`);

interface Violation {
  file: string;
  fnName: string;
  earlyReturnLine: number;
  hookLine: number;
  earlyReturnCode: string;
  hookCode: string;
}

function listSourceFiles(): Array<{ path: string; source: string }> {
  const out: Array<{ path: string; source: string }> = [];
  for (const [path, source] of Object.entries(SOURCE_FILES)) {
    if (/\.test\.(tsx?|jsx?)$/.test(path)) continue;
    out.push({ path, source });
  }
  return out;
}

// Découpe le fichier en "blocs de fonction de premier niveau" :
// retourne { fnName, startLine, endLine, bodyLines } pour chaque
// `function Name(...)` ou `const Name = (...) => {` à la racine du module
// (depth 0 hors function body). On ne descend pas dans les sous-fonctions.
interface TopLevelFn {
  name: string;
  startLine: number;     // 1-based, ligne de la signature
  bodyStartLine: number; // ligne du `{` ouvrant le body
  bodyEndLine: number;   // ligne du `}` fermant
  bodyLines: string[];   // contenu du body, ligne par ligne
}

function extractTopLevelFunctions(source: string): TopLevelFn[] {
  const lines = source.split(/\r?\n/);
  const fns: TopLevelFn[] = [];

  let depth = 0;             // {} de niveau module
  let inString: false | '"' | "'" | "`" = false;
  let inLineComment = false;
  let inBlockComment = false;

  // i = index ligne courant
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Détection début de fonction au top-level (depth === 0)
    if (depth === 0 && !inString && !inBlockComment) {
      const fnMatch =
        line.match(/^\s*(?:export\s+(?:default\s+)?)?(?:async\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]/) ||
        line.match(/^\s*(?:export\s+(?:default\s+)?)?const\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?::[^=]+)?=\s*(?:async\s*)?(?:\([^)]*\)|[A-Za-z_][A-Za-z0-9_]*)\s*=>\s*\{/);
      if (fnMatch) {
        const name = fnMatch[1];
        // On ne considère que les composants React (PascalCase) ou hooks (useXxx).
        if (/^[A-Z]/.test(name) || /^use[A-Z]/.test(name)) {
          // Trouve le `{` qui ouvre le body
          let bodyStart = -1;
          for (let j = i; j < lines.length; j++) {
            if (lines[j].includes("{")) { bodyStart = j; break; }
          }
          if (bodyStart === -1) continue;

          // Suit la profondeur pour trouver le `}` fermant
          let localDepth = 0;
          let bodyEnd = -1;
          let localInString: false | '"' | "'" | "`" = false;
          let localInLineComment = false;
          let localInBlockComment = false;
          for (let j = bodyStart; j < lines.length; j++) {
            const l = lines[j];
            localInLineComment = false;
            for (let k = 0; k < l.length; k++) {
              const c = l[k], next = l[k + 1];
              if (localInLineComment) continue;
              if (localInBlockComment) { if (c === "*" && next === "/") { localInBlockComment = false; k++; } continue; }
              if (localInString) {
                if (c === "\\") { k++; continue; }
                if (c === localInString) localInString = false;
                continue;
              }
              if (c === "/" && next === "/") { localInLineComment = true; break; }
              if (c === "/" && next === "*") { localInBlockComment = true; k++; continue; }
              if (c === '"' || c === "'" || c === "`") { localInString = c; continue; }
              if (c === "{") localDepth++;
              else if (c === "}") {
                localDepth--;
                if (localDepth === 0) { bodyEnd = j; break; }
              }
            }
            if (bodyEnd !== -1) break;
          }
          if (bodyEnd === -1) continue;

          fns.push({
            name,
            startLine: i + 1,
            bodyStartLine: bodyStart + 1,
            bodyEndLine: bodyEnd + 1,
            bodyLines: lines.slice(bodyStart, bodyEnd + 1),
          });
          // Saute au-delà du body pour ne pas re-trouver une sous-fonction
          i = bodyEnd;
          continue;
        }
      }
    }

    // Met à jour la profondeur module — on parcourt le line caractère par caractère.
    inLineComment = false;
    for (let k = 0; k < line.length; k++) {
      const c = line[k], next = line[k + 1];
      if (inLineComment) continue;
      if (inBlockComment) { if (c === "*" && next === "/") { inBlockComment = false; k++; } continue; }
      if (inString) {
        if (c === "\\") { k++; continue; }
        if (c === inString) inString = false;
        continue;
      }
      if (c === "/" && next === "/") { inLineComment = true; break; }
      if (c === "/" && next === "*") { inBlockComment = true; k++; continue; }
      if (c === '"' || c === "'" || c === "`") { inString = c; continue; }
      if (c === "{") depth++;
      else if (c === "}") depth--;
    }
  }

  return fns;
}

// Scrute UN body de fonction et retourne les early returns suivis de hooks
// au MÊME niveau d'indentation/profondeur du body.
function findViolationsInBody(fn: TopLevelFn, file: string): Violation[] {
  const out: Violation[] = [];
  const lines = fn.bodyLines;

  // Calcule la profondeur cumulée par ligne (relative au body, qui démarre à 0
  // juste après le `{` ouvrant — on ignore donc le `{` ouvrant lui-même).
  // Une instruction est "au top-level du body" si la profondeur AVANT la ligne == 0
  // (la première accolade après le body ouvrant n'a pas encore poussé).
  // On approxime : profondeur AVANT la ligne = somme des `{` − somme des `}` rencontrés
  // avant le début de cette ligne, EN PARTANT de la première ligne du body
  // mais en soustrayant le `{` initial.
  const depthBeforeLine: number[] = new Array(lines.length).fill(0);
  let d = 0;
  let seenInitial = false;
  let inString: false | '"' | "'" | "`" = false;
  let inBlockComment = false;
  for (let i = 0; i < lines.length; i++) {
    depthBeforeLine[i] = d;
    const l = lines[i];
    let lineHadInitial = seenInitial;
    let inLineComment = false;
    for (let k = 0; k < l.length; k++) {
      const c = l[k], next = l[k + 1];
      if (inLineComment) continue;
      if (inBlockComment) { if (c === "*" && next === "/") { inBlockComment = false; k++; } continue; }
      if (inString) {
        if (c === "\\") { k++; continue; }
        if (c === inString) inString = false;
        continue;
      }
      if (c === "/" && next === "/") { inLineComment = true; break; }
      if (c === "/" && next === "*") { inBlockComment = true; k++; continue; }
      if (c === '"' || c === "'" || c === "`") { inString = c; continue; }
      if (c === "{") {
        if (!seenInitial) { seenInitial = true; lineHadInitial = true; continue; }
        d++;
      } else if (c === "}") {
        d--;
      }
    }
    // Si cette ligne contenait le `{` initial du body, depthBeforeLine reste à 0
    // mais d a été pushé à 1 implicitement à partir de la prochaine itération
    // → on ne décale pas, c'est intentionnel.
    if (lineHadInitial && !seenInitial) seenInitial = true;
  }

  // Cherche les early returns au top-level (profondeur 0) suivis d'un appel
  // à un hook au top-level (profondeur 0) plus bas.
  let earlyReturnIdx = -1;
  let earlyReturnLine = "";
  for (let i = 0; i < lines.length; i++) {
    const l = lines[i];
    const trimmed = l.trim();
    if (trimmed.startsWith("//") || trimmed.startsWith("*") || trimmed.startsWith("/*")) continue;

    const depthForLine = depthBeforeLine[i];

    // Pattern: `if (...) return ...;` ou `if (...) { return ...; }` ou ternaire
    // (on se limite au premier pour rester précis).
    if (depthForLine === 0 && /^\s*if\s*\([^)]*\)\s*return\b/.test(l)) {
      earlyReturnIdx = i;
      earlyReturnLine = l.trim();
      continue;
    }
    // Pattern multi-ligne : `if (...) {\n   return ...;\n}` au top-level
    if (depthForLine === 0 && /^\s*if\s*\(/.test(l) && /\{\s*$/.test(l)) {
      // Cherche le `return` à l'intérieur du bloc
      for (let j = i + 1; j < lines.length; j++) {
        if (depthBeforeLine[j] === 1 && /^\s*return\b/.test(lines[j])) {
          earlyReturnIdx = i;
          earlyReturnLine = l.trim();
          break;
        }
        if (depthBeforeLine[j] === 0) break;
      }
    }

    // Détecte un appel à hook au TOP-LEVEL APRÈS un early return
    if (earlyReturnIdx !== -1 && depthForLine === 0 && i > earlyReturnIdx) {
      const m = l.match(HOOK_CALL);
      if (m && !/^\s*\/\//.test(l)) {
        // Ignore les appels du type `useGlucoseStore.getState()` (PAS un hook)
        // — ce sont des accès statiques au store Zustand, pas un appel React.
        if (l.includes(`${m[1]}.getState`)) continue;
        if (l.includes(`${m[1]}.setState`)) continue;
        if (l.includes(`${m[1]}.subscribe`)) continue;
        out.push({
          file,
          fnName: fn.name,
          earlyReturnLine: fn.bodyStartLine + earlyReturnIdx,
          hookLine: fn.bodyStartLine + i,
          earlyReturnCode: earlyReturnLine,
          hookCode: l.trim(),
        });
        // On NE break PAS : on rapporte chaque hook fautif pour un même return.
      }
    }
  }
  return out;
}

describe("hook order — analyse statique", () => {
  it("[méta] l'analyseur attrape bien un cas fautif synthétique", () => {
    // Sanity-check : on construit un body fautif inline et on vérifie que
    // findViolationsInBody le signale. Sans ça, un faux-positif silencieux
    // (l'analyseur ne trouvant rien) ferait passer le test global à tort.
    const fakeSource = [
      "export default function BadComponent() {",
      "  const [x, setX] = useState(0);",
      "  if (x === 0) return null;",
      "  useEffect(() => {}, []);",
      "  return null;",
      "}",
    ].join("\n");
    const fns = extractTopLevelFunctions(fakeSource);
    expect(fns.length).toBe(1);
    expect(fns[0].name).toBe("BadComponent");
    const violations = findViolationsInBody(fns[0], "fake.tsx");
    expect(violations.length).toBeGreaterThan(0);
    expect(violations[0].hookCode).toContain("useEffect");
  });

  it("[méta] l'analyseur ne lève rien sur un composant correct", () => {
    const goodSource = [
      "export default function GoodComponent() {",
      "  const [x, setX] = useState(0);",
      "  useEffect(() => {}, []);",
      "  if (x === 0) return null;",
      "  return null;",
      "}",
    ].join("\n");
    const fns = extractTopLevelFunctions(goodSource);
    expect(fns.length).toBe(1);
    const violations = findViolationsInBody(fns[0], "good.tsx");
    expect(violations).toEqual([]);
  });

  it("aucun composant React ne contient un early return AVANT un hook au top-level", () => {
    const files = listSourceFiles();
    const violations: Violation[] = [];

    for (const { path, source } of files) {
      const fns = extractTopLevelFunctions(source);
      for (const fn of fns) {
        violations.push(...findViolationsInBody(fn, path));
      }
    }

    if (violations.length > 0) {
      const report = violations.map(v =>
        `\n  ❌ ${v.file}\n` +
        `     fonction : ${v.fnName}\n` +
        `     early-return ligne ${v.earlyReturnLine}: ${v.earlyReturnCode}\n` +
        `     hook ligne ${v.hookLine}: ${v.hookCode}\n` +
        `     ↳ ce hook ne sera PAS appelé si la condition de l'early-return est vraie,\n` +
        `       ce qui cause React #310 quand l'état bascule.`
      ).join("\n");
      throw new Error(`Violations de la Rules of Hooks détectées :${report}`);
    }
    expect(violations).toEqual([]);
  });
});
