// ────────────────────────────────────────────────────────────────────────────
// R-FIL-02 — Tests du layout de scan + action createFolderWithContent.
//
// Le scan lui-même appelle un invoke Tauri qu'on stubbe (vi.mock). Ce qu'on
// vérifie ici :
//   - Le layout grille calcule cols/rows + dimensions correctes.
//   - Les fichiers texte/code passent en sticky bleuté (lecture inline reportée).
//   - Les sous-dossiers passent en sticky 📁 (sous-folder réel en R-FIL-02 v2).
//   - Les autres binaires passent en launcher classique (makeSourceSticky).
//   - mirrorSource est correctement rempli.
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

interface DirEntryDto {
  path: string;
  name: string;
  size: number;
  is_dir: boolean;
  ext: string;
}

beforeEach(() => {
  vi.mocked(invoke).mockReset();
});

describe("scanFolderForMirror — layout en grille", () => {
  it("9 fichiers → grille 3×3 + folder dimensionné", async () => {
    const entries: DirEntryDto[] = Array.from({ length: 9 }, (_, i) => ({
      path: `C:/work/file${i}.png`,
      name: `file${i}.png`,
      size: 1024,
      is_dir: false,
      ext: "png",
    }));
    vi.mocked(invoke).mockResolvedValueOnce(entries);

    const result = await scanFolderForMirror("C:/work", 100, 200);
    expect(result.annotations.length).toBe(9);
    expect(result.folder.x).toBe(100);
    expect(result.folder.y).toBe(200);
    // grille 3x3 + padding 80
    expect(result.folder.width).toBeGreaterThanOrEqual(3 * 220 + 160);
    expect(result.folder.height).toBeGreaterThanOrEqual(3 * 220 + 160);
    expect(result.folder.mirrorSource?.rootPath).toBe("C:/work");
    expect(result.folder.mirrorSource?.mode).toBe("snapshot");
  });

  it("sous-dossiers → sticky 📁 + couleur dédiée", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      { path: "C:/work/sub", name: "sub", size: 0, is_dir: true, ext: "" },
    ] as DirEntryDto[]);
    const result = await scanFolderForMirror("C:/work", 0, 0);
    expect(result.annotations.length).toBe(1);
    const ann = result.annotations[0];
    expect(isStickyAnnotation(ann)).toBe(true);
    if (isStickyAnnotation(ann)) {
      expect(ann.text).toBe("📁 sub");
      expect(ann.bgColor).toBe("#3a3a4a");
    }
  });

  it(".md/.json/.ts → sticky bleuté (text-readable)", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      { path: "C:/w/a.md",   name: "a.md",   size: 100, is_dir: false, ext: "md" },
      { path: "C:/w/b.json", name: "b.json", size: 100, is_dir: false, ext: "json" },
      { path: "C:/w/c.ts",   name: "c.ts",   size: 100, is_dir: false, ext: "ts" },
    ] as DirEntryDto[]);
    const result = await scanFolderForMirror("C:/w", 0, 0);
    expect(result.annotations.length).toBe(3);
    for (const ann of result.annotations) {
      expect(isStickyAnnotation(ann)).toBe(true);
      if (isStickyAnnotation(ann)) expect(ann.bgColor).toBe("#2a3a4a");
    }
  });

  it(".blend/.psd → launcher classique (EXT_COLOR)", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      { path: "C:/scene.blend", name: "scene.blend", size: 1e6, is_dir: false, ext: "blend" },
      { path: "C:/photo.psd",   name: "photo.psd",   size: 5e6, is_dir: false, ext: "psd" },
    ] as DirEntryDto[]);
    const result = await scanFolderForMirror("C:/", 0, 0);
    const ann1 = result.annotations[0];
    const ann2 = result.annotations[1];
    if (isStickyAnnotation(ann1)) expect(ann1.bgColor).toBe("#e87d0d"); // EXT_COLOR.blend
    if (isStickyAnnotation(ann2)) expect(ann2.bgColor).toBe("#31a8ff"); // EXT_COLOR.psd
  });

  it("dossier vide → 0 annotations + folder minimum", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([] as DirEntryDto[]);
    const result = await scanFolderForMirror("C:/empty", 0, 0);
    expect(result.annotations.length).toBe(0);
    expect(result.folder.width).toBeGreaterThanOrEqual(400);
    expect(result.folder.height).toBeGreaterThanOrEqual(300);
  });

  it("nom du folder = dernier segment du path", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([] as DirEntryDto[]);
    const result = await scanFolderForMirror("C:/Users/admin/projects/glucose", 0, 0);
    expect(result.folder.name).toBe("glucose");
  });
});

describe("createFolderWithContent — action store", () => {
  it("crée un folder + child board peuplé + folderId retourné", () => {
    const childId = `c-${nanoid()}`;
    void childId; // sanity unused
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

    const folderId = useGlucoseStore.getState().createFolderWithContent(
      "root",
      {
        name: "MyMirror",
        color: "#abcdef",
        x: 100, y: 200,
        width: 500, height: 400,
      },
      [
        { id: nanoid(), type: "sticky", x: 0, y: 0, text: "A",
          bgColor: "#fff", width: 100, height: 100 },
        { id: nanoid(), type: "sticky", x: 100, y: 0, text: "B",
          bgColor: "#fff", width: 100, height: 100 },
      ],
    );

    expect(folderId).toMatch(/^[A-Za-z0-9_-]{8,}$/);

    const project = useGlucoseStore.getState().project;
    const parent = project.boards.find(b => b.id === "root");
    expect(parent?.folders?.length).toBe(1);
    const folder = parent?.folders?.[0];
    expect(folder?.id).toBe(folderId);
    expect(folder?.name).toBe("MyMirror");

    // Le child board doit exister et contenir 2 annotations
    const child = project.boards.find(b => b.id === folder?.childBoardId);
    expect(child).toBeDefined();
    expect(child?.annotations.length).toBe(2);
    expect(child?.annotations[0].type).toBe("sticky");
    if (child?.annotations[0].type === "sticky") {
      expect(child.annotations[0].text).toBe("A");
    }
  });

  it("undo annule l'ensemble folder + child board", () => {
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

    useGlucoseStore.getState().createFolderWithContent("root",
      { name: "M", color: "#fff", x: 0, y: 0, width: 300, height: 300 },
      [{ id: nanoid(), type: "sticky", x: 0, y: 0, text: "x",
         bgColor: "#fff", width: 100, height: 100 }],
    );
    expect(useGlucoseStore.getState().project.boards.length).toBe(2); // root + child

    useGlucoseStore.getState().undo();
    expect(useGlucoseStore.getState().project.boards.length).toBe(1); // child purgé
    expect(useGlucoseStore.getState().project.boards[0].folders?.length ?? 0).toBe(0);
  });
});
