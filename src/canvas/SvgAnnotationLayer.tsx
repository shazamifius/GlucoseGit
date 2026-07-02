import { useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Annotation, MembraneAnnotation } from "../types";
import { useGlucoseStore } from "../store";
import { showToast } from "../components/Toast";

import { toAbsolute } from "../utils/pathResolver";

function openSourceFile(path: string) {
  const absPath = toAbsolute(path);
  const name = absPath.split(/[\\/]/).pop() || absPath;
  window.dispatchEvent(new CustomEvent("glucose:app-launching", { detail: { path: absPath } }));
  showToast(`Ouverture de ${name}…`, "🚀");
  invoke("open_in_app", { path: absPath }).catch((err) => {
    showToast(`Impossible d'ouvrir : ${String(err)}`, "⚠️");
  });
}

const textMeasureCanvas = document.createElement("canvas");
const textMeasureCtx = textMeasureCanvas.getContext("2d")!;

export function measureTextSize(text: string, fontSize: number) {
  textMeasureCtx.font = `${fontSize}px system-ui, sans-serif`;
  const lines = (text || "Aa").split("\n");
  let maxW = 0;
  for (const line of lines) {
    maxW = Math.max(maxW, textMeasureCtx.measureText(line || "Aa").width);
  }
  return { w: maxW + 8, h: lines.length * fontSize * 1.4 + 8 };
}

interface Props {
  annotations: Annotation[];
  selectedIds: string[];
  editingId: string | null;
  vpRef: React.MutableRefObject<{ x: number; y: number; scale: number }>;
  onSelect: (id: string, multi: boolean) => void;
  onEdit: (id: string) => void;
  onResize: (id: string, x: number, y: number, w: number, h: number) => void;
}

interface DragState {
  id: string;
  startX: number; startY: number;
  pStartX: number; pStartY: number;
  didMove: boolean;
  t0: number;
  corner?: string;
  startW?: number; startH?: number;
}

export default function SvgAnnotationLayer({
  annotations, selectedIds, editingId, vpRef,
  onSelect, onEdit, onResize,
}: Props) {
  const svgRef   = useRef<SVGSVGElement>(null);
  const groupRef = useRef<SVGGElement>(null);
  const dragRef  = useRef<DragState | null>(null);
  const lastClickRef = useRef<{ id: string; time: number } | null>(null);
  const activeTool = useGlucoseStore((s) => s.activeTool);

  // Synchroniser le transform SVG avec le viewport PixiJS (sans re-render React)
  useEffect(() => {
    const apply = (x: number, y: number, scale: number) => {
      groupRef.current?.setAttribute("transform", `translate(${x},${y}) scale(${scale})`);
    };
    const onVp = (e: Event) => {
      const { x, y, scale } = (e as CustomEvent<{ x: number; y: number; scale: number }>).detail;
      apply(x, y, scale);
    };
    window.addEventListener("glucose:viewport-changed", onVp);
    // Appliquer immédiatement la valeur courante
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

  function startDrag(ann: Annotation, e: React.PointerEvent, corner?: string) {
    if (activeTool !== "select") return;
    e.stopPropagation();
    // UNDO-1 — pas de snapshot au down ; la transaction s'ouvre au 1er mouvement
    // réel (cf. didMove) → un clic ne crée aucune entrée, un drag/resize = 1 entrée.
    const { x: wx, y: wy } = screenToWorld(e.clientX, e.clientY);
    // width/height n'existent pas sur les flèches (qui utilisent x2/y2).
    const annW = ann.type === "arrow" ? 160 : (ann.width ?? 160);
    const annH = ann.type === "arrow" ? 120 : (ann.height ?? 120);
    dragRef.current = {
      id: ann.id,
      startX: ann.x, startY: ann.y,
      pStartX: wx, pStartY: wy,
      didMove: false, t0: Date.now(),
      corner, startW: annW, startH: annH,
    };

    function onGlobalMove(ev: PointerEvent) {
      const ds = dragRef.current;
      if (!ds) return;
      const { x: wx2, y: wy2 } = screenToWorld(ev.clientX, ev.clientY);
      const dx = wx2 - ds.pStartX;
      const dy = wy2 - ds.pStartY;
      if (Math.abs(dx) + Math.abs(dy) > 2 && !ds.didMove) {
        ds.didMove = true;
        useGlucoseStore.getState().beginLiveEdit(); // UNDO-1 — ouvre la transaction au 1er vrai mouvement
      }
      if (!ds.didMove) return;

      if (ds.corner) {
        const sw = ds.startW ?? 160;
        const sh = ds.startH ?? 120;
        let nx = ds.startX, ny = ds.startY, nw = sw, nh = sh;
        if (ds.corner === "br") { nw = Math.max(60, sw + dx); nh = Math.max(40, sh + dy); }
        else if (ds.corner === "bl") { nw = Math.max(60, sw - dx); nh = Math.max(40, sh + dy); nx = ds.startX + (sw - nw); }
        else if (ds.corner === "tr") { nw = Math.max(60, sw + dx); nh = Math.max(40, sh - dy); ny = ds.startY + (sh - nh); }
        else if (ds.corner === "tl") { nw = Math.max(60, sw - dx); nh = Math.max(40, sh - dy); nx = ds.startX + (sw - nw); ny = ds.startY + (sh - nh); }
        onResize(ds.id, nx, ny, nw, nh);
      } else {
        const currentDX = wx2 - ds.pStartX;
        const currentDY = wy2 - ds.pStartY;
        ds.pStartX = wx2;
        ds.pStartY = wy2;
        const boardId = useGlucoseStore.getState().activeBoardId;
        useGlucoseStore.getState().moveSelected(boardId, currentDX, currentDY);
      }
    }

    function onGlobalUp(ev: PointerEvent) {
      const ds = dragRef.current;
      if (ds && !ds.didMove && Date.now() - ds.t0 < 500 && !ds.corner) {
        const multi = ev.ctrlKey || ev.metaKey || ev.shiftKey;
        if (!multi) {
          // Si on a cliqué sans bouger sur un élément déjà sélectionné,
          // on désélectionne les autres au relâchement du clic.
          onSelect(ds.id, false);
        }
      }
      useGlucoseStore.getState().endLiveEdit(); // UNDO-1 — referme la transaction (no-op si simple clic)
      dragRef.current = null;
      window.removeEventListener("pointermove", onGlobalMove);
      window.removeEventListener("pointerup",   onGlobalUp);
    }

    window.addEventListener("pointermove", onGlobalMove);
    window.addEventListener("pointerup",   onGlobalUp);
  }

  function handleDown(ann: Annotation, e: React.PointerEvent, corner?: string) {
    if (e.button !== 0) return;
    if (activeTool !== "select") return;
    e.stopPropagation();

    const now = Date.now();
    if (lastClickRef.current?.id === ann.id && now - lastClickRef.current.time < 350) {
      lastClickRef.current = null;
      // App Bridge : seuls les sticky portent un sourceFile
      const sourceFile = ann.type === "sticky" ? ann.sourceFile : undefined;
      if (sourceFile) {
        openSourceFile(sourceFile);
      } else {
        onEdit(ann.id);
      }
      return;
    }
    lastClickRef.current = { id: ann.id, time: now };

    const isSelected = selectedIds.includes(ann.id);
    const multi = e.ctrlKey || e.metaKey || e.shiftKey;

    if (multi) {
      onSelect(ann.id, true);
    } else if (!isSelected) {
      onSelect(ann.id, false);
    }

    startDrag(ann, e, corner);
  }

  function handleDblClick(ann: Annotation, e: React.MouseEvent) {
    // Strict : édition uniquement en mode select
    if (activeTool !== "select") return;
    // Si un drag vient juste de se terminer, ne pas ouvrir l'édition
    if (dragRef.current?.didMove) return;
    e.stopPropagation();
    const sourceFile = ann.type === "sticky" ? ann.sourceFile : undefined;
    if (sourceFile) {
      openSourceFile(sourceFile);
      return;
    }
    onEdit(ann.id);
  }

  // Forward wheel events to the canvas underneath so zoom always works
  // (even when the cursor is over a sticky/text/membrane).
  function forwardWheel(e: React.WheelEvent) {
    const canvas = document.querySelector("canvas");
    if (!canvas) return;
    const evt = new WheelEvent("wheel", {
      deltaX: e.deltaX, deltaY: e.deltaY, deltaZ: e.deltaZ, deltaMode: e.deltaMode,
      clientX: e.clientX, clientY: e.clientY,
      ctrlKey: e.ctrlKey, shiftKey: e.shiftKey, altKey: e.altKey, metaKey: e.metaKey,
      bubbles: false, cancelable: true,
    });
    // `wheelDeltaY` (legacy) n'est pas recopié par le constructeur → sans ça
    // `classifyWheel` confondrait un cran de molette avec un pan 2 doigts et la
    // vue défilerait au lieu de zoomer quand on scrolle sur l'annotation.
    const nativeWd = (e.nativeEvent as unknown as { wheelDeltaY?: number }).wheelDeltaY;
    if (typeof nativeWd === "number") {
      Object.defineProperty(evt, "wheelDeltaY", { value: nativeWd, configurable: true });
    }
    canvas.dispatchEvent(evt);
    e.preventDefault();
  }

  return (
    <svg
      ref={svgRef}
      onWheel={forwardWheel}
      style={{
        position: "absolute", inset: 0,
        width: "100%", height: "100%",
        pointerEvents: "none",
        zIndex: 2,
        overflow: "visible",
      }}
    >
      <defs>
        <filter id="membrane-glow" x="-50%" y="-50%" width="200%" height="200%">
          <feGaussianBlur stdDeviation="25" result="blur" />
          <feComposite in="SourceGraphic" in2="blur" operator="over" />
        </filter>
      </defs>
      <g ref={groupRef}>
        {annotations.map((ann) => {
          if (ann.id === editingId) return null;
          const sel = selectedIds.includes(ann.id);

          if (ann.type === "membrane") {
            return renderMembrane(ann, sel);
          }

          return null;
        })}
      </g>
    </svg>
  );

  function renderMembrane(ann: MembraneAnnotation, sel: boolean) {
    const w   = ann.width;
    const h   = ann.height;
    const col = ann.color  ?? "#60a5fa";
    const HANDLE = 7;
    const corners: Array<[string, number, number]> = [
      ["br", w, h], ["bl", 0, h], ["tr", w, 0], ["tl", 0, 0],
    ];
    return (
      <g
        key={ann.id}
        transform={`translate(${ann.x},${ann.y})`}
        style={{ pointerEvents: activeTool === "select" ? "all" : "none", cursor: activeTool === "select" ? "move" : "default" }}
        onPointerDown={(e) => handleDown(ann, e)}
        onDoubleClick={(e) => handleDblClick(ann, e)}
      >
        {/* Glow ultra-discret — quasi imperceptible, ne voile jamais le texte */}
        {[{p:20,a:0.012}, {p:10,a:0.018}].map(({p,a}) => (
          <rect key={p} x={-p} y={-p} width={w + p * 2} height={h + p * 2}
                fill={col} fillOpacity={a} rx={60} style={{ pointerEvents: "none" }} filter="url(#membrane-glow)" />
        ))}
        {/* Fill — quasi invisible : la zone se lit par sa bordure + son label,
            jamais par un remplissage qui masque le texte qu'elle contient */}
        <rect width={w} height={h} fill={col} fillOpacity={0.02} rx={60} />
        {/* Border — frontière nette (porte l'identité de la région) */}
        <rect width={w} height={h} fill="none"
              stroke={col} strokeWidth={2} strokeOpacity={sel ? 0.9 : 0.45}
              strokeDasharray={sel ? "none" : "10 10"}
              rx={60} vectorEffect="non-scaling-stroke" />
        {/* Label — nom du thème, lisible (halo sombre) sur n'importe quel fond */}
        {ann.text && (
          <text x={16} y={-12} fill={col} fontSize={16} fontWeight={600}
                fontFamily="system-ui, sans-serif"
                stroke="#0b0b12" strokeWidth={4} strokeLinejoin="round"
                style={{ paintOrder: "stroke" }}>
            {ann.text}
          </text>
        )}
        {sel && corners.map(([corner, cx, cy]) => (
          <rect
            key={corner}
            x={cx - HANDLE / 2} y={cy - HANDLE / 2}
            width={HANDLE} height={HANDLE}
            fill="#1a1a1a" stroke={col} strokeWidth={1}
            vectorEffect="non-scaling-stroke"
            style={{ pointerEvents: "all", cursor: `${corner}-resize` }}
            onPointerDown={(e) => { e.stopPropagation(); handleDown(ann, e, corner); }}
          />
        ))}
      </g>
    );
  }

}
