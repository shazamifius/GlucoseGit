//! Ancrage des flèches au périmètre des blocs — 0 dépendance.

use crate::geometry::Point;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnchorBox {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowAnchor {
    pub x: f64,
    pub y: f64,
    pub box_rect: Option<AnchorBox>,
}

impl ArrowAnchor {
    pub fn point(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            box_rect: None,
        }
    }

    pub fn with_box(x: f64, y: f64, left: f64, right: f64, top: f64, bottom: f64) -> Self {
        Self {
            x,
            y,
            box_rect: Some(AnchorBox {
                left,
                right,
                top,
                bottom,
            }),
        }
    }
}

pub const ANCHOR_MARGIN: f64 = 12.0;
pub const MIN_ARROW_LEN: f64 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct PerimeterExit {
    x: f64,
    y: f64,
    ux: f64,
    uy: f64,
}

fn perimeter_exit(anchor: ArrowAnchor, target: Point) -> PerimeterExit {
    let ax = anchor.x;
    let ay = anchor.y;
    let dx = target.x - ax;
    let dy = target.y - ay;
    let len = f64::hypot(dx, dy);

    let b = match anchor.box_rect {
        Some(b) if len > 0.0 => b,
        _ => return PerimeterExit { x: ax, y: ay, ux: 0.0, uy: 0.0 },
    };

    let mut t = f64::INFINITY;
    if dx > 0.0 {
        t = t.min((b.right - ax) / dx);
    } else if dx < 0.0 {
        t = t.min((b.left - ax) / dx);
    }
    if dy > 0.0 {
        t = t.min((b.bottom - ay) / dy);
    } else if dy < 0.0 {
        t = t.min((b.top - ay) / dy);
    }

    if !t.is_finite() || t < 0.0 {
        t = 0.0;
    }

    PerimeterExit {
        x: ax + dx * t,
        y: ay + dy * t,
        ux: dx / len,
        uy: dy / len,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowEndpoints {
    pub start: Point,
    pub end: Point,
}

pub fn arrow_endpoints(
    start: ArrowAnchor,
    end: ArrowAnchor,
    waypoints: &[Point],
) -> ArrowEndpoints {
    let a_target = waypoints.first().copied().unwrap_or(Point::new(end.x, end.y));
    let b_target = waypoints.last().copied().unwrap_or(Point::new(start.x, start.y));

    let a = perimeter_exit(start, a_target);
    let b = perimeter_exit(end, b_target);

    let room = |from: Point, to: Point| -> f64 {
        0.0f64.max(from.distance_to(to) - MIN_ARROW_LEN)
    };

    let (m_start, m_end) = if !waypoints.is_empty() {
        let first_wp = waypoints[0];
        let last_wp = waypoints[waypoints.len() - 1];
        let ms = ANCHOR_MARGIN.min(room(Point::new(a.x, a.y), first_wp));
        let me = ANCHOR_MARGIN.min(room(Point::new(b.x, b.y), last_wp));
        (ms, me)
    } else {
        let half = room(Point::new(a.x, a.y), Point::new(b.x, b.y)) / 2.0;
        let ms = ANCHOR_MARGIN.min(half);
        (ms, ms)
    };

    ArrowEndpoints {
        start: Point::new(a.x + a.ux * m_start, a.y + a.uy * m_start),
        end: Point::new(b.x + b.ux * m_end, b.y + b.uy * m_end),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(x: f64, y: f64, w: f64, h: f64) -> ArrowAnchor {
        ArrowAnchor {
            x: x + w / 2.0,
            y: y + h / 2.0,
            box_rect: Some(AnchorBox {
                left: x,
                right: x + w,
                top: y,
                bottom: y + h,
            }),
        }
    }

    #[test]
    fn test_arrow_endpoints_exit_facing_side() {
        let a = note(0.0, 0.0, 200.0, 100.0);
        let b = note(0.0, 500.0, 200.0, 100.0);
        let endpoints = arrow_endpoints(a, b, &[]);
        assert!(endpoints.start.y > 100.0); // sort par le bas de A (y=100)
        assert!(endpoints.end.y < 500.0);   // au-dessus du haut de B (y=500)
        assert!((endpoints.start.x - 100.0).abs() < 1e-6);
        assert!((endpoints.end.x - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_arrow_never_inverts() {
        // Deux notes très proches : gap = 2px
        let a = note(0.0, 0.0, 200.0, 100.0);
        let b = note(0.0, 102.0, 200.0, 100.0);
        let endpoints = arrow_endpoints(a, b, &[]);
        // La start reste au-dessus de end
        assert!(endpoints.start.y <= endpoints.end.y);
    }
}
