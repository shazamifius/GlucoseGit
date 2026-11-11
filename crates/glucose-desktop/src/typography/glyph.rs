//! Le cache de glyphes et le positionnement sous-pixel (GLYPH-1, R-46).
//!
//! Ce module ne sait rien du texte : il rastérise, décale, mélange et borne. La façon dont
//! une chaîne s'enchaîne et se mesure appartient au module parent.

use fontdue::Metrics;
use std::rc::Rc;

/// Nombre de positions sous-pixel retenues **par axe** (GLYPH-1).
///
/// Quatre phases par axe arrondissent la position au huitième de pixel près — sous le seuil
/// de perception — pour au plus seize variantes par glyphe. Monter à huit diviserait
/// l'erreur par deux et multiplierait par quatre la pression sur le cache, pour une
/// différence que personne ne verrait.
pub const SUBPIXEL_PHASES: u8 = 4;

/// La variante non décalée : celle que `fontdue` a réellement rastérisée.
pub(super) const PHASE_ORIGIN: u8 = 0;

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

/// Clé de cache d'un glyphe : visage, caractère, taille en dixièmes de point, phase.
pub(crate) type GlyphKey = (super::Face, char, u16, u8);

/// Valeur de cache : le glyphe partagé, et la dernière image qui l'a demandé (fiche 104).
pub(super) type CachedGlyph = (Rc<GlyphEntry>, u64);

/// Sépare une coordonnée en une position entière et la phase sous-pixel la plus proche.
///
/// L'arrondi porte sur la position quantifiée, pas sur la position elle-même : l'erreur
/// maximale est donc d'une demi-phase, soit un huitième de pixel.
/// **Une position sur la grille des glyphes** : au quart de pixel le plus proche (GLYPH-1).
///
/// C'est là que le texte se pose, et c'est là que la carte qui le porte doit se poser aussi
/// (LUEUR-3) : son fond et ses lettres avancent alors ensemble, par les mêmes pas. Et sur une
/// grille dyadique, les sommes sont **exactes** — `80,75 + 240` comme `2,75 + 240` —, là où
/// `80,8 + 240` et `2,8 + 240` s'arrondissent différemment : la même forme, posée en place ou
/// dans une texture décalée d'un nombre entier de pixels, se calcule alors au bit près.
pub(crate) fn sur_la_grille(valeur: f32) -> f32 {
    let phases = f32::from(SUBPIXEL_PHASES);
    (valeur * phases).round() / phases
}

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
    let weights = [
        (1.0 - dx) * (1.0 - dy),
        dx * (1.0 - dy),
        (1.0 - dx) * dy,
        dx * dy,
    ];
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
    GlyphEntry {
        metrics: origin.metrics,
        bitmap,
        width: out_w,
        height: out_h,
    }
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

/// Le bit qui distingue, dans la clé du cache, le halo d'une lettre de ses variantes (TITRE-2).
pub(super) const HALO: u8 = 0x80;

/// **Le halo d'une lettre** (TITRE-2, fiche 95) : ses huit copies du détourage
/// ([`super::DETOURAGE`]), composées une fois.
///
/// Composer une teinte sur elle-même ne change que son opacité — la loi de MEMB-FORME-1 : huit
/// passes d'une même couleur valent une passe d'opacité `1 − Π(1 − cₖ)`. Un texte détouré se
/// compose donc en deux passes, le halo puis la lettre, au lieu de neuf : le profil d'un zoom sur
/// l'article « Eau », ses titres à la taille de leur région, mettait [`blend_glyph`] à 32 % du
/// temps. Chaque passe d'avant tronquait sa composition à l'octet : le halo en diffère d'au plus
/// un niveau par copie qui se recouvre.
///
/// `copies` : l'image de chaque copie, et son coin relatif à la cellule de la lettre.
pub(super) fn halo(copies: &[(Rc<GlyphEntry>, (i32, i32))], modele: &Metrics) -> GlyphEntry {
    let gauche = copies.iter().map(|(_, c)| c.0).min().unwrap_or(0);
    let haut = copies.iter().map(|(_, c)| c.1).min().unwrap_or(0);
    let droite = copies.iter().map(|(e, c)| c.0 + e.width as i32);
    let bas = copies.iter().map(|(e, c)| c.1 + e.height as i32);
    let largeur = (droite.max().unwrap_or(0) - gauche).max(0) as usize;
    let hauteur = (bas.max().unwrap_or(0) - haut).max(0) as usize;
    let mut transparence = vec![1.0f32; largeur * hauteur];
    for (e, (x, y)) in copies {
        let (dx, dy) = ((x - gauche) as usize, (y - haut) as usize);
        for rang in 0..e.height {
            for col in 0..e.width {
                let c = f32::from(e.bitmap[rang * e.width + col]) / 255.0;
                transparence[(rang + dy) * largeur + col + dx] *= 1.0 - c;
            }
        }
    }
    let bitmap = transparence
        .iter()
        .map(|t| ((1.0 - t) * 255.0).round() as u8)
        .collect();
    let metrics = Metrics {
        xmin: gauche,
        ymin: -haut - hauteur as i32,
        width: largeur,
        height: hauteur,
        ..*modele
    };
    GlyphEntry {
        metrics,
        bitmap,
        width: largeur,
        height: hauteur,
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
