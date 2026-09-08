import { describe, expect, it } from "vitest";
import {
  BUILTIN_ENGINE,
  buildBoard,
  cardHeight,
  cardMarkdown,
  chooseModel,
  fileTitle,
  modelSizeB,
  parseCards,
  sectionTitle,
  splitSource,
  type EngineSection,
} from "./builtinEngine";

// Le moteur intégré tourne pendant plusieurs minutes contre une IA locale : ce
// qu'on peut tester sans elle, ce sont ses fonctions PURES — découpe, parsing de
// la réponse, géométrie. C'est là que vivent les bugs qui ruinent un run entier.

describe("splitSource", () => {
  it("rend une liste vide pour un texte vide", () => {
    expect(splitSource("")).toEqual([]);
    expect(splitSource("   \n\n  ")).toEqual([]);
  });

  it("coupe sur les titres Markdown quand le plafond l'impose", () => {
    const chunks = splitSource("# Un\ncorps un\n\n# Deux\ncorps deux", 20);
    expect(chunks.length).toBe(2);
    expect(chunks[0]).toContain("# Un");
    expect(chunks[1]).toContain("# Deux");
  });

  // Regrouper est VOULU : un appel par mini-section rendrait un cours de 30 Ko
  // interminable. Le titre retenu est alors le premier du groupe.
  it("regroupe deux petites sections qui tiennent dans le meme appel", () => {
    const chunks = splitSource("# Un\ncorps un\n\n# Deux\ncorps deux", 4500);
    expect(chunks.length).toBe(1);
    expect(sectionTitle(chunks[0], 0)).toBe("Un");
  });

  it("respecte le plafond même sans respiration dans le texte", () => {
    const chunks = splitSource("a".repeat(1000), 100);
    expect(chunks.length).toBeGreaterThan(1);
    for (const c of chunks) expect(c.length).toBeLessThanOrEqual(100);
  });

  it("regroupe les miettes plutôt que de faire un appel par ligne", () => {
    const source = Array.from({ length: 20 }, (_, i) => `# T${i}\nx`).join("\n");
    const chunks = splitSource(source, 4500);
    expect(chunks.length).toBe(1);
  });

  it("ne perd aucun contenu", () => {
    const source = "# A\nalpha\n\n# B\nbeta\n\n# C\ngamma";
    const joined = splitSource(source, 12).join("\n");
    for (const word of ["alpha", "beta", "gamma"]) expect(joined).toContain(word);
  });
});

describe("sectionTitle", () => {
  it("prend le titre Markdown quand il y en a un", () => {
    expect(sectionTitle("## La lumière\ntexte", 0)).toBe("La lumière");
  });

  it("retombe sur les premiers mots sinon", () => {
    expect(sectionTitle("la lumière voyage vite", 0)).toBe("la lumière voyage vite…");
  });

  it("nomme la partie quand le morceau n'a aucun texte", () => {
    expect(sectionTitle("   ", 3)).toBe("Partie 4");
  });
});

describe("parseCards", () => {
  it("lit la forme demandée", () => {
    const cards = parseCards('{"cartes":[{"titre":"T","resume":"R","points":["p1","p2"]}]}');
    expect(cards).toEqual([{ titre: "T", resume: "R", points: ["p1", "p2"] }]);
  });

  it("accepte un tableau nu et les clés anglaises", () => {
    const cards = parseCards('[{"title":"T","summary":"S","key_points":["k"]}]');
    expect(cards[0]).toEqual({ titre: "T", resume: "S", points: ["k"] });
  });

  it("accepte une clé de tableau inattendue", () => {
    expect(parseCards('{"concepts":[{"titre":"X"}]}')[0].titre).toBe("X");
    expect(parseCards('{"nimportequoi":[{"titre":"Y"}]}')[0].titre).toBe("Y");
  });

  it("survit aux fences et au bavardage autour du JSON", () => {
    const raw = 'Voici :\n```json\n{"cartes":[{"titre":"T"}]}\n```\nVoilà.';
    expect(parseCards(raw)[0].titre).toBe("T");
  });

  it("rend une liste vide plutôt que de jeter sur du non-JSON", () => {
    expect(parseCards("désolé je n'ai pas compris")).toEqual([]);
    expect(parseCards("")).toEqual([]);
  });

  it("ignore les entrées vides et borne les champs", () => {
    const raw = JSON.stringify({
      cartes: [{}, { titre: "x".repeat(400), points: Array.from({ length: 30 }, () => "p") }],
    });
    const cards = parseCards(raw);
    expect(cards.length).toBe(1);
    expect(cards[0].titre.length).toBeLessThanOrEqual(120);
    expect(cards[0].points.length).toBeLessThanOrEqual(8);
  });

  it("accepte une liste de chaînes", () => {
    expect(parseCards('{"cartes":["idée A","idée B"]}').map((c) => c.titre)).toEqual([
      "idée A",
      "idée B",
    ]);
  });
});

describe("chooseModel", () => {
  it("préfère le modèle conseillé s'il est installé", () => {
    expect(chooseModel("qwen2.5:32b", ["llama3:8b", "qwen2.5:32b"])).toBe("qwen2.5:32b");
  });

  it("retombe sur la même famille", () => {
    expect(chooseModel("qwen2.5:32b", ["qwen2.5:7b"])).toBe("qwen2.5:7b");
  });

  it("prend n'importe quel modèle présent plutôt que de refuser de tourner", () => {
    expect(chooseModel("qwen2.5:32b", ["mistral:7b"])).toBe("mistral:7b");
  });

  it("rend null quand rien n'est installé", () => {
    expect(chooseModel("qwen2.5:32b", [])).toBeNull();
    expect(chooseModel(null, [])).toBeNull();
  });
});

describe("modelSizeB", () => {
  it("lit la taille dans le tag du modèle", () => {
    expect(modelSizeB("qwen2.5:32b")).toBe(32);
    expect(modelSizeB("qwen2.5:7b")).toBe(7);
    expect(modelSizeB("llama3.1:8b-instruct-q4")).toBe(8);
  });

  it("rend null quand le nom ne dit rien", () => {
    expect(modelSizeB("mistral")).toBeNull();
    expect(modelSizeB("")).toBeNull();
  });

  // Le cas vécu : 32b installé alors que 7b tient dans la carte -> on avertit.
  it("permet de comparer le modèle utilisé au modèle qui tient en VRAM", () => {
    const used = modelSizeB("qwen2.5:32b");
    const fits = modelSizeB("qwen2.5:7b");
    expect(used !== null && fits !== null && used > fits * 1.5).toBe(true);
  });
});

describe("buildBoard", () => {
  const sections: EngineSection[] = [
    {
      titre: "Section A",
      cartes: [
        { titre: "A1", resume: "r", points: ["p"] },
        { titre: "A2", resume: "r", points: [] },
      ],
    },
    { titre: "Section B", cartes: [{ titre: "B1", resume: "r", points: [] }] },
  ];

  it("produit un board valide et non vide", () => {
    const board = buildBoard("Cours", sections);
    expect(board.name).toBe("Cours");
    expect(board.images).toEqual([]);
    expect(board.panels).toEqual([]);
    expect(board.zones).toEqual([]);
    expect(board.folders).toEqual([]);
    expect(board.annotations.length).toBeGreaterThan(0);
  });

  it("crée un titre par section et une carte par concept", () => {
    const board = buildBoard("Cours", sections);
    const titles = board.annotations.filter((a) => a.type === "text");
    const cards = board.annotations.filter((a) => a.type === "sticky");
    expect(titles.length).toBe(2);
    expect(cards.length).toBe(3);
  });

  it("relie les cartes d'une section dans l'ordre, jamais entre sections", () => {
    const board = buildBoard("Cours", sections);
    const arrows = board.annotations.filter((a) => a.type === "arrow");
    // 2 cartes dans A → 1 flèche ; 1 carte dans B → 0.
    expect(arrows.length).toBe(1);
    const cardIds = new Set(board.annotations.filter((a) => a.type === "sticky").map((a) => a.id));
    for (const a of arrows) {
      if (a.type !== "arrow") continue;
      expect(cardIds.has(a.sourceId ?? "")).toBe(true);
      expect(cardIds.has(a.targetId ?? "")).toBe(true);
    }
  });

  it("donne des identifiants uniques et des coordonnées finies", () => {
    const board = buildBoard("Cours", sections);
    const ids = board.annotations.map((a) => a.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (const a of board.annotations) {
      expect(Number.isFinite(a.x)).toBe(true);
      expect(Number.isFinite(a.y)).toBe(true);
    }
  });

  it("n'empile jamais deux cartes au même endroit", () => {
    const many: EngineSection[] = [
      {
        titre: "S",
        cartes: Array.from({ length: 9 }, (_, i) => ({
          titre: `C${i}`,
          resume: "r".repeat(i * 40),
          points: [],
        })),
      },
    ];
    const cards = buildBoard("Cours", many).annotations.filter((a) => a.type === "sticky");
    const seen = new Set(cards.map((c) => `${c.x}:${c.y}`));
    expect(seen.size).toBe(cards.length);
  });

  it("aligne tout sur une seule ligne en disposition « fil »", () => {
    const many: EngineSection[] = [
      { titre: "S", cartes: Array.from({ length: 7 }, (_, i) => ({ titre: `C${i}`, resume: "", points: [] })) },
    ];
    const cards = buildBoard("Cours", many, "fil").annotations.filter((a) => a.type === "sticky");
    expect(new Set(cards.map((c) => c.y)).size).toBe(1);
  });

  it("gère une section sans carte sans planter", () => {
    const board = buildBoard("Cours", [{ titre: "vide", cartes: [] }]);
    expect(board.annotations.length).toBe(1);
  });
});

describe("rendu d'une carte", () => {
  it("met le titre en gras et les points en liste", () => {
    const md = cardMarkdown({ titre: "T", resume: "R", points: ["a", "b"] });
    expect(md).toContain("**T**");
    expect(md).toContain("- a");
    expect(md).toContain("- b");
  });

  it("omet proprement les parties absentes", () => {
    expect(cardMarkdown({ titre: "T", resume: "", points: [] })).toBe("**T**");
  });

  it("garde une hauteur bornée", () => {
    const small = cardHeight({ titre: "T", resume: "", points: [] });
    const huge = cardHeight({ titre: "T", resume: "x".repeat(5000), points: [] });
    expect(small).toBeGreaterThanOrEqual(140);
    expect(huge).toBeLessThanOrEqual(420);
  });
});

describe("fileTitle", () => {
  it("retire le chemin et l'extension", () => {
    expect(fileTitle("C:\\Users\\x\\Downloads\\04-LUMIERE.md")).toBe("04-LUMIERE");
    expect(fileTitle("/home/x/cours.txt")).toBe("cours");
  });
});

describe("manifeste du moteur intégré", () => {
  it("expose des options dont chaque défaut existe dans ses choix", () => {
    expect(BUILTIN_ENGINE.options?.length).toBeGreaterThan(0);
    for (const opt of BUILTIN_ENGINE.options ?? []) {
      const values = (opt.choices ?? []).map((c) => c.value);
      expect(values).toContain(opt.default);
    }
  });
});
