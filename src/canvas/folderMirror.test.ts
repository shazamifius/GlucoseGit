// ────────────────────────────────────────────────────────────────────────────
// R-FIL-02 v2 — Tests du scan récursif (arbre) + action createFolderTree.
//
// Le scan appelle `scan_tree` (Tauri) qu'on stubbe (vi.mock). On vérifie :
//   - Fichiers → stickies launchers (sourceFile + icône via EXT_COLOR).
//   - Sous-dossiers → FolderTreeNode enfants NAVIGABLES (pas des stickies).
//   - Tri R-FIL-03 (name-asc défaut, size-desc, dossiers d'abord).
//   - createFolderTree : 1 child board par dossier, imbrication, undo atomique.
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it, vi } from "vitest";
import { isStickyAnnotation } from "../types";
import { useGlucoseStore } from "../store";
import { nanoid } from "../utils/nanoid";

// ── Mock Tauri invoke ─────────────────────────────────────────────────────
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  convertFileSrc: (s: string) => s,
}));

import { invoke } from "@tauri-apps/api/core";
import { scanFolderForMirror } from "./folderMirror";
import type { FolderTreeNode } from "../types";

interface DirNode {
  path: string;
  name: string;
  is_dir: boolean;
  ext: string;
  size: number;
  modified: number;
  children: DirNode[];
}

function file(name: string, opts: Partial<DirNode> = {}): DirNode {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  return { path: `C:/w/${name}`, name, is_dir: false, ext, size: 1024, modified: 0, children: [], ...opts };
}
function dir(name: string, children: DirNode[] = [], opts: Partial<DirNode> = {}): DirNode {
  return { path: `C:/w/${name}`, name, is_dir: true, ext: "", size: 0, modified: 0, children, ...opts };
}
function root(children: DirNode[], path = "C:/w"): DirNode {
  return { path, name: path.split("/").pop() || path, is_dir: true, ext: "", size: 0, modified: 0, children };
}

beforeEach(() => {
  vi.mocked(invoke).mockReset();
});

describe("scanFolderForMirror — arbre + layout", () => {
  it("9 fichiers → 9 stickies + folder dimensionné, 0 enfant", async () => {
    const files = Array.from({ length: 9 }, (_, i) => file(`f${i}.png`));
    vi.mocked(invoke).mockResolvedValueOnce(root(files));

    const result = await scanFolderForMirror("C:/w", 100, 200);
    expect(result.tree.annotations.length).toBe(9);
    expect(result.tree.children.length).toBe(0);
    expect(result.tree.folder.x).toBe(100);
    expect(result.tree.folder.y).toBe(200);
    expect(result.tree.folder.width).toBeGreaterThanOrEqual(3 * 220 + 160);
    expect(result.tree.folder.mirrorSource?.rootPath).toBe("C:/w");
    expect(result.tree.folder.mirrorSource?.recursive).toBe(true);
    expect(result.totalEntries).toBe(9);
  });

  it("sous-dossier → FolderTreeNode enfant navigable (pas un sticky)", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(
      root([dir("sub", [file("a.txt"), file("b.txt")])]),
    );
    const result = await scanFolderForMirror("C:/w", 0, 0);
    expect(result.tree.annotations.length).toBe(0);
    expect(result.tree.children.length).toBe(1);
    const child: FolderTreeNode = result.tree.children[0];
    expect(child.folder.name).toBe("sub");
    expect(child.folder.mirrorSource?.rootPath).toBe("C:/w/sub");
    expect(child.annotations.length).toBe(2);
    expect(result.totalEntries).toBe(3);
  });

  it("fichier → sticky launcher avec sourceFile", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(root([file("scene.blend", { size: 1e6 })]));
    const result = await scanFolderForMirror("C:/w", 0, 0);
    const ann = result.tree.annotations[0];
    expect(isStickyAnnotation(ann)).toBe(true);
    if (isStickyAnnotation(ann)) {
      expect(ann.sourceFile).toBe("C:/w/scene.blend");
      expect(ann.bgColor).toBe("#e87d0d"); // EXT_COLOR.blend
    }
  });

  it("imbrication profonde (a/b/c) → 3 niveaux d'enfants", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(
      root([dir("a", [dir("b", [dir("c", [file("deep.txt")])])])]),
    );
    const result = await scanFolderForMirror("C:/w", 0, 0);
    const a = result.tree.children[0];
    const b = a.children[0];
    const c = b.children[0];
    expect(a.folder.name).toBe("a");
    expect(b.folder.name).toBe("b");
    expect(c.folder.name).toBe("c");
    expect(c.annotations.length).toBe(1);
  });

  it("nom du folder racine = dernier segment du path", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(root([], "C:/Users/admin/projects/glucose"));
    const result = await scanFolderForMirror("C:/Users/admin/projects/glucose", 0, 0);
    expect(result.tree.folder.name).toBe("glucose");
  });
});

describe("scanFolderForMirror — tri R-FIL-03", () => {
  it("dossiers toujours avant les fichiers (façon Windows)", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(
      root([file("z.txt"), dir("aaa"), file("a.txt")]),
    );
    const result = await scanFolderForMirror("C:/w", 0, 0, "name-asc");
    expect(result.tree.children.length).toBe(1);
    expect(result.tree.children[0].folder.x).toBe(80); // PADDING, première cellule
    const names = result.tree.annotations.map((a) => (isStickyAnnotation(a) ? a.text : ""));
    expect(names).toEqual(["a.txt", "z.txt"]);
  });

  it("size-desc → plus gros fichier en premier", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(
      root([file("small.bin", { size: 10 }), file("big.bin", { size: 9999 })]),
    );
    const result = await scanFolderForMirror("C:/w", 0, 0, "size-desc");
    const names = result.tree.annotations.map((a) => (isStickyAnnotation(a) ? a.text : ""));
    expect(names).toEqual(["big.bin", "small.bin"]);
  });
});

describe("createFolderTree — action store", () => {
  function loadRoot() {
    useGlucoseStore.getState().loadProject({
      version: "2.0.0", name: "test",
      boards: [{
        id: "root", name: "root", images: [], annotations: [],
        panels: [], zones: [], folders: [],
        viewport: { x: 0, y: 0, scale: 1 }, createdAt: 0, updatedAt: 0,
      }],
      activeBoardId: "root", presets: [], domains: [],
      createdAt: 0, updatedAt: 0,
    });
  }

  it("arbre plat → folder + child board peuplé", () => {
    loadRoot();
    const tree: FolderTreeNode = {
      folder: { name: "M", color: "#abc", x: 10, y: 20, width: 400, height: 300 },
      annotations: [
        { id: nanoid(), type: "sticky", x: 0, y: 0, text: "A", bgColor: "#fff", width: 100, height: 100 },
        { id: nanoid(), type: "sticky", x: 100, y: 0, text: "B", bgColor: "#fff", width: 100, height: 100 },
      ],
      children: [],
    };
    const folderId = useGlucoseStore.getState().createFolderTree("root", tree);
    expect(folderId).toMatch(/^[A-Za-z0-9_-]{8,}$/);

    const project = useGlucoseStore.getState().project;
    const parent = project.boards.find((b) => b.id === "root");
    expect(parent?.folders?.length).toBe(1);
    const folder = parent?.folders?.[0];
    expect(folder?.id).toBe(folderId);
    const child = project.boards.find((b) => b.id === folder?.childBoardId);
    expect(child?.annotations.length).toBe(2);
  });

  it("arbre imbriqué → 1 child board par dossier", () => {
    loadRoot();
    const leaf: FolderTreeNode = {
      folder: { name: "sub", color: "#abc", x: 0, y: 0, width: 400, height: 300 },
      annotations: [
        { id: nanoid(), type: "sticky", x: 0, y: 0, text: "leaf", bgColor: "#fff", width: 100, height: 100 },
      ],
      children: [],
    };
    const tree: FolderTreeNode = {
      folder: { name: "M", color: "#abc", x: 0, y: 0, width: 400, height: 300 },
      annotations: [],
      children: [leaf],
    };
    useGlucoseStore.getState().createFolderTree("root", tree);

    const project = useGlucoseStore.getState().project;
    expect(project.boards.length).toBe(3); // root + M + sub
    const m = project.boards.find((b) => b.id === "root")?.folders?.[0];
    const mBoard = project.boards.find((b) => b.id === m?.childBoardId);
    expect(mBoard?.folders?.length).toBe(1);
    const sub = mBoard?.folders?.[0];
    const subBoard = project.boards.find((b) => b.id === sub?.childBoardId);
    expect(subBoard?.annotations.length).toBe(1);
  });

  it("undo annule tout l'arbre (folder + child boards)", () => {
    loadRoot();
    const tree: FolderTreeNode = {
      folder: { name: "M", color: "#abc", x: 0, y: 0, width: 400, height: 300 },
      annotations: [],
      children: [{
        folder: { name: "sub", color: "#abc", x: 0, y: 0, width: 400, height: 300 },
        annotations: [{ id: nanoid(), type: "sticky", x: 0, y: 0, text: "x", bgColor: "#fff", width: 100, height: 100 }],
        children: [],
      }],
    };
    useGlucoseStore.getState().createFolderTree("root", tree);
    expect(useGlucoseStore.getState().project.boards.length).toBe(3);

    useGlucoseStore.getState().undo();
    expect(useGlucoseStore.getState().project.boards.length).toBe(1);
    expect(useGlucoseStore.getState().project.boards[0].folders?.length ?? 0).toBe(0);
  });
});
