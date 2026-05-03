import { Container, Graphics } from "pixi.js";
import { BoardImage, Domain } from "../types";

// Clustering distance threshold in world pixels
const CLUSTER_DIST = 600;

// Convertit une couleur (hex `#rrggbb` ou hsl(h, s%, l%)) en teinte HSL [0..360)
function colorToHue(color: string): number {
  if (color.startsWith("hsl")) {
    const m = color.match(/hsl\(\s*(\d+(?:\.\d+)?)/);
    if (m) return parseFloat(m[1]) % 360;
  }
  // hex #rrggbb
  let hex = color.startsWith("#") ? color.slice(1) : color;
  if (hex.length === 3) hex = hex.split("").map(c => c + c).join("");
  if (hex.length !== 6) return 0;
  const r = parseInt(hex.slice(0, 2), 16) / 255;
  const g = parseInt(hex.slice(2, 4), 16) / 255;
  const b = parseInt(hex.slice(4, 6), 16) / 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  if (max === min) return 0;
  const d = max - min;
  let h = 0;
  if (max === r) h = ((g - b) / d) % 6;
  else if (max === g) h = (b - r) / d + 2;
  else h = (r - g) / d + 4;
  h *= 60;
  if (h < 0) h += 360;
  return h;
}

// Convex hull — Gift Wrapping (Jarvis march)
function convexHull(pts: { x: number; y: number }[]): { x: number; y: number }[] {
  if (pts.length < 3) return pts;
  const cross = (o: typeof pts[0], a: typeof pts[0], b: typeof pts[0]) =>
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);

  const sorted = [...pts].sort((a, b) => a.x !== b.x ? a.x - b.x : a.y - b.y);
  const lower: typeof pts = [];
  for (const p of sorted) {
    while (lower.length >= 2 && cross(lower[lower.length - 2], lower[lower.length - 1], p) <= 0)
      lower.pop();
    lower.push(p);
  }
  const upper: typeof pts = [];
  for (let i = sorted.length - 1; i >= 0; i--) {
    const p = sorted[i];
    while (upper.length >= 2 && cross(upper[upper.length - 2], upper[upper.length - 1], p) <= 0)
      upper.pop();
    upper.push(p);
  }
  upper.pop(); lower.pop();
  return lower.concat(upper);
}

// Union-Find for clustering
class UnionFind {
  private parent: number[];
  constructor(n: number) { this.parent = Array.from({ length: n }, (_, i) => i); }
  find(i: number): number {
    if (this.parent[i] !== i) this.parent[i] = this.find(this.parent[i]);
    return this.parent[i];
  }
  union(i: number, j: number) { this.parent[this.find(i)] = this.find(j); }
}

function clusterImages(images: BoardImage[]): Map<number, BoardImage[]> {
  const n = images.length;
  const uf = new UnionFind(n);
  for (let i = 0; i < n; i++) {
    for (let j = i + 1; j < n; j++) {
      const dx = images[i].x - images[j].x;
      const dy = images[i].y - images[j].y;
      if (Math.hypot(dx, dy) < CLUSTER_DIST) uf.union(i, j);
    }
  }
  const clusters = new Map<number, BoardImage[]>();
  for (let i = 0; i < n; i++) {
    const root = uf.find(i);
    if (!clusters.has(root)) clusters.set(root, []);
    clusters.get(root)!.push(images[i]);
  }
  return clusters;
}

// Derive a hue (0-360) from an image id for stable per-cluster color
function idToHue(id: string): number {
  let h = 0;
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) & 0xfffff;
  return h % 360;
}

function hslToHex(h: number, s: number, l: number): number {
  s /= 100; l /= 100;
  const k = (n: number) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
  return (Math.round(f(0) * 255) << 16) | (Math.round(f(8) * 255) << 8) | Math.round(f(4) * 255);
}

export class MembraneRenderer {
  private gfx: Graphics;

  constructor(parent: Container) {
    this.gfx = new Graphics();
    // Insert at index 0 so membranes render behind sprites
    parent.addChildAt(this.gfx, 0);
  }

  update(images: BoardImage[], domains: Domain[] = []) {
    this.gfx.clear();
    if (images.length < 2) return;

    const clusters = clusterImages(images);
    const PADDING = 80;
    const domainById = new Map(domains.map(d => [d.id, d]));

    clusters.forEach((members) => {
      if (members.length < 2) return;

      // Build corner points (anchor 0.5 → top-left/bottom-right of each image)
      const pts: { x: number; y: number }[] = [];
      for (const img of members) {
        const hw = img.width  / 2 + PADDING;
        const hh = img.height / 2 + PADDING;
        pts.push({ x: img.x - hw, y: img.y - hh });
        pts.push({ x: img.x + hw, y: img.y - hh });
        pts.push({ x: img.x + hw, y: img.y + hh });
        pts.push({ x: img.x - hw, y: img.y + hh });
      }

      const hull = convexHull(pts);
      if (hull.length < 3) return;

      // ── Couleur dérivée des domaines pondérés (Phase 3) ──
      // Somme vectorielle des teintes domaine × poids ; fallback sur idToHue si rien.
      let sumX = 0, sumY = 0, totalWeight = 0;
      for (const img of members) {
        for (const da of img.domains ?? []) {
          const dom = domainById.get(da.domainId);
          if (!dom) continue;
          const hue = colorToHue(dom.color);
          const rad = hue * Math.PI / 180;
          sumX += Math.cos(rad) * da.weight;
          sumY += Math.sin(rad) * da.weight;
          totalWeight += da.weight;
        }
      }

      let hue: number;
      if (totalWeight > 0.001) {
        hue = Math.atan2(sumY, sumX) * 180 / Math.PI;
        if (hue < 0) hue += 360;
      } else {
        hue = idToHue(members[0].id);
      }
      // Saturation un peu plus forte si la membrane est dominée par un domaine clair
      const sat = totalWeight > 0.001 ? 70 : 65;
      const color = hslToHex(hue, sat, 55);

      this.gfx.moveTo(hull[0].x, hull[0].y);
      for (let i = 1; i < hull.length; i++) this.gfx.lineTo(hull[i].x, hull[i].y);
      this.gfx.closePath();
      this.gfx.fill({ color, alpha: 0.04 });
      this.gfx.moveTo(hull[0].x, hull[0].y);
      for (let i = 1; i < hull.length; i++) this.gfx.lineTo(hull[i].x, hull[i].y);
      this.gfx.closePath();
      this.gfx.stroke({ color, width: 1.5, alpha: 0.22 });
    });
  }

  destroy() { this.gfx.destroy(); }
}
