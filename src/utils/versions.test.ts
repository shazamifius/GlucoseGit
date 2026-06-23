import { describe, it, expect } from "vitest";
import {
  slugifyLabel,
  formatVersionFile,
  parseVersionFile,
  versionsDirFor,
} from "./versions";

describe("versions — nom de fichier (pur)", () => {
  it("versionsDirFor met le dossier à côté du .glucose", () => {
    expect(versionsDirFor("/a/b/film.glucose")).toBe("/a/b/film.glucose.versions");
  });

  it("slugifyLabel aplatit espaces et caractères interdits", () => {
    expect(slugifyLabel("Avant refonte")).toBe("Avant-refonte");
    expect(slugifyLabel('a/b:c*d?"<>|')).toBe("abcd");
    expect(slugifyLabel("   ")).toBe("jalon"); // jamais vide
    expect(slugifyLabel("a__b")).toBe("a_b"); // pas de double underscore (séparateur)
  });

  it("slugifyLabel borne la longueur", () => {
    expect(slugifyLabel("x".repeat(200)).length).toBeLessThanOrEqual(60);
  });

  it("format puis parse = round-trip cohérent", () => {
    const file = formatVersionFile(1700000000000, "manuel", "Première version");
    const parsed = parseVersionFile(file);
    expect(parsed).not.toBeNull();
    expect(parsed!.time).toBe(1700000000000);
    expect(parsed!.kind).toBe("manuel");
    expect(parsed!.label).toBe("Première version");
  });

  it("parse reconnaît le kind auto", () => {
    const file = formatVersionFile(123, "auto", "grosse modif");
    expect(parseVersionFile(file)!.kind).toBe("auto");
  });

  it("parse rejette un fichier qui n'est pas une version", () => {
    expect(parseVersionFile("film.glucose")).toBeNull(); // pas de séparateurs __
    expect(parseVersionFile("notes.txt")).toBeNull(); // mauvaise extension
    expect(parseVersionFile("xx__manuel__a.glucose")).toBeNull(); // time non numérique
  });

  it("un kind inconnu retombe sur manuel", () => {
    expect(parseVersionFile("123__bidon__a.glucose")!.kind).toBe("manuel");
  });

  it("garde un label contenant un tiret lisible", () => {
    const file = formatVersionFile(5, "manuel", "v1-final");
    expect(parseVersionFile(file)!.label).toBe("v1 final");
  });
});
