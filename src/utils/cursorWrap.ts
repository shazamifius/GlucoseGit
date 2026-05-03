import { getCurrentWindow, LogicalPosition } from "@tauri-apps/api/window";

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
  const win = getCurrentWindow();
  win.setCursorGrab(true).catch(() => {});
  win.setCursorVisible(false).catch(() => {});
}
export function stopCursorGrab(): void {
  if (!_grabActive) return;
  _grabActive = false;
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

  if (nearEdge) {
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
