//! **GRILLE-1 — une lettre de l'interface se pose sur la grille de l'écran** (fiche 120).
//!
//! # Ce qu'il a vu
//!
//! Sa capture du 17/11 : *« sur énormément d'écrans, l'intégralité des boutons, que ce soient des
//! noms, des mots, des logos, est très très pixelisée, alors que je possède souvent des écrans bien
//! plus performants »*. Au pixel près, la bande de Glucose à 100 % : le haut et le bas des minuscules
//! s'étalent sur deux rangées grises, chaque fût sur deux colonnes.
//!
//! # Pourquoi
//!
//! Deux choses, que la typographie du canevas fait exprès et que l'interface paie :
//!
//! 1. **Rien n'est aligné sur la grille.** La ligne de base tombe où le centrage la met, au quart
//!    de pixel, et la hauteur d'x d'Inter vaut 0,545 du corps : 6,55 pixels à 12. Le haut d'un
//!    « x » couvre donc à moitié une rangée. Les moteurs du système (FreeType en mode « léger »,
//!    DirectWrite) alignent les lettres **verticalement seulement** : la ligne de base et la
//!    hauteur d'x sur des pixels entiers, l'espacement horizontal intact
//!    ([FreeType](https://freetype.org/freetype2/docs/hinting/text-rendering-general.html)).
//! 2. **La position au quart de pixel se fait par ré-échantillonnage** (GLYPH-1) : la lettre,
//!    rastérisée une fois, est décalée par interpolation bilinéaire. Une interpolation de couvertures
//!    n'est pas la couverture du contour décalé : elle étale chaque bord d'un pixel de plus.
//!
//! Le canevas a ses raisons de ne pas faire autrement : ses lettres vivent dans le monde, changent
//! de taille à chaque image d'un zoom et se reposent dans des textures transformées (REPOS-4) ; un
//! alignement sur la grille y ferait sauter les lettres d'un pixel au fil du zoom, et une lettre
//! seize fois plus chère à rastériser y coûterait chaque image. **L'interface, elle, se dessine au
//! pixel près et ne bouge pas** : c'est là que l'alignement a un sens, et qu'il ne coûte rien.
//!
//! # La loi
//!
//! Une typographie **posée sur l'écran** ([`super::Typography::posee_sur_l_ecran`]), que
//! l'interface reçoit, et elle seule :
//!
//! * **la ligne de base sur un pixel entier** ;
//! * **la hauteur d'x sur un nombre entier de pixels**, par un étirement **vertical seul** autour de
//!   la ligne de base ([`etirement`]) — la hauteur la plus proche ; les avances ne changent pas, donc
//!   aucune mise en page non plus : la mesure des deux typographies est la même ;
//! * **la position horizontale au quart de pixel, exacte** : la lettre se rastérise une fois à
//!   [`SOUS`] fois sa taille ([`maitre`]), et chaque phase est la moyenne de blocs de `SOUS × SOUS`
//!   sous-pixels décalés d'autant de sous-pixels que la phase ([`variante`]). La couverture d'un
//!   pixel **est** la moyenne de celle de ses sous-pixels : la réduction est exacte, et l'encre ne
//!   sort jamais de la boîte du contour décalé.
//!
//! Aucun nombre choisi : le sur-échantillonnage est le nombre de phases de GLYPH-1, la hauteur
//! visée est l'entier le plus proche de celle de la police.

use super::glyph::{
    split_position, CachedGlyph, GlyphEntry, GlyphKey, PHASE_ORIGIN, SUBPIXEL_PHASES,
};
use super::trace::Glyphe;
use super::Typography;
use fontdue::Metrics;
use std::collections::HashMap;
use std::rc::Rc;

/// **Le sur-échantillonnage du maître** : autant de sous-pixels par pixel que de phases — une
/// phase est alors un décalage d'un nombre entier de sous-pixels.
pub(super) const SOUS: usize = SUBPIXEL_PHASES as usize;

/// Le bit qui distingue, dans la clé du cache, le maître d'une lettre de ses phases — comme
/// [`super::glyph::HALO`] distingue son halo.
pub(super) const MAITRE: u8 = 0x40;

/// **L'étirement vertical qui pose la hauteur d'x sur la grille** (GRILLE-1) : le rapport de la
/// hauteur entière la plus proche à la vraie, ou 1 sans hauteur d'x (une police d'emojis) ou sous
/// un pixel.
pub(super) fn etirement(hauteur_d_x: Option<f32>, size: f32) -> f32 {
    let Some(h) = hauteur_d_x.map(|x| x * size).filter(|h| *h > 0.0) else {
        return 1.0;
    };
    let entiere = h.round();
    if entiere >= 1.0 {
        entiere / h
    } else {
        1.0
    }
}

/// **Le maître d'une lettre** : sa rastérisation à `SOUS` fois sa taille, étirée verticalement de
/// `k` autour de la ligne de base. Ses métriques sont en sous-pixels, l'ordonnée vers le haut comme
/// celles de `fontdue` ; l'avance reste celle du corps (`avance`), en pixels. `(m, source)` est
/// la rastérisation à `SOUS` fois la taille ([`Typography::rasteriser`]).
pub(super) fn maitre((m, source): (Metrics, Vec<u8>), (k, avance): (f32, f32)) -> GlyphEntry {
    let (w, h) = (m.width, m.height);
    if w == 0 || h == 0 {
        return vide(m, avance);
    }
    // Les rangées visées, l'ordonnée vers le haut : l'image de [ymin, ymin + h) par l'étirement.
    let bas = (k * m.ymin as f32).floor() as i32;
    let haut = (k * (m.ymin + h as i32) as f32).ceil() as i32;
    let hauteur = (haut - bas).max(0) as usize;
    // La rangée `i` comptée du bas, `0` hors de l'image.
    let rangee = |i: i32, c: usize| -> f32 {
        if i < 0 || i >= h as i32 {
            0.0
        } else {
            f32::from(source[(h - 1 - i as usize) * w + c])
        }
    };
    let mut bitmap = vec![0u8; w * hauteur];
    for cible in 0..hauteur {
        // Le centre de la rangée visée, ramené dans la source ; à `k = 1`, une copie exacte.
        let y = (bas + cible as i32) as f32 + 0.5;
        let u = y / k - m.ymin as f32 - 0.5;
        let (i, t) = (u.floor() as i32, u - u.floor());
        let ligne = hauteur - 1 - cible;
        for c in 0..w {
            let v = rangee(i, c) * (1.0 - t) + rangee(i + 1, c) * t;
            bitmap[ligne * w + c] = v.round().clamp(0.0, 255.0) as u8;
        }
    }
    let metrics = Metrics {
        ymin: bas,
        height: hauteur,
        advance_width: avance,
        ..m
    };
    GlyphEntry {
        metrics,
        bitmap,
        width: w,
        height: hauteur,
    }
}

/// **Une phase de la lettre** : son maître réduit par blocs de `SOUS × SOUS`, décalé de `phase`
/// sous-pixels vers la droite (GRILLE-1). Les métriques reviennent en pixels ; la ligne de base du
/// maître est sur une frontière de bloc, donc sur celle d'un pixel.
pub(super) fn variante(maitre: &GlyphEntry, phase: u8) -> GlyphEntry {
    let m = &maitre.metrics;
    let (w, h) = (maitre.width, maitre.height);
    if w == 0 || h == 0 {
        return vide(*m, m.advance_width);
    }
    let sous = SOUS as i32;
    let gauche = m.xmin + i32::from(phase % SUBPIXEL_PHASES);
    let (x0, x1) = (
        gauche.div_euclid(sous),
        (gauche + w as i32 + sous - 1).div_euclid(sous),
    );
    let (y0, y1) = (
        m.ymin.div_euclid(sous),
        (m.ymin + h as i32 + sous - 1).div_euclid(sous),
    );
    let (largeur, hauteur) = ((x1 - x0) as usize, (y1 - y0) as usize);
    let mut somme = vec![0u32; largeur * hauteur];
    for r in 0..h {
        // L'ordonnée du sous-pixel, vers le haut, puis la rangée du pixel, comptée du haut.
        let y = m.ymin + (h - 1 - r) as i32;
        let ligne = (y1 - 1 - y.div_euclid(sous)) as usize;
        for c in 0..w {
            let colonne = ((gauche + c as i32).div_euclid(sous) - x0) as usize;
            somme[ligne * largeur + colonne] += u32::from(maitre.bitmap[r * w + c]);
        }
    }
    let blocs = (SOUS * SOUS) as f32;
    let bitmap = somme
        .iter()
        .map(|s| (*s as f32 / blocs).round().min(255.0) as u8)
        .collect();
    GlyphEntry {
        metrics: Metrics {
            xmin: x0,
            ymin: y0,
            width: largeur,
            height: hauteur,
            ..*m
        },
        bitmap,
        width: largeur,
        height: hauteur,
    }
}

impl Typography {
    /// **La typographie de l'interface** (GRILLE-1) : les mêmes polices, posées sur la grille de
    /// l'écran — la ligne de base et la hauteur d'x sur des pixels entiers, la position exacte au
    /// quart de pixel. Ses avances sont celles de `self` : la mesure ne change pas.
    ///
    /// Son cache n'oublie rien d'une image à l'autre : l'interface n'écrit qu'un petit nombre fini
    /// de tailles, et sa chrome ne se redessine qu'à son changement — oublier ferait rastériser de
    /// nouveau chaque lettre au premier survol. Ce que le canevas oublie, il l'oublie parce qu'un
    /// zoom pose des tailles neuves à chaque image (fiche 104).
    pub fn posee_sur_l_ecran(&self) -> Self {
        Self::de(self.polices.clone(), true)
    }

    /// **Le maître d'une lettre posée sur l'écran** (GRILLE-1), gardé dans le cache sous sa clé à
    /// lui : étiré pour que la hauteur d'x de son visage tombe sur la grille — une lettre que le
    /// repli des emojis dessine est une image, que rien n'étire.
    pub(super) fn maitre(
        &self,
        cache: &mut HashMap<GlyphKey, CachedGlyph>,
        glyphe: Glyphe,
        size: f32,
        size_key: u16,
    ) -> Rc<GlyphEntry> {
        let cle = (glyphe.0 as u8, glyphe.1, size_key, MAITRE);
        if let Some((maitre, _)) = cache.get(&cle) {
            return maitre.clone();
        }
        // Seuls les visages ont une hauteur d'x : un émoji est une image, que rien n'étire.
        let hauteur_d_x = self.polices.hauteurs_d_x.get(glyphe.0).copied().flatten();
        let pas = (etirement(hauteur_d_x, size), self.avance_de(glyphe, size));
        let rasterise = self.rasteriser(glyphe, size * SOUS as f32);
        let maitre = Rc::new(maitre(rasterise, pas));
        cache.insert(cle, (maitre.clone(), self.image.get()));
        maitre
    }

    /// **La ligne de base d'une ligne** : au quart de pixel dans le monde (GLYPH-1), sur un pixel
    /// entier sur l'écran (GRILLE-1).
    pub(super) fn ligne_de_base(&self, y: f32) -> (i32, u8) {
        if self.sur_l_ecran && y.is_finite() {
            (y.round() as i32, PHASE_ORIGIN)
        } else {
            split_position(y)
        }
    }
}

/// Une lettre sans encre (une espace) : son avance seule.
fn vide(m: Metrics, avance: f32) -> GlyphEntry {
    GlyphEntry {
        metrics: Metrics {
            width: 0,
            height: 0,
            advance_width: avance,
            ..m
        },
        bitmap: Vec::new(),
        width: 0,
        height: 0,
    }
}

#[cfg(test)]
#[path = "ecran/tests.rs"]
mod tests;
