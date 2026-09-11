//! Le cache de glyphes et le positionnement sous-pixel (GLYPH-1, R-46).
//!
//! Ce module ne sait rien du texte : il rastérise, décale, mélange et borne. La façon dont
//! une chaîne s'enchaîne et se mesure appartient au module parent.

use fontdue::Metrics;
use std::collections::HashMap;
use std::rc::Rc;

/// Nombre de positions sous-pixel retenues **par axe** (GLYPH-1).
///
/// Quatre phases par axe arrondissent la position au huitième de pixel près — sous le seuil
/// de perception — pour au plus seize variantes par glyphe. Monter à huit diviserait
/// l'erreur par deux et multiplierait par quatre la pression sur le cache, pour une
/// différence que personne ne verrait.
pub(super) const SUBPIXEL_PHASES: u8 = 4;

/// La variante non décalée : celle que `fontdue` a réellement rastérisée.
pub(super) const PHASE_ORIGIN: u8 = 0;

/// Nombre maximal de variantes de glyphes gardées en mémoire.
///
/// Mesuré, pas deviné : une frame complète en 1440×900, docks compris, occupe **531**
/// variantes (`test_full_frame_render_stays_within_time_budget`), et un balayage continu de
/// 600 positions sur cinq tailles — qui traverse donc les seize phases de chaque glyphe —
/// s'arrête à **772** (`test_glyph_1_the_cache_stays_bounded_across_every_phase`). Ces
/// deux chiffres sont ceux d'Inter (FONT-1) ; ils valaient 518 et 754 avec la police
/// d'origine, et le passage n'a rien changé au plafond. Le
/// plafond hérité de R-40 laisse donc plus de cinq fois la marge nécessaire, et l'éviction
/// LRU borne le reste par construction : le positionnement sous-pixel n'a pas eu besoin
/// d'agrandir le cache.
pub(super) const GLYPH_CACHE_CAPACITY: usize = 4096;

#[derive(Clone)]
pub struct GlyphEntry {
    pub metrics: Metrics,
    pub bitmap: Vec<u8>,
    /// Dimensions réelles de `bitmap`. Elles dépassent celles de `metrics` d'un pixel sur
    /// les variantes décalées, qui débordent sur le pixel voisin — c'est précisément ce
    /// débord qui porte le positionnement sous-pixel.
    pub width: usize,
    pub height: usize,
}

/// Clé de cache d'un glyphe : graisse, caractère, taille en dixièmes de point, phase.
pub(crate) type GlyphKey = (bool, char, u16, u8);

/// Valeur de cache : le glyphe partagé et l'horodatage de son dernier accès (LRU, R-40).
pub(super) type CachedGlyph = (Rc<GlyphEntry>, u64);

/// Sépare une coordonnée en une position entière et la phase sous-pixel la plus proche.
///
/// L'arrondi porte sur la position quantifiée, pas sur la position elle-même : l'erreur
/// maximale est donc d'une demi-phase, soit un huitième de pixel.
pub(super) fn split_position(value: f32) -> (i32, u8) {
    if !value.is_finite() {
        return (0, PHASE_ORIGIN);
    }
    let phases = SUBPIXEL_PHASES as f32;
    let quantised = (value * phases).round();
    let cell = (quantised / phases).floor();
    (cell as i32, (quantised - cell * phases) as u8)
}

/// Décalage, en fraction de pixel, porté par une phase.
fn phase_offset(phase: u8) -> (f32, f32) {
    let phases = SUBPIXEL_PHASES as f32;
    (
        (phase % SUBPIXEL_PHASES) as f32 / phases,
        (phase / SUBPIXEL_PHASES) as f32 / phases,
    )
}

/// Recompose la couverture d'un glyphe décalée de `phase`, par interpolation bilinéaire.
///
/// Le résultat fait un pixel de plus sur chaque axe : la couverture qui débordait entre
/// deux pixels est désormais écrite, au lieu d'être perdue par la troncature.
pub(super) fn shifted_glyph(origin: &GlyphEntry, phase: u8) -> GlyphEntry {
    let (w, h) = (origin.width, origin.height);
    if w == 0 || h == 0 {
        return origin.clone();
    }
    let (dx, dy) = phase_offset(phase);
    let weights = [(1.0 - dx) * (1.0 - dy), dx * (1.0 - dy), (1.0 - dx) * dy, dx * dy];
    let (out_w, out_h) = (w + 1, h + 1);
    let mut bitmap = vec![0u8; out_w * out_h];

    let sample = |row: isize, col: isize| -> f32 {
        if row < 0 || col < 0 || row >= h as isize || col >= w as isize {
            0.0
        } else {
            origin.bitmap[row as usize * w + col as usize] as f32
        }
    };

    for row in 0..out_h {
        for col in 0..out_w {
            let (r, c) = (row as isize, col as isize);
            let coverage = sample(r, c) * weights[0]
                + sample(r, c - 1) * weights[1]
                + sample(r - 1, c) * weights[2]
                + sample(r - 1, c - 1) * weights[3];
            bitmap[row * out_w + col] = coverage.round().min(255.0) as u8;
        }
    }
    GlyphEntry { metrics: origin.metrics, bitmap, width: out_w, height: out_h }
}

/// Évince les 25 % les plus anciens quand le cache est plein (LRU, R-40).
///
/// Une table rase complète relancerait une rastérisation de toute la frame suivante ;
/// évincer par quarts garde le texte affiché à l'écran, qui est par construction le plus
/// récemment consulté.
pub(super) fn evict_if_full(cache: &mut HashMap<GlyphKey, CachedGlyph>) {
    if cache.len() < GLYPH_CACHE_CAPACITY {
        return;
    }
    let mut accesses: Vec<u64> = cache.values().map(|(_, a)| *a).collect();
    accesses.sort_unstable();
    let cutoff = accesses[accesses.len() / 4];
    cache.retain(|_, (_, a)| *a > cutoff);
}

/// Compose la couverture d'un glyphe sur le tampon, en « source-over » prémultiplié (R-27).
///
/// La position est **entière** : la fraction de pixel a déjà été absorbée par la phase du
/// glyphe (GLYPH-1), donc cette boucle reste exactement aussi chère qu'avant le correctif.
pub(super) fn blend_glyph(
    data: &mut [u8],
    bounds: (i32, i32),
    entry: &GlyphEntry,
    at: (i32, i32),
    color: (f32, f32, f32, f32),
) {
    let (w, h) = bounds;
    let (r, g, b, a) = color;
    for row in 0..entry.height {
        let py = at.1 + row as i32;
        if py < 0 || py >= h {
            continue;
        }
        for col in 0..entry.width {
            let coverage = entry.bitmap[row * entry.width + col] as f32 / 255.0;
            if coverage <= 0.0 {
                continue;
            }
            let px = at.0 + col as i32;
            if px < 0 || px >= w {
                continue;
            }
            let idx = ((py as usize) * (w as usize) + (px as usize)) * 4;
            let src_a = a * coverage;
            let inv_src_a = 1.0 - src_a;

            let dr = data[idx] as f32;
            let dg = data[idx + 1] as f32;
            let db = data[idx + 2] as f32;
            let da = data[idx + 3] as f32;

            // R-27 : mélange alpha prémultiplié sans écraser l'alpha de destination.
            data[idx] = (r * src_a + dr * inv_src_a).min(255.0) as u8;
            data[idx + 1] = (g * src_a + dg * inv_src_a).min(255.0) as u8;
            data[idx + 2] = (b * src_a + db * inv_src_a).min(255.0) as u8;
            data[idx + 3] = (src_a * 255.0 + da * inv_src_a).min(255.0) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_1_split_position_rounds_to_the_nearest_phase() {
        assert_eq!(split_position(10.0), (10, 0));
        assert_eq!(split_position(10.25), (10, 1));
        assert_eq!(split_position(10.5), (10, 2));
        assert_eq!(split_position(10.74), (10, 3));
        // 10,9 est plus proche de 11 que de 10,75 : il bascule sur le pixel suivant.
        assert_eq!(split_position(10.9), (11, 0));
        assert_eq!(split_position(-0.3), (-1, 3));
        assert_eq!(split_position(f32::NAN), (0, 0));
    }
}
