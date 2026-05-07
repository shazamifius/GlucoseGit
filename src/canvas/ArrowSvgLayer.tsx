import { useEffect, useMemo, useRef } from "react";
import { Board } from "../types";
import { useGlucoseStore, getActiveBoard } from "../store";
import { getSymbioticHue } from "./HtmlAnnotationLayer";

// CLEANUP C-02 — Type explicite pour les obstacles (remplace `any[]`)
interface Obstacle {
  id: string;
  left: number;
  right: number;
  top: number;
  bottom: number;
}

// Interpolation de teinte par le chemin le plus court sur le cercle chromatique
function lerpHue(h1: number, h2: number, t: number): number {
  let diff = h2 - h1;
  if (diff > 180) diff -= 360;
  if (diff < -180) diff += 360;
  return ((h1 + diff * t) % 360 + 360) % 360;
}

function linesIntersect(a: {x:number, y:number}, b: {x:number, y:number}, c: {x:number, y:number}, d: {x:number, y:number}): boolean {
  let det = (b.x - a.x) * (d.y - c.y) - (b.y - a.y) * (d.x - c.x);
  if (det === 0) return false;
  let lambda = ((d.y - c.y) * (d.x - a.x) + (c.x - d.x) * (d.y - a.y)) / det;
  let gamma = ((a.y - b.y) * (d.x - a.x) + (b.x - a.x) * (d.y - a.y)) / det;
  return (0 < lambda && lambda < 1) && (0 < gamma && gamma < 1);
}

function lineIntersectsBox(p1: {x:number, y:number}, p2: {x:number, y:number}, box: {left:number, right:number, top:number, bottom:number}) {
  const tl = {x: box.left, y: box.top};
  const tr = {x: box.right, y: box.top};
  const bl = {x: box.left, y: box.bottom};
  const br = {x: box.right, y: box.bottom};
  
  if (linesIntersect(p1, p2, tl, tr)) return true;
  if (linesIntersect(p1, p2, tr, br)) return true;
  if (linesIntersect(p1, p2, br, bl)) return true;
  if (linesIntersect(p1, p2, bl, tl)) return true;
  
  const inside = (p: {x:number, y:number}) => p.x >= box.left && p.x <= box.right && p.y >= box.top && p.y <= box.bottom;
  if (inside(p1) || inside(p2)) return true;
  
  return false;
}

function getDynamicRoute(p1: {x:number, y:number}, p2: {x:number, y:number}, boxes: Obstacle[], visitedBoxes = new Set<string>(), depth = 0): {x:number, y:number}[] {
  if (depth > 10) return [];

  let closestBox = null;
  let minT = Infinity;

  for (const box of boxes) {
    if (visitedBoxes.has(box.id)) continue;
    if (lineIntersectsBox(p1, p2, box)) {
       let d = Math.hypot((box.left+box.right)/2 - p1.x, (box.top+box.bottom)/2 - p1.y);
       if (d < minT) { minT = d; closestBox = box; }
    }
  }

  if (!closestBox) return [];

  const PAD = 32;
  const cTL = {x: closestBox.left - PAD, y: closestBox.top - PAD};
  const cTR = {x: closestBox.right + PAD, y: closestBox.top - PAD};
  const cBL = {x: closestBox.left - PAD, y: closestBox.bottom + PAD};
  const cBR = {x: closestBox.right + PAD, y: closestBox.bottom + PAD};

  const paths = [
    [cTL], [cTR], [cBL], [cBR],
    [cTL, cTR], [cTR, cBR], [cBR, cBL], [cBL, cTL],
    [cTR, cTL], [cBR, cTR], [cBL, cBR], [cTL, cBL]
  ];

  let bestPath: {x:number, y:number}[] = [];
  let bestDist = Infinity;

  for (let path of paths) {
    let valid = true;
    let localPts = [p1, ...path, p2];
    for (let i = 0; i < localPts.length - 1; i++) {
      if (lineIntersectsBox(localPts[i], localPts[i+1], closestBox)) {
        valid = false;
        break;
      }
    }
    if (valid) {
      let d = 0;
      for (let i = 0; i < localPts.length - 1; i++) {
        d += Math.hypot(localPts[i+1].x - localPts[i].x, localPts[i+1].y - localPts[i].y);
      }
      if (d < bestDist) {
        bestDist = d;
        bestPath = path;
      }
    }
  }

  const newVisited = new Set(visitedBoxes);
  newVisited.add(closestBox.id);

  let fullRoute: {x:number, y:number}[] = [];
  let currentP = p1;
  for (let pt of bestPath) {
    fullRoute.push(...getDynamicRoute(currentP, pt, boxes, newVisited, depth + 1));
    fullRoute.push(pt);
    currentP = pt;
  }
  fullRoute.push(...getDynamicRoute(currentP, p2, boxes, newVisited, depth + 1));

  return fullRoute;
}

function forwardWheel(e: React.WheelEvent) {
  const canvas = document.querySelector("canvas");
  if (!canvas) return;
  canvas.dispatchEvent(new WheelEvent("wheel", {
    deltaX: e.deltaX, deltaY: e.deltaY, deltaZ: e.deltaZ, deltaMode: e.deltaMode,
    clientX: e.clientX, clientY: e.clientY,
    ctrlKey: e.ctrlKey, shiftKey: e.shiftKey, altKey: e.altKey, metaKey: e.metaKey,
    bubbles: false, cancelable: true,
  }));
  e.preventDefault();
}

function WaypointHandle({ wp, annId, wpIndex, vpScale }: { wp: {x: number, y: number}, annId: string, wpIndex: number, vpScale: number }) {
  const isDragging = useRef(false);

  return (
    <g
      style={{ cursor: "crosshair" }}
      onPointerDown={(e) => {
        e.stopPropagation();
        isDragging.current = true;
        e.currentTarget.setPointerCapture(e.pointerId);
      }}
      onPointerMove={(e) => {
        if (!isDragging.current) return;
        const state = useGlucoseStore.getState();
        const boardId = getActiveBoard(state.project).id;
        const ann = state.project.boards.find(b => b.id === boardId)?.annotations.find(a => a.id === annId);
        if (!ann || !ann.waypoints) return;
        const newWps = [...ann.waypoints];
        newWps[wpIndex] = { 
          x: newWps[wpIndex].x + e.movementX / vpScale, 
          y: newWps[wpIndex].y + e.movementY / vpScale 
        };
        state.updateAnnotation(boardId, annId, { waypoints: newWps });
      }}
      onPointerUp={(e) => {
        isDragging.current = false;
        e.currentTarget.releasePointerCapture(e.pointerId);
      }}
      onDoubleClick={(e) => {
        e.stopPropagation();
        const state = useGlucoseStore.getState();
        const boardId = getActiveBoard(state.project).id;
        const ann = state.project.boards.find(b => b.id === boardId)?.annotations.find(a => a.id === annId);
        if (!ann || !ann.waypoints) return;
        const newWps = [...ann.waypoints];
        newWps.splice(wpIndex, 1);
        state.updateAnnotation(boardId, annId, { waypoints: newWps });
      }}
    >
      <circle cx={wp.x} cy={wp.y} r={16 / vpScale} fill="transparent" />
      <circle cx={wp.x} cy={wp.y} r={6 / vpScale} fill="#ff8c00" stroke="#000" strokeWidth={1.5} vectorEffect="non-scaling-stroke" />
    </g>
  );
}

function MidHandle({ midX, midY, annId, insertIndex, vpScale }: { midX: number, midY: number, annId: string, insertIndex: number, vpScale: number }) {
  return (
    <g
      style={{ cursor: "copy" }}
      onPointerDown={(e) => {
        e.stopPropagation();
        const state = useGlucoseStore.getState();
        const boardId = getActiveBoard(state.project).id;
        const ann = state.project.boards.find(b => b.id === boardId)?.annotations.find(a => a.id === annId);
        if (!ann) return;
        const newWps = [...(ann.waypoints || [])];
        newWps.splice(insertIndex, 0, { x: midX, y: midY });
        state.updateAnnotation(boardId, annId, { waypoints: newWps });
      }}
    >
      <circle cx={midX} cy={midY} r={16 / vpScale} fill="transparent" />
      <polygon 
        points={`${midX},${midY - 6/vpScale} ${midX + 6/vpScale},${midY} ${midX},${midY + 6/vpScale} ${midX - 6/vpScale},${midY}`} 
        fill="rgba(255,255,255,0.7)" stroke="#ffffff" strokeWidth={1.5} vectorEffect="non-scaling-stroke" 
      />
    </g>
  );
}

interface Props {
  board: Board;
  vpRef: React.MutableRefObject<{ x: number; y: number; scale: number }>;
  editingId: string | null;
  selectedIds: string[];
  onSelect: (id: string, multi: boolean) => void;
}

export default function ArrowSvgLayer({ board, vpRef, editingId, selectedIds, onSelect }: Props) {
  const groupRef = useRef<SVGGElement>(null);
  // Phase 7.5 — LOD supprimé : les flèches sont toujours rendues. Le toggle
  // `transDomainVisible` permet juste de masquer les flèches trans-domaines.
  const transDomainVisible = useGlucoseStore(s => s.transDomainVisible);

  // CLEANUP P-06 + C-02 — Calculs lourds extraits en useMemo (évite refait O(n) à chaque render)
  const obstacles = useMemo<Obstacle[]>(() => {
    const arr: Obstacle[] = [];
    for (const a of board.annotations) {
      if (a.type !== "arrow" && a.type !== "membrane") {
        const w = a.width ?? (a.type === "text" ? 80 : 160);
        const h = a.height ?? (a.type === "text" ? 20 : 120);
        arr.push({ left: a.x, right: a.x + w, top: a.y, bottom: a.y + h, id: a.id });
      }
    }
    for (const i of board.images) {
      arr.push({
        left: i.x - i.width / 2, right: i.x + i.width / 2,
        top: i.y - i.height / 2, bottom: i.y + i.height / 2,
        id: i.id,
      });
    }
    return arr;
  }, [board.annotations, board.images]);

  const nodeDomains = useMemo(() => {
    const map = new Map<string, Set<string>>();
    for (const a of board.annotations) {
      if (a.domains?.length) {
        map.set(a.id, new Set(a.domains.filter(d => d.weight > 0.1).map(d => d.domainId)));
      }
    }
    for (const i of board.images) {
      if (i.domains?.length) {
        map.set(i.id, new Set(i.domains.filter(d => d.weight > 0.1).map(d => d.domainId)));
      }
    }
    return map;
  }, [board.annotations, board.images]);

  useEffect(() => {
    const apply = (x: number, y: number, scale: number) => {
      groupRef.current?.setAttribute("transform", `translate(${x},${y}) scale(${scale})`);
    };
    const onVp = (e: Event) => {
      const { x, y, scale } = (e as CustomEvent).detail;
      apply(x, y, scale);
    };
    window.addEventListener("glucose:viewport-changed", onVp);
    const { x, y, scale } = vpRef.current;
    apply(x, y, scale);
    return () => window.removeEventListener("glucose:viewport-changed", onVp);
  }, []);

  return (
    <svg
      style={{
        position: "absolute", inset: 0,
        width: "100%", height: "100%",
        pointerEvents: "none",
        zIndex: 3,
        overflow: "visible",
      }}
    >
      <defs>
      </defs>
      <g ref={groupRef}>
        {(() => {
          // `obstacles` et `nodeDomains` viennent maintenant des useMemo ci-dessus (P-06)
          const isTransDomain = (sourceId?: string, targetId?: string) => {
            if (!sourceId || !targetId) return false;
            const sd = nodeDomains.get(sourceId);
            const td = nodeDomains.get(targetId);
            if (!sd || !td || sd.size === 0 || td.size === 0) return false;
            // Trans-domaine si AUCUN domaine n'est commun entre source et cible
            for (const did of sd) if (td.has(did)) return false;
            return true;
          };

          return board.annotations.filter(a => a.type === "arrow").map((ann) => {
            if (ann.id === editingId) return null;
            // Phase 7.5 — LOD supprimé : flèches toujours visibles. On masque
            // seulement les trans-domaines si l'utilisateur a coupé le toggle.
            const transDomain = isTransDomain(ann.sourceId, ann.targetId);
            if (transDomain && !transDomainVisible) return null;

          const vp = vpRef.current;
          
          // Trouver la position Y du texte sélectionné dans un élément DOM
          const findTextSelPosition = (annId: string, textSel: string): { relY: number; relH: number } | null => {
            if (!textSel) return null;
            const el = document.querySelector(`[data-id="${annId}"]`) as HTMLElement;
            if (!el) return null;
            
            const selections = textSel.split(" ‖ ").map(s => s.trim()).filter(Boolean);
            const positions: { top: number; bottom: number }[] = [];
            
            for (const sel of selections) {
              const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT, null);
              let nd: Node | null;
              while ((nd = walker.nextNode())) {
                const nodeText = nd.textContent || "";
                const idx = nodeText.toLowerCase().indexOf(sel.toLowerCase());
                if (idx === -1) continue;
                
                try {
                  const range = document.createRange();
                  range.setStart(nd, idx);
                  range.setEnd(nd, idx + sel.length);
                  const rect = range.getBoundingClientRect();
                  const elRect = el.getBoundingClientRect();
                  // Position relative à l'élément (non-scalé, car offsetParent)
                  const scale = elRect.width / (el.offsetWidth || 1);
                  positions.push({
                    top: (rect.top - elRect.top) / scale,
                    bottom: (rect.bottom - elRect.top) / scale,
                  });
                } catch {}
                break;
              }
            }
            
            if (positions.length === 0) return null;
            const avgTop = positions.reduce((s, p) => s + p.top, 0) / positions.length;
            const avgBottom = positions.reduce((s, p) => s + p.bottom, 0) / positions.length;
            return { relY: (avgTop + avgBottom) / 2, relH: avgBottom - avgTop };
          };
          
          const getAnchor = (refId?: string, refBlockId?: string, fallbackX: number = 0, fallbackY: number = 0, textSel?: string) => {
            if (!refId) return { x: fallbackX, y: fallbackY };
            
            // Sub-block HTML
            if (refBlockId) {
              const el = document.querySelector(`[data-ann-id="${refId}"][data-block-id="${refBlockId}"]`) as HTMLElement;
              const parentEl = document.querySelector(`[data-id="${refId}"]`) as HTMLElement;
              const refAnn = board.annotations.find(a => a.id === refId);
              if (el && parentEl && refAnn) {
                let offsetLeft = 0;
                let offsetTop = 0;
                let curr: HTMLElement | null = el;
                while (curr && curr !== parentEl) {
                  offsetLeft += curr.offsetLeft;
                  offsetTop += curr.offsetTop;
                  curr = curr.offsetParent as HTMLElement;
                }
                const cx = offsetLeft + el.offsetWidth / 2;
                const cy = offsetTop + el.offsetHeight / 2;
                return {
                  x: refAnn.x + cx,
                  y: refAnn.y + cy,
                  box: {
                    left: refAnn.x + offsetLeft,
                    right: refAnn.x + offsetLeft + el.offsetWidth,
                    top: refAnn.y + offsetTop,
                    bottom: refAnn.y + offsetTop + el.offsetHeight
                  }
                };
              }
            }

            // Annotation entière
            const refAnn = board.annotations.find(a => a.id === refId);
            if (refAnn) {
              const w = refAnn.width ?? (refAnn.type === "text" ? 80 : 160);
              const h = refAnn.height ?? (refAnn.type === "text" ? 20 : 120);
              
              // Si une sélection de texte existe, ajuster le Y
              let anchorY = refAnn.y + h / 2;
              if (textSel) {
                const pos = findTextSelPosition(refId, textSel);
                if (pos) {
                  anchorY = refAnn.y + pos.relY;
                }
              }
              
              return { 
                x: refAnn.x + w / 2, 
                y: anchorY,
                box: {
                  left: refAnn.x,
                  right: refAnn.x + w,
                  top: refAnn.y,
                  bottom: refAnn.y + h
                }
              };
            }

            // Image
            const refImg = board.images.find(i => i.id === refId);
            if (refImg) {
              return { 
                x: refImg.x, 
                y: refImg.y,
                box: {
                  left: refImg.x - refImg.width / 2,
                  right: refImg.x + refImg.width / 2,
                  top: refImg.y - refImg.height / 2,
                  bottom: refImg.y + refImg.height / 2
                }
              };
            }

            return { x: fallbackX, y: fallbackY };
          };

          const anchorStart = getAnchor(ann.sourceId, ann.sourceBlockId, ann.x, ann.y, ann.sourceTextSel);
          const anchorEnd = getAnchor(ann.targetId, ann.targetBlockId, ann.x2 ?? ann.x + 100, ann.y2 ?? ann.y, ann.targetTextSel);

          const targetForStart = ann.waypoints?.length ? ann.waypoints[0] : anchorEnd;
          const targetForEnd = ann.waypoints?.length ? ann.waypoints[ann.waypoints.length - 1] : anchorStart;

          const pStart = { x: anchorStart.x, y: anchorStart.y };
          const pEnd = { x: anchorEnd.x, y: anchorEnd.y };

          const MARGIN = 12;

          if (anchorStart.box) {
            if (targetForStart.x < anchorStart.x) {
              pStart.x = anchorStart.box.left - MARGIN;
            } else {
              pStart.x = anchorStart.box.right + MARGIN;
            }
          }

          if (anchorEnd.box) {
            if (targetForEnd.x < anchorEnd.x) {
              pEnd.x = anchorEnd.box.left - MARGIN;
            } else {
              pEnd.x = anchorEnd.box.right + MARGIN;
            }
          }

          const pts = [{ x: pStart.x, y: pStart.y }];
          if (ann.waypoints && ann.waypoints.length > 0) {
            pts.push(...ann.waypoints);
          } else {
            const validObstacles = obstacles.filter(o => o.id !== ann.sourceId && o.id !== ann.targetId);
            const dynamicWps = getDynamicRoute(pStart, pEnd, validObstacles);
            if (dynamicWps.length > 0) {
              pts.push(...dynamicWps);
            }
          }
          pts.push({ x: pEnd.x, y: pEnd.y });
          const n = pts.length;

          // ── Couleurs dégradé source → target ──
          const srcAnn = ann.sourceId ? board.annotations.find(a => a.id === ann.sourceId) : null;
          const tgtAnn = ann.targetId ? board.annotations.find(a => a.id === ann.targetId) : null;
          
          const srcHue = srcAnn ? getSymbioticHue(srcAnn, board.annotations) : getSymbioticHue({ ...ann, x: pts[0].x, y: pts[0].y } as any, board.annotations);
          const tgtHue = tgtAnn ? getSymbioticHue(tgtAnn, board.annotations) : getSymbioticHue({ ...ann, x: pts[n-1].x, y: pts[n-1].y } as any, board.annotations);
          
          const colStart = `hsl(${srcHue}, 80%, 65%)`;
          const colEnd = `hsl(${tgtHue}, 80%, 65%)`;
          const gradId = `grad-${ann.id}`;
          const midHue = lerpHue(srcHue, tgtHue, 0.5);
          const colMid = `hsl(${midHue}, 85%, 70%)`;

          const sw = ann.strokeWidth ?? 2;
          const curved = ann.arrowType === "curved";

          let d = `M ${pts[0].x} ${pts[0].y}`;
          if (!curved || n === 2) {
            for (let i = 1; i < n; i++) d += ` L ${pts[i].x} ${pts[i].y}`;
          } else {
            const p = [
              { x: 2 * pts[0].x - pts[1].x, y: 2 * pts[0].y - pts[1].y },
              ...pts,
              { x: 2 * pts[n - 1].x - pts[n - 2].x, y: 2 * pts[n - 1].y - pts[n - 2].y },
            ];
            for (let i = 1; i < p.length - 2; i++) {
              const p0 = p[i - 1], p1 = p[i], p2 = p[i + 1], p3 = p[i + 2];
              const cp1x = p1.x + (p2.x - p0.x) / 6;
              const cp1y = p1.y + (p2.y - p0.y) / 6;
              const cp2x = p2.x - (p3.x - p1.x) / 6;
              const cp2y = p2.y - (p3.y - p1.y) / 6;
              d += ` C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${p2.x} ${p2.y}`;
            }
          }

          const midSeg = Math.floor(n / 2);
          const mp1 = pts[midSeg - 1] ?? pts[0];
          const mp2 = pts[midSeg];
          const midX = (mp1.x + mp2.x) / 2;
          const midY = (mp1.y + mp2.y) / 2;

          let badgeColor = "#888888";
          let badgeLabel = "?";
          if (ann.predicate) {
            const PREDICATE_COLORS: Record<string, string> = {
              est_precurseur: "#f59e0b", contredit: "#ef4444", herite_de: "#8b5cf6",
              inspire: "#10b981", depend_de: "#3b82f6", illustre: "#f472b6",
            };
            const PREDICATE_LABELS: Record<string, string> = {
              est_precurseur: "→", contredit: "✗", herite_de: "⊂",
              inspire: "✦", depend_de: "⊕", illustre: "◎",
            };
            badgeColor = PREDICATE_COLORS[ann.predicate] ?? "#888888";
            badgeLabel = PREDICATE_LABELS[ann.predicate] ?? "?";
          }
          const labelOffY = ann.predicate ? -14 : 0;
          const badgeOffY = ann.text ? 14 : 0;
          const isSelected = selectedIds.includes(ann.id);

          const hoverDetail = {
            sourceId: ann.sourceId,
            sourceBlockId: ann.sourceBlockId,
            targetId: ann.targetId,
            targetBlockId: ann.targetBlockId,
            sourceTextSel: ann.sourceTextSel,
            targetTextSel: ann.targetTextSel,
          };

          return (
            <g key={ann.id}>
              <defs>
                <linearGradient id={gradId} x1={pts[0].x} y1={pts[0].y} x2={pts[n-1].x} y2={pts[n-1].y} gradientUnits="userSpaceOnUse">
                  <stop offset="0%" stopColor={colStart} />
                  <stop offset="50%" stopColor={colMid} />
                  <stop offset="100%" stopColor={colEnd} />
                </linearGradient>
              </defs>

              <g style={{ pointerEvents: "auto", cursor: "pointer" }}
                onWheel={forwardWheel}
                onMouseEnter={() => {
                  window.dispatchEvent(new CustomEvent("glucose:hover-arrow", { detail: hoverDetail }));
                }}
                onMouseLeave={() => {
                  window.dispatchEvent(new CustomEvent("glucose:hover-arrow", { detail: null }));
                }}
                onPointerDown={(e) => {
                  e.stopPropagation();
                  onSelect(ann.id, e.ctrlKey || e.metaKey || e.shiftKey);
                }}
              >
                <path d={d} fill="none" stroke="rgba(0,0,0,0.01)" strokeWidth={24} vectorEffect="non-scaling-stroke" strokeLinecap="round" strokeLinejoin="round" />
                <path d={d} fill="none" stroke={`url(#${gradId})`} strokeWidth={isSelected ? sw + 10 : sw + 4} strokeOpacity={isSelected ? 0.45 : 0.18} vectorEffect="non-scaling-stroke" strokeLinecap="round" strokeLinejoin="round"
                  strokeDasharray={transDomain ? "6 4" : undefined} />
                <path d={d} fill="none" stroke={isSelected ? "#ffffff" : `url(#${gradId})`} strokeWidth={sw} strokeOpacity={0.92} vectorEffect="non-scaling-stroke" strokeLinecap="round" strokeLinejoin="round"
                  strokeDasharray={transDomain ? "6 4" : undefined} />

                {isSelected && (
                  <>
                    <circle cx={pts[0].x} cy={pts[0].y} r={sw * 2.5} fill={colStart} stroke="#fff" strokeWidth={2} vectorEffect="non-scaling-stroke" />
                    <circle cx={pts[n - 1].x} cy={pts[n - 1].y} r={sw * 2.5} fill={colEnd} stroke="#fff" strokeWidth={2} vectorEffect="non-scaling-stroke" />
                    {ann.waypoints?.map((wp, i) => (
                      <WaypointHandle key={`wp-${i}`} wp={wp} annId={ann.id} wpIndex={i} vpScale={vp.scale} />
                    ))}
                    {Array.from({ length: n - 1 }).map((_, i) => {
                      const p1 = pts[i];
                      const p2 = pts[i + 1];
                      const mx = (p1.x + p2.x) / 2;
                      const my = (p1.y + p2.y) / 2;
                      return <MidHandle key={`mid-${i}`} midX={mx} midY={my} annId={ann.id} insertIndex={i} vpScale={vp.scale} />;
                    })}
                  </>
                )}

                {/* Pointe terminale standard ou portail inter-boards (Phase 5) */}
                {!isSelected && !ann.targetBoardId && (
                  <circle cx={pts[n - 1].x} cy={pts[n - 1].y} r={sw * 1.5} fill="#111" stroke={colEnd} strokeWidth={2} vectorEffect="non-scaling-stroke" />
                )}
                {/* Anneau portail si la flèche pointe vers un autre board */}
                {ann.targetBoardId && (
                  <g
                    transform={`translate(${pts[n - 1].x}, ${pts[n - 1].y})`}
                    style={{ cursor: "pointer", pointerEvents: "auto" }}
                    onPointerDown={(e) => e.stopPropagation()}
                    onClick={(e) => {
                      e.stopPropagation();
                      window.dispatchEvent(new CustomEvent("glucose:portal-jump", {
                        detail: { boardId: ann.targetBoardId, targetId: ann.targetId },
                      }));
                    }}
                  >
                    <title>Portail vers un autre board — cliquer pour y aller</title>
                    {/* Anneau extérieur pulsant */}
                    <circle cx={0} cy={0} r={sw * 4} fill="none" stroke="#93c5fd" strokeOpacity={0.35}
                      strokeWidth={1.5} strokeDasharray="3 3" vectorEffect="non-scaling-stroke">
                      <animateTransform attributeName="transform" type="rotate" from="0" to="360" dur="6s" repeatCount="indefinite" />
                    </circle>
                    <circle cx={0} cy={0} r={sw * 2.6} fill="rgba(15,15,25,0.85)"
                      stroke="#93c5fd" strokeWidth={1.8} vectorEffect="non-scaling-stroke" />
                    <text textAnchor="middle" dominantBaseline="middle" y={1}
                      fill="#93c5fd" fontSize={Math.max(8, sw * 2.2)} fontFamily="system-ui, sans-serif"
                      fontWeight={700} style={{ pointerEvents: "none", userSelect: "none" }}>↗</text>
                  </g>
                )}
                {!isSelected && ann.arrowBidirectional && (
                  <circle cx={pts[0].x} cy={pts[0].y} r={sw * 1.5} fill="#111" stroke={colStart} strokeWidth={2} vectorEffect="non-scaling-stroke" />
                )}

                {ann.text && (() => {
                  const textWidthEstimate = ann.text.length * 6.5;
                  return (
                    <g transform={`translate(${midX}, ${midY + labelOffY})`}>
                      <rect x={-textWidthEstimate / 2 - 4} y={-11} width={textWidthEstimate + 8} height={22} fill="#111111" fillOpacity={0.75} rx={3} />
                      <text textAnchor="middle" dominantBaseline="middle" fill={colMid} fontSize={11} fontFamily="system-ui, sans-serif">{ann.text}</text>
                    </g>
                  );
                })()}

                {ann.predicate && (
                  <g transform={`translate(${midX}, ${midY + badgeOffY})`}>
                    <circle cx={0} cy={0} r={10} fill="#111111" fillOpacity={0.9} stroke={badgeColor} strokeWidth={1.5} vectorEffect="non-scaling-stroke" />
                    <text textAnchor="middle" dominantBaseline="middle" y={1} fill={badgeColor} fontSize={9} fontFamily="system-ui, sans-serif">{badgeLabel}</text>
                  </g>
                )}

                {/* Badge "i" — Phase 5 : ouvre le panneau de description longue.
                    Toujours présent (ouvre en édition si vide), pastille plus marquée si contenu existant. */}
                {(() => {
                  const hasDesc = !!ann.longText && ann.longText.trim().length > 0;
                  const iOffX = (ann.predicate || ann.text) ? 24 : 0;
                  return (
                    <g
                      transform={`translate(${midX + iOffX}, ${midY + badgeOffY})`}
                      style={{ cursor: "pointer", pointerEvents: "auto" }}
                      onPointerDown={(e) => e.stopPropagation()}
                      onClick={(e) => {
                        e.stopPropagation();
                        // Convertit la position monde (midX, midY) en position écran via vpRef
                        const v = vpRef.current;
                        const canvas = document.querySelector("canvas");
                        const rect = canvas?.getBoundingClientRect();
                        const screenX = (rect?.left ?? 0) + v.x + (midX + iOffX) * v.scale;
                        const screenY = (rect?.top ?? 0) + v.y + (midY + badgeOffY) * v.scale;
                        window.dispatchEvent(new CustomEvent("glucose:open-arrow-description", {
                          detail: { arrowId: ann.id, screenX, screenY },
                        }));
                      }}
                    >
                      <title>Description longue (Markdown)</title>
                      <circle cx={0} cy={0} r={9}
                        fill={hasDesc ? "rgba(147,197,253,0.18)" : "#111111"}
                        fillOpacity={0.9}
                        stroke="#93c5fd" strokeOpacity={hasDesc ? 0.9 : 0.5}
                        strokeWidth={1.5}
                        vectorEffect="non-scaling-stroke" />
                      <text textAnchor="middle" dominantBaseline="middle" y={1}
                        fill="#93c5fd" fillOpacity={hasDesc ? 1 : 0.7}
                        fontSize={10} fontFamily="system-ui, serif" fontStyle="italic" fontWeight="700"
                        style={{ pointerEvents: "none", userSelect: "none" }}>i</text>
                    </g>
                  );
                })()}
              </g>
            </g>
          );
        })})()}
      </g>
    </svg>
  );
}
