//! **Le processeur, instrument de la loi** : chaque ligne découpée en segments.
//!
//! # Où le champ est constant, et comment le savoir sans le mesurer
//!
//! Sur une ligne, chaque couche n'a de bord que dans deux petites zones : là où sa distance
//! passe de « un demi-pixel dedans » à « un demi-pixel dehors » — pour le trait, d'une
//! demi-largeur et un demi-pixel de part et d'autre de son milieu. Ces zones se **calculent** :
//! ce sont les étendues de la forme rentrée et repoussée d'autant, et [`super::Arrondi::dilate`]
//! est exacte. Hors de toutes les zones, chaque couche couvre le pixel entier ou pas du tout :
//! `A` y est constant, et le segment se compose d'une seule couleur.
//!
//! Une membrane qui remplit l'écran ne coûte donc qu'une composition par pixel, et le calcul
//! de la loi sur les seuls pixels de ses bords — quelques milliers, au lieu de trois
//! remplissages anticrénelés de l'écran entier.
//!
//! # Pourquoi les zones sont élargies d'un pixel
//!
//! Un pixel appartient à une zone si son **centre** y tombe. Arrondir la frontière vers
//! l'extérieur d'un pixel de chaque côté ne coûte que deux évaluations de plus par bord, et
//! rend impossible qu'un pixel de bord soit pris pour un pixel constant par un arrondi de
//! flottant. L'épreuve compare le résultat, au bit près, à l'évaluation de chaque pixel.

use super::{Arrondi, Membrane};
use crate::report::Pixel;

/// Au plus deux zones de bord par couche, et une membrane a quatre couches.
const ZONES: usize = 8;

/// **Compose la membrane dans l'image**, et rend vrai si elle y a posé de l'encre.
///
/// L'image est en RGBA prémultiplié, `largeur × hauteur` pixels ; une taille qui ment ne
/// dessine rien.
pub fn peindre(m: &Membrane, image: &mut [Pixel], largeur: u32, hauteur: u32) -> bool {
    let (l, h) = (largeur as usize, hauteur as usize);
    if l == 0 || image.len() != l * h {
        return false;
    }
    let enveloppe = m.enveloppe();
    let premiere = enveloppe.haut.floor().max(0.0);
    let derniere = enveloppe.bas.ceil().min(hauteur as f32);
    // Une enveloppe `NaN` n'est ni avant ni après : elle ne dessine rien.
    if premiere.partial_cmp(&derniere) != Some(std::cmp::Ordering::Less) {
        return false;
    }
    let mut encre = false;
    for y in premiere as usize..derniere as usize {
        let ligne = &mut image[y * l..(y + 1) * l];
        encre |= peindre_la_ligne(m, ligne, y as f32 + 0.5);
    }
    encre
}

/// Les zones de bord d'une ligne, en pixels, et l'étendue de ce que la membrane y touche.
#[derive(Default)]
struct Ligne {
    zones: [(i64, i64); ZONES],
    combien: usize,
    etendue: Option<(f32, f32)>,
}

impl Ligne {
    /// Relève la couche `forme`, dont la couverture est constante au-delà de `marge` de part
    /// et d'autre de son bord.
    ///
    /// `le_long_du_bord` dit si sa couverture ne dépend que de la distance au bord — vrai d'un
    /// plein et d'un trait plein, faux d'un pointillé, qui change le long du bord.
    fn relever(&mut self, forme: &Arrondi, (marge, le_long_du_bord): (f32, bool), y: f32) {
        let Some((x0, x1)) = forme.dilate(marge).etendue(y) else {
            return;
        };
        self.etendue = Some(match self.etendue {
            Some((a, b)) => (a.min(x0), b.max(x1)),
            None => (x0, x1),
        });
        let interieur = forme.dilate(-marge).etendue(y);
        // Près d'un côté haut ou bas, la ligne entière est au bord. Mais sur sa partie droite,
        // la distance ne dépend que de la ligne : seuls les coins changent d'un pixel à l'autre.
        let droit = forme.droit_horizontal(y).filter(|_| le_long_du_bord);
        match (interieur, droit) {
            (Some((i0, i1)), _) | (None, Some((i0, i1))) => {
                self.ajouter(x0, i0);
                self.ajouter(i1, x1);
            }
            (None, None) => self.ajouter(x0, x1),
        }
    }

    /// Les pixels dont le centre tombe dans `[a, b]`, élargis d'un pixel de chaque côté.
    fn ajouter(&mut self, a: f32, b: f32) {
        if self.combien < ZONES {
            self.zones[self.combien] = (pixel_gauche(a), pixel_droit(b) + 1);
            self.combien += 1;
        }
    }

    /// Les zones de gauche à droite.
    ///
    /// Elles peuvent se chevaucher, et ce n'est pas à fondre : la peinture reprend chaque
    /// zone là où la précédente s'est arrêtée, donc un pixel n'est jamais posé deux fois.
    fn triees(&mut self) -> &[(i64, i64)] {
        let zones = &mut self.zones[..self.combien];
        zones.sort_unstable_by_key(|z| z.0);
        zones
    }
}

/// Le premier pixel dont le centre peut être à droite de `x`, un pixel plus tôt.
fn pixel_gauche(x: f32) -> i64 {
    (x - 0.5).floor() as i64 - 1
}

/// Le dernier pixel dont le centre peut être à gauche de `x`, un pixel plus tard.
fn pixel_droit(x: f32) -> i64 {
    (x - 0.5).ceil() as i64 + 1
}

/// Compose une ligne : segments constants et zones de bord, de gauche à droite.
fn peindre_la_ligne(m: &Membrane, ligne: &mut [Pixel], y: f32) -> bool {
    let mut releve = Ligne::default();
    for (forme, alpha) in &m.remplissages {
        if *alpha > 0.0 {
            releve.relever(forme, (0.5, true), y);
        }
    }
    let marge = m.bord.demi_largeur + 0.5;
    releve.relever(&m.bord.forme, (marge, m.bord.pointille.is_none()), y);
    let Some((x0, x1)) = releve.etendue else {
        return false;
    };
    let largeur = ligne.len() as i64;
    let debut = pixel_gauche(x0).clamp(0, largeur);
    let fin = (pixel_droit(x1) + 1).clamp(0, largeur);
    let mut x = debut;
    let mut encre = false;
    for &(a, b) in releve.triees() {
        let (a, b) = (a.clamp(x, fin), b.clamp(x, fin));
        encre |= segment_constant(m, ligne, (x, a), y);
        encre |= pixel_par_pixel(m, ligne, (a, b), y);
        x = x.max(b);
    }
    encre | segment_constant(m, ligne, (x, fin), y)
}

/// Un segment où `A` ne change pas : la loi s'évalue une fois, et chaque pixel se compose.
fn segment_constant(m: &Membrane, ligne: &mut [Pixel], (a, b): (i64, i64), y: f32) -> bool {
    if a >= b {
        return false;
    }
    let alpha = m.alpha(a as f32 + 0.5, y);
    if alpha <= 0.0 {
        return false;
    }
    let melange = Melange::de(m.teinte, alpha);
    for pixel in &mut ligne[a as usize..b as usize] {
        melange.poser(pixel);
    }
    true
}

/// Un segment de bord : la loi s'évalue en chaque pixel.
fn pixel_par_pixel(m: &Membrane, ligne: &mut [Pixel], (a, b): (i64, i64), y: f32) -> bool {
    let mut encre = false;
    for x in a.max(0)..b {
        let alpha = m.alpha(x as f32 + 0.5, y);
        if alpha > 0.0 {
            Melange::de(m.teinte, alpha).poser(&mut ligne[x as usize]);
            encre = true;
        }
    }
    encre
}

/// **La teinte d'opacité `A`, prête à se composer** : `source + destination × (1 − A)`, en
/// prémultiplié, en virgule fixe sur seize bits.
///
/// C'est la loi que la carte applique en mélangeant, arrondie au plus proche comme elle
/// arrondit en écrivant ; l'écart entre les deux voies n'est que celui des égalités d'arrondi.
#[derive(Clone, Copy)]
pub(super) struct Melange {
    source: [u32; 4],
    garde: u32,
}

/// L'unité de la virgule fixe.
const UN: f32 = 65_536.0;

impl Melange {
    pub(super) fn de(teinte: [f32; 3], alpha: f32) -> Self {
        // Au plus proche par `+ ½` : les valeurs sont positives, et `round` serait un appel de
        // bibliothèque sur un processeur sans instruction d'arrondi.
        let fixe = |v: f32| (v * UN + 0.5) as u32;
        Self {
            source: [
                fixe(255.0 * teinte[0] * alpha),
                fixe(255.0 * teinte[1] * alpha),
                fixe(255.0 * teinte[2] * alpha),
                fixe(255.0 * alpha),
            ],
            garde: fixe(1.0 - alpha),
        }
    }

    /// Compose sur un pixel. La somme ne dépasse jamais `255,5 × 2¹⁶` : la source vaut au
    /// plus `255·A` et la destination garde au plus `255·(1 − A)`, chacune à un demi près.
    #[inline]
    pub(super) fn poser(&self, pixel: &mut Pixel) {
        for (octet, source) in pixel.iter_mut().zip(self.source) {
            let v = (source + u32::from(*octet) * self.garde + 0x8000) >> 16;
            *octet = v.min(255) as u8;
        }
    }
}
