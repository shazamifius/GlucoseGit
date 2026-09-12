//! MEMB-2 — Mode Focus : « zoomer assez sur une membrane et n'avoir plus qu'elle ».
//! 100% Rust Standard Library (0 dépendance).

use crate::geometry::{Point, Rect};
use crate::membrane_space::{contained_in, content_extent, parent_map, SpaceItem};
use crate::types::{Annotation, Viewport};
use std::collections::{HashMap, HashSet};

pub mod focus_consts {
    pub const ENTER_COVERAGE: f64 = 0.92;
    pub const EXIT_SCALE_RATIO: f64 = 0.8;
    pub const EXIT_CENTER_MARGIN: f64 = 0.35;
    pub const COOLDOWN_MS: i64 = 400;
    pub const FIT_PADDING: f64 = 0.06;
    pub const FIT_ANIM_MS: i64 = 320;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FocusState {
    pub membrane_id: Option<String>,
    pub enter_scale: f64,
    pub t: i64,
}

impl Default for FocusState {
    fn default() -> Self {
        Self {
            membrane_id: None,
            enter_scale: 0.0,
            t: 0,
        }
    }
}

pub fn to_screen(b: Rect, vp: Viewport) -> Rect {
    Rect {
        left: b.left * vp.scale + vp.x,
        top: b.top * vp.scale + vp.y,
        width: b.width * vp.scale,
        height: b.height * vp.scale,
    }
}

pub fn coverage(b: Rect, vp: Viewport, screen: ScreenSize) -> f64 {
    let total = screen.width * screen.height;
    if total <= 0.0 {
        return 0.0;
    }
    let s = to_screen(b, vp);
    let ix = 0.0f64.max((s.left + s.width).min(screen.width) - s.left.max(0.0));
    let iy = 0.0f64.max((s.top + s.height).min(screen.height) - s.top.max(0.0));
    (ix * iy) / total
}

pub fn screen_center_world(vp: Viewport, screen: ScreenSize) -> Point {
    Point {
        x: (screen.width / 2.0 - vp.x) / vp.scale,
        y: (screen.height / 2.0 - vp.y) / vp.scale,
    }
}

pub fn focus_box(membrane: &SpaceItem, children: &[&SpaceItem]) -> Rect {
    let m_rect = membrane.rect();
    let (ext_w, ext_h) = content_extent(m_rect, children);
    Rect {
        left: membrane.x,
        top: membrane.y,
        width: membrane.width.max(ext_w),
        height: membrane.height.max(ext_h),
    }
}

pub fn fit_viewport(b: Rect, screen: ScreenSize, pad: f64) -> Viewport {
    let usable_w = screen.width * (1.0 - pad * 2.0);
    let usable_h = screen.height * (1.0 - pad * 2.0);
    let scale = (usable_w / b.width.max(1e-6)).min(usable_h / b.height.max(1e-6));
    Viewport {
        scale,
        x: screen.width / 2.0 - (b.left + b.width / 2.0) * scale,
        y: screen.height / 2.0 - (b.top + b.height / 2.0) * scale,
    }
}

#[derive(Debug, Clone)]
pub struct FocusInput<'a> {
    pub items: &'a [SpaceItem],
    pub resolved: &'a HashMap<String, crate::membrane_space::ResolvedItem>,
    pub vp: Viewport,
    pub screen: ScreenSize,
    pub state: FocusState,
    pub now: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusAction {
    Stay(FocusState),
    Enter {
        state: FocusState,
        membrane_id: String,
        fit: Viewport,
    },
    Exit(FocusState),
}

fn children_of<'a>(items: &'a [SpaceItem], membrane_id: &str) -> Vec<&'a SpaceItem> {
    let parents = parent_map(items);
    items
        .iter()
        .filter(|i| parents.get(&i.id).map(|s| s.as_str()) == Some(membrane_id))
        .collect()
}

pub fn focus_decision(input: FocusInput) -> FocusAction {
    let FocusInput {
        items,
        resolved,
        vp,
        screen,
        state,
        now,
    } = input;

    if now - state.t < focus_consts::COOLDOWN_MS {
        return FocusAction::Stay(state);
    }

    // ── Déjà en focus : seule la sortie est évaluée ─────────────────────────
    if let Some(ref mid) = state.membrane_id {
        let membrane = items
            .iter()
            .find(|i| i.id == *mid && i.kind == crate::membrane_space::SpaceItemKind::Membrane);
        let membrane = match membrane {
            Some(m) => m,
            None => {
                return FocusAction::Exit(FocusState {
                    membrane_id: None,
                    enter_scale: 0.0,
                    t: now,
                });
            }
        };

        if vp.scale < state.enter_scale * focus_consts::EXIT_SCALE_RATIO {
            return FocusAction::Exit(FocusState {
                membrane_id: None,
                enter_scale: 0.0,
                t: now,
            });
        }

        let kids = children_of(items, mid);
        let fb = focus_box(membrane, &kids);
        let c = screen_center_world(vp, screen);
        let mx = fb.width * focus_consts::EXIT_CENTER_MARGIN;
        let my = fb.height * focus_consts::EXIT_CENTER_MARGIN;
        let inside = c.x >= fb.left - mx
            && c.x <= fb.right() + mx
            && c.y >= fb.top - my
            && c.y <= fb.bottom() + my;

        if !inside {
            return FocusAction::Exit(FocusState {
                membrane_id: None,
                enter_scale: 0.0,
                t: now,
            });
        }

        return FocusAction::Stay(state);
    }

    // ── Hors focus : chercher la membrane qui remplit l'écran ───────────────
    let center = screen_center_world(vp, screen);
    let mut best: Option<&SpaceItem> = None;
    let mut best_area = f64::INFINITY;

    for it in items {
        if it.kind != crate::membrane_space::SpaceItemKind::Membrane {
            continue;
        }
        let r = match resolved.get(&it.id) {
            Some(r) => r,
            None => continue,
        };
        let b = r.rect();
        if center.x < b.left || center.x > b.right() || center.y < b.top || center.y > b.bottom() {
            continue;
        }
        if coverage(b, vp, screen) < focus_consts::ENTER_COVERAGE {
            continue;
        }
        let a = (b.width * b.height).abs();
        if a < best_area {
            best = Some(it);
            best_area = a;
        }
    }

    if let Some(target) = best {
        let kids = children_of(items, &target.id);
        let fit = fit_viewport(focus_box(target, &kids), screen, focus_consts::FIT_PADDING);
        let new_state = FocusState {
            membrane_id: Some(target.id.clone()),
            enter_scale: fit.scale,
            t: now,
        };
        FocusAction::Enter {
            state: new_state,
            membrane_id: target.id.clone(),
            fit,
        }
    } else {
        FocusAction::Stay(state)
    }
}

pub fn focus_frame_of(items: &[SpaceItem], focused_id: Option<&str>) -> Option<Rect> {
    let fid = focused_id?;
    let m = items
        .iter()
        .find(|i| i.id == fid && i.kind == crate::membrane_space::SpaceItemKind::Membrane)?;
    let kids = children_of(items, fid);
    Some(focus_box(m, &kids))
}

pub fn visible_under_focus(
    items: &[SpaceItem],
    focused_id: Option<&str>,
) -> Option<HashSet<String>> {
    let fid = focused_id?;
    let membrane = items.iter().find(|i| i.id == fid)?;

    let parents = parent_map(items);
    let mut kids_of: HashMap<&str, Vec<&SpaceItem>> = HashMap::new();
    for it in items {
        if let Some(p) = parents.get(&it.id) {
            kids_of.entry(p.as_str()).or_default().push(it);
        }
    }

    let mut out = HashSet::new();
    out.insert(fid.to_string());
    let mut stack = vec![fid];
    while let Some(id) = stack.pop() {
        if let Some(kids) = kids_of.get(id) {
            for kid in kids {
                if !out.contains(&kid.id) {
                    out.insert(kid.id.clone());
                    stack.push(&kid.id);
                }
            }
        }
    }

    for it in contained_in(items, membrane) {
        if !parents.contains_key(&it.id) {
            out.insert(it.id.clone());
        }
    }

    Some(out)
}

pub fn arrow_visible_under_focus(
    ann: &Annotation,
    visible: Option<&HashSet<String>>,
    focus: Option<Rect>,
) -> bool {
    let (vis, fc) = match (visible, focus) {
        (Some(v), Some(f)) => (v, f),
        _ => return true,
    };
    let (x, y, x2, y2, source_id, target_id) = match ann {
        Annotation::Arrow {
            x,
            y,
            x2,
            y2,
            source_id,
            target_id,
            ..
        } => (*x, *y, *x2, *y2, source_id, target_id),
        _ => return true,
    };

    let mut ends = Vec::new();
    if let Some(s) = source_id {
        ends.push(s.as_str());
    }
    if let Some(t) = target_id {
        ends.push(t.as_str());
    }

    if !ends.is_empty() {
        return ends.iter().all(|id| vis.contains(*id));
    }

    let in_box = |px: f64, py: f64| -> bool {
        px >= fc.left && px <= fc.right() && py >= fc.top && py <= fc.bottom()
    };
    in_box(x, y) && in_box(x2, y2)
}

pub fn annotation_visible_under_focus(
    ann: &Annotation,
    visible: Option<&HashSet<String>>,
    frame: Option<Rect>,
) -> bool {
    let vis = match visible {
        Some(v) => v,
        None => return true,
    };
    if matches!(ann, Annotation::Arrow { .. }) {
        return arrow_visible_under_focus(ann, visible, frame);
    }
    if vis.contains(ann.id()) {
        return true;
    }

    let (measured, ax, ay) = match ann {
        Annotation::Text {
            width,
            height,
            x,
            y,
            ..
        } => (
            width.unwrap_or(0.0) > 0.0 && height.unwrap_or(0.0) > 0.0,
            *x,
            *y,
        ),
        Annotation::Sticky {
            width,
            height,
            x,
            y,
            ..
        } => (
            width.unwrap_or(0.0) > 0.0 && height.unwrap_or(0.0) > 0.0,
            *x,
            *y,
        ),
        _ => (true, ann.x(), ann.y()),
    };

    if measured {
        return false;
    }
    let fr = match frame {
        Some(f) => f,
        None => return false,
    };
    ax >= fr.left && ax <= fr.right() && ay >= fr.top && ay <= fr.bottom()
}

fn parse_hex(c: &str) -> Option<(u8, u8, u8)> {
    let m = c.trim().trim_start_matches('#');
    if m.len() == 3 {
        let r = u8::from_str_radix(&m[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&m[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&m[2..3].repeat(2), 16).ok()?;
        Some((r, g, b))
    } else if m.len() == 6 {
        let r = u8::from_str_radix(&m[0..2], 16).ok()?;
        let g = u8::from_str_radix(&m[2..4], 16).ok()?;
        let b = u8::from_str_radix(&m[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

pub fn focus_background(color: Option<&str>, base: Option<&str>, amount: Option<f64>) -> String {
    let base_str = base.unwrap_or("#0d0d0d");
    let amt = amount.unwrap_or(0.16).clamp(0.0, 1.0);
    let col = match color {
        Some(c) if !c.is_empty() => c,
        _ => return base_str.to_string(),
    };
    let c = match parse_hex(col) {
        Some(rgb) => rgb,
        None => return base_str.to_string(),
    };
    let b = match parse_hex(base_str) {
        Some(rgb) => rgb,
        None => return base_str.to_string(),
    };

    let mix = |ci: u8, bi: u8| -> u8 {
        let val = (bi as f64 + (ci as f64 - bi as f64) * amt).round();
        val.clamp(0.0, 255.0) as u8
    };

    format!(
        "#{:02x}{:02x}{:02x}",
        mix(c.0, b.0),
        mix(c.1, b.1),
        mix(c.2, b.2)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::membrane_space::SpaceItemKind;

    #[test]
    fn test_coverage() {
        let b = Rect::new(0.0, 0.0, 1000.0, 1000.0);
        let screen = ScreenSize {
            width: 1000.0,
            height: 1000.0,
        };
        let vp = Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        };
        assert!((coverage(b, vp, screen) - 1.0).abs() < 1e-6);

        let vp_half = Viewport {
            x: 0.0,
            y: 0.0,
            scale: 0.5,
        };
        // 500x500 sur 1000x1000 = 0.25
        assert!((coverage(b, vp_half, screen) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_asymmetric_exit() {
        let m = SpaceItem {
            id: "M1".into(),
            kind: SpaceItemKind::Membrane,
            x: 0.0,
            y: 0.0,
            width: 1000.0,
            height: 1000.0,
            mode: None,
            membrane_id: None,
        };
        let items = [m];
        let mut resolved = HashMap::new();
        resolved.insert(
            "M1".into(),
            crate::membrane_space::ResolvedItem {
                id: "M1".into(),
                x: 0.0,
                y: 0.0,
                width: 1000.0,
                height: 1000.0,
                scale: 1.0,
                membrane_id: None,
                content_scale: None,
            },
        );

        let screen = ScreenSize {
            width: 1000.0,
            height: 1000.0,
        };
        let state = FocusState {
            membrane_id: Some("M1".into()),
            enter_scale: 1.0,
            t: 0,
        };

        // Dézoom à 0.79 (< 0.8 * enter_scale) -> Sortie
        let action = focus_decision(FocusInput {
            items: &items,
            resolved: &resolved,
            vp: Viewport {
                x: 0.0,
                y: 0.0,
                scale: 0.79,
            },
            screen,
            state: state.clone(),
            now: 1000,
        });
        assert!(matches!(action, FocusAction::Exit(_)));

        // Dézoom à 0.85 (>= 0.8 * enter_scale) -> Reste
        let action2 = focus_decision(FocusInput {
            items: &items,
            resolved: &resolved,
            vp: Viewport {
                x: 0.0,
                y: 0.0,
                scale: 0.85,
            },
            screen,
            state,
            now: 1000,
        });
        assert!(matches!(action2, FocusAction::Stay(_)));
    }
}
