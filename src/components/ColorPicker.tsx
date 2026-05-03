import { useEffect, useLayoutEffect, useRef, useState, useCallback } from "react";

// ── Color math ────────────────────────────────────────────────
function hsvToRgb(h: number, s: number, v: number): [number, number, number] {
  const hi = Math.floor(h / 60) % 6;
  const f  = h / 60 - Math.floor(h / 60);
  const p  = v * (1 - s);
  const q  = v * (1 - f * s);
  const t  = v * (1 - (1 - f) * s);
  const table: [number, number, number][] = [
    [v,t,p],[q,v,p],[p,v,t],[p,q,v],[t,p,v],[v,p,q],
  ];
  const [r, g, b] = table[hi];
  return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
}

function rgbToHsv(r: number, g: number, b: number): [number, number, number] {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  const d = max - min;
  const s = max === 0 ? 0 : d / max;
  const v = max;
  let h = 0;
  if (d !== 0) {
    if      (max === r) h = ((g - b) / d + 6) % 6;
    else if (max === g) h = (b - r) / d + 2;
    else                h = (r - g) / d + 4;
    h *= 60;
  }
  return [h, s, v];
}

export function hexToRgb(hex: string): [number, number, number] {
  const clean = hex.replace("#", "").padEnd(6, "0").slice(0, 6);
  const n = parseInt(clean, 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

export function rgbToHex(r: number, g: number, b: number): string {
  return "#" + [r, g, b].map((x) => x.toString(16).padStart(2, "0")).join("");
}

function hexToHsv(hex: string): [number, number, number] {
  const [r, g, b] = hexToRgb(hex.startsWith("#") ? hex : "#" + hex);
  return rgbToHsv(r, g, b);
}

// ── Constants ─────────────────────────────────────────────────
const W = 196;   // wheel size
const SW = 20;   // slider width
const SH = W;    // slider height = wheel size
const R  = W / 2 - 3;  // wheel radius
const CX = W / 2;
const CY = W / 2;

// ── Component ─────────────────────────────────────────────────
interface Props {
  color: string;
  onChange: (hex: string) => void;
  style?: React.CSSProperties;
}

export default function ColorPicker({ color, onChange, style }: Props) {
  const wheelRef        = useRef<HTMLCanvasElement>(null);
  const sliderRef       = useRef<HTMLCanvasElement>(null);
  const containerRef    = useRef<HTMLDivElement>(null);
  const wheelDragging   = useRef(false);
  const sliderDragging  = useRef(false);

  const [hsv, setHsv]         = useState<[number, number, number]>(() => hexToHsv(color));
  const [hexStr, setHexStr]   = useState(() => color.replace("#", "").toUpperCase());
  const [mode, setMode]       = useState<"HSV" | "RGB">("HSV");
  const [nudge, setNudge]     = useState({ x: 0, y: 0 });

  useLayoutEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    let dx = 0, dy = 0;
    if (r.right  > window.innerWidth  - 4) dx = window.innerWidth  - 4 - r.right;
    if (r.left   < 4)                      dx = 4 - r.left;
    if (r.bottom > window.innerHeight - 4) dy = window.innerHeight - 4 - r.bottom;
    if (r.top    < 4)                      dy = 4 - r.top;
    if (dx !== 0 || dy !== 0) setNudge({ x: dx, y: dy });
  }, []);

  const [h, s, v] = hsv;

  // ── Wheel redraw (depends on V) ────────────────────────────
  useEffect(() => {
    const canvas = wheelRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d")!;
    const img = ctx.createImageData(W, W);
    for (let py = 0; py < W; py++) {
      for (let px = 0; px < W; px++) {
        const dx = px - CX, dy = CY - py;
        const dist = Math.sqrt(dx * dx + dy * dy);
        const i = (py * W + px) * 4;
        if (dist > R + 1) { img.data[i + 3] = 0; continue; }
        const hue = ((Math.atan2(dy, dx) * 180 / Math.PI) + 360) % 360;
        const sat = Math.min(dist / R, 1);
        const aa  = dist > R ? Math.max(0, R + 1 - dist) : 1;
        const [r, g, b] = hsvToRgb(hue, sat, v);
        img.data[i]     = r;
        img.data[i + 1] = g;
        img.data[i + 2] = b;
        img.data[i + 3] = Math.round(aa * 255);
      }
    }
    ctx.putImageData(img, 0, 0);
  }, [v]);

  // ── Slider redraw (depends on H, S) ───────────────────────
  useEffect(() => {
    const canvas = sliderRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d")!;
    const img = ctx.createImageData(SW, SH);
    for (let py = 0; py < SH; py++) {
      const val = 1 - py / (SH - 1);
      const [r, g, b] = hsvToRgb(h, s, val);
      for (let px = 0; px < SW; px++) {
        const i = (py * SW + px) * 4;
        img.data[i] = r; img.data[i + 1] = g; img.data[i + 2] = b; img.data[i + 3] = 255;
      }
    }
    ctx.putImageData(img, 0, 0);
  }, [h, s]);

  // ── Emit color ─────────────────────────────────────────────
  const emit = useCallback((nh: number, ns: number, nv: number) => {
    const [r, g, b] = hsvToRgb(nh, ns, nv);
    const hex = rgbToHex(r, g, b);
    setHexStr(hex.replace("#", "").toUpperCase());
    onChange(hex);
  }, [onChange]);

  // ── Wheel interaction ──────────────────────────────────────
  function pickWheel(e: React.PointerEvent<HTMLCanvasElement>) {
    const rect = wheelRef.current!.getBoundingClientRect();
    const dx = e.clientX - rect.left - CX;
    const dy = CY - (e.clientY - rect.top);
    const dist = Math.sqrt(dx * dx + dy * dy);
    const newH = ((Math.atan2(dy, dx) * 180 / Math.PI) + 360) % 360;
    const newS = Math.min(dist / R, 1);
    setHsv([newH, newS, v]);
    emit(newH, newS, v);
  }

  // ── Slider interaction ─────────────────────────────────────
  function pickSlider(e: React.PointerEvent<HTMLCanvasElement>) {
    const rect = sliderRef.current!.getBoundingClientRect();
    const newV = Math.max(0, Math.min(1, 1 - (e.clientY - rect.top) / SH));
    setHsv([h, s, newV]);
    emit(h, s, newV);
  }

  // ── Hex input ──────────────────────────────────────────────
  function handleHex(raw: string) {
    const val = raw.replace(/[^0-9a-fA-F]/g, "").slice(0, 6);
    setHexStr(val.toUpperCase());
    if (val.length === 6) {
      const [r, g, b] = hexToRgb("#" + val);
      const newHsv = rgbToHsv(r, g, b);
      setHsv(newHsv);
      onChange("#" + val.toLowerCase());
    }
  }

  // ── HSV / RGB field change ─────────────────────────────────
  function handleField(field: "h"|"s"|"v"|"r"|"g"|"b", raw: string) {
    const n = parseFloat(raw) || 0;
    if (mode === "HSV") {
      const [nh, ns, nv] = [
        field === "h" ? Math.max(0, Math.min(360, n)) : h,
        field === "s" ? Math.max(0, Math.min(1, n))   : s,
        field === "v" ? Math.max(0, Math.min(1, n))   : v,
      ];
      setHsv([nh, ns, nv]); emit(nh, ns, nv);
    } else {
      const [cr, cg, cb] = hsvToRgb(h, s, v);
      const nr = field === "r" ? Math.max(0, Math.min(255, Math.round(n))) : cr;
      const ng = field === "g" ? Math.max(0, Math.min(255, Math.round(n))) : cg;
      const nb = field === "b" ? Math.max(0, Math.min(255, Math.round(n))) : cb;
      const newHsv = rgbToHsv(nr, ng, nb);
      setHsv(newHsv); emit(...newHsv);
    }
  }

  // ── Indicator positions ────────────────────────────────────
  const hRad = h * Math.PI / 180;
  const indX = CX + R * s * Math.cos(hRad);
  const indY = CY - R * s * Math.sin(hRad);
  const sliderY = (1 - v) * (SH - 1);

  const [cr, cg, cb] = hsvToRgb(h, s, v);
  const fields = mode === "HSV"
    ? [
        { key: "h" as const, label: "Hue",        val: h.toFixed(1),   step: 1,     min: 0, max: 360 },
        { key: "s" as const, label: "Saturation",  val: s.toFixed(3),   step: 0.001, min: 0, max: 1   },
        { key: "v" as const, label: "Value",       val: v.toFixed(3),   step: 0.001, min: 0, max: 1   },
      ]
    : [
        { key: "r" as const, label: "R", val: String(cr), step: 1, min: 0, max: 255 },
        { key: "g" as const, label: "G", val: String(cg), step: 1, min: 0, max: 255 },
        { key: "b" as const, label: "B", val: String(cb), step: 1, min: 0, max: 255 },
      ];

  const inputStyle: React.CSSProperties = {
    flex: 1, background: "#111", border: "1px solid #2a2a2a", borderRadius: 3,
    color: "#aaa", fontSize: 11, padding: "2px 5px", outline: "none",
    fontFamily: "inherit", minWidth: 0,
  };

  return (
    <div
      ref={containerRef}
      style={{
        background: "#1c1c1c", border: "1px solid #3a3a3a", borderRadius: 6,
        padding: 12, width: W + SW + 8 + 24, userSelect: "none",
        boxShadow: "0 8px 32px rgba(0,0,0,0.8)",
        transform: `translate(${nudge.x}px, ${nudge.y}px)`,
        ...style,
      }}
      onPointerDown={(e) => e.stopPropagation()}
    >
      {/* ── Wheel + slider ── */}
      <div style={{ display: "flex", gap: 8 }}>
        {/* Wheel */}
        <div style={{ position: "relative", flexShrink: 0 }}>
          <canvas
            ref={wheelRef}
            width={W} height={W}
            style={{ display: "block", borderRadius: "50%", cursor: "crosshair" }}
            onPointerDown={(e) => {
              wheelDragging.current = true;
              e.currentTarget.setPointerCapture(e.pointerId);
              pickWheel(e);
            }}
            onPointerMove={(e) => { if (wheelDragging.current) pickWheel(e); }}
            onPointerUp={() => { wheelDragging.current = false; }}
          />
          {/* Cursor dot */}
          <div style={{
            position: "absolute", pointerEvents: "none",
            left: indX - 6, top: indY - 6,
            width: 12, height: 12, borderRadius: "50%",
            border: "2px solid #fff",
            boxShadow: "0 0 0 1px rgba(0,0,0,0.6), inset 0 0 0 1px rgba(0,0,0,0.3)",
          }} />
        </div>

        {/* Value slider */}
        <div style={{ position: "relative", flexShrink: 0 }}>
          <canvas
            ref={sliderRef}
            width={SW} height={SH}
            style={{ display: "block", borderRadius: 3, cursor: "ns-resize" }}
            onPointerDown={(e) => {
              sliderDragging.current = true;
              e.currentTarget.setPointerCapture(e.pointerId);
              pickSlider(e);
            }}
            onPointerMove={(e) => { if (sliderDragging.current) pickSlider(e); }}
            onPointerUp={() => { sliderDragging.current = false; }}
          />
          {/* Notch */}
          <div style={{
            position: "absolute", pointerEvents: "none",
            top: sliderY - 2, left: -3, right: -3, height: 4,
            border: "1.5px solid #fff", borderRadius: 2,
            boxShadow: "0 0 0 1px rgba(0,0,0,0.5)",
          }} />
        </div>
      </div>

      {/* ── Mode tabs ── */}
      <div style={{ display: "flex", marginTop: 10, gap: 2 }}>
        {(["HSV", "RGB"] as const).map((m) => (
          <button
            key={m}
            onClick={() => setMode(m)}
            style={{
              flex: 1, padding: "3px 0", fontSize: 10, borderRadius: 3,
              border: "none", cursor: "pointer",
              background: mode === m ? "#4a90d9" : "#2a2a2a",
              color: mode === m ? "#fff" : "#666",
            }}
          >{m}</button>
        ))}
      </div>

      {/* ── Numeric fields ── */}
      <div style={{ marginTop: 6, display: "flex", flexDirection: "column", gap: 3 }}>
        {fields.map(({ key, label, val, step, min, max }) => (
          <div key={key} style={{ display: "flex", alignItems: "center", gap: 6, height: 22 }}>
            <span style={{
              width: mode === "HSV" ? 68 : 12, fontSize: 10, color: "#555",
              textAlign: mode === "HSV" ? "right" : "center", flexShrink: 0,
            }}>{label}</span>
            <input
              type="number" value={val} step={step} min={min} max={max}
              onChange={(e) => handleField(key, e.target.value)}
              onKeyDown={(e) => e.stopPropagation()}
              style={inputStyle}
            />
          </div>
        ))}

        {/* Hex + swatch */}
        <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 2, height: 22 }}>
          <span style={{ fontSize: 10, color: "#555", width: 68, textAlign: "right", flexShrink: 0 }}>Hex</span>
          <span style={{ fontSize: 11, color: "#555" }}>#</span>
          <input
            value={hexStr}
            onChange={(e) => handleHex(e.target.value)}
            onKeyDown={(e) => e.stopPropagation()}
            maxLength={6}
            style={{ ...inputStyle, fontFamily: "monospace" }}
          />
          <div style={{
            width: 22, height: 18, borderRadius: 3, flexShrink: 0,
            background: rgbToHex(cr, cg, cb),
            border: "1px solid #3a3a3a",
          }} />
        </div>
      </div>
    </div>
  );
}
