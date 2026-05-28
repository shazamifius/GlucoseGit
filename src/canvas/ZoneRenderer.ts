import { Container, Graphics, Text, TextStyle, FederatedPointerEvent, Rectangle } from "pixi.js";
import { Board, BoardZone, Preset, PresetSlot } from "../types";
import { useGlucoseStore } from "../store";

// Graphics tagués avec des marqueurs custom pour le hit-test des poignées /
// boutons (Pixi ne typant pas les propriétés ad-hoc, on s'en évite des `any`).
interface MarkedGraphics extends Graphics {
  _handlePart?: string;
  _zoneAction?: "delete";
}

interface ZoneDragState {
  slotId:  string;
  part:    string;   // "body" | "resize-br/bl/tr/tl"
  startX:  number; startY: number;
  startW:  number; startH: number;
  pStartX: number; pStartY: number;
  didMove: boolean;
  liveX:   number; liveY: number; liveW: number; liveH: number;
}

type OnZonesChange = (zones: BoardZone[]) => void;

export class ZoneRenderer {
  private container:      Container;
  private zones           = new Map<string, Container>();
  private selectedSlotId: string | null  = null;
  private dragState:      ZoneDragState | null = null;
  private currentZones:   BoardZone[]    = [];
  private currentPreset:  Preset | null  = null;
  private onZonesChange:  OnZonesChange | null = null;
  private getWorld:       () => Container | null;

  constructor(parent: Container, getWorld: () => Container | null) {
    this.container = new Container();
    this.container.label = "zones";
    parent.addChildAt(this.container, 0);
    this.getWorld = getWorld;
  }

  update(board: Board, preset: Preset | null, onZonesChange?: OnZonesChange) {
    this.onZonesChange = onZonesChange ?? null;
    this.currentZones  = board.zones;
    this.currentPreset = preset;

    // Supprime les zones qui n'existent plus
    const currentSlotIds = new Set(board.zones.map((z) => z.slotId));
    this.zones.forEach((cont, slotId) => {
      if (!currentSlotIds.has(slotId)) {
        this.container.removeChild(cont);
        cont.destroy({ children: true });
        this.zones.delete(slotId);
      }
    });

    if (!preset || !board.zones.length) {
      this.container.removeChildren();
      this.zones.clear();
      return;
    }

    board.zones.forEach((zone) => {
      const slot = preset.slots.find((s) => s.id === zone.slotId);
      if (!slot) return;
      const sel      = this.selectedSlotId === zone.slotId;
      const existing = this.zones.get(zone.slotId);

      if (existing) {
        // Ne pas réinitialiser la position si drag en cours
        if (!this.dragState || this.dragState.slotId !== zone.slotId) {
          existing.x = zone.x;
          existing.y = zone.y;
          this.rebuildZoneGfx(existing, zone, slot, sel);
        }
      } else {
        const cont = new Container();
        cont.interactive = true;
        cont.cursor = "grab";
        cont.x = zone.x;
        cont.y = zone.y;
        this.rebuildZoneGfx(cont, zone, slot, sel);
        this.attachZoneEvents(cont, zone.slotId);
        this.container.addChild(cont);
        this.zones.set(zone.slotId, cont);
      }
    });
  }

  // ── Appelé depuis stage.pointermove ──────────────────────────
  handleGlobalMove(e: FederatedPointerEvent, world: Container) {
    const ds = this.dragState;
    if (!ds) return;
    const wx = (e.globalX - world.x) / world.scale.x;
    const wy = (e.globalY - world.y) / world.scale.y;
    const dx = wx - ds.pStartX;
    const dy = wy - ds.pStartY;
    if (Math.abs(dx) > 2 || Math.abs(dy) > 2) ds.didMove = true;
    if (!ds.didMove) return;

    const cont = this.zones.get(ds.slotId);
    if (!cont) return;

    if (ds.part === "body") {
      cont.x = ds.startX + dx;
      cont.y = ds.startY + dy;
      ds.liveX = cont.x;
      ds.liveY = cont.y;
    } else if (ds.part.startsWith("resize-")) {
      const corner = ds.part.slice(7);
      let newX = ds.startX, newY = ds.startY, newW = ds.startW, newH = ds.startH;
      if      (corner === "br") { newW = Math.max(100, ds.startW + dx); newH = Math.max(80, ds.startH + dy); }
      else if (corner === "bl") { newW = Math.max(100, ds.startW - dx); newH = Math.max(80, ds.startH + dy); newX = ds.startX + (ds.startW - newW); }
      else if (corner === "tr") { newW = Math.max(100, ds.startW + dx); newH = Math.max(80, ds.startH - dy); newY = ds.startY + (ds.startH - newH); }
      else if (corner === "tl") { newW = Math.max(100, ds.startW - dx); newH = Math.max(80, ds.startH - dy); newX = ds.startX + (ds.startW - newW); newY = ds.startY + (ds.startH - newH); }

      cont.x = newX; cont.y = newY;
      ds.liveX = newX; ds.liveY = newY; ds.liveW = newW; ds.liveH = newH;

      const slot = this.currentPreset?.slots.find((s) => s.id === ds.slotId);
      if (slot) {
        const tmpZone: BoardZone = { slotId: ds.slotId, x: newX, y: newY, width: newW, height: newH };
        this.rebuildZoneGfx(cont, tmpZone, slot, true);
      }
    }
  }

  // ── Appelé depuis stage.pointerup / pointerupoutside ─────────
  clearDragState() {
    const ds = this.dragState;
    this.dragState = null;
    if (!ds?.didMove) return;

    const cont  = this.zones.get(ds.slotId);
    const zones = this.currentZones.map((z) => {
      if (z.slotId !== ds.slotId) return z;
      if (ds.part === "body") return { ...z, x: cont?.x ?? z.x, y: cont?.y ?? z.y };
      return { ...z, x: ds.liveX, y: ds.liveY, width: ds.liveW, height: ds.liveH };
    });
    this.onZonesChange?.(zones);
  }

  // ── Déselectionne quand on clique ailleurs ───────────────────
  deselect() {
    if (this.selectedSlotId === null) return;
    this.selectedSlotId = null;
    this.onZonesChange?.([...this.currentZones]);
  }

  destroy() { this.container.destroy({ children: true }); }

  // ── Private ──────────────────────────────────────────────────

  private rebuildZoneGfx(c: Container, zone: BoardZone, slot: PresetSlot, selected: boolean) {
    while (c.children.length > 0) {
      const ch = c.removeChildAt(0) as Container;
      ch.destroy({ children: true });
    }
    const { width: w, height: h } = zone;
    const color = parseInt(slot.color.replace("#", ""), 16);
    c.hitArea = new Rectangle(0, 0, w, h);

    // Fond
    const bg = new Graphics();
    bg.rect(0, 0, w, h).fill({ color: 0x111111, alpha: 0.5 });
    c.addChild(bg);

    // Bordure dashed
    const border = new Graphics();
    const dashLen = 8, gap = 6;
    for (let x = 0; x < w; x += dashLen + gap) border.rect(x, 0, Math.min(dashLen, w - x), 1);
    for (let x = 0; x < w; x += dashLen + gap) border.rect(x, h - 1, Math.min(dashLen, w - x), 1);
    for (let y = 0; y < h; y += dashLen + gap) border.rect(0, y, 1, Math.min(dashLen, h - y));
    for (let y = 0; y < h; y += dashLen + gap) border.rect(w - 1, y, 1, Math.min(dashLen, h - y));
    border.fill({ color, alpha: selected ? 0.75 : 0.4 });
    c.addChild(border);

    // Contour blanc si sélectionné
    if (selected) {
      const sel = new Graphics();
      sel.rect(-2, -2, w + 4, h + 4).stroke({ color: 0xffffff, width: 1.5, alpha: 0.5 });
      c.addChild(sel);
    }

    // Header coloré
    const header = new Graphics();
    header.rect(0, 0, w, 36).fill({ color, alpha: 0.15 });
    c.addChild(header);
    const headerLine = new Graphics();
    headerLine.rect(0, 0, w, 2).fill({ color, alpha: 0.8 });
    c.addChild(headerLine);

    // Titre
    const title = new Text({
      text: slot.name.toUpperCase(),
      style: new TextStyle({ fontFamily: "system-ui, sans-serif", fontSize: 13, fontWeight: "600", fill: slot.color, letterSpacing: 1 }),
    });
    title.x = 12; title.y = 10;
    c.addChild(title);

    // Description
    const desc = new Text({
      text: slot.description,
      style: new TextStyle({ fontFamily: "system-ui, sans-serif", fontSize: 11, fill: "#555555", wordWrap: true, wordWrapWidth: w - 24 }),
    });
    desc.x = 12; desc.y = h / 2 - 10;
    c.addChild(desc);

    if (selected) {
      addZoneResizeHandle(c, w, h, "br");
      addZoneResizeHandle(c, 0, h, "bl");
      addZoneResizeHandle(c, w, 0, "tr");
      addZoneResizeHandle(c, 0, 0, "tl");
      addZoneDeleteButton(c, w);
    }
  }

  private attachZoneEvents(c: Container, slotId: string) {
    c.on("pointerdown", (e: FederatedPointerEvent) => {
      if (e.button !== 0) return;
      if (useGlucoseStore.getState().activeTool !== "select") return;
      e.stopPropagation();
      const world = this.getWorld();
      if (!world) return;
      const wx = (e.globalX - world.x) / world.scale.x;
      const wy = (e.globalY - world.y) / world.scale.y;

      // Bouton supprimer
      const tgt = e.target as MarkedGraphics | null;
      if (tgt?._zoneAction === "delete") {
        const zones = this.currentZones.filter((z) => z.slotId !== slotId);
        this.selectedSlotId = null;
        this.onZonesChange?.(zones);
        return;
      }

      const part = tgt?._handlePart ?? "body";
      const zone = this.currentZones.find((z) => z.slotId === slotId);
      if (!zone) return;

      // Premier clic = sélection uniquement, pas de drag
      // Deuxième clic sur la zone déjà sélectionnée = drag autorisé
      if (this.selectedSlotId !== slotId) {
        this.selectedSlotId = slotId;
        this.onZonesChange?.([...this.currentZones]);
        return; // Pas de drag au premier clic
      }

      this.dragState = {
        slotId, part,
        startX: c.x, startY: c.y,
        startW: zone.width, startH: zone.height,
        pStartX: wx, pStartY: wy,
        didMove: false,
        liveX: c.x, liveY: c.y, liveW: zone.width, liveH: zone.height,
      };
    });
  }
}

// ── Helpers ──────────────────────────────────────────────────────

function addZoneResizeHandle(parent: Container, x: number, y: number, corner: string) {
  const h: MarkedGraphics = new Graphics();
  h.circle(0, 0, 12).fill({ color: 0xffffff, alpha: 0.01 });
  h.rect(-4, -4, 8, 8).fill({ color: 0xffffff, alpha: 0.8 }).stroke({ color: 0x000000, width: 1, alpha: 0.4 });
  h.x = x; h.y = y;
  h.interactive = true;
  h.cursor = (corner === "br" || corner === "tl") ? "nwse-resize" : "nesw-resize";
  h._handlePart = `resize-${corner}`;
  parent.addChild(h);
}

function addZoneDeleteButton(parent: Container, w: number) {
  const btn: MarkedGraphics = new Graphics();
  btn.circle(0, 0, 10).fill({ color: 0x222222, alpha: 0.95 }).stroke({ color: 0x555555, width: 1 });
  btn.moveTo(-4, -4).lineTo(4, 4).stroke({ color: 0xff4444, width: 1.5 });
  btn.moveTo(4, -4).lineTo(-4, 4).stroke({ color: 0xff4444, width: 1.5 });
  btn.x = w - 6; btn.y = 6;
  btn.interactive = true;
  btn.cursor = "pointer";
  btn._zoneAction = "delete";
  parent.addChild(btn);
}
