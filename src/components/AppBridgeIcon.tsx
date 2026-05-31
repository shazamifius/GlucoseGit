/**
 * Phase 5 — Icônes reconnaissables par logiciel sur les nœuds Fichier Source
 * (sticky avec `sourceFile` — App Bridge).
 *
 * Détection par extension. Icônes vectorielles SVG inline (pas de bundling externe).
 * Couleurs officielles approximatives. Fallback générique pour les inconnus.
 */

import { useMemo } from "react";

export interface AppBridgeIconProps {
  /** Chemin absolu du fichier source. */
  filePath: string;
  size?: number;
}

interface AppDef {
  /** Couleur de fond (badge). */
  bg: string;
  /** Couleur du glyphe. */
  fg: string;
  /** Glyphe SVG (path d) ou label texte court. */
  label: string;
  /** Nom complet pour tooltip. */
  name: string;
}

const APP_DEFS: Record<string, AppDef> = {
  blend:  { bg: "#EA7600", fg: "#fff", label: "B",   name: "Blender" },
  psd:    { bg: "#001E36", fg: "#31A8FF", label: "Ps", name: "Photoshop" },
  ai:     { bg: "#330000", fg: "#FF9A00", label: "Ai", name: "Illustrator" },
  kra:    { bg: "#3DAEE9", fg: "#fff", label: "K",   name: "Krita" },
  xcf:    { bg: "#5C5543", fg: "#fff", label: "G",   name: "GIMP" },
  fig:    { bg: "#1E1E1E", fg: "#0ACF83", label: "F",   name: "Figma" },
  sketch: { bg: "#FDB300", fg: "#222", label: "S",   name: "Sketch" },
  procreate: { bg: "#2D2D2D", fg: "#FFCE3D", label: "Pr", name: "Procreate" },
  zpr:    { bg: "#4D4D4D", fg: "#fff", label: "Z",   name: "ZBrush" },
  c4d:    { bg: "#0089DB", fg: "#fff", label: "C4D", name: "Cinema 4D" },
  max:    { bg: "#1675C8", fg: "#fff", label: "Mx",  name: "3ds Max" },
  ma:     { bg: "#0078D4", fg: "#fff", label: "Ma",  name: "Maya" },
  mb:     { bg: "#0078D4", fg: "#fff", label: "Ma",  name: "Maya" },
  fbx:    { bg: "#0078D4", fg: "#fff", label: "fbx", name: "Autodesk FBX" },
  obj:    { bg: "#666666", fg: "#fff", label: "obj", name: "Wavefront OBJ" },
  // VFX / compositing / montage
  nuke:   { bg: "#FFCC00", fg: "#222", label: "Nk", name: "Nuke" },
  nk:     { bg: "#FFCC00", fg: "#222", label: "Nk", name: "Nuke" },
  hip:    { bg: "#FF6600", fg: "#fff", label: "H",  name: "Houdini" },
  hipnc:  { bg: "#FF6600", fg: "#fff", label: "H",  name: "Houdini" },
  aep:    { bg: "#00005B", fg: "#9999FF", label: "Ae", name: "After Effects" },
  prproj: { bg: "#00005B", fg: "#EA77FF", label: "Pr", name: "Premiere Pro" },
  drp:    { bg: "#000", fg: "#FF6A00", label: "DR", name: "DaVinci Resolve" },
  // Design / édition
  indd:   { bg: "#49021F", fg: "#FF3366", label: "Id", name: "InDesign" },
  clip:   { bg: "#111", fg: "#fff", label: "Cs", name: "Clip Studio" },
  afphoto:{ bg: "#1B1B3A", fg: "#76B6F4", label: "Ph", name: "Affinity Photo" },
  afdesign:{ bg: "#1B1B3A", fg: "#A35CF4", label: "De", name: "Affinity Designer" },
  exr:    { bg: "#222", fg: "#7CFFB2", label: "exr", name: "OpenEXR" },
  usd:    { bg: "#222", fg: "#FFD21F", label: "usd", name: "USD" },
  abc:    { bg: "#444", fg: "#fff", label: "abc", name: "Alembic" },
  // Archives
  zip:    { bg: "#7C4DFF", fg: "#fff", label: "zip", name: "Archive ZIP" },
  rar:    { bg: "#7C4DFF", fg: "#fff", label: "rar", name: "Archive RAR" },
  "7z":   { bg: "#7C4DFF", fg: "#fff", label: "7z", name: "Archive 7-Zip" },
  // Documents bureautiques
  docx:   { bg: "#2B579A", fg: "#fff", label: "W",  name: "Word" },
  xlsx:   { bg: "#217346", fg: "#fff", label: "X",  name: "Excel" },
  pptx:   { bg: "#D24726", fg: "#fff", label: "P",  name: "PowerPoint" },
  glb:    { bg: "#FFD21F", fg: "#222", label: "glb", name: "glTF binaire" },
  gltf:   { bg: "#FFD21F", fg: "#222", label: "gltf",name: "glTF" },
  unitypackage: { bg: "#222", fg: "#fff", label: "U", name: "Unity" },
  uproject: { bg: "#222", fg: "#fff", label: "UE", name: "Unreal" },
  // Audio
  wav:    { bg: "#1DB954", fg: "#fff", label: "♪", name: "WAV" },
  mp3:    { bg: "#1DB954", fg: "#fff", label: "♪", name: "MP3" },
  flac:   { bg: "#1DB954", fg: "#fff", label: "♪", name: "FLAC" },
  // Video
  mp4:    { bg: "#FF4D4D", fg: "#fff", label: "▶", name: "MP4" },
  mov:    { bg: "#FF4D4D", fg: "#fff", label: "▶", name: "MOV" },
  webm:   { bg: "#FF4D4D", fg: "#fff", label: "▶", name: "WebM" },
  // Code
  ts:     { bg: "#3178C6", fg: "#fff", label: "ts", name: "TypeScript" },
  tsx:    { bg: "#3178C6", fg: "#fff", label: "tsx",name: "TSX" },
  js:     { bg: "#F7DF1E", fg: "#222", label: "js", name: "JavaScript" },
  py:     { bg: "#3776AB", fg: "#FFD43B", label: "py", name: "Python" },
  rs:     { bg: "#CE422B", fg: "#fff", label: "rs", name: "Rust" },
  go:     { bg: "#00ADD8", fg: "#fff", label: "go", name: "Go" },
  // Documents
  pdf:    { bg: "#D40C0C", fg: "#fff", label: "pdf",name: "PDF" },
  md:     { bg: "#1f1f1f", fg: "#fff", label: "md", name: "Markdown" },
  txt:    { bg: "#444", fg: "#fff", label: "txt",name: "Texte" },
  json:   { bg: "#222", fg: "#FFC83D", label: "{}", name: "JSON" },
};

const FALLBACK: AppDef = { bg: "#3a3a3a", fg: "#aaa", label: "?", name: "Fichier" };

export function getAppDef(filePath: string): AppDef {
  const m = filePath.match(/\.([a-z0-9]+)$/i);
  if (!m) return FALLBACK;
  return APP_DEFS[m[1].toLowerCase()] ?? FALLBACK;
}

export default function AppBridgeIcon({ filePath, size = 18 }: AppBridgeIconProps) {
  const def = useMemo(() => getAppDef(filePath), [filePath]);
  // Police adaptée à la longueur du label (1-3 chars)
  const fontSize = def.label.length <= 1 ? size * 0.55 : def.label.length === 2 ? size * 0.45 : size * 0.36;
  return (
    <svg
      width={size} height={size} viewBox="0 0 24 24"
      style={{ flexShrink: 0, display: "inline-block" }}
      role="img"
      aria-label={def.name}
    >
      <title>{def.name}</title>
      <rect x={1.5} y={1.5} width={21} height={21} rx={4} fill={def.bg} stroke="rgba(0,0,0,0.25)" strokeWidth={0.5} />
      <text
        x={12} y={12.5}
        textAnchor="middle" dominantBaseline="central"
        fill={def.fg}
        fontFamily="system-ui, -apple-system, sans-serif"
        fontWeight={700}
        fontSize={fontSize / size * 24}
      >{def.label}</text>
    </svg>
  );
}
