//! SNAP-1 — Alignement intelligent : MOTEUR PUR (0 dépendance).

use crate::types::{Annotation, Board, BoardImage, CanvasFolder};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignRect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl AlignRect {
    pub fn new(left: f64, top: f64, width: f64, height: f64) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignKind {
    Image,
    Text,
    Sticky,
    Membrane,
    Folder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlignTarget {
    pub id: String,
    pub kind: AlignKind,
    pub rect: AlignRect,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SnapGuides {
    pub x: Option<Vec<f64>>,
    pub y: Option<Vec<f64>>,
}

pub const SNAP_SCREEN_PX: f64 = 8.0;
pub const DEFAULT_ANN_W: f64 = 200.0;
pub const DEFAULT_ANN_H: f64 = 100.0;

#[derive(Debug, Clone, Copy)]
pub struct SnapOptions {
    pub scale: f64,
    pub threshold_px: f64,
    pub axis_x: bool,
    pub axis_y: bool,
}

impl Default for SnapOptions {
    fn default() -> Self {
        Self {
            scale: 1.0,
            threshold_px: SNAP_SCREEN_PX,
            axis_x: true,
            axis_y: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveSnap {
    pub dx: f64,
    pub dy: f64,
    pub guides: SnapGuides,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResizeSnap {
    pub rect: AlignRect,
    pub guides: SnapGuides,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointSnap {
    pub x: f64,
    pub y: f64,
    pub guides: SnapGuides,
}

pub fn rect_of_image(img: &BoardImage) -> AlignRect {
    AlignRect {
        left: img.x - img.width / 2.0,
        top: img.y - img.height / 2.0,
        width: img.width,
        height: img.height,
    }
}

pub fn rect_of_annotation(ann: &Annotation) -> Option<AlignRect> {
    match ann {
        Annotation::Arrow { .. } => None,
        Annotation::Membrane {
            x,
            y,
            width,
            height,
            ..
        } => Some(AlignRect {
            left: *x,
            top: *y,
            width: *width,
            height: *height,
        }),
        Annotation::Text {
            x,
            y,
            width,
            height,
            ..
        } => Some(AlignRect {
            left: *x,
            top: *y,
            width: width.unwrap_or(DEFAULT_ANN_W),
            height: height.unwrap_or(DEFAULT_ANN_H),
        }),
        Annotation::Sticky {
            x,
            y,
            width,
            height,
            ..
        } => Some(AlignRect {
            left: *x,
            top: *y,
            width: width.unwrap_or(160.0),
            height: height.unwrap_or(120.0),
        }),
    }
}

pub fn rect_of_folder(f: &CanvasFolder) -> AlignRect {
    AlignRect {
        left: f.x,
        top: f.y,
        width: f.width,
        height: f.height,
    }
}

pub fn union_rect(rects: &[AlignRect]) -> Option<AlignRect> {
    if rects.is_empty() {
        return None;
    }
    let mut l = f64::INFINITY;
    let mut t = f64::INFINITY;
    let mut r = f64::NEG_INFINITY;
    let mut b = f64::NEG_INFINITY;
    for rc in rects {
        l = l.min(rc.left);
        t = t.min(rc.top);
        r = r.max(rc.left + rc.width);
        b = b.max(rc.top + rc.height);
    }
    Some(AlignRect {
        left: l,
        top: t,
        width: r - l,
        height: b - t,
    })
}

pub fn collect_align_targets(board: &Board, exclude: &HashSet<String>) -> Vec<AlignTarget> {
    let mut out = Vec::new();

    for img in &board.images {
        if exclude.contains(&img.id) {
            continue;
        }
        out.push(AlignTarget {
            id: img.id.clone(),
            kind: AlignKind::Image,
            rect: rect_of_image(img),
        });
    }

    for ann in &board.annotations {
        if exclude.contains(ann.id()) {
            continue;
        }
        if let Some(rect) = rect_of_annotation(ann) {
            let kind = match ann {
                Annotation::Membrane { .. } => AlignKind::Membrane,
                Annotation::Sticky { .. } => AlignKind::Sticky,
                Annotation::Text { .. } => AlignKind::Text,
                Annotation::Arrow { .. } => unreachable!(),
            };
            out.push(AlignTarget {
                id: ann.id().to_string(),
                kind,
                rect,
            });
        }
    }

    for f in &board.folders {
        if exclude.contains(&f.id) {
            continue;
        }
        out.push(AlignTarget {
            id: f.id.clone(),
            kind: AlignKind::Folder,
            rect: rect_of_folder(f),
        });
    }

    out
}

fn target_lines_x(rect: AlignRect) -> [f64; 3] {
    [
        rect.left,
        rect.left + rect.width / 2.0,
        rect.left + rect.width,
    ]
}

fn target_lines_y(rect: AlignRect) -> [f64; 3] {
    [
        rect.top,
        rect.top + rect.height / 2.0,
        rect.top + rect.height,
    ]
}

#[derive(Debug, Clone, Copy)]
struct AxisSnap {
    delta: f64,
    line: f64,
}

fn best_axis_snap(mine: &[f64], theirs: &[f64], threshold: f64) -> Option<AxisSnap> {
    let mut best = None;
    let mut best_dist = threshold;
    for &m in mine {
        for &t in theirs {
            let d = t - m;
            let dist = d.abs();
            if dist < best_dist {
                best_dist = dist;
                best = Some(AxisSnap { delta: d, line: t });
            }
        }
    }
    best
}

fn threshold_of(opts: &SnapOptions) -> f64 {
    let scale = if opts.scale > 0.0 { opts.scale } else { 1.0 };
    opts.threshold_px / scale
}

pub fn snap_move(rect: AlignRect, targets: &[AlignTarget], opts: SnapOptions) -> MoveSnap {
    let threshold = threshold_of(&opts);
    let cx = rect.left + rect.width / 2.0;
    let cy = rect.top + rect.height / 2.0;
    let mine_x = [cx, rect.left, rect.left + rect.width];
    let mine_y = [cy, rect.top, rect.top + rect.height];

    let mut theirs_x = Vec::new();
    let mut theirs_y = Vec::new();
    for t in targets {
        theirs_x.extend_from_slice(&target_lines_x(t.rect));
        theirs_y.extend_from_slice(&target_lines_y(t.rect));
    }

    let sx = if opts.axis_x {
        best_axis_snap(&mine_x, &theirs_x, threshold)
    } else {
        None
    };
    let sy = if opts.axis_y {
        best_axis_snap(&mine_y, &theirs_y, threshold)
    } else {
        None
    };

    MoveSnap {
        dx: sx.map_or(0.0, |s| s.delta),
        dy: sy.map_or(0.0, |s| s.delta),
        guides: SnapGuides {
            x: sx.map(|s| vec![s.line]),
            y: sy.map(|s| vec![s.line]),
        },
    }
}

pub fn snap_resize(
    rect: AlignRect,
    handle: &str,
    targets: &[AlignTarget],
    opts: SnapOptions,
    min_width: f64,
    min_height: f64,
) -> ResizeSnap {
    let threshold = threshold_of(&opts);
    let min_w = min_width.max(1.0);
    let min_h = min_height.max(1.0);

    let moves_left = handle.contains('l');
    let moves_right = handle.contains('r');
    let moves_top = handle.contains('t');
    let moves_bottom = handle.contains('b');

    let mut theirs_x = Vec::new();
    let mut theirs_y = Vec::new();
    for t in targets {
        theirs_x.extend_from_slice(&target_lines_x(t.rect));
        theirs_y.extend_from_slice(&target_lines_y(t.rect));
    }

    let mut left = rect.left;
    let mut top = rect.top;
    let mut width = rect.width;
    let mut height = rect.height;
    let mut guides = SnapGuides::default();

    if opts.axis_x {
        let mut mine_x = Vec::new();
        if moves_left {
            mine_x.push(left);
        }
        if moves_right {
            mine_x.push(left + width);
        }
        if let Some(s) = best_axis_snap(&mine_x, &theirs_x, threshold) {
            if moves_left {
                let right = left + width;
                let nw = right - s.line;
                if nw >= min_w {
                    left = s.line;
                    width = nw;
                    guides.x = Some(vec![s.line]);
                }
            } else {
                let nw = s.line - left;
                if nw >= min_w {
                    width = nw;
                    guides.x = Some(vec![s.line]);
                }
            }
        }
    }

    if opts.axis_y {
        let mut mine_y = Vec::new();
        if moves_top {
            mine_y.push(top);
        }
        if moves_bottom {
            mine_y.push(top + height);
        }
        if let Some(s) = best_axis_snap(&mine_y, &theirs_y, threshold) {
            if moves_top {
                let bottom = top + height;
                let nh = bottom - s.line;
                if nh >= min_h {
                    top = s.line;
                    height = nh;
                    guides.y = Some(vec![s.line]);
                }
            } else {
                let nh = s.line - top;
                if nh >= min_h {
                    height = nh;
                    guides.y = Some(vec![s.line]);
                }
            }
        }
    }

    ResizeSnap {
        rect: AlignRect {
            left,
            top,
            width,
            height,
        },
        guides,
    }
}

pub fn snap_point(x: f64, y: f64, targets: &[AlignTarget], opts: SnapOptions) -> PointSnap {
    let r = snap_move(AlignRect::new(x, y, 0.0, 0.0), targets, opts);
    PointSnap {
        x: x + r.dx,
        y: y + r.dy,
        guides: r.guides,
    }
}

pub fn same_guides(a: Option<&SnapGuides>, b: Option<&SnapGuides>) -> bool {
    let ax = a.and_then(|g| g.x.as_deref()).unwrap_or(&[]);
    let ay = a.and_then(|g| g.y.as_deref()).unwrap_or(&[]);
    let bx = b.and_then(|g| g.x.as_deref()).unwrap_or(&[]);
    let by = b.and_then(|g| g.y.as_deref()).unwrap_or(&[]);
    ax == bx && ay == by
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ref_target() -> AlignTarget {
        AlignTarget {
            id: "ref".into(),
            kind: AlignKind::Text,
            rect: AlignRect::new(0.0, 0.0, 100.0, 100.0),
        }
    }

    #[test]
    fn test_snap_move_left_edge() {
        let r = snap_move(
            AlignRect::new(3.0, 500.0, 40.0, 40.0),
            &[ref_target()],
            SnapOptions::default(),
        );
        assert!((r.dx - (-3.0)).abs() < 1e-6);
        assert_eq!(r.guides.x, Some(vec![0.0]));
    }

    #[test]
    fn test_snap_move_centers() {
        // centre cible = 50, boîte left=27 width=40 => centre = 47 => dx = +3
        let r = snap_move(
            AlignRect::new(27.0, 500.0, 40.0, 40.0),
            &[ref_target()],
            SnapOptions::default(),
        );
        assert!((r.dx - 3.0).abs() < 1e-6);
        assert_eq!(r.guides.x, Some(vec![50.0]));
    }

    #[test]
    fn test_snap_beyond_threshold() {
        let r = snap_move(
            AlignRect::new(40.0, 500.0, 40.0, 40.0),
            &[ref_target()],
            SnapOptions::default(),
        );
        assert_eq!(r.dx, 0.0);
        assert_eq!(r.guides.x, None);
    }

    #[test]
    fn test_snap_scale_threshold() {
        let t = [ref_target()];
        // scale 0.1 -> threshold = 8 / 0.1 = 80 unités monde -> 20 unités accroche
        let p1 = snap_point(
            20.0,
            500.0,
            &t,
            SnapOptions {
                scale: 0.1,
                ..Default::default()
            },
        );
        assert!((p1.x - 0.0).abs() < 1e-6);

        // scale 4.0 -> threshold = 8 / 4 = 2 unités monde -> 20 unités n'accroche pas
        let p2 = snap_point(
            20.0,
            500.0,
            &t,
            SnapOptions {
                scale: 4.0,
                ..Default::default()
            },
        );
        assert_eq!(p2.x, 20.0);
    }
}
