import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { useGlucoseStore } from "../store";
import { showToast } from "./Toast";
import { exportProject, ExportFormat, FORMAT_META } from "../utils/export";
import { exportPortableBundle } from "../utils/bundleActions";

// Menu « Exporter ▾ » — remplace l'ancien bouton PNG unique. Propose 4 formats
// auto-suffisants, partageables sans installer Glucose :
//   • HTML interactif (pan/zoom + descriptions, 1 fichier ouvrable au navigateur)
//   • PNG HD plein-board (rendu fidèle, contrairement à l'ancien screenshot)
//   • SVG vectoriel (zoom infini, texte sélectionnable)
//   • Markdown (texte structuré, ré-éditable)

interface FormatRow {
  format: ExportFormat;
  desc: string;
  icon: React.ReactNode;
}

const ROWS: FormatRow[] = [
  {
    format: "html",
    desc: "Partage zéro-install : pan/zoom & flèches dans le navigateur",
    icon: <path d="M2 4h12v8H2z M2 6.5h12" stroke="currentColor" strokeWidth="1.2" fill="none" strokeLinejoin="round" />,
  },
  {
    format: "png",
    desc: "Board entier en haute résolution",
    icon: <><rect x="2" y="3" width="12" height="10" rx="1.5" stroke="currentColor" strokeWidth="1.2" fill="none" /><circle cx="6" cy="6.5" r="1.2" fill="currentColor" /><path d="M3 12l3.5-4 2.5 2.5L11 8l2 2.5" stroke="currentColor" strokeWidth="1.2" fill="none" strokeLinejoin="round" /></>,
  },
  {
    format: "svg",
    desc: "Vectoriel : net à tout zoom, texte sélectionnable",
    icon: <><circle cx="4" cy="4" r="1.5" stroke="currentColor" strokeWidth="1.2" fill="none" /><circle cx="12" cy="12" r="1.5" stroke="currentColor" strokeWidth="1.2" fill="none" /><path d="M5 5l6 6" stroke="currentColor" strokeWidth="1.2" /></>,
  },
  {
    format: "markdown",
    desc: "Texte structuré, ré-éditable (titres, liens)",
    icon: <><rect x="1.5" y="4" width="13" height="8" rx="1" stroke="currentColor" strokeWidth="1.2" fill="none" /><path d="M4 10V6l2 2 2-2v4 M11 6v4 M9.5 8.5L11 10l1.5-1.5" stroke="currentColor" strokeWidth="1.1" fill="none" strokeLinejoin="round" strokeLinecap="round" /></>,
  },
];

export default function ExportMenu() {
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState<ExportFormat | null>(null);
  const [bundleBusy, setBundleBusy] = useState(false);
  const [pos, setPos] = useState<{ top: number; left: number; maxHeight: number } | null>(null);
  const project = useGlucoseStore((s) => s.project);
  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);

  const MENU_W = 300;
  const GAP = 6;
  const EDGE = 8;

  // Place le menu SOUS le bouton et le garde DANS la fenêtre (clamp horizontal +
  // hauteur max ; la Toolbar est en haut → on ouvre toujours vers le bas).
  const place = useCallback(() => {
    const t = triggerRef.current;
    if (!t) return;
    const r = t.getBoundingClientRect();
    const vw = window.innerWidth, vh = window.innerHeight;
    const left = Math.min(Math.max(r.right - MENU_W, EDGE), Math.max(EDGE, vw - MENU_W - EDGE));
    const top = r.bottom + GAP;
    const maxHeight = Math.max(140, vh - top - EDGE);
    setPos({ top, left, maxHeight });
  }, []);

  useLayoutEffect(() => {
    if (open) place();
    else setPos(null);
  }, [open, place]);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") setOpen(false); };
    const onReflow = () => place();
    window.addEventListener("mousedown", onDown);
    window.addEventListener("keydown", onKey);
    window.addEventListener("resize", onReflow);
    window.addEventListener("scroll", onReflow, true);
    return () => {
      window.removeEventListener("mousedown", onDown);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("resize", onReflow);
      window.removeEventListener("scroll", onReflow, true);
    };
  }, [open, place]);

  async function handleExport(format: ExportFormat) {
    if (busy) return;
    setBusy(format);
    try {
      const path = await exportProject(project, format);
      if (path) {
        const file = path.split(/[\\/]/).pop() || path;
        showToast(`Exporté : ${file}`, "📤");
      }
    } catch (err) {
      showToast(`Échec export ${FORMAT_META[format].label} : ${(err as Error)?.message || err}`, "⚠");
    } finally {
      setBusy(null);
      setOpen(false);
    }
  }

  async function handleBundle() {
    if (busy || bundleBusy) return;
    setBundleBusy(true);
    try {
      await exportPortableBundle();
    } catch (err) {
      showToast(`Échec bundle portable : ${(err as Error)?.message || err}`, "⚠");
    } finally {
      setBundleBusy(false);
      setOpen(false);
    }
  }

  return (
    <div ref={rootRef} style={{ position: "relative" }}>
      <button
        ref={triggerRef}
        onClick={() => setOpen((v) => !v)}
        title="Exporter le board (partageable sans Glucose)"
        style={{
          display: "flex", alignItems: "center", gap: 5,
          padding: "4px 10px", borderRadius: 4, border: "none",
          fontSize: 12, cursor: "pointer",
          background: open ? "#2d2d2d" : "transparent",
          color: open ? "#ccc" : "#666",
          outline: open ? "1px solid #444" : "none",
          whiteSpace: "nowrap",
        }}
        onMouseOver={(e) => { e.currentTarget.style.color = "#ccc"; if (!open) e.currentTarget.style.background = "#1e1e1e"; }}
        onMouseOut={(e) => { e.currentTarget.style.color = open ? "#ccc" : "#666"; if (!open) e.currentTarget.style.background = "transparent"; }}
      >
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M8 2v9M4 8l4 4 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
          <path d="M2 13h12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
        </svg>
        Exporter
        <span style={{ fontSize: 9, opacity: 0.7 }}>▾</span>
      </button>

      {open && pos && (
        <div
          style={{
            position: "fixed", top: pos.top, left: pos.left,
            width: MENU_W, maxHeight: pos.maxHeight, overflowY: "auto",
            background: "#16161a",
            border: "1px solid #34343e", borderRadius: 9,
            boxShadow: "0 10px 30px rgba(0,0,0,0.55)", padding: 6, zIndex: 1000,
          }}
        >
          <div style={{ fontSize: 10, color: "#6a6a78", padding: "4px 8px 6px", letterSpacing: 0.4, textTransform: "uppercase" }}>
            Partager sans Glucose
          </div>
          {ROWS.map((row) => {
            const meta = FORMAT_META[row.format];
            const isBusy = busy === row.format;
            return (
              <button
                key={row.format}
                disabled={!!busy}
                onClick={() => handleExport(row.format)}
                style={{
                  display: "flex", alignItems: "flex-start", gap: 10, width: "100%",
                  textAlign: "left", padding: "8px 9px", borderRadius: 6,
                  border: "none", background: "transparent",
                  color: busy && !isBusy ? "#555" : "#d4d4dd",
                  cursor: busy ? "default" : "pointer",
                }}
                onMouseOver={(e) => { if (!busy) e.currentTarget.style.background = "#23232b"; }}
                onMouseOut={(e) => { e.currentTarget.style.background = "transparent"; }}
              >
                <svg width="16" height="16" viewBox="0 0 16 16" style={{ marginTop: 1, flexShrink: 0, color: "#9a9aa0" }}>
                  {row.icon}
                </svg>
                <span style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                  <span style={{ fontSize: 12.5, fontWeight: 600 }}>
                    {meta.label}{isBusy ? " — export…" : ""}
                  </span>
                  <span style={{ fontSize: 11, color: "#7d7d8c", lineHeight: 1.35 }}>{row.desc}</span>
                </span>
              </button>
            );
          })}

          <div style={{ height: 1, background: "#26262e", margin: "6px 4px" }} />
          <div style={{ fontSize: 10, color: "#6a6a78", padding: "2px 8px 6px", letterSpacing: 0.4, textTransform: "uppercase" }}>
            Copie portable (ré-ouvrable)
          </div>
          <button
            disabled={busy != null || bundleBusy}
            onClick={handleBundle}
            style={{
              display: "flex", alignItems: "flex-start", gap: 10, width: "100%",
              textAlign: "left", padding: "8px 9px", borderRadius: 6,
              border: "none", background: "transparent",
              color: busy != null ? "#555" : "#d4d4dd",
              cursor: busy != null ? "default" : "pointer",
            }}
            onMouseOver={(e) => { if (busy == null && !bundleBusy) e.currentTarget.style.background = "#23232b"; }}
            onMouseOut={(e) => { e.currentTarget.style.background = "transparent"; }}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" style={{ marginTop: 1, flexShrink: 0, color: "#9a9aa0" }}>
              <path d="M4 5V4a2 2 0 0 1 4 0v1 M2.5 5h11l-.6 8H3.1z" stroke="currentColor" strokeWidth="1.2" fill="none" strokeLinejoin="round" strokeLinecap="round" />
            </svg>
            <span style={{ display: "flex", flexDirection: "column", gap: 2 }}>
              <span style={{ fontSize: 12.5, fontWeight: 600 }}>
                Projet portable{bundleBusy ? " — export…" : ""}
              </span>
              <span style={{ fontSize: 11, color: "#7d7d8c", lineHeight: 1.35 }}>
                Dossier auto-suffisant (doc + images) — survit au déplacement / changement de PC
              </span>
            </span>
          </button>
        </div>
      )}
    </div>
  );
}
