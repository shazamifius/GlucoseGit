//! MEMB-3 — Rideau : le panneau personnel (géométrie et temporisation).
//! 100% Rust Standard Library (0 dépendance).

use crate::geometry::Rect;

pub mod curtain_consts {
    pub const DEFAULT_COLLAPSED: f64 = 0.1;
    pub const DEFAULT_EXPANDED: f64 = 0.9;
    pub const MIN_COLLAPSED: f64 = 0.02;
    pub const MAX_COLLAPSED: f64 = 0.3;
    pub const MIN_EXPANDED: f64 = 0.3;
    pub const MAX_EXPANDED: f64 = 0.97;
    pub const MIN_SPAN: f64 = 0.05;

    pub const EXPAND_DWELL_MS: i64 = 90;
    pub const COLLAPSE_DWELL_MS: i64 = 40;
    pub const ANIM_MS: i64 = 260;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurtainPhase {
    Collapsed,
    Expanded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurtainConfig {
    pub collapsed: f64,
    pub expanded: f64,
}

impl Default for CurtainConfig {
    fn default() -> Self {
        Self {
            collapsed: curtain_consts::DEFAULT_COLLAPSED,
            expanded: curtain_consts::DEFAULT_EXPANDED,
        }
    }
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    if !v.is_finite() {
        lo
    } else {
        v.max(lo).min(hi)
    }
}

pub fn normalize_config(cfg: Option<CurtainConfig>) -> CurtainConfig {
    let raw = cfg.unwrap_or_default();
    let collapsed = clamp(
        raw.collapsed,
        curtain_consts::MIN_COLLAPSED,
        curtain_consts::MAX_COLLAPSED,
    );
    let expanded = clamp(
        raw.expanded,
        curtain_consts::MIN_EXPANDED.max(collapsed + curtain_consts::MIN_SPAN),
        curtain_consts::MAX_EXPANDED,
    );
    CurtainConfig {
        collapsed,
        expanded,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurtainState {
    pub ratio: f64,
    pub phase: CurtainPhase,
    pub pending: Option<CurtainPhase>,
    pub pending_since: i64,
}

impl CurtainState {
    pub fn new(cfg: CurtainConfig) -> Self {
        Self {
            ratio: cfg.collapsed,
            phase: CurtainPhase::Collapsed,
            pending: None,
            pending_since: 0,
        }
    }
}

pub fn panel_rect(ratio: f64, screen_w: f64, screen_h: f64) -> Rect {
    let width = screen_w * ratio;
    Rect::new(screen_w - width, 0.0, width, screen_h)
}

pub fn canvas_strip(ratio: f64, screen_w: f64, screen_h: f64) -> Rect {
    let p = panel_rect(ratio, screen_w, screen_h);
    Rect::new(0.0, 0.0, screen_w - p.width, screen_h)
}

pub fn is_over_panel(cursor_x: Option<f64>, ratio: f64, screen_w: f64) -> bool {
    match cursor_x {
        Some(cx) => cx >= screen_w * (1.0 - ratio),
        None => false,
    }
}

pub fn decide(
    state: &CurtainState,
    cursor_x: Option<f64>,
    screen_w: f64,
    now: i64,
) -> CurtainState {
    let want = if is_over_panel(cursor_x, state.ratio, screen_w) {
        CurtainPhase::Expanded
    } else {
        CurtainPhase::Collapsed
    };

    if want == state.phase {
        if state.pending.is_none() {
            return state.clone();
        } else {
            let mut s = state.clone();
            s.pending = None;
            s.pending_since = 0;
            return s;
        }
    }

    if state.pending != Some(want) {
        let mut s = state.clone();
        s.pending = Some(want);
        s.pending_since = now;
        return s;
    }

    let dwell = if want == CurtainPhase::Expanded {
        curtain_consts::EXPAND_DWELL_MS
    } else {
        curtain_consts::COLLAPSE_DWELL_MS
    };

    if now - state.pending_since < dwell {
        return state.clone();
    }

    let mut s = state.clone();
    s.phase = want;
    s.pending = None;
    s.pending_since = 0;
    s
}

pub fn advance(state: &CurtainState, cfg: &CurtainConfig, dt_ms: f64) -> CurtainState {
    let target = match state.phase {
        CurtainPhase::Expanded => cfg.expanded,
        CurtainPhase::Collapsed => cfg.collapsed,
    };
    if (state.ratio - target).abs() < 1e-9 {
        return state.clone();
    }

    let tau = (curtain_consts::ANIM_MS as f64) / 3.0;
    let k = 1.0 - (-dt_ms.max(0.0) / tau).exp();
    let mut ratio = state.ratio + (target - state.ratio) * k;
    if (target - ratio).abs() < 0.0005 {
        ratio = target;
    }
    let mut s = state.clone();
    s.ratio = ratio;
    s
}

pub fn step(
    state: &CurtainState,
    cursor_x: Option<f64>,
    screen_w: f64,
    cfg: &CurtainConfig,
    now: i64,
    dt_ms: f64,
) -> CurtainState {
    let decided = decide(state, cursor_x, screen_w, now);
    advance(&decided, cfg, dt_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_config() {
        let bad = CurtainConfig {
            collapsed: -0.5,
            expanded: 2.0,
        };
        let norm = normalize_config(Some(bad));
        assert_eq!(norm.collapsed, curtain_consts::MIN_COLLAPSED);
        assert_eq!(norm.expanded, curtain_consts::MAX_EXPANDED);
    }

    #[test]
    fn test_is_over_panel() {
        // écran 1000px, ratio 0.2 -> frontière à 800px
        assert!(is_over_panel(Some(850.0), 0.2, 1000.0));
        assert!(!is_over_panel(Some(750.0), 0.2, 1000.0));
        assert!(!is_over_panel(None, 0.2, 1000.0));
    }
}
