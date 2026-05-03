import { useEffect, useLayoutEffect, useRef, useState } from "react";
import OrganizePanel from "./OrganizePanel";
import StoryboardControls from "./StoryboardControls";
import PomodoroTimer from "./PomodoroTimer";

export type TabId = "organize" | "storyboard" | "pomodoro";

interface Props {
  openTabs: TabId[];
  dismissingTabs: TabId[];
  onDismiss: (id: TabId) => void;
}

function renderPanel(id: TabId) {
  if (id === "organize")   return <OrganizePanel   key={id} docked />;
  if (id === "storyboard") return <StoryboardControls key={id} docked />;
  if (id === "pomodoro")   return <PomodoroTimer   key={id} />;
  return null;
}

const DISMISS_THRESHOLD = 80;
const DISMISS_DURATION  = 200;

export default function PanelDock({ openTabs, dismissingTabs, onDismiss }: Props) {
  const [localOrder, setLocalOrder] = useState<TabId[]>([]);

  useEffect(() => {
    setLocalOrder((prev) => {
      const kept  = prev.filter((t) => openTabs.includes(t));
      const added = openTabs.filter((t) => !prev.includes(t));
      return [...kept, ...added];
    });
  }, [openTabs.join(",")]); // eslint-disable-line react-hooks/exhaustive-deps

  const ordered = localOrder.filter((t) => openTabs.includes(t));

  // ── Entry animation — track which tabs have completed their mount frame ─────
  const [mountedTabs, setMountedTabs] = useState<Set<TabId>>(new Set());

  useEffect(() => {
    const toMount = ordered.filter((id) => !mountedTabs.has(id));
    if (toMount.length === 0) return;
    // Defer by one frame so the browser paints the initial (collapsed) state first
    const raf = requestAnimationFrame(() => {
      setMountedTabs((prev) => {
        const next = new Set(prev);
        toMount.forEach((id) => next.add(id));
        return next;
      });
    });
    return () => cancelAnimationFrame(raf);
  }, [ordered.join(",")]); // eslint-disable-line react-hooks/exhaustive-deps

  // Clean up mounted state when a tab is fully removed
  useEffect(() => {
    setMountedTabs((prev) => {
      const next = new Set(prev);
      for (const id of prev) {
        if (!openTabs.includes(id)) next.delete(id);
      }
      return next;
    });
  }, [openTabs.join(",")]); // eslint-disable-line react-hooks/exhaustive-deps

  // ── FLIP animation ────────────────────────────────────────────
  const outerRefs    = useRef<(HTMLDivElement | null)[]>([]);
  const posBeforeRef = useRef<Record<string, DOMRect>>({});

  function capturePositions() {
    const snap: Record<string, DOMRect> = {};
    ordered.forEach((id, i) => {
      const el = outerRefs.current[i];
      if (el) snap[id] = el.getBoundingClientRect();
    });
    posBeforeRef.current = snap;
  }

  useLayoutEffect(() => {
    const before = posBeforeRef.current;
    if (Object.keys(before).length === 0) return;
    ordered.forEach((id, i) => {
      const el = outerRefs.current[i];
      if (!el || !before[id]) return;
      const dx = before[id].left - el.getBoundingClientRect().left;
      if (Math.abs(dx) < 1) return;
      el.style.transition = "none";
      el.style.transform = `translateX(${dx}px)`;
      void el.offsetWidth;
      el.style.transition = "transform 0.25s cubic-bezier(0.22, 1, 0.36, 1)";
      el.style.transform = "";
    });
    posBeforeRef.current = {};
  }, [ordered.join(",")]); // eslint-disable-line react-hooks/exhaustive-deps

  // ── Drag state ────────────────────────────────────────────────
  const dragRef      = useRef<{ fromIdx: number; startX: number; startY: number } | null>(null);
  const [draggingIdx, setDraggingIdx]     = useState<number | null>(null);
  const [dragOffsetX, setDragOffsetX]     = useState(0);
  const [dismissingTab, setDismissingTab] = useState<TabId | null>(null);

  function onGripDown(e: React.PointerEvent<HTMLDivElement>, idx: number) {
    e.preventDefault();
    e.currentTarget.setPointerCapture(e.pointerId);
    dragRef.current = { fromIdx: idx, startX: e.clientX, startY: e.clientY };
    setDraggingIdx(idx);
    setDragOffsetX(0);
  }

  function onGripMove(e: React.PointerEvent<HTMLDivElement>) {
    const drag = dragRef.current;
    if (!drag) return;

    const dx = e.clientX - drag.startX;
    const dy = e.clientY - drag.startY;
    const idx = drag.fromIdx;

    // Drag DOWN past threshold → dismiss
    if (dy > DISMISS_THRESHOLD && dismissingTab === null) {
      const tab = ordered[idx];
      setDismissingTab(tab);
      setDraggingIdx(null);
      setDragOffsetX(0);
      dragRef.current = null;
      setTimeout(() => {
        onDismiss(tab);
        setDismissingTab(null);
      }, DISMISS_DURATION);
      return;
    }

    // Horizontal swap — only if not mostly vertical
    if (Math.abs(dy) > Math.abs(dx)) return;

    // Update the visual drag offset (partial follow: 35% of cursor travel)
    setDragOffsetX(dx);

    const panelEl = outerRefs.current[idx];
    if (!panelEl) return;
    const halfW = panelEl.getBoundingClientRect().width / 2;

    if (dx > halfW && idx < ordered.length - 1) {
      capturePositions();
      setLocalOrder((prev) => {
        const copy = [...prev];
        const ai = copy.indexOf(ordered[idx]);
        const bi = copy.indexOf(ordered[idx + 1]);
        [copy[ai], copy[bi]] = [copy[bi], copy[ai]];
        return copy;
      });
      // Reset offset & origin for the new position
      dragRef.current = { fromIdx: idx + 1, startX: e.clientX, startY: drag.startY };
      setDraggingIdx(idx + 1);
      setDragOffsetX(0);
    } else if (dx < -halfW && idx > 0) {
      capturePositions();
      setLocalOrder((prev) => {
        const copy = [...prev];
        const ai = copy.indexOf(ordered[idx]);
        const bi = copy.indexOf(ordered[idx - 1]);
        [copy[ai], copy[bi]] = [copy[bi], copy[ai]];
        return copy;
      });
      dragRef.current = { fromIdx: idx - 1, startX: e.clientX, startY: drag.startY };
      setDraggingIdx(idx - 1);
      setDragOffsetX(0);
    }
  }

  function onGripUp() {
    dragRef.current = null;
    setDraggingIdx(null);
    setDragOffsetX(0);
  }

  if (ordered.length === 0) return null;

  return (
    <div style={{
      position: "absolute",
      bottom: 8, left: 8,
      display: "flex",
      flexDirection: "row",
      alignItems: "flex-end",
      gap: 8,
      zIndex: 200,
      pointerEvents: "none",
    }}>
      {ordered.map((id, idx) => {
        const isDragged    = draggingIdx === idx;
        const isDismissing = dismissingTab === id || dismissingTabs.includes(id);
        const isEntering   = !mountedTabs.has(id) && !isDismissing;

        return (
          // Outer div: FLIP target (translateX only, set imperatively)
          // Also carries the live drag-follow offset during horizontal drag
          <div
            key={id}
            ref={(el) => { outerRefs.current[idx] = el; }}
            style={{
              pointerEvents: "all",
              transform: isDragged ? `translateX(${dragOffsetX * 0.35}px)` : undefined,
              transition: isDragged ? "none" : undefined,
              willChange: isDragged ? "transform" : undefined,
            }}
          >
            {/* Inner div: lift / dismiss / entry transforms */}
            <div
              style={{
                position: "relative",
                borderRadius: 6,
                boxShadow: isDragged
                  ? "0 16px 48px rgba(0,0,0,0.9), 0 0 0 1px #555"
                  : "0 4px 20px rgba(0,0,0,0.5)",
                transform: isDismissing
                  ? "translateY(48px) scale(0.88)"
                  : isEntering
                    ? "translateY(48px) scale(0.88)"
                    : isDragged
                      ? "translateY(-4px) scale(1.02)"
                      : "translateY(0) scale(1)",
                opacity: isDismissing || isEntering ? 0 : 1,
                transition: isDragged
                  ? "box-shadow 0.12s"
                  : isDismissing
                    ? `transform ${DISMISS_DURATION}ms cubic-bezier(0.4,0,1,1), opacity ${DISMISS_DURATION}ms ease`
                    : "transform 0.22s cubic-bezier(0.34,1.56,0.64,1), box-shadow 0.18s, opacity 0.18s",
                zIndex: isDragged ? 10 : 1,
              }}
            >
              {/* Drag grip */}
              <div
                onPointerDown={(e) => onGripDown(e, idx)}
                onPointerMove={onGripMove}
                onPointerUp={onGripUp}
                title="Glisser horizontalement pour réordonner · Glisser vers le bas pour fermer"
                style={{
                  position: "absolute",
                  top: 0, left: 0, right: 0,
                  height: 30,
                  cursor: isDragged ? "grabbing" : "grab",
                  zIndex: 2,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  borderRadius: "6px 6px 0 0",
                }}
              >
                <span style={{
                  fontSize: 12,
                  color: isDragged ? "#888" : "#333",
                  letterSpacing: 3,
                  pointerEvents: "none",
                  userSelect: "none",
                  transition: "color 0.15s",
                }}>⠿⠿</span>
              </div>

              {renderPanel(id)}
            </div>
          </div>
        );
      })}
    </div>
  );
}
