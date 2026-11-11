//! **L'aspect d'une flèche** — ses couleurs, ses épaisseurs, ses disques (FLECHE-1).
//!
//! Ce sont les nombres de Glucose Tauri (`ArrowSvgLayer.tsx`), qu'il trouve *« trop
//! beaux »*, réunis en un seul endroit que les deux instruments de dessin lisent.
//!
//! # Une flèche porte la couleur de ce qu'elle relie
//!
//! Son trait passe, en dégradé, de la couleur de sa source à celle de sa cible — **celles de leurs
//! lueurs** : la famille de sens d'un texte, ou la couleur qu'on lui a choisie (PARENTE-2, fiche
//! 106) ; le gris neutre de ce dont on ne sait pas le sens (une photo, un bout dans le vide).
//! Chez Tauri, c'était une teinte de bruit (`hsl(h, 80 %, 65 %)`), que la lueur partageait : sa
//! parole du 11/11/2026, *« n'était que temporaire »*.
//!
//! Le milieu est **leur couleur médiane dans Oklch** : la clarté et la vivacité moyennes, la
//! teinte au milieu du plus court arc — de 350° à 10°, 0°, pas 180° —, chaque teinte pesant de sa
//! vivacité : un bout gris ne prête que sa clarté.
//!
//! # Trois épaisseurs, et elles sont **d'écran**
//!
//! Le trait (`strokeWidth`, 2 par défaut) et son halo (`+ 4`, `+ 10` sélectionnée) gardent
//! leur épaisseur à l'écran quel que soit le zoom — le `non-scaling-stroke` de Tauri. De
//! loin, le graphe reste lisible ; de près, la flèche reste un fil. Ses disques, eux, sont de
//! la géométrie du monde : `1,5 × strokeWidth` pour la pastille terminale, `2,5 ×` pour les
//! bouts d'une flèche sélectionnée, cerclés d'un contour de 2 pixels d'écran.

use crate::couleur::Oklch;
pub use crate::couleur::Rgb;

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

/// Les couleurs d'une flèche qui va d'un nœud de couleur `depart` à un nœud de couleur `arrivee`.
pub fn teintes(depart: Rgb, arrivee: Rgb) -> Teintes {
    let (a, b) = (Oklch::des_octets(depart), Oklch::des_octets(arrivee));
    // La teinte médiane : la somme des deux points du plan (a, b), chacun à sa vivacité.
    let (x, y) = (
        a.c * a.h.cos() + b.c * b.h.cos(),
        a.c * a.h.sin() + b.c * b.h.sin(),
    );
    let milieu = Oklch {
        l: (a.l + b.l) / 2.0,
        c: (a.c + b.c) / 2.0,
        h: y.atan2(x),
    };
    Teintes {
        depart,
        milieu: milieu.octets(),
        arrivee,
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

    /// L'écart de deux teintes, en radians.
    fn ecart(a: f64, b: f64) -> f64 {
        let e = (a - b).rem_euclid(std::f64::consts::TAU);
        e.min(std::f64::consts::TAU - e)
    }

    /// **Le milieu d'un dégradé passe par le plus court arc** : d'un rouge à un rouge voisin, il
    /// reste rouge — il ne traverse pas le cyan.
    #[test]
    fn test_fleche_1_le_milieu_prend_le_plus_court_arc() {
        let rouge = |h: f64| {
            Oklch {
                l: 0.7,
                c: 0.12,
                h: h.to_radians(),
            }
            .octets()
        };
        let m = Oklch::des_octets(teintes(rouge(10.0), rouge(50.0)).milieu);
        assert!(
            ecart(m.h, 30f64.to_radians()) < 0.05,
            "{}",
            m.h.to_degrees()
        );
        let m = Oklch::des_octets(teintes(rouge(-20.0), rouge(20.0)).milieu);
        assert!(ecart(m.h, 0.0) < 0.05, "{}", m.h.to_degrees());
    }

    /// **Les bouts sont les couleurs de ce que la flèche relie**, telles quelles ; **un bout gris ne
    /// prête que sa clarté** : le milieu garde la teinte de l'autre bout, à mi-vivacité.
    #[test]
    fn test_fleche_1_les_bouts_sont_leurs_couleurs_et_le_gris_ne_prete_que_sa_clarte() {
        let (bleu, gris) = ((96, 165, 250), (160, 160, 160));
        let t = teintes(bleu, gris);
        assert_eq!((t.depart, t.arrivee), (bleu, gris));
        let (b, g, m) = (
            Oklch::des_octets(bleu),
            Oklch::des_octets(gris),
            Oklch::des_octets(t.milieu),
        );
        assert!(ecart(m.h, b.h) < 0.05, "la teinte du bleu");
        assert!(
            (m.c - (b.c + g.c) / 2.0).abs() < 0.01,
            "à mi-vivacité : {}",
            m.c
        );
        assert!(
            (m.l - (b.l + g.l) / 2.0).abs() < 0.01,
            "à mi-clarté : {}",
            m.l
        );
        assert_eq!(teintes(bleu, bleu).milieu, bleu);
    }
}
