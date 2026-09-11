//! Les poignées de redimensionnement — dessinées **là où `hit_priority` les cherche**.
//!
//! Une poignée est une affordance : elle garde une taille écran constante (exception SCALE-1,
//! `WorldScale::screen`) et se pose aux positions que [`Handle::position_on`] donne pour la
//! boîte écran du nœud. C'est la même fonction que le noyau utilise pour le test de clic, sur
//! la même boîte : une poignée dessinée est donc une poignée cliquable, par construction
//! (loi L4, appliquée au canevas).

use super::scale::WorldScale;
use crate::theme::Theme;
use glucose_core::resize::Handle;
use glucose_core::smart_align::AlignRect;
use tiny_skia::{Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Côté d'une poignée, en pixels écran.
const HANDLE_SIDE: f32 = 8.0;
/// Épaisseur du liseré d'une poignée, en pixels écran.
const HANDLE_OUTLINE: f32 = 1.0;

/// Dessine `handles` sur la boîte écran `(x, y, w, h)` d'un nœud sélectionné.
pub(super) fn draw_resize_handles(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    scale: WorldScale,
    screen_box: (f32, f32, f32, f32),
    handles: &[Handle],
) {
    let (x, y, w, h) = screen_box;
    let rect = AlignRect::new(x as f64, y as f64, w as f64, h as f64);
    let side = scale.screen(HANDLE_SIDE);
    let outline = scale.screen(HANDLE_OUTLINE);

    let mut fill = Paint { anti_alias: true, ..Default::default() };
    fill.set_color(theme.handle_fill);
    let mut border = Paint { anti_alias: true, ..Default::default() };
    border.set_color(theme.handle_outline);
    let stroke = Stroke { width: outline, ..Default::default() };

    for handle in handles {
        let (cx, cy) = handle.position_on(rect);
        let Some(square) = Rect::from_xywh(cx as f32 - side / 2.0, cy as f32 - side / 2.0, side, side) else {
            continue;
        };
        pixmap.fill_rect(square, &fill, Transform::identity(), None);
        pixmap.stroke_path(&PathBuilder::from_rect(square), &border, &stroke, Transform::identity(), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::{Color, Pixmap};

    fn lit(pixmap: &Pixmap, x: u32, y: u32) -> bool {
        let i = ((y * pixmap.width() + x) * 4) as usize;
        pixmap.data()[i] > 200
    }

    #[test]
    fn test_handles_are_drawn_where_hit_priority_looks_for_them() {
        let mut pixmap = Pixmap::new(300, 200).expect("pixmap");
        pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
        let theme = Theme::dark();
        let screen_box = (50.0, 40.0, 200.0, 100.0);
        draw_resize_handles(&mut pixmap.as_mut(), &theme, WorldScale::new(1.0), screen_box, &Handle::ALL);

        let rect = AlignRect::new(50.0, 40.0, 200.0, 100.0);
        for handle in Handle::ALL {
            let (cx, cy) = handle.position_on(rect);
            assert!(lit(&pixmap, cx as u32, cy as u32), "{} sans encre en ({cx}, {cy})", handle.as_str());
        }
        // Le centre du nœud, lui, reste vierge : les poignées sont sur le bord.
        assert!(!lit(&pixmap, 150, 90));
    }

    #[test]
    fn test_scale_1_a_handle_keeps_its_screen_size_at_every_zoom() {
        let theme = Theme::dark();
        let mut widths = Vec::new();
        for zoom in [0.25_f64, 1.0, 4.0] {
            let mut pixmap = Pixmap::new(120, 120).expect("pixmap");
            pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
            draw_resize_handles(&mut pixmap.as_mut(), &theme, WorldScale::new(zoom), (60.0, 60.0, 40.0, 40.0), &[Handle::TopLeft]);
            let lit_on_row = (0..120).filter(|&x| lit(&pixmap, x, 60)).count();
            widths.push(lit_on_row);
        }
        assert!(widths.iter().all(|&w| w == widths[0]), "{widths:?}");
        assert!(widths[0] >= 6, "{widths:?}");
    }
}
