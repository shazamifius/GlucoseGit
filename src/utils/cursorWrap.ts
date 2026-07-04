import { getCurrentWindow, LogicalPosition } from "@tauri-apps/api/window";

// ── Linux / Wayland — pan robuste ────────────────────────────────────────────
// DEUX bugs sous Linux (WebKitGTK) cassaient le déplacement :
//  1) `movementX/Y` (deltas relatifs) sont NON FIABLES sous WebKitGTK → souvent 0
//     ou faux ⇒ le pan (qui reposait dessus) ne bougeait pas / saccadait.
//  2) le « cursor warp » (téléporter le curseur au bord, setCursorPosition) est
//     bloqué sous Wayland ⇒ le curseur atteint le vrai bord, Niri scrolle l'espace
//     (« boucle d'écran ») + IPC qui rate = saccades.
// FIX sous Linux : on NE téléporte PAS et on NE grab PAS le curseur (il reste
// libre et visible → ses positions absolues `clientX/Y` restent vivantes), et on
// calcule le déplacement à partir des DELTAS de clientX/Y (fiables partout) au
// lieu de movementX/Y. Compromis : on ne pane plus « à l'infini » en un seul geste
// (le curseur peut atteindre le bord), mais le déplacement FONCTIONNE.
const IS_LINUX =
  typeof navigator !== "undefined" &&
  /Linux/i.test(navigator.userAgent) &&
  !/Android/i.test(navigator.userAgent);

let _wayland = false;
export function setWaylandMode(on: boolean): void {
  _wayland = on;
}
/** Vrai si on doit éviter warp+grab et paner via clientX/Y (Linux ou Wayland). */
function noWarp(): boolean {
  return IS_LINUX || _wayland;
}

// Dernière position absolue du curseur (mode clientX/Y). Remis à null au début/fin
// d'un pan pour que le 1er event serve de référence (delta 0), pas de saut.
let _lastClient: { x: number; y: number } | null = null;

// Diagnostic pan (lu par l'overlay PanDebug) — aide à voir ce que la machine
// renvoie vraiment quand un utilisateur signale un pan cassé.
const _dbg = { mvX: 0, mvY: 0, cdx: 0, cdy: 0 };
export function getPanDebug(): {
  linux: boolean; wayland: boolean; mode: "clientXY" | "movement";
  mvX: number; mvY: number; cdx: number; cdy: number;
} {
  return {
    linux: IS_LINUX, wayland: _wayland, mode: noWarp() ? "clientXY" : "movement",
    ..._dbg,
  };
}

// ── Canvas bounds (updated by GlucoseCanvas on mount + resize) ───────────────
let _bounds = { top: 0, bottom: 0, left: 0, right: 0 };
export function setWrapBounds(top: number, bottom: number, left: number, right: number) {
  _bounds = { top, bottom, left, right };
}

// ── Cursor grab + hide ────────────────────────────────────────────────────────
// Hides and confines cursor during pan — same as Blender Continuous Grab.
// No cursor = no visual artifacts. movementX/Y gives clean relative deltas.
let _grabActive = false;
export function startCursorGrab(): void {
  if (_grabActive) return;
  _grabActive = true;
  _lastClient = null; // nouvelle session de pan → 1re position = référence
  // Linux/Wayland : NI grab NI masquage (le curseur doit rester libre et visible
  // pour que clientX/Y bougent → deltas fiables).
  if (noWarp()) return;
  const win = getCurrentWindow();
  win.setCursorGrab(true).catch(() => {});
  win.setCursorVisible(false).catch(() => {});
}
export function stopCursorGrab(): void {
  if (!_grabActive) return;
  _grabActive = false;
  _lastClient = null;
  if (noWarp()) return;
  const win = getCurrentWindow();
  win.setCursorGrab(false).catch(() => {});
  win.setCursorVisible(true).catch(() => {});
}

// ── Delta-based pan ───────────────────────────────────────────────────────────
// Uses PointerEvent.movementX/Y (OS relative deltas) instead of absolute
// clientX/Y. No stale-event problem: the single post-teleport event has a
// huge movement value (cursor jumped from edge to center) and is discarded.
const JUMP_THRESHOLD = 150; // px/event — impossible from real mouse, = post-teleport

/**
 * Call from every pointermove during a canvas pan/drag.
 * Returns { dx, dy } to apply to the camera, or null to skip this event.
 * Uses velocity-adaptive margin: wide when moving fast, narrow when slow,
 * so the reset zone is only as large as it needs to be.
 */
export function getPanDelta(
  movementX: number,
  movementY: number,
  clientX: number,
  clientY: number,
): { dx: number; dy: number } | null {
  _dbg.mvX = movementX;
  _dbg.mvY = movementY;

  // ── Mode Linux/Wayland : delta = variation de clientX/Y (fiable sur WebKitGTK) ──
  // Pas de warp, donc clientX/Y varient continûment → deltas propres, sans dépendre
  // de movementX/Y (cassés sous WebKitGTK).
  if (noWarp()) {
    if (_lastClient === null) {
      _lastClient = { x: clientX, y: clientY };
      _dbg.cdx = 0; _dbg.cdy = 0;
      return { dx: 0, dy: 0 };
    }
    const dx = clientX - _lastClient.x;
    const dy = clientY - _lastClient.y;
    _lastClient = { x: clientX, y: clientY };
    _dbg.cdx = dx; _dbg.cdy = dy;
    return { dx, dy };
  }

  // ── Mode warp (Windows / X11 / macOS) : movementX/Y + téléportation au bord ──
  // Skip the one stale post-teleport event (cursor jumped from edge to center)
  if (Math.abs(movementX) + Math.abs(movementY) > JUMP_THRESHOLD) return null;

  const left   = _bounds.left   > 0   ? _bounds.left   : 0;
  const right  = _bounds.right  > left ? _bounds.right  : window.innerWidth;
  const top    = _bounds.top    >= 0  ? _bounds.top    : 0;
  const bottom = _bounds.bottom > top  ? _bounds.bottom : window.innerHeight;

  // Velocity-adaptive margin: 16px when barely moving, up to 120px when very fast
  const speed  = Math.sqrt(movementX * movementX + movementY * movementY);
  const margin = Math.max(16, Math.min(120, speed * 3.5));

  const nearEdge =
    clientX - left   < margin ||
    right  - clientX < margin ||
    clientY - top    < margin ||
    bottom - clientY < margin;

  // Wayland : PAS de warp (bloqué/instable → boucle au bord + saccades). Le pan
  // reste piloté par movementX/Y ; on ne téléporte simplement plus le curseur.
  if (nearEdge && !_wayland) {
    const cx = Math.round((left + right)  / 2);
    const cy = Math.round((top  + bottom) / 2);
    getCurrentWindow()
      .setCursorPosition(new LogicalPosition(cx, cy))
      .catch(() => {});
  }

  return { dx: movementX, dy: movementY };
}

/**
 * Lightweight skip check for small embedded elements (minimap).
 * No edge-reset logic — just filters out the one post-teleport jump event.
 */
export function getMinimapDelta(
  movementX: number,
  movementY: number,
): { dx: number; dy: number } | null {
  if (Math.abs(movementX) + Math.abs(movementY) > JUMP_THRESHOLD) return null;
  return { dx: movementX, dy: movementY };
}
