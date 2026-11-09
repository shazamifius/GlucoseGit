//! **La roue de couleur** (ROUE-1, fiche 99) : une couleur choisie par sa clarté perçue, sa teinte
//! et sa saturation — la roue « à la Blender », mais dans Oklch.
//!
//! # Ce que Blender fait, et ce qu'on lui reprend
//!
//! Le sélecteur de Blender (le manuel, « Color Picker ») : un champ de deux composantes — un cercle
//! teinte × saturation —, un curseur pour la troisième (la valeur ou la clarté), les champs RVB/TSV
//! et hexadécimal, la pipette. `Ctrl`-glisser cale la teinte tous les 30°, `Maj`-glisser affine, la
//! molette règle la clarté, `Retour arrière` rend la valeur par défaut. Ce cercle est en **TSV** : un
//! jaune et un bleu de même « valeur » n'ont pas du tout la même clarté à l'œil, et les teintes n'y
//! sont pas réparties comme l'œil les distingue (Ottosson, *Okhsv and Okhsl*, 2021).
//!
//! # Une roue exacte, sans approximation
//!
//! À la clarté `L` du curseur, l'angle est la **teinte d'Oklch** — des angles égaux, des écarts que
//! l'œil voit égaux —, et le rayon `s` la **part de la plus forte chroma que l'écran montre à cette
//! clarté et cette teinte** ([`Oklch::chroma_maximale`]). Le bord du disque est donc toujours le
//! bord de l'écran : chaque point du disque est une couleur qu'on peut afficher, et la clarté ne
//! bouge pas quand on tourne. C'est l'idée d'Okhsl (la clarté orthogonale) — calculée ici
//! exactement, par la bissection du bord, là où Okhsl en donne une approximation lisse.

use super::Oklch;
use std::f64::consts::{PI, TAU};

/// Une place dans la roue : la clarté (le curseur), la teinte (l'angle, en radians) et la
/// saturation (le rayon, de 0 au bord de l'écran).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaceDansLaRoue {
    pub clarte: f64,
    pub teinte: f64,
    pub saturation: f64,
}

impl PlaceDansLaRoue {
    /// **La couleur de cette place** : toujours sur l'écran.
    pub fn couleur(self) -> Oklch {
        let l = self.clarte.clamp(0.0, 1.0);
        let s = self.saturation.clamp(0.0, 1.0);
        Oklch {
            l,
            c: s * Oklch::chroma_maximale(l, self.teinte),
            h: self.teinte,
        }
    }

    /// **La place d'une couleur** ; un gris garde la teinte `teinte_d_avant` — il n'en a pas, et la
    /// roue ne doit pas sauter au rouge quand la saturation passe par zéro.
    pub fn de(couleur: Oklch, teinte_d_avant: f64) -> Self {
        let maximale = Oklch::chroma_maximale(couleur.l, couleur.h);
        let gris = couleur.c < 1e-6 || maximale < 1e-9;
        Self {
            clarte: couleur.l,
            teinte: if gris { teinte_d_avant } else { couleur.h },
            saturation: if gris {
                0.0
            } else {
                (couleur.c / maximale).min(1.0)
            },
        }
    }

    /// La teinte calée sur le plus proche multiple de 30° — `Ctrl`, comme dans Blender : les douze
    /// teintes d'une roue de peintre.
    pub fn calee(self) -> Self {
        let pas = PI / 6.0;
        Self {
            teinte: (self.teinte / pas).round() * pas,
            ..self
        }
    }
}

/// **La place sous un point du disque** : `(dx, dy)` depuis son centre, en fractions de son rayon
/// (le haut de l'écran vers les `dy` négatifs). La teinte zéro d'Oklch — un rose rouge — est à
/// droite, et les teintes tournent dans le sens inverse des aiguilles d'une montre, comme les
/// angles du plan `(a, b)`.
pub fn place_sous(dx: f64, dy: f64, clarte: f64) -> PlaceDansLaRoue {
    PlaceDansLaRoue {
        clarte,
        teinte: (-dy).atan2(dx).rem_euclid(TAU),
        saturation: dx.hypot(dy).min(1.0),
    }
}

/// Le point du disque d'une place, l'inverse de [`place_sous`].
pub fn point_de(place: PlaceDansLaRoue) -> (f64, f64) {
    let r = place.saturation.clamp(0.0, 1.0);
    (r * place.teinte.cos(), -r * place.teinte.sin())
}

/// **Les pixels du disque** à cette clarté, `diametre` de côté, ligne après ligne : la couleur de
/// chaque point, en sRGB, et sa couverture par le disque (le bord est anticrénelé par la distance
/// au cercle — exacte pour un bord franc). Hors du disque, une couverture nulle.
///
/// Le bord de l'écran se calcule une fois par teinte, non par pixel : autant de teintes que de
/// pixels sur le tour du disque, et chaque pixel lit celle de son angle — la plus proche, à moins
/// d'un pixel de tour, donc invisible.
pub fn pixels_du_disque(clarte: f64, diametre: u32) -> Vec<[u8; 4]> {
    let d = f64::from(diametre);
    let rayon = d / 2.0;
    let tour = (TAU * rayon).ceil().max(1.0) as usize;
    let bords: Vec<f64> = (0..tour)
        .map(|i| Oklch::chroma_maximale(clarte, i as f64 * TAU / tour as f64))
        .collect();
    let mut pixels = Vec::with_capacity((diametre * diametre) as usize);
    for y in 0..diametre {
        for x in 0..diametre {
            let (dx, dy) = (f64::from(x) + 0.5 - rayon, f64::from(y) + 0.5 - rayon);
            let distance = dx.hypot(dy);
            let couverture = (rayon - distance + 0.5).clamp(0.0, 1.0);
            if couverture == 0.0 {
                pixels.push([0, 0, 0, 0]);
                continue;
            }
            let place = place_sous(dx / rayon, dy / rayon, clarte);
            let rang = ((place.teinte / TAU * tour as f64).round() as usize) % tour;
            let couleur = Oklch {
                l: clarte,
                c: place.saturation * bords[rang],
                h: place.teinte,
            };
            let (r, g, b) = couleur.octets();
            pixels.push([r, g, b, (couverture * 255.0).round() as u8]);
        }
    }
    pixels
}

/// **Une couleur en `#rrggbb`** — ce que le document écrit.
pub fn hex((r, g, b): (u8, u8, u8)) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Une couleur retrouve sa place, et la place sa couleur** : l'aller-retour d'une couleur de
    /// l'écran par la roue rend ses octets, à un niveau près.
    #[test]
    fn roue_1_une_couleur_retrouve_sa_place_et_sa_couleur() {
        for (r, g, b) in [
            (244, 114, 182),
            (96, 165, 250),
            (251, 146, 60),
            (52, 211, 153),
            (20, 20, 20),
        ] {
            let place = PlaceDansLaRoue::de(Oklch::des_octets((r, g, b)), 0.0);
            let (r2, g2, b2) = place.couleur().octets();
            for (a, b) in [(r, r2), (g, g2), (b, b2)] {
                assert!(
                    a.abs_diff(b) <= 1,
                    "({r},{g},{b}) → {place:?} → ({r2},{g2},{b2})"
                );
            }
        }
    }

    /// **Le bord du disque est le bord de l'écran**, et **la clarté ne bouge pas quand on tourne** :
    /// à saturation pleine, chaque teinte est sur l'écran, à la clarté du curseur.
    #[test]
    fn roue_1_tourner_garde_la_clarte_et_le_bord_est_l_ecran() {
        for i in 0..72 {
            let teinte = f64::from(i) * TAU / 72.0;
            let couleur = PlaceDansLaRoue {
                clarte: 0.7,
                teinte,
                saturation: 1.0,
            }
            .couleur();
            assert!(couleur
                .srgb()
                .iter()
                .all(|v| (-1e-6..=1.0 + 1e-6).contains(v)));
            assert!(
                (Oklch::des_octets(couleur.octets()).l - 0.7).abs() < 0.01,
                "teinte {teinte}"
            );
        }
    }

    /// **Le point d'une place est sous elle** : le disque et le clic lisent la même géométrie.
    #[test]
    fn roue_1_le_point_d_une_place_est_sous_elle() {
        for (dx, dy) in [(0.5, 0.0), (0.0, -0.3), (-0.7, 0.2), (0.1, 0.9)] {
            let place = place_sous(dx, dy, 0.6);
            let (x, y) = point_de(place);
            assert!(
                (x - dx).abs() < 1e-12 && (y - dy).abs() < 1e-12,
                "({dx}, {dy}) → ({x}, {y})"
            );
        }
    }

    /// **`Ctrl` cale la teinte sur les douze d'une roue de peintre**, sans toucher au reste.
    #[test]
    fn roue_1_ctrl_cale_la_teinte_tous_les_trente_degres() {
        let place = PlaceDansLaRoue {
            clarte: 0.5,
            teinte: 0.6,
            saturation: 0.4,
        }
        .calee();
        assert!((place.teinte - PI / 6.0).abs() < 1e-12);
        assert_eq!((place.clarte, place.saturation), (0.5, 0.4));
    }

    /// **Le disque** : son centre est le gris de la clarté, son bord saturé, ses coins vides.
    #[test]
    fn roue_1_le_disque_va_du_gris_au_bord_de_l_ecran() {
        let d = 64;
        let pixels = pixels_du_disque(0.7, d);
        let a = |x: u32, y: u32| pixels[(y * d + x) as usize];
        assert_eq!(a(0, 0)[3], 0, "un coin est hors du disque");
        let centre = a(d / 2, d / 2);
        assert_eq!(centre[3], 255);
        assert!(
            !Oklch::des_octets((centre[0], centre[1], centre[2])).a_une_teinte(),
            "le centre est gris"
        );
        let bord = a(d - 2, d / 2);
        assert!(
            Oklch::des_octets((bord[0], bord[1], bord[2])).c > 0.1,
            "le bord est saturé"
        );
    }

    #[test]
    fn roue_1_une_couleur_s_ecrit_en_hexadecimal() {
        assert_eq!(hex((244, 114, 182)), "#f472b6");
    }
}
