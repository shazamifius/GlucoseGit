import { useEffect, useRef } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useGlucoseStore, getActiveBoard } from "../store";
import { addImagesFromFiles } from "../canvas/fileImport";
import { Tool } from "../types";

interface ToolbarProps {
  onTogglePreset: () => void;
  presetPanelOpen: boolean;
  onToggleDomains: () => void;
  domainsPanelOpen: boolean;
  onToggleOrganize: () => void;
  organizePanelOpen: boolean;
  onToggleStoryboard: () => void;
  storyboardPanelOpen: boolean;
  onTogglePomodoro: () => void;
  pomodoroOpen: boolean;
}

export default function Toolbar({
  onTogglePreset, presetPanelOpen,
  onToggleDomains, domainsPanelOpen,
  onToggleOrganize, organizePanelOpen,
  onToggleStoryboard, storyboardPanelOpen,
  onTogglePomodoro, pomodoroOpen,
}: ToolbarProps) {
  // CLEANUP P-08 — Selectors atomiques (pas de full-store subscribe)
  const activeTool = useGlucoseStore(s => s.activeTool);
  const setActiveTool = useGlucoseStore(s => s.setActiveTool);
  const project = useGlucoseStore(s => s.project);
  const getAllPresets = useGlucoseStore(s => s.getAllPresets);
  const board = getActiveBoard(project);
  const activePreset = board.presetId ? getAllPresets().find((p) => p.id === board.presetId) : null;
  const fileInputRef = useRef<HTMLInputElement>(null);

  async function handleOpenFiles() {
    try {
      const result = await openDialog({
        multiple: true,
        filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "webp", "avif", "bmp"] }],
      });
      if (!result) return;
      const paths = (Array.isArray(result) ? result : [result]) as string[];
      if (paths.length > 0) {
        await addImagesFromFiles(paths, 0, 0, board.id, useGlucoseStore.getState().addImage);
        return;
      }
    } catch (_) {
      fileInputRef.current?.click();
    }
  }

  // Raccourci Ctrl+I — App.tsx dispatch ce custom event
  useEffect(() => {
    const onTrigger = () => { void handleOpenFiles(); };
    window.addEventListener("glucose:trigger-import", onTrigger);
    return () => window.removeEventListener("glucose:trigger-import", onTrigger);
  }, [board.id]);

  async function handleFileInputChange(e: React.ChangeEvent<HTMLInputElement>) {
    const files = Array.from(e.target.files || []);
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      const reader = new FileReader();
      reader.onload = async () => {
        const src = reader.result as string;
        const img = new Image();
        img.onload = () => {
          const { naturalWidth: w, naturalHeight: h } = img;
          const maxW = 600;
          const scale = w > maxW ? maxW / w : 1;
          useGlucoseStore.getState().addImage(board.id, {
            id: crypto.randomUUID().slice(0, 16),
            src, x: i * 24, y: i * 24,
            width: w * scale, height: h * scale,
            rotation: 0, locked: false, tags: [],
            originalWidth: w, originalHeight: h,
          });
        };
        img.src = src;
      };
      reader.readAsDataURL(file);
    }
    e.target.value = "";
  }

  const sep = <div style={{ width: 1, height: 20, background: "#2a2a2a", margin: "0 4px" }} />;

  return (
    <div style={{
      display: "flex", alignItems: "center", gap: 2,
      padding: "0 12px", height: 44,
      background: "#1a1a1a", borderBottom: "1px solid #2a2a2a",
      userSelect: "none", flexShrink: 0,
    }}>
      <span style={{ color: "#fff", fontWeight: 700, fontSize: 14, letterSpacing: 2, marginRight: 12, textTransform: "uppercase" }}>
        Atelier
      </span>

      {/* Tool group 1: select + pan */}
      <ToolBtn tool="select" active={activeTool === "select"} onClick={() => setActiveTool("select")} title="V — Sélectionner">
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
          <path d="M2 2L6.5 12L8 8L12 6.5L2 2Z" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round"/>
        </svg>
      </ToolBtn>

      <ToolBtn tool="pan" active={activeTool === "pan"} onClick={() => setActiveTool("pan")} title="Espace — Déplacer la vue">
        <svg width="13" height="13" viewBox="0 0 20 20" fill="none">
          <circle cx="10" cy="10" r="3" stroke="currentColor" strokeWidth="1.5"/>
          <path d="M10 2v3M10 15v3M2 10h3M15 10h3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
        </svg>
      </ToolBtn>

      {sep}

      {/* Tool group 2: annotations */}
      <ToolBtn tool="text" active={activeTool === "text"} onClick={() => setActiveTool("text")} title="T — Texte">
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
          <path d="M2 3h10M7 3v8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
        </svg>
      </ToolBtn>

      <ToolBtn tool="sticky" active={activeTool === "sticky"} onClick={() => setActiveTool("sticky")} title="N — Note sticky">
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
          <rect x="1.5" y="1.5" width="11" height="11" rx="1" stroke="currentColor" strokeWidth="1.3"/>
          <path d="M4 5h6M4 7.5h4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/>
        </svg>
      </ToolBtn>

      <ToolBtn tool="arrow" active={activeTool === "arrow"} onClick={() => setActiveTool("arrow")} title="A — Flèche">
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
          <path d="M2 12L12 2M12 2H7M12 2V7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
        </svg>
      </ToolBtn>

      <ToolBtn tool="folder" active={activeTool === "folder"} onClick={() => setActiveTool("folder")} title="F — Créer un fichier (sous-canvas)">
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
          <path d="M1 4.5V11.5a1 1 0 001 1h10a1 1 0 001-1V5.5a1 1 0 00-1-1H7L5.5 2.5H2a1 1 0 00-1 2z" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round"/>
        </svg>
      </ToolBtn>

      <ToolBtn tool="membrane" active={activeTool === "membrane"} onClick={() => setActiveTool("membrane")} title="M — Dessiner une membrane fixe">
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
          <rect x="1.5" y="1.5" width="11" height="11" rx="3" stroke="currentColor" strokeWidth="1.3" strokeDasharray="3 2"/>
        </svg>
      </ToolBtn>

      {sep}

      {/* +Images */}
      <input ref={fileInputRef} type="file" multiple accept="image/*" style={{ display: "none" }} onChange={handleFileInputChange} />
      <ActionBtn onClick={handleOpenFiles} title="Ajouter images (Ctrl+I)">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M8 2v12M2 8h12" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
        </svg>
        Images
      </ActionBtn>

      {sep}

      {/* Panel buttons — grouped with consistent 8px gap */}
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>

      <ActionBtn onClick={onToggleOrganize} active={organizePanelOpen} title="Organiser les images">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <rect x="1" y="1" width="6" height="4" rx="1" stroke="currentColor" strokeWidth="1.3"/>
          <rect x="9" y="1" width="6" height="4" rx="1" stroke="currentColor" strokeWidth="1.3"/>
          <rect x="1" y="8" width="6" height="4" rx="1" stroke="currentColor" strokeWidth="1.3"/>
          <rect x="9" y="8" width="6" height="4" rx="1" stroke="currentColor" strokeWidth="1.3"/>
        </svg>
        Ordonner
      </ActionBtn>

      <ActionBtn onClick={onTogglePomodoro} active={pomodoroOpen} title="Timer Pomodoro">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="9" r="6" stroke="currentColor" strokeWidth="1.3"/>
          <path d="M8 6v3.5l2 1.5" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/>
          <path d="M6 2h4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/>
        </svg>
        Timer
      </ActionBtn>

      <ActionBtn
        onClick={() => useGlucoseStore.getState().toggleSmartGuides()}
        active={useGlucoseStore((s) => s.smartGuidesEnabled)}
        title="G — Activer/Désactiver l'alignement intelligent"
      >
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M2 2l12 12M14 2L2 14" stroke="currentColor" strokeWidth="1" strokeOpacity="0.4"/>
          <rect x="3" y="3" width="10" height="10" rx="1" stroke="currentColor" strokeWidth="1.3" strokeDasharray="2 1"/>
        </svg>
        Aimant
      </ActionBtn>

      <ActionBtn
        onClick={() => useGlucoseStore.getState().toggleTransDomainVisible()}
        active={useGlucoseStore((s) => s.transDomainVisible)}
        title="Afficher les liens trans-domaines (visibles en pointillés à tout zoom)"
      >
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <circle cx="3.5" cy="8" r="2" stroke="currentColor" strokeWidth="1.3"/>
          <circle cx="12.5" cy="8" r="2" stroke="currentColor" strokeWidth="1.3"/>
          <path d="M5.5 8h5" stroke="currentColor" strokeWidth="1.3" strokeDasharray="1.5 1.5"/>
        </svg>
        Trans-domaines
      </ActionBtn>

      <ActionBtn onClick={onToggleStoryboard} active={storyboardPanelOpen || !!board.storyboard} title="Storyboard">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <rect x="1" y="2" width="6" height="5" rx="1" stroke="currentColor" strokeWidth="1.3"/>
          <rect x="9" y="2" width="6" height="5" rx="1" stroke="currentColor" strokeWidth="1.3"/>
          <rect x="1" y="9" width="6" height="5" rx="1" stroke="currentColor" strokeWidth="1.3"/>
          <rect x="9" y="9" width="6" height="5" rx="1" stroke="currentColor" strokeWidth="1.3"/>
        </svg>
        Storyboard
        {board.storyboard && (
          <span style={{ fontSize: 9, background: "#2a2a2a", borderRadius: 8, padding: "0 5px", color: "#666" }}>
            {board.panels.length}
          </span>
        )}
      </ActionBtn>

      </div>{/* end panel group */}

      <div style={{ flex: 1 }} />

      {/* Export PNG */}
      <ActionBtn
        onClick={() => window.dispatchEvent(new Event("glucose:export-png"))}
        title="Exporter le canvas en PNG"
      >
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M8 2v9M4 8l4 4 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
          <path d="M2 13h12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
        </svg>
        Export PNG
      </ActionBtn>

      {sep}

      {/* Preset */}
      <ActionBtn onClick={onTogglePreset} active={presetPanelOpen} title="Presets artistiques">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <rect x="1" y="1" width="6" height="6" rx="1" stroke="currentColor" strokeWidth="1.3"/>
          <rect x="9" y="1" width="6" height="6" rx="1" stroke="currentColor" strokeWidth="1.3"/>
          <rect x="1" y="9" width="6" height="6" rx="1" stroke="currentColor" strokeWidth="1.3"/>
          <rect x="9" y="9" width="6" height="6" rx="1" stroke="currentColor" strokeWidth="1.3"/>
        </svg>
        {activePreset ? activePreset.name : "Preset"}
      </ActionBtn>

      <ActionBtn onClick={onToggleDomains} active={domainsPanelOpen} title="Domaines sémantiques (couleurs des membranes)">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <circle cx="5" cy="6" r="3" stroke="currentColor" strokeWidth="1.3"/>
          <circle cx="11" cy="6" r="3" stroke="currentColor" strokeWidth="1.3"/>
          <circle cx="8" cy="11" r="3" stroke="currentColor" strokeWidth="1.3"/>
        </svg>
        Domaines
      </ActionBtn>

      <span style={{ fontSize: 11, color: "#3a3a3a", marginLeft: 8 }}>
        {board.images.length > 0 && `${board.images.length}img`}
      </span>
    </div>
  );
}

function ToolBtn({ tool: _tool, active, onClick, title, children }: {
  tool: Tool; active: boolean; onClick: () => void; title: string; children: React.ReactNode;
}) {
  return (
    <button onClick={onClick} title={title} style={{
      display: "flex", alignItems: "center", justifyContent: "center",
      width: 30, height: 30, borderRadius: 4, border: "none",
      cursor: "pointer",
      background: active ? "#2d2d2d" : "transparent",
      color: active ? "#fff" : "#666",
      outline: active ? "1px solid #444" : "none",
    }}>
      {children}
    </button>
  );
}

function ActionBtn({ active, onClick, title, children }: {
  active?: boolean; onClick: () => void; title: string; children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      style={{
        display: "flex", alignItems: "center", gap: 5,
        padding: "4px 10px", borderRadius: 4, border: "none",
        fontSize: 12, cursor: "pointer",
        background: active ? "#2d2d2d" : "transparent",
        color: active ? "#ccc" : "#666",
        outline: active ? "1px solid #444" : "none",
      }}
      onMouseOver={(e) => { e.currentTarget.style.color = "#ccc"; if (!active) e.currentTarget.style.background = "#1e1e1e"; }}
      onMouseOut={(e) => {
        e.currentTarget.style.color = active ? "#ccc" : "#666";
        if (!active) e.currentTarget.style.background = "transparent";
      }}
    >
      {children}
    </button>
  );
}
