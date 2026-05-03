import { Container, Graphics, Text, TextStyle, Rectangle, FederatedPointerEvent } from "pixi.js";
import { Annotation } from "../types";
import { useGlucoseStore, getActiveBoard } from "../store";

type OnSelect = (id: string, multi: boolean) => void;
type OnMove   = (id: string, x: number, y: number) => void;
type OnEdit   = (id: string) => void;

interface DragState {
  id: string;
  part: string;        // "body" | "start-handle" | "end-handle" | "wp-N" | "mid-N" | "resize-br/bl/tr/tl"
  startX: number;      // annotation x (body/start), OR waypoint world X (wp drag)
  startY: number;
  startX2: number; startY2: number;
  startWidth?: number;
  startHeight?: number;
  pStartX: number; pStartY: number;
  didMove: boolean;
  downTime: number;
}

// Points monde de la flèche (start + waypoints + end)
function arrowWorldPoints(ann: Annotation): { x: number; y: number }[] {
  const pts: { x: number; y: number }[] = [{ x: ann.x, y: ann.y }];
  if (ann.waypoints) pts.push(...ann.waypoints);
  pts.push({ x: ann.x2 ?? ann.x + 100, y: ann.y2 ?? ann.y });
  return pts;
}

// Convertit un point monde en coordonnées locales du container (origine = ann.x/y)
function toLocal(pt: { x: number; y: number }, ann: Annotation) {
  return { x: pt.x - ann.x, y: pt.y - ann.y };
}

export class AnnotationLayer {
  private container: Container;
  private objects   = new Map<string, Container>();
  private dragState: DragState | null = null;
  private getWorld: () => Container | null;
  private _onMove:  OnMove | null = null;

  constructor(parent: Container, getWorld: () => Container | null) {
    this.container = new Container();
    parent.addChild(this.container);
    this.getWorld = getWorld;
  }

  sync(
    annotations: Annotation[],
    selectedIds: string[],
    editingId: string | null,
    onSelect: OnSelect,
    onMove: OnMove,
    onEdit: OnEdit,
  ) {
    this._onMove = onMove;
    const current = new Set(annotations.map((a) => a.id));

    // Supprimer les objets absents ou non-flèches (gérés par SvgAnnotationLayer)
    this.objects.forEach((obj, id) => {
      const ann = annotations.find((a) => a.id === id);
      if (!current.has(id) || (ann && ann.type !== "arrow")) {
        this.container.removeChild(obj);
        obj.destroy({ children: true });
        this.objects.delete(id);
      }
    });

    for (const ann of annotations) {
      if (ann.type !== "arrow") continue; // texte + sticky → SvgAnnotationLayer
      const sel = selectedIds.includes(ann.id);
      const existing = this.objects.get(ann.id);
      if (existing) {
        existing.x = ann.x;
        existing.y = ann.y;
        existing.visible = ann.id !== editingId;
        this.rebuildGfx(existing, ann, sel, onSelect, onEdit);
      } else {
        const obj = new Container();
        obj.interactive = true;
        obj.cursor = "pointer";
        obj.x = ann.x;
        obj.y = ann.y;
        obj.visible = ann.id !== editingId;
        this.rebuildGfx(obj, ann, sel, onSelect, onEdit);
        this.container.addChild(obj);
        this.objects.set(ann.id, obj);
      }
    }
  }

  // ── Drag global (appelé depuis stage.pointermove) ────────────
  handleGlobalMove(e: FederatedPointerEvent, world: Container) {
    const ds = this.dragState;
    if (!ds) return;
    const wx = (e.globalX - world.x) / world.scale.x;
    const wy = (e.globalY - world.y) / world.scale.y;
    const dx = wx - ds.pStartX;
    const dy = wy - ds.pStartY;
    if (Math.abs(dx) > 3 || Math.abs(dy) > 3) ds.didMove = true;
    if (!ds.didMove) return;

    const boardId = getActiveBoard(useGlucoseStore.getState().project).id;

    if (ds.part.startsWith("wp-")) {
      const idx = parseInt(ds.part.slice(3));
      const ann = useGlucoseStore.getState().project.boards
        .find((b) => b.id === boardId)?.annotations.find((a) => a.id === ds.id);
      if (!ann?.waypoints) return;
      const wps = [...ann.waypoints];
      wps[idx] = { x: ds.startX + dx, y: ds.startY + dy };
      useGlucoseStore.getState().updateAnnotation(boardId, ds.id, { waypoints: wps });
    } else if (ds.part === "end-handle") {
      useGlucoseStore.getState().updateAnnotation(boardId, ds.id, {
        x2: ds.startX2 + dx, y2: ds.startY2 + dy,
      });
    } else if (ds.part === "start-handle") {
      useGlucoseStore.getState().updateAnnotation(boardId, ds.id, {
        x: ds.startX + dx, y: ds.startY + dy,
      });
    } else if (ds.part.startsWith("resize-")) {
      const corner = ds.part.slice(7); // "br" | "bl" | "tr" | "tl"
      const sw = ds.startWidth  ?? 160;
      const sh = ds.startHeight ?? 120;
      let newX = ds.startX, newY = ds.startY, newW = sw, newH = sh;
      if (corner === "br") { newW = Math.max(60, sw + dx); newH = Math.max(40, sh + dy); }
      else if (corner === "bl") { newW = Math.max(60, sw - dx); newH = Math.max(40, sh + dy); newX = ds.startX + (sw - newW); }
      else if (corner === "tr") { newW = Math.max(60, sw + dx); newH = Math.max(40, sh - dy); newY = ds.startY + (sh - newH); }
      else if (corner === "tl") { newW = Math.max(60, sw - dx); newH = Math.max(40, sh - dy); newX = ds.startX + (sw - newW); newY = ds.startY + (sh - newH); }
      useGlucoseStore.getState().updateAnnotation(boardId, ds.id, {
        x: newX, y: newY, width: newW, height: newH,
      });
    } else if (this._onMove) {
      this._onMove(ds.id, ds.startX + dx, ds.startY + dy);
    }
  }

  clearDragState() { this.dragState = null; }
  hasDragState()   { return !!this.dragState; }

  // ── Rebuild ──────────────────────────────────────────────────
  private rebuildGfx(
    c: Container, ann: Annotation, sel: boolean,
    onSelect: OnSelect, onEdit: OnEdit,
  ) {
    c.removeAllListeners();
    while (c.children.length > 0) {
      const ch = c.removeChildAt(0);
      (ch as any).destroy?.({ children: true });
    }
    this.buildGfx(c, ann, sel);
    this.attachEvents(c, ann, onSelect, onEdit);
  }

  private buildGfx(c: Container, ann: Annotation, selected: boolean) {
    if      (ann.type === "arrow")  this.buildArrow(c, ann, selected);
    else if (ann.type === "sticky") this.buildSticky(c, ann, selected);
    else                            this.buildText(c, ann, selected);
  }

  // ── Arrow ────────────────────────────────────────────────────
  private buildArrow(c: Container, ann: Annotation, selected: boolean) {
    const worldPts  = arrowWorldPoints(ann);
    const localPts  = worldPts.map((p) => toLocal(p, ann));
    const n         = localPts.length;
    const lastLocal = localPts[n - 1];
    // const col = parseInt((ann.color ?? "#ffffff").replace("#", ""), 16) || 0xffffff;
    // const sw  = ann.strokeWidth ?? 2;
    const curved = ann.arrowType === "curved";

    // Zone de clic invisible — trait épais sur tout le parcours
    const hit = new Graphics();
    drawPath(hit, localPts, curved);
    hit.stroke({ color: 0xffffff, width: 24, alpha: 0.01 });
    c.addChild(hit);

    // Pointe finale (calculée pour les éventuels besoins futurs)
    // const endTangentAngle = getTangentAngle(localPts, n - 1, "end");
    // (Le rendu visuel de la flèche, du texte et du badge a été déplacé dans ArrowSvgLayer pour passer au-dessus des post-its)

    if (selected) {
      // Handles extrémités
      addHandle(c, 0, 0, 0xffffff, "start-handle");
      addHandle(c, lastLocal.x, lastLocal.y, 0xaaaaaa, "end-handle");

      // Handles waypoints (orange)
      if (ann.waypoints) {
        ann.waypoints.forEach((wp, i) => {
          const lp = toLocal(wp, ann);
          addHandle(c, lp.x, lp.y, 0xff8c00, `wp-${i}`);
        });
      }

      // Handles "+" au milieu de chaque segment (pour ajouter un waypoint)
      for (let i = 0; i < n - 1; i++) {
        const p1 = localPts[i];
        const p2 = localPts[i + 1];
        const mx = (p1.x + p2.x) / 2;
        const my = (p1.y + p2.y) / 2;
        addMidHandle(c, mx, my, `mid-${i}`);
      }
    }
  }

  private buildSticky(c: Container, ann: Annotation, selected: boolean) {
    if (ann.sourceFile) { this.buildSourceSticky(c, ann, selected); return; }

    const w = ann.width  ?? 160;
    const h = ann.height ?? 120;
    const bg = parseInt((ann.bgColor ?? "#f5c542").replace("#", ""), 16) || 0xf5c542;
    const g = new Graphics();
    g.rect(0, 0, w, h);
    g.fill({ color: bg, alpha: 0.92 });
    if (selected) {
      g.rect(-2, -2, w + 4, h + 4);
      g.stroke({ color: 0xffffff, width: 1.5, alpha: 0.9 });
    }
    c.hitArea = new Rectangle(0, 0, w, h);
    c.addChild(g);

    const SCALE = 3;
    const label = ann.text
      ? new Text({ text: ann.text, style: { fontSize: (ann.fontSize ?? 13) * SCALE, fill: 0x222222, wordWrap: true, wordWrapWidth: (w - 16) * SCALE, breakWords: true } })
      : new Text({ text: "Cliquer pour écrire...", style: { fontSize: 10 * SCALE, fill: 0x886633 } });
    label.scale.set(1 / SCALE);
    label.x = 8; label.y = 8;
    if (!ann.text) label.alpha = 0.5;
    c.addChild(label);

    if (selected) {
      addResizeHandle(c, w, h, "br");
      addResizeHandle(c, 0, h, "bl");
      addResizeHandle(c, w, 0, "tr");
      addResizeHandle(c, 0, 0, "tl");
    }
  }

  private buildSourceSticky(c: Container, ann: Annotation, selected: boolean) {
    const w = ann.width  ?? 220;
    const h = ann.height ?? 80;
    const SCALE = 3;

    const ext    = (ann.sourceFile?.split(".").pop() ?? "").toUpperCase();
    const name   = ann.sourceFile?.split(/[\\/]/).pop() ?? ann.text ?? "";
    const extCol = parseInt((ann.bgColor ?? "#1a1a2e").replace("#", ""), 16) || 0x1a1a2e;

    // ── Lueur de fond (couches concentriques, du plus large au plus proche) ──
    // Simule un glow ambiant de la couleur de l'extension
    const glowLayers = [
      { pad: 28, alpha: 0.04 },
      { pad: 18, alpha: 0.08 },
      { pad: 10, alpha: 0.13 },
      { pad:  4, alpha: 0.20 },
    ];
    for (const { pad, alpha } of glowLayers) {
      const glow = new Graphics();
      glow.rect(-pad, -pad, w + pad * 2, h + pad * 2)
          .fill({ color: extCol, alpha });
      c.addChild(glow);
    }

    // ── Corps principal ────────────────────────────────────────────────
    const g = new Graphics();
    g.rect(0, 0, w, h).fill({ color: 0x0e0e1a, alpha: 0.97 });
    // Barre accent gauche (couleur extension)
    g.rect(0, 0, 4, h).fill({ color: extCol, alpha: 1 });
    // Bordure fine de la même couleur
    g.rect(0, 0, w, h).stroke({ color: extCol, width: 0.8, alpha: 0.35 });
    if (selected) g.rect(-2, -2, w + 4, h + 4).stroke({ color: 0x60a5fa, width: 1.5, alpha: 0.9 });
    c.hitArea = new Rectangle(-28, -28, w + 56, h + 56); // hitArea inclut la lueur
    c.addChild(g);

    // ── Badge extension ────────────────────────────────────────────────
    const badgeBg = new Graphics();
    badgeBg.rect(10, 8, 38, 16).fill({ color: extCol, alpha: 0.9 });
    c.addChild(badgeBg);
    const extLabel = new Text({
      text: ext.slice(0, 6),
      style: new TextStyle({ fontSize: 8 * SCALE, fill: 0xffffff, fontWeight: "bold", fontFamily: "system-ui,sans-serif" }),
    });
    extLabel.scale.set(1 / SCALE); extLabel.x = 12; extLabel.y = 9;
    c.addChild(extLabel);

    // ── Nom du fichier ─────────────────────────────────────────────────
    const nameLabel = new Text({
      text: name,
      style: new TextStyle({ fontSize: 10 * SCALE, fill: 0xcccccc, wordWrap: true, wordWrapWidth: (w - 18) * SCALE, breakWords: true, fontFamily: "system-ui,sans-serif" }),
    });
    nameLabel.scale.set(1 / SCALE); nameLabel.x = 10; nameLabel.y = 30;
    c.addChild(nameLabel);

    // ── Hint ───────────────────────────────────────────────────────────
    const hint = new Text({
      text: "↩ double-clic pour ouvrir",
      style: new TextStyle({ fontSize: 7 * SCALE, fill: extCol, fontFamily: "system-ui,sans-serif" }),
    });
    hint.scale.set(1 / SCALE); hint.x = 10; hint.y = h - 13; hint.alpha = 0.6;
    c.addChild(hint);
  }

  private buildText(c: Container, ann: Annotation, selected: boolean) {
    const col = parseInt((ann.color ?? "#ffffff").replace("#", ""), 16) || 0xffffff;
    const SCALE = 3;
    const t = new Text({
      text: ann.text || "Aa",
      style: { fontSize: (ann.fontSize ?? 14) * SCALE, fill: col },
    });
    t.scale.set(1 / SCALE);
    c.hitArea = new Rectangle(-8, -8, Math.max(48, t.width + 16), Math.max(24, t.height + 16));
    if (selected) {
      const g = new Graphics();
      g.rect(-4, -4, Math.max(40, t.width) + 8, Math.max(20, t.height) + 8);
      g.stroke({ color: 0xffffff, width: 1, alpha: 0.5 });
      c.addChild(g);
    }
    c.addChild(t);
  }

  // ── Events ───────────────────────────────────────────────────
  private attachEvents(c: Container, ann: Annotation, onSelect: OnSelect, onEdit: OnEdit) {
    c.on("pointerdown", (e: FederatedPointerEvent) => {
      if (e.button !== 0) return;
      if (useGlucoseStore.getState().activeTool !== "select") return;
      e.stopPropagation();
      useGlucoseStore.getState().pushHistory();

      const world = this.getWorld();
      if (!world) return;
      const wx = (e.globalX - world.x) / world.scale.x;
      const wy = (e.globalY - world.y) / world.scale.y;
      const part = (e.target as any)?._handlePart ?? "body";

      // Resize handle sticky → commencer le redimensionnement
      if (part.startsWith("resize-")) {
        onSelect(ann.id, false);
        this.dragState = {
          id: ann.id, part,
          startX: ann.x, startY: ann.y,
          startX2: ann.x2 ?? ann.x + 100, startY2: ann.y2 ?? ann.y,
          startWidth: ann.width ?? 160,
          startHeight: ann.height ?? 120,
          pStartX: wx, pStartY: wy,
          didMove: false, downTime: Date.now(),
        };
        return;
      }

      // Clic sur handle "+" → ajouter un waypoint et commencer à le dragger
      if (part.startsWith("mid-")) {
        const segIdx = parseInt(part.slice(4));
        const worldPts = arrowWorldPoints(ann);
        const p1 = worldPts[segIdx];
        const p2 = worldPts[segIdx + 1];
        const midX = (p1.x + p2.x) / 2;
        const midY = (p1.y + p2.y) / 2;
        const wps = [...(ann.waypoints ?? [])];
        wps.splice(segIdx, 0, { x: midX, y: midY });
        const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
        useGlucoseStore.getState().updateAnnotation(boardId, ann.id, { waypoints: wps });
        // Démarre un drag sur le nouveau waypoint
        this.dragState = {
          id: ann.id, part: `wp-${segIdx}`,
          startX: midX, startY: midY,
          startX2: ann.x2 ?? ann.x + 100, startY2: ann.y2 ?? ann.y,
          pStartX: wx, pStartY: wy,
          didMove: false, downTime: Date.now(),
        };
        onSelect(ann.id, false);
        return;
      }

      onSelect(ann.id, e.ctrlKey || e.metaKey || e.shiftKey);

      // Pour les waypoints, startX/Y = position monde originale du waypoint
      let startX = ann.x;
      let startY = ann.y;
      if (part.startsWith("wp-")) {
        const idx = parseInt(part.slice(3));
        const wp = ann.waypoints?.[idx];
        if (wp) { startX = wp.x; startY = wp.y; }
      }

      this.dragState = {
        id: ann.id, part,
        startX, startY,
        startX2: ann.x2 ?? ann.x + 100, startY2: ann.y2 ?? ann.y,
        pStartX: wx, pStartY: wy,
        didMove: false, downTime: Date.now(),
      };
    });

    c.on("pointerup", (e: FederatedPointerEvent) => {
      const ds = this.dragState;
      if (!ds || ds.id !== ann.id) return;
      const wasDrag = ds.didMove;
      const part    = ds.part;
      this.dragState = null;

      // Ctrl+clic sur un waypoint → le supprimer
      if (!wasDrag && part.startsWith("wp-") && (e.ctrlKey || e.metaKey)) {
        const idx = parseInt(part.slice(3));
        const wps = [...(ann.waypoints ?? [])];
        wps.splice(idx, 1);
        const boardId = getActiveBoard(useGlucoseStore.getState().project).id;
        useGlucoseStore.getState().updateAnnotation(boardId, ann.id, { waypoints: wps });
        return;
      }

      // Clic court sur texte/sticky → ouvrir éditeur (pas si on cliquait un handle resize)
      if (!wasDrag && !part.startsWith("resize-") && Date.now() - ds.downTime <= 500) {
        if (ann.type === "text" || ann.type === "sticky") onEdit(ann.id);
      }
    });

    c.on("pointerupoutside", () => { this.dragState = null; });
  }

  clearDrag()  { this.dragState = null; }
  destroy()    { this.container.destroy({ children: true }); }
}

// ── Helpers dessin ────────────────────────────────────────────

function drawPath(g: Graphics, pts: { x: number; y: number }[], curved: boolean) {
  if (pts.length < 2) return;
  g.moveTo(pts[0].x, pts[0].y);
  if (!curved || pts.length === 2) {
    for (let i = 1; i < pts.length; i++) g.lineTo(pts[i].x, pts[i].y);
  } else {
    // Catmull-Rom → Bézier cubique
    const p = [
      { x: 2 * pts[0].x - pts[1].x, y: 2 * pts[0].y - pts[1].y },
      ...pts,
      { x: 2 * pts[pts.length - 1].x - pts[pts.length - 2].x, y: 2 * pts[pts.length - 1].y - pts[pts.length - 2].y },
    ];
    for (let i = 1; i < p.length - 2; i++) {
      const p0 = p[i - 1], p1 = p[i], p2 = p[i + 1], p3 = p[i + 2];
      const cp1x = p1.x + (p2.x - p0.x) / 6;
      const cp1y = p1.y + (p2.y - p0.y) / 6;
      const cp2x = p2.x - (p3.x - p1.x) / 6;
      const cp2y = p2.y - (p3.y - p1.y) / 6;
      g.bezierCurveTo(cp1x, cp1y, cp2x, cp2y, p2.x, p2.y);
    }
  }
}

/*
function getTangentAngle(
  pts: { x: number; y: number }[],
  _ptIdx: number,
  role: "start" | "end",
): number {
  const n = pts.length;
  if (n < 2) return 0;
  if (role === "end") {
    const a = pts[n - 2]; const b = pts[n - 1];
    return Math.atan2(b.y - a.y, b.x - a.x);
  }
  const a = pts[0]; const b = pts[1];
  return Math.atan2(a.y - b.y, a.x - b.x);
}
*/

/*
function drawArrowhead(g: Graphics, col: number, sw: number, x: number, y: number, angle: number) {
  const head = Math.max(10, sw * 5);
  g.moveTo(x, y);
  g.lineTo(x - head * Math.cos(angle - 0.4), y - head * Math.sin(angle - 0.4));
  g.moveTo(x, y);
  g.lineTo(x - head * Math.cos(angle + 0.4), y - head * Math.sin(angle + 0.4));
  g.stroke({ color: col, width: sw, alpha: 0.9 });
}
*/

function addHandle(parent: Container, x: number, y: number, col: number, part: string) {
  const h = new Graphics();
  h.circle(0, 0, 14); h.fill({ color: 0xffffff, alpha: 0.01 });  // zone de clic large
  h.circle(0, 0, 6);  h.fill({ color: col, alpha: 0.9 });
  h.stroke({ color: 0x000000, width: 1.5, alpha: 0.5 });
  h.x = x; h.y = y;
  h.interactive = true; h.cursor = "crosshair";
  (h as any)._handlePart = part;
  parent.addChild(h);
}

function addResizeHandle(parent: Container, x: number, y: number, corner: string) {
  const h = new Graphics();
  h.circle(0, 0, 12); h.fill({ color: 0xffffff, alpha: 0.01 });
  h.rect(-4, -4, 8, 8); h.fill({ color: 0xffffff, alpha: 0.85 });
  h.stroke({ color: 0x000000, width: 1, alpha: 0.4 });
  h.x = x; h.y = y;
  h.interactive = true;
  h.cursor = (corner === "br" || corner === "tl") ? "nwse-resize" : "nesw-resize";
  (h as any)._handlePart = `resize-${corner}`;
  parent.addChild(h);
}

function addMidHandle(parent: Container, x: number, y: number, part: string) {
  const h = new Graphics();
  h.circle(0, 0, 16); h.fill({ color: 0xffffff, alpha: 0.01 });  // zone de clic invisible
  // Losange visible petit et discret
  h.poly([{ x: 0, y: -5 }, { x: 5, y: 0 }, { x: 0, y: 5 }, { x: -5, y: 0 }]);
  h.fill({ color: 0xffffff, alpha: 0.5 });
  h.stroke({ color: 0xffffff, width: 1, alpha: 0.8 });
  h.x = x; h.y = y;
  h.interactive = true; h.cursor = "copy";
  (h as any)._handlePart = part;
  parent.addChild(h);
}
