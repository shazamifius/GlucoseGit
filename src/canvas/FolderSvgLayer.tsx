import { useEffect, useRef, useState } from "react";
import { Board, CanvasFolder } from "../types";
import { useGlucoseStore } from "../store";

interface Props {
  folders: CanvasFolder[];
  boards: Board[];
  selectedId: string | null;
  vpRef: React.MutableRefObject<{ x: number; y: number; scale: number }>;
  onSelect: (id: string | null) => void;
  onEnter: (id: string) => void;
  onMove: (id: string, x: number, y: number) => void;
  onResize: (id: string, w: number, h: number) => void;
  onRename: (id: string, name: string) => void;
}

interface DragState {
  id: string;
  startX: number; startY: number;
  pStartX: number; pStartY: number;
  startW: number; startH: number;
  didMove: boolean;
  t0: number;
  mode: "move" | "resize";
}

export default function FolderSvgLayer({
  folders, boards, selectedId, vpRef,
  onSelect, onEnter, onMove, onResize, onRename,
}: Props) {
  const svgRef    = useRef<SVGSVGElement>(null);
  const groupRef  = useRef<SVGGElement>(null);
  const dragRef   = useRef<DragState | null>(null);
  const lastClickRef = useRef<{ id: string; t: number }>({ id: "", t: 0 });
  const activeTool = useGlucoseStore((s) => s.activeTool);
  const lod = useGlucoseStore((s) => s.currentLod);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameVal, setRenameVal]   = useState("");

  // Sync transform with viewport
  useEffect(() => {
    const apply = (x: number, y: number, scale: number) => {
      groupRef.current?.setAttribute("transform", `translate(${x},${y}) scale(${scale})`);
    };
    const onVp = (e: Event) => {
      const { x, y, scale } = (e as CustomEvent<{ x: number; y: number; scale: number }>).detail;
      apply(x, y, scale);
    };
    window.addEventListener("glucose:viewport-changed", onVp);
    const { x, y, scale } = vpRef.current;
    apply(x, y, scale);
    return () => window.removeEventListener("glucose:viewport-changed", onVp);
  }, []);

  function screenToWorld(clientX: number, clientY: number) {
    const rect = svgRef.current!.getBoundingClientRect();
    const vp = vpRef.current;
    return {
      x: (clientX - rect.left - vp.x) / vp.scale,
      y: (clientY - rect.top  - vp.y) / vp.scale,
    };
  }

  function startDrag(folder: CanvasFolder, e: React.PointerEvent, mode: "move" | "resize") {
    if (activeTool !== "select") return;
    e.stopPropagation();
    useGlucoseStore.getState().pushHistory();
    const { x: wx, y: wy } = screenToWorld(e.clientX, e.clientY);
    dragRef.current = {
      id: folder.id,
      startX: folder.x, startY: folder.y,
      startW: folder.width, startH: folder.height,
      pStartX: wx, pStartY: wy,
      didMove: false, t0: Date.now(),
      mode,
    };

    function onGlobalMove(ev: PointerEvent) {
      const ds = dragRef.current;
      if (!ds) return;
      const { x: wx2, y: wy2 } = screenToWorld(ev.clientX, ev.clientY);
      const dx = wx2 - ds.pStartX;
      const dy = wy2 - ds.pStartY;
      if (Math.abs(dx) + Math.abs(dy) > 2) ds.didMove = true;
      if (!ds.didMove) return;
      if (ds.mode === "move") {
        onMove(ds.id, ds.startX + dx, ds.startY + dy);
      } else {
        onResize(ds.id, Math.max(180, ds.startW + dx), Math.max(120, ds.startH + dy));
      }
    }

    function onGlobalUp() {
      const ds = dragRef.current;
      if (ds && !ds.didMove && Date.now() - ds.t0 < 400 && ds.mode === "move") {
        // Détecter double-clic
        const now = Date.now();
        const isDbl = lastClickRef.current.id === ds.id && now - lastClickRef.current.t < 400;
        lastClickRef.current = { id: ds.id, t: now };
        if (isDbl) {
          onEnter(ds.id);
        } else {
          onSelect(ds.id);
        }
      }
      dragRef.current = null;
      window.removeEventListener("pointermove", onGlobalMove);
      window.removeEventListener("pointerup",   onGlobalUp);
    }

    window.addEventListener("pointermove", onGlobalMove);
    window.addEventListener("pointerup",   onGlobalUp);
  }

  function forwardWheel(e: React.WheelEvent) {
    const canvas = document.querySelector("canvas");
    if (!canvas) return;
    const evt = new WheelEvent("wheel", {
      deltaX: e.deltaX, deltaY: e.deltaY, deltaZ: e.deltaZ, deltaMode: e.deltaMode,
      clientX: e.clientX, clientY: e.clientY,
      ctrlKey: e.ctrlKey, shiftKey: e.shiftKey, altKey: e.altKey, metaKey: e.metaKey,
      bubbles: false, cancelable: true,
    });
    canvas.dispatchEvent(evt);
    e.preventDefault();
  }

  function startRename(folder: CanvasFolder) {
    setRenamingId(folder.id);
    const child = boards.find((b) => b.id === folder.childBoardId);
    setRenameVal(child?.name || folder.name || "");
  }

  function commitRename(folder: CanvasFolder) {
    const v = renameVal.trim();
    if (v) onRename(folder.id, v);
    setRenamingId(null);
  }

  return (
    <svg
      ref={svgRef}
      onWheel={forwardWheel}
      style={{
        position: "absolute", inset: 0,
        width: "100%", height: "100%",
        pointerEvents: "none",
        zIndex: 1, // sous les annotations (zIndex 2)
        overflow: "visible",
      }}
    >
      <g ref={groupRef}>
        {folders.map((folder) => {
          const child = boards.find((b) => b.id === folder.childBoardId) ?? null;
          const sel   = selectedId === folder.id;
          const W = folder.width;
          const H = folder.height;
          const HEADER = 38;
          const RADIUS = 14;
          const col = folder.color;
          const total = child ? child.images.length + child.annotations.length + (child.folders ?? []).length : 0;
          const displayName = child?.name || folder.name || "Sans titre";
          const interactive = activeTool === "select";

          return (
            <g
              key={folder.id}
              transform={`translate(${folder.x},${folder.y})`}
              style={{ pointerEvents: interactive ? "all" : "none" }}
            >
              {/* Glow externe quand sélectionné */}
              {sel && (
                <rect
                  x={-3} y={-3} width={W + 6} height={H + 6}
                  rx={RADIUS + 3}
                  fill="none" stroke={col} strokeOpacity={0.35}
                  strokeWidth={1.5}
                  vectorEffect="non-scaling-stroke"
                />
              )}

              {/* Fond — zone draggable */}
              <rect
                width={W} height={H} rx={RADIUS}
                fill={col} fillOpacity={0.04}
                stroke={col} strokeOpacity={sel ? 0.55 : 0.22}
                strokeWidth={sel ? 1.5 : 1}
                vectorEffect="non-scaling-stroke"
                style={{ cursor: interactive ? "grab" : "default" }}
                onPointerDown={(e) => startDrag(folder, e, "move")}
              />

              {/* Icône dossier vectorielle (SVG path propre) */}
              <g transform={`translate(12, 11)`} style={{ pointerEvents: "none" }}>
                <path
                  d="M0 3 Q0 1 2 1 L7 1 L9 3 L14 3 Q16 3 16 5 L16 13 Q16 15 14 15 L2 15 Q0 15 0 13 Z"
                  fill={col} fillOpacity={0.75}
                />
              </g>

              {/* Nom du dossier (caché en macro pour réduire la densité visuelle) */}
              {lod !== "macro" && (renamingId === folder.id ? (
                <foreignObject x={34} y={8} width={W - 80} height={24}>
                  <input
                    // @ts-ignore
                    xmlns="http://www.w3.org/1999/xhtml"
                    autoFocus
                    value={renameVal}
                    onChange={(e) => setRenameVal(e.target.value)}
                    onBlur={() => commitRename(folder)}
                    onKeyDown={(e) => {
                      e.stopPropagation();
                      if (e.key === "Enter") commitRename(folder);
                      if (e.key === "Escape") setRenamingId(null);
                    }}
                    onClick={(e) => e.stopPropagation()}
                    onPointerDown={(e) => e.stopPropagation()}
                    style={{
                      width: "100%", height: "100%",
                      background: "rgba(0,0,0,0.5)", color: "#fff",
                      border: `1px solid ${col}`, borderRadius: 3,
                      padding: "1px 6px", fontSize: 13,
                      fontFamily: "system-ui, sans-serif", fontWeight: 600,
                      outline: "none", letterSpacing: 0.3,
                    }}
                  />
                </foreignObject>
              ) : (
                <text
                  x={34} y={HEADER / 2 + 4}
                  fill={col} fillOpacity={0.95}
                  fontSize={14} fontWeight="600"
                  fontFamily="system-ui, -apple-system, sans-serif"
                  letterSpacing={0.3}
                  style={{ pointerEvents: interactive ? "all" : "none", cursor: interactive ? "text" : "default", userSelect: "none" }}
                  onDoubleClick={(e) => { e.stopPropagation(); if (interactive) startRename(folder); }}
                >
                  {displayName.length > 28 ? displayName.slice(0, 25) + "…" : displayName}
                </text>
              ))}

              {/* Badge compteur */}
              {total > 0 && (
                <g transform={`translate(${W - 38}, ${HEADER / 2 - 8})`} style={{ pointerEvents: "none" }}>
                  <rect width={30} height={16} rx={8}
                    fill={col} fillOpacity={0.18}
                    stroke={col} strokeOpacity={0.4} strokeWidth={0.8}
                    vectorEffect="non-scaling-stroke" />
                  <text x={15} y={11.5}
                    textAnchor="middle"
                    fill={col} fillOpacity={0.95}
                    fontSize={10} fontWeight="600"
                    fontFamily="system-ui, sans-serif">
                    {total}
                  </text>
                </g>
              )}

              {/* Badge miroir ↻ (Phase 4) — coin haut-gauche au-dessus du cadre.
                  Indique que ce dossier est un alias d'un original (changements propagés). */}
              {folder.mirrorOf && (
                <g
                  transform={`translate(-10, -10)`}
                  style={{ cursor: interactive ? "pointer" : "default", pointerEvents: interactive ? "all" : "none" }}
                  onPointerDown={(e) => e.stopPropagation()}
                  onClick={(e) => {
                    e.stopPropagation();
                    window.dispatchEvent(new CustomEvent("glucose:teleport-to-mirror-original", {
                      detail: { mirrorOf: folder.mirrorOf, type: "folder" },
                    }));
                  }}
                >
                  <title>Dossier-miroir → cliquer pour aller à l'original</title>
                  <circle cx={11} cy={11} r={11}
                    fill="rgba(15,15,25,0.9)" stroke="#93c5fd" strokeOpacity={0.6}
                    strokeWidth={1.5} vectorEffect="non-scaling-stroke" />
                  <text x={11} y={15.5} textAnchor="middle"
                    fill="#93c5fd" fontSize={13} fontWeight="600"
                    fontFamily="system-ui, sans-serif"
                    style={{ pointerEvents: "none", userSelect: "none" }}>↻</text>
                </g>
              )}

              {/* Preview du contenu (cachée en macro) */}
              {lod !== "macro" && <FolderPreview folder={folder} child={child} headerH={HEADER} />}

              {/* Hint vide */}
              {lod !== "macro" && total === 0 && (
                <text x={W / 2} y={HEADER + (H - HEADER) / 2}
                  textAnchor="middle" dominantBaseline="middle"
                  fill="#444" fontSize={11}
                  fontFamily="system-ui, sans-serif"
                  style={{ pointerEvents: "none" }}>
                  Double-clic pour ouvrir
                </text>
              )}

              {/* Handle resize bas-droit (visible quand sélectionné) */}
              {sel && interactive && (
                <g transform={`translate(${W - 16}, ${H - 16})`}>
                  <rect width={16} height={16}
                    fill="transparent"
                    style={{ cursor: "nwse-resize", pointerEvents: "all" }}
                    onPointerDown={(e) => startDrag(folder, e, "resize")} />
                  <line x1={4} y1={14} x2={14} y2={4}
                    stroke={col} strokeOpacity={0.7} strokeWidth={1.5}
                    vectorEffect="non-scaling-stroke"
                    style={{ pointerEvents: "none" }} />
                  <line x1={9} y1={14} x2={14} y2={9}
                    stroke={col} strokeOpacity={0.4} strokeWidth={1}
                    vectorEffect="non-scaling-stroke"
                    style={{ pointerEvents: "none" }} />
                </g>
              )}
            </g>
          );
        })}
      </g>
    </svg>
  );
}

function FolderPreview({ folder, child, headerH }: {
  folder: CanvasFolder; child: Board | null; headerH: number;
}) {
  if (!child) return null;
  const W = folder.width;
  const H = folder.height;
  const PAD = 10;
  const items = [
    ...child.images.map((img) => ({
      type: "img" as const,
      x: img.x - img.width / 2, y: img.y - img.height / 2,
      w: img.width, h: img.height, color: "#ffffff",
    })),
    ...child.annotations.filter((a) => a.type !== "arrow").map((ann) => ({
      type: "ann" as const,
      x: ann.x, y: ann.y,
      w: ann.width ?? 80, h: ann.height ?? 20,
      color: ann.bgColor ?? ann.color ?? "#888888",
    })),
    ...(child.folders ?? []).map((f) => ({
      type: "fld" as const,
      x: f.x, y: f.y, w: f.width, h: f.height, color: f.color,
    })),
  ];
  if (items.length === 0) return null;

  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const it of items) {
    minX = Math.min(minX, it.x);
    minY = Math.min(minY, it.y);
    maxX = Math.max(maxX, it.x + it.w);
    maxY = Math.max(maxY, it.y + it.h);
  }
  const cw = Math.max(1, maxX - minX);
  const ch = Math.max(1, maxY - minY);
  const drawW = W - PAD * 2;
  const drawH = H - headerH - PAD * 2;
  const sc = Math.min(drawW / cw, drawH / ch) * 0.88;
  const offX = PAD + (drawW - cw * sc) / 2;
  const offY = headerH + PAD + (drawH - ch * sc) / 2;

  const clipId = `clip-${folder.id}`;
  return (
    <g style={{ pointerEvents: "none" }}>
      <defs>
        <clipPath id={clipId}>
          <rect x={0} y={headerH} width={W} height={H - headerH} />
        </clipPath>
      </defs>
      <g clipPath={`url(#${clipId})`}>
        {items.slice(0, 60).map((it, i) => {
          const px = (it.x - minX) * sc + offX;
          const py = (it.y - minY) * sc + offY;
          const pw = it.w * sc;
          const ph = it.h * sc;
          if (it.type === "img") {
            return <rect key={i} x={px} y={py} width={pw} height={ph}
              fill={it.color} fillOpacity={0.10}
              stroke={it.color} strokeOpacity={0.22} strokeWidth={0.5}
              vectorEffect="non-scaling-stroke" />;
          }
          if (it.type === "fld") {
            return <rect key={i} x={px} y={py} width={pw} height={ph} rx={2}
              fill={it.color} fillOpacity={0.08}
              stroke={it.color} strokeOpacity={0.32} strokeWidth={0.5}
              vectorEffect="non-scaling-stroke" />;
          }
          return <rect key={i} x={px} y={py} width={Math.max(2, pw)} height={Math.max(2, ph)}
            fill={it.color} fillOpacity={0.15} />;
        })}
      </g>
    </g>
  );
}
