import { useGlucoseStore } from "../store";

const PRESETS = [
  { label: "25 min", seconds: 25 * 60 },
  { label: "15 min", seconds: 15 * 60 },
  { label: "5 min",  seconds:  5 * 60 },
];

export default function PomodoroTimer() {
  const {
    pomodoroTotal, pomodoroLeft, pomodoroRunning, pomodoroDone,
    pomodoroStart, pomodoroPause, pomodoroReset,
  } = useGlucoseStore();

  const mm = String(Math.floor(pomodoroLeft / 60)).padStart(2, "0");
  const ss = String(pomodoroLeft % 60).padStart(2, "0");
  const progress = pomodoroTotal > 0 ? 1 - pomodoroLeft / pomodoroTotal : 1;
  const R = 30;
  const circ = 2 * Math.PI * R;
  const dash = circ * (1 - progress);
  const SIZE = 80;

  function handleToggle() {
    if (pomodoroDone) { pomodoroReset(pomodoroTotal); return; }
    if (pomodoroRunning) pomodoroPause(); else pomodoroStart();
  }

  return (
    <div style={{
      background: "#111", border: "1px solid #222", borderRadius: 8,
      padding: "18px 18px 14px 18px",
      display: "flex", flexDirection: "column", alignItems: "center", gap: 10,
      minWidth: 160,
    }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", width: "100%", gap: 8 }}>
        <span style={{ fontSize: 10, color: "#555", textTransform: "uppercase", letterSpacing: 1 }}>Pomodoro</span>
      </div>

      {/* Ring + Time overlay */}
      <div style={{ position: "relative", width: SIZE, height: SIZE }}>
        <svg width={SIZE} height={SIZE} style={{ transform: "rotate(-90deg)" }}>
          <circle cx={SIZE/2} cy={SIZE/2} r={R} fill="none" stroke="#1e1e1e" strokeWidth={5} />
          <circle
            cx={SIZE/2} cy={SIZE/2} r={R} fill="none"
            stroke={pomodoroDone ? "#4ade80" : "#60a5fa"}
            strokeWidth={5}
            strokeDasharray={circ}
            strokeDashoffset={dash}
            strokeLinecap="round"
            style={{ transition: "stroke-dashoffset 0.5s linear" }}
          />
        </svg>
        <div style={{
          position: "absolute", inset: 0,
          display: "flex", alignItems: "center", justifyContent: "center",
          fontSize: pomodoroDone ? 22 : 17,
          fontWeight: 700,
          letterSpacing: 1,
          color: pomodoroDone ? "#4ade80" : "#ccc",
          fontVariantNumeric: "tabular-nums",
        }}>
          {pomodoroDone ? "✓" : `${mm}:${ss}`}
        </div>
      </div>

      {/* Controls */}
      <div style={{ display: "flex", gap: 6 }}>
        <button
          onClick={handleToggle}
          style={{
            padding: "4px 14px", fontSize: 11, borderRadius: 4, cursor: "pointer",
            background: "#1e1e1e", border: "1px solid #333", color: "#ccc",
          }}
        >
          {pomodoroDone ? "Relancer" : pomodoroRunning ? "Pause" : "Démarrer"}
        </button>
        <button
          onClick={() => pomodoroReset(pomodoroTotal)}
          style={{
            padding: "4px 10px", fontSize: 11, borderRadius: 4, cursor: "pointer",
            background: "transparent", border: "1px solid #2a2a2a", color: "#555",
          }}
        >
          ↺
        </button>
      </div>

      {/* Presets */}
      <div style={{ display: "flex", gap: 4 }}>
        {PRESETS.map((p) => (
          <button
            key={p.label}
            onClick={() => pomodoroReset(p.seconds)}
            style={{
              padding: "2px 8px", fontSize: 10, borderRadius: 3, cursor: "pointer",
              background: pomodoroTotal === p.seconds ? "#2a2a2a" : "transparent",
              border: `1px solid ${pomodoroTotal === p.seconds ? "#444" : "#1e1e1e"}`,
              color: pomodoroTotal === p.seconds ? "#aaa" : "#444",
            }}
          >
            {p.label}
          </button>
        ))}
      </div>
    </div>
  );
}
