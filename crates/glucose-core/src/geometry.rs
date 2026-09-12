//! Géométrie 2D PURE — 100% Rust Standard Library (0 dépendance).

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: Point) -> f64 {
        f64::hypot(other.x - self.x, other.y - self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub const ZERO: Self = Self {
        left: 0.0,
        top: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub fn new(left: f64, top: f64, width: f64, height: f64) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }

    pub fn right(&self) -> f64 {
        self.left + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.top + self.height
    }

    pub fn center(&self) -> Point {
        Point::new(self.left + self.width / 2.0, self.top + self.height / 2.0)
    }

    pub fn area(&self) -> f64 {
        (self.width * self.height).abs()
    }

    pub fn contains_point(&self, p: Point) -> bool {
        p.x >= self.left && p.x <= self.right() && p.y >= self.top && p.y <= self.bottom()
    }

    pub fn contains_center(&self, other: Rect) -> bool {
        self.contains_point(other.center())
    }

    pub fn overlaps(&self, other: Rect) -> bool {
        self.left < other.right()
            && other.left < self.right()
            && self.top < other.bottom()
            && other.top < self.bottom()
    }

    pub fn union(&self, other: Rect) -> Rect {
        let left = self.left.min(other.left);
        let top = self.top.min(other.top);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::new(left, top, right - left, bottom - top)
    }

    pub fn corners(&self) -> [(&'static str, Point); 4] {
        [
            ("tl", Point::new(self.left, self.top)),
            ("tr", Point::new(self.right(), self.top)),
            ("bl", Point::new(self.left, self.bottom())),
            ("br", Point::new(self.right(), self.bottom())),
        ]
    }
}

/// Point dans un rectangle ancré en haut-gauche.
pub fn in_rect(px: f64, py: f64, x: f64, y: f64, w: f64, h: f64) -> bool {
    px >= x && px <= x + w && py >= y && py <= y + h
}

/// Point dans un rectangle ancré au CENTRE et pivoté de `rot` radians.
pub fn in_rotated_box(px: f64, py: f64, cx: f64, cy: f64, w: f64, h: f64, rot: f64) -> bool {
    let mut dx = px - cx;
    let mut dy = py - cy;
    if rot != 0.0 {
        let c = (-rot).cos();
        let s = (-rot).sin();
        let rx = dx * c - dy * s;
        let ry = dx * s + dy * c;
        dx = rx;
        dy = ry;
    }
    dx.abs() <= w / 2.0 && dy.abs() <= h / 2.0
}

/// Point dans la BANDE de bord d'un rectangle (dedans ET dehors de `band` px).
pub fn on_rect_edge(px: f64, py: f64, x: f64, y: f64, w: f64, h: f64, band: f64) -> bool {
    let outer = px >= x - band && px <= x + w + band && py >= y - band && py <= y + h + band;
    if !outer {
        return false;
    }
    if w <= band * 2.0 || h <= band * 2.0 {
        return true;
    }
    let inner = px >= x + band && px <= x + w - band && py >= y + band && py <= y + h - band;
    !inner
}

/// Intersection de deux segments [p1, p2] et [p3, p4].
pub fn lines_intersect(p1: Point, p2: Point, p3: Point, p4: Point) -> bool {
    let det = (p2.x - p1.x) * (p4.y - p3.y) - (p2.y - p1.y) * (p4.x - p3.x);
    if det.abs() < 1e-9 {
        return false;
    }
    let lambda = ((p4.y - p3.y) * (p4.x - p1.x) + (p3.x - p4.x) * (p4.y - p1.y)) / det;
    let gamma = ((p1.y - p2.y) * (p4.x - p1.x) + (p2.x - p1.x) * (p4.y - p1.y)) / det;
    (0.0..1.0).contains(&lambda) && (0.0..1.0).contains(&gamma)
}

/// Intersection d'un segment [p1, p2] avec un rectangle.
pub fn line_intersects_rect(p1: Point, p2: Point, r: Rect) -> bool {
    let tl = Point::new(r.left, r.top);
    let tr = Point::new(r.right(), r.top);
    let bl = Point::new(r.left, r.bottom());
    let br = Point::new(r.right(), r.bottom());

    if lines_intersect(p1, p2, tl, tr)
        || lines_intersect(p1, p2, tr, br)
        || lines_intersect(p1, p2, br, bl)
        || lines_intersect(p1, p2, bl, tl)
    {
        return true;
    }

    r.contains_point(p1) || r.contains_point(p2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(3.0, 4.0);
        assert!((a.distance_to(b) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_rect_contains_and_overlaps() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(r1.contains_point(Point::new(50.0, 50.0)));
        assert!(!r1.contains_point(Point::new(150.0, 50.0)));

        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        assert!(r1.overlaps(r2));

        let r3 = Rect::new(200.0, 200.0, 50.0, 50.0);
        assert!(!r1.overlaps(r3));
    }

    #[test]
    fn test_rotated_box() {
        // Carré 100x100 centré en (100, 100), rotation 0
        assert!(in_rotated_box(
            100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 0.0
        ));
        assert!(in_rotated_box(
            140.0, 140.0, 100.0, 100.0, 100.0, 100.0, 0.0
        ));
        assert!(!in_rotated_box(
            160.0, 100.0, 100.0, 100.0, 100.0, 100.0, 0.0
        ));
    }

    #[test]
    fn test_on_rect_edge() {
        // Rectangle de 0,0 à 100,100 avec bande de 10px
        // Sur le bord intérieur
        assert!(on_rect_edge(5.0, 50.0, 0.0, 0.0, 100.0, 100.0, 10.0));
        // Sur le bord extérieur
        assert!(on_rect_edge(-5.0, 50.0, 0.0, 0.0, 100.0, 100.0, 10.0));
        // Plein centre -> pas sur le bord
        assert!(!on_rect_edge(50.0, 50.0, 0.0, 0.0, 100.0, 100.0, 10.0));
    }
}
