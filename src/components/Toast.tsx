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

const ICON_MAP: Record<string, React.ReactElement> = {
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
  "💾": (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
      <polyline points="17 21 17 13 7 13 7 21" />
      <polyline points="7 3 7 8 15 8" />
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
