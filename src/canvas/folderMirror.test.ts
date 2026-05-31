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
  text?: string | null;
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
  it("9 images → 9 vignettes (images), 0 sticky, boîte compacte", async () => {
    const files = Array.from({ length: 9 }, (_, i) => file(`f${i}.png`));
    vi.mocked(invoke).mockResolvedValueOnce(root(files));

    const result = await scanFolderForMirror("C:/w", 100, 200);
    // .png → vignettes images (pas des stickies launchers).
    expect(result.tree.images.length).toBe(9);
    expect(result.tree.annotations.length).toBe(0);
    expect(result.tree.children.length).toBe(0);
    expect(result.tree.folder.x).toBe(100);
    expect(result.tree.folder.y).toBe(200);
    // Boîte COMPACTE (contenu dans le child board) — pas taillée au contenu.
    expect(result.tree.folder.width).toBeLessThanOrEqual(220);
    expect(result.tree.folder.width).toBeGreaterThan(0);
    expect(result.tree.folder.mirrorSource?.rootPath).toBe("C:/w");
    expect(result.tree.folder.mirrorSource?.recursive).toBe(true);
    expect(result.totalEntries).toBe(9);
  });

  it("dispatch par type : image→images, vidéo→images(isVideo), texte→annotation, binaire→launcher", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(root([
      file("a.png"),
      file("b.mp4"),
      file("c.md", { text: "# Titre\n$x^2$" }),
      file("d.blend"),
    ]));
    const result = await scanFolderForMirror("C:/w", 0, 0);
    expect(result.tree.images.length).toBe(2);          // png + mp4
    expect(result.tree.images.some((im) => im.isVideo)).toBe(true);
    // c.md (texte lu) → annotation texte ; d.blend → launcher sticky
    expect(result.tree.annotations.length).toBe(2);
    const texts = result.tree.annotations.map((a) => a.type);
    expect(texts).toContain("text");
    expect(texts).toContain("sticky");
  });

  it("scan PARESSEUX : sous-dossier → boîte pendingScan VIDE (scannée à l'entrée)", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(
      root([dir("sub", [file("a.txt"), file("b.txt")])]),
    );
    const result = await scanFolderForMirror("C:/w", 0, 0);
    expect(result.tree.annotations.length).toBe(0);
    expect(result.tree.children.length).toBe(1);
    const child: FolderTreeNode = result.tree.children[0];
    expect(child.folder.name).toBe("sub");
    expect(child.folder.mirrorSource?.rootPath).toBe("C:/w/sub");
    // Le contenu du sous-dossier n'est PAS encore scanné (pendingScan).
    expect(child.folder.mirrorSource?.pendingScan).toBe(true);
    expect(child.annotations.length).toBe(0);
    expect(child.children.length).toBe(0);
    // Le folder racine, lui, EST scanné.
    expect(result.tree.folder.mirrorSource?.pendingScan).toBe(false);
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

  it("scan PARESSEUX : un seul niveau (les petits-enfants ne sont PAS scannés)", async () => {
    // Même si le mock renvoie b/c imbriqués, scanFolderForMirror n'en garde
    // que le 1er niveau (a) en pendingScan ; b/c seront scannés en entrant.
    vi.mocked(invoke).mockResolvedValueOnce(
      root([dir("a", [dir("b", [dir("c", [file("deep.txt")])])])]),
    );
    const result = await scanFolderForMirror("C:/w", 0, 0);
    expect(result.tree.children.length).toBe(1);
    const a = result.tree.children[0];
    expect(a.folder.name).toBe("a");
    expect(a.folder.mirrorSource?.pendingScan).toBe(true);
    expect(a.children.length).toBe(0); // pas de b/c — paresseux
  });

  it("nom du folder racine = dernier segment du path", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(root([], "C:/Users/admin/projects/glucose"));
    const result = await scanFolderForMirror("C:/Users/admin/projects/glucose", 0, 0);
    expect(result.tree.folder.name).toBe("glucose");
  });
});

describe("scanFolderForMirror — disposition en croix (LAYOUT-1)", () => {
  it("apps à GAUCHE, sous-dossiers au CENTRE, médias à DROITE, textes en BAS", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(root([
      file("scene.blend"),                       // app (launcher)  → gauche
      file("photo.png"),                         // média           → droite
      file("notes.md", { text: "# notes" }),     // texte           → bas
      dir("sub"),                                // sous-dossier    → centre
    ]));
    const result = await scanFolderForMirror("C:/w", 0, 0);
    const app = result.tree.annotations.find((a) => isStickyAnnotation(a) && a.sourceFile?.endsWith(".blend"));
    const media = result.tree.images[0];
    const text = result.tree.annotations.find((a) => a.type === "text");
    const folder = result.tree.children[0]?.folder; // dossier central
    expect(app).toBeDefined();
    expect(media).toBeDefined();
    expect(text).toBeDefined();
    expect(folder).toBeDefined();
    // Apps à gauche du centre, médias à droite du centre.
    expect(app!.x).toBeLessThan(folder!.x);
    expect(media!.x).toBeGreaterThan(folder!.x);
    // Apps strictement à gauche des médias.
    expect(app!.x).toBeLessThan(media!.x);
    // Textes EN DESSOUS du centre.
    expect(text!.y).toBeGreaterThan(folder!.y);
  });

  it("médias centrés sur leur cellule (alignés, pas dans un coin) + grille CELL", async () => {
    // 2 médias adjacents : sprites ancrés au centre → espacés d'exactement CELL,
    // même y, dimensions de tuile préservées.
    vi.mocked(invoke).mockResolvedValueOnce(root([file("a.png"), file("b.png")]));
    const result = await scanFolderForMirror("C:/w", 0, 0);
    const [m0, m1] = result.tree.images;
    expect(result.tree.images.length).toBe(2);
    expect(Math.abs(m1.x - m0.x)).toBe(220);   // CELL
    expect(m0.y).toBe(m1.y);                    // même rangée
    expect(m0.width).toBe(190);
    expect(m0.height).toBe(150);
    expect(m0.fit).toBe("contain");             // ratio préservé
  });

  it("tuiles texte de folder : taille VARIABLE bornée + sourceFile pour ouvrir", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(root([
      file("a.md", { text: "ligne\n".repeat(500) }), // contenu énorme
    ]));
    const result = await scanFolderForMirror("C:/w", 0, 0);
    const text = result.tree.annotations.find((a) => a.type === "text");
    expect(text).toBeDefined();
    // Dimensions VARIABLES mais BORNÉES (clippées) → plus de chevauchement, plus
    // de boîtes de milliers de px. Un fichier énorme est capé à la hauteur max.
    expect(text!.width).toBeGreaterThanOrEqual(200);
    expect(text!.width).toBeLessThanOrEqual(360);
    expect(text!.height).toBe(300); // 500 lignes → capé à TEXT_MAX_H
    // sourceFile présent → double-clic ouvre le fichier natif.
    expect((text as { sourceFile?: string }).sourceFile).toBe("C:/w/a.md");
  });

  it("tuiles texte : un petit fichier a une tuile plus PETITE qu'un gros", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(root([
      file("small.txt", { text: "hi" }),
      file("big.txt", { text: "x".repeat(80) + "\n".repeat(40) }),
    ]));
    const result = await scanFolderForMirror("C:/w", 0, 0);
    const texts = result.tree.annotations.filter((a) => a.type === "text");
    const small = texts.find((a) => (a as { sourceFile?: string }).sourceFile?.endsWith("small.txt"));
    const big = texts.find((a) => (a as { sourceFile?: string }).sourceFile?.endsWith("big.txt"));
    expect(small).toBeDefined();
    expect(big).toBeDefined();
    // Le gros (lignes longues + nombreuses) est plus large ET plus haut.
    expect(big!.width).toBeGreaterThan(small!.width!);
    expect(big!.height).toBeGreaterThan(small!.height!);
  });
});

describe("scanFolderForMirror — tri R-FIL-03", () => {
  it("dossiers toujours avant les fichiers (façon Windows)", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(
      root([file("z.txt"), dir("aaa"), file("a.txt")]),
    );
    const result = await scanFolderForMirror("C:/w", 0, 0, "name-asc");
    expect(result.tree.children.length).toBe(1); // "aaa" → dossier central
    // Le tri reste « dossiers d'abord » puis fichiers par nom (position = croix).
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
      images: [],
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
      images: [],
      children: [],
    };
    const tree: FolderTreeNode = {
      folder: { name: "M", color: "#abc", x: 0, y: 0, width: 400, height: 300 },
      annotations: [],
      images: [],
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
      images: [],
      children: [{
        folder: { name: "sub", color: "#abc", x: 0, y: 0, width: 400, height: 300 },
        annotations: [{ id: nanoid(), type: "sticky", x: 0, y: 0, text: "x", bgColor: "#fff", width: 100, height: 100 }],
        images: [],
        children: [],
      }],
    };
    useGlucoseStore.getState().createFolderTree("root", tree);
    expect(useGlucoseStore.getState().project.boards.length).toBe(3);

    useGlucoseStore.getState().undo();
    expect(useGlucoseStore.getState().project.boards.length).toBe(1);
    expect(useGlucoseStore.getState().project.boards[0].folders?.length ?? 0).toBe(0);
  });

  it("expandFolder remplit une boîte pendingScan + la marque scannée", () => {
    loadRoot();
    // 1) Crée un folder pendingScan vide (comme un sous-dossier lazy).
    const pendingTree: FolderTreeNode = {
      folder: {
        name: "lazy", color: "#abc", x: 0, y: 0, width: 200, height: 168,
        mirrorSource: { rootPath: "C:/w/lazy", mode: "snapshot", lastScannedAt: 0, recursive: true, pendingScan: true },
      },
      annotations: [], images: [], children: [],
    };
    const folderId = useGlucoseStore.getState().createFolderTree("root", pendingTree);

    // 2) Niveau scanné (2 fichiers + 1 sous-dossier pending).
    const level: FolderTreeNode = {
      folder: { name: "lazy", color: "#abc", x: 0, y: 0, width: 200, height: 168 },
      annotations: [{ id: nanoid(), type: "sticky", x: 0, y: 0, text: "f1", bgColor: "#fff", width: 100, height: 100, sourceFile: "C:/w/lazy/f1.blend" }],
      images: [],
      children: [{
        folder: {
          name: "deep", color: "#abc", x: 0, y: 0, width: 200, height: 168,
          mirrorSource: { rootPath: "C:/w/lazy/deep", mode: "snapshot", lastScannedAt: 0, recursive: true, pendingScan: true },
        },
        annotations: [], images: [], children: [],
      }],
    };
    useGlucoseStore.getState().expandFolder("root", folderId, level);

    const project = useGlucoseStore.getState().project;
    const folder = project.boards.find((b) => b.id === "root")?.folders?.[0];
    expect(folder?.mirrorSource?.pendingScan).toBe(false); // marqué scanné
    const childBoard = project.boards.find((b) => b.id === folder?.childBoardId);
    expect(childBoard?.annotations.length).toBe(1);          // f1
    expect(childBoard?.folders?.length).toBe(1);             // sous-dossier "deep"
    expect(childBoard?.folders?.[0].mirrorSource?.pendingScan).toBe(true); // encore pending
  });
});
