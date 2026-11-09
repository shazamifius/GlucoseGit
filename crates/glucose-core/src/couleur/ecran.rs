//! **Ce que l'écran sait montrer** (fiche 99) : le retour d'Oklab vers sRGB, le bord du gamut — la
//! plus forte chroma qu'un écran sRGB montre à une clarté et une teinte données —, et le papier
//! teinté qui en découle.
//!
//! La moitié aller (sRGB → Oklab) servait à ranger les images par leur couleur (ORDONNER-2). La
//! couleur d'un texte, et demain la roue de couleur, demandent le retour : choisir une teinte et une
//! clarté perçues, puis savoir quels pixels les montrent.

use super::{oklab, Oklch};

/// Une lumière ramenée au canal sRGB qui la représente (IEC 61966-2-1) : l'inverse de la
/// linéarisation du module parent.
fn gamma(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// **Une couleur Oklab en sRGB** (`0..=1` par canal, **hors de l'écran** si un canal en sort), par
/// les matrices inverses d'Ottosson : l'aller-retour avec [`oklab`] rend la couleur à 10⁻⁶ près.
pub fn srgb([l, a, b]: [f64; 3]) -> [f64; 3] {
    let l_ = (l + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m_ = (l - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s_ = (l - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);
    [
        4.076_741_662_1 * l_ - 3.307_711_591_3 * m_ + 0.230_969_929_2 * s_,
        -1.268_438_004_6 * l_ + 2.609_757_401_1 * m_ - 0.341_319_396_5 * s_,
        -0.004_196_086_3 * l_ - 0.703_418_614_7 * m_ + 1.707_614_701_0 * s_,
    ]
    .map(gamma)
}

impl Oklch {
    /// La couleur sRGB d'une couleur sRGB en octets.
    pub fn des_octets((r, g, b): (u8, u8, u8)) -> Self {
        let [l, a, b] = oklab([r, g, b].map(|c| f64::from(c) / 255.0));
        Self {
            l,
            c: a.hypot(b),
            h: b.atan2(a),
        }
    }

    /// **En sRGB**, chaque canal dans `0..=1` s'il est sur l'écran.
    pub fn srgb(self) -> [f64; 3] {
        srgb([self.l, self.c * self.h.cos(), self.c * self.h.sin()])
    }

    /// **La plus grande chroma que l'écran sait montrer** à cette clarté et cette teinte : le bord
    /// du gamut sRGB, trouvé par bissection — la chroma où un canal quitte `0..=1`. Cinquante pas
    /// la fixent bien au-delà du huit bits ; zéro pour le noir et le blanc.
    pub fn chroma_maximale(l: f64, h: f64) -> f64 {
        let sur_l_ecran = |c: f64| {
            Oklch { l, c, h }
                .srgb()
                .iter()
                .all(|v| (-1e-9..=1.0 + 1e-9).contains(v))
        };
        let (mut dedans, mut dehors) = (0.0, 0.5);
        if !sur_l_ecran(0.0) {
            return 0.0;
        }
        for _ in 0..50 {
            let milieu = (dedans + dehors) / 2.0;
            if sur_l_ecran(milieu) {
                dedans = milieu;
            } else {
                dehors = milieu;
            }
        }
        dedans
    }

    /// **En octets sRGB**, arrondis au plus proche.
    pub fn octets(self) -> (u8, u8, u8) {
        let [r, g, b] = self
            .srgb()
            .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
        (r, g, b)
    }
}

/// **Un papier teinté sans changer sa clarté** (fiche 99) : la clarté perçue de `papier`, la
/// teinte de `couleur`, et sa chroma — ramenée, s'il le faut, à la plus grande que l'écran montre à
/// cette clarté. Ce qui s'écrit sur le papier se lit exactement aussi bien qu'avant : la clarté est
/// ce qui fait le contraste, et elle ne bouge pas. Aucune opacité n'est à choisir.
pub fn papier_teinte(papier: (u8, u8, u8), couleur: (u8, u8, u8)) -> (u8, u8, u8) {
    let (p, c) = (Oklch::des_octets(papier), Oklch::des_octets(couleur));
    Oklch {
        l: p.l,
        c: c.c.min(Oklch::chroma_maximale(p.l, c.h)),
        h: c.h,
    }
    .octets()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **L'aller-retour rend la couleur** : chaque octet sRGB, par Oklab et retour, à 10⁻⁶ près.
    #[test]
    fn ecran_1_l_aller_retour_d_oklab_rend_la_couleur() {
        for (r, g, b) in [
            (0, 0, 0),
            (255, 255, 255),
            (244, 114, 182),
            (20, 20, 20),
            (96, 165, 250),
            (251, 146, 60),
        ] {
            let retour = Oklch::des_octets((r, g, b)).srgb();
            for (canal, attendu) in retour.iter().zip([r, g, b]) {
                assert!(
                    (canal * 255.0 - f64::from(attendu)).abs() < 1e-4,
                    "{r},{g},{b} → {retour:?}"
                );
            }
        }
    }

    /// **Le bord du gamut est un bord** : à la chroma maximale, la couleur est sur l'écran ; un
    /// millième au-delà, elle en sort — pour des clartés et des teintes tirées sur tout le cercle.
    #[test]
    fn ecran_1_la_chroma_maximale_est_le_bord_de_l_ecran() {
        for i in 1..20 {
            let l = f64::from(i) / 20.0;
            for j in 0..36 {
                let h = f64::from(j) * std::f64::consts::TAU / 36.0 - std::f64::consts::PI;
                let c = Oklch::chroma_maximale(l, h);
                let dedans = Oklch { l, c, h }.srgb();
                assert!(
                    dedans.iter().all(|v| (-1e-6..=1.0 + 1e-6).contains(v)),
                    "L {l} h {h} c {c}"
                );
                let dehors = Oklch { l, c: c + 1e-3, h }.srgb();
                assert!(
                    dehors.iter().any(|v| !(0.0..=1.0).contains(v)),
                    "L {l} h {h} : {c} n'est pas le bord"
                );
            }
        }
    }

    /// **Le papier teinté garde sa clarté et prend la teinte** : le texte s'y lit pareil.
    #[test]
    fn ecran_1_le_papier_teinte_garde_sa_clarte_et_prend_la_teinte() {
        let papier = (21, 21, 23);
        for couleur in [
            (244, 114, 182),
            (251, 146, 60),
            (96, 165, 250),
            (52, 211, 153),
        ] {
            let teinte = papier_teinte(papier, couleur);
            let (p, t, c) = (
                Oklch::des_octets(papier),
                Oklch::des_octets(teinte),
                Oklch::des_octets(couleur),
            );
            assert!(
                (t.l - p.l).abs() < 0.01,
                "la clarté du papier : {} contre {}",
                t.l,
                p.l
            );
            assert!(t.a_une_teinte(), "{couleur:?} se voit : chroma {}", t.c);
            let ecart = (t.h - c.h + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
                - std::f64::consts::PI;
            assert!(
                ecart.abs() < 0.15,
                "{couleur:?} : la teinte {} contre {}",
                t.h,
                c.h
            );
        }
    }
}
