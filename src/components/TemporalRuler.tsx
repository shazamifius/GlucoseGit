// ────────────────────────────────────────────────────────────────────────────
// Phase 6 — Réglette Temporelle Sémantique (composant UI)
// ────────────────────────────────────────────────────────────────────────────
// Réglette zoomable en bas du canvas. Deux poignées draggables définissent
// la fenêtre [start, end] du filtre temporel (store.temporalFilter).
//
// Interactions :
//   • Drag poignée gauche/droite  → modifie start/end
//   • Drag entre les poignées      → translate la fenêtre
//   • Wheel sur la réglette        → zoome l'échelle (autour du curseur)
//   • Bouton ⛌                     → désactive le filtre (filter = null)
//
// Les nœuds avec `temporalAnchor` qui n'intersectent pas le filtre sont
// atténués par les couches de rendu (opacity 0.12). Les nœuds atemporels
// restent toujours visibles.

import { useEffect, useMemo, useRef, useState } from "react";
import { useGlucoseStore } from "../store";
import {
  DEFAULT_ERAS,
  YEAR_MIN, YEAR_MAX,
  formatYear, tickStep,
} from "../utils/timeline";

interface Props {
  onClose: () => void;
}

const RULER_HEIGHT = 88;
const HANDLE_W = 14;

export default function TemporalRuler({ onClose }: Props) {
  const filter = useGlucoseStore((s) => s.temporalFilter);
  const setFilter = useGlucoseStore((s) => s.setTemporalFilter);

  // Échelle visible (window.start..window.end). Indépendante du filtre lui-même
  // pour pouvoir zoomer sans changer la sélection.
  const [view, setView] = useState<{ start: number; end: number }>({ start: -1000, end: 2050 });
  const ref = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(800);

  // Initialise le filtre à la 1re ouverture si vide
  useEffect(() => {
    if (!filter) setFilter({ start: view.start, end: view.end });
    // on ne dépend volontairement que du mount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Mesure la largeur réelle du conteneur
  useEffect(() => {
    if (!ref.current) return;
    const ro = new ResizeObserver((entries) => {
      for (const e of entries) setWidth(e.contentRect.width);
    });
    ro.observe(ref.current);
    return () => ro.disconnect();
  }, []);

  // Conversions année ↔ pixel
  const span = view.end - view.start;
  const yearToPx = (y: number) => ((y - view.start) / span) * width;
  const pxToYear = (px: number) => view.start + (px / width) * span;

  // ── Drag des poignées et de la fenêtre ────────────────────────────────
  const dragRef = useRef<{ kind: "start" | "end" | "window"; startPx: number; orig: { start: number; end: number } } | null>(null);

  function startDrag(kind: "start" | "end" | "window") {
    return (e: React.MouseEvent) => {
      e.stopPropagation();
      e.preventDefault();
      const cur = filter ?? { start: view.start, end: view.end };
      dragRef.current = { kind, startPx: e.clientX, orig: { ...cur } };
      const onMove = (m: MouseEvent) => {
        if (!dragRef.current) return;
        const dx = m.clientX - dragRef.current.startPx;
        const dy = (dx / width) * span;
        const o = dragRef.current.orig;
        let next = { ...o };
        if (dragRef.current.kind === "start") {
          next.start = Math.min(o.end - 1, Math.max(YEAR_MIN, Math.round(o.start + dy)));
        } else if (dragRef.current.kind === "end") {
          next.end = Math.max(o.start + 1, Math.min(YEAR_MAX, Math.round(o.end + dy)));
        } else {
          const len = o.end - o.start;
          let s = Math.round(o.start + dy);
          if (s < YEAR_MIN) s = YEAR_MIN;
          if (s + len > YEAR_MAX) s = YEAR_MAX - len;
          next = { start: s, end: s + len };
        }
        setFilter(next);
      };
      const onUp = () => {
        dragRef.current = null;
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
      };
      window.addEventListener("mousemove", onMove);
      window.addEventListener("mouseup", onUp);
    };
  }

  // ── Zoom à la molette (centré sur le curseur) ─────────────────────────
  function onWheel(e: React.WheelEvent) {
    e.preventDefault();
    const rect = ref.current?.getBoundingClientRect();
    if (!rect) return;
    const px = e.clientX - rect.left;
    const yearAtCursor = pxToYear(px);
    const factor = e.deltaY > 0 ? 1.15 : 1 / 1.15; // déZoom si on scroll vers le bas
    const newSpan = Math.min(YEAR_MAX - YEAR_MIN, Math.max(2, span * factor));
    // ratio constant : (yearAtCursor - newStart) / newSpan == px / width
    const newStart = Math.max(YEAR_MIN, Math.min(YEAR_MAX - newSpan, yearAtCursor - (px / width) * newSpan));
    setView({ start: Math.round(newStart), end: Math.round(newStart + newSpan) });
  }

  // ── Graduations ───────────────────────────────────────────────────────
  const ticks = useMemo(() => {
    const step = tickStep(span);
    const first = Math.ceil(view.start / step) * step;
    const out: { y: number; px: number; major: boolean }[] = [];
    for (let y = first; y <= view.end; y += step) {
      out.push({ y, px: yearToPx(y), major: (y / step) % 5 === 0 });
    }
    return out;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [view.start, view.end, width]);

  // ── Bandes des époques visibles ───────────────────────────────────────
  const erasInView = useMemo(() => {
    return DEFAULT_ERAS.filter((e) => e.end >= view.start && e.start <= view.end)
      .map((e) => {
        const a = Math.max(e.start, view.start);
        const b = Math.min(e.end, view.end);
        return {
          name: e.name,
          left: yearToPx(a),
          width: Math.max(2, yearToPx(b) - yearToPx(a)),
          duration: e.end - e.start,
        };
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [view.start, view.end, width]);

  const f = filter ?? view;
  const handleLeftPx = yearToPx(f.start);
  const handleRightPx = yearToPx(f.end);

  return (
    <div
      ref={ref}
      onWheel={onWheel}
      style={{
        position: "absolute", left: 16, right: 16, bottom: 16,
        height: RULER_HEIGHT,
        background: "linear-gradient(180deg, rgba(20,20,24,0.92), rgba(15,15,18,0.92))",
        border: "1px solid #2a2a2a",
        borderRadius: 8,
        boxShadow: "0 6px 24px rgba(0,0,0,0.5)",
        userSelect: "none",
        zIndex: 50,
        backdropFilter: "blur(6px)",
        WebkitBackdropFilter: "blur(6px)",
      }}
    >
      {/* Header : bornes courantes + close */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center",
                    padding: "4px 10px", color: "#9ca3af", fontSize: 10, letterSpacing: 0.4 }}>
        <span>RÉGLETTE TEMPORELLE · molette pour zoomer · drag des poignées</span>
        <span style={{ display: "flex", gap: 12, alignItems: "center" }}>
          <span style={{ color: "#fde68a" }}>
            {formatYear(f.start)} → {formatYear(f.end)}
          </span>
          <button
            onClick={(e) => { e.stopPropagation(); setFilter(null); onClose(); }}
            title="Désactiver le filtre temporel"
            style={{
              background: "transparent", border: "1px solid #444", color: "#aaa",
              borderRadius: 4, fontSize: 11, padding: "2px 8px", cursor: "pointer",
            }}
          >
            ⛌
          </button>
        </span>
      </div>

      {/* Bande des époques nommées */}
      <div style={{ position: "relative", height: 18, marginInline: 8, marginTop: 2 }}>
        {erasInView.map((e, i) => (
          <div
            key={`${e.name}-${i}`}
            title={e.name}
            style={{
              position: "absolute", top: 0,
              left: e.left, width: e.width, height: 16,
              background: `hsla(${(hashHue(e.name)) % 360}, 38%, 38%, 0.55)`,
              borderRadius: 2,
              fontSize: 9, color: "#e5e7eb",
              padding: "1px 4px", overflow: "hidden", whiteSpace: "nowrap",
              textOverflow: "ellipsis",
              border: "1px solid rgba(255,255,255,0.05)",
            }}
          >
            {e.width > 40 ? e.name : ""}
          </div>
        ))}
      </div>

      {/* Piste principale + ticks + sélection + poignées */}
      <div style={{ position: "relative", height: 36, marginInline: 8, marginTop: 4,
                    background: "rgba(255,255,255,0.025)", borderRadius: 4, overflow: "hidden" }}>
        {/* Ticks */}
        {ticks.map((t) => (
          <div key={t.y} style={{ position: "absolute", left: t.px - 0.5, top: 0, bottom: 0,
                                  width: 1, background: t.major ? "#3a3a3a" : "#2a2a2a" }} />
        ))}
        {ticks.filter((t) => t.major).map((t) => (
          <div key={`lbl-${t.y}`} style={{
            position: "absolute", left: t.px + 2, top: 1,
            fontSize: 9, color: "#888", pointerEvents: "none", whiteSpace: "nowrap",
          }}>{formatYear(t.y)}</div>
        ))}

        {/* Bande de sélection (entre les deux poignées) */}
        <div
          onMouseDown={startDrag("window")}
          style={{
            position: "absolute", left: handleLeftPx, width: Math.max(0, handleRightPx - handleLeftPx),
            top: 0, bottom: 0,
            background: "rgba(253, 224, 71, 0.16)",
            borderTop: "1px solid #fde68a55",
            borderBottom: "1px solid #fde68a55",
            cursor: "grab",
          }}
        />

        {/* Poignée gauche */}
        <div
          onMouseDown={startDrag("start")}
          title={`Début : ${formatYear(f.start)}`}
          style={{
            position: "absolute", left: handleLeftPx - HANDLE_W / 2, top: 0, bottom: 0,
            width: HANDLE_W, cursor: "ew-resize",
            background: "linear-gradient(180deg, #fde68a, #fbbf24)",
            borderRadius: 2,
            boxShadow: "0 0 6px rgba(253,224,71,0.4)",
          }}
        />
        {/* Poignée droite */}
        <div
          onMouseDown={startDrag("end")}
          title={`Fin : ${formatYear(f.end)}`}
          style={{
            position: "absolute", left: handleRightPx - HANDLE_W / 2, top: 0, bottom: 0,
            width: HANDLE_W, cursor: "ew-resize",
            background: "linear-gradient(180deg, #fde68a, #fbbf24)",
            borderRadius: 2,
            boxShadow: "0 0 6px rgba(253,224,71,0.4)",
          }}
        />
      </div>
    </div>
  );
}

// hash trivial pour donner une couleur stable à chaque époque
function hashHue(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;
  return Math.abs(h);
}
