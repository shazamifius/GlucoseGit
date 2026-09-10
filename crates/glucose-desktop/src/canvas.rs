//! Module Canvas : transformations de coordonnées Écran <-> Monde et caméra PureRef.

use glucose_core::types::Viewport;

/// Convertit des coordonnées écran (pixels fenêtre) en coordonnées monde (espace canvas infini).
pub fn screen_to_world(screen_x: f64, screen_y: f64, vp: &Viewport) -> (f64, f64) {
    let wx = (screen_x - vp.x) / vp.scale;
    let wy = (screen_y - vp.y) / vp.scale;
    (wx, wy)
}

/// Convertit des coordonnées monde en coordonnées écran.
pub fn world_to_screen(world_x: f64, world_y: f64, vp: &Viewport) -> (f64, f64) {
    let sx = world_x * vp.scale + vp.x;
    let sy = world_y * vp.scale + vp.y;
    (sx, sy)
}

/// Calcule le nouveau viewport après un zoom centré sur la position du curseur `(cx, cy)`.
pub fn zoom_at(vp: &mut Viewport, factor: f64, cx: f64, cy: f64) {
    let old_scale = vp.scale;
    let new_scale = (old_scale * factor).clamp(0.01, 50.0);
    vp.x = cx - (cx - vp.x) * (new_scale / old_scale);
    vp.y = cy - (cy - vp.y) * (new_scale / old_scale);
    vp.scale = new_scale;
}

/// Déplace la caméra de `(dx, dy)` pixels écran.
#[allow(dead_code)]
pub fn pan(vp: &mut Viewport, dx: f64, dy: f64) {
    vp.x += dx;
    vp.y += dy;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_world_roundtrip() {
        let vp = Viewport { x: 100.0, y: 50.0, scale: 2.0 };
        let (sx, sy) = (250.0, 150.0);
        let (wx, wy) = screen_to_world(sx, sy, &vp);
        let (rx, ry) = world_to_screen(wx, wy, &vp);
        assert!((sx - rx).abs() < 1e-6);
        assert!((sy - ry).abs() < 1e-6);
    }

    #[test]
    fn test_zoom_preserves_cursor_world_position() {
        let mut vp = Viewport { x: 50.0, y: 50.0, scale: 1.0 };
        let cx = 300.0;
        let cy = 200.0;
        let (w_before_x, w_before_y) = screen_to_world(cx, cy, &vp);

        zoom_at(&mut vp, 1.5, cx, cy);

        let (w_after_x, w_after_y) = screen_to_world(cx, cy, &vp);
        assert!((w_before_x - w_after_x).abs() < 1e-6);
        assert!((w_before_y - w_after_y).abs() < 1e-6);
    }

    #[test]
    fn test_smart_align_drag_snapping_simulation() {
        use glucose_core::smart_align::{snap_move, AlignKind, AlignRect, AlignTarget, SnapOptions};

        let target = AlignTarget {
            id: "card-target".into(),
            kind: AlignKind::Text,
            rect: AlignRect::new(0.0, 0.0, 200.0, 100.0),
        };

        // Glisser un élément de 200x100 à x=203.0 (3px de décalage -> seuil 8px)
        let dragged = AlignRect::new(203.0, 0.0, 200.0, 100.0);
        let snap = snap_move(dragged, &[target], SnapOptions::default());

        // L'aimantation comble le décalage de -3px et affiche le guide vertical à 200.0
        assert!((snap.dx - (-3.0)).abs() < 1e-6);
        assert_eq!(snap.guides.x, Some(vec![200.0]));
    }
}
