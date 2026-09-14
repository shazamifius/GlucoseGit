//! La grille infinie : des points, et rien d'autre.
//!
//! Elle ne connaît que le viewport. Aucune annotation, aucune image, aucun thème : une
//! formule et un tampon. C'est la partie de la scène qui se raisonne entièrement seule, et
//! celle dont l'optimisation — seize masques de couverture plutôt que huit mille cercles de
//! Bézier par image — n'a rien à dire au reste du dessin.

use crate::canvas::{screen_to_world, world_to_screen};
use glucose_core::types::Viewport;
use tiny_skia::PixmapMut;

/// Échelle de viewport en dessous de laquelle la grille n'est plus dessinable.
const MIN_GRID_SCALE: f64 = 1e-6;
/// Nombre maximal de doublements du pas de grille (borne la boucle d'adaptation).
const MAX_GRID_DOUBLINGS: u32 = 64;
/// Pas nominal de la grille, en unités monde (fiche 06 § 3 : 60 px monde).
const GRID_STEP: f64 = 60.0;
/// Espacement minimal des points à l'écran, en pixels : en deçà, le pas double.
///
/// **Écart assumé avec la fiche 06, lié au rastériseur.** La référence dessine le pas
/// nominal jusqu'à l'extinction, soit jusqu'à 73 000 points par frame en 1440 × 900 à
/// l'échelle 0,07 — sur GPU. En CPU, ce doublement borne le nombre de points ; il disparaît
/// avec le rendu GPU (plan de marche RQ-2).
const GRID_MIN_SCREEN_STEP: f64 = 32.0;
/// Échelle en dessous de laquelle la grille s'éteint (fiche 06 § 3.5).
const GRID_FADE_OUT_SCALE: f64 = 0.07;
/// Gris des points de grille, `rgb(136, 136, 136)` (fiche 06 § 3.6).
const GRID_DOT_GREY: u8 = 136;

/// Rayon et opacité d'un point de grille à l'échelle `scale` (fiche 06 § 3.2, § 3.3, § 3.5) :
///
/// ```text
///     R = clamp(1.2 × scale, 0.5, 2.5)        α = clamp(0.3 × scale, 0.08, 0.45)
/// ```
///
/// `None` sous l'échelle d'extinction. Pure : c'est ce que les tests tiennent.
pub(in crate::renderer) fn grid_dot(scale: f64) -> Option<(f32, f32)> {
    if !scale.is_finite() || scale < GRID_FADE_OUT_SCALE {
        return None;
    }
    let radius = (1.2 * scale).clamp(0.5, 2.5) as f32;
    let alpha = (0.3 * scale).clamp(0.08, 0.45) as f32;
    Some((radius, alpha))
}
// ── Grille ──────────────────────────────────────────────────────────────────

pub(in crate::renderer) fn draw_grid(
    pixmap: &mut PixmapMut,
    vp: &Viewport,
    w: u32,
    h: u32,
    header_h: f32,
) {
    // Une échelle nulle, négative ou NaN rendait la boucle d'adaptation du pas
    // infinie : on la borne inconditionnellement. Et sous l'échelle d'extinction, il n'y a
    // rien à dessiner.
    if !vp.scale.is_finite() || vp.scale <= MIN_GRID_SCALE {
        return;
    }
    let Some((dot_radius, dot_alpha)) = grid_dot(vp.scale) else {
        return;
    };

    let (min_wx, min_wy) = screen_to_world(0.0, header_h as f64, vp);
    let (max_wx, max_wy) = screen_to_world(w as f64, h as f64, vp);
    if ![min_wx, min_wy, max_wx, max_wy]
        .iter()
        .all(|v| v.is_finite())
    {
        return;
    }

    // Pas dynamique adaptatif : ne descend jamais sous ~32 px à l'écran pour éviter toute
    // explosion CPU. C'est un choix de densité, pas une borne sur une longueur du monde.
    let mut step = GRID_STEP;
    let mut doublings = 0u32;
    while step * vp.scale < GRID_MIN_SCREEN_STEP && doublings < MAX_GRID_DOUBLINGS {
        step *= 2.0;
        doublings += 1;
    }

    let start_x = (min_wx / step).floor() * step;
    let end_x = (max_wx / step).ceil() * step;
    let start_y = (min_wy / step).floor() * step;
    let end_y = (max_wy / step).ceil() * step;

    let tampons = DotStamps::new(dot_radius, dot_alpha);
    let mut gx = start_x;
    while gx <= end_x {
        let mut gy = start_y;
        while gy <= end_y {
            let (sx, sy) = world_to_screen(gx, gy, vp);
            if sy >= header_h as f64 && sx >= 0.0 && sx <= w as f64 && sy <= h as f64 {
                tampons.stamp(pixmap, sx as f32, sy as f32);
            }
            gy += step;
        }
        gx += step;
    }
}

/// Le nombre de phases sous-pixel d'un point de grille, par axe.
///
/// Un point ne tombe presque jamais sur un pixel entier : sa position dépend du pan, qui est
/// continu. Quatre phases par axe placent le point au quart de pixel près — l'écart résiduel est
/// d'un huitième de pixel, très en dessous de ce qu'un œil distingue sur un point de deux pixels
/// de large, et c'est déjà la quantification retenue pour les glyphes (GLYPH-1).
const DOT_PHASES: usize = 4;

/// Les seize versions d'un point de grille, une par phase sous-pixel.
///
/// # Pourquoi ceci existe
///
/// La grille construisait **un cercle de Bézier par point** — environ huit mille en 4K — puis les
/// rastérisait d'un seul trait. Le banc a chiffré ce que cela coûte : **4,35 ms en 4K**, plus que
/// tout le contenu du document réuni, pour dessiner un fond de points.
///
/// Or tous les points sont identiques : même rayon, même couleur. Seule leur position varie, et
/// seulement d'une fraction de pixel. Il suffit donc de calculer la couverture **une fois par
/// phase** — seize petits masques, soit quelques centaines de valeurs par image — et de la
/// recopier.
///
/// La couverture est analytique : un pixel à distance `d` du centre est couvert à
/// `clamp(r + 0,5 − d, 0, 1)`. C'est l'approximation d'aire d'un disque par sa distance signée,
/// exacte au centre et sur les bords, et indiscernable d'une rastérisation complète sur un disque
/// de deux pixels.
struct DotStamps {
    /// `DOT_PHASES × DOT_PHASES` masques de `side × side` octets de couverture.
    masks: Vec<u8>,
    side: usize,
    /// Décalage du coin du masque par rapport au centre du point, en pixels entiers.
    offset: i32,
    alpha: f32,
}

impl DotStamps {
    fn new(radius: f32, alpha: f32) -> Self {
        let side = (radius * 2.0).ceil() as usize + 2;
        let offset = -((side as i32) / 2);
        let mut masks = vec![0u8; DOT_PHASES * DOT_PHASES * side * side];

        for py in 0..DOT_PHASES {
            for px in 0..DOT_PHASES {
                let (fx, fy) = (px as f32 / DOT_PHASES as f32, py as f32 / DOT_PHASES as f32);
                // Le centre du point dans le repère du masque.
                let (cx, cy) = (-offset as f32 + fx, -offset as f32 + fy);
                let base = (py * DOT_PHASES + px) * side * side;
                for y in 0..side {
                    for x in 0..side {
                        let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                        let d = (dx * dx + dy * dy).sqrt();
                        let couverture = (radius + 0.5 - d).clamp(0.0, 1.0);
                        masks[base + y * side + x] = (couverture * 255.0).round() as u8;
                    }
                }
            }
        }
        Self {
            masks,
            side,
            offset,
            alpha,
        }
    }

    /// Pose un point centré en `(x, y)`, à la phase la plus proche.
    fn stamp(&self, pixmap: &mut PixmapMut, x: f32, y: f32) {
        let (ix, iy) = (x.floor(), y.floor());
        let px = ((x - ix) * DOT_PHASES as f32) as usize % DOT_PHASES;
        let py = ((y - iy) * DOT_PHASES as f32) as usize % DOT_PHASES;
        let base = (py * DOT_PHASES + px) * self.side * self.side;

        let (w, h) = (pixmap.width() as i32, pixmap.height() as i32);
        let (x0, y0) = (ix as i32 + self.offset, iy as i32 + self.offset);
        let gris = GRID_DOT_GREY as u32;
        let data = pixmap.data_mut();

        for row in 0..self.side {
            let sy = y0 + row as i32;
            if sy < 0 || sy >= h {
                continue;
            }
            for col in 0..self.side {
                let sx = x0 + col as i32;
                if sx < 0 || sx >= w {
                    continue;
                }
                let couverture = self.masks[base + row * self.side + col] as f32;
                if couverture == 0.0 {
                    continue;
                }
                let a = (couverture * self.alpha) as u32;
                if a == 0 {
                    continue;
                }
                let i = ((sy * w + sx) * 4) as usize;
                for k in 0..3 {
                    let fond = data[i + k] as u32;
                    data[i + k] = ((gris * a + fond * (255 - a)) / 255) as u8;
                }
                let fond_a = data[i + 3] as u32;
                data[i + 3] = (a + fond_a * (255 - a) / 255) as u8;
            }
        }
    }
}

#[cfg(test)]
mod terminaison {
    use super::*;
    use tiny_skia::Pixmap;

    #[test]
    fn test_draw_grid_terminates_on_degenerate_scale() {
        let mut pixmap = Pixmap::new(320, 240).expect("pixmap 320x240");

        for bad_scale in [0.0_f64, -1.0, f64::NAN, f64::INFINITY, 1e-12] {
            let vp = Viewport {
                scale: bad_scale,
                ..Default::default()
            };
            let started = std::time::Instant::now();
            let mut view = pixmap.as_mut();
            draw_grid(&mut view, &vp, 320, 240, 40.0);
            assert!(
                started.elapsed().as_millis() < 500,
                "draw_grid ne se termine pas pour scale={bad_scale}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::grid_dot;

    /// Fiche 06 § 3 — rayon `clamp(1.2 × s, 0.5, 2.5)`, opacité `clamp(0.3 × s, 0.08, 0.45)`,
    /// extinction sous 0,07. Pure, donc tenue sans un seul pixel.
    #[test]
    fn test_the_grid_dot_follows_the_formula_of_the_spec() {
        assert_eq!(grid_dot(1.0), Some((1.2, 0.3)));
        assert_eq!(
            grid_dot(2.0),
            Some((2.4, 0.45)),
            "l'opacité plafonne à 0,45"
        );
        assert_eq!(grid_dot(4.0), Some((2.5, 0.45)), "le rayon plafonne à 2,5");
        assert_eq!(
            grid_dot(0.2),
            Some((0.5, 0.08)),
            "planchers : 0,5 px et 0,08"
        );
        assert_eq!(
            grid_dot(0.07),
            Some((0.5, 0.08)),
            "à 0,07 la grille est encore là"
        );
        assert_eq!(grid_dot(0.069), None, "en dessous, elle s'éteint");
        assert_eq!(grid_dot(f64::NAN), None);
    }
}

#[cfg(test)]
mod dot_stamp_tests {
    use super::{DotStamps, DOT_PHASES};
    use tiny_skia::Pixmap;

    /// **L'aire d'un tampon est celle d'un disque, surestimée d'autant moins que le disque est
    /// grand.** La couverture est analytique — la distance signée au bord — et cette formule a
    /// deux propriétés que ce test fixe :
    ///
    /// * elle **surestime toujours** : un point n'est jamais plus pâle qu'un vrai disque, donc
    ///   jamais moins visible ;
    /// * l'erreur relative **décroît avec le rayon** : 12 % à un pixel, moins de 6 % à deux et
    ///   demi. Un point de grille au zoom 1 fait 1,2 px de rayon.
    ///
    /// Ces chiffres sont mesurés, pas choisis, et le test les tient pour qu'ils ne bougent pas
    /// en silence.
    #[test]
    fn test_l_aire_d_un_tampon_est_celle_d_un_disque() {
        let erreur_moyenne = |radius: f32| -> f32 {
            let t = DotStamps::new(radius, 1.0);
            let par_phase = t.side * t.side;
            let disque = std::f32::consts::PI * radius * radius;
            let mut total = 0.0;
            for phase in 0..DOT_PHASES * DOT_PHASES {
                let somme: f32 = t.masks[phase * par_phase..(phase + 1) * par_phase]
                    .iter()
                    .map(|&c| c as f32 / 255.0)
                    .sum();
                assert!(
                    somme >= disque * 0.99,
                    "rayon {radius}, phase {phase} : sous-estimé ({somme} < {disque})"
                );
                total += (somme - disque) / disque;
            }
            total / (DOT_PHASES * DOT_PHASES) as f32
        };

        let (e1, e12, e25) = (
            erreur_moyenne(1.0),
            erreur_moyenne(1.2),
            erreur_moyenne(2.5),
        );
        assert!(e1 < 0.13, "à 1 px : {e1}");
        assert!(e12 < 0.11, "à 1,2 px : {e12}");
        assert!(e25 < 0.06, "à 2,5 px : {e25}");
        assert!(
            e1 > e12 && e12 > e25,
            "l'erreur décroît avec le rayon : {e1} > {e12} > {e25}"
        );
    }

    /// Sous un pixel de rayon, le tampon couvre un peu plus qu'un disque, et pas plus d'un pixel
    /// et demi : la borne est fixée ici pour que l'approximation ne dérive pas sans qu'on le voie.
    #[test]
    fn test_un_point_sous_le_pixel_reste_visible_sans_baver() {
        let t = DotStamps::new(0.5, 1.0);
        let par_phase = t.side * t.side;
        let somme: f32 = t.masks[..par_phase].iter().map(|&c| c as f32 / 255.0).sum();
        assert!(
            somme > 0.785,
            "un point de 0,5 px ne doit pas disparaître : {somme}"
        );
        assert!(
            somme < 1.5,
            "et ne doit pas baver sur ses voisins : {somme}"
        );
    }

    /// Les seize phases ne sont pas la même image : sinon la quantification sous-pixel ne
    /// servirait à rien, et les points sauteraient d'un pixel entier pendant un déplacement.
    #[test]
    fn test_les_phases_different_vraiment() {
        let t = DotStamps::new(1.2, 1.0);
        let par_phase = t.side * t.side;
        let distinctes: std::collections::HashSet<&[u8]> = (0..DOT_PHASES * DOT_PHASES)
            .map(|p| &t.masks[p * par_phase..(p + 1) * par_phase])
            .collect();
        assert_eq!(
            distinctes.len(),
            DOT_PHASES * DOT_PHASES,
            "des phases sont confondues"
        );
    }

    /// Le centre d'un tampon est couvert en plein, et ses coins ne le sont pas du tout.
    #[test]
    fn test_le_centre_est_plein_et_les_coins_sont_vides() {
        let t = DotStamps::new(1.5, 1.0);
        let centre = t.side / 2;
        assert_eq!(t.masks[centre * t.side + centre], 255, "le centre");
        assert_eq!(t.masks[0], 0, "le coin haut gauche");
        assert_eq!(t.masks[t.side * t.side - 1], 0, "le coin bas droit");
    }

    /// Tamponner hors du pixmap ne déborde pas, et tamponner dedans met de l'encre.
    #[test]
    fn test_tamponner_encre_dedans_et_ne_deborde_pas_dehors() {
        let t = DotStamps::new(1.2, 0.5);
        let mut p = Pixmap::new(20, 20).expect("pixmap");
        for (x, y) in [(-5.0, 10.0), (25.0, 10.0), (10.0, -5.0), (10.0, 25.0)] {
            t.stamp(&mut p.as_mut(), x, y);
        }
        assert_eq!(
            p.pixels().iter().filter(|px| px.alpha() > 0).count(),
            0,
            "rien dehors"
        );

        t.stamp(&mut p.as_mut(), 10.3, 9.7);
        let encre = p.pixels().iter().filter(|px| px.alpha() > 0).count();
        assert!(
            (4..=16).contains(&encre),
            "un point de rayon 1,2 touche quelques pixels : {encre}"
        );
    }

    /// Deux points à la même phase donnent exactement les mêmes pixels — c'est ce qui rend la
    /// grille reproductible d'une image à l'autre.
    #[test]
    fn test_deux_points_a_la_meme_phase_sont_identiques() {
        let t = DotStamps::new(1.2, 0.4);
        let mut a = Pixmap::new(12, 12).expect("pixmap");
        let mut b = Pixmap::new(12, 12).expect("pixmap");
        t.stamp(&mut a.as_mut(), 5.25, 5.75);
        t.stamp(&mut b.as_mut(), 5.25, 5.75);
        assert_eq!(a.data(), b.data());
    }
}
