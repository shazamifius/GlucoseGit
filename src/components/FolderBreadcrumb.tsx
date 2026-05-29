// Phase 7.5.B5 — Breadcrumb façon VSCode.
// Aligné à gauche, segments cliquables avec icônes folder, séparateurs « › »,
// dropdown sur hover qui liste les siblings du dossier courant pour navigation
// rapide entre frères.

import { useEffect, useRef, useState } from "react";
import { useGlucoseStore } from "../store";

interface Crumb {
  boardId: string;       // board parent qui contient ce folder
  folderId: string;
  name: string;
  color: string;
}

export default function FolderBreadcrumb() {
  const folderStack = useGlucoseStore((s) => s.folderStack);
  const project = useGlucoseStore((s) => s.project);
  const exitFolder = useGlucoseStore((s) => s.exitFolder);
  const exitToRoot = useGlucoseStore((s) => s.exitToRoot);
  const enterFolder = useGlucoseStore((s) => s.enterFolder);
  const setActiveBoardId = useGlucoseStore((s) => s.setActiveBoardId);

  const [siblingsFor, setSiblingsFor] = useState<{ idx: number; siblings: Crumb[] } | null>(null);
  const hideTimerRef = useRef<number | null>(null);

  // ⚠️ Tous les hooks DOIVENT être appelés inconditionnellement avant tout early return.
  // Auparavant, ce useEffect était placé après `if (folderStack.length === 0) return null;`,
  // ce qui causait React error #310 ("Rendered more hooks than during the previous render")
  // au moment précis où on entrait dans le premier folder (passage de 0 → N segments).
  useEffect(() => {
    return () => { if (hideTimerRef.current) window.clearTimeout(hideTimerRef.current); };
  }, []);

  if (folderStack.length === 0) return null;

  const crumbs: Crumb[] = folderStack.map(({ boardId, folderId }) => {
    const board = project.boards.find((b) => b.id === boardId);
    const folder = (board?.folders ?? []).find((f) => f.id === folderId);
    return { boardId, folderId, name: folder?.name ?? "Dossier", color: folder?.color ?? "#60a5fa" };
  });

  // Pour "Racine" : siblings = tous les boards racine (boards qui ne sont enfants
  // d'aucun folder). Mais en pratique l'app a un seul board principal au niveau 0.
  // On expose donc pour chaque crumb les autres folders du même parent board.
  function siblingsAt(idx: number): Crumb[] {
    const cur = crumbs[idx];
    const parent = project.boards.find((b) => b.id === cur.boardId);
    if (!parent) return [];
    return (parent.folders ?? [])
      .filter((f) => f.id !== cur.folderId)
      .map((f) => ({ boardId: cur.boardId, folderId: f.id, name: f.name, color: f.color }));
  }

  function jumpTo(targetIdx: number) {
    // Exit jusqu'au niveau souhaité
    const stepsBack = crumbs.length - 1 - targetIdx;
    for (let k = 0; k < stepsBack; k++) exitFolder();
  }

  function jumpToSibling(parentIdx: number, sib: Crumb) {
    // Remonte au niveau parent puis enter dans le sibling
    const stepsBack = crumbs.length - parentIdx;
    for (let k = 0; k < stepsBack; k++) exitFolder();
    // Assure le bon board actif (devrait l'être déjà)
    setActiveBoardId(sib.boardId);
    enterFolder(sib.folderId);
    setSiblingsFor(null);
  }

  function showSiblings(idx: number) {
    if (hideTimerRef.current) { window.clearTimeout(hideTimerRef.current); hideTimerRef.current = null; }
    const siblings = siblingsAt(idx);
    if (siblings.length > 0) setSiblingsFor({ idx, siblings });
  }

  function scheduleHide() {
    if (hideTimerRef.current) window.clearTimeout(hideTimerRef.current);
    hideTimerRef.current = window.setTimeout(() => setSiblingsFor(null), 220);
  }

  const FolderIcon = ({ color, size = 10 }: { color: string; size?: number }) => (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none" style={{ flexShrink: 0 }}>
      <path d="M0 3 Q0 1 2 1 L7 1 L9 3 L14 3 Q16 3 16 5 L16 13 Q16 15 14 15 L2 15 Q0 15 0 13 Z"
            fill={color} fillOpacity={0.85} />
    </svg>
  );

  return (
    <div style={{
      position: "absolute", top: 8, left: 16,
      display: "flex", alignItems: "center", gap: 0,
      background: "rgba(15,15,18,0.82)", backdropFilter: "blur(8px)",
      WebkitBackdropFilter: "blur(8px)",
      border: "1px solid #2a2a2a", borderRadius: 6,
      padding: "4px 6px", zIndex: 50,
      boxShadow: "0 2px 12px rgba(0,0,0,0.45)",
      fontSize: 11, color: "#888",
      fontFamily: "system-ui, -apple-system, sans-serif",
      userSelect: "none",
    }}>
      {/* Racine */}
      <button
        onClick={exitToRoot}
        title="Retour à la racine"
        style={crumbBtnStyle("#666")}
        onMouseEnter={(e) => { e.currentTarget.style.background = "#222"; e.currentTarget.style.color = "#bbb"; }}
        onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "#666"; }}
      >
        <svg width="10" height="10" viewBox="0 0 12 12" fill="none">
          <path d="M1 5.5L6 1l5 4.5V11H8V8H4v3H1V5.5Z" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round"/>
        </svg>
        <span>racine</span>
      </button>

      {crumbs.map((crumb, i) => {
        const isLast = i === crumbs.length - 1;
        return (
          <div
            key={crumb.folderId}
            style={{ display: "flex", alignItems: "center", position: "relative" }}
            onMouseEnter={() => showSiblings(i)}
            onMouseLeave={scheduleHide}
          >
            <span style={{ color: "#3a3a3a", padding: "0 4px" }}>›</span>
            <button
              onClick={() => !isLast && jumpTo(i)}
              style={{
                ...crumbBtnStyle(isLast ? crumb.color : crumb.color + "aa"),
                fontWeight: isLast ? 600 : 500,
                cursor: isLast ? "default" : "pointer",
              }}
              onMouseEnter={(e) => { if (!isLast) { e.currentTarget.style.background = "#222"; } }}
              onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
            >
              <FolderIcon color={crumb.color} />
              <span>{crumb.name.length > 22 ? crumb.name.slice(0, 19) + "…" : crumb.name}</span>
            </button>

            {/* Dropdown des siblings */}
            {siblingsFor?.idx === i && siblingsFor.siblings.length > 0 && (
              <div
                onMouseEnter={() => showSiblings(i)}
                onMouseLeave={scheduleHide}
                style={{
                  position: "absolute", top: "100%", left: 16, marginTop: 4,
                  background: "rgba(20,20,24,0.96)",
                  border: "1px solid #2a2a2a", borderRadius: 6,
                  padding: 4, minWidth: 160, maxHeight: 280, overflowY: "auto",
                  boxShadow: "0 8px 24px rgba(0,0,0,0.6)",
                  zIndex: 60,
                }}
              >
                {siblingsFor.siblings.map((sib) => (
                  <button
                    key={sib.folderId}
                    onClick={() => jumpToSibling(i, sib)}
                    style={{
                      display: "flex", alignItems: "center", gap: 6,
                      width: "100%", padding: "5px 8px",
                      background: "transparent", border: "none",
                      color: "#bbb", cursor: "pointer",
                      fontSize: 11, textAlign: "left",
                      borderRadius: 4,
                    }}
                    onMouseOver={(e) => { e.currentTarget.style.background = "#2a2a2a"; }}
                    onMouseOut={(e) => { e.currentTarget.style.background = "transparent"; }}
                  >
                    <FolderIcon color={sib.color} />
                    <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      {sib.name}
                    </span>
                  </button>
                ))}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

function crumbBtnStyle(color: string): React.CSSProperties {
  return {
    display: "flex", alignItems: "center", gap: 5,
    background: "transparent", border: "none", color,
    fontSize: 11, padding: "3px 6px",
    borderRadius: 3,
    fontFamily: "inherit",
    transition: "background 80ms, color 80ms",
  };
}
