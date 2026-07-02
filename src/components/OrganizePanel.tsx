import { useState, useCallback } from "react";
import { useGlucoseStore, getActiveBoard } from "../store";
import { BoardImage } from "../types";
import { gridLayout, compactRowsLayout, sameHeightLayout, masonryLayout, bySlotLayout, boundsOfImages } from "../utils/layout";
import { resolveImageSrc } from "../utils/assets";

interface Props {
  docked?: boolean;
}

type LayoutType = "compact" | "grid" | "masonry" | "sameHeight" | "bySlot";
type SortType   = "none" | "size-desc" | "size-asc" | "ratio-port" | "ratio-land" | "lum-asc" | "lum-desc" | "color";

interface ImageColorStats {
  lum: number;
  hue: number;
  sat: number;
}

const statsCache = new Map<string, ImageColorStats>();

export function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  let h = 0, s = 0, l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r: h = (g - b) / d + (g < b ? 6 : 0); break;
      case g: h = (b - r) / d + 2; break;
      case b: h = (r - g) / d + 4; break;
    }
    h /= 6;
  }
  return [h * 360, s, l];
}

async function computeImageStats(img: BoardImage): Promise<ImageColorStats> {
  const key = img.asset?.sha256 || img.src || img.id;
  const cached = statsCache.get(key);
  if (cached) return cached;

  const blobs = useGlucoseStore.getState().project.blobs;
  const url = await resolveImageSrc(img.asset, img.src, blobs);
  if (!url) {
    const fallback = { lum: 0.5, hue: 0, sat: 0 };
    statsCache.set(key, fallback);
    return fallback;
  }

  return new Promise((resolve) => {
    const el = new Image();
    el.crossOrigin = "anonymous";

    const timer = setTimeout(() => {
      el.onload = null;
      el.onerror = null;
      const fallback = { lum: 0.5, hue: 0, sat: 0 };
      statsCache.set(key, fallback);
      resolve(fallback);
    }, 1200);

    el.onload = () => {
      clearTimeout(timer);
      try {
        const c = document.createElement("canvas");
        c.width = 32; c.height = 32;
        const ctx = c.getContext("2d")!;
        ctx.drawImage(el, 0, 0, 32, 32);
        const data = ctx.getImageData(0, 0, 32, 32).data;
        
        let sumLum = 0;
        let sumSat = 0;
        let sumHueX = 0;
        let sumHueY = 0;
        let satWeightSum = 0;

        for (let i = 0; i < data.length; i += 4) {
          const r = data[i];
          const g = data[i + 1];
          const b = data[i + 2];
          
          const l = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
          sumLum += l;

          const [h, s] = rgbToHsl(r, g, b);
          sumSat += s;

          const rad = (h * Math.PI) / 180;
          const weight = s + 0.01;
          sumHueX += Math.cos(rad) * weight;
          sumHueY += Math.sin(rad) * weight;
          satWeightSum += weight;
        }

        const avgLum = sumLum / (32 * 32);
        const avgSat = sumSat / (32 * 32);
        
        let avgHue = 0;
        if (satWeightSum > 0) {
          avgHue = (Math.atan2(sumHueY, sumHueX) * 180) / Math.PI;
          if (avgHue < 0) avgHue += 360;
        }

        const stats = { lum: avgLum, hue: avgHue, sat: avgSat };
        statsCache.set(key, stats);
        resolve(stats);
      } catch (err) {
        const fallback = { lum: 0.5, hue: 0, sat: 0 };
        statsCache.set(key, fallback);
        resolve(fallback);
      }
    };

    el.onerror = () => {
      clearTimeout(timer);
      const fallback = { lum: 0.5, hue: 0, sat: 0 };
      statsCache.set(key, fallback);
      resolve(fallback);
    };

    el.src = url;
  });
}

const LAYOUT_OPTIONS: { value: LayoutType; label: string; desc: string }[] = [
  { value: "compact", label: "Rangées compactes", desc: "Respecte les ratios, remplit chaque ligne" },
  { value: "masonry", label: "Masonry (colonnes)", desc: "Colonnes indépendantes — Pinterest" },
  { value: "grid", label: "Grille alignée", desc: "Même largeur par colonne, ratios conservés" },
  { value: "sameHeight", label: "Même hauteur", desc: "Hauteur fixe, largeur proportionnelle" },
  { value: "bySlot", label: "Par slot preset", desc: "Colonnes séparées par catégorie" },
];

export default function OrganizePanel({ docked }: Props) {
  const { project, updateMultipleImages, getAllPresets, selectedImageIds } = useGlucoseStore();
  const board = getActiveBoard(project);

  const [layout, setLayout] = useState<LayoutType>("compact");
  const [sortBy, setSortBy] = useState<SortType>("none");
  const [size, setSize] = useState(280);
  const [gap, setGap] = useState(16);
  const [cols, setCols] = useState(0);

  // Work on selected images if any, otherwise all images
  const targetImages = selectedImageIds.length > 0
    ? board.images.filter((img) => selectedImageIds.includes(img.id))
    : board.images;

  const [applying, setApplying] = useState(false);

  const apply = useCallback(async () => {
    if (!targetImages.length || applying) return;
    setApplying(true);

    let sorted = [...targetImages];

    if (sortBy === "lum-asc" || sortBy === "lum-desc" || sortBy === "color") {
      const stats = await Promise.all(sorted.map(computeImageStats));
      const pairs = sorted.map((img, i) => ({ img, stat: stats[i] }));
      if (sortBy === "color") {
        pairs.sort((a, b) => {
          const isMonoA = a.stat.sat < 0.15 || a.stat.lum < 0.1 || a.stat.lum > 0.92;
          const isMonoB = b.stat.sat < 0.15 || b.stat.lum < 0.1 || b.stat.lum > 0.92;

          if (isMonoA && !isMonoB) return 1;
          if (!isMonoA && isMonoB) return -1;
          if (isMonoA && isMonoB) return a.stat.lum - b.stat.lum;
          return a.stat.hue - b.stat.hue;
        });
      } else {
        pairs.sort((a, b) => sortBy === "lum-asc" ? a.stat.lum - b.stat.lum : b.stat.lum - a.stat.lum);
      }
      sorted = pairs.map((p) => p.img);
    } else {
      sorted.sort((a, b) => {
        if (sortBy === "size-desc")  return (b.width * b.height) - (a.width * a.height);
        if (sortBy === "size-asc")   return (a.width * a.height) - (b.width * b.height);
        if (sortBy === "ratio-port") return (a.width / a.height) - (b.width / b.height);
        if (sortBy === "ratio-land") return (b.width / b.height) - (a.width / a.height);
        return 0;
      });
    }

    const bounds = boundsOfImages(sorted);
    const startX = bounds.minX;
    const startY = bounds.minY;

    let results: { id: string; x: number; y: number; width: number; height: number }[] = [];

    if (layout === "grid") {
      results = gridLayout(sorted, { cols, targetSize: size, gap, startX, startY });
    } else if (layout === "masonry") {
      results = masonryLayout(sorted, { cols: cols > 0 ? cols : 3, targetSize: size, gap, startX, startY });
    } else if (layout === "compact") {
      results = compactRowsLayout(sorted, {
        targetHeight: size, maxRowWidth: size * 5, gap, startX, startY,
      });
    } else if (layout === "sameHeight") {
      results = sameHeightLayout(sorted, {
        targetHeight: size, maxRowWidth: size * 6, gap, startX, startY,
        cols: cols > 0 ? cols : undefined,
      });
    } else if (layout === "bySlot") {
      const allPresets = getAllPresets();
      const preset = board.presetId ? allPresets.find((p) => p.id === board.presetId) : null;
      const slotOrder = preset ? preset.slots.map((s) => s.id) : [];
      results = bySlotLayout(sorted, slotOrder, { colWidth: size, gap, startX, startY });
    }

    updateMultipleImages(
      board.id,
      results.map((r) => ({ id: r.id, patch: { x: r.x, y: r.y, width: r.width, height: r.height } })),
    );
    setApplying(false);
  }, [targetImages, sortBy, layout, size, gap, cols, board, applying]);

  const inputStyle: React.CSSProperties = {
    background: "#1a1a1a", border: "1px solid #2a2a2a", borderRadius: 3,
    color: "#ccc", fontSize: 12, padding: "3px 7px", width: "100%", outline: "none",
  };

  const labelStyle: React.CSSProperties = {
    fontSize: 10, color: "#555", textTransform: "uppercase" as const, letterSpacing: 1, marginBottom: 4,
  };

  return (
    <div style={{
      ...(docked ? {} : { position: "absolute", bottom: 60, left: 60, zIndex: 200, boxShadow: "0 8px 32px rgba(0,0,0,0.6)" }),
      width: 250, background: "#111", border: "1px solid #222", borderRadius: 6,
    }}>
      <div style={{
        display: "flex", alignItems: "center", justifyContent: "space-between",
        padding: "10px 14px", borderBottom: "1px solid #1e1e1e",
      }}>
        <span style={{ color: "#ccc", fontSize: 12, fontWeight: 600, letterSpacing: 1, textTransform: "uppercase" }}>
          Ordonner
        </span>
      </div>

      <div style={{ padding: "12px 14px", display: "flex", flexDirection: "column", gap: 12 }}>

        {/* Target info */}
        <div style={{
          fontSize: 11, padding: "5px 8px", borderRadius: 3,
          background: selectedImageIds.length > 0 ? "#1a1800" : "#1a1a1a",
          border: `1px solid ${selectedImageIds.length > 0 ? "#3a3000" : "#222"}`,
          color: selectedImageIds.length > 0 ? "#aa9900" : "#555",
        }}>
          {selectedImageIds.length > 0
            ? `${selectedImageIds.length} image${selectedImageIds.length > 1 ? "s" : ""} sélectionnée${selectedImageIds.length > 1 ? "s" : ""}`
            : `Toutes les images (${board.images.length})`}
        </div>

        {/* Sort */}
        <div>
          <div style={labelStyle}>Trier avant disposition</div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
            {([
              ["none",       "Ordre actuel"],
              ["color",      "Couleur"],
              ["size-desc",  "Grand → Petit"],
              ["size-asc",   "Petit → Grand"],
              ["ratio-port", "Portrait"],
              ["ratio-land", "Paysage"],
              ["lum-asc",   "Sombre → Clair"],
              ["lum-desc",  "Clair → Sombre"],
            ] as [SortType, string][]).map(([val, label]) => (
              <button
                key={val}
                onClick={() => setSortBy(val)}
                style={{
                  padding: "3px 8px", fontSize: 10, borderRadius: 3, cursor: "pointer",
                  background: sortBy === val ? "#2d2d2d" : "#1a1a1a",
                  color:      sortBy === val ? "#ccc"    : "#555",
                  border: `1px solid ${sortBy === val ? "#444" : "#2a2a2a"}`,
                }}
              >
                {label}
              </button>
            ))}
          </div>
        </div>

        {/* Layout type */}
        <div>
          <div style={labelStyle}>Disposition</div>
          <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
            {LAYOUT_OPTIONS.map((opt) => {
              const active = layout === opt.value;
              return (
                <div
                  key={opt.value}
                  onClick={() => setLayout(opt.value)}
                  style={{
                    padding: "5px 9px", borderRadius: 4, cursor: "pointer",
                    background: active ? "#1e1e1e" : "transparent",
                    border: `1px solid ${active ? "#333" : "transparent"}`,
                  }}
                >
                  <div style={{ color: active ? "#fff" : "#888", fontSize: 12 }}>{opt.label}</div>
                  <div style={{ color: "#444", fontSize: 10 }}>{opt.desc}</div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Params */}
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
          <div>
            <div style={labelStyle}>{layout === "sameHeight" ? "Hauteur" : "Largeur cible"}</div>
            <input type="number" min={50} max={2000} step={10} value={size}
              onChange={(e) => setSize(Number(e.target.value))} style={inputStyle} />
          </div>
          <div>
            <div style={labelStyle}>Espacement</div>
            <input type="number" min={0} max={200} step={4} value={gap}
              onChange={(e) => setGap(Number(e.target.value))} style={inputStyle} />
          </div>
          {layout !== "bySlot" && layout !== "compact" && (
            <div>
              <div style={labelStyle}>Colonnes (0=auto)</div>
              <input type="number" min={0} max={20} step={1} value={cols}
                onChange={(e) => setCols(Number(e.target.value))} style={inputStyle} />
            </div>
          )}
        </div>

        <button
          onClick={apply}
          disabled={targetImages.length === 0 || applying}
          style={{
            width: "100%", padding: "7px", fontSize: 12, fontWeight: 600,
            background: targetImages.length === 0 || applying ? "#1a1a1a" : "#222",
            color: targetImages.length === 0 || applying ? "#444" : "#ccc",
            border: "1px solid #333", borderRadius: 4,
            cursor: targetImages.length === 0 || applying ? "not-allowed" : "pointer",
          }}
          onMouseOver={(e) => { if (targetImages.length > 0 && !applying) e.currentTarget.style.background = "#2a2a2a"; }}
          onMouseOut={(e) => { if (targetImages.length > 0 && !applying) e.currentTarget.style.background = "#222"; }}
        >
          {applying ? "Calcul en cours…" : "Appliquer"}
        </button>
      </div>
    </div>
  );
}
