// R-FIL — Animation de lancement d'app native.
//
// Quand on double-clique une tuile fichier (sourceFile) → openSourceFile
// dispatche `glucose:app-launching` { path }. Cet overlay affiche, pendant
// ~2.6 s, le LOGO de l'app dans sa COULEUR DOMINANTE (Blender = orange, etc.),
// flottant et pulsant, avec « Lancement de {app}… ». Ça donne un signal clair
// que l'app démarre — même si elle met 10-40 s à apparaître (cas Blender).
//
// Aucune interaction (pointer-events: none) : on ne bloque pas le canvas.

import { useEffect, useState } from "react";
import AppBridgeIcon, { getAppDef } from "./AppBridgeIcon";

interface Launch {
  id: number;
  path: string;
  name: string;
  color: string;
}

const DURATION = 2600; // ms

export default function AppLaunchOverlay() {
  const [launches, setLaunches] = useState<Launch[]>([]);

  useEffect(() => {
    let seq = 0;
    const onLaunch = (e: Event) => {
      const detail = (e as CustomEvent).detail as { path?: string };
      if (!detail?.path) return;
      const def = getAppDef(detail.path);
      const id = ++seq;
      const name = detail.path.split(/[\\/]/).pop() || "Fichier";
      setLaunches((prev) => [...prev, { id, path: detail.path!, name, color: def.bg }]);
      window.setTimeout(() => {
        setLaunches((prev) => prev.filter((l) => l.id !== id));
      }, DURATION);
    };
    window.addEventListener("glucose:app-launching", onLaunch);
    return () => window.removeEventListener("glucose:app-launching", onLaunch);
  }, []);

  if (launches.length === 0) return null;

  // On n'affiche que le lancement le plus récent (empilés sinon illisible).
  const cur = launches[launches.length - 1];
  const def = getAppDef(cur.path);

  return (
    <>
      <style>{`
        @keyframes glucoseLaunchFloat {
          0%   { transform: translateY(14px) scale(0.82); opacity: 0; }
          18%  { transform: translateY(0) scale(1); opacity: 1; }
          78%  { transform: translateY(-6px) scale(1); opacity: 1; }
          100% { transform: translateY(-22px) scale(1.04); opacity: 0; }
        }
        @keyframes glucoseLaunchGlow {
          0%, 100% { opacity: 0.35; }
          50%      { opacity: 0.7; }
        }
      `}</style>
      <div
        style={{
          position: "fixed",
          inset: 0,
          pointerEvents: "none",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          zIndex: 80,
        }}
      >
        {/* Halo couleur dominante de l'app */}
        <div
          style={{
            position: "absolute",
            width: 520,
            height: 520,
            borderRadius: "50%",
            background: `radial-gradient(circle, ${cur.color}55 0%, ${cur.color}22 38%, transparent 70%)`,
            animation: `glucoseLaunchGlow ${DURATION}ms ease-in-out`,
            filter: "blur(8px)",
          }}
        />
        {/* Carte flottante : logo + nom app + fichier */}
        <div
          style={{
            position: "relative",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: 14,
            padding: "30px 40px",
            borderRadius: 22,
            background: `radial-gradient(circle at 50% 30%, ${cur.color}26 0%, #12121a 75%)`,
            border: `1px solid ${cur.color}66`,
            boxShadow: `0 0 50px ${cur.color}55, 0 18px 50px rgba(0,0,0,0.55)`,
            animation: `glucoseLaunchFloat ${DURATION}ms cubic-bezier(0.22,1,0.36,1) forwards`,
          }}
        >
          <div style={{ filter: `drop-shadow(0 0 18px ${cur.color}aa)` }}>
            <AppBridgeIcon filePath={cur.path} size={84} />
          </div>
          <div style={{ textAlign: "center" }}>
            <div style={{
              fontSize: 16, fontWeight: 700, color: "#fff",
              fontFamily: "system-ui, sans-serif", letterSpacing: 0.3,
            }}>
              Lancement de {def.name}…
            </div>
            <div style={{
              marginTop: 4, fontSize: 12, color: "#aab",
              fontFamily: "system-ui, sans-serif",
              maxWidth: 320, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
            }}>
              {cur.name}
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
