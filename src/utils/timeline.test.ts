import { describe, it, expect } from "vitest";
import {
  parseAnchor, formatYear, formatAnchor,
  nodeMatchesTemporalFilter, tickStep, DEFAULT_ERAS,
} from "./timeline";

describe("formatYear", () => {
  it("affiche les années positives telles quelles", () => {
    expect(formatYear(2026)).toBe("2026");
    expect(formatYear(0)).toBe("0");
  });

  it("ajoute « av. J.-C. » sous 10 000 ans avant zéro", () => {
    expect(formatYear(-500)).toBe("500 av. J.-C.");
    expect(formatYear(-9999)).toBe("9999 av. J.-C.");
  });

  it("passe en kiloannées entre 10 ka et 1 Ma", () => {
    expect(formatYear(-10_000)).toBe("10.0 ka");
    expect(formatYear(-200_000)).toBe("200 ka");
  });

  it("passe en mégaannées au-delà", () => {
    expect(formatYear(-1_500_000)).toBe("1.5 Ma");
    expect(formatYear(-100_000_000)).toBe("100 Ma");
  });
});

describe("parseAnchor — année unique", () => {
  it("parse un entier", () => {
    expect(parseAnchor("1789")).toEqual({ start: 1789, end: 1789 });
  });

  it("parse un négatif", () => {
    expect(parseAnchor("-500")).toEqual({ start: -500, end: -500 });
  });

  it("parse « 500 av JC » et « 500 BC »", () => {
    expect(parseAnchor("500 av JC")).toEqual({ start: -500, end: -500 });
    expect(parseAnchor("500 BC")).toEqual({ start: -500, end: -500 });
    expect(parseAnchor("500 av. J.-C.")).toEqual({ start: -500, end: -500 });
  });

  it("parse « 1789 ap JC » et « 1789 AD »", () => {
    expect(parseAnchor("1789 ap JC")).toEqual({ start: 1789, end: 1789 });
    expect(parseAnchor("1789 AD")).toEqual({ start: 1789, end: 1789 });
  });

  it("parse les unités ka et Ma", () => {
    expect(parseAnchor("10 ka")).toEqual({ start: -10_000, end: -10_000 });
    expect(parseAnchor("12,5 ka")).toEqual({ start: -12_500, end: -12_500 });
    expect(parseAnchor("1,5 Ma")).toEqual({ start: -1_500_000, end: -1_500_000 });
    expect(parseAnchor("100 Ma")).toEqual({ start: -100_000_000, end: -100_000_000 });
  });
});

describe("parseAnchor — plages", () => {
  it("parse « 1789-1799 »", () => {
    expect(parseAnchor("1789-1799")).toEqual({ start: 1789, end: 1799 });
  });

  it("parse avec tiret cadratin et espaces", () => {
    expect(parseAnchor("1789 – 1799")).toEqual({ start: 1789, end: 1799 });
    expect(parseAnchor("1789 - 1799")).toEqual({ start: 1789, end: 1799 });
  });

  it("ordonne start <= end même si saisie inverse", () => {
    expect(parseAnchor("1799-1789")).toEqual({ start: 1789, end: 1799 });
  });
});

describe("parseAnchor — époques nommées", () => {
  it("reconnaît « Renaissance » avec son label", () => {
    const a = parseAnchor("Renaissance");
    expect(a).toEqual({ start: 1400, end: 1600, label: "Renaissance" });
  });

  it("est insensible à la casse", () => {
    expect(parseAnchor("renaissance")?.label).toBe("Renaissance");
    expect(parseAnchor("RENAISSANCE")?.label).toBe("Renaissance");
  });

  it("renvoie null pour une saisie absurde", () => {
    expect(parseAnchor("blabla")).toBeNull();
    expect(parseAnchor("")).toBeNull();
    expect(parseAnchor("   ")).toBeNull();
  });
});

describe("formatAnchor", () => {
  it("affiche le label si présent", () => {
    expect(formatAnchor({ start: 1400, end: 1600, label: "Renaissance" })).toBe("Renaissance");
  });

  it("affiche un point unique sans tiret si start == end", () => {
    expect(formatAnchor({ start: 1789, end: 1789 })).toBe("1789");
  });

  it("affiche une plage avec tiret cadratin", () => {
    expect(formatAnchor({ start: 1789, end: 1799 })).toBe("1789 – 1799");
  });
});

describe("nodeMatchesTemporalFilter", () => {
  it("nœud atemporel : toujours visible", () => {
    expect(nodeMatchesTemporalFilter(undefined, { start: 1000, end: 2000 })).toBe(true);
  });

  it("filtre nul : tout visible", () => {
    expect(nodeMatchesTemporalFilter({ start: 1789, end: 1799 }, null)).toBe(true);
  });

  it("intervalle entièrement dans le filtre : visible", () => {
    expect(nodeMatchesTemporalFilter({ start: 1789, end: 1799 }, { start: 1700, end: 1900 })).toBe(true);
  });

  it("intervalle qui chevauche partiellement : visible", () => {
    expect(nodeMatchesTemporalFilter({ start: 1789, end: 1900 }, { start: 1800, end: 2000 })).toBe(true);
    expect(nodeMatchesTemporalFilter({ start: 1700, end: 1800 }, { start: 1750, end: 2000 })).toBe(true);
  });

  it("intervalle qui contient le filtre : visible", () => {
    expect(nodeMatchesTemporalFilter({ start: 1000, end: 2000 }, { start: 1700, end: 1800 })).toBe(true);
  });

  it("intervalle disjoint avant : caché", () => {
    expect(nodeMatchesTemporalFilter({ start: 1500, end: 1600 }, { start: 1700, end: 1800 })).toBe(false);
  });

  it("intervalle disjoint après : caché", () => {
    expect(nodeMatchesTemporalFilter({ start: 1900, end: 2000 }, { start: 1700, end: 1800 })).toBe(false);
  });

  it("touche pile la borne : visible (bornes inclusives)", () => {
    expect(nodeMatchesTemporalFilter({ start: 1700, end: 1700 }, { start: 1700, end: 1800 })).toBe(true);
    expect(nodeMatchesTemporalFilter({ start: 1800, end: 1800 }, { start: 1700, end: 1800 })).toBe(true);
  });
});

describe("tickStep", () => {
  it("renvoie un pas adapté à la durée affichée", () => {
    expect(tickStep(10)).toBe(1);
    expect(tickStep(100)).toBe(10);
    expect(tickStep(2000)).toBe(100);
    expect(tickStep(20_000)).toBe(1000);
    expect(tickStep(2_000_000)).toBe(100_000);
    expect(tickStep(200_000_000)).toBe(10_000_000);
  });
});

describe("DEFAULT_ERAS", () => {
  it("toutes les époques ont start <= end", () => {
    for (const e of DEFAULT_ERAS) {
      expect(e.start).toBeLessThanOrEqual(e.end);
    }
  });

  it("contient les jalons attendus", () => {
    const names = DEFAULT_ERAS.map((e) => e.name);
    expect(names).toContain("Renaissance");
    expect(names).toContain("Révolution française");
    expect(names).toContain("Antiquité");
  });
});
