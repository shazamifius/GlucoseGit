//! Les poignées de redimensionnement — dessinées **là où `hit_priority` les cherche**.
//!
//! Une poignée est une affordance : elle garde une taille écran constante (exception SCALE-1,
//! `WorldScale::screen`) et se pose aux positions que [`Handle::position_on`] donne pour la
//! boîte écran du nœud. C'est la même fonction que le noyau utilise pour le test de clic, sur
//! la même boîte : une poignée dessinée est donc une poignée cliquable, par construction
//! (loi L4, appliquée au canevas).

use super::scale::{fill_crisp, WorldScale};
use crate::theme::Theme;
use glucose_core::resize::Handle;
use glucose_core::smart_align::AlignRect;
use tiny_skia::{Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Côté d'une poignée, en pixels écran (fiche 06 § 4.2 : carré de 9 px).
pub(super) const HANDLE_SIDE: f32 = 9.0;
/// Épaisseur du liseré d'une poignée, en pixels écran (fiche 06 § 4.2 : 1,25 px).
pub(super) const HANDLE_OUTLINE: f32 = 1.25;

/// Dessine `handles` sur la boîte écran `(x, y, w, h)` d'un nœud sélectionné.
pub(super) fn draw_resize_handles(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    scale: WorldScale,
    screen_box: (f32, f32, f32, f32),
    handles: &[Handle],
) {
    draw_rotated_handles(pixmap, theme, scale, screen_box, handles, 0.0);
}

/// Les mêmes, sur un nœud tourné de `rotation` radians.
///
/// Les poignées **suivent** le nœud : elles tournent avec lui, aux positions exactes que
/// l'arbitre de clic interroge ([`glucose_core::rotate::place`], la même formule des deux
/// côtés). Les carrés, eux, restent droits — une poignée est une prise, pas un ornement, et
/// un carré incliné se vise moins bien.
pub(super) fn draw_rotated_handles(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    scale: WorldScale,
    screen_box: (f32, f32, f32, f32),
    handles: &[Handle],
    rotation: f64,
) {
    let (x, y, w, h) = screen_box;
    let centre = ((x + w / 2.0) as f64, (y + h / 2.0) as f64);
    let local = AlignRect::new(-(w as f64) / 2.0, -(h as f64) / 2.0, w as f64, h as f64);
    let side = scale.screen(HANDLE_SIDE);
    let outline = scale.screen(HANDLE_OUTLINE);
    for handle in handles {
        let (cx, cy) = glucose_core::rotate::place(centre, handle.position_on(local), rotation);
        poser_une_poignee(
            pixmap,
            (cx as f32, cy as f32),
            (side, outline),
            (theme.handle_fill, theme.handle_outline),
        );
    }
}

/// **Une poignée, nette** : un carré de liseré, et un carré de fond dedans, tous deux posés
/// sur la grille de pixels (SCALE-3).
///
/// Elle était un carré plein puis un contour, anti-crénelés : deux passages dans le
/// rastériseur pour une forme alignée sur les axes, à qui l'anti-crénelage n'apporte rien —
/// il la rendait même floue sur une demi-position. Et c'étaient seize appels par photo
/// sélectionnée : `bench_ornements` les chiffre à 12,5 ms pour 243 photos, la moitié du poste
/// qui faisait tomber le tempo à 48 images par seconde sur la longue session du 23/09.
///
/// La géométrie est celle du contour d'avant, centré sur le bord du carré : le liseré déborde
/// d'une demi-épaisseur au-dehors et mord d'autant au-dedans.
fn poser_une_poignee(
    pixmap: &mut PixmapMut,
    (cx, cy): (f32, f32),
    (side, outline): (f32, f32),
    (fond, liseré): (tiny_skia::Color, tiny_skia::Color),
) {
    let exterieur = side + outline;
    let Some(bord) = Rect::from_xywh(
        (cx - exterieur / 2.0).round(),
        (cy - exterieur / 2.0).round(),
        exterieur.round(),
        exterieur.round(),
    ) else {
        return;
    };
    fill_crisp(pixmap, bord, liseré);
    // Le fond se retire du bord d'une épaisseur entière de chaque côté : sur la grille, un
    // liseré a un nombre entier de pixels, et jamais moins d'un.
    let e = outline.round().max(1.0);
    if let Some(dedans) = Rect::from_xywh(
        bord.x() + e,
        bord.y() + e,
        bord.width() - 2.0 * e,
        bord.height() - 2.0 * e,
    ) {
        fill_crisp(pixmap, dedans, fond);
    }
}

/// Rayon d'une poignée de coude, en pixels écran (`glucose_core::arrow::HANDLE_RADIUS_PX`).
///
/// Sa valeur vient du noyau, celui-là même que l'arbitre de clic interroge : une poignée
/// dessinée est une poignée cliquable, par construction (loi L4).
///
/// `poignees` est la **même** liste que `begin_arrow_bend` interroge
/// ([`glucose_core::arrow::handles`]) : impossible de dessiner une poignée là où le clic n'en
/// trouvera pas.
pub(super) fn draw_arrow_handles(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    poignees: &[glucose_core::arrow::ArrowHandle],
    vp: &glucose_core::types::Viewport,
) {
    use glucose_core::arrow::{HandleKind, HANDLE_RADIUS_PX};

    let rayon = HANDLE_RADIUS_PX as f32;
    for handle in poignees {
        let (sx, sy) = crate::canvas::world_to_screen(handle.at.0, handle.at.1, vp);
        let (cx, cy) = (sx as f32, sy as f32);
        let (chemin, fond) = match handle.kind {
            // Un coude : un disque plein, de la couleur qui le désigne.
            HandleKind::Bend(_) => {
                let mut b = PathBuilder::new();
                b.push_circle(cx, cy, rayon);
                (b.finish(), theme.arrow_bend)
            }
            // Un milieu : un losange, plus discret — il n'existe pas encore, il s'offre.
            HandleKind::Midpoint(_) => {
                let mut b = PathBuilder::new();
                b.move_to(cx, cy - rayon);
                b.line_to(cx + rayon, cy);
                b.line_to(cx, cy + rayon);
                b.line_to(cx - rayon, cy);
                b.close();
                (b.finish(), theme.handle_fill)
            }
        };
        let Some(chemin) = chemin else { continue };

        let mut paint = Paint {
            anti_alias: true,
            ..Paint::default()
        };
        paint.set_color(fond);
        pixmap.fill_path(
            &chemin,
            &paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );
        paint.set_color(theme.handle_outline);
        pixmap.stroke_path(
            &chemin,
            &paint,
            &Stroke {
                width: HANDLE_OUTLINE,
                ..Stroke::default()
            },
            Transform::identity(),
            None,
        );
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

    /// Fiche 06 § 4.2 — un carré de 9 px à l'écran, liseré de 1,25 px.
    #[test]
    fn test_a_handle_is_a_nine_pixel_square_with_a_hairline_outline() {
        assert_eq!(HANDLE_SIDE, 9.0);
        assert_eq!(HANDLE_OUTLINE, 1.25);
    }

    #[test]
    fn test_handles_are_drawn_where_hit_priority_looks_for_them() {
        let mut pixmap = Pixmap::new(300, 200).expect("pixmap");
        pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
        let theme = Theme::dark();
        let screen_box = (50.0, 40.0, 200.0, 100.0);
        draw_resize_handles(
            &mut pixmap.as_mut(),
            &theme,
            WorldScale::new(1.0),
            screen_box,
            &Handle::ALL,
        );

        let rect = AlignRect::new(50.0, 40.0, 200.0, 100.0);
        for handle in Handle::ALL {
            let (cx, cy) = handle.position_on(rect);
            assert!(
                lit(&pixmap, cx as u32, cy as u32),
                "{} sans encre en ({cx}, {cy})",
                handle.as_str()
            );
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
            draw_resize_handles(
                &mut pixmap.as_mut(),
                &theme,
                WorldScale::new(zoom),
                (60.0, 60.0, 40.0, 40.0),
                &[Handle::TopLeft],
            );
            let lit_on_row = (0..120).filter(|&x| lit(&pixmap, x, 60)).count();
            widths.push(lit_on_row);
        }
        assert!(widths.iter().all(|&w| w == widths[0]), "{widths:?}");
        assert!(widths[0] >= 6, "{widths:?}");
    }
}
