import { useEffect, useRef, useState } from "react";
import { useGlucoseStore } from "../store";

export default function BoardTabs() {
  const { project, setActiveBoardId, addBoard, removeBoard, renameBoard, reorderBoards } = useGlucoseStore();
  const { boards, activeBoardId } = project;
  const [renamingId, setRenamingId]   = useState<string | null>(null);
  const [renameVal, setRenameVal]     = useState("");
  const [draggingIdx, setDraggingIdx] = useState<number | null>(null);
  const [overIdx, setOverIdx]         = useState<number | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Release drag on mouseup anywhere
  useEffect(() => {
    function onUp() {
      if (draggingIdx !== null && overIdx !== null && draggingIdx !== overIdx) {
        reorderBoards(draggingIdx, overIdx);
      }
      setDraggingIdx(null);
      setOverIdx(null);
    }
    window.addEventListener("mouseup", onUp);
    return () => window.removeEventListener("mouseup", onUp);
  }, [draggingIdx, overIdx]);

  function startRename(id: string, current: string) {
    setRenamingId(id);
    setRenameVal(current);
    setTimeout(() => inputRef.current?.select(), 30);
  }

  function commitRename() {
    if (renamingId && renameVal.trim()) renameBoard(renamingId, renameVal.trim());
    setRenamingId(null);
  }

  // Filtrer pour ne garder que les boards principaux (qui ne sont pas des sous-dossiers)
  const allFolderChildIds = new Set(
    boards.flatMap((b) => (b.folders ?? []).map((f) => f.childBoardId))
  );
  const rootBoards = boards.filter((b) => !allFolderChildIds.has(b.id));

  return (
    <div style={{
      display: "flex", alignItems: "stretch",
      height: 34, background: "#111", borderBottom: "1px solid #222",
      overflowX: "auto", overflowY: "hidden", flexShrink: 0,
      userSelect: "none",
    }}>
      {rootBoards.map((board, idx) => {
        const active     = board.id === activeBoardId || useGlucoseStore.getState().folderStack.some(f => f.boardId === board.id);
        const isRenaming = renamingId === board.id;
        const isDragging = draggingIdx === idx;
        const isOver     = overIdx === idx && draggingIdx !== null && draggingIdx !== idx;

        const presetName = board.presetId
          ? useGlucoseStore.getState().getAllPresets().find((p) => p.id === board.presetId)?.name
          : null;

        return (
          <div
            key={board.id}
            onMouseDown={(e) => {
              if (e.button !== 0 || isRenaming) return;
              setDraggingIdx(idx);
            }}
            onMouseEnter={() => {
              if (draggingIdx !== null) setOverIdx(idx);
            }}
            onClick={() => { if (!isRenaming && draggingIdx === null) setActiveBoardId(board.id); }}
            onDoubleClick={() => { if (draggingIdx === null) startRename(board.id, board.name); }}
            style={{
              display: "flex", alignItems: "center", gap: 6,
              padding: "0 12px", minWidth: 100, maxWidth: 180,
              cursor: isDragging ? "grabbing" : "grab",
              flexShrink: 0,
              borderRight: "1px solid #1a1a1a",
              background: active ? "#1a1a1a" : isOver ? "#1e1e1e" : "transparent",
              borderBottom: active ? "2px solid #fff" : isOver ? "2px solid #555" : "2px solid transparent",
              color: active ? "#fff" : "#555",
              opacity: isDragging ? 0.5 : 1,
              fontSize: 12,
              position: "relative",
              transition: "opacity 0.1s",
            }}
          >
            {isRenaming ? (
              <input
                ref={inputRef}
                value={renameVal}
                onChange={(e) => setRenameVal(e.target.value)}
                onBlur={commitRename}
                onKeyDown={(e) => {
                  if (e.key === "Enter") commitRename();
                  if (e.key === "Escape") setRenamingId(null);
                }}
                style={{
                  background: "#333", color: "#fff", border: "1px solid #555",
                  borderRadius: 3, padding: "1px 6px", fontSize: 12,
                  width: 100, outline: "none",
                }}
                onClick={(e) => e.stopPropagation()}
                onMouseDown={(e) => e.stopPropagation()}
                autoFocus
              />
            ) : (
              <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1 }}>
                {board.name}
              </span>
            )}

            {presetName && (
              <span style={{
                fontSize: 9, padding: "1px 5px", borderRadius: 8,
                background: "#2a2a2a", color: "#666", whiteSpace: "nowrap",
              }}>
                {presetName}
              </span>
            )}

            {boards.length > 1 && !isRenaming && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  if (window.confirm(`Supprimer "${board.name}" ? Cette action est annulable (Ctrl+Z).`)) {
                    removeBoard(board.id);
                  }
                }}
                onMouseDown={(e) => e.stopPropagation()}
                style={{
                  width: 14, height: 14, borderRadius: "50%", border: "none",
                  background: "transparent", color: "#444", cursor: "pointer",
                  display: "flex", alignItems: "center", justifyContent: "center",
                  fontSize: 11, padding: 0, flexShrink: 0,
                }}
                onMouseOver={(e) => { e.currentTarget.style.background = "#333"; e.currentTarget.style.color = "#aaa"; }}
                onMouseOut={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "#444"; }}
                title="Supprimer ce board"
              >
                ×
              </button>
            )}
          </div>
        );
      })}

      <button
        onClick={() => addBoard(`Board ${rootBoards.length + 1}`)}
        title="Nouveau board"
        style={{
          display: "flex", alignItems: "center", justifyContent: "center",
          width: 34, border: "none", background: "transparent",
          color: "#444", cursor: "pointer", fontSize: 16, flexShrink: 0,
        }}
        onMouseOver={(e) => { e.currentTarget.style.color = "#aaa"; }}
        onMouseOut={(e) => { e.currentTarget.style.color = "#444"; }}
      >
        +
      </button>
    </div>
  );
}
