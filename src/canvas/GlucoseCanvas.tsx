import { useEffect, useRef, useCallback, useState } from "react";
import { Application, Assets, Container, Sprite, Texture, Graphics, FederatedPointerEvent } from "pixi.js";

// Sprite augmenté pour conserver la référence du contour de sélection
// blanc dessiné autour de l'image quand elle est sélectionnée. Pixi ne
// typant pas les propriétés ad-hoc, on définit une interface locale
// plutôt que d'utiliser `as any`.
interface SpriteWithSelGfx extends Sprite {
  _selGfx?: Graphics | null;
}
import SvgAnnotationLayer, { measureTextSize } from "./SvgAnnotationLayer";
import HtmlAnnotationLayer from "./HtmlAnnotationLayer";
import ArrowSvgLayer from "./ArrowSvgLayer";
import FolderSvgLayer from "./FolderSvgLayer";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGlucoseStore, getActiveBoard } from "../store";
import { addImagesFromDrop, addPathsFromNativeDrop, VIDEO_FILE_EXTS, VIDEO_URL_RE } from "./dropHandler";
import { scanFolderForMirror } from "./folderMirror";
import { ZoneRenderer } from "./ZoneRenderer";
import { SpatialHash } from "./Quadtree";
import { StoryboardLayer } from "./StoryboardLayer";
import { MembraneRenderer } from "./MembraneRenderer";
import FolderBreadcrumb from "../components/FolderBreadcrumb";
import { nanoid } from "../utils/nanoid";
import { Annotation } from "../types";
import ArrowOptions from "../components/ArrowOptions";
import ArrowTextEditor from "../components/ArrowTextEditor";
import ArrowDescriptionPanel from "../components/ArrowDescriptionPanel";
import Minimap from "../components/Minimap";
import ColorPicker from "../components/ColorPicker";
import { exportCanvasPng } from "../utils/exportPng";
import { getPanDelta, setWrapBounds, startCursorGrab, stopCursorGrab } from "../utils/cursorWrap";
import SyntaxEditor from "../components/SyntaxEditor";
import ZoneSelectorOverlay from "./ZoneSelectorOverlay";
import { nodeMatchesTemporalFilter } from "../utils/timeline";
// Phase 7.0 — résolution des `asset:<filename>` vers une URL Tauri utilisable
// R-EMB-01 (Sprint 2) — résolveur unifié AssetRef / src legacy
import { resolveImageSrc } from "../utils/assets";

const MIN_SCALE = 0.02;
const MAX_SCALE = 20;
const DOT_GRID_SIZE = 60;
// const SNAP_DIST = 80;

// Données du ghost de preview layout (preset ou storyboard)
interface GhostData {
  type: "preset" | "storyboard";
  locked?: boolean;
  // preset
  slots?: Array<{ id: string; name: string; color: string; description?: string }>;
  presetId?: string;
  // storyboard
  cols?: number;
  ratio?: number;
  count?: number;
  panelWidth?: number;
  gap?: number;
  aspectRatio?: string;
}

// Données de l'overlay d'édition en place (textarea transparent sur le canvas)
interface EditOverlay {
  annId: string;
  type: "text" | "sticky";
  screenX: number;   // position:fixed left — coin supérieur gauche de l'annotation
  screenY: number;   // position:fixed top
  scale: number;   // world.scale.x (pour la taille de police)
  width?: number;   // largeur en pixels écran (sticky)
  height?: number;   // hauteur en pixels écran (sticky)
  fontSize: number;
  color: string;
  bgColor?: string;
  cursorPos?: number;
}

export default function GlucoseCanvas() {
  const canvasRef = useRef<HTMLDivElement>(null);
  const appRef = useRef<Application | null>(null);
  const worldRef = useRef<Container | null>(null);
  const gridOverlayRef = useRef<HTMLDivElement>(null);
  const vpRef = useRef({ x: 0, y: 0, scale: 1 });
  const selRectGfxRef = useRef<Graphics | null>(null);
  const zoneRendererRef = useRef<ZoneRenderer | null>(null);
  const sbLayerRef = useRef<StoryboardLayer | null>(null);
  const membraneRendererRef = useRef<MembraneRenderer | null>(null);
  const spritesRef = useRef<Map<string, Sprite>>(new Map());
  const pendingLoadsRef = useRef<Set<string>>(new Set());
  const spatialHashRef = useRef(new SpatialHash());
  const zoneGfxRef = useRef<Graphics | null>(null);
  const zoneStartRef = useRef<{ sx: number; sy: number } | null>(null);
  const zonePendingActionRef = useRef<"folder" | "membrane" | null>(null);
  const zoneLabelRef = useRef<HTMLDivElement>(null);
  const isDraggingRef = useRef(false);
  const arrowIdRef = useRef<string | null>(null);
  const jumpAnimRef = useRef<number | null>(null);
  const selDragRef = useRef<{ sx: number; sy: number } | null>(null);
  const draggedSpriteRef = useRef<{
    id: string; startX: number; startY: number; pStartX: number; pStartY: number;
  } | null>(null);
  const textCursorPosRef = useRef<number | undefined>(undefined);
  const lastDomCreateRef = useRef<number>(0);

  const [pixiReady, setPixiReady] = useState(false);
  const [selectedFolderId, setSelectedFolderId] = useState<string | null>(null);
  const [editOverlay, setEditOverlay] = useState<EditOverlay | null>(null);
  const [editText, setEditText] = useState("");
  const [tagInput, setTagInput] = useState("");
  const [showStickyPicker, setShowStickyPicker] = useState(false);
  const [videoLoading, setVideoLoading] = useState(false);

  // const [hoveredArrowTarget, setHoveredArrowTarget] = useState<{ annId: string, blockId?: string } | null>(null);
  const [showGhost, setShowGhost] = useState(false);
  const ghostDataRef = useRef<GhostData | null>(null);  // preview data (no re-render)
  const ghostElRef = useRef<HTMLDivElement>(null);     // DOM ref for cursor-following
  const wrapperRef = useRef<HTMLDivElement>(null);     // outer div ref
  const cursorPosRef = useRef({ x: 0, y: 0 });          // dernière position curseur dans le wrapper

  const [arrowEditorId, setArrowEditorId] = useState<string | null>(null);
  const [zonePendingAction, setZonePendingAction] = useState<"folder" | "membrane" | null>(null);
  const [arrowDescPanel, setArrowDescPanel] = useState<{ arrowId: string; screenX: number; screenY: number } | null>(null);

  useEffect(() => {
    if (editOverlay) {
      // Place le caret en fin de texte n'est plus pertinent car SyntaxEditor s'occupe de l'autoFocus
    }
  }, [editOverlay?.annId]);

  const {
    project, addImage, addAnnotation, updateImage, setViewport,
    setSelectedImageIds, selectedImageIds,
    selectedAnnotationIds, setSelectedAnnotationIds,
    updateAnnotation, removeAnnotations, removeImages,
    applyPresetToBoard,
    createFolder, enterFolder,
    activeTool,
  } = useGlucoseStore();
  const board = getActiveBoard(project);

  const selectedArrow = selectedAnnotationIds.length === 1
    ? board.annotations.find((a): a is import("../types").ArrowAnnotation =>
        a.id === selectedAnnotationIds[0] && a.type === "arrow")
    : undefined;

  // ── Convertit la position monde d'une annotation en coordonnées écran ──
  function annToScreen(ann: Annotation) {
    const world = worldRef.current;
    const canvas = appRef.current?.canvas;
    if (!world || !canvas) return null;
    const rect = canvas.getBoundingClientRect();
    const scale = world.scale.x;
    return {
      screenX: rect.left + world.x + ann.x * scale,
      screenY: rect.top + world.y + ann.y * scale,
      scale,
    };
  }

  function openEditOverlay(ann: Annotation) {
    // L'édition in-place ne concerne que text et sticky.
    if (ann.type !== "text" && ann.type !== "sticky") return;
    const pos = annToScreen(ann);
    if (!pos) return;
    setEditOverlay({
      annId: ann.id,
      type: ann.type,
      screenX: pos.screenX,
      screenY: pos.screenY,
      scale: pos.scale,
      width: ann.width ? ann.width * pos.scale : undefined,
      height: ann.height ? ann.height * pos.scale : undefined,
      fontSize: ann.fontSize ?? (ann.type === "sticky" ? 13 : 14),
      color: ann.color ?? "#ffffff",
      bgColor: ann.type === "sticky" ? ann.bgColor : undefined,
      cursorPos: ann.cursorPos,
    });
    setEditText(ann.text ?? "");
  }

  // ── Init PixiJS ──────────────────────────────────────────────
  useEffect(() => {
    const container = canvasRef.current;
    if (!container || appRef.current) return;
    const app = new Application();
    appRef.current = app;

    app.init({
      backgroundAlpha: 0, resizeTo: container,
      antialias: true, resolution: window.devicePixelRatio || 1,
      autoDensity: true, preference: "webgl",
    }).then(() => {
      if (!canvasRef.current || !appRef.current) return;
      app.canvas.style.cssText = "display:block;width:100%;height:100%;position:absolute;inset:0;";
      container.appendChild(app.canvas);

      const world = new Container();
      app.stage.addChild(world);
      worldRef.current = world;

      const selGfx = new Graphics();
      app.stage.addChild(selGfx);
      selRectGfxRef.current = selGfx;

      const zoneGfx = new Graphics();
      app.stage.addChild(zoneGfx);
      zoneGfxRef.current = zoneGfx;

      membraneRendererRef.current = new MembraneRenderer(world);
      zoneRendererRef.current = new ZoneRenderer(world, () => worldRef.current);
      sbLayerRef.current = new StoryboardLayer(world);

      const saved = getActiveBoard(useGlucoseStore.getState().project).viewport;
      world.x = saved.x || app.screen.width / 2;
      world.y = saved.y || app.screen.height / 2;
      world.scale.set(saved.scale || 1);

      setupEvents(app, world);
      app.renderer.on("resize", (_w: number, _h: number) => {
        const r = app.canvas.getBoundingClientRect();
        setWrapBounds(r.top, r.bottom, r.left, r.right);
      });
      requestAnimationFrame(() => {
        const r = app.canvas.getBoundingClientRect();
        setWrapBounds(r.top, r.bottom, r.left, r.right);
      });
      setPixiReady(true);
    }).catch((err) => {
      if (canvasRef.current)
        canvasRef.current.innerHTML = `<div style="padding:32px;color:#f87171;font-size:13px">Erreur canvas: ${err?.message || err}</div>`;
    });

    return () => {
      // Annule toute animation de transition de dossier en cours (Phase 4.5)
      if (folderTransitionRafRef.current !== null) {
        cancelAnimationFrame(folderTransitionRafRef.current);
        folderTransitionRafRef.current = null;
      }
      const a = appRef.current;
      appRef.current = null; worldRef.current = null;
      selRectGfxRef.current = null;
      zoneRendererRef.current = null;
      membraneRendererRef.current?.destroy(); membraneRendererRef.current = null;
      sbLayerRef.current?.destroy(); sbLayerRef.current = null;
      spritesRef.current.clear(); pendingLoadsRef.current.clear();
      zoneGfxRef.current = null; zoneStartRef.current = null;
      setPixiReady(false);
      // En StrictMode dev, l'unmount peut survenir AVANT que app.init() ait terminé
      // d'installer ses plugins (ResizePlugin etc.). destroy() lance alors
      // "this._cancelResize is not a function". On l'avale, c'est un cas connu.
      try { a?.destroy(true); } catch (err) {
        console.warn("[GlucoseCanvas] PixiJS destroy threw during cleanup", err);
      }
    };
  }, []);

  // ── Sync images ──────────────────────────────────────────────
  useEffect(() => {
    const world = worldRef.current;
    if (!world || !pixiReady) return;
    const currentIds = new Set(board.images.map((img) => img.id));
    spritesRef.current.forEach((sprite, id) => {
      if (!currentIds.has(id)) {
        world.removeChild(sprite); sprite.destroy();
        spritesRef.current.delete(id); pendingLoadsRef.current.delete(id);
      }
    });
    board.images.forEach((img) => {
      const s = spritesRef.current.get(img.id);
      if (s) { s.x = img.x; s.y = img.y; s.width = img.width; s.height = img.height; s.rotation = img.rotation; }
    });
    spatialHashRef.current.build(board.images);
    applyCulling();
    const newImgs = board.images.filter(
      (img) => !spritesRef.current.has(img.id) && !pendingLoadsRef.current.has(img.id)
    );
    newImgs.forEach(async (img) => {
      pendingLoadsRef.current.add(img.id);
      try {
        // R-EMB-01 (Sprint 2) : résolveur unifié — privilégie asset (AssetRef)
        // avec blob URL pour les embeds, fallback sur src legacy.
        const blobs = useGlucoseStore.getState().project.blobs;
        const resolvedSrc = await resolveImageSrc(img.asset, img.src, blobs);
        const tex: Texture = img.isVideo
          ? await Assets.load({ src: resolvedSrc, data: { autoPlay: true, loop: true, muted: true } })
          : await Assets.load(resolvedSrc);
        if (!worldRef.current || spritesRef.current.has(img.id)) return;
        const sprite = new Sprite(tex);
        sprite.anchor.set(0.5);
        sprite.interactive = true; sprite.cursor = "pointer";
        sprite.x = img.x; sprite.y = img.y;
        sprite.width = img.width; sprite.height = img.height;
        sprite.rotation = img.rotation;
        // Phase 6 — applique le dim temporel dès la création (sinon flash plein opacity)
        const tf = useGlucoseStore.getState().temporalFilter;
        sprite.alpha = nodeMatchesTemporalFilter(img.temporalAnchor, tf) ? 1 : 0.12;
        const idx = Math.max(1, worldRef.current.children.length - 2);
        worldRef.current.addChildAt(sprite, idx);
        spritesRef.current.set(img.id, sprite);
        attachSpriteEvents(sprite, img.id);
        applyCulling();
      } catch (err) { console.error("Texture load failed", img.id, err); }
      finally { pendingLoadsRef.current.delete(img.id); }
    });
  }, [board.images, pixiReady]);

  // ── Sync annotations Pixi : RETIRÉ (R-MOD-03) ───────────────
  // Le rendu des annotations est désormais 100 % SVG (flèches) + HTML
  // (texte/sticky), il n'y a plus rien à synchroniser côté Pixi.

  // ── Sync storyboard ──────────────────────────────────────────
  useEffect(() => {
    if (!pixiReady || !sbLayerRef.current) return;
    sbLayerRef.current.sync(
      board.panels, board.storyboard, [],
      (imageId) => board.images.find((img) => img.id === imageId)?.src,
    );
  }, [board.panels, board.storyboard, pixiReady]);

  // ── Folders : rendu géré par FolderSvgLayer (composant React SVG) ──

  // ── Sync membranes ───────────────────────────────────────────
  useEffect(() => {
    if (!pixiReady || !membraneRendererRef.current) return;
    membraneRendererRef.current.update(board.images, project.domains ?? []);
  }, [board.images, project.domains, pixiReady]);

  // ── Phase 6 — Filtrage temporel des sprites images ────────────
  // On dimme (alpha 0.12) les sprites dont le temporalAnchor n'intersecte pas
  // la fenêtre du filtre. Les images atemporelles restent à alpha 1.
  // La culling spatiale (sprite.visible) reste indépendante.
  const temporalFilter = useGlucoseStore(s => s.temporalFilter);
  useEffect(() => {
    if (!pixiReady) return;
    const byId = new Map(board.images.map((img) => [img.id, img]));
    spritesRef.current.forEach((sprite, id) => {
      const img = byId.get(id);
      if (!img) return;
      sprite.alpha = nodeMatchesTemporalFilter(img.temporalAnchor, temporalFilter) ? 1 : 0.12;
    });
  }, [temporalFilter, board.images, pixiReady]);

  // Cancel zone drawing when tool changes away (Échap in App.tsx sets tool to "select")
  useEffect(() => {
    if (activeTool !== "zone-select") {
      zoneStartRef.current = null;
      zonePendingActionRef.current = null;
      zoneGfxRef.current?.clear();
    }
  }, [activeTool]);

  // Phase 4 — Téléportation vers un original quand on clique sur un badge ↻ de miroir
  useEffect(() => {
    function onTeleport(e: Event) {
      const detail = (e as CustomEvent).detail as { mirrorOf: string; type: "annotation" | "folder" | "image" };
      if (!detail || !detail.mirrorOf) return;
      const state = useGlucoseStore.getState();
      // 1) Localiser l'original : board parent + coordonnées x/y
      let targetBoardId: string | null = null;
      let tx = 0, ty = 0;
      for (const b of state.project.boards) {
        if (detail.type === "annotation") {
          const a = b.annotations.find((a) => a.id === detail.mirrorOf);
          if (a) { targetBoardId = b.id; tx = a.x; ty = a.y; break; }
        } else if (detail.type === "image") {
          const i = b.images.find((i) => i.id === detail.mirrorOf);
          if (i) { targetBoardId = b.id; tx = i.x; ty = i.y; break; }
        } else if (detail.type === "folder") {
          const f = (b.folders ?? []).find((f) => f.id === detail.mirrorOf);
          if (f) { targetBoardId = b.id; tx = f.x + f.width / 2; ty = f.y + f.height / 2; break; }
        }
      }
      if (!targetBoardId) return;
      // 2) Si nécessaire, basculer sur ce board (utilise setActiveBoardId — pas enterFolder qui empile le stack)
      if (state.project.activeBoardId !== targetBoardId) {
        useGlucoseStore.getState().setActiveBoardId(targetBoardId);
      }
      // 3) Centrer le viewport sur l'original (animation simple avec rAF)
      const world = worldRef.current;
      const app = appRef.current;
      if (!world || !app) return;
      const targetScale = Math.max(0.6, world.scale.x); // au moins en méso pour identifier
      const sx0 = world.x, sy0 = world.y, ss0 = world.scale.x;
      const sx1 = app.screen.width / 2 - tx * targetScale;
      const sy1 = app.screen.height / 2 - ty * targetScale;
      const t0 = performance.now();
      const DUR = 400;
      const ease = (t: number) => 1 - Math.pow(1 - t, 3);
      function step() {
        const w2 = worldRef.current;
        if (!w2) return;
        const t = Math.min(1, (performance.now() - t0) / DUR);
        const k = ease(t);
        w2.x = sx0 + (sx1 - sx0) * k;
        w2.y = sy0 + (sy1 - sy0) * k;
        w2.scale.set(ss0 + (targetScale - ss0) * k);
        emitViewport(w2);
        if (t < 1) requestAnimationFrame(step);
      }
      requestAnimationFrame(step);
    }
    window.addEventListener("glucose:teleport-to-mirror-original", onTeleport);
    return () => window.removeEventListener("glucose:teleport-to-mirror-original", onTeleport);
  }, []);

  // Phase 5 — Ouverture du panneau de description longue de flèche
  useEffect(() => {
    function onOpen(e: Event) {
      const detail = (e as CustomEvent).detail as { arrowId: string; screenX: number; screenY: number };
      if (!detail) return;
      setArrowDescPanel(detail);
    }
    window.addEventListener("glucose:open-arrow-description", onOpen);
    return () => window.removeEventListener("glucose:open-arrow-description", onOpen);
  }, []);

  // Phase 5 — Portail inter-boards : bascule de board + centre sur la cible
  useEffect(() => {
    function onPortal(e: Event) {
      const detail = (e as CustomEvent).detail as { boardId: string; targetId?: string };
      if (!detail?.boardId) return;
      const state = useGlucoseStore.getState();
      if (state.project.activeBoardId !== detail.boardId) {
        state.setActiveBoardId(detail.boardId);
      }
      // Centrage sur la cible après le tick (le board doit avoir basculé)
      if (detail.targetId) {
        requestAnimationFrame(() => {
          const targetBoard = state.project.boards.find(b => b.id === detail.boardId);
          if (!targetBoard) return;
          let tx = 0, ty = 0, found = false;
          const a = targetBoard.annotations.find(a => a.id === detail.targetId);
          if (a) {
            const aw = a.type === "arrow" ? 80 : (a.width ?? 80);
            const ah = a.type === "arrow" ? 20 : (a.height ?? 20);
            tx = a.x + aw / 2;
            ty = a.y + ah / 2;
            found = true;
          }
          if (!found) {
            const i = targetBoard.images.find(i => i.id === detail.targetId);
            if (i) { tx = i.x; ty = i.y; found = true; }
          }
          if (!found) return;
          const world = worldRef.current; const app = appRef.current;
          if (!world || !app) return;
          world.x = app.screen.width / 2 - tx * world.scale.x;
          world.y = app.screen.height / 2 - ty * world.scale.y;
          emitViewport(world);
        });
      }
    }
    window.addEventListener("glucose:portal-jump", onPortal);
    return () => window.removeEventListener("glucose:portal-jump", onPortal);
  }, []);

  // ── Viewport restoration + animation au changement de board (Phase 4.5) ──
  // Détecte enter/exit via la profondeur du folderStack pour orienter l'effet :
  //   • Enter (folderStack ↑) : "plongée" — scale part à 0.6× du final, grandit
  //   • Exit  (folderStack ↓) : "recul"   — scale part à 1.4× du final, rétrécit
  //   • Bascule via tabs (longueur égale) : pas d'animation (snap)
  const prevBoardIdRef = useRef<string>("");
  const prevFolderStackLenRef = useRef<number>(0);
  const folderTransitionRafRef = useRef<number | null>(null);
  useEffect(() => {
    if (!pixiReady) return;
    const boardId = project.activeBoardId;
    if (!prevBoardIdRef.current) {
      prevBoardIdRef.current = boardId;
      prevFolderStackLenRef.current = useGlucoseStore.getState().folderStack.length;
      return;
    }
    if (prevBoardIdRef.current === boardId) return;
    prevBoardIdRef.current = boardId;

    const world = worldRef.current;
    const app = appRef.current;
    if (!world || !app) return;

    const targetBoard = getActiveBoard(useGlucoseStore.getState().project);
    const saved = targetBoard.viewport;
    const finalX = saved.x || app.screen.width / 2;
    const finalY = saved.y || app.screen.height / 2;
    const finalScale = saved.scale || 1;

    // Annule toute animation en cours
    if (folderTransitionRafRef.current !== null) {
      cancelAnimationFrame(folderTransitionRafRef.current);
      folderTransitionRafRef.current = null;
    }

    const newStackLen = useGlucoseStore.getState().folderStack.length;
    const direction: "enter" | "exit" | "neutral" =
      newStackLen > prevFolderStackLenRef.current ? "enter" :
      newStackLen < prevFolderStackLenRef.current ? "exit" : "neutral";
    prevFolderStackLenRef.current = newStackLen;

    if (direction === "neutral") {
      // Bascule via tabs (board sibling) — snap instantané comme avant
      world.x = finalX; world.y = finalY; world.scale.set(finalScale);
      refreshGrid(world);
      emitViewport(world);
      if (saved.scale === 1) requestAnimationFrame(() => fitView());
      return;
    }

    // Animation 400ms cubic ease-out (rapide, satisfaisante)
    const SCALE_BURST = direction === "enter" ? 0.6 : 1.4;
    const startScale = finalScale * SCALE_BURST;
    // Pour garder le point logique sous le curseur invariant, on ajuste x/y
    // de sorte que l'écart de scale soit centré sur (finalX, finalY) en écran.
    // Ici on centre simplement sur le centre écran : ça suffit pour la perception.
    const startX = app.screen.width / 2 - (app.screen.width / 2 - finalX) * (startScale / finalScale);
    const startY = app.screen.height / 2 - (app.screen.height / 2 - finalY) * (startScale / finalScale);

    world.x = startX; world.y = startY; world.scale.set(startScale);
    refreshGrid(world);
    emitViewport(world);

    const t0 = performance.now();
    const DUR = 400;
    const ease = (t: number) => 1 - Math.pow(1 - t, 3);
    const animate = () => {
      const w = worldRef.current;
      if (!w) { folderTransitionRafRef.current = null; return; }
      const t = Math.min(1, (performance.now() - t0) / DUR);
      const k = ease(t);
      w.x = startX + (finalX - startX) * k;
      w.y = startY + (finalY - startY) * k;
      w.scale.set(startScale + (finalScale - startScale) * k);
      refreshGrid(w);
      emitViewport(w);
      if (t < 1) {
        folderTransitionRafRef.current = requestAnimationFrame(animate);
      } else {
        folderTransitionRafRef.current = null;
        if (saved.scale === 1) requestAnimationFrame(() => fitView());
      }
    };
    folderTransitionRafRef.current = requestAnimationFrame(animate);
  }, [project.activeBoardId, pixiReady]);

  // ── Selection border ─────────────────────────────────────────
  useEffect(() => {
    spritesRef.current.forEach((spriteBase, id) => {
      const sprite = spriteBase as SpriteWithSelGfx;
      const sel = selectedImageIds.includes(id);
      const old = sprite._selGfx;
      if (old) { sprite.removeChild(old); old.destroy(); sprite._selGfx = null; }
      if (!sel) return;
      const g = new Graphics();
      const w = sprite.texture.width; const h = sprite.texture.height;
      g.rect(-w / 2 - 3, -h / 2 - 3, w + 6, h + 6);
      g.stroke({ color: 0xffffff, width: 1.5 / (worldRef.current?.scale.x || 1), alpha: 0.7 });
      sprite.addChild(g);
      sprite._selGfx = g;
    });
  }, [selectedImageIds]);

  // ── Tauri file drop (natif — chemins absolus OS) ─────────────
  // R-FIL (Sprint 2) : depuis dragDropEnabled:true, l'event natif nous donne
  // les vrais chemins absolus. On gère les vidéos ici (besoin convertFileSrc +
  // dimensions), puis on délègue TOUT le reste (dossiers, texte, images,
  // launchers) au routeur `addPathsFromNativeDrop`.
  useEffect(() => {
    const unlisten = listen<{ paths: string[]; position: { x: number; y: number } }>(
      "tauri://drag-drop",
      async (event) => {
        const world = worldRef.current;
        if (!world) return;
        const { paths, position } = event.payload;
        const wx = (position.x - world.x) / world.scale.x;
        const wy = (position.y - world.y) / world.scale.y;
        const boardId = getActiveBoard(useGlucoseStore.getState().project).id;

        // 1) Vidéos locales → lecteur inline (chemin gardé en relatif via asset URL)
        const videos = paths.filter((p) => VIDEO_FILE_EXTS.test(p));
        for (let i = 0; i < videos.length; i++) {
          const path = videos[i];
          const assetUrl = convertFileSrc(path);
          const { width: vw, height: vh } = await getVideoDimensions(assetUrl);
          const s = vw > 800 ? 800 / vw : 1;
          addImage(boardId, {
            id: nanoid(), src: assetUrl, isVideo: true,
            x: wx + i * 24, y: wy + i * 24,
            width: vw * s, height: vh * s,
            rotation: 0, locked: false, tags: [],
            originalWidth: vw, originalHeight: vh,
          });
        }

        // 2) Tout le reste (dossiers, texte/code, images, binaires) → routeur
        const rest = paths.filter((p) => !VIDEO_FILE_EXTS.test(p));
        if (rest.length > 0) {
          await addPathsFromNativeDrop(rest, wx, wy, boardId, {
            addImage,
            addAnnotation,
            createFolderTree: useGlucoseStore.getState().createFolderTree,
          });
        }
      }
    );
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // ── Zones preset ─────────────────────────────────────────────
  useEffect(() => {
    if (!pixiReady || !zoneRendererRef.current) return;
    const allPresets = useGlucoseStore.getState().getAllPresets();
    const preset = board.presetId ? allPresets.find((p) => p.id === board.presetId) ?? null : null;
    zoneRendererRef.current.update(board, preset, (zones) => {
      const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
      useGlucoseStore.getState().setBoardZones(boardId, zones);
    });
  }, [board.presetId, board.zones, pixiReady]);

  // ── Clipboard paste ──────────────────────────────────────────
  useEffect(() => {
    async function onPaste(e: ClipboardEvent) {
      const items = Array.from(e.clipboardData?.items || []);
      const imageItem = items.find((it) => it.type.startsWith("image/"));
      if (imageItem) { const blob = imageItem.getAsFile(); if (blob) { await addBlob(blob); return; } }

      // Video URL (YouTube / TikTok / Instagram / Vimeo)
      const textItem = items.find((it) => it.kind === "string" && it.type === "text/plain");
      if (textItem) {
        const text = await new Promise<string>((res) => textItem.getAsString(res));
        if (VIDEO_URL_RE.test(text)) {
          await importVideoFromUrl(text.trim());
          return;
        }
      }

      try {
        const clips = await navigator.clipboard.read();
        for (const item of clips) {
          const t = item.types.find((x) => x.startsWith("image/"));
          if (t) { await addBlob(await item.getType(t)); return; }
        }
      } catch (_) { }
    }
    async function addBlob(blob: Blob) {
      const src = await new Promise<string>((res) => { const r = new FileReader(); r.onload = () => res(r.result as string); r.readAsDataURL(blob); });
      const { width: w, height: h } = await getImageDimensions(src);
      const world = worldRef.current;
      const cx = world ? (appRef.current!.screen.width / 2 - world.x) / world.scale.x : 0;
      const cy = world ? (appRef.current!.screen.height / 2 - world.y) / world.scale.y : 0;
      const s = w > 600 ? 600 / w : 1;
      useGlucoseStore.getState().addImage(getActiveBoard(useGlucoseStore.getState().project).id, {
        id: nanoid(), src, x: cx, y: cy, width: w * s, height: h * s,
        rotation: 0, locked: false, tags: [], originalWidth: w, originalHeight: h,
      });
      const { showToast } = await import("../components/Toast");
      showToast("Image collée", "📌");
    }
    window.addEventListener("paste", onPaste);
    return () => window.removeEventListener("paste", onPaste);
  }, []);

  // ── Écoute preview layout (preset / storyboard ghost) ──────────────
  useEffect(() => {
    function onPreview(e: Event) {
      const data: GhostData | null = (e as CustomEvent).detail ?? null;
      if (data === null) {
        // Ignore null if currently locked (mouse-leave ne doit pas effacer le mode placement)
        if (ghostDataRef.current?.locked) return;
        ghostDataRef.current = null;
        setShowGhost(false);
      } else {
        ghostDataRef.current = data;
        setShowGhost(true);
      }
    }
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape" && ghostDataRef.current?.locked) {
        ghostDataRef.current = null;
        setShowGhost(false);
      }
      if (e.key === "Backspace") {
        if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
        const state = useGlucoseStore.getState();
        // Exit folder uniquement si rien n'est sélectionné (la suppression prime)
        const hasSelection = state.selectedImageIds.length > 0 || state.selectedAnnotationIds.length > 0;
        if (!hasSelection && state.folderStack.length > 0) {
          e.preventDefault();
          state.exitFolder();
        }
      }
    }
    window.addEventListener("glucose:layout-preview", onPreview);
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("glucose:layout-preview", onPreview);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  // ── Placement capture-phase : intercepte le click avant PixiJS ──────
  useEffect(() => {
    const wrapper = wrapperRef.current;
    if (!wrapper) return;
    function onDown(e: PointerEvent) {
      const data = ghostDataRef.current;
      if (!data?.locked) return;
      if (e.button !== 0) return;
      e.stopPropagation();
      const world = worldRef.current;
      if (!world) return;
      const rect = wrapper!.getBoundingClientRect();
      // wx/wy = position monde du curseur = centre du ghost
      const wx = (e.clientX - rect.left - world.x) / world.scale.x;
      const wy = (e.clientY - rect.top - world.y) / world.scale.y;

      const state = useGlucoseStore.getState();
      const boardId = getActiveBoard(state.project).id;

      if (data.type === "preset" && data.presetId) {
        // Le store centre la grille de zones sur wx/wy
        applyPresetToBoard(boardId, data.presetId, wx, wy);
      }

      if (data.type === "storyboard") {
        const panelW = data.panelWidth ?? 280;
        const cols = data.cols ?? 4;
        const gapVal = data.gap ?? 24;
        const ratio = data.ratio ?? (16 / 9);
        const count = data.count ?? cols * 2;
        const s = {
          aspectRatio: (data.aspectRatio ?? "16:9") as "16:9" | "4:3" | "2.35:1" | "1:1" | "9:16",
          panelWidth: panelW, cols, gap: gapVal,
        };
        state.setStoryboardSettings(boardId, s);

        const h = panelW / Math.max(0.3, ratio);
        const descH = 40;
        const rows = Math.ceil(count / cols);
        // Centrer la grille sur (wx, wy)
        const totalW = cols * panelW + (cols - 1) * gapVal;
        const totalH = rows * (h + descH) + (rows - 1) * gapVal;
        const originX = wx - totalW / 2;
        const originY = wy - totalH / 2;

        const board = getActiveBoard(state.project);
        if (board.panels.length > 0) {
          // Relayouter les panels existants à la nouvelle origine
          [...board.panels].sort((a, b) => a.order - b.order).forEach((p, i) => {
            const col = i % cols;
            const row = Math.floor(i / cols);
            state.updatePanel(boardId, p.id, {
              x: originX + col * (panelW + gapVal),
              y: originY + row * (h + descH + gapVal),
              width: panelW, height: h, order: i,
            });
          });
        } else {
          // Créer les panels initiaux à la position placée
          for (let i = 0; i < count; i++) {
            const col = i % cols;
            const row = Math.floor(i / cols);
            state.addPanel(boardId, {
              id: nanoid(), order: i, description: "",
              x: originX + col * (panelW + gapVal),
              y: originY + row * (h + descH + gapVal),
              width: panelW, height: h,
            });
          }
        }
      }

      ghostDataRef.current = null;
      setShowGhost(false);
    }
    wrapper.addEventListener("pointerdown", onDown, { capture: true });
    return () => wrapper.removeEventListener("pointerdown", onDown, { capture: true });
  }, []);

  // ── Import vidéo depuis URL (YouTube / TikTok / Instagram / Vimeo) ──
  async function importVideoFromUrl(url: string) {
    setVideoLoading(true);
    try {
      const localPath: string = await invoke("download_video", { url });
      const assetUrl = convertFileSrc(localPath);
      const { width: vw, height: vh } = await getVideoDimensions(assetUrl);
      const world = worldRef.current;
      const cx = world ? (appRef.current!.screen.width / 2 - world.x) / world.scale.x : 0;
      const cy = world ? (appRef.current!.screen.height / 2 - world.y) / world.scale.y : 0;
      const s = vw > 800 ? 800 / vw : 1;
      const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
      addImage(boardId, {
        id: nanoid(), src: assetUrl, isVideo: true,
        x: cx, y: cy,
        width: vw * s, height: vh * s,
        rotation: 0, locked: false, tags: [],
        sourceUrl: url,
        originalWidth: vw, originalHeight: vh,
      });
    } catch (err) {
      alert(`Erreur import vidéo:\n${err}`);
    } finally {
      setVideoLoading(false);
    }
  }

  // ── Grille CSS vectorielle (radial-gradient — aucun bitmap) ───
  function updateCssGrid(x: number, y: number, scale: number) {
    const el = gridOverlayRef.current;
    if (!el) return;
    if (scale < 0.07) { el.style.opacity = "0"; return; }
    const gridSize = DOT_GRID_SIZE * scale;
    const dotR = Math.min(2.5, Math.max(0.5, 1.2 * scale));
    const px = ((x % gridSize) + gridSize) % gridSize;
    const py = ((y % gridSize) + gridSize) % gridSize;
    const alpha = Math.min(0.45, Math.max(0.08, scale * 0.3));
    el.style.opacity = "1";
    el.style.backgroundImage = `radial-gradient(circle ${dotR}px at ${dotR}px ${dotR}px, rgba(136,136,136,${alpha}) 0px, transparent ${dotR}px)`;
    el.style.backgroundSize = `${gridSize}px ${gridSize}px`;
    el.style.backgroundPosition = `${px}px ${py}px`;
  }
  function refreshGrid(world: Container) {
    updateCssGrid(world.x, world.y, world.scale.x);
  }
  function applyCulling() {
    const app = appRef.current;
    if (!app) return;
    const vp = vpRef.current;
    const ww = app.screen.width / vp.scale;
    const wh = app.screen.height / vp.scale;
    const wx = -vp.x / vp.scale;
    const wy = -vp.y / vp.scale;
    // 50% margin on each side to pre-show sprites before they enter view
    const margin = Math.max(ww, wh) * 0.5;
    const visible = spatialHashRef.current.queryIds(wx, wy, ww, wh, margin);
    spritesRef.current.forEach((sprite, id) => { sprite.visible = visible.has(id); });
  }

  function emitViewport(world: Container) {
    vpRef.current = { x: world.x, y: world.y, scale: world.scale.x };
    applyCulling();
    updateCssGrid(world.x, world.y, world.scale.x);
    window.dispatchEvent(new CustomEvent("glucose:viewport-changed", {
      detail: { x: world.x, y: world.y, scale: world.scale.x },
    }));
    // Phase 7.5 — navigation auto par zoom (folder enter/exit)
    checkAutoNavigate(world);
    // Met à jour l'échelle du ghost quand l'utilisateur zoome
    const { x: cx, y: cy } = cursorPosRef.current;
    updateGhostPosition(cx, cy, world.scale.x);
  }

  // ── Phase 7.5 — Navigation automatique par zoom ─────────────────
  // Zoomer fortement (scale ≥ ENTER) sur un dossier : on entre.
  // Dézoomer fortement (scale ≤ EXIT) à l'intérieur d'un dossier : on sort.
  // Cooldown 700 ms entre deux transitions pour éviter les ping-pong.
  const lastNavRef = useRef(0);
  const ENTER_SCALE = 3.0;
  const EXIT_SCALE  = 0.4;
  // Cooldown > durée de l'animation de transition (400 ms) pour qu'aucune
  // transition auto ne se déclenche pendant l'arrivée (anti-cascade).
  const NAV_COOLDOWN = 850;

  /** Bounding box de tout le contenu d'un board (images + annotations +
   *  folders). Renvoie null si le board est vide. */
  function contentBounds(board: ReturnType<typeof getActiveBoard>) {
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    const add = (x: number, y: number, w: number, h: number) => {
      minX = Math.min(minX, x); minY = Math.min(minY, y);
      maxX = Math.max(maxX, x + w); maxY = Math.max(maxY, y + h);
    };
    for (const im of board.images) add(im.x, im.y, im.width, im.height);
    for (const f of board.folders ?? []) add(f.x, f.y, f.width, f.height);
    for (const a of board.annotations) {
      if (a.type === "arrow") { add(Math.min(a.x, a.x2 ?? a.x), Math.min(a.y, a.y2 ?? a.y), Math.abs((a.x2 ?? a.x) - a.x), Math.abs((a.y2 ?? a.y) - a.y)); continue; }
      add(a.x, a.y, a.width ?? 160, a.height ?? 80);
    }
    if (minX === Infinity) return null;
    return { minX, minY, w: maxX - minX, h: maxY - minY };
  }

  /** Viewport {x,y,scale} qui cadre TOUT le contenu d'un board, centré, à une
   *  échelle confortable (ni en zone d'entrée, ni d'éjection). Sert à l'arrivée
   *  dans un dossier (entrée OU sortie) → on voit tout d'un coup. */
  function fitViewportFor(board: ReturnType<typeof getActiveBoard>, screenW: number, screenH: number) {
    const b = contentBounds(board);
    if (!b || b.w <= 0 || b.h <= 0) {
      return { x: screenW / 2, y: screenH / 2, scale: 1 };
    }
    const margin = 1.25;
    // Clamp [MIN_SCALE, 1.1] : assez haut pour voir un petit dossier sans être
    // en zone d'entrée (ENTER=3), aussi bas que le moteur le permet pour les
    // dossiers énormes (milliers de fichiers directs).
    const scale = Math.min(1.1, Math.max(MIN_SCALE, Math.min(
      screenW / (b.w * margin),
      screenH / (b.h * margin),
    )));
    const cx = b.minX + b.w / 2;
    const cy = b.minY + b.h / 2;
    return { x: screenW / 2 - cx * scale, y: screenH / 2 - cy * scale, scale };
  }

  /** R-FIL — Scale de sortie ADAPTATIF : on ne quitte le folder que lorsque
   *  TOUT son contenu tient dans l'écran (puis qu'on dézoome encore un peu).
   *  Un gros dossier (1M de fichiers) se laisse explorer sans éjection ;
   *  un petit dossier se quitte vite. Borné à [0.05, 0.6]. */
  function adaptiveExitScale(board: ReturnType<typeof getActiveBoard>, screenW: number, screenH: number): number {
    const b = contentBounds(board);
    if (!b || b.w <= 0 || b.h <= 0) return EXIT_SCALE;
    const margin = 1.15;
    const scaleFit = Math.min(screenW / (b.w * margin), screenH / (b.h * margin));
    // Facteur 0.55 : on garde une marge de zoom-out confortable pour explorer
    // le dossier AVANT d'être éjecté. Doit rester < le SCALE_BURST d'entrée
    // (0.6) pour qu'une entrée ne déclenche jamais une sortie immédiate. Borne
    // basse = MIN_SCALE pour qu'un dossier énorme reste explorable sans
    // éjection prématurée.
    return Math.min(0.6, Math.max(MIN_SCALE, scaleFit * 0.55));
  }

  // R-FIL-02 v3 — entrée PARESSEUSE : si le dossier cible n'est pas encore
  // scanné, on scanne son niveau (async) puis on l'étale, AVANT de naviguer et
  // de cadrer. `navigatingRef` empêche toute ré-entrée pendant le scan async.
  const navigatingRef = useRef(false);
  async function lazyEnter(parentBoardId: string, targetId: string, app: Application) {
    if (navigatingRef.current) return;
    navigatingRef.current = true;
    try {
      const st0 = useGlucoseStore.getState();
      const parent0 = st0.project.boards.find((b) => b.id === parentBoardId);
      const target0 = (parent0?.folders ?? []).find((f) => f.id === targetId);
      if (!target0) return;

      if (target0.mirrorSource?.pendingScan) {
        const result = await scanFolderForMirror(
          target0.mirrorSource.rootPath, 0, 0, target0.mirrorSource.sortBy,
        );
        useGlucoseStore.getState().expandFolder(parentBoardId, targetId, result.tree);
      }

      const st = useGlucoseStore.getState();
      const childBoard = st.project.boards.find((b) => b.id === target0.childBoardId);
      if (childBoard) {
        st.setViewport(target0.childBoardId, fitViewportFor(childBoard, app.screen.width, app.screen.height));
      }
      st.enterFolder(targetId);
      lastNavRef.current = performance.now(); // re-arme le cooldown après l'async
    } catch (e) {
      console.error("[lazyEnter] échec:", e);
    } finally {
      navigatingRef.current = false;
    }
  }

  function checkAutoNavigate(world: Container) {
    const now = performance.now();
    if (now - lastNavRef.current < NAV_COOLDOWN) return;
    if (navigatingRef.current) return;
    const app = appRef.current;
    if (!app) return;
    const scale = world.scale.x;
    const state = useGlucoseStore.getState();
    const board = getActiveBoard(state.project);

    if (scale >= ENTER_SCALE) {
      const cx = (app.screen.width / 2 - world.x) / scale;
      const cy = (app.screen.height / 2 - world.y) / scale;
      const target = (board.folders ?? []).find(
        (f) => cx >= f.x && cx <= f.x + f.width && cy >= f.y && cy <= f.y + f.height
      );
      if (target) {
        lastNavRef.current = now;
        // Entrée paresseuse (scan à la volée si nécessaire) + cadrage fit.
        void lazyEnter(board.id, target.id, app);
      }
    } else if (state.folderStack.length > 0) {
      const exitScale = adaptiveExitScale(board, app.screen.width, app.screen.height);
      if (scale <= exitScale) {
        lastNavRef.current = now;
        // À la sortie : cadre tout le contenu du PARENT → on atterrit en voyant
        // le dossier entier (avec celui qu'on quitte dedans), à une échelle sûre
        // bien au-dessus du seuil d'éjection → pas de cascade vers le haut.
        const prev = state.folderStack[state.folderStack.length - 1];
        const parentBoard = state.project.boards.find((b) => b.id === prev.boardId);
        if (parentBoard) {
          state.setViewport(prev.boardId, fitViewportFor(parentBoard, app.screen.width, app.screen.height));
        }
        state.exitFolder();
      }
    }
  }

  // Positionne le ghost en coordonnées écran avec le bon scale
  function updateGhostPosition(cx: number, cy: number, scale: number) {
    const el = ghostElRef.current;
    const data = ghostDataRef.current;
    if (!el || !data) return;

    if (data.type === "preset") {
      // Zones réelles : 340 × 700, gap 30 — CSS scale() gère le zoom
      const n = Math.max(1, data.slots?.length ?? 1);
      const W = n * 340 + (n - 1) * 30;
      const H = 700;
      el.style.left = `${cx - (W * scale) / 2}px`;
      el.style.top = `${cy - (H * scale) / 2}px`;
      el.style.transform = `scale(${scale})`;
      el.style.transformOrigin = "top left";
    } else {
      // Storyboard : dimensions réelles monde aussi
      const panelW = data.panelWidth ?? 280;
      const cols = Math.max(1, data.cols ?? 4);
      const gapVal = data.gap ?? 24;
      const ratio = Math.max(0.3, data.ratio ?? 16 / 9);
      const count = Math.max(1, data.count ?? cols * 2);
      const cellH = panelW / ratio;
      const descH = 40;
      const rows = Math.ceil(count / cols);
      const W = cols * panelW + (cols - 1) * gapVal;
      const H = rows * (cellH + descH) + (rows - 1) * gapVal;
      el.style.left = `${cx - (W * scale) / 2}px`;
      el.style.top = `${cy - (H * scale) / 2}px`;
      el.style.transform = `scale(${scale})`;
      el.style.transformOrigin = "top left";
    }
  }

  function fitView() {
    const world = worldRef.current;
    const app = appRef.current;
    if (!world || !app) return;
    const board = getActiveBoard(useGlucoseStore.getState().project);
    // CLEANUP P-05 : itération directe (le spread ...xs crash >65k éléments)
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    let count = 0;
    for (const img of board.images) {
      const l = img.x - img.width / 2, r = img.x + img.width / 2;
      const t = img.y - img.height / 2, b = img.y + img.height / 2;
      if (l < minX) minX = l; if (r > maxX) maxX = r;
      if (t < minY) minY = t; if (b > maxY) maxY = b;
      count++;
    }
    for (const ann of board.annotations) {
      // Flèches : étendre à (x2,y2). Autres : prendre la boîte (x,y,w,h) pour
      // que les tuiles entières comptent dans le fit.
      if (ann.type === "arrow") {
        const ax2 = ann.x2, ay2 = ann.y2;
        if (ann.x < minX) minX = ann.x; if (ann.x > maxX) maxX = ann.x;
        if (ax2 < minX) minX = ax2; if (ax2 > maxX) maxX = ax2;
        if (ann.y < minY) minY = ann.y; if (ann.y > maxY) maxY = ann.y;
        if (ay2 < minY) minY = ay2; if (ay2 > maxY) maxY = ay2;
      } else {
        const aw = ann.width ?? 160, ah = ann.height ?? 80;
        if (ann.x < minX) minX = ann.x; if (ann.x + aw > maxX) maxX = ann.x + aw;
        if (ann.y < minY) minY = ann.y; if (ann.y + ah > maxY) maxY = ann.y + ah;
      }
      count++;
    }
    // R-FIL — CRITIQUE : inclure les sous-dossiers (boîtes folder) dans le fit.
    // Sans ça, un dossier plein de sous-dossiers se cadrait sur du vide → "je
    // vois rien" + cascade de zoom (cf. retours utilisateur).
    for (const f of board.folders ?? []) {
      if (f.x < minX) minX = f.x; if (f.x + f.width > maxX) maxX = f.x + f.width;
      if (f.y < minY) minY = f.y; if (f.y + f.height > maxY) maxY = f.y + f.height;
      count++;
    }
    if (count === 0) return;
    const cw = maxX - minX || 1, ch = maxY - minY || 1;
    const pad = 80;
    const ns = Math.min(
      (app.screen.width - pad * 2) / cw,
      (app.screen.height - pad * 2) / ch,
      2,
    );
    world.scale.set(ns);
    world.x = app.screen.width / 2 - (minX + cw / 2) * ns;
    world.y = app.screen.height / 2 - (minY + ch / 2) * ns;
    refreshGrid(world);
    emitViewport(world);
    const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
    setViewport(boardId, { x: world.x, y: world.y, scale: ns });
  }

  function zoomToAnnotation(annId: string, padding = 80) {
    const world = worldRef.current;
    const app = appRef.current;
    if (!world || !app) return;
    const board = getActiveBoard(useGlucoseStore.getState().project);
    const ann = board.annotations.find(a => a.id === annId);
    if (!ann) return;
    // Les flèches n'ont pas de width/height — on cible le milieu du segment.
    const w = ann.type === "arrow" ? 0 : (ann.width  || 200);
    const h = ann.type === "arrow" ? 0 : (ann.height || 100);
    const centerX = ann.type === "arrow" ? (ann.x + ann.x2) / 2 : (ann.x + w / 2);
    const centerY = ann.type === "arrow" ? (ann.y + ann.y2) / 2 : (ann.y + h / 2);
    // Zoom pour que le bloc remplisse ~60% de l'écran
    const ns = Math.min(
      (app.screen.width - padding * 2) / w,
      (app.screen.height - padding * 2) / h,
      3,
    );
    world.scale.set(ns);
    world.x = app.screen.width / 2 - centerX * ns;
    world.y = app.screen.height / 2 - centerY * ns;
    refreshGrid(world);
    emitViewport(world);
  }

  useEffect(() => {
    const onFit = () => fitView();
    const onExport = () => {
      const app = appRef.current;
      if (!app) return;
      // Force a render to fill the WebGL buffer, then capture synchronously
      // BEFORE any async operation (dialog), otherwise the buffer is cleared
      app.renderer.render({ container: app.stage });
      const dataUrl = app.canvas.toDataURL("image/png");
      const name = useGlucoseStore.getState().project.name || "atelier";
      exportCanvasPng(dataUrl, name).catch((err) =>
        alert(`Erreur export:\n${err?.message || String(err)}`)
      );
    };
    const onJump = (e: Event) => {
      const { wx, wy } = (e as CustomEvent<{ wx: number; wy: number }>).detail;
      const world = worldRef.current;
      const app = appRef.current;
      if (!world || !app) return;
      const scale = world.scale.x;
      const targetX = app.screen.width / 2 - wx * scale;
      const targetY = app.screen.height / 2 - wy * scale;
      const startX = world.x;
      const startY = world.y;
      const startTime = performance.now();
      const duration = 280;
      if (jumpAnimRef.current) cancelAnimationFrame(jumpAnimRef.current);
      function easeOut(t: number) { return 1 - Math.pow(1 - t, 3); }
      function step() {
        const t = easeOut(Math.min(1, (performance.now() - startTime) / duration));
        world!.x = startX + (targetX - startX) * t;
        world!.y = startY + (targetY - startY) * t;
        refreshGrid(world!);
        emitViewport(world!);
        if (t < 1) {
          jumpAnimRef.current = requestAnimationFrame(step);
        } else {
          const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
          setViewport(boardId, { x: world!.x, y: world!.y, scale });
        }
      }
      jumpAnimRef.current = requestAnimationFrame(step);
    };
    const onPanTo = (e: Event) => {
      const { wx, wy } = (e as CustomEvent<{ wx: number; wy: number }>).detail;
      const world = worldRef.current;
      const app = appRef.current;
      if (!world || !app) return;
      if (jumpAnimRef.current) { cancelAnimationFrame(jumpAnimRef.current); jumpAnimRef.current = null; }
      const scale = world.scale.x;
      world.x = app.screen.width / 2 - wx * scale;
      world.y = app.screen.height / 2 - wy * scale;
      refreshGrid(world);
      emitViewport(world);
    };
    // ── Numpad navigation ────────────────────────────────────────
    const PAN_STEP = 60;
    function onNumpadKey(e: KeyboardEvent) {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      const world = worldRef.current;
      const app = appRef.current;
      if (!world || !app) return;
      let dx = 0, dy = 0, zoom = 0;
      switch (e.code) {
        case "Numpad4": dx = PAN_STEP; break;
        case "Numpad6": dx = -PAN_STEP; break;
        case "Numpad8": dy = PAN_STEP; break;
        case "Numpad2": dy = -PAN_STEP; break;
        case "NumpadAdd": zoom = 1; break;
        case "NumpadSubtract": zoom = -1; break;
        default: return;
      }
      e.preventDefault();
      if (dx !== 0 || dy !== 0) {
        world.x += dx;
        world.y += dy;
        refreshGrid(world);
        emitViewport(world);
      }
      if (zoom !== 0) {
        const factor = zoom > 0 ? 1.1 : 0.9;
        const ns = Math.min(MAX_SCALE, Math.max(MIN_SCALE, world.scale.x * factor));
        const cx = app.screen.width / 2;
        const cy = app.screen.height / 2;
        world.x = cx - (cx - world.x) * (ns / world.scale.x);
        world.y = cy - (cy - world.y) * (ns / world.scale.y);
        world.scale.set(ns);
        refreshGrid(world);
        emitViewport(world);
      }
    }
    window.addEventListener("keydown", onNumpadKey);

    const onZoomToAnn = (e: Event) => {
      const { annId, padding } = (e as CustomEvent<{ annId: string; padding?: number }>).detail;
      zoomToAnnotation(annId, padding);
    };

    window.addEventListener("glucose:fit-view", onFit);
    window.addEventListener("glucose:export-png", onExport);
    window.addEventListener("glucose:jump-viewport", onJump);
    window.addEventListener("glucose:pan-viewport-to", onPanTo);
    window.addEventListener("glucose:zoom-to-annotation", onZoomToAnn);
    return () => {
      window.removeEventListener("keydown", onNumpadKey);
      window.removeEventListener("glucose:fit-view", onFit);
      window.removeEventListener("glucose:export-png", onExport);
      window.removeEventListener("glucose:jump-viewport", onJump);
      window.removeEventListener("glucose:pan-viewport-to", onPanTo);
      window.removeEventListener("glucose:zoom-to-annotation", onZoomToAnn);
    };
  }, [pixiReady]);

  // ── Listener supprimé (les points d'attache ont été retirés) ─────────


  // ── Snap flèche vers le bord le plus proche d'un élément ─────
  function snapToNearest(wx: number, wy: number, excludeAnnId?: string, excludeElementId?: string): { x: number; y: number; elementId?: string; elementBlockId?: string } {
    let best = { x: wx, y: wy };
    const HIT_DIST = 120; // Distance de snap fixe en pixels monde
    let bestDist = HIT_DIST;
    let bestId: string | undefined;
    let bestBlockId: string | undefined;

    const hitAnnIds = new Map<string, number>();

    function tryEdgeBox(bx: number, by: number, bw: number, bh: number, elementId: string, blockId?: string) {
      const inside = wx >= bx && wx <= bx + bw && wy >= by && wy <= by + bh;
      let ex: number, ey: number;
      if (inside) {
        ex = wx; ey = wy; // If inside, snap exactly to cursor
      } else {
        ex = Math.max(bx, Math.min(bx + bw, wx));
        ey = Math.max(by, Math.min(by + bh, wy));
      }
      const d = Math.hypot(ex - wx, ey - wy);
      if (d <= HIT_DIST) {
        if (blockId) hitAnnIds.set(elementId, (hitAnnIds.get(elementId) || 0) + 1);
        if (d < bestDist) { 
          bestDist = d; 
          best = { x: ex, y: ey }; 
          bestId = elementId; 
          bestBlockId = blockId; 
        }
      }
    }

    // Images (anchor 0.5 → top-left = sprite.x - w/2, sprite.y - h/2)
    spritesRef.current.forEach((sprite, id) => {
      if (id === excludeElementId) return;
      tryEdgeBox(
        sprite.x - sprite.width / 2,
        sprite.y - sprite.height / 2,
        sprite.width, sprite.height,
        id,
      );
    });

    // Annotations — lire depuis le store directement (évite la closure figée)
    const liveAnnotations = getActiveBoard(useGlucoseStore.getState().project).annotations;
    liveAnnotations.forEach((ann) => {
      if (ann.id === excludeAnnId || ann.id === excludeElementId) return;
      if (ann.type === "sticky") {
        tryEdgeBox(ann.x, ann.y, ann.width ?? 160, ann.height ?? 120, ann.id);
      } else if (ann.type === "text") {
        tryEdgeBox(ann.x - 4, ann.y - 4, 80, 20, ann.id);
      }
    });

    // Sub-blocks (DOM nodes)
    const world = worldRef.current;
    if (world && appRef.current) {
      const canvasRect = appRef.current.canvas.getBoundingClientRect();
      document.querySelectorAll('.glucose-sub-block').forEach((el) => {
        const rect = el.getBoundingClientRect();
        const bx = (rect.left - canvasRect.left - world.x) / world.scale.x;
        const by = (rect.top - canvasRect.top - world.y) / world.scale.y;
        const bw = rect.width / world.scale.x;
        const bh = rect.height / world.scale.y;
        const annId = el.getAttribute('data-ann-id');
        const blockId = el.getAttribute('data-block-id');
        if (annId && annId !== excludeAnnId && annId !== excludeElementId) {
          tryEdgeBox(bx, by, bw, bh, annId, blockId || undefined);
        }
      });
    }

    // Si le radius englobe plusieurs sous-parties du MÊME texte, on annule la sous-sélection
    // pour que l'entièreté de la boîte de texte soit ciblée (stade 2 / stade 3)
    if (bestId && (hitAnnIds.get(bestId) || 0) > 1) {
      bestBlockId = undefined;
    }

    return { ...best, elementId: bestId, elementBlockId: bestBlockId };
  }

  // ── Sprite events ─────────────────────────────────────────────
  function attachSpriteEvents(sprite: Sprite, id: string) {
    sprite.on("pointerover", () => useGlucoseStore.getState().setHoveredNodeId(id));
    sprite.on("pointerout", () => {
      if (useGlucoseStore.getState().hoveredNodeId === id) {
        useGlucoseStore.getState().setHoveredNodeId(null);
      }
    });
    sprite.on("pointerdown", (e: FederatedPointerEvent) => {
      if (e.button !== 0) return;
      const state = useGlucoseStore.getState();
      if (state.activeTool !== "select") return;
      e.stopPropagation();
      const img = getActiveBoard(state.project).images.find((i) => i.id === id);
      if (!img || img.locked) return;

      const curImgs = state.selectedImageIds;
      const isSelected = curImgs.includes(id);
      const multi = e.ctrlKey || e.metaKey || e.shiftKey;

      if (multi) {
        setSelectedImageIds(isSelected ? curImgs.filter((x) => x !== id) : [...curImgs, id]);
      } else if (!isSelected) {
        setSelectedImageIds([id]);
        setSelectedAnnotationIds([]);
      }
      // Si déjà sélectionné, on ne touche à rien pour permettre le drag groupé (images + texte)

      state.pushHistory();
      const world = worldRef.current!;
      draggedSpriteRef.current = {
        id, startX: img.x, startY: img.y,
        pStartX: (e.globalX - world.x) / world.scale.x,
        pStartY: (e.globalY - world.y) / world.scale.y,
      };
    });
  }

  function setupEvents(app: Application, world: Container) {
    const stage = app.stage;
    stage.interactive = true;
    stage.hitArea = app.screen;

    stage.on("pointerdown", (e: FederatedPointerEvent) => {
      if (draggedSpriteRef.current) return;
      if (e.button !== 0) return;
      const tool = useGlucoseStore.getState().activeTool;
      const wx = (e.globalX - world.x) / world.scale.x;
      const wy = (e.globalY - world.y) / world.scale.y;

      if (tool === "text" || tool === "sticky") {
        // Marque pour que le fallback DOM ne re-crée pas.
        lastDomCreateRef.current = Date.now();
        const ann: Annotation = tool === "sticky"
          ? {
              id: nanoid(), type: "sticky", x: wx, y: wy, text: "",
              fontSize: 13,
              color: "#ffffff",
              bgColor: "#f5c542",
              width: 160,
              height: 120,
            }
          : {
              id: nanoid(), type: "text", x: wx, y: wy, text: "",
              fontSize: 14,
              color: "#ffffff",
            };
        useGlucoseStore.getState().addAnnotation(
          getActiveBoard(useGlucoseStore.getState().project).id, ann
        );
        useGlucoseStore.getState().setActiveTool("select");
        // Ouvre l'overlay en calculant la position depuis les coords monde
        const rect = app.canvas.getBoundingClientRect();
        const scale = world.scale.x;
        const annW = ann.type === "sticky" ? ann.width  : undefined;
        const annH = ann.type === "sticky" ? ann.height : undefined;
        const annBg = ann.type === "sticky" ? ann.bgColor : undefined;
        setEditOverlay({
          annId: ann.id, type: tool,
          screenX: rect.left + world.x + ann.x * scale,
          screenY: rect.top + world.y + ann.y * scale,
          scale,
          width: annW ? annW * scale : undefined,
          height: annH ? annH * scale : undefined,
          fontSize: ann.fontSize ?? (tool === "sticky" ? 13 : 14),
          color: ann.color ?? "#ffffff",
          bgColor: annBg,
          cursorPos: undefined,
        });
        setEditText("");
        return;
      }

      if (tool === "arrow") {
        const snapped = snapToNearest(wx, wy);
        const ann: Annotation = {
          id: nanoid(), type: "arrow",
          x: snapped.x, y: snapped.y,
          x2: snapped.x + 1, y2: snapped.y,
          color: "#ffffff", arrowType: "straight",
          sourceId: snapped.elementId,
          sourceBlockId: snapped.elementBlockId,
        };
        useGlucoseStore.getState().addAnnotation(
          getActiveBoard(useGlucoseStore.getState().project).id, ann
        );
        arrowIdRef.current = ann.id;
        return;
      }

      if (tool === "folder") {
        zonePendingActionRef.current = "folder";
        setZonePendingAction("folder");
        zoneStartRef.current = { sx: e.globalX, sy: e.globalY };
        useGlucoseStore.getState().setActiveTool("zone-select");
        return;
      }

      if (tool === "membrane") {
        zonePendingActionRef.current = "membrane";
        setZonePendingAction("membrane");
        zoneStartRef.current = { sx: e.globalX, sy: e.globalY };
        useGlucoseStore.getState().setActiveTool("zone-select");
        return;
      }

      if (tool === "zone-select") {
        zoneStartRef.current = { sx: e.globalX, sy: e.globalY };
        return;
      }

      if (tool === "select") {
        zoneRendererRef.current?.deselect();
        setSelectedFolderId(null);
        selDragRef.current = { sx: e.globalX, sy: e.globalY };
        setSelectedImageIds([]);
        setSelectedAnnotationIds([]);
        return;
      }

      isDraggingRef.current = true;
      startCursorGrab();
      try { app.canvas.setPointerCapture((e.nativeEvent as PointerEvent).pointerId); } catch { /* ignore */ }
    });

    stage.on("pointermove", (e: FederatedPointerEvent) => {
      zoneRendererRef.current?.handleGlobalMove(e, world);

      if (zoneStartRef.current) {
        const { sx, sy } = zoneStartRef.current;
        const rx = Math.min(sx, e.globalX); const ry = Math.min(sy, e.globalY);
        const rw = Math.abs(e.globalX - sx); const rh = Math.abs(e.globalY - sy);
        const gfx = zoneGfxRef.current;
        if (gfx) {
          gfx.clear();
          if (rw > 4 || rh > 4) {
            gfx.rect(rx, ry, rw, rh).fill({ color: 0x3b82f6, alpha: 0.08 });
            gfx.rect(rx, ry, rw, rh).stroke({ color: 0x3b82f6, width: 2, alpha: 0.85 });
            gfx.rect(rx - 4, ry - 4, 8, 8).fill({ color: 0xffffff, alpha: 0.9 });
          }
        }
        // Live dimensions label — mise à jour impérative sans re-render React
        const lbl = zoneLabelRef.current;
        if (lbl) {
          if (rw > 8 || rh > 8) {
            const wW = Math.round(rw / world.scale.x);
            const wH = Math.round(rh / world.scale.y);
            lbl.style.display = "block";
            lbl.style.left = `${e.globalX + 14}px`;
            lbl.style.top = `${e.globalY + 14}px`;
            lbl.textContent = `${wW} × ${wH}`;
          } else {
            lbl.style.display = "none";
          }
        }
        return;
      }

      if (arrowIdRef.current) {
        const wx = (e.globalX - world.x) / world.scale.x;
        const wy = (e.globalY - world.y) / world.scale.y;
        const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
        const arrowAnnRaw = useGlucoseStore.getState().project.boards.find(b => b.id === boardId)?.annotations.find(a => a.id === arrowIdRef.current);
        const arrowAnn = arrowAnnRaw && arrowAnnRaw.type === "arrow" ? arrowAnnRaw : undefined;
        const snapped = snapToNearest(wx, wy, arrowIdRef.current, arrowAnn?.sourceId);
        useGlucoseStore.getState().updateAnnotation(boardId, arrowIdRef.current, {
          x2: snapped.x,
          y2: snapped.y,
          targetId: snapped.elementId,
          targetBlockId: snapped.elementBlockId
        });
        
        // Dispatch target preview for strong glow
        window.dispatchEvent(new CustomEvent("glucose:arrow-target-preview", { 
          detail: snapped.elementId ? { annId: snapped.elementId, blockId: snapped.elementBlockId } : null 
        }));
        return;
      } else if (useGlucoseStore.getState().activeTool === "arrow") {
        const wx = (e.globalX - world.x) / world.scale.x;
        const wy = (e.globalY - world.y) / world.scale.y;
        const snapped = snapToNearest(wx, wy);
        window.dispatchEvent(new CustomEvent("glucose:arrow-target-preview", { 
          detail: snapped.elementId ? { annId: snapped.elementId, blockId: snapped.elementBlockId } : null 
        }));
      }
      if (draggedSpriteRef.current) {
        const { pStartX, pStartY } = draggedSpriteRef.current;
        const currentWX = (e.globalX - world.x) / world.scale.x;
        const currentWY = (e.globalY - world.y) / world.scale.y;
        const dx = currentWX - pStartX;
        const dy = currentWY - pStartY;
        draggedSpriteRef.current.pStartX = currentWX;
        draggedSpriteRef.current.pStartY = currentWY;
        const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
        useGlucoseStore.getState().moveSelected(boardId, dx, dy);
        return;
      }
      if (selDragRef.current) {
        const { sx, sy } = selDragRef.current;
        const rx = Math.min(sx, e.globalX); const ry = Math.min(sy, e.globalY);
        const rw = Math.abs(e.globalX - sx); const rh = Math.abs(e.globalY - sy);
        const gfx = selRectGfxRef.current;
        if (gfx) {
          gfx.clear();
          if (rw > 4 || rh > 4) {
            gfx.rect(rx, ry, rw, rh);
            gfx.stroke({ color: 0xffffff, width: 1, alpha: 0.5 });
            gfx.rect(rx, ry, rw, rh);
            gfx.fill({ color: 0xffffff, alpha: 0.03 });
          }
        }
        return;
      }
    });

    function finishArrow(wx: number, wy: number) {
      if (!arrowIdRef.current) return;
      const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
      const arrowAnnRaw = useGlucoseStore.getState().project.boards.find(b => b.id === boardId)?.annotations.find(a => a.id === arrowIdRef.current);
      const arrowAnn = arrowAnnRaw && arrowAnnRaw.type === "arrow" ? arrowAnnRaw : undefined;
      const snapped = snapToNearest(wx, wy, arrowIdRef.current, arrowAnn?.sourceId);

      let targetId = snapped.elementId;
      const targetBlockId = snapped.elementBlockId;

      if (!targetId) {
        const newBlockId = nanoid();
        const newBlock: Annotation = {
          id: newBlockId, type: "text", x: snapped.x, y: snapped.y, text: "",
          fontSize: 14, color: "#ffffff"
        };
        useGlucoseStore.getState().addAnnotation(boardId, newBlock);
        targetId = newBlockId;
        
        // Auto-open edit
        const rect = appRef.current!.canvas.getBoundingClientRect();
        const scale = worldRef.current!.scale.x;
        setEditOverlay({
          annId: newBlock.id, type: "text",
          screenX: rect.left + worldRef.current!.x + newBlock.x * scale,
          screenY: rect.top + worldRef.current!.y + newBlock.y * scale,
          scale,
          fontSize: 14,
          color: newBlock.color || "#ffffff",
          cursorPos: undefined,
        });
        setEditText("");
      }

      useGlucoseStore.getState().updateAnnotation(boardId, arrowIdRef.current, {
        x2: snapped.x, y2: snapped.y,
        targetId: targetId,
        targetBlockId: targetBlockId,
      });
      useGlucoseStore.getState().setActiveTool("select");
      arrowIdRef.current = null;
      window.dispatchEvent(new CustomEvent("glucose:arrow-target-preview", { detail: null }));
    }

    stage.on("pointerup", (e: FederatedPointerEvent) => {
      zoneRendererRef.current?.clearDragState();

      if (zoneStartRef.current) {
        const { sx, sy } = zoneStartRef.current;
        const rw = Math.abs(e.globalX - sx); const rh = Math.abs(e.globalY - sy);
        if (rw > 20 && rh > 20) {
          const minSx = Math.min(sx, e.globalX); const minSy = Math.min(sy, e.globalY);
          const wX = (minSx - world.x) / world.scale.x;
          const wY = (minSy - world.y) / world.scale.y;
          const wW = rw / world.scale.x;
          const wH = rh / world.scale.y;
          const action = zonePendingActionRef.current;
          if (action === "folder") {
            const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
            createFolder(boardId, { id: nanoid(), name: "Dossier", color: "#60a5fa", x: wX, y: wY, width: wW, height: wH });
          } else if (action === "membrane") {
            const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
            const MEMBRANE_COLORS = ["#60a5fa", "#34d399", "#f472b6", "#fbbf24", "#a78bfa", "#f87171"];
            const col = MEMBRANE_COLORS[Math.floor(Math.random() * MEMBRANE_COLORS.length)];
            useGlucoseStore.getState().addAnnotation(boardId, {
              id: nanoid(), type: "membrane",
              x: wX, y: wY, width: wW, height: wH,
              color: col, text: "",
            });
          }
          window.dispatchEvent(new CustomEvent("glucose:zone-selected", { detail: { x: wX, y: wY, w: wW, h: wH } }));
        }
        zoneStartRef.current = null; zonePendingActionRef.current = null;
        if (zoneLabelRef.current) zoneLabelRef.current.style.display = "none";
        setZonePendingAction(null);
        zoneGfxRef.current?.clear();
        useGlucoseStore.getState().setActiveTool("select");
        return;
      }

      if (arrowIdRef.current) {
        const wx = (e.globalX - world.x) / world.scale.x;
        const wy = (e.globalY - world.y) / world.scale.y;
        finishArrow(wx, wy);
        return;
      }
      draggedSpriteRef.current = null;
      if (selDragRef.current) {
        const { sx, sy } = selDragRef.current;
        const rw = Math.abs(e.globalX - sx); const rh = Math.abs(e.globalY - sy);
        if (rw > 8 && rh > 8) {
          const rx0 = Math.min(sx, e.globalX); const ry0 = Math.min(sy, e.globalY);
          const rx1 = rx0 + rw; const ry1 = ry0 + rh;
          // Images — test par le centre du sprite (anchor 0.5)
          const imgHits: string[] = [];
          spritesRef.current.forEach((sprite, id) => {
            const cx = sprite.x * world.scale.x + world.x;
            const cy = sprite.y * world.scale.y + world.y;
            const hw = (sprite.width * world.scale.x) / 2;
            const hh = (sprite.height * world.scale.y) / 2;
            if (cx + hw >= rx0 && cx - hw <= rx1 && cy + hh >= ry0 && cy - hh <= ry1) imgHits.push(id);
          });
          setSelectedImageIds(imgHits);

          // Annotations (text, sticky, membrane, arrow…)
          const liveboard = getActiveBoard(useGlucoseStore.getState().project);
          const annHits: string[] = [];
          liveboard.annotations.forEach((ann) => {
            const sx2 = ann.x * world.scale.x + world.x;
            const sy2 = ann.y * world.scale.y + world.y;
            // Les flèches sont des segments — on les considère comme un point
            // (x,y). Pour les autres, on a un rectangle width × height.
            let aw = ann.type === "arrow" ? 0 : (ann.width  ?? 0);
            let ah = ann.type === "arrow" ? 0 : (ann.height ?? 0);
            aw *= world.scale.x;
            ah *= world.scale.y;
            if (sx2 + aw >= rx0 && sx2 <= rx1 && sy2 + ah >= ry0 && sy2 <= ry1) annHits.push(ann.id);
          });
          setSelectedAnnotationIds(annHits);
        }
        selDragRef.current = null;
        selRectGfxRef.current?.clear();
        return;
      }
      if (isDraggingRef.current) {
        isDraggingRef.current = false;
        stopCursorGrab();
        try { app.canvas.releasePointerCapture((e.nativeEvent as PointerEvent).pointerId); } catch { /* ignore */ }
        const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
        setViewport(boardId, { x: world.x, y: world.y, scale: world.scale.x });
      }
    });

    stage.on("pointerupoutside", (e: FederatedPointerEvent) => {
      zoneRendererRef.current?.clearDragState();
      if (arrowIdRef.current) {
        const wx = (e.globalX - world.x) / world.scale.x;
        const wy = (e.globalY - world.y) / world.scale.y;
        finishArrow(wx, wy);
      }
      draggedSpriteRef.current = null;
      selDragRef.current = null;
      selRectGfxRef.current?.clear();
      zoneStartRef.current = null; zonePendingActionRef.current = null;
      zoneGfxRef.current?.clear();
      if (isDraggingRef.current) {
        isDraggingRef.current = false;
        stopCursorGrab();
        try { app.canvas.releasePointerCapture((e.nativeEvent as PointerEvent).pointerId); } catch { /* ignore */ }
      }
    });

    // ── Native pan handler — bypasses PixiJS event routing so events still fire
    // when the cursor is outside the canvas hitArea (e.g. in the toolbar).
    // Uses raw DOM clientX/Y (viewport coords) consistent with _bounds.
    app.canvas.addEventListener("pointermove", (e: PointerEvent) => {
      if (!isDraggingRef.current) return;
      if (e.buttons === 0) { isDraggingRef.current = false; stopCursorGrab(); return; }
      const delta = getPanDelta(e.movementX, e.movementY, e.clientX, e.clientY);
      if (!delta) return;
      world.x += delta.dx;
      world.y += delta.dy;
      refreshGrid(world);
      emitViewport(world);
    });

    app.canvas.addEventListener("wheel", (e: WheelEvent) => {
      e.preventDefault();
      
      // Continuous zoom proportional to scroll speed — smooth on trackpad,
      // consistent on mouse wheel (120 units/notch on Windows)
      let delta = e.deltaY;
      if (e.deltaMode === 1) delta *= 40;  // line → pixel
      if (e.deltaMode === 2) delta *= 600; // page → pixel
      const factor = Math.pow(0.999, delta);
      const ns = Math.min(MAX_SCALE, Math.max(MIN_SCALE, world.scale.x * factor));
      const rect = app.canvas.getBoundingClientRect();
      const mx = e.clientX - rect.left; const my = e.clientY - rect.top;
      world.x = mx - (mx - world.x) * (ns / world.scale.x);
      world.y = my - (my - world.y) * (ns / world.scale.y);
      world.scale.set(ns);
      // L'entrée/sortie de dossier par zoom est gérée dans checkAutoNavigate
      // (appelé via emitViewport). On capture le board AVANT : si une navigation
      // a eu lieu, checkAutoNavigate a déjà posé le bon cadrage du board cible —
      // on NE DOIT PAS l'écraser avec le scale de la molette (sinon on atterrit
      // à scale 3 dans le vide → "je vois rien" + cascade).
      const boardBefore = getActiveBoard(useGlucoseStore.getState().project).id;
      refreshGrid(world);
      emitViewport(world);
      const boardAfter = getActiveBoard(useGlucoseStore.getState().project).id;
      if (boardAfter === boardBefore) {
        setViewport(boardBefore, { x: world.x, y: world.y, scale: ns });
      }
    }, { passive: false });

    app.canvas.addEventListener("pointerdown", (e: PointerEvent) => {
      if (e.button === 1) {
        isDraggingRef.current = true;
        selDragRef.current = null;
        startCursorGrab();
        try { app.canvas.setPointerCapture(e.pointerId); } catch { /* ignore */ }
      }
    });

    app.canvas.addEventListener("pointerup", (e: PointerEvent) => {
      if (!isDraggingRef.current) return;
      if (e.button === 0 || e.button === 1) {
        isDraggingRef.current = false;
        stopCursorGrab();
        try { app.canvas.releasePointerCapture(e.pointerId); } catch { /* ignore */ }
        const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
        const w = worldRef.current;
        if (w) useGlucoseStore.getState().setViewport(boardId, { x: w.x, y: w.y, scale: w.scale.x });
      }
    });
  }

  const handleDrop = useCallback(async (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    // Diagnostic — visible dans DevTools console (Ctrl+Shift+I)
    console.debug("[handleDrop] FIRED", {
      types: Array.from(e.dataTransfer?.types || []),
      filesCount: e.dataTransfer?.files?.length ?? 0,
      itemsCount: e.dataTransfer?.items?.length ?? 0,
    });
    const world = worldRef.current;
    if (!world) return;

    // Detect video URL dragged from browser (address bar or link)
    const plainText = (e.nativeEvent.dataTransfer?.getData("text/plain") || "").trim();
    const uriList = (e.nativeEvent.dataTransfer?.getData("text/uri-list") || "").split(/\r?\n/).find(s => s && !s.startsWith("#"))?.trim() ?? "";
    const candidate = plainText || uriList;
    if (candidate && VIDEO_URL_RE.test(candidate)) {
      // importVideoFromUrl uses only stable refs + stable setState — safe from stale closure
      setVideoLoading(true);
      try {
        const localPath: string = await invoke("download_video", { url: candidate });
        const assetUrl = convertFileSrc(localPath);
        const { width: vw, height: vh } = await getVideoDimensions(assetUrl);
        const w = worldRef.current;
        const cx = w ? (appRef.current!.screen.width / 2 - w.x) / w.scale.x : 0;
        const cy = w ? (appRef.current!.screen.height / 2 - w.y) / w.scale.y : 0;
        const s = vw > 800 ? 800 / vw : 1;
        addImage(getActiveBoard(useGlucoseStore.getState().project).id, {
          id: nanoid(), src: assetUrl, isVideo: true,
          x: cx, y: cy, width: vw * s, height: vh * s,
          rotation: 0, locked: false, tags: [],
          originalWidth: vw, originalHeight: vh,
        });
      } catch (err) {
        alert(`Erreur import vidéo:\n${err}`);
      } finally {
        setVideoLoading(false);
      }
      return;
    }

    const rect = canvasRef.current!.getBoundingClientRect();
    const wx = (e.clientX - rect.left - world.x) / world.scale.x;
    const wy = (e.clientY - rect.top - world.y) / world.scale.y;
    const state = useGlucoseStore.getState();
    await addImagesFromDrop(
      e.nativeEvent, wx, wy,
      getActiveBoard(state.project).id,
      addImage,
      addAnnotation,
      state.createFolderTree, // R-FIL-02 v2 : drop dossier OS → folder mirror navigable
    );
  }, []);

  function commitEdit() {
    if (!editOverlay) return;
    const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
    const ann = useGlucoseStore.getState().project.boards
      .find((b) => b.id === boardId)?.annotations
      .find((a) => a.id === editOverlay.annId);
    if (ann) {
      if (!editText.trim() && ann.type !== "sticky") {
        removeAnnotations(boardId, [editOverlay.annId]);
      } else {
        updateAnnotation(boardId, editOverlay.annId, { 
          text: editText,
          cursorPos: textCursorPosRef.current
        });
      }
    }
    setEditOverlay(null);
    setEditText("");
    setShowStickyPicker(false);
    textCursorPosRef.current = undefined;
  }

  // Taille dynamique du textarea texte (s'adapte à la ligne la plus longue ou à la largeur forcée)
  const textareaWidth = (() => {
    if (editOverlay?.type === "sticky") {
      return Math.max(60, (editOverlay.width ?? 160) - 16);
    }
    // Pour le texte, si on a une largeur forcée (redimensionnée), on l'utilise
    if (editOverlay?.width) {
      // On retire le padding (24px de chaque côté = 48px monde -> 48 * scale écran)
      return Math.max(60, editOverlay.width - 48 * (editOverlay.scale ?? 1));
    }
    // Sinon on mesure le texte pour auto-fit
    const { w } = measureTextSize(editText || "Aa", editOverlay?.fontSize ?? 14);
    return Math.max(60, w * (editOverlay?.scale ?? 1));
  })();

  return (
    <div
      ref={wrapperRef}
      style={{ flex: 1, position: "relative", overflow: "hidden", background: "#0d0d0d", cursor: showGhost || activeTool === "zone-select" ? "crosshair" : undefined }}
      onDrop={handleDrop}
      onDragEnter={(e) => { e.preventDefault(); e.stopPropagation(); }}
      onDragOver={(e) => {
        e.preventDefault();
        e.stopPropagation();
        // Indique au navigateur qu'on accepte le drop (curseur "+" au lieu de ⊘)
        if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
      }}
      onPointerDown={(e) => {
        // Fallback DOM-level pour création texte/sticky : si pour une raison
        // quelconque Pixi ne reçoit pas le pointerdown sur la stage, on crée
        // l'annotation ici directement. Le store déduplique de fait (le dernier
        // appel ne change rien si Pixi a déjà créé le bloc, et il n'y a qu'un
        // seul handler React).
        if (e.button !== 0) return;
        const tool = useGlucoseStore.getState().activeTool;
        const tgt = e.target as HTMLElement;
        if (tool !== "text" && tool !== "sticky") return;
        if (tgt.tagName !== "CANVAS") return;
        if (lastDomCreateRef.current && Date.now() - lastDomCreateRef.current < 100) return;
        lastDomCreateRef.current = Date.now();

        const rect = tgt.getBoundingClientRect();
        const vp = vpRef.current;
        const wx = (e.clientX - rect.left - vp.x) / vp.scale;
        const wy = (e.clientY - rect.top - vp.y) / vp.scale;
        const ann: Annotation = tool === "sticky"
          ? {
              id: nanoid(), type: "sticky", x: wx, y: wy, text: "",
              fontSize: 13,
              color: "#ffffff",
              bgColor: "#f5c542",
              width: 160,
              height: 120,
            }
          : {
              id: nanoid(), type: "text", x: wx, y: wy, text: "",
              fontSize: 14,
              color: "#ffffff",
            };
        const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
        useGlucoseStore.getState().addAnnotation(boardId, ann);
        useGlucoseStore.getState().setActiveTool("select");
        const annW = ann.type === "sticky" ? ann.width  : undefined;
        const annH = ann.type === "sticky" ? ann.height : undefined;
        const annBg = ann.type === "sticky" ? ann.bgColor : undefined;
        setEditOverlay({
          annId: ann.id, type: tool,
          screenX: rect.left + vp.x + ann.x * vp.scale,
          screenY: rect.top + vp.y + ann.y * vp.scale,
          scale: vp.scale,
          width: annW ? annW * vp.scale : undefined,
          height: annH ? annH * vp.scale : undefined,
          fontSize: ann.fontSize ?? (tool === "sticky" ? 13 : 14),
          color: ann.color ?? "#ffffff",
          bgColor: annBg,
          cursorPos: undefined,
        });
        setEditText("");
      }}
      onMouseMove={(e) => {
        const rect = e.currentTarget.getBoundingClientRect();
        const cx = e.clientX - rect.left;
        const cy = e.clientY - rect.top;
        cursorPosRef.current = { x: cx, y: cy };

        if (!showGhost) return;
        updateGhostPosition(cx, cy, vpRef.current.scale);
      }}
    >
      {/* Grille CSS vectorielle — radial-gradient, mise à jour via ref sans re-render */}
      <div
        ref={gridOverlayRef}
        style={{ position: "absolute", inset: 0, pointerEvents: "none", zIndex: 1 }}
      />
      {/* Canvas PixiJS transparent — images uniquement. */}
      <div
        ref={canvasRef}
        style={{ position: "absolute", inset: 0 }}
      />
      {/* Fil d'Ariane navigation dossiers */}
      <FolderBreadcrumb />

      {/* ── Zone Selector overlay ── */}
      <ZoneSelectorOverlay
        active={activeTool === "zone-select"}
        pendingAction={zonePendingAction}
        zoneLabelRef={zoneLabelRef}
      />
      {/* ── Couche SVG vectorielle pour les membranes ── */}
      <SvgAnnotationLayer
        annotations={board.annotations.filter((a) => a.type === "membrane")}
        selectedIds={selectedAnnotationIds}
        editingId={editOverlay?.annId ?? null}
        vpRef={vpRef}
        onSelect={(id, multi) => {
          setSelectedAnnotationIds(
            multi
              ? selectedAnnotationIds.includes(id)
                ? selectedAnnotationIds.filter((x) => x !== id)
                : [...selectedAnnotationIds, id]
              : [id]
          );
          if (!multi) setSelectedImageIds([]);
        }}
        onEdit={(id) => {
          const ann = board.annotations.find((a) => a.id === id);
          if (ann) openEditOverlay(ann);
        }}
        onResize={(id, x, y, w, h) => {
          const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
          updateAnnotation(boardId, id, { x, y, width: w, height: h });
        }}
      />

      {/* ── Couche HTML pour les textes et post-its (Markdown) ── */}
      <HtmlAnnotationLayer
        annotations={board.annotations.filter((a) => a.type === "text" || a.type === "sticky")}
        selectedIds={selectedAnnotationIds}
        editingId={editOverlay?.annId ?? null}
        vpRef={vpRef}
        onSelect={(id, multi) => {
          setSelectedAnnotationIds(
            multi
              ? selectedAnnotationIds.includes(id)
                ? selectedAnnotationIds.filter((x) => x !== id)
                : [...selectedAnnotationIds, id]
              : [id]
          );
          if (!multi) setSelectedImageIds([]);
        }}
        onEdit={(id) => {
          const ann = board.annotations.find((a) => a.id === id);
          if (ann) openEditOverlay(ann);
        }}
        onResize={(id, x, y, w, h) => {
          const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
          updateAnnotation(boardId, id, { x, y, width: w, height: h });
        }}
      />

      {/* ── Couche SVG vectorielle des flèches — au-dessus des stickies ── */}
      <ArrowSvgLayer
        board={board}
        vpRef={vpRef}
        editingId={editOverlay?.annId ?? null}
        selectedIds={selectedAnnotationIds}
        onSelect={(id, multi) => {
          setSelectedAnnotationIds(
            multi
              ? selectedAnnotationIds.includes(id)
                ? selectedAnnotationIds.filter((x) => x !== id)
                : [...selectedAnnotationIds, id]
              : [id]
          );
          if (!multi) setSelectedImageIds([]);
        }}
      />

      {/* ── Couche SVG dossiers — rendu vectoriel net ── */}
      <FolderSvgLayer
        folders={board.folders ?? []}
        boards={project.boards}
        selectedId={selectedFolderId}
        vpRef={vpRef}
        onSelect={(id) => {
          setSelectedFolderId(id);
          if (id) { setSelectedImageIds([]); setSelectedAnnotationIds([]); }
        }}
        onEnter={(id) => enterFolder(id)}
        onMove={(id, x, y) => {
          const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
          useGlucoseStore.getState().updateFolder(boardId, id, { x, y });
        }}
        onResize={(id, w, h) => {
          const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
          useGlucoseStore.getState().updateFolder(boardId, id, { width: w, height: h });
        }}
        onRename={(id, name) => {
          const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
          const folder = (getActiveBoard(useGlucoseStore.getState().project).folders ?? []).find((f) => f.id === id);
          // On met à jour name + child board name pour cohérence partout
          useGlucoseStore.getState().updateFolder(boardId, id, { name });
          if (folder?.childBoardId) useGlucoseStore.getState().renameBoard(folder.childBoardId, name);
        }}
      />

      {/* ── Overlay édition en place — textarea transparent positionné sur l'annotation ── */}
      {editOverlay && (
        <>
          {/* Détecte clic extérieur */}
          <div
            style={{ position: "fixed", inset: 0, zIndex: 9998 }}
            onPointerDown={commitEdit}
          />

          {/* Fond visuel du postit (l'annotation PixiJS est masquée pendant l'édition) */}
          {editOverlay.type === "sticky" && (
            <>
              <div style={{
                position: "fixed",
                left: editOverlay.screenX, top: editOverlay.screenY,
                width: editOverlay.width, height: editOverlay.height,
                background: editOverlay.bgColor ?? "#f5c542",
                opacity: 0.93, borderRadius: 2, zIndex: 9999,
                pointerEvents: "none",
              }} />
              {/* Bouton couleur sticky + picker */}
              <div
                style={{
                  position: "fixed",
                  left: editOverlay.screenX,
                  top: (editOverlay.screenY ?? 0) - 34,
                  zIndex: 10001,
                  display: "flex", alignItems: "center", gap: 6, padding: "4px 8px",
                  background: "#1a1a1a", borderRadius: 5, border: "1px solid #333",
                  boxShadow: "0 2px 8px rgba(0,0,0,0.5)",
                }}
                onPointerDown={(e) => e.stopPropagation()}
              >
                <span style={{ fontSize: 10, color: "#555" }}>Couleur</span>
                <div
                  onClick={() => setShowStickyPicker((v) => !v)}
                  style={{
                    width: 18, height: 18, borderRadius: 4, cursor: "pointer",
                    background: editOverlay.bgColor ?? "#f5c542",
                    border: showStickyPicker ? "2px solid #fff" : "2px solid #555",
                    flexShrink: 0,
                  }}
                />
              </div>
              {/* Picker flottant */}
              {showStickyPicker && (
                <div
                  style={{
                    position: "fixed",
                    left: editOverlay.screenX,
                    top: (editOverlay.screenY ?? 0) - 34 - 310,
                    zIndex: 10002,
                  }}
                  onPointerDown={(e) => e.stopPropagation()}
                >
                  <ColorPicker
                    color={editOverlay.bgColor ?? "#f5c542"}
                    onChange={(hex) => {
                      const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
                      updateAnnotation(boardId, editOverlay.annId, { bgColor: hex });
                      setEditOverlay((prev) => prev ? { ...prev, bgColor: hex } : null);
                    }}
                  />
                </div>
              )}
            </>
          )}

          <SyntaxEditor
            value={editText}
            onChange={(val) => setEditText(val)}
            scale={editOverlay.scale}
            type={editOverlay.type as "text" | "sticky"}
            onHeightChange={(need) => {
              const padding = editOverlay.type === "sticky" ? 16 : 32; // 16x2 pour text
              const minH = editOverlay.type === "sticky" ? 120 : 40;
              const newWorldH = Math.max(minH, need / editOverlay.scale + padding);
              if (Math.abs(newWorldH - (editOverlay.height ?? 0) / editOverlay.scale) > 4) {
                const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
                updateAnnotation(boardId, editOverlay.annId, { height: newWorldH });
                setEditOverlay((prev) => prev ? { ...prev, height: newWorldH * editOverlay.scale } : null);
              }
            }}
            onKeyDown={(e) => {
              e.stopPropagation();
              if (e.key === "Escape") {
                const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
                const ann = useGlucoseStore.getState().project.boards
                  .find((b) => b.id === boardId)?.annotations
                  .find((a) => a.id === editOverlay.annId);
                if (ann && !editText.trim() && ann.type !== "sticky") {
                  removeAnnotations(boardId, [editOverlay.annId]);
                } else if (ann) {
                  // Sauvegarder le curseur même si on Escape (si du texte existe)
                  updateAnnotation(boardId, editOverlay.annId, { 
                    text: editText,
                    cursorPos: textCursorPosRef.current
                  });
                }
                setEditOverlay(null); setEditText(""); setShowStickyPicker(false);
                textCursorPosRef.current = undefined;
              }
              if (e.key === "Enter" && e.ctrlKey) {
                e.preventDefault(); commitEdit();
              }
            }}
            initialCursorPos={editOverlay.cursorPos}
            onCursorChange={(pos) => { textCursorPosRef.current = pos; }}
            style={{
              position: "fixed",
              left: editOverlay.screenX + (editOverlay.type === "sticky" ? 8 : -4 * editOverlay.scale),
              top: editOverlay.screenY + (editOverlay.type === "sticky" ? 8 : -4 * editOverlay.scale),
              width: textareaWidth,
              minWidth: editOverlay.type === "text" ? 60 : undefined,
              height: editOverlay.type === "sticky"
                ? Math.max(40, (editOverlay.height ?? 120) - 16) : undefined,
              fontSize: (editOverlay.fontSize) * (editOverlay.scale),
              background: editOverlay.type === "sticky" ? "transparent" : "transparent",
              outline: editOverlay.type === "text"
                ? "1px dashed rgba(255,255,255,0.3)"
                : `1px solid ${editOverlay.bgColor ? "rgba(0,0,0,0.25)" : "#888"}`,
              color: editOverlay.type === "sticky" ? "#222222" : (editOverlay.color || "#ffffff"),
              zIndex: 10000,
            }}
          />
        </>
      )}

      {/* ── Options flèche sélectionnée ── */}
      {selectedArrow && <ArrowOptions arrow={selectedArrow} onEditText={() => setArrowEditorId(selectedArrow.id)} />}

      {/* ── Éditeur de texte flèche (modal) ── */}
      {arrowEditorId && (
        <ArrowTextEditor arrowId={arrowEditorId} onClose={() => setArrowEditorId(null)} />
      )}

      {/* ── Panneau description longue de flèche (Phase 5) ── */}
      {arrowDescPanel && (
        <ArrowDescriptionPanel
          arrowId={arrowDescPanel.arrowId}
          midX={arrowDescPanel.screenX}
          midY={arrowDescPanel.screenY}
          onClose={() => setArrowDescPanel(null)}
        />
      )}

      {/* ── Barre contextuelle dossier sélectionné ── */}
      {selectedFolderId && (() => {
        const selFolder = (board.folders ?? []).find((f) => f.id === selectedFolderId);
        if (!selFolder) return null;
        const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
        return (
          <FolderContextBar
            folder={selFolder}
            onColor={(hex) => useGlucoseStore.getState().updateFolder(boardId, selFolder.id, { color: hex })}
            onDelete={() => {
              if (window.confirm(`Supprimer le dossier "${selFolder.name}" ? (Annulable avec Ctrl+Z)`)) {
                useGlucoseStore.getState().removeFolders(boardId, [selFolder.id]);
                setSelectedFolderId(null);
                import("../components/Toast").then(({ showToast }) => showToast("Dossier supprimé", "🗑"));
              }
            }}
            onClose={() => setSelectedFolderId(null)}
          />
        );
      })()}



      {showGhost && ghostDataRef.current && (
        <div
          ref={ghostElRef}
          style={{
            position: "absolute",
            left: -9999, top: -9999,
            transformOrigin: "top left",
            pointerEvents: "none",
            zIndex: 10,
            filter: "blur(2px)",
            opacity: 0.55,
            animation: "ghostPulse 2s ease-in-out infinite",
          }}
        >
          <LayoutGhostSvg preview={ghostDataRef.current} />
        </div>
      )}

      {/* ── Chargement vidéo ── */}
      {videoLoading && (
        <div style={{
          position: "absolute", inset: 0,
          display: "flex", flexDirection: "column",
          alignItems: "center", justifyContent: "center",
          background: "rgba(0,0,0,0.5)", zIndex: 200,
          gap: 10, pointerEvents: "none",
        }}>
          <svg width="32" height="32" viewBox="0 0 32 32" fill="none" style={{ animation: "spin 1s linear infinite" }}>
            <circle cx="16" cy="16" r="13" stroke="#333" strokeWidth="3" />
            <path d="M16 3 A13 13 0 0 1 29 16" stroke="#60a5fa" strokeWidth="3" strokeLinecap="round" />
          </svg>
          <span style={{ color: "#ccc", fontSize: 13, fontFamily: "system-ui" }}>Import vidéo en cours…</span>
          <span style={{ color: "#555", fontSize: 10, fontFamily: "system-ui" }}>1ère fois : téléchargement yt-dlp (~12 Mo)</span>
        </div>
      )}
      {/* ── Minimap ── */}
      <Minimap />

      {/* ── Barre contextuelle sélection ── */}
      {(selectedImageIds.length + selectedAnnotationIds.length > 0) && !editOverlay && (
        <div style={{
          position: "absolute", bottom: 12, left: "50%", transform: "translateX(-50%)",
          background: "#1a1a1a", border: "1px solid #333", borderRadius: 6,
          display: "flex", alignItems: "center", gap: 4,
          padding: "4px 8px", fontSize: 11, color: "#666",
          boxShadow: "0 2px 12px rgba(0,0,0,0.5)",
        }}>
          <span style={{ paddingRight: 6, borderRight: "1px solid #2a2a2a" }}>
            {selectedImageIds.length + selectedAnnotationIds.length} sélectionné{selectedImageIds.length + selectedAnnotationIds.length > 1 ? "s" : ""}
          </span>

          {selectedImageIds.length > 0 && (() => {
            const selImgs = board.images.filter((img) => selectedImageIds.includes(img.id));
            const allLocked = selImgs.every((img) => img.locked);
            return (
              <button
                title={allLocked ? "Déverrouiller (L)" : "Verrouiller (L)"}
                onClick={() => {
                  const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
                  selectedImageIds.forEach((id) => updateImage(boardId, id, { locked: !allLocked }));
                }}
                style={{
                  display: "flex", alignItems: "center", gap: 4,
                  background: allLocked ? "#2a1a1a" : "transparent",
                  border: allLocked ? "1px solid #553333" : "1px solid transparent",
                  borderRadius: 4, padding: "2px 7px", cursor: "pointer",
                  color: allLocked ? "#f87171" : "#666", fontSize: 11,
                }}
              >
                {allLocked ? (
                  <svg width="10" height="10" viewBox="0 0 14 14" fill="none">
                    <rect x="2" y="6" width="10" height="7" rx="1" stroke="currentColor" strokeWidth="1.3" />
                    <path d="M4.5 6V4.5a2.5 2.5 0 015 0V6" stroke="currentColor" strokeWidth="1.3" />
                  </svg>
                ) : (
                  <svg width="10" height="10" viewBox="0 0 14 14" fill="none">
                    <rect x="2" y="6" width="10" height="7" rx="1" stroke="currentColor" strokeWidth="1.3" />
                    <path d="M4.5 6V4.5a2.5 2.5 0 015 0" stroke="currentColor" strokeWidth="1.3" />
                  </svg>
                )}
                {allLocked ? "Verrouillé" : "Verrouiller"}
              </button>
            );
          })()}

          {/* Tags — visible seulement quand 1 image sélectionnée */}
          {selectedImageIds.length === 1 && selectedAnnotationIds.length === 0 && (() => {
            const img = board.images.find((i) => i.id === selectedImageIds[0]);
            if (!img) return null;
            const tags = img.tags ?? [];
            function addTag(val: string) {
              const clean = val.trim().toLowerCase();
              if (!clean || tags.includes(clean)) return;
              updateImage(getActiveBoard(useGlucoseStore.getState().project).id, img!.id, { tags: [...tags, clean] });
            }
            function removeTag(t: string) {
              updateImage(getActiveBoard(useGlucoseStore.getState().project).id, img!.id, { tags: tags.filter((x) => x !== t) });
            }
            return (
              <>
                <div style={{ width: 1, height: 16, background: "#2a2a2a" }} />
                {tags.map((t) => (
                  <span key={t} style={{
                    display: "flex", alignItems: "center", gap: 3,
                    background: "#2a2a2a", borderRadius: 10,
                    padding: "1px 7px", fontSize: 10, color: "#888",
                  }}>
                    {t}
                    <span
                      onClick={() => removeTag(t)}
                      style={{ cursor: "pointer", color: "#555", marginLeft: 2 }}
                      onMouseOver={(e) => { (e.target as HTMLElement).style.color = "#f87171"; }}
                      onMouseOut={(e) => { (e.target as HTMLElement).style.color = "#555"; }}
                    >×</span>
                  </span>
                ))}
                <input
                  value={tagInput}
                  onChange={(e) => setTagInput(e.target.value)}
                  onPointerDown={(e) => e.stopPropagation()}
                  onKeyDown={(e) => {
                    e.stopPropagation();
                    if (e.key === "Enter" || e.key === ",") {
                      e.preventDefault();
                      addTag(tagInput);
                      setTagInput("");
                    }
                  }}
                  placeholder="+ tag"
                  style={{
                    background: "transparent", border: "none", outline: "none",
                    color: "#555", fontSize: 10, width: 44, padding: 0,
                  }}
                />
              </>
            );
          })()}

          <button
            title="Supprimer (Suppr)"
            onClick={() => {
              const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
              const total = selectedImageIds.length + selectedAnnotationIds.length;
              if (selectedImageIds.length > 0) removeImages(boardId, selectedImageIds);
              if (selectedAnnotationIds.length > 0) removeAnnotations(boardId, selectedAnnotationIds);
              setSelectedImageIds([]);
              setSelectedAnnotationIds([]);
              import("../components/Toast").then(({ showToast }) =>
                showToast(`${total} élément${total > 1 ? "s" : ""} supprimé${total > 1 ? "s" : ""}`, "🗑")
              );
            }}
            style={{
              display: "flex", alignItems: "center", gap: 4,
              background: "transparent", border: "1px solid transparent",
              borderRadius: 4, padding: "2px 7px", cursor: "pointer",
              color: "#666", fontSize: 11,
            }}
            onMouseOver={(e) => { e.currentTarget.style.color = "#f87171"; e.currentTarget.style.borderColor = "#553333"; }}
            onMouseOut={(e) => { e.currentTarget.style.color = "#666"; e.currentTarget.style.borderColor = "transparent"; }}
          >
            <svg width="10" height="10" viewBox="0 0 14 14" fill="none">
              <path d="M2 4h10M5 4V2.5h4V4M5.5 6.5v4M8.5 6.5v4M3 4l.5 8h7L11 4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
            </svg>
            Supprimer
          </button>
        </div>
      )}
    </div>
  );
}

// ── Barre contextuelle quand un dossier est sélectionné ─────────────────────
function FolderContextBar({
  folder, onColor, onDelete, onClose,
}: {
  folder: { id: string; name: string; color: string };
  onColor: (hex: string) => void;
  onDelete: () => void;
  onClose: () => void;
}) {
  const [showPicker, setShowPicker] = useState(false);

  // Écoute le raccourci Delete/Backspace si aucune sélection image/annotation
  useEffect(() => {
    function onDel() { onDelete(); }
    window.addEventListener("glucose:delete-selected-folder", onDel);
    return () => window.removeEventListener("glucose:delete-selected-folder", onDel);
  }, [onDelete]);

  return (
    <>
      <div style={{
        position: "absolute", bottom: 56, left: "50%", transform: "translateX(-50%)",
        background: "rgba(18,18,18,0.96)", border: "1px solid #2a2a2a",
        borderRadius: 8, padding: "6px 10px",
        display: "flex", alignItems: "center", gap: 8,
        zIndex: 40, boxShadow: "0 4px 20px rgba(0,0,0,0.5)",
      }}>
        <span style={{ fontSize: 11, color: "#888", paddingRight: 6, borderRight: "1px solid #2a2a2a" }}>
          {folder.name || "Sans titre"}
        </span>

        {/* Bouton couleur (ouvre/ferme le picker) */}
        <button
          onClick={() => setShowPicker((v) => !v)}
          title="Changer la couleur"
          style={{
            display: "flex", alignItems: "center", gap: 5,
            background: "transparent", border: "1px solid transparent",
            borderRadius: 4, padding: "3px 7px", cursor: "pointer",
            color: "#888", fontSize: 11,
          }}
          onMouseOver={(e) => { e.currentTarget.style.background = "#222"; }}
          onMouseOut={(e) => { e.currentTarget.style.background = "transparent"; }}
        >
          <div style={{
            width: 12, height: 12, borderRadius: 3,
            background: folder.color, border: "1px solid #444",
          }} />
          Couleur
        </button>

        {/* Bouton supprimer */}
        <button
          onClick={onDelete}
          title="Supprimer le dossier (Suppr)"
          style={{
            display: "flex", alignItems: "center", gap: 4,
            background: "transparent", border: "1px solid transparent",
            borderRadius: 4, padding: "3px 7px", cursor: "pointer",
            color: "#888", fontSize: 11,
          }}
          onMouseOver={(e) => { e.currentTarget.style.color = "#f87171"; e.currentTarget.style.borderColor = "#553333"; }}
          onMouseOut={(e) => { e.currentTarget.style.color = "#888"; e.currentTarget.style.borderColor = "transparent"; }}
        >
          <svg width="10" height="10" viewBox="0 0 14 14" fill="none">
            <path d="M2 4h10M5 4V2.5h4V4M5.5 6.5v4M8.5 6.5v4M3 4l.5 8h7L11 4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
          Supprimer
        </button>

        <button
          onClick={onClose}
          title="Désélectionner (Échap)"
          style={{
            background: "none", border: "none", color: "#555", cursor: "pointer",
            fontSize: 14, lineHeight: 1, padding: "0 2px",
          }}
        >✕</button>
      </div>

      {/* Picker flottant au-dessus */}
      {showPicker && (
        <div
          style={{
            position: "absolute", bottom: 100, left: "50%", transform: "translateX(-50%)",
            zIndex: 41,
          }}
          onPointerDown={(e) => e.stopPropagation()}
        >
          <ColorPicker color={folder.color} onChange={onColor} />
        </div>
      )}
    </>
  );
}

// ── Ghost SVG — contenu seul, positionné par le parent ───────────────────────
function LayoutGhostSvg({ preview }: { preview: GhostData }) {
  let svgW = 0, svgH = 0;
  let content: React.ReactNode = null;

  if (preview.type === "preset") {
    // Dimensions exactes du ZoneRenderer — CSS scale() gère le zoom, pas nous
    const W = 340, H = 700, GAP = 30, HEADER = 36;
    const slots = preview.slots ?? [];
    const n = Math.max(1, slots.length);
    svgW = n * W + (n - 1) * GAP;
    svgH = H;
    content = slots.map((slot, i) => (
      <g key={slot.id} transform={`translate(${i * (W + GAP)}, 0)`}>
        {/* Fond sombre — identique à ZoneRenderer */}
        <rect width={W} height={H} fill="#111111" fillOpacity={0.5} />
        {/* Bordure tiretée colorée */}
        <rect width={W} height={H} fill="none"
          stroke={slot.color} strokeOpacity={0.4} strokeWidth={1}
          strokeDasharray="8 6" />
        {/* Header coloré */}
        <rect width={W} height={HEADER} fill={slot.color} fillOpacity={0.15} />
        {/* Ligne 2px en haut */}
        <rect width={W} height={2} fill={slot.color} fillOpacity={0.85} />
        {/* Titre uppercase */}
        <text x={12} y={HEADER * 0.65}
          dominantBaseline="middle"
          fill={slot.color} fillOpacity={0.95}
          fontSize={13} fontWeight="600"
          fontFamily="system-ui, sans-serif"
          letterSpacing={1}>
          {slot.name.toUpperCase()}
        </text>
        {/* Description centrée */}
        {slot.description && (
          <text x={12} y={H / 2}
            dominantBaseline="middle"
            fill="#555555" fontSize={11}
            fontFamily="system-ui, sans-serif">
            {slot.description}
          </text>
        )}
      </g>
    ));
  }

  if (preview.type === "storyboard") {
    // Dimensions réelles monde — identiques à StoryboardLayer.buildPanel
    const { cols = 4, ratio = 16 / 9, count = 8, panelWidth: panW = 280, gap: gapVal = 24 } = preview;
    const safeCols = Math.max(1, cols);
    const safeCount = Math.max(1, Math.min(count, 24));
    const rows = Math.ceil(safeCount / safeCols);
    const cellW = panW;
    const cellH = cellW / Math.max(0.3, ratio);
    const descH = 40;
    svgW = cellW * safeCols + gapVal * (safeCols - 1);
    svgH = rows * (cellH + descH) + (rows - 1) * gapVal;
    content = Array.from({ length: safeCount }).map((_, idx) => {
      const col = idx % safeCols;
      const row = Math.floor(idx / safeCols);
      const x = col * (cellW + gapVal);
      const y = row * (cellH + descH + gapVal);
      return (
        <g key={idx} transform={`translate(${x}, ${y})`}>
          {/* Frame background — #1a1a1a comme StoryboardLayer */}
          <rect width={cellW} height={cellH} fill="#1a1a1a" stroke="#333333" strokeWidth={1} />
          {/* Badge numéro */}
          <rect x={4} y={4} width={24} height={18} fill="#222222" fillOpacity={0.9} />
          <text x={16} y={13}
            textAnchor="middle" dominantBaseline="middle"
            fill="#888888" fontSize={10} fontWeight="bold" fontFamily="system-ui">
            {idx + 1}
          </text>
          {/* Zone description */}
          <rect y={cellH} width={cellW} height={descH} fill="#111111" />
        </g>
      );
    });
  }

  return (
    <svg width={svgW} height={svgH} style={{ display: "block" }}>
      {content}
    </svg>
  );
}

function getImageDimensions(src: string): Promise<{ width: number; height: number }> {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => resolve({ width: img.naturalWidth, height: img.naturalHeight });
    img.onerror = () => resolve({ width: 400, height: 300 });
    img.src = src;
  });
}

function getVideoDimensions(src: string): Promise<{ width: number; height: number }> {
  return new Promise((resolve) => {
    const video = document.createElement("video");
    video.onloadedmetadata = () => resolve({ width: video.videoWidth || 640, height: video.videoHeight || 360 });
    video.onerror = () => resolve({ width: 640, height: 360 });
    video.src = src;
  });
}
