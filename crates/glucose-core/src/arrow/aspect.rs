//! **L'aspect d'une flèche** — ses couleurs, ses épaisseurs, ses disques (FLECHE-1).
//!
//! Ce sont les nombres de Glucose Tauri (`ArrowSvgLayer.tsx`), qu'il trouve *« trop
//! beaux »*, réunis en un seul endroit que les deux instruments de dessin lisent.
//!
//! # Une flèche porte la couleur de ce qu'elle relie
//!
//! Son trait passe, en dégradé, de la teinte de sa source à celle de sa cible, par une teinte
//! médiane un peu plus vive et plus claire. Les deux bouts sont en `hsl(h, 80 %, 65 %)`, le
//! milieu en `hsl(h̄, 85 %, 70 %)`, où `h̄` est la moyenne des deux teintes prise **par le plus
//! court chemin** sur le cercle chromatique : de 350° à 10°, le milieu est 0°, pas 180°.
//!
//! # Trois épaisseurs, et elles sont **d'écran**
//!
//! Le trait (`strokeWidth`, 2 par défaut) et son halo (`+ 4`, `+ 10` sélectionnée) gardent
//! leur épaisseur à l'écran quel que soit le zoom — le `non-scaling-stroke` de Tauri. De
//! loin, le graphe reste lisible ; de près, la flèche reste un fil. Ses disques, eux, sont de
//! la géométrie du monde : `1,5 × strokeWidth` pour la pastille terminale, `2,5 ×` pour les
//! bouts d'une flèche sélectionnée, cerclés d'un contour de 2 pixels d'écran.

use crate::symbiotic_hue::hsl_to_rgb;

/// Une couleur, en octets.
pub type Rgb = (u8, u8, u8);

/// L'épaisseur du trait quand la flèche n'en dit pas, en pixels d'écran (`strokeWidth ?? 2`).
pub const EPAISSEUR: f64 = 2.0;
/// Ce que le halo ajoute au trait, en pixels d'écran (`sw + 4`).
pub const HALO: f64 = 4.0;
/// Ce qu'il ajoute quand la flèche est sélectionnée (`sw + 10`).
pub const HALO_SELECTIONNE: f64 = 10.0;
/// L'opacité du halo (`strokeOpacity={0.18}`), et sélectionné (`0.45`).
pub const OPACITE_DU_HALO: f32 = 0.18;
pub const OPACITE_DU_HALO_SELECTIONNE: f32 = 0.45;
/// L'opacité du trait (`strokeOpacity={0.92}`).
pub const OPACITE_DU_TRAIT: f32 = 0.92;
/// Le rayon de la pastille terminale, en multiples de l'épaisseur, en unités du monde
/// (`r={sw * 1.5}`).
pub const POINTE: f64 = 1.5;
/// Le rayon des bouts d'une flèche sélectionnée, en multiples de l'épaisseur (`r={sw * 2.5}`).
pub const BOUT_SELECTIONNE: f64 = 2.5;
/// Le contour des disques, en pixels d'écran (`strokeWidth={2}`, `non-scaling-stroke`).
pub const CONTOUR: f64 = 2.0;
/// Le fond de la pastille terminale (`fill="#111"`).
pub const FOND_DE_LA_POINTE: Rgb = (0x11, 0x11, 0x11);

/// Les trois couleurs du dégradé d'une flèche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Teintes {
    pub depart: Rgb,
    pub milieu: Rgb,
    pub arrivee: Rgb,
}

/// La teinte à la fraction `t` du chemin de `h1` à `h2`, **par le plus court arc** du cercle
/// chromatique (`lerpHue` de Tauri).
pub fn entre_deux_teintes(h1: f64, h2: f64, t: f64) -> f64 {
    let mut ecart = h2 - h1;
    if ecart > 180.0 {
        ecart -= 360.0;
    }
    if ecart < -180.0 {
        ecart += 360.0;
    }
    (h1 + ecart * t).rem_euclid(360.0)
}

/// Les couleurs d'une flèche qui va d'un nœud de teinte `depart` à un nœud de teinte
/// `arrivee`, en degrés.
pub fn teintes(depart: f64, arrivee: f64) -> Teintes {
    Teintes {
        depart: hsl_to_rgb(depart, 0.80, 0.65),
        milieu: hsl_to_rgb(entre_deux_teintes(depart, arrivee, 0.5), 0.85, 0.70),
        arrivee: hsl_to_rgb(arrivee, 0.80, 0.65),
    }
}

/// Ce qui change quand une flèche est sélectionnée : l'épaisseur et l'opacité de son halo.
pub fn halo(epaisseur: f64, selectionnee: bool) -> (f64, f32) {
    if selectionnee {
        (epaisseur + HALO_SELECTIONNE, OPACITE_DU_HALO_SELECTIONNE)
    } else {
        (epaisseur + HALO, OPACITE_DU_HALO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Le milieu d'un dégradé passe par le plus court arc** : de 350° à 10°, il est à 0°,
    /// et non à 180° — un dégradé du rouge au rouge ne traverse pas le cyan.
    #[test]
    fn test_fleche_1_le_milieu_prend_le_plus_court_arc() {
        assert!((entre_deux_teintes(350.0, 10.0, 0.5) - 0.0).abs() < 1e-9);
        assert!((entre_deux_teintes(10.0, 350.0, 0.5) - 0.0).abs() < 1e-9);
        assert!((entre_deux_teintes(90.0, 210.0, 0.5) - 150.0).abs() < 1e-9);
    }

    /// **Les couleurs sont celles de Tauri** : `hsl(h, 80 %, 65 %)` aux bouts, `hsl(h̄, 85 %,
    /// 70 %)` au milieu. Valeurs de référence : l'algorithme HSL de CSS (`colorsys` de
    /// Python, arrondi à l'octet) pour `hsl(200, 80%, 65%)` et `hsl(200, 85%, 70%)`.
    #[test]
    fn test_fleche_1_les_couleurs_sont_celles_de_tauri() {
        let t = teintes(200.0, 200.0);
        assert_eq!(t.depart, (94, 190, 237));
        assert_eq!(t.arrivee, (94, 190, 237));
        assert_eq!(t.milieu, (113, 200, 244));
    }
}
