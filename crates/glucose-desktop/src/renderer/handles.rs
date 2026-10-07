//! Les poignées de redimensionnement — dessinées **là où `hit_priority` les cherche**.
//!
//! Une poignée est une affordance : sa taille se compte en pixels écran (exception SCALE-1,
//! `WorldScale::screen`), et suit la place que le nœud lui laisse à l'écran (POIGNEE-1,
//! [`glucose_core::hit_priority::handle_side_px`] — la loi que le clic lit aussi). Elle se pose
//! aux positions que [`Handle::position_on`] donne pour la boîte écran du nœud. C'est la même fonction que le noyau utilise pour le test de clic, sur
//! la même boîte : une poignée dessinée est donc une poignée cliquable, par construction
//! (loi L4, appliquée au canevas).

use super::scale::{fill_crisp, WorldScale};
use crate::theme::Theme;
use glucose_core::resize::Handle;
use glucose_core::smart_align::AlignRect;
use tiny_skia::{Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

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
    let Some(cote) = glucose_core::hit_priority::handle_side_px(petit_cote_logique(scale, (w, h)))
    else {
        return;
    };
    let centre = ((x + w / 2.0) as f64, (y + h / 2.0) as f64);
    let local = AlignRect::new(-(w as f64) / 2.0, -(h as f64) / 2.0, w as f64, h as f64);
    let side = scale.screen(cote as f32);
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

/// Le petit côté d'une boîte écran, en pixels **logiques** : la boîte est en pixels
/// physiques, les lois des ornements en pixels logiques (DPI-1).
fn petit_cote_logique(scale: WorldScale, (w, h): (f32, f32)) -> f64 {
    f64::from(w.abs().min(h.abs())) / f64::from(scale.screen(1.0))
}

/// **La part de leur taille que gardent les ornements** d'un nœud posé sur cette boîte écran
/// — cadre et anneau de sélection (POIGNEE-1, [`glucose_core::hit_priority::chrome_ratio`]).
pub(crate) fn part_des_ornements(scale: WorldScale, boite: (f32, f32)) -> f32 {
    glucose_core::hit_priority::chrome_ratio(petit_cote_logique(scale, boite)) as f32
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
    (vp, scale): (&glucose_core::types::Viewport, WorldScale),
) {
    use glucose_core::arrow::{HandleKind, HANDLE_RADIUS_PX};

    let rayon = scale.screen(HANDLE_RADIUS_PX as f32);
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
                width: scale.screen(HANDLE_OUTLINE),
                ..Stroke::default()
            },
            Transform::identity(),
            None,
        );
    }
}

/// **Le cadre du groupe et ses quatre poignées** (fiche 53 § 10) : quand la sélection compte
/// deux nœuds ou plus, un fil autour de leur emprise — la même boîte que l'arbitre de clic lit
/// ([`glucose_core::hit_priority::emprise_du_groupe`]) —, et ses coins pour la tirer ou, `Alt`
/// tenu, la faire tourner. Chaque nœud garde son propre cadre, sans poignée.
pub(super) fn dessiner_le_cadre_du_groupe(
    theme: &Theme,
    pixmap: &mut PixmapMut,
    store: &glucose_core::store::Store,
    pass: crate::params::ViewPass<'_>,
) {
    let Some(groupe) = store.emprise_du_groupe(&store.project.active_board_id) else {
        return;
    };
    let (x0, y0) = crate::canvas::world_to_screen(groupe.left, groupe.top, &pass.vp);
    let (x1, y1) = crate::canvas::world_to_screen(
        groupe.left + groupe.width,
        groupe.top + groupe.height,
        &pass.vp,
    );
    let ecran = (x0 as f32, y0 as f32, (x1 - x0) as f32, (y1 - y0) as f32);
    let scale = pass.echelle();
    if let Some(r) = Rect::from_xywh(ecran.0, ecran.1, ecran.2, ecran.3) {
        let mut paint = Paint {
            anti_alias: true,
            ..Paint::default()
        };
        paint.set_color(theme.selection_frame);
        let trait_ = Stroke {
            width: scale.screen(1.0),
            ..Stroke::default()
        };
        pixmap.stroke_path(
            &PathBuilder::from_rect(r),
            &paint,
            &trait_,
            Transform::identity(),
            None,
        );
    }
    draw_resize_handles(
        pixmap,
        theme,
        scale,
        ecran,
        &glucose_core::hit_priority::POIGNEES_DU_GROUPE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::{Color, Pixmap};

    fn lit(pixmap: &Pixmap, x: u32, y: u32) -> bool {
        let i = ((y * pixmap.width() + x) * 4) as usize;
        pixmap.data()[i] > 200
    }

    /// Fiche 06 § 4.2 — un carré de 9 px à l'écran, liseré de 1,25 px, sur un nœud qui a
    /// toute la place.
    #[test]
    fn test_a_handle_is_a_nine_pixel_square_with_a_hairline_outline() {
        assert_eq!(glucose_core::hit_priority::handle_side_px(400.0), Some(9.0));
        assert_eq!(HANDLE_OUTLINE, 1.25);
    }

    /// Les pixels allumés sur la rangée `y`.
    fn allumes(pixmap: &Pixmap, y: u32) -> usize {
        (0..pixmap.width()).filter(|&x| lit(pixmap, x, y)).count()
    }

    /// **POIGNEE-1 — la poignée suit la place que le nœud lui laisse à l'écran** : pleine sur un
    /// grand nœud, plus petite sur un petit, absente sur une vignette — où elle était un carré
    /// de 9 px plus gros que la photo (son retour du 07/10).
    #[test]
    fn test_poignee_1_le_carre_suit_le_noeud_et_disparait_sur_une_vignette() {
        let theme = Theme::dark();
        let coin = |cote: f32| {
            let mut pixmap = Pixmap::new(400, 400).expect("pixmap");
            pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
            draw_resize_handles(
                &mut pixmap.as_mut(),
                &theme,
                WorldScale::new(1.0, 1.0),
                (100.0, 100.0, cote, cote),
                &[Handle::TopLeft],
            );
            allumes(&pixmap, 100)
        };
        let (grand, moyen, vignette) = (coin(200.0), coin(30.0), coin(16.0));
        // Le blanc seul s'allume : 8 px dans un carre de 9 borde de son liseré sombre.
        assert!(grand >= 8, "un grand noeud : {grand} px");
        assert!(
            moyen < grand && moyen > 0,
            "un noeud de 30 px : {moyen} contre {grand}"
        );
        assert_eq!(
            vignette, 0,
            "une vignette de 16 px porte encore une poignee"
        );
    }

    /// La loi se lit en pixels **logiques** : à 150 %, le même nœud logique porte la même
    /// poignée, une fois et demie plus grande en pixels physiques (DPI-1).
    #[test]
    fn test_poignee_1_la_densite_ne_change_pas_la_decision() {
        let theme = Theme::dark();
        let coin = |densite: f32| {
            let mut pixmap = Pixmap::new(400, 400).expect("pixmap");
            pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
            draw_resize_handles(
                &mut pixmap.as_mut(),
                &theme,
                WorldScale::new(1.0, densite),
                (100.0, 100.0, 20.0 * densite, 20.0 * densite),
                &[Handle::TopLeft],
            );
            allumes(&pixmap, 100)
        };
        // 20 px logiques : prise de 7 px, carré de 2,6 px — moins qu'un carré.
        assert_eq!((coin(1.0), coin(1.5), coin(2.0)), (0, 0, 0));
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
            WorldScale::new(1.0, 1.0),
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

    /// Le zoom seul ne change pas la poignée : c'est la place du nœud **à l'écran** qui compte.
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
                WorldScale::new(zoom, 1.0),
                (60.0, 60.0, 80.0, 80.0),
                &[Handle::TopLeft],
            );
            let lit_on_row = (0..120).filter(|&x| lit(&pixmap, x, 60)).count();
            widths.push(lit_on_row);
        }
        assert!(widths.iter().all(|&w| w == widths[0]), "{widths:?}");
        assert!(widths[0] >= 6, "{widths:?}");
    }
}
