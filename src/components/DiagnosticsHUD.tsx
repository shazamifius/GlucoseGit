// ────────────────────────────────────────────────────────────────────────────
// HUD de diagnostic local — Ctrl+Shift+D
// ────────────────────────────────────────────────────────────────────────────
//
// Overlay LOCAL (aucun réseau, aucune télémétrie) qui affiche en direct la santé
// du rendu : FPS réel, pire frame, gels, + le renderer GPU et le matériel. C'est
// l'outil pour diagnostiquer le lag sur place — en particulier « pourquoi Niri
// rame » : si le renderer est un rendu logiciel (llvmpipe), la ligne devient rouge
// et tout s'explique.
//
// Toujours disponible (indépendant du consentement télémétrie) : c'est purement
// local et sert au débogage immédiat.

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  startPerfMonitor,
  getPerfSnapshot,
  getGpuInfo,
  type PerfSnapshot,
  type GpuInfo,
} from "../telemetry/perfMonitor";
import { getConsentState, setTelemetryConsent, type Consent } from "../telemetry/telemetry";

interface Specs {
  ram_gb: number;
  cores: number;
  vram_gb: number | null;
}

function osLabel(): string {
  const ua = navigator.userAgent;
  if (/windows/i.test(ua)) return "Windows";
  if (/mac os|macintosh/i.test(ua)) return "macOS";
  if (/linux|x11/i.test(ua)) return "Linux";
  return "?";
}

function Row({ k, v, bad, good }: { k: string; v: string; bad?: boolean; good?: boolean }) {
  const color = bad ? "#f87171" : good ? "#4ade80" : "#d4d4dd";
  return (
    <div style={{ display: "flex", justifyContent: "space-between", gap: 16 }}>
      <span>{k}</span>
      <span style={{ color, textAlign: "right", maxWidth: 220, wordBreak: "break-word" }}>{v}</span>
    </div>
  );
}

export default function DiagnosticsHUD() {
  const [open, setOpen] = useState(false);
  const [perf, setPerf] = useState<PerfSnapshot | null>(null);
  const [gpu, setGpu] = useState<GpuInfo | null>(null);
  const [wayland, setWayland] = useState<boolean | null>(null);
  const [specs, setSpecs] = useState<Specs | null>(null);
  const [consent, setConsent] = useState<Consent>(() => getConsentState());

  // La mesure tourne dès le montage (idempotent) — même HUD fermé, pour que
  // l'ouverture montre tout de suite des chiffres significatifs.
  useEffect(() => {
    startPerfMonitor();
    setGpu(getGpuInfo());
    invoke<boolean>("is_wayland").then(setWayland).catch(() => setWayland(null));
    invoke<Specs>("system_specs").then(setSpecs).catch(() => setSpecs(null));
  }, []);

  // Bascule au clavier — Ctrl+Shift+D (D = Diagnostic). Évite les champs de saisie.
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "d" || e.key === "D")) {
        e.preventDefault();
        setOpen((v) => !v);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Rafraîchit les compteurs 2×/s quand le HUD est ouvert.
  useEffect(() => {
    if (!open) return;
    setPerf(getPerfSnapshot());
    const id = setInterval(() => setPerf(getPerfSnapshot()), 500);
    return () => clearInterval(id);
  }, [open]);

  if (!open) return null;

  const dpr = Math.round((window.devicePixelRatio || 1) * 100) / 100;
  const screen = `${window.innerWidth}×${window.innerHeight} @${dpr}×`;
  const fpsBad = perf != null && perf.fps > 0 && perf.fps < 30;
  const worstBad = perf != null && perf.worstMs > JANK_UI_MS;

  return (
    <div
      style={{
        position: "fixed",
        top: 64,
        right: 12,
        zIndex: 2000,
        background: "#0d0d0dee",
        border: "1px solid #26262e",
        borderRadius: 6,
        padding: "8px 10px",
        minWidth: 240,
        font: "11px/1.55 ui-monospace, SFMono-Regular, Menlo, monospace",
        color: "#9a9aa0",
        pointerEvents: "none",
        userSelect: "none",
      }}
    >
      <div style={{ color: "#d4d4dd", marginBottom: 5, letterSpacing: "0.05em" }}>
        <span
          style={{
            display: "inline-block",
            width: 8,
            height: 8,
            borderRadius: 8,
            background: gpu?.software || fpsBad ? "#f87171" : "#4ade80",
            marginRight: 6,
            verticalAlign: "middle",
          }}
        />
        DIAGNOSTIC · Ctrl+Shift+D
      </div>

      <Row k="FPS" v={perf ? String(perf.fps) : "…"} bad={fpsBad} good={perf != null && perf.fps >= 55} />
      <Row k="Pire frame" v={perf ? `${perf.worstMs} ms` : "…"} bad={worstBad} />
      <Row k="Médiane" v={perf ? `${perf.medianMs} ms` : "…"} />
      <Row k="Frames lentes" v={perf ? String(perf.jankFrames) : "…"} bad={perf != null && perf.jankFrames > 0} />
      <Row k="Gels (>200ms)" v={perf ? String(perf.stalls) : "…"} bad={perf != null && perf.stalls > 0} />

      <div style={{ height: 1, background: "#26262e", margin: "6px 0" }} />

      <Row
        k="GPU"
        v={gpu ? gpu.renderer : "…"}
        bad={gpu?.software}
        good={gpu != null && !gpu.software && gpu.renderer !== "inconnu"}
      />
      {gpu?.software && <Row k="⚠ Rendu" v="LOGICIEL (CPU)" bad />}
      <Row k="WebGL" v={gpu ? gpu.api : "…"} bad={gpu?.api === "none"} />
      <Row k="Affichage" v={wayland == null ? "?" : wayland ? "Wayland" : "X11/natif"} />

      <div style={{ height: 1, background: "#26262e", margin: "6px 0" }} />

      <Row k="OS" v={osLabel()} />
      <Row k="RAM" v={specs ? `${specs.ram_gb} Go` : "…"} />
      <Row k="Cœurs" v={specs ? String(specs.cores) : "…"} />
      {specs?.vram_gb != null && <Row k="VRAM" v={`${specs.vram_gb} Go`} />}
      <Row k="Écran" v={screen} />

      <div style={{ height: 1, background: "#26262e", margin: "6px 0" }} />

      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 16 }}>
        <span>Télémétrie</span>
        <button
          type="button"
          onClick={() => {
            const grant = consent !== "granted";
            setTelemetryConsent(grant);
            setConsent(grant ? "granted" : "denied");
          }}
          style={{
            pointerEvents: "auto",
            cursor: "pointer",
            background: "transparent",
            border: "1px solid #34343e",
            borderRadius: 4,
            padding: "1px 7px",
            font: "inherit",
            color: consent === "granted" ? "#4ade80" : "#9a9aa0",
          }}
        >
          {consent === "granted" ? "activée" : "désactivée"}
        </button>
      </div>
    </div>
  );
}

// Au-delà de ~33 ms (sous 30 fps) une frame est ressentie comme un à-coup.
const JANK_UI_MS = 33;
