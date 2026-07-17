// ────────────────────────────────────────────────────────────────────────────
// Anti-écrasement — deux mains sur le même .glucose.
//
// LE scénario, reproduit tel que l'utilisateur le vit : il ouvre un projet dans
// l'app, l'IA y écrit par le pont MCP pendant ce temps, il tape deux fois. Avant
// ce correctif, la compaction (mode "full" + rename) remplaçait le fichier par
// une version qui n'avait jamais entendu parler des notes de l'IA. Zéro erreur,
// zéro log : le pire défaut possible pour un outil qui promet d'être
// indestructible.
//
// FS en mémoire (convention du repo, pas de node:fs) : on exerce le VRAI code
// d'I/O de saveProject, y compris le tmp + rename.
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it, vi } from "vitest";

const files = new Map<string, Uint8Array>();
/** Relire le fichier pendant un save = une fusion a eu lieu. Signal précis :
 *  en solo, saveProject n'ouvre jamais sa cible. (setState, lui, est appelé à
 *  chaque save par le bloc de portabilité des chemins — il ne dit rien ici.) */
const relectures: string[] = [];

vi.mock("@tauri-apps/plugin-fs", () => ({
  writeFile: async (p: string, data: Uint8Array, opts?: { append?: boolean }) => {
    if (opts?.append && files.has(p)) {
      const prev = files.get(p)!;
      const merged = new Uint8Array(prev.length + data.length);
      merged.set(prev, 0); merged.set(data, prev.length);
      files.set(p, merged);
    } else {
      files.set(p, data.slice());
    }
  },
  rename: async (a: string, b: string) => {
    if (!files.has(a)) throw new Error(`rename: source absente ${a}`);
    files.set(b, files.get(a)!); files.delete(a);
  },
  readFile: async (p: string) => {
    relectures.push(p);
    if (!files.has(p)) throw new Error(`readFile: absent ${p}`);
    return files.get(p)!;
  },
  stat: async (p: string) => {
    if (!files.has(p)) throw new Error(`stat: absent ${p}`);
    return { size: files.get(p)!.length };
  },
  exists: async (p: string) => files.has(p),
}));

// Le store n'est qu'un CONFORT ici (voir les notes apparaître) : on le stub pour
// exercer l'I/O, qui est la garantie.
const setState = vi.fn();
vi.mock("../store", () => ({ useGlucoseStore: { setState: (...a: unknown[]) => setState(...a) } }));

import * as A from "../store/automerge";
import type { Project } from "../types";
import { saveProject } from "./project";
import { markLoaded, resetSaveState, expectedFileSize } from "./saveState";

const PATH = "/mem/projet.glucose";

const mkProject = (): Project => ({
  version: "2.0.0",
  name: "duo",
  boards: [{
    id: "b0", name: "B0",
    images: [], annotations: [], panels: [], zones: [], folders: [],
    viewport: { x: 0, y: 0, scale: 1 },
    createdAt: 0, updatedAt: 0,
  }],
  activeBoardId: "b0",
  presets: [], domains: [],
  createdAt: 0, updatedAt: 0,
} as unknown as Project);

/** Ajoute une note, comme une frappe de l'utilisateur ou une écriture de l'IA. */
const addNote = (doc: A.Doc<Project>, text: string) =>
  A.change(doc, `note ${text}`, (d: Project) => {
    (d.boards[0].annotations as unknown[]).push({
      id: `n-${text}`, type: "text", x: 0, y: 0, width: 380, text,
    });
  });

/** Dernier appel enregistré par un spy (`.at()` hors du lib target ici). */
const dernierAppel = <T>(spy: { mock: { calls: T[][] } }): T[] | undefined =>
  spy.mock.calls[spy.mock.calls.length - 1];

const notesOnDisk = (): string[] => {
  const d = A.load<Project>(files.get(PATH)!);
  return (A.asPlain(d) as Project).boards[0].annotations.map((a) => (a as { text: string }).text);
};

/** Le pont MCP : il relit le fichier, écrit ses notes, réécrit un save COMPLET. */
const iaEcritParLePont = (textes: string[]) => {
  let doc = A.load<Project>(files.get(PATH)!);
  for (const t of textes) doc = addNote(doc, t);
  files.set(PATH, A.save(doc));
};

describe("deux mains sur le même fichier", () => {
  beforeEach(() => {
    files.clear();
    relectures.length = 0;
    setState.mockClear();
    resetSaveState();
  });

  it("la compaction ne détruit plus le travail de l'IA — LE bug", async () => {
    // 1. L'utilisateur ouvre son projet. Assez de matière pour que la compaction
    //    se déclenche vite (elle arrive quand les deltas dépassent le full).
    let doc = A.create<Project>(mkProject());
    doc = addNote(doc, "note de l'utilisateur");
    await saveProject(doc as unknown as Project, PATH);

    // 2. L'IA écrit 10 notes par le pont, pendant que l'app est ouverte.
    const dixNotes = Array.from({ length: 10 }, (_, i) => `note IA ${i}`);
    iaEcritParLePont(dixNotes);
    expect(notesOnDisk()).toHaveLength(11);

    // 3. L'utilisateur tape. Plusieurs fois : c'est ce qui déclenchait la
    //    compaction, donc le mode "full", donc le rename qui écrase tout.
    for (let i = 0; i < 12; i++) {
      doc = addNote(doc, `frappe ${i}`);
      doc = (await saveProjectAndReadBack(doc)) as A.Doc<Project>;
    }

    // 4. Le verdict. Avant : 0/10. La perte était totale et silencieuse.
    const surLeDisque = notesOnDisk();
    for (const n of dixNotes) expect(surLeDisque, `« ${n} » a disparu`).toContain(n);
    expect(surLeDisque).toContain("note de l'utilisateur");
    for (let i = 0; i < 12; i++) expect(surLeDisque).toContain(`frappe ${i}`);
  });

  it("relit et adopte : l'utilisateur VOIT les notes de l'IA arriver", async () => {
    let doc = A.create<Project>(mkProject());
    doc = addNote(doc, "à moi");
    await saveProject(doc as unknown as Project, PATH);

    iaEcritParLePont(["à l'IA"]);
    doc = addNote(doc, "encore moi");
    await saveProject(doc as unknown as Project, PATH);

    expect(setState).toHaveBeenCalled();
    const adopte = dernierAppel(setState)![0] as { project: Project };
    const vues = adopte.project.boards[0].annotations.map((a) => (a as { text: string }).text);
    expect(vues).toContain("à l'IA");   // la note de l'IA est dans le doc adopté
    expect(vues).toContain("à moi");
  });

  it("ne se réveille pas quand personne d'autre n'écrit (pas de coût inutile)", async () => {
    let doc = A.create<Project>(mkProject());
    await saveProject(doc as unknown as Project, PATH);
    for (let i = 0; i < 6; i++) {
      doc = addNote(doc, `solo ${i}`);
      await saveProject(doc as unknown as Project, PATH);
    }
    expect(relectures).toHaveLength(0);   // le fichier n'a jamais été rouvert
    expect(notesOnDisk()).toHaveLength(6);
  });

  it("expectedFileSize colle à l'octet tant qu'on est seul à écrire", async () => {
    let doc = A.create<Project>(mkProject());
    await saveProject(doc as unknown as Project, PATH);
    for (let i = 0; i < 5; i++) {
      doc = addNote(doc, `n${i}`);
      await saveProject(doc as unknown as Project, PATH);
      expect(expectedFileSize(PATH)).toBe(files.get(PATH)!.length);
    }
  });

  it("un fichier illisible fait ÉCHOUER le save au lieu de l'écraser", async () => {
    const doc = A.create<Project>(mkProject());
    await saveProject(doc as unknown as Project, PATH);
    const avant = files.get(PATH)!;

    // Quelqu'un a laissé des octets qui ne sont pas de l'Automerge.
    files.set(PATH, new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]));
    const doc2 = addNote(doc, "ma frappe");
    await expect(saveProject(doc2 as unknown as Project, PATH)).rejects.toThrow(/écraser|fusion/i);

    // Et surtout : on n'y a PAS touché. Dans le doute, on ne détruit pas.
    expect(files.get(PATH)).not.toEqual(avant);        // (c'est bien le fichier trafiqué)
    expect(files.get(PATH)!.length).toBe(8);
  });

  it("un fichier absent n'empêche pas d'enregistrer", async () => {
    const doc = A.create<Project>(mkProject());
    await saveProject(doc as unknown as Project, PATH);
    markLoaded(PATH, doc as A.Doc<Project>, 999);      // baseline qui ment
    files.delete(PATH);                                 // et plus de fichier
    const doc2 = addNote(doc, "quand même");
    await expect(saveProject(doc2 as unknown as Project, PATH)).resolves.toBe(PATH);
    expect(notesOnDisk()).toContain("quand même");
  });
});

/** saveProject peut adopter un doc fusionné : on récupère celui qui fait foi. */
async function saveProjectAndReadBack(doc: A.Doc<Project>): Promise<A.Doc<Project>> {
  await saveProject(doc as unknown as Project, PATH);
  const last = dernierAppel(setState);
  return (last?.[0]?._doc as A.Doc<Project>) ?? doc;
}
