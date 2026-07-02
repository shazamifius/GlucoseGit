// ────────────────────────────────────────────────────────────────────────────
// Bundle portable — orchestration bout-en-bout sur FS mémoire (convention repo,
// pas de node:fs) + magasin global d'assets simulé via mock de `invoke`.
// La garantie : exporter puis IMPORTER sur une machine au magasin VIDE ré-hydrate
// exactement les images, et le doc se recharge à l'identique.
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it, vi } from "vitest";

const files = new Map<string, Uint8Array>();
const dirs = new Set<string>();
const store = new Map<string, Uint8Array>(); // magasin global d'assets : filename -> bytes

vi.mock("@tauri-apps/plugin-fs", () => ({
  mkdir: async (p: string) => { dirs.add(p); },
  writeFile: async (p: string, data: Uint8Array) => { files.set(p, data.slice()); },
  readFile: async (p: string) => {
    if (!files.has(p)) throw new Error(`readFile: absent ${p}`);
    return files.get(p)!;
  },
}));

// La copie d'octets se fait côté Rust (bundle_export_assets / bundle_import_assets) :
// on les mocke comme une copie entre le magasin global (`store`) et le fs (`files`).
vi.mock("@tauri-apps/api/core", () => ({
  invoke: async (cmd: string, args: Record<string, unknown>) => {
    if (cmd === "bundle_export_assets") {
      const names = args.assetNames as string[];
      const dest = args.destObjectsDir as string;
      let copied = 0;
      const missing: string[] = [];
      for (const n of names) {
        const b = store.get(n);
        if (!b) { missing.push(n); continue; }
        files.set(`${dest}/${n}`, b.slice()); // magasin global → objects/ du bundle
        copied++;
      }
      return { copied, missing, corrupt: [] };
    }
    if (cmd === "bundle_import_assets") {
      const names = args.assetNames as string[];
      const src = args.srcObjectsDir as string;
      let copied = 0;
      const missing: string[] = [];
      for (const n of names) {
        const key = `${src}/${n}`;
        if (!files.has(key)) { missing.push(n); continue; }
        store.set(n, files.get(key)!.slice()); // objects/ du bundle → magasin global
        copied++;
      }
      return { copied, missing, corrupt: [] };
    }
    throw new Error(`invoke inattendu: ${cmd}`);
  },
}));

import * as A from "../store/automerge";
import type { BoardImage, Project } from "../types";
import { sha256Hex } from "./assetRef";
import { BundleError, exportBundle, importBundle } from "./bundle";

const DIR = "/mem/bundle";

function imgBytes(seed: number, n = 64): Uint8Array {
  const b = new Uint8Array(n);
  for (let i = 0; i < n; i++) b[i] = (i * 31 + seed * 7 + 3) & 0xff;
  return b;
}

async function storeName(bytes: Uint8Array, ext = "png"): Promise<string> {
  const full = await sha256Hex(bytes);
  return `${full.slice(0, 16)}.${ext}`;
}

function linkImg(id: string, name: string, sha256: string): BoardImage {
  return {
    id, asset: { mode: "link", href: `asset:${name}`, sha256 },
    x: 0, y: 0, width: 10, height: 10, rotation: 0, locked: false,
    tags: [], originalWidth: 10, originalHeight: 10,
  };
}

/** Doc à 2 images, dont les octets sont PRÉSENTS dans le magasin global. */
async function mkDocWithAssets(): Promise<{ doc: A.Doc<Project>; names: string[] }> {
  const b1 = imgBytes(1);
  const b2 = imgBytes(2);
  const n1 = await storeName(b1);
  const n2 = await storeName(b2);
  store.set(n1, b1);
  store.set(n2, b2);
  const doc = A.create<Project>({
    version: "2.0.0", name: "projet",
    boards: [{
      id: "b0", name: "B0",
      images: [linkImg("i1", n1, await sha256Hex(b1)), linkImg("i2", n2, await sha256Hex(b2))],
      annotations: [], panels: [], zones: [], folders: [],
      viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0,
    }],
    activeBoardId: "b0", presets: [], domains: [], createdAt: 0, updatedAt: 0,
  });
  return { doc, names: [n1, n2] };
}

beforeEach(() => {
  files.clear();
  dirs.clear();
  store.clear();
});

describe("exportBundle — écriture du dossier portable", () => {
  it("écrit le doc + objects/<hash> + manifeste, tout inclus", async () => {
    const { doc, names } = await mkDocWithAssets();
    const res = await exportBundle(doc, DIR);

    expect(res.included).toBe(2);
    expect(res.missing).toEqual([]);
    expect(res.corrupt).toEqual([]);
    expect(files.has(`${DIR}/project.glucose`)).toBe(true);
    expect(files.has(`${DIR}/bundle.json`)).toBe(true);
    for (const n of names) expect(files.has(`${DIR}/objects/${n}`)).toBe(true);

    const m = JSON.parse(new TextDecoder().decode(files.get(`${DIR}/bundle.json`)!));
    expect(m.format).toBe("glucose-bundle");
    expect(m.assets.map((a: { name: string }) => a.name).sort()).toEqual([...names].sort());
  });

  it("signale les assets introuvables (sans échouer) et écrit quand même le doc", async () => {
    const { doc, names } = await mkDocWithAssets();
    store.delete(names[0]); // un asset a disparu du magasin

    const res = await exportBundle(doc, DIR);
    expect(res.included).toBe(1);
    expect(res.missing).toEqual([names[0]]);
    expect(files.has(`${DIR}/project.glucose`)).toBe(true); // export complété
    expect(files.has(`${DIR}/objects/${names[0]}`)).toBe(false);
  });
});

describe("export → import (roundtrip portable)", () => {
  it("ré-hydrate les assets sur une machine au magasin VIDE + doc rechargeable à l'identique", async () => {
    const { doc, names } = await mkDocWithAssets();
    await exportBundle(doc, DIR);

    store.clear(); // simule un autre PC : le magasin global est vide

    const res = await importBundle(DIR);
    expect(res.rehydrated).toBe(2);
    expect(res.missing).toEqual([]);
    // Les octets sont revenus dans le magasin global, sous les MÊMES noms
    // (save_asset recalcule le même hash content-addressed).
    for (const n of names) expect(store.has(n)).toBe(true);

    // Le doc du bundle se recharge exactement sur l'état d'origine.
    const reloaded = A.loadResilient<Project>(files.get(res.docPath)!).doc;
    expect(A.asPlain(reloaded)).toEqual(A.asPlain(doc));
  });

  it("importBundle rejette un dossier qui n'est pas un bundle Glucose", async () => {
    files.set("/mem/autre/bundle.json", new TextEncoder().encode(JSON.stringify({ format: "autre-chose" })));
    await expect(importBundle("/mem/autre")).rejects.toBeInstanceOf(BundleError);
  });
});
