// ────────────────────────────────────────────────────────────────────────────
// Git #1 Phase 4 — Test du filet anti-corruption au chargement.
// On branche `@tauri-apps/plugin-fs` sur un FS en mémoire (cf. convention repo,
// pas de node:fs) pour exercer le vrai code I/O de versions.ts.
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
import {
  saveVersion, listVersions, loadLatestHealthyVersion, formatVersionFile, versionsDirFor,
} from "./versions";

const mainPath = "/mem/projet.glucose";

function mkDoc(name: string): A.Doc<Project> {
  return A.create<Project>({
    version: "2.0.0", name,
    boards: [{
      id: "b0", name: "B0", images: [], annotations: [], panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0,
    }],
    activeBoardId: "b0", presets: [], domains: [], createdAt: 0, updatedAt: 0,
  });
}

beforeEach(() => { files.clear(); dirs.clear(); });

describe("loadLatestHealthyVersion — filet Git #1 Phase 4", () => {
  it("aucun jalon → null", async () => {
    expect(await loadLatestHealthyVersion(mainPath)).toBeNull();
  });

  it("renvoie le jalon le plus récent quand tout est sain", async () => {
    await saveVersion(mainPath, mkDoc("v1"), "premier", "manuel");
    await new Promise((r) => setTimeout(r, 3)); // horodatages distincts
    await saveVersion(mainPath, mkDoc("v2"), "dernier", "auto");

    const res = await loadLatestHealthyVersion(mainPath);
    expect(res).not.toBeNull();
    expect((A.asPlain(res!.doc) as Project).name).toBe("v2");
    expect(res!.meta.label).toBe("dernier");
  });

  it("saute un jalon CORROMPU et renvoie le précédent sain", async () => {
    await saveVersion(mainPath, mkDoc("sain"), "bon", "manuel");

    // Injecte un jalon corrompu avec un horodatage PLUS RÉCENT (listé en premier).
    const later = Date.now() + 100_000;
    const dir = versionsDirFor(mainPath);
    files.set(`${dir}/${formatVersionFile(later, "auto", "abime")}`,
      new Uint8Array([0xff, 0x00, 0x13, 0x37, 0x42, 0x99, 0x01, 0x02]));

    const versions = await listVersions(mainPath);
    expect(versions).toHaveLength(2);
    expect(versions[0].time).toBe(later); // le corrompu est bien le plus récent

    const res = await loadLatestHealthyVersion(mainPath);
    expect(res).not.toBeNull();
    expect((A.asPlain(res!.doc) as Project).name).toBe("sain"); // corrompu sauté
    expect(res!.meta.label).toBe("bon");
  });

  it("tous les jalons corrompus → null", async () => {
    const dir = versionsDirFor(mainPath);
    dirs.add(dir);
    for (let i = 0; i < 3; i++) {
      files.set(`${dir}/${formatVersionFile(Date.now() + i, "auto", `k${i}`)}`,
        new Uint8Array([1, 2, 3, 4]));
    }
    expect(await loadLatestHealthyVersion(mainPath)).toBeNull();
  });
});
