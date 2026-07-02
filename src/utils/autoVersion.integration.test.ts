// ────────────────────────────────────────────────────────────────────────────
// Git #1 Phase 3 — Test d'INTÉGRATION du jalon auto (chaîne complète).
//
// But : vérifier de bout en bout (sans la fenêtre Tauri) que
//   accumulateur → maybeCreateAutoVersion → saveVersion (I/O) → listVersions
// produit bien un jalon `auto`, et que l'élagage garde N jalons. On branche
// `@tauri-apps/plugin-fs` sur un système de fichiers EN MÉMOIRE (déterministe,
// sans dépendance node:fs — cf. convention du repo, hook-order.test.ts).
//
// Bonus : CALIBRATION — on mesure combien d'octets de delta Automerge pèsent des
// gestes typiques, pour juger si le seuil est atteignable en usage réel.
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it, vi } from "vitest";

// ── FS en mémoire branché sur l'API plugin-fs ────────────────────────────────
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
import { LIMITS } from "../constants";
import {
  noteSavedDelta, resetAutoVersionAccumulator, _peekAutoAccum, maybeCreateAutoVersion,
} from "./autoVersion";
import { listVersions } from "./versions";

const mainPath = "/mem/projet.glucose";

function mkDoc(): A.Doc<Project> {
  return A.create<Project>({
    version: "2.0.0", name: "calib",
    boards: [{
      id: "b0", name: "B0", images: [], annotations: [], panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0,
    }],
    activeBoardId: "b0", presets: [], domains: [], createdAt: 0, updatedAt: 0,
  });
}

function deltaSize(before: A.Doc<Project>, after: A.Doc<Project>): number {
  return A.getChanges(before, after).reduce((n, c) => n + c.length, 0);
}

beforeEach(() => {
  resetAutoVersionAccumulator();
  files.clear();
  dirs.clear();
});

describe("autoVersion — intégration (chaîne complète, fs mémoire)", () => {
  it("sous le seuil → aucun jalon écrit", async () => {
    noteSavedDelta(LIMITS.AUTO_VERSION_DELTA_BYTES - 1);
    await maybeCreateAutoVersion(mainPath, mkDoc());
    expect(await listVersions(mainPath)).toHaveLength(0);
  });

  it("au seuil → un jalon 'auto' est écrit et listé, compteur remis à zéro", async () => {
    noteSavedDelta(LIMITS.AUTO_VERSION_DELTA_BYTES);
    await maybeCreateAutoVersion(mainPath, mkDoc());
    const versions = await listVersions(mainPath);
    expect(versions).toHaveLength(1);
    expect(versions[0].kind).toBe("auto");
    expect(_peekAutoAccum()).toBe(0);
    // Le jalon se recharge en un doc valide (incorruptible par construction).
    const doc = await A.loadResilient<Project>(files.get(versions[0].path)!).doc;
    expect((A.asPlain(doc) as Project).name).toBe("calib");
  });

  it("élagage : au-delà de KEEP jalons auto, on ne garde que les KEEP plus récents", async () => {
    const N = LIMITS.AUTO_VERSION_KEEP + 4;
    for (let i = 0; i < N; i++) {
      resetAutoVersionAccumulator();
      noteSavedDelta(LIMITS.AUTO_VERSION_DELTA_BYTES);
      await maybeCreateAutoVersion(mainPath, mkDoc());
      await new Promise((r) => setTimeout(r, 2)); // horodatages distincts (nom = time)
    }
    const autos = (await listVersions(mainPath)).filter((v) => v.kind === "auto");
    expect(autos.length).toBe(LIMITS.AUTO_VERSION_KEEP);
  });

  it("CALIBRATION — combien d'octets pèsent des gestes typiques ?", () => {
    let doc = mkDoc();

    let before = doc;
    doc = A.change(doc, "add-text", (d) => {
      d.boards[0].annotations.push({
        id: "t1", type: "text", x: 100, y: 100, width: 300, height: 120,
        text: "Une idée de taille moyenne, avec un peu de contenu à retenir.", color: "",
      } as never);
    });
    const addText = deltaSize(before, doc);

    before = doc;
    doc = A.change(doc, "move", (d) => { (d.boards[0].annotations[0] as { x: number }).x = 250; });
    const move = deltaSize(before, doc);

    before = doc;
    doc = A.change(doc, "add-img", (d) => {
      d.boards[0].images.push({
        id: "i1", src: "asset:abcdef0123456789.jpg", x: 0, y: 0, width: 800, height: 600,
        rotation: 0, locked: false, tags: [], originalWidth: 800, originalHeight: 600,
      } as never);
    });
    const addImg = deltaSize(before, doc);

    const thresh = LIMITS.AUTO_VERSION_DELTA_BYTES;
    console.log(`[CALIBRATION] seuil = ${thresh} o (${(thresh / 1024).toFixed(0)} Ko)`);
    console.log(`[CALIBRATION] ajout bloc texte  = ${addText} o -> ~${Math.round(thresh / addText)} gestes`);
    console.log(`[CALIBRATION] déplacement        = ${move} o -> ~${Math.round(thresh / move)} gestes`);
    console.log(`[CALIBRATION] ajout image (lien) = ${addImg} o -> ~${Math.round(thresh / addImg)} gestes`);

    expect(addText).toBeGreaterThan(0);
    expect(move).toBeGreaterThan(0);
    expect(addImg).toBeGreaterThan(0);
  });
});
