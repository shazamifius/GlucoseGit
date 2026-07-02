// ────────────────────────────────────────────────────────────────────────────
// Git #1 Phase 4 p2 — Orchestration de la compaction, bout en bout, sur un FS en
// mémoire (cf. convention repo, pas de node:fs). On exerce le VRAI code I/O :
// jalon de secours écrit AVANT, remplacement ATOMIQUE du fichier principal, et le
// fichier rechargé doit être identique à l'état courant. Plus le garde-fou solo.
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it, vi } from "vitest";

const files = new Map<string, Uint8Array>();
const dirs = new Set<string>();

vi.mock("@tauri-apps/plugin-fs", () => ({
  mkdir: async (p: string) => { dirs.add(p); },
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
    if (!files.has(p)) throw new Error(`readFile: absent ${p}`);
    return files.get(p)!;
  },
  readDir: async (p: string) => {
    if (!dirs.has(p)) throw new Error(`readDir: dossier absent ${p}`);
    const prefix = `${p}/`;
    const out: { name: string; isFile: boolean; isDirectory: boolean; isSymlink: boolean }[] = [];
    for (const key of files.keys()) {
      if (key.startsWith(prefix) && !key.slice(prefix.length).includes("/")) {
        out.push({ name: key.slice(prefix.length), isFile: true, isDirectory: false, isSymlink: false });
      }
    }
    return out;
  },
  remove: async (p: string) => { files.delete(p); dirs.delete(p); },
  exists: async (p: string) => files.has(p) || dirs.has(p),
}));

import * as A from "../store/automerge";
import type { Project } from "../types";
import { runCompaction, CompactionError } from "./compaction";
import { listVersions, versionsDirFor } from "./versions";
import { resetSaveState, _peekBaseline } from "./saveState";
import { setCollabHandle } from "../multiplayer/collabHandle";

const mainPath = "/mem/projet.glucose";

function mkDocWithHistory(n: number): A.Doc<Project> {
  let d = A.create<Project>({
    version: "2.0.0", name: "projet",
    boards: [{
      id: "b0", name: "B0", images: [], annotations: [], panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0,
    }],
    activeBoardId: "b0", presets: [], domains: [], createdAt: 0, updatedAt: 0,
  });
  d = A.change(d, "add", (p) => {
    p.boards[0].annotations.push({
      id: "a0", type: "text", content: "x", x: 0, y: 0, width: 100, height: 40,
    } as unknown as Project["boards"][number]["annotations"][number]);
  });
  for (let i = 0; i < n; i++) {
    d = A.change(d, "move", (p) => {
      const a = p.boards[0].annotations[0] as unknown as { x: number; y: number };
      a.x = i; a.y = i;
    });
  }
  return d;
}

beforeEach(() => {
  files.clear();
  dirs.clear();
  resetSaveState();
  setCollabHandle(null); // s'assure qu'on est en SOLO
});

describe("runCompaction — orchestration béton", () => {
  it("pose un jalon « avant compaction » AVANT de remplacer le fichier", async () => {
    const doc = mkDocWithHistory(300);
    files.set(mainPath, A.save(doc)); // le fichier vivant existe

    const res = await runCompaction(mainPath, doc);
    expect(res).not.toBeNull();

    const versions = await listVersions(mainPath);
    expect(versions).toHaveLength(1);
    expect(versions[0].label).toBe("avant compaction");
    expect(versions[0].kind).toBe("auto");
  });

  it("remplace le fichier ATOMIQUEMENT, plus léger, et rechargeable à l'identique", async () => {
    const doc = mkDocWithHistory(400);
    const originalBytes = A.save(doc);
    files.set(mainPath, originalBytes);

    const res = await runCompaction(mainPath, doc);
    expect(res).not.toBeNull();

    // Fichier principal remplacé et allégé.
    const onDisk = files.get(mainPath)!;
    expect(onDisk.length).toBeLessThan(originalBytes.length);
    expect(res!.after).toBe(onDisk.length);
    expect(res!.before).toBe(originalBytes.length);

    // Aucun .tmp résiduel (écriture atomique menée à terme).
    expect(files.has(`${mainPath}.tmp`)).toBe(false);

    // Le fichier sur disque se recharge EXACTEMENT sur l'état courant.
    const reloaded = A.loadResilient<Project>(onDisk).doc;
    expect(A.asPlain(reloaded)).toEqual(A.asPlain(doc));
  });

  it("pose le baseline de save incrémental sur le doc compacté", async () => {
    const doc = mkDocWithHistory(200);
    files.set(mainPath, A.save(doc));

    const res = await runCompaction(mainPath, doc);
    const baseline = _peekBaseline();
    expect(baseline).not.toBeNull();
    expect(baseline!.path).toBe(mainPath);
    expect(baseline!.fullSize).toBe(res!.after); // taille du compacté
    expect(baseline!.appendedSize).toBe(0);
  });

  it("REFUSE en collaboration (mode solo requis) sans toucher au fichier", async () => {
    const doc = mkDocWithHistory(300);
    const originalBytes = A.save(doc);
    files.set(mainPath, originalBytes);
    setCollabHandle({ url: "automerge:test" } as unknown as Parameters<typeof setCollabHandle>[0]);

    await expect(runCompaction(mainPath, doc)).rejects.toBeInstanceOf(CompactionError);

    // Fichier intact, aucun jalon écrit.
    expect(files.get(mainPath)).toBe(originalBytes);
    expect(dirs.has(versionsDirFor(mainPath))).toBe(false);

    setCollabHandle(null);
  });

  it("renvoie null (no-op) si le doc est déjà compact — aucun jalon, fichier intact", async () => {
    const doc = mkDocWithHistory(0); // 2 changes seulement (init + add)
    // Un doc quasi neuf : le compacté n'est pas plus petit → rien à gagner.
    const compactAlready = A.create<Project>(A.asPlain(doc));
    const bytes = A.save(compactAlready);
    files.set(mainPath, bytes);

    const res = await runCompaction(mainPath, compactAlready);
    expect(res).toBeNull();
    expect(files.get(mainPath)).toBe(bytes); // fichier non touché
    expect(dirs.has(versionsDirFor(mainPath))).toBe(false); // pas de jalon
  });
});
