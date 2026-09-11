//! Halos symbiotiques d'ambiance : composition directe par anneaux d'opacité.
//!
//! # HALO-1 — un dégradé radial ne contient que 35 valeurs distinctes
//!
//! Le halo d'une carte est un dégradé radial qui part de [`HALO_CENTER_ALPHA`] au
//! centre et s'éteint linéairement au bord. Son opacité ne prend donc que
//! `HALO_CENTER_ALPHA` valeurs entières une fois écrite sur 8 bits : le disque se
//! décompose exactement en autant d'**anneaux de couleur constante**.
//!
//! L'implémentation d'origine ignorait cette structure. Elle reconstruisait un
//! `RadialGradient` tiny-skia à chaque frame et **par carte**, ce qui imposait une
//! racine carrée, une interpolation de butées et une conversion de couleur sur chaque
//! pixel du disque — environ 500 000 pixels pour une carte de taille par défaut.
//! Le poste `halos` était, avec les docks, le plus cher du rendu.
//!
//! Ici on parcourt les anneaux : pour chacun, la couleur source est **constante** et
//! précalculée une fois, et la boucle interne se réduit à une composition
//! « source-over » prémultipliée sur des entiers.
//!
//! # HALO-2 — pourquoi il n'y a *aucun* cache ici
//!
//! Un cache de halos pré-rastérisés est tentant, mais il achèterait de la mémoire — et
//! une borne à tenir, et une invalidation — pour supprimer un travail qui, décomposé en
//! anneaux, ne représente plus que `HALO_CENTER_ALPHA` multiplications par carte.
//! Le seul état conservé d'une frame à l'autre reste donc la teinte symbiotique, dans
//! [`SymbioticHueCache`] : elle, contrairement au profil du halo, dépend du voisinage et
//! coûte cher à recalculer.

use super::SymbioticHueCache;
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use tiny_skia::{PixmapMut, PremultipliedColorU8};

/// Opacité du halo en son centre, sur 255. Elle décroît linéairement jusqu'au bord.
const HALO_CENTER_ALPHA: u8 = 35;

/// Largeur par défaut d'une carte de texte, reprise du modèle.
pub(super) const DEFAULT_TEXT_CARD_WIDTH: f64 = 240.0;
/// Hauteur par défaut d'une carte de texte, reprise du modèle.
pub(super) const DEFAULT_TEXT_CARD_HEIGHT: f64 = 48.0;

/// Facteur appliqué à la plus grande dimension de la carte pour obtenir le rayon.
const HALO_RADIUS_FACTOR: f64 = 1.5;
/// Marge constante, en unités monde, ajoutée au rayon du halo.
const HALO_RADIUS_MARGIN: f64 = 50.0;
/// Rayon minimal du halo, en unités monde.
const HALO_MIN_RADIUS: f64 = 20.0;
/// En dessous de ce rayon écran, le halo ne couvre plus assez de pixels pour être vu.
const HALO_CULL_RADIUS: f32 = 4.0;

/// Divise par 255 avec arrondi au plus proche, sans division entière.
///
/// Exact pour tout produit de deux octets : c'est l'identité `(x + 128 + (x + 128) / 256) / 256`
/// qu'utilisent les compositeurs 8 bits pour rester au niveau près de `x / 255.0`.
fn div255(value: u32) -> u32 {
    let biased = value + 128;
    (biased + (biased >> 8)) >> 8
}

/// Terme source d'un anneau : la couleur du halo déjà multipliée par son opacité.
#[derive(Clone, Copy)]
struct RingSource {
    /// Canaux `r`, `g`, `b` et alpha, chacun multiplié par `alpha`. Somme à diviser par 255.
    scaled: [u32; 4],
    /// `255 - alpha`, le poids qui reste à la destination.
    inv_alpha: u32,
}

impl RingSource {
    fn new((r, g, b): (u8, u8, u8), alpha: u8) -> Self {
        let weight = alpha as u32;
        Self {
            scaled: [
                r as u32 * weight,
                g as u32 * weight,
                b as u32 * weight,
                255 * weight,
            ],
            inv_alpha: 255 - weight,
        }
    }
}

/// Compose une couleur constante sur une tranche de pixels (boucle chaude du halo).
///
/// La composition « source-over » prémultipliée s'écrit `(c * a + d * (255 - a)) / 255`
/// pour chaque canal. L'écrire ainsi — plutôt qu'en prémultipliant d'abord la source —
/// n'arrondit **qu'une fois**, comme le fait le pipeline flottant de tiny-skia : le
/// résultat reste donc à un niveau près du dégradé d'origine.
fn blend_span(row: &mut [PremultipliedColorU8], source: RingSource) {
    let [sr, sg, sb, sa] = source.scaled;
    let inv_alpha = source.inv_alpha;

    for pixel in row {
        let r = div255(sr + pixel.red() as u32 * inv_alpha);
        let g = div255(sg + pixel.green() as u32 * inv_alpha);
        let b = div255(sb + pixel.blue() as u32 * inv_alpha);
        let a = div255(sa + pixel.alpha() as u32 * inv_alpha);
        // `r`, `g` et `b` restent ≤ `a` : la source est valide (canal ≤ 255) et la
        // destination l'est aussi (canal ≤ alpha). `from_rgba` ne peut pas rendre `None`.
        if let Some(blended) = PremultipliedColorU8::from_rgba(r as u8, g as u8, b as u8, a as u8) {
            *pixel = blended;
        }
    }
}

/// Première rangée (ou colonne) dont le centre de pixel atteint `position`, bornée à l'écran.
fn first_pixel_at_or_after(position: f32, limit: i32) -> i32 {
    if !position.is_finite() {
        return if position.is_sign_negative() { 0 } else { limit };
    }
    let index = (position - 0.5).ceil();
    if index <= 0.0 {
        0
    } else if index >= limit as f32 {
        limit
    } else {
        index as i32
    }
}

/// Composite un anneau d'opacité constante, compris entre les rayons `inner` et `outer`.
///
/// L'anneau est découpé en tranches horizontales : une seule quand la ligne ne traverse
/// pas le disque intérieur, deux sinon.
fn fill_ring(
    pixels: &mut [PremultipliedColorU8],
    (width, height): (i32, i32),
    (cx, cy): (f32, f32),
    (inner, outer): (f32, f32),
    source: RingSource,
) {
    let outer_sq = outer * outer;
    let inner_sq = inner * inner;

    let y_start = first_pixel_at_or_after(cy - outer, height);
    let y_end = first_pixel_at_or_after(cy + outer, height);

    for y in y_start..y_end {
        let dy = y as f32 + 0.5 - cy;
        let dy_sq = dy * dy;
        let gap_sq = outer_sq - dy_sq;
        if gap_sq <= 0.0 {
            continue;
        }
        let half_outer = gap_sq.sqrt();
        let row_base = (y * width) as usize;

        let spans = if dy_sq < inner_sq {
            let half_inner = (inner_sq - dy_sq).sqrt();
            [
                Some((cx - half_outer, cx - half_inner)),
                Some((cx + half_inner, cx + half_outer)),
            ]
        } else {
            [Some((cx - half_outer, cx + half_outer)), None]
        };

        for (from, to) in spans.into_iter().flatten() {
            let x0 = first_pixel_at_or_after(from, width) as usize;
            let x1 = first_pixel_at_or_after(to, width) as usize;
            if x1 > x0 {
                blend_span(&mut pixels[row_base + x0..row_base + x1], source);
            }
        }
    }
}

/// Compose un halo de centre `(cx, cy)` et de rayon `radius` sur `dst` (HALO-1).
pub fn draw_halo(dst: &mut PixmapMut, cx: f32, cy: f32, radius: f32, rgb: (u8, u8, u8)) {
    if !radius.is_finite() || radius <= 0.0 || !cx.is_finite() || !cy.is_finite() {
        return;
    }
    let width = dst.width() as i32;
    let height = dst.height() as i32;
    let pixels = dst.pixels_mut();

    // Le niveau `alpha` couvre les rayons où `HALO_CENTER_ALPHA * (1 - r / radius)`
    // s'arrondit à `alpha`, soit une demi-marche de part et d'autre.
    let levels = HALO_CENTER_ALPHA as f32;
    for alpha in 1..=HALO_CENTER_ALPHA {
        let outer = radius * ((levels - alpha as f32 + 0.5) / levels);
        let inner = (radius * ((levels - alpha as f32 - 0.5) / levels)).max(0.0);
        fill_ring(
            pixels,
            (width, height),
            (cx, cy),
            (inner, outer),
            RingSource::new(rgb, alpha),
        );
    }
}

/// Géométrie écran du halo d'une carte de texte, ou `None` si elle sort du cadre.
fn halo_geometry(
    ann: &Annotation,
    vp: &Viewport,
    screen_w: f32,
    screen_h: f32,
    header_h: f32,
) -> Option<(f32, f32, f32)> {
    let Annotation::Text {
        x,
        y,
        width,
        height,
        ..
    } = ann
    else {
        return None;
    };

    let (sx, sy) = world_to_screen(*x, *y, vp);
    let w = width.unwrap_or(DEFAULT_TEXT_CARD_WIDTH) * vp.scale;
    let h = height.unwrap_or(DEFAULT_TEXT_CARD_HEIGHT) * vp.scale;

    let cx = (sx + w / 2.0) as f32;
    let cy = (sy + h / 2.0) as f32;
    let radius = ((w.max(h) * HALO_RADIUS_FACTOR) as f32 + (HALO_RADIUS_MARGIN * vp.scale) as f32)
        .max((HALO_MIN_RADIUS * vp.scale) as f32);

    // Frustum culling : hors écran, ou trop microscopique pour couvrir un pixel.
    if cx + radius < 0.0
        || cx - radius > screen_w
        || cy + radius < header_h
        || cy - radius > screen_h
        || radius < HALO_CULL_RADIUS
    {
        return None;
    }

    Some((cx, cy, radius))
}

/// Passe de rendu des halos d'ambiance (L1 : ne parcourt que les cartes visibles).
pub fn draw_halos(
    hue_cache: &mut SymbioticHueCache,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) {
    let ViewPass { visible_ids, header_h, .. } = pass;
    let vp = &pass.vp;
    let Some(board) = store.active_board() else {
        return;
    };

    let screen_w = pixmap.width() as f32;
    let screen_h = pixmap.height() as f32;

    for ann in &board.annotations {
        if !visible_ids.contains(ann.id()) {
            continue;
        }
        let Some((cx, cy, radius)) = halo_geometry(ann, vp, screen_w, screen_h, header_h) else {
            continue;
        };
        let (_hue, rgb) = hue_cache.get_or_compute(ann, &board.annotations);
        draw_halo(pixmap, cx, cy, radius, rgb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tiny_skia::{
        Color, FillRule, GradientStop, Paint, PathBuilder, Pixmap, Point, RadialGradient,
        SpreadMode, Transform,
    };

    /// Fond de référence des tests : la couleur de toile du thème sombre.
    fn background() -> Color {
        Color::from_rgba8(13, 14, 18, 255)
    }

    /// Rend le halo comme avant : un `RadialGradient` reconstruit et un disque rempli.
    fn draw_reference_halo(
        dst: &mut PixmapMut,
        cx: f32,
        cy: f32,
        radius: f32,
        (r, g, b): (u8, u8, u8),
    ) {
        let shader = RadialGradient::new(
            Point::from_xy(cx, cy),
            Point::from_xy(cx, cy),
            radius,
            vec![
                GradientStop::new(0.0, Color::from_rgba8(r, g, b, HALO_CENTER_ALPHA)),
                GradientStop::new(1.0, Color::from_rgba8(r, g, b, 0)),
            ],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .expect("rayon strictement positif : le dégradé ne peut pas être dégénéré");
        let paint = Paint { shader, anti_alias: true, ..Default::default() };

        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, radius);
        let path = pb
            .finish()
            .expect("un cercle de rayon > 0 est toujours un chemin valide");
        dst.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
    }

    fn render(radius: f32, rgb: (u8, u8, u8), reference: bool) -> Pixmap {
        let side = (radius * 2.0).ceil() as u32 + 8;
        let mut pixmap = Pixmap::new(side, side).expect("pixmap de test");
        pixmap.fill(background());
        let center = side as f32 / 2.0;
        let mut view = pixmap.as_mut();
        if reference {
            draw_reference_halo(&mut view, center, center, radius, rgb);
        } else {
            draw_halo(&mut view, center, center, radius, rgb);
        }
        pixmap
    }

    /// Écart maximal, canal par canal, entre deux pixmaps de même taille.
    fn max_channel_delta(a: &Pixmap, b: &Pixmap) -> u8 {
        a.data()
            .iter()
            .zip(b.data().iter())
            .map(|(x, y)| x.abs_diff(*y))
            .max()
            .unwrap_or(0)
    }

    /// Écart maximal admis entre la composition par anneaux et le dégradé d'origine.
    ///
    /// Deux niveaux sur 255, soit 0,8 % : c'est le cumul de la quantification de
    /// l'opacité sur des entiers et de l'arrondi propre au pipeline de tiny-skia.
    /// Mesuré sur les cas ci-dessous, aucun pixel ne dépasse cet écart, et l'immense
    /// majorité est à zéro ou un niveau — invisible sur la toile sombre.
    const MAX_TOLERATED_DELTA: u8 = 2;

    /// HALO-1 — la composition par anneaux reste le dégradé d'origine, à deux niveaux près.
    #[test]
    fn test_ring_composition_matches_reference_gradient() {
        for rgb in [(96, 165, 250), (250, 204, 21), (255, 255, 255)] {
            for radius in [12.0_f32, 60.0, 180.0, 440.0] {
                let delta =
                    max_channel_delta(&render(radius, rgb, true), &render(radius, rgb, false));
                assert!(
                    delta <= MAX_TOLERATED_DELTA,
                    "halo {rgb:?} de rayon {radius} : écart max {delta} niveaux                      (toléré : {MAX_TOLERATED_DELTA})"
                );
            }
        }
    }

    /// HALO-1 — hors du disque, pas un pixel n'est touché.
    #[test]
    fn test_nothing_is_drawn_outside_the_disc() {
        let radius = 40.0_f32;
        let rendered = render(radius, (255, 0, 0), false);
        let side = rendered.width();
        let center = side as f32 / 2.0;

        for y in 0..side {
            for x in 0..side {
                let dx = x as f32 + 0.5 - center;
                let dy = y as f32 + 0.5 - center;
                if dx * dx + dy * dy <= radius * radius {
                    continue;
                }
                let pixel = rendered.pixels()[(y * side + x) as usize];
                assert_eq!(
                    (pixel.red(), pixel.green(), pixel.blue()),
                    (13, 14, 18),
                    "le pixel ({x}, {y}), hors du disque, a été modifié"
                );
            }
        }
    }

    /// HALO-1 — le centre atteint bien l'opacité nominale sur fond transparent.
    #[test]
    fn test_center_reaches_the_nominal_opacity() {
        let mut pixmap = Pixmap::new(64, 64).expect("pixmap de test");
        let mut view = pixmap.as_mut();
        draw_halo(&mut view, 32.0, 32.0, 30.0, (255, 255, 255));
        let center = pixmap.pixels()[32 * 64 + 32];
        assert!(
            center.alpha().abs_diff(HALO_CENTER_ALPHA) <= 1,
            "alpha central {}, attendu {HALO_CENTER_ALPHA}",
            center.alpha()
        );
    }

    /// Un rayon dégénéré ne doit ni paniquer ni boucler : le coût reste borné par l'écran.
    #[test]
    fn test_degenerate_radius_is_bounded_and_safe() {
        let mut pixmap = Pixmap::new(128, 128).expect("pixmap de test");
        for radius in [0.0_f32, -12.0, f32::NAN, f32::INFINITY, 1e9] {
            let started = std::time::Instant::now();
            let mut view = pixmap.as_mut();
            draw_halo(&mut view, 64.0, 64.0, radius, (120, 200, 255));
            assert!(
                started.elapsed().as_millis() < 500,
                "draw_halo ne se termine pas pour radius={radius}"
            );
        }
    }

    /// `div255` rend exactement l'arrondi au plus proche de `x / 255`.
    #[test]
    fn test_div255_is_exact_rounding() {
        for x in 0..=(255u32 * 255) {
            let expected = ((x as f64) / 255.0).round() as u32;
            assert_eq!(div255(x), expected, "div255({x})");
        }
    }

    /// Nombre de cartes du banc de performance de la passe de halos.
    const BENCH_CARD_COUNT: usize = 40;

    /// Budget de la passe de halos pour [`BENCH_CARD_COUNT`] cartes, en 1440x900, en debug.
    ///
    /// Loi L2 — le coût d'une frame ne suit pas la taille du document. Mesuré sur la
    /// machine de développement : 364 ms avec le `RadialGradient` reconstruit par carte,
    /// 58 ms avec la composition par anneaux. Le budget laisse de la marge au matériel
    /// le plus lent tout en échouant franchement si le dégradé par frame revenait.
    const HALO_PASS_BUDGET_MS: u128 = 250;

    fn bench_card(id: &str, x: f64, y: f64) -> Annotation {
        Annotation::Text {
            id: id.into(),
            x,
            y,
            width: Some(200.0),
            height: Some(50.0),
            text: String::new(),
            font_size: Some(14.0),
            color: None,
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        }
    }

    /// HALO-1 — banc de performance : la passe reste dans son budget pour 40 cartes.
    #[test]
    fn test_halo_pass_stays_within_budget_for_a_dense_board() {
        let mut pixmap = Pixmap::new(1440, 900).expect("pixmap 1440x900");
        let mut store = Store::new("Banc halos");
        let board_id = store.project.active_board_id.clone();
        for i in 0..BENCH_CARD_COUNT {
            let card = bench_card(
                &format!("halo-bench-{i}"),
                ((i % 8) as f64) * 180.0,
                ((i / 8) as f64) * 180.0,
            );
            store.add_annotation(&board_id, card);
        }

        let mut hue_cache = SymbioticHueCache::new();
        let vp = Viewport::default();
        let board = store.active_board().expect("le board vient d'être rempli");
        let visible: HashSet<&str> = board.annotations.iter().map(|a| a.id()).collect();
        hue_cache.update_positions_and_invalidate(&board.annotations);

        // Frame de chauffe : remplit le cache de teintes symbiotiques.
        {
            let mut view = pixmap.as_mut();
            draw_halos(&mut hue_cache, &mut view, &store, ViewPass { vp, visible_ids: &visible, header_h: 40.0 });
        }

        let started = std::time::Instant::now();
        {
            let mut view = pixmap.as_mut();
            draw_halos(&mut hue_cache, &mut view, &store, ViewPass { vp, visible_ids: &visible, header_h: 40.0 });
        }
        let elapsed = started.elapsed().as_millis();

        assert!(
            elapsed < HALO_PASS_BUDGET_MS,
            "passe de halos pour {BENCH_CARD_COUNT} cartes : {elapsed} ms              (budget {HALO_PASS_BUDGET_MS} ms)"
        );
    }
}
