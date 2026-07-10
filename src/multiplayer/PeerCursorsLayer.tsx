import React, { useEffect, useState, useRef } from "react";
import { useGlucoseStore } from "../store";
import { getCollabHandle, isCollabActive } from "./collabHandle";

// Helper to get or create a session-based random peer name & color
const ADJECTIVES = ["Bleu", "Rouge", "Vert", "Jaune", "Orange", "Violet", "Rose", "Cyan", "Indigo", "Émeraude"];
const ANIMALS = ["Renard", "Ours", "Aigle", "Chat", "Chien", "Lapin", "Loup", "Cerf", "Hibou", "Écureuil"];
const COLORS = [
  "#38bdf8", // Sky blue
  "#34d399", // Emerald
  "#fb923c", // Orange
  "#c084fc", // Purple
  "#f472b6", // Pink
  "#fbbf24", // Yellow
  "#2dd4bf", // Teal
  "#818cf8", // Indigo
  "#f87171", // Red
  "#a3e635"  // Lime
];

function getOrCreateLocalUser() {
  const user = sessionStorage.getItem("glucose:local-user");
  if (user) {
    try {
      return JSON.parse(user) as { name: string; color: string };
    } catch {}
  }
  const adj = ADJECTIVES[Math.floor(Math.random() * ADJECTIVES.length)];
  const anim = ANIMALS[Math.floor(Math.random() * ANIMALS.length)];
  const name = `${anim} ${adj}`;
  const color = COLORS[Math.floor(Math.random() * COLORS.length)];
  const newUser = { name, color };
  sessionStorage.setItem("glucose:local-user", JSON.stringify(newUser));
  return newUser;
}

interface PeerState {
  senderId: string;
  boardId: string;
  cursor?: { x: number; y: number };
  userName: string;
  userColor: string;
  selectedAnnotationIds: string[];
  selectedImageIds: string[];
  lastActive: number;
}

interface Props {
  vpRef: React.MutableRefObject<{ x: number; y: number; scale: number }>;
}

export default function PeerCursorsLayer({ vpRef }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scale, setScale] = useState(1);
  const [peers, setPeers] = useState<Record<string, PeerState>>({});

  const activeBoardId = useGlucoseStore((s) => s.activeBoardId);
  const selectedAnnotationIds = useGlucoseStore((s) => s.selectedAnnotationIds);
  const selectedImageIds = useGlucoseStore((s) => s.selectedImageIds);
  const project = useGlucoseStore((s) => s.project);

  // État collab réactif : `isCollabActive()` est un singleton hors-React, mais on
  // le relit à chaque rendu ; comme `project` change à l'entrée/sortie de collab
  // (wireHandle/leaveCollab), ce booléen suit fidèlement les transitions collab.
  const collabActive = isCollabActive();

  const board = project.boards.find((b) => b.id === activeBoardId);
  const lastCursorRef = useRef<{ x: number; y: number } | null>(null);

  // 1. Sync CSS transform with viewport (translate & scale).
  //    Ne tourne QUE en collab : en solo le composant rend `null` (aucun
  //    container), donc une boucle rAF permanente ne ferait que gaspiller du CPU
  //    à contre-courant du rendu à la demande du canvas.
  useEffect(() => {
    if (!collabActive) return;
    let rafId: number;
    function updateTransform() {
      if (containerRef.current && vpRef.current) {
        const { x, y, scale: s } = vpRef.current;
        containerRef.current.style.transform = `translate(${x}px, ${y}px) scale(${s})`;
        // La transform CSS suit chaque frame, mais on ne re-render React que si
        // l'échelle change réellement : sinon setScale(s) forçait 60 re-renders/s
        // du composant + enfants pour rien. Renvoyer `prev` fait bailout React.
        setScale((prev) => (Math.abs(prev - s) < 0.001 ? prev : s));
      }
      rafId = requestAnimationFrame(updateTransform);
    }
    updateTransform();
    return () => cancelAnimationFrame(rafId);
  }, [vpRef, collabActive]);

  // 2. Track & broadcast local cursor position (throttled at ~40ms / 25fps)
  useEffect(() => {
    let lastSend = 0;

    const onPointerMove = (e: PointerEvent) => {
      if (!isCollabActive()) return;
      const handle = getCollabHandle();
      if (!handle) return;

      const now = Date.now();
      if (now - lastSend < 40) return;
      lastSend = now;

      const canvas = document.querySelector("canvas");
      if (!canvas) return;
      const rect = canvas.getBoundingClientRect();
      const vx = vpRef.current.x;
      const vy = vpRef.current.y;
      const vs = vpRef.current.scale;

      const worldX = (e.clientX - rect.left - vx) / vs;
      const worldY = (e.clientY - rect.top - vy) / vs;

      const cursor = { x: worldX, y: worldY };
      lastCursorRef.current = cursor;

      const localUser = getOrCreateLocalUser();
      handle.broadcast({
        type: "presence",
        boardId: activeBoardId,
        cursor,
        userName: localUser.name,
        userColor: localUser.color,
        selectedAnnotationIds,
        selectedImageIds,
      });
    };

    window.addEventListener("pointermove", onPointerMove);
    return () => window.removeEventListener("pointermove", onPointerMove);
  }, [activeBoardId, selectedAnnotationIds, selectedImageIds, vpRef]);

  // 3. Broadcast instant update when local selection changes
  useEffect(() => {
    if (!isCollabActive() || !lastCursorRef.current) return;
    const handle = getCollabHandle();
    if (!handle) return;

    const localUser = getOrCreateLocalUser();
    handle.broadcast({
      type: "presence",
      boardId: activeBoardId,
      cursor: lastCursorRef.current,
      userName: localUser.name,
      userColor: localUser.color,
      selectedAnnotationIds,
      selectedImageIds,
    });
  }, [selectedAnnotationIds, selectedImageIds, activeBoardId]);

  // 4. Listen to peer presence updates via Automerge ephemeral channels
  useEffect(() => {
    const handle = getCollabHandle();
    if (!handle) {
      setPeers({});
      return;
    }

    const onEphemeralMessage = (event: any) => {
      const payload = event.message as any;
      if (payload?.type === "presence") {
        setPeers((prev) => ({
          ...prev,
          [event.senderId]: {
            senderId: event.senderId,
            boardId: payload.boardId,
            cursor: payload.cursor,
            userName: payload.userName,
            userColor: payload.userColor,
            selectedAnnotationIds: payload.selectedAnnotationIds ?? [],
            selectedImageIds: payload.selectedImageIds ?? [],
            lastActive: Date.now(),
          },
        }));
      }
    };

    handle.on("ephemeral-message", onEphemeralMessage);

    // Prune stale cursors after 5 seconds of inactivity
    const timer = setInterval(() => {
      const threshold = Date.now() - 5000;
      setPeers((prev) => {
        let changed = false;
        const next = { ...prev };
        for (const [key, peer] of Object.entries(next)) {
          if (peer.lastActive < threshold) {
            delete next[key];
            changed = true;
          }
        }
        return changed ? next : prev;
      });
    }, 1000);

    return () => {
      handle.off("ephemeral-message", onEphemeralMessage);
      clearInterval(timer);
    };
    // (dés)abonnement aux SEULES transitions collab (host/join/leave), pas à
    // chaque édition du doc — le handle automerge-repo est stable pendant une
    // session (repo.find renvoie la même instance, même après reconnexion).
  }, [collabActive]);

  if (!collabActive || !board) return null;

  // Active peers on the current board
  const activePeers = Object.values(peers).filter(
    (p) => p.boardId === activeBoardId && p.lastActive > Date.now() - 5000
  );

  return (
    <div
      ref={containerRef}
      style={{
        position: "absolute",
        left: 0,
        top: 0,
        width: 0,
        height: 0,
        transformOrigin: "0 0",
        pointerEvents: "none",
        zIndex: 55, // sits between normal elements and edit overlay
      }}
    >
      {/* ── Draw remote selection outline frames ── */}
      {activePeers.map((peer) => {
        const selections: React.ReactNode[] = [];

        // 1. Remote Annotation Selections
        peer.selectedAnnotationIds.forEach((id) => {
          const ann = board.annotations.find((a) => a.id === id);
          if (!ann) return;

          // Arrow has no simple width/height, skip or handle separately if needed
          if (ann.type === "arrow") return;

          const w = ann.width ?? 160;
          const h = ann.height ?? 120;

          selections.push(
            <div
              key={`sel-ann-${peer.senderId}-${ann.id}`}
              style={{
                position: "absolute",
                left: ann.x - 4,
                top: ann.y - 4,
                width: w + 8,
                height: h + 8,
                border: `2px dashed ${peer.userColor}`,
                borderRadius: ann.type === "text" ? 32 : 12,
                boxShadow: `0 0 12px ${peer.userColor}33`,
                transition: "all 0.15s ease-out",
                pointerEvents: "none",
              }}
            >
              {/* Floating tag showing who is selecting it */}
              <div
                style={{
                  position: "absolute",
                  left: -2,
                  top: -20,
                  background: peer.userColor,
                  color: "#000",
                  padding: "1px 6px",
                  borderRadius: 4,
                  fontSize: 9,
                  fontWeight: 700,
                  lineHeight: 1.2,
                  boxShadow: "0 2px 4px rgba(0,0,0,0.2)",
                  whiteSpace: "nowrap",
                  transformOrigin: "0 100%",
                  transform: `scale(${Math.max(0.6, Math.min(1.2, 1 / scale))})`,
                }}
              >
                ✏️ {peer.userName}
              </div>
            </div>
          );
        });

        // 2. Remote Image Selections
        peer.selectedImageIds.forEach((id) => {
          const img = board.images.find((i) => i.id === id);
          if (!img) return;

          selections.push(
            <div
              key={`sel-img-${peer.senderId}-${img.id}`}
              style={{
                position: "absolute",
                // Les sprites image sont ANCRÉS AU CENTRE (anchor 0.5) : (img.x, img.y)
                // = centre, pas coin haut-gauche (cf. GlucoseCanvas). Le coin haut-gauche
                // en coords monde = (img.x - width/2, img.y - height/2). Sans ce demi-
                // décalage, la box d'un pair s'affichait toujours en bas-droite de l'image
                // réelle (bug « box qui entoure du vide »). Les annotations, elles, SONT
                // ancrées au coin → leur box (ann.x - 4) reste correcte.
                left: img.x - img.width / 2 - 4,
                top: img.y - img.height / 2 - 4,
                width: img.width + 8,
                height: img.height + 8,
                border: `2px dashed ${peer.userColor}`,
                borderRadius: 8,
                boxShadow: `0 0 12px ${peer.userColor}33`,
                transform: `rotate(${img.rotation || 0}rad)`,
                transformOrigin: "center center",
                transition: "all 0.15s ease-out",
                pointerEvents: "none",
              }}
            >
              <div
                style={{
                  position: "absolute",
                  left: -2,
                  top: -20,
                  background: peer.userColor,
                  color: "#000",
                  padding: "1px 6px",
                  borderRadius: 4,
                  fontSize: 9,
                  fontWeight: 700,
                  lineHeight: 1.2,
                  boxShadow: "0 2px 4px rgba(0,0,0,0.2)",
                  whiteSpace: "nowrap",
                  transformOrigin: "0 100%",
                  transform: `scale(${Math.max(0.6, Math.min(1.2, 1 / scale))})`,
                }}
              >
                🖼️ {peer.userName}
              </div>
            </div>
          );
        });

        return <React.Fragment key={`peer-selections-${peer.senderId}`}>{selections}</React.Fragment>;
      })}

      {/* ── Draw remote cursors ── */}
      {activePeers.map((peer) => {
        if (!peer.cursor) return null;

        const cursorScale = Math.max(0.4, Math.min(1.5, 1 / scale));

        return (
          <div
            key={`cursor-${peer.senderId}`}
            style={{
              position: "absolute",
              left: peer.cursor.x,
              top: peer.cursor.y,
              width: 0,
              height: 0,
              zIndex: 100,
              pointerEvents: "none",
            }}
          >
            {/* The cursor arrow & label container */}
            <div
              style={{
                position: "absolute",
                left: 0,
                top: 0,
                transform: `scale(${cursorScale})`,
                transformOrigin: "0 0",
                display: "flex",
                flexDirection: "column",
                alignItems: "flex-start",
                gap: 4,
              }}
            >
              {/* Custom SVG cursor arrow matching peer's color */}
              <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                style={{
                  filter: "drop-shadow(0 2px 4px rgba(0,0,0,0.35))",
                }}
              >
                <path
                  d="M4.5 3V20.5L10 15L18.5 15L4.5 3Z"
                  fill={peer.userColor}
                  stroke="#ffffff"
                  strokeWidth="1.5"
                  strokeLinejoin="round"
                />
              </svg>

              {/* Cursor tooltip name label */}
              <div
                style={{
                  background: peer.userColor,
                  color: "#000",
                  fontFamily: "system-ui, -apple-system, sans-serif",
                  fontSize: 10,
                  fontWeight: 700,
                  padding: "3px 8px",
                  borderRadius: 6,
                  boxShadow: "0 3px 8px rgba(0,0,0,0.35)",
                  whiteSpace: "nowrap",
                  marginTop: -2,
                  marginLeft: 8,
                }}
              >
                {peer.userName}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
