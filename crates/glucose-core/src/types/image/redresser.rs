//! **EXIF-1 — un nœud posé couché se redresse** (fiche 165 § 3.3 et § 6).
//!
//! Avant EXIF-1, une photo de téléphone prise en portrait arrivait couchée : ses dimensions
//! d'origine, et la boîte de son nœud, étaient celles de la photo couchée. Depuis, ses pixels
//! arrivent droits, et une orientation qui échange largeur et hauteur (EXIF 5 à 8) les montrerait
//! écrasés dans une boîte restée en paysage. **Le signe est dans les données du nœud** : après
//! EXIF-1, les dimensions d'origine sont toujours celles de la photo droite ; un nœud dont elles
//! sont la transposée des pixels décodés (et pas un carré) ne peut donc venir que d'avant — sans
//! champ neuf, ni en-tête relu à l'ouverture.
//!
//! Les orientations 2, 3 et 4 n'en laissent aucun, ni un portrait carré : la photo y paraît enfin
//! droite, et ce qui avait été posé sur la photo couchée (un recadrage, un demi-tour) reste tel
//! quel.
//!
//! L'orientation se dit ici par son numéro EXIF (1 à 8) : le noyau n'a aucune dépendance.

use super::BoardImage;
use crate::rotate::normalize;
use crate::types::Recadrage;
use std::f64::consts::FRAC_PI_2;

/// **Ce que l'orientation EXIF fait aux pixels pour les montrer debout**, écrit comme un nœud
/// (MIROIR-1) : retournés de gauche à droite d'abord (le booléen), puis tournés d'autant de quarts
/// de tour dans le sens des aiguilles. Le sens d'`image` : « 6 » est un quart, « 5 » la
/// transposée (trois quarts après le miroir). `None` hors de 1 à 8.
fn transformation(exif: u8) -> Option<(i64, bool)> {
    Some(match exif {
        1 => (0, false),
        2 => (0, true),
        3 => (2, false),
        4 => (2, true),
        5 => (3, true),
        6 => (1, false),
        7 => (1, true),
        8 => (3, false),
        _ => return None,
    })
}

/// **Cet angle en quarts de tour, s'il en est un au grain du document** — qui garde les rotations
/// en `f32` : le quart de tour d'un glisser verrouillé (`Maj`) n'y est pas exactement `π/2`.
fn quarts_exacts(angle: f64) -> Option<i64> {
    let quarts = (angle / FRAC_PI_2).round();
    (angle as f32 == (quarts * FRAC_PI_2) as f32).then_some(quarts as i64)
}

impl Recadrage {
    /// **Le même cadrage, sur les pixels tournés par cette orientation** : chaque marge suit son
    /// bord. Dans l'ordre gauche, haut, droite, bas, un quart de tour avance chaque bord d'un rang ;
    /// le miroir échange la gauche et la droite.
    pub fn oriente(self, exif: u8) -> Self {
        let Some((quarts, miroir)) = transformation(exif) else {
            return self;
        };
        let avant = [self.gauche, self.haut, self.droite, self.bas];
        let mut apres = avant;
        for (bord, marge) in (0i64..).zip(avant) {
            let bord = if miroir { 2 - bord } else { bord };
            apres[(bord + quarts).rem_euclid(4) as usize] = marge;
        }
        let [gauche, haut, droite, bas] = apres;
        Self::depuis_les_marges(gauche, haut, droite, bas)
    }
}

impl BoardImage {
    /// **Ce nœud montre-t-il couchée la photo décodée ?** `pixels` sont ses dimensions droites,
    /// `exif` l'orientation qu'on a donnée à ses pixels.
    pub fn couchee(&self, exif: u8, (largeur, hauteur): (u32, u32)) -> bool {
        transformation(exif).is_some_and(|(quarts, _)| quarts % 2 == 1)
            && largeur != hauteur
            && (self.original_width, self.original_height)
                == (f64::from(hauteur), f64::from(largeur))
    }

    /// **Redresse ce nœud s'il montre la photo couchée** ; rend `true` s'il a changé.
    ///
    /// Sa boîte tourne d'un quart autour de son centre (largeur et hauteur échangées), ses
    /// dimensions d'origine aussi, et son recadrage suit les pixels. **Sa rotation se lit de deux
    /// façons** : un geste sur la photo telle qu'elle paraissait — elle reste la sienne, et la
    /// photo paraît enfin debout —, ou une correction de la photo couchée, qu'il avait redressée à
    /// la main — elle n'a plus d'objet, et ce qu'il voyait ne change pas. Des deux, le nœud garde
    /// celle qui le laisse **le moins tourné** : un nœud jamais tourné reste droit, un nœud
    /// redressé d'un quart revient à zéro, un nœud redressé puis penché garde sa pente.
    pub fn redresser(&mut self, exif: u8, pixels: (u32, u32)) -> bool {
        let Some((quarts, miroir)) = transformation(exif).filter(|_| self.couchee(exif, pixels))
        else {
            return false;
        };
        (self.width, self.height) = (self.height, self.width);
        (self.original_width, self.original_height) = (self.original_height, self.original_width);
        self.crop = self.crop.oriente(exif);
        // Le nœud était R(θ)·M^m, l'orientation R(α)·M^μ ; le nœud qui montre la même chose sur
        // les pixels droits est R(θ)·M^m·(R(α)·M^μ)⁻¹ = R(θ ± α)·M^(m+μ) — le signe de α dépend
        // des deux miroirs.
        let defaite = if miroir == self.miroir {
            -quarts
        } else {
            quarts
        };
        let compensee = match quarts_exacts(self.rotation) {
            Some(k) => {
                let reste = (k + defaite).rem_euclid(4);
                normalize(reste as f64 * FRAC_PI_2)
            }
            None => normalize(self.rotation + defaite as f64 * FRAC_PI_2),
        };
        if compensee.abs() < normalize(self.rotation).abs() {
            self.rotation = compensee;
            self.miroir ^= miroir;
        }
        true
    }
}

#[cfg(test)]
mod tests;
