//! La lueur d'une carte : l'ombre portée de Glucose Tauri, reproduite.
//!
//! # Ce que Glucose Tauri dessine, et ce que celui-ci dessinait
//!
//! Glucose Tauri pose une ombre CSS sur la boîte de la carte (`HtmlAnnotationLayer.tsx`) :
//!
//! ```css
//! box-shadow: 0 0 60px 30px color-mix(in srgb, AURA 15%, transparent);
//! ```
//!
//! — la boîte **dilatée de 30**, puis **floutée de 60**, à 15 % d'opacité. Son conteneur
//! porte `translate(…) scale(…)`, donc ces longueurs sont des unités **monde** : la lueur
//! grandit avec la carte, comme le veut SCALE-1, et rien ici n'a besoin de l'exception
//! écran.
//!
//! Ici, c'était un **disque** de rayon `max(w, h) × 1,5 + 50`. Sur une carte de 240 × 60,
//! cela fait 410 : la lueur débordait de 380 unités au-dessus d'une carte qui n'en mesure
//! que 60 de haut, et plus une carte est large, plus la bulle déborde **verticalement**.
//! L'opacité, elle, était juste — 35 sur 255 contre les 38 de Tauri. Ce n'était donc pas
//! une question d'intensité : c'était une forme qui ne désignait plus sa carte.
//!
//! # HALO-1 — une ombre de boîte est SÉPARABLE, et c'est ce qui la rend gratuite
//!
//! Flouter un rectangle par une gaussienne, c'est multiplier deux profils d'une seule
//! dimension :
//!
//! ```text
//! α(x, y) = A · P(x ; gauche, droite) · P(y ; haut, bas)
//! P(t ; a, b) = Φ(t − a) − Φ(t − b)
//! ```
//!
//! C'est **exact**, et non une approximation : une gaussienne à deux dimensions est le
//! produit de deux gaussiennes à une dimension, et l'intégrale sur un rectangle se
//! factorise donc terme à terme. Il suffit d'une table de `Φ` — elle ne dépend que de
//! l'écart-type en pixels, donc du zoom, et une frame n'en construit qu'une —, d'un
//! tableau par carte, et d'une multiplication par pixel.
//!
//! La seule chose que le produit ne capture pas est le `border-radius: 32px` de la boîte
//! de Tauri : il sépare un rectangle **droit**, donc la lueur est très légèrement plus
//! carrée dans les coins. L'écart s'y borne à la différence d'aire entre le carré du coin
//! et son quart de disque, étalée par le flou — quelques niveaux sur une lueur qui
//! plafonne à 38.
//!
//! La comparaison de coût, à géométrie égale (une carte de 240 × 60 à ×1) :
//!
//! | | disque | ombre séparable |
//! |---|---|---|
//! | pixels touchés | π · 410² ≈ **528 000** | 432 × 252 ≈ **109 000** |
//! | par pixel | recherche d'anneau, composition | une multiplication, composition |
//!
//! # HALO-2 — la portée se déduit du huit bits, elle ne se règle pas
//!
//! Une gaussienne n'a pas de bord : il faut bien décider où l'on cesse de la dessiner. Ce
//! n'est pas un réglage. La lueur s'écrit sur huit bits, donc une contribution inférieure
//! à un demi-niveau ne change aucun pixel, et la table de `Φ` se tronque exactement là où
//! sa masse restante passe sous ce seuil — pas avant, jamais après. La portée qui en sort
//! dépend de l'opacité et de l'écart-type, et c'est [`EdgeProfile::reach`] qui la dit.
//!
//! # HALO-3 — pourquoi il n'y a *aucun* cache ici
//!
//! Un cache de lueurs pré-rastérisées achèterait de la mémoire — et une borne à tenir, et
//! une invalidation — pour supprimer une multiplication par pixel. Le seul état conservé
//! d'une frame à l'autre reste la teinte symbiotique, dans [`SymbioticHueCache`] : elle,
//! contrairement au profil, dépend du voisinage et coûte cher à recalculer.

use super::scale::WorldScale;
use super::SymbioticHueCache;
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use tiny_skia::{PixmapMut, PremultipliedColorU8};

/// Dilatation de la boîte avant le flou, en unités monde — le `30px` de Tauri.
pub const HALO_SPREAD: f32 = 30.0;

/// Rayon de flou, en unités monde — le `60px` de Tauri.
///
/// CSS définit ce rayon comme **le double** de l'écart-type de la gaussienne : σ vaut
/// donc 30 unités monde.
pub const HALO_BLUR: f32 = 60.0;

/// Opacité de la lueur, sur 255 — les 15 % de `color-mix(in srgb, AURA 15%, transparent)`.
pub const HALO_ALPHA: u8 = 38;

/// L'écart-type de la gaussienne, en unités monde (CSS : `blur = 2σ`).
const HALO_SIGMA: f32 = HALO_BLUR / 2.0;

/// Divise par 255 avec arrondi au plus proche, sans division entière.
///
/// Exact pour tout produit de deux octets : c'est l'identité `(x + 128 + (x + 128) / 256) / 256`
/// qu'utilisent les compositeurs 8 bits pour rester au niveau près de `x / 255.0`.
fn div255(value: u32) -> u32 {
    let biased = value + 128;
    (biased + (biased >> 8)) >> 8
}

/// Terme source d'un niveau d'opacité : la couleur de la lueur déjà multipliée par lui.
///
/// La lueur ne prend que [`HALO_ALPHA`] niveaux distincts une fois écrite sur huit bits,
/// donc tous se précalculent en une fois, et la boucle chaude se réduit à une indexation.
#[derive(Clone, Copy)]
struct LevelSource {
    /// Canaux `r`, `g`, `b` et alpha, chacun multiplié par `alpha`. Somme à diviser par 255.
    scaled: [u32; 4],
    /// `255 - alpha`, le poids qui reste à la destination.
    inv_alpha: u32,
}

impl LevelSource {
    fn new((r, g, b): (u8, u8, u8), alpha: u8) -> Self {
        let weight = u32::from(alpha);
        Self {
            scaled: [
                u32::from(r) * weight,
                u32::from(g) * weight,
                u32::from(b) * weight,
                255 * weight,
            ],
            inv_alpha: 255 - weight,
        }
    }
}

/// Compose une couleur d'opacité constante sur un pixel (boucle chaude de la lueur).
///
/// La composition « source-over » prémultipliée s'écrit `(c * a + d * (255 - a)) / 255`
/// pour chaque canal. L'écrire ainsi — plutôt qu'en prémultipliant d'abord la source —
/// n'arrondit **qu'une fois**, comme le fait le pipeline flottant de tiny-skia.
fn blend_pixel(pixel: &mut PremultipliedColorU8, source: LevelSource) {
    let [sr, sg, sb, sa] = source.scaled;
    let inv_alpha = source.inv_alpha;
    let r = div255(sr + u32::from(pixel.red()) * inv_alpha);
    let g = div255(sg + u32::from(pixel.green()) * inv_alpha);
    let b = div255(sb + u32::from(pixel.blue()) * inv_alpha);
    let a = div255(sa + u32::from(pixel.alpha()) * inv_alpha);
    // `r`, `g` et `b` restent ≤ `a` : la source est valide (canal ≤ 255) et la
    // destination l'est aussi (canal ≤ alpha). `from_rgba` ne peut pas rendre `None`.
    if let Some(blended) = PremultipliedColorU8::from_rgba(r as u8, g as u8, b as u8, a as u8) {
        *pixel = blended;
    }
}

/// Le profil d'un bord flouté : la gaussienne cumulée, échantillonnée au pixel.
///
/// `Φ(t)` est la fraction de la lueur qui tombe à gauche de la distance `t`. Elle ne
/// dépend que de l'écart-type **en pixels écran**, donc du zoom : une seule table sert à
/// toutes les cartes d'une frame (HALO-1).
struct EdgeProfile {
    /// Masse cumulée aux frontières de pixel, de `0` à `1`. Sa longueur vaut `2r + 2`.
    cdf: Vec<f32>,
    /// Le rayon `r` du noyau, en pixels : `cdf[0]` est la masse à gauche de `−r − ½`.
    radius: f32,
}

impl EdgeProfile {
    /// Le profil d'une gaussienne d'écart-type `sigma`, en pixels écran.
    ///
    /// Un écart-type nul — ou sous le demi-pixel — donne un noyau d'un seul échantillon,
    /// c'est-à-dire exactement la marche d'un bord net. Ce cas ne demande donc aucune
    /// garde : il tombe de la construction.
    fn new(sigma: f32) -> Self {
        // Majorant d'allocation : au-delà de quatre écarts-types, la masse restante vaut
        // 3 · 10⁻⁵ — sous le demi-niveau de huit bits pour toute opacité admissible.
        // Ce n'est pas la portée : celle-ci est mesurée plus bas, sur la table obtenue.
        let radius = if sigma.is_finite() && sigma > 0.0 {
            (4.0 * sigma).ceil().min(4096.0)
        } else {
            0.0
        };
        let span = 2.0f32.mul_add(radius, 1.0) as usize;
        let two_sigma_squared = 2.0 * sigma * sigma;

        let mut cdf = Vec::with_capacity(span + 1);
        cdf.push(0.0);
        let mut total = 0.0f32;
        for k in 0..span {
            let d = k as f32 - radius;
            total += if two_sigma_squared > 0.0 {
                (-d * d / two_sigma_squared).exp()
            } else {
                f32::from(d == 0.0)
            };
            cdf.push(total);
        }
        for value in &mut cdf {
            *value /= total;
        }
        Self { cdf, radius }
    }

    /// La distance, en pixels, au-delà de laquelle la lueur ne peut plus changer un pixel.
    ///
    /// Elle ne se règle pas, elle se mesure (HALO-2). À la distance `d` au-delà d'un bord,
    /// la lueur vaut au plus `alpha · Q(d)`, où `Q` est la masse que la table laisse
    /// derrière elle ; elle s'arrondit donc à zéro dès que `Q(d)` passe sous `½ / alpha`.
    /// On avance dans la queue jusqu'à ce point, et pas d'un pixel de plus.
    fn reach(&self, alpha: u8) -> f32 {
        let negligible = 0.5 / f32::from(alpha).max(1.0);
        let inside = self
            .cdf
            .iter()
            .rposition(|&mass| 1.0 - mass > negligible)
            .unwrap_or(0);
        // `inside` est la dernière frontière qui compte encore ; sa distance au centre est
        // `inside − (r + ½)`, et la portée est cette distance vue depuis le bord.
        (inside as f32 - self.radius - 0.5).max(0.0)
    }

    /// `Φ(t)` : la fraction de la lueur qui tombe à gauche de la distance `t`, en pixels.
    fn cumulative(&self, t: f32) -> f32 {
        let u = t + self.radius + 0.5;
        if u <= 0.0 {
            return 0.0;
        }
        let last = self.cdf.len() - 1;
        if u >= last as f32 {
            return 1.0;
        }
        let index = u as usize;
        let fraction = u - index as f32;
        self.cdf[index] + (self.cdf[index + 1] - self.cdf[index]) * fraction
    }

    /// La part de la lueur qu'une bande `[a, b]` dépose à la position `t`.
    ///
    /// C'est `∫ₐᵇ g(t − s) ds` : le changement de variable `u = t − s` le rend égal à
    /// `Φ(t − a) − Φ(t − b)`, soit une soustraction de deux lectures de table.
    fn band(&self, t: f32, a: f32, b: f32) -> f32 {
        self.cumulative(t - a) - self.cumulative(t - b)
    }
}

/// La boîte **dilatée** d'une carte, en pixels écran : celle que le flou étale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct HaloBox {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    /// L'écart-type de la gaussienne, en pixels écran.
    pub sigma: f32,
}

/// Première rangée (ou colonne) dont le centre de pixel atteint `position`, bornée à l'écran.
fn first_pixel_at_or_after(position: f32, limit: i32) -> i32 {
    if !position.is_finite() {
        return if position.is_sign_negative() {
            0
        } else {
            limit
        };
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

/// Compose la lueur d'une boîte sur `dst` (HALO-1).
pub(super) fn draw_halo(dst: &mut PixmapMut, halo: HaloBox, rgb: (u8, u8, u8), alpha: u8) {
    if alpha == 0
        || !halo.left.is_finite()
        || !halo.top.is_finite()
        || !halo.right.is_finite()
        || !halo.bottom.is_finite()
    {
        return;
    }
    let profile = EdgeProfile::new(halo.sigma);
    let reach = profile.reach(alpha);

    let width = dst.width() as i32;
    let height = dst.height() as i32;
    let x0 = first_pixel_at_or_after(halo.left - reach, width);
    let x1 = first_pixel_at_or_after(halo.right + reach, width);
    let y0 = first_pixel_at_or_after(halo.top - reach, height);
    let y1 = first_pixel_at_or_after(halo.bottom + reach, height);
    if x1 <= x0 || y1 <= y0 {
        return;
    }

    // Le profil horizontal ne dépend que de la colonne : il se calcule une fois pour
    // toutes les lignes, et la boucle chaude n'y fait plus qu'une lecture.
    let columns: Vec<f32> = (x0..x1)
        .map(|x| profile.band(x as f32 + 0.5, halo.left, halo.right))
        .collect();
    // La lueur ne prend que `alpha` niveaux distincts : ils se précalculent tous.
    let levels: Vec<LevelSource> = (0..=alpha).map(|a| LevelSource::new(rgb, a)).collect();

    let pixels = dst.pixels_mut();
    let peak = f32::from(alpha);
    for y in y0..y1 {
        let row_weight = profile.band(y as f32 + 0.5, halo.top, halo.bottom) * peak;
        if row_weight < 0.5 {
            continue;
        }
        let base = (y * width) as usize;
        for (column, x) in (x0..x1).enumerate() {
            let level = (row_weight * columns[column]).round() as usize;
            if level == 0 {
                continue;
            }
            blend_pixel(
                &mut pixels[base + x as usize],
                levels[level.min(levels.len() - 1)],
            );
        }
    }
}

/// La boîte dilatée de la lueur d'une carte, ou `None` si elle ne touche pas le cadre.
pub(super) fn halo_geometry(
    ann: &Annotation,
    vp: &Viewport,
    screen_w: f32,
    screen_h: f32,
    header_h: f32,
) -> Option<HaloBox> {
    if !matches!(ann, Annotation::Text { .. }) {
        return None;
    }
    let rect = ann.rect()?;

    let scale = WorldScale::new(vp.scale);
    let (sx, sy) = world_to_screen(rect.left, rect.top, vp);
    // Toutes ces longueurs sont des unités monde mises à l'échelle : la lueur grandit
    // avec sa carte (SCALE-1), et rien ici ne demande l'exception écran.
    let spread = scale.world(HALO_SPREAD);
    let halo = HaloBox {
        left: sx as f32 - spread,
        top: sy as f32 - spread,
        right: sx as f32 + scale.world(rect.width as f32) + spread,
        bottom: sy as f32 + scale.world(rect.height as f32) + spread,
        sigma: scale.world(HALO_SIGMA),
    };

    // Frustum culling : la lueur déborde de la portée du flou, et pas d'un pixel de plus.
    //
    // La condition dit ce qu'il faut pour dessiner, et non les quatre façons de sortir.
    // Ce n'est pas qu'une question de lisibilité : un viewport brisé rend des bornes
    // `NaN`, toute comparaison avec `NaN` est fausse, et une condition de **rejet** les
    // laissait donc toutes passer — `halo_geometry` annonçait alors une boîte qui n'existe
    // pas. Écrite en positif, elle les rejette sans avoir à les nommer.
    let reach = EdgeProfile::new(halo.sigma).reach(HALO_ALPHA);
    let visible = halo.right + reach >= 0.0
        && halo.left - reach <= screen_w
        && halo.bottom + reach >= header_h
        && halo.top - reach <= screen_h;
    visible.then_some(halo)
}

/// Passe de rendu des lueurs d'ambiance (L1 : ne parcourt que les cartes visibles).
pub fn draw_halos(
    hue_cache: &mut SymbioticHueCache,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) {
    let ViewPass {
        visibles, header_h, ..
    } = pass;
    let vp = &pass.vp;
    let Some(board) = store.active_board() else {
        return;
    };

    let screen_w = pixmap.width() as f32;
    let screen_h = pixmap.height() as f32;

    // On va droit aux nœuds visibles (CULL-1). La version précédente parcourait le tableau
    // entier en demandant de chacun s'il était visible : sur un million de nœuds dont cinq
    // cents à l'écran, c'était un million de hachages de chaîne pour cette seule passe.
    for ann in Visibles::nouvelles(visibles, board).annotations() {
        let Some(halo) = halo_geometry(ann, vp, screen_w, screen_h, header_h) else {
            continue;
        };
        let (_hue, rgb) = hue_cache.get_or_compute(ann, &board.annotations);
        draw_halo(pixmap, halo, rgb, HALO_ALPHA);
    }
}

#[cfg(test)]
mod tests;
