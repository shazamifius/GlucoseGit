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
//!
//! # HALO-3 — le rayon suit la carte, mais il est plafonné en pixels écran
//!
//! Le halo est un effet de **présentation** : il teinte le voisinage d'une carte, il n'en
//! fait pas partie. Son rayon suivait pourtant le zoom sans aucune limite — 410 px pour la
//! carte par défaut à ×1, 1 230 px à ×3 — et le coût du disque suit le carré du rayon :
//! mesuré, `halos` passait de 1,1 ms à ×1 à 10,1 ms à ×3 pour **une seule carte**, dont le
//! halo couvrait alors l'écran entier d'un lavis uniforme (**R-50**, loi L2).
//!
//! Le rayon reste une longueur monde — il grandit avec la carte, comme le veut SCALE-1 —
//! puis il est plafonné par [`HALO_MAX_SCREEN_RADIUS`], une longueur écran passée par
//! [`WorldScale::screen`] : c'est l'exception admise par SCALE-1, et elle est nommée. Le
//! plafond ne touche pas l'algorithme en anneaux de [`draw_halo`], qui reste exact.

use super::scale::WorldScale;
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
const HALO_RADIUS_FACTOR: f32 = 1.5;
/// Marge constante, en unités monde, ajoutée au rayon du halo.
const HALO_RADIUS_MARGIN: f32 = 50.0;
/// Rayon minimal du halo, en unités monde.
const HALO_MIN_RADIUS: f32 = 20.0;
/// En dessous de ce rayon écran, le halo ne couvre plus assez de pixels pour être vu.
const HALO_CULL_RADIUS: f32 = 4.0;

/// Rayon maximal d'un halo, **en pixels écran** (HALO-3).
///
/// 512 px, pour trois raisons mesurées :
///
/// 1. c'est **au-dessus** des 410 px de la carte par défaut à ×1 (240 × 1,5 + 50) : l'aspect
///    de référence ne change pas d'un pixel, et son coût de 1,1 ms non plus ;
/// 2. un disque de 1 024 px de diamètre couvre déjà toute la hauteur d'un écran 1080p ;
///    au-delà, le dégradé cesse d'être perçu comme un halo — il ne reste qu'un lavis ;
/// 3. l'aire est bornée à π · 512² ≈ 0,82 Mpx, soit **au plus 1,6 ms** pour un halo, quel
///    que soit le zoom ou la taille de la carte, contre 10 ms sans plafond à ×3.
pub(super) const HALO_MAX_SCREEN_RADIUS: f32 = 512.0;

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
pub(super) fn halo_geometry(
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

    let scale = WorldScale::new(vp.scale);
    let (sx, sy) = world_to_screen(*x, *y, vp);
    let w = scale.world(width.unwrap_or(DEFAULT_TEXT_CARD_WIDTH) as f32);
    let h = scale.world(height.unwrap_or(DEFAULT_TEXT_CARD_HEIGHT) as f32);

    let cx = sx as f32 + w / 2.0;
    let cy = sy as f32 + h / 2.0;
    // Une longueur monde, mise à l'échelle avec la carte (SCALE-1)…
    let world_radius = (w.max(h) * HALO_RADIUS_FACTOR + scale.world(HALO_RADIUS_MARGIN))
        .max(scale.world(HALO_MIN_RADIUS));
    // … puis plafonnée en pixels écran (HALO-3) : un halo ne grandit pas au-delà de ce
    // qu'un écran peut encore montrer comme un dégradé.
    let radius = world_radius.min(scale.screen(HALO_MAX_SCREEN_RADIUS));

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
mod tests;
