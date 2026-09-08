import { useEffect, useRef, useState } from "react";

interface ToastItem {
  id: number;
  message: string;
  icon?: string;
}

let _nextId = 1;
const _listeners: ((item: ToastItem) => void)[] = [];

export function showToast(message: string, icon?: string) {
  const item: ToastItem = { id: _nextId++, message, icon };
  _listeners.forEach((fn) => fn(item));
}

// ⚠ existe en DEUX chaînes distinctes selon la source : U+26A0 seul et U+26A0
// suivi du sélecteur de variante emoji (U+FE0F). Les deux clés doivent exister,
// sinon la moitié des alertes retombe sur l'emoji couleur.
const ALERT = (
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
    <line x1="12" y1="9" x2="12" y2="13" />
    <line x1="12" y1="17" x2="12.01" y2="17" />
  </svg>
);

/** Emoji → glyphe SVG monochrome (`currentColor`). Un emoji ABSENT de cette
 *  table retombe sur son rendu système, en COULEUR — ce qui jure avec le reste
 *  de l'interface. `Toast.test.tsx` vérifie que le cas ne se reproduit pas.
 *  Exporté pour ce test uniquement. */
export const ICON_MAP: Record<string, React.ReactElement> = {
  "📌": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M16 4v5.33l3.33 3.33V16h-6v6l-1.33 1.33L10.67 22v-6H4.67v-3.34L8 9.33V4A2 2 0 0 1 10 2h4a2 2 0 0 1 2 2z" />
    </svg>
  ),
  "🗑": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" />
    </svg>
  ),
  "↩": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M9 14 4 9l5-5" /><path d="M4 9h10.5a5.5 5.5 0 0 1 5.5 5.5v0a5.5 5.5 0 0 1-5.5 5.5H11" />
    </svg>
  ),
  "↪": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M15 14l5-5-5-5" /><path d="M20 9H9.5A5.5 5.5 0 0 0 4 14.5v0A5.5 5.5 0 0 0 9.5 20H13" />
    </svg>
  ),
  "📋": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
      <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
    </svg>
  ),
  "✂": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="6" cy="6" r="3" />
      <circle cx="6" cy="18" r="3" />
      <line x1="20" y1="4" x2="8.12" y2="15.88" />
      <line x1="14.47" y1="14.48" x2="20" y2="20" />
      <line x1="8.12" y1="8.12" x2="12" y2="12" />
    </svg>
  ),
  // Alignement intelligent — MÊME glyphe que le bouton « Aimant » de la barre
  // d'outils (dessiné là-bas en viewBox 16 / trait 1.3, transposé ici en
  // viewBox 24 / trait 2 comme les autres entrées) : le toast et le bouton
  // doivent se lire comme la même fonction.
  "🧲": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 3l18 18M21 3L3 21" strokeOpacity="0.4" strokeWidth="1.5" />
      <rect x="4.5" y="4.5" width="15" height="15" rx="1.5" strokeDasharray="3 1.5" />
    </svg>
  ),
  "💾": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
      <polyline points="17 21 17 13 7 13 7 21" />
      <polyline points="7 3 7 8 15 8" />
    </svg>
  ),
  "⚠": ALERT,
  "⚠️": ALERT,
  // Ouverture dans l'application native du système.
  "🚀": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
      <polyline points="15 3 21 3 21 9" />
      <line x1="10" y1="14" x2="21" y2="3" />
    </svg>
  ),
  // Fichier réassocié / lien vivant.
  "🔗": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
    </svg>
  ),
  // Bundle portable (paquet auto-suffisant).
  "🎒": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="21 8 21 21 3 21 3 8" />
      <rect x="1" y="3" width="22" height="5" rx="1" />
      <line x1="10" y1="12" x2="14" y2="12" />
    </svg>
  ),
  // Restauration depuis un jalon de secours.
  "🛟": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <circle cx="12" cy="12" r="4" />
      <line x1="4.93" y1="4.93" x2="9.17" y2="9.17" />
      <line x1="14.83" y1="9.17" x2="19.07" y2="4.93" />
      <line x1="14.83" y1="14.83" x2="19.07" y2="19.07" />
      <line x1="9.17" y1="14.83" x2="4.93" y2="19.07" />
    </svg>
  ),
  // Compaction de l'historique (le fichier se resserre).
  "🗜️": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="4 14 10 14 10 20" />
      <polyline points="20 10 14 10 14 4" />
      <line x1="14" y1="10" x2="21" y2="3" />
      <line x1="3" y1="21" x2="10" y2="14" />
    </svg>
  ),
  // Export réussi (sort du logiciel).
  "📤": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="17 8 12 3 7 8" />
      <line x1="12" y1="3" x2="12" y2="15" />
    </svg>
  ),
  // Ancrage temporel d'un nœud.
  "📅": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
      <line x1="16" y1="2" x2="16" y2="6" />
      <line x1="8" y1="2" x2="8" y2="6" />
      <line x1="3" y1="10" x2="21" y2="10" />
    </svg>
  ),
  // Rien à gagner / déjà optimal.
  "✨": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 2.5 13.9 9.1 20.5 11 13.9 12.9 12 19.5 10.1 12.9 3.5 11 10.1 9.1 12 2.5z" />
      <path d="M19 17v3" />
      <path d="M20.5 18.5h-3" />
    </svg>
  ),
  // Version restaurée (on remonte l'historique).
  "⏮": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polygon points="19 20 9 12 19 4 19 20" />
      <line x1="5" y1="19" x2="5" y2="5" />
    </svg>
  ),
  // Opérateur logique posé sur une note.
  "⊕": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <line x1="12" y1="8" x2="12" y2="16" />
      <line x1="8" y1="12" x2="16" y2="12" />
    </svg>
  ),
  // Miroirs créés (alias vivants qui se resynchronisent).
  "↻": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="23 4 23 10 17 10" />
      <polyline points="1 20 1 14 7 14" />
      <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
    </svg>
  ),
};

export default function Toast() {
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const timers = useRef<Map<number, ReturnType<typeof setTimeout>>>(new Map());

  useEffect(() => {
    const handler = (item: ToastItem) => {
      setToasts((prev) => [...prev.slice(-4), item]);
      const t = setTimeout(() => {
        setToasts((prev) => prev.filter((x) => x.id !== item.id));
        timers.current.delete(item.id);
      }, 2200);
      timers.current.set(item.id, t);
    };
    _listeners.push(handler);
    return () => {
      const idx = _listeners.indexOf(handler);
      if (idx !== -1) _listeners.splice(idx, 1);
      timers.current.forEach((t) => clearTimeout(t));
    };
  }, []);

  if (toasts.length === 0) return null;

  return (
    <div style={{
      position: "fixed", bottom: 56, left: "50%", transform: "translateX(-50%)",
      display: "flex", flexDirection: "column", alignItems: "center", gap: 6,
      zIndex: 99999, pointerEvents: "none",
    }}>
      {toasts.map((t) => {
        const svgIcon = t.icon ? ICON_MAP[t.icon] : null;
        return (
          <div
            key={t.id}
            style={{
              background: "rgba(26,26,26,0.97)",
              border: "1px solid #2a2a2a",
              borderRadius: 6,
              padding: "7px 16px",
              fontSize: 12,
              color: "#ccc",
              display: "flex", alignItems: "center", gap: 8,
              boxShadow: "0 4px 20px rgba(0,0,0,0.6)",
              animation: "toastIn 0.18s ease-out",
              whiteSpace: "nowrap",
            }}
          >
            {svgIcon ? (
              <span style={{ display: "flex", alignItems: "center", color: "#888" }}>{svgIcon}</span>
            ) : t.icon ? (
              <span style={{ fontSize: 14 }}>{t.icon}</span>
            ) : null}
            {t.message}
          </div>
        );
      })}
      <style>{`
        @keyframes toastIn {
          from { opacity: 0; transform: translateY(8px); }
          to   { opacity: 1; transform: translateY(0); }
        }
      `}</style>
    </div>
  );
}
