import { useGlucoseStore } from "../store";

interface Props {
  onOpen: () => void;
}

// CLEANUP P-08 — Selectors atomiques : un re-render seulement quand
// LA valeur lue change (pas à chaque mutation du store).
export default function PomodoroOverlay({ onOpen }: Props) {
  const pomodoroLeft = useGlucoseStore(s => s.pomodoroLeft);
  const pomodoroRunning = useGlucoseStore(s => s.pomodoroRunning);
  const pomodoroDone = useGlucoseStore(s => s.pomodoroDone);

  if (!pomodoroRunning && !pomodoroDone) return null;

  const mm = String(Math.floor(pomodoroLeft / 60)).padStart(2, "0");
  const ss = String(pomodoroLeft % 60).padStart(2, "0");

  return (
    <div
      onClick={onOpen}
      title="Ouvrir le Pomodoro"
      style={{
        position: "fixed", top: 10, left: "50%", transform: "translateX(-50%)",
        zIndex: 500,
        background: "rgba(13,13,13,0.75)",
        border: `1px solid ${pomodoroDone ? "#4ade80" : "#60a5fa"}`,
        borderRadius: 20,
        padding: "3px 12px",
        display: "flex", alignItems: "center", gap: 6,
        cursor: "pointer",
        backdropFilter: "blur(6px)",
        fontSize: 12,
        color: pomodoroDone ? "#4ade80" : "#aaa",
        fontVariantNumeric: "tabular-nums",
        userSelect: "none",
        transition: "border-color 0.3s",
      }}
    >
      <span style={{ fontSize: 10, opacity: 0.6 }}>{pomodoroDone ? "✓" : "⏱"}</span>
      {pomodoroDone ? "Terminé !" : `${mm}:${ss}`}
    </div>
  );
}
