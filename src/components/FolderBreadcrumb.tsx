import { useGlucoseStore } from "../store";

export default function FolderBreadcrumb() {
  const { folderStack, project, exitFolder, exitToRoot } = useGlucoseStore();
  if (folderStack.length === 0) return null;

  // Construire les items du fil d'Ariane
  const crumbs = folderStack.map(({ boardId, folderId }) => {
    const board  = project.boards.find((b) => b.id === boardId);
    const folder = (board?.folders ?? []).find((f) => f.id === folderId);
    return { boardId, folderId, name: folder?.name ?? "Fichier", color: folder?.color ?? "#60a5fa" };
  });

  return (
    <div style={{
      position: "absolute", top: 8, left: "50%", transform: "translateX(-50%)",
      display: "flex", alignItems: "center", gap: 4,
      background: "rgba(15,15,15,0.88)", backdropFilter: "blur(8px)",
      border: "1px solid #2a2a2a", borderRadius: 20,
      padding: "4px 10px", zIndex: 50,
      boxShadow: "0 2px 12px rgba(0,0,0,0.5)",
      fontSize: 11, color: "#555",
    }}>
      {/* Racine */}
      <button
        onClick={exitToRoot}
        style={{
          background: "none", border: "none", color: "#444", cursor: "pointer",
          fontSize: 11, padding: "0 2px", display: "flex", alignItems: "center", gap: 4,
        }}
        onMouseOver={(e) => { e.currentTarget.style.color = "#888"; }}
        onMouseOut={(e) => { e.currentTarget.style.color = "#444"; }}
        title="Retour à la racine"
      >
        <svg width="10" height="10" viewBox="0 0 12 12" fill="none">
          <path d="M1 5.5L6 1l5 4.5V11H8V8H4v3H1V5.5Z" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round"/>
        </svg>
        Racine
      </button>

      {crumbs.map((crumb, i) => (
        <span key={crumb.folderId} style={{ display: "flex", alignItems: "center", gap: 4 }}>
          <span style={{ color: "#2a2a2a" }}>›</span>
          {i < crumbs.length - 1 ? (
            <button
              onClick={() => {
                // Remonter jusqu'à ce niveau
                const stepsBack = crumbs.length - 1 - i;
                for (let k = 0; k < stepsBack; k++) exitFolder();
              }}
              style={{
                background: "none", border: "none",
                color: crumb.color + "99", cursor: "pointer",
                fontSize: 11, padding: "0 2px",
              }}
              onMouseOver={(e) => { e.currentTarget.style.color = crumb.color; }}
              onMouseOut={(e) => { e.currentTarget.style.color = crumb.color + "99"; }}
            >
              {crumb.name}
            </button>
          ) : (
            <span style={{ color: crumb.color, fontWeight: 600 }}>
              {crumb.name}
            </span>
          )}
        </span>
      ))}

      {/* Bouton retour */}
      <span style={{ color: "#2a2a2a", marginLeft: 4 }}>|</span>
      <button
        onClick={exitFolder}
        style={{
          background: "none", border: "none", color: "#444", cursor: "pointer",
          fontSize: 11, padding: "0 2px", display: "flex", alignItems: "center", gap: 3,
        }}
        onMouseOver={(e) => { e.currentTarget.style.color = "#888"; }}
        onMouseOut={(e) => { e.currentTarget.style.color = "#444"; }}
        title="Remonter (Backspace)"
      >
        ← Sortir
      </button>
    </div>
  );
}
