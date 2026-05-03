import { useRef, useEffect, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useGlucoseStore, getActiveBoard } from "../store";

const MAP_W = 180;
const MAP_H = 120;
const PAD   = 12;

interface MapInfo {
  minX: number; minY: number;
  scale: number; offX: number; offY: number;
}

export default function Minimap() {
  const canvasRef   = useRef<HTMLCanvasElement>(null);
  const mapInfoRef  = useRef<MapInfo | null>(null);
  const liveVpRef   = useRef<{ x: number; y: number; scale: number } | null>(null);
  const isDragging       = useRef(false);
  const worldPosRef      = useRef({ wx: 0, wy: 0 });
  const lastPosRef       = useRef({ x: 0, y: 0 });
  const clickDownRef     = useRef({ x: 0, y: 0 });
  const clickWorldRef    = useRef({ wx: 0, wy: 0 });

  // CLEANUP P-08 — Selector atomique au lieu de subscribe au store entier
  const project = useGlucoseStore(s => s.project);
  const board = getActiveBoard(project);
  // Phase 4 — décale la minimap à gauche quand un panel droit est ouvert
  // (panel droit largeur 320px + petit gap 12px = right: 344)
  const rightPanelOpen = useGlucoseStore((s) => s.rightPanelOpen);

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.clearRect(0, 0, MAP_W, MAP_H);
    ctx.fillStyle = "rgba(13,13,13,0.92)";
    ctx.fillRect(0, 0, MAP_W, MAP_H);

    const imgs = board.images;
    const hasContent = imgs.length > 0 || board.annotations.length > 0 || (board.folders ?? []).length > 0;
    if (!hasContent) {
      mapInfoRef.current = null;
      ctx.strokeStyle = "#2a2a2a";
      ctx.lineWidth = 1;
      ctx.strokeRect(0.5, 0.5, MAP_W - 1, MAP_H - 1);
      return;
    }

    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    imgs.forEach((img) => {
      minX = Math.min(minX, img.x - img.width  / 2);
      minY = Math.min(minY, img.y - img.height / 2);
      maxX = Math.max(maxX, img.x + img.width  / 2);
      maxY = Math.max(maxY, img.y + img.height / 2);
    });
    board.annotations.forEach((ann) => {
      const w = ann.width ?? (ann.type === "text" ? 80 : ann.type === "sticky" ? 160 : 0);
      const h = ann.height ?? (ann.type === "text" ? 20 : ann.type === "sticky" ? 120 : 0);
      minX = Math.min(minX, ann.x); minY = Math.min(minY, ann.y);
      maxX = Math.max(maxX, ann.x + w); maxY = Math.max(maxY, ann.y + h);
      
      if (ann.type === "arrow") {
        const p2x = ann.x2 ?? ann.x;
        const p2y = ann.y2 ?? ann.y;
        minX = Math.min(minX, p2x); minY = Math.min(minY, p2y);
        maxX = Math.max(maxX, p2x); maxY = Math.max(maxY, p2y);
      }
    });
    (board.folders ?? []).forEach((f) => {
      minX = Math.min(minX, f.x); minY = Math.min(minY, f.y);
      maxX = Math.max(maxX, f.x + f.width); maxY = Math.max(maxY, f.y + f.height);
    });

    const cw = maxX - minX || 1;
    const ch = maxY - minY || 1;
    const innerW = MAP_W - PAD * 2;
    const innerH = MAP_H - PAD * 2;
    const scale  = Math.min(innerW / cw, innerH / ch);
    const offX   = PAD + (innerW - cw * scale) / 2;
    const offY   = PAD + (innerH - ch * scale) / 2;

    mapInfoRef.current = { minX, minY, scale, offX, offY };

    function toMap(wx: number, wy: number) {
      return { x: offX + (wx - minX) * scale, y: offY + (wy - minY) * scale };
    }

    imgs.forEach((img) => {
      const tl = toMap(img.x - img.width / 2, img.y - img.height / 2);
      ctx.fillStyle = img.locked ? "#3a2a1a" : "#2a2a2a";
      ctx.fillRect(tl.x, tl.y, Math.max(1, img.width * scale), Math.max(1, img.height * scale));
      ctx.strokeStyle = img.locked ? "#f87171" : "#444";
      ctx.lineWidth = 0.5;
      ctx.strokeRect(tl.x, tl.y, Math.max(1, img.width * scale), Math.max(1, img.height * scale));
    });

    const getAnchor = (refId?: string, fallbackX: number = 0, fallbackY: number = 0) => {
      if (!refId) return { x: fallbackX, y: fallbackY };
      const refAnn = board.annotations.find(a => a.id === refId);
      if (refAnn) {
        const w = refAnn.width ?? (refAnn.type === "text" ? 80 : 160);
        const h = refAnn.height ?? (refAnn.type === "text" ? 20 : 120);
        return { x: refAnn.x + w / 2, y: refAnn.y + h / 2 };
      }
      const refImg = board.images.find(i => i.id === refId);
      if (refImg) return { x: refImg.x, y: refImg.y };
      return { x: fallbackX, y: fallbackY };
    };

    board.annotations.forEach((ann) => {
      const p = toMap(ann.x, ann.y);
      if (ann.type === "sticky") {
        ctx.fillStyle = ann.bgColor ?? "#f5c542";
        ctx.fillRect(p.x, p.y, Math.max(1, (ann.width ?? 160) * scale), Math.max(1, (ann.height ?? 120) * scale));
      } else if (ann.type === "arrow") {
        const pStartWorld = getAnchor(ann.sourceId, ann.x, ann.y);
        const pEndWorld = getAnchor(ann.targetId, ann.x2 ?? ann.x, ann.y2 ?? ann.y);
        const p1 = toMap(pStartWorld.x, pStartWorld.y);
        const p2 = toMap(pEndWorld.x, pEndWorld.y);
        ctx.beginPath(); ctx.moveTo(p1.x, p1.y); 
        
        if (ann.waypoints && ann.waypoints.length > 0) {
          ann.waypoints.forEach(wp => {
            const m = toMap(wp.x, wp.y);
            ctx.lineTo(m.x, m.y);
          });
        }
        
        ctx.lineTo(p2.x, p2.y);
        ctx.strokeStyle = ann.color ?? "#888"; ctx.lineWidth = 1.5; ctx.stroke();
      } else if (ann.type === "text") {
        ctx.fillStyle = ann.color ? ann.color + "80" : "rgba(255,255,255,0.5)";
        ctx.fillRect(p.x, p.y, Math.max(1, (ann.width ?? 80) * scale), Math.max(1, (ann.height ?? 20) * scale));
      }
    });

    // Folders
    (board.folders ?? []).forEach((f) => {
      const tl = toMap(f.x, f.y);
      const fw = f.width * scale; const fh = f.height * scale;
      const col = f.color ?? "#60a5fa";
      ctx.fillStyle = col + "18";
      ctx.fillRect(tl.x, tl.y, fw, fh);
      ctx.strokeStyle = col;
      ctx.lineWidth = 0.8;
      ctx.setLineDash([2, 2]);
      ctx.strokeRect(tl.x, tl.y, fw, fh);
      ctx.setLineDash([]);
    });

    // Viewport indicator — live ref for real-time tracking
    const vp = liveVpRef.current ?? board.viewport;
    if (vp) {
      const screenW = window.innerWidth;
      const screenH = window.innerHeight - 80;
      const vpScale = vp.scale || 1;
      const vpX = -vp.x / vpScale, vpY = -vp.y / vpScale;
      const tl = toMap(vpX, vpY);
      const br = toMap(vpX + screenW / vpScale, vpY + screenH / vpScale);
      ctx.strokeStyle = "rgba(255,255,255,0.35)"; ctx.lineWidth = 1;
      ctx.strokeRect(tl.x, tl.y, br.x - tl.x, br.y - tl.y);
      ctx.fillStyle = "rgba(255,255,255,0.04)";
      ctx.fillRect(tl.x, tl.y, br.x - tl.x, br.y - tl.y);
    }

    ctx.strokeStyle = "#2a2a2a"; ctx.lineWidth = 1;
    ctx.strokeRect(0.5, 0.5, MAP_W - 1, MAP_H - 1);
  }, [board]);

  useEffect(() => { draw(); }, [draw]);

  // Real-time viewport — bypass React render cycle
  useEffect(() => {
    function onVpChanged(e: Event) {
      const { x, y, scale } = (e as CustomEvent<{ x: number; y: number; scale: number }>).detail;
      liveVpRef.current = { x, y, scale };
      draw();
    }
    window.addEventListener("glucose:viewport-changed", onVpChanged);
    return () => window.removeEventListener("glucose:viewport-changed", onVpChanged);
  }, [draw]);

  // Native DOM pointer handlers — bypasses React synthetic event delegation
  // so setPointerCapture works correctly when cursor leaves the minimap canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    function onDown(e: PointerEvent) {
      e.preventDefault();

      // Always capture + start drag immediately, before any info guard
      isDragging.current = true;
      lastPosRef.current = { x: e.clientX, y: e.clientY };
      clickDownRef.current = { x: e.clientX, y: e.clientY };
      canvas!.setPointerCapture(e.pointerId);
      getCurrentWindow().setCursorGrab(true).catch(() => {});

      const info = mapInfoRef.current;
      if (!info) return;

      const rect = canvas!.getBoundingClientRect();
      const clickWx = info.minX + (e.clientX - rect.left - info.offX) / info.scale;
      const clickWy = info.minY + (e.clientY - rect.top  - info.offY) / info.scale;
      clickWorldRef.current = { wx: clickWx, wy: clickWy };

      // Seed worldPosRef from current viewport center so first move delta is zero
      const vp = liveVpRef.current;
      if (vp && vp.scale) {
        worldPosRef.current = {
          wx: (window.innerWidth  / 2 - vp.x) / vp.scale,
          wy: (window.innerHeight / 2 - vp.y) / vp.scale,
        };
      } else {
        worldPosRef.current = { wx: clickWx, wy: clickWy };
      }
    }

    function onMove(e: PointerEvent) {
      if (!isDragging.current) return;
      if (e.buttons === 0) { isDragging.current = false; return; }
      const info = mapInfoRef.current;
      if (!info) return;

      const dx = e.clientX - lastPosRef.current.x;
      const dy = e.clientY - lastPosRef.current.y;
      lastPosRef.current = { x: e.clientX, y: e.clientY };

      worldPosRef.current.wx += dx / info.scale;
      worldPosRef.current.wy += dy / info.scale;
      window.dispatchEvent(new CustomEvent("glucose:pan-viewport-to", {
        detail: { wx: worldPosRef.current.wx, wy: worldPosRef.current.wy },
      }));
    }

    function onUp(e: PointerEvent) {
      if (!isDragging.current) return;
      isDragging.current = false;
      canvas!.releasePointerCapture(e.pointerId);
      getCurrentWindow().setCursorGrab(false).catch(() => {});

      const dx = e.clientX - clickDownRef.current.x;
      const dy = e.clientY - clickDownRef.current.y;
      if (Math.sqrt(dx * dx + dy * dy) < 5) {
        window.dispatchEvent(new CustomEvent("glucose:jump-viewport", {
          detail: clickWorldRef.current,
        }));
      }
    }

    canvas.addEventListener("pointerdown", onDown);
    canvas.addEventListener("pointermove", onMove);
    canvas.addEventListener("pointerup", onUp);
    return () => {
      canvas.removeEventListener("pointerdown", onDown);
      canvas.removeEventListener("pointermove", onMove);
      canvas.removeEventListener("pointerup", onUp);
    };
  }, []);

  const isEmpty = board.images.length === 0 && board.annotations.length === 0 && (board.folders ?? []).length === 0;

  return (
    <div style={{
      position: "absolute", bottom: 12,
      right: rightPanelOpen ? 332 : 12,
      transition: "right 0.18s ease-out",
      width: MAP_W, height: MAP_H,
      borderRadius: 4, opacity: 0.85,
      zIndex: 1000,
    }}>
      <canvas
        ref={canvasRef}
        width={MAP_W}
        height={MAP_H}
        title="Minimap — cliquer/glisser pour naviguer"
        style={{ display: "block", cursor: "crosshair", borderRadius: 4 }}
      />
      {isEmpty && (
        <div style={{
          position: "absolute", inset: 0,
          display: "flex", alignItems: "center", justifyContent: "center",
          pointerEvents: "none",
          color: "#444",
          fontSize: 10,
          fontFamily: "system-ui, sans-serif",
          letterSpacing: "0.04em",
        }}>
          Canvas vide
        </div>
      )}
    </div>
  );
}
