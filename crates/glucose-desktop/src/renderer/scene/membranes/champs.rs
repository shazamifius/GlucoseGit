//! **Les champs d'une membrane** (REGION-1, fiche 105) : son sol — ses halos, puis son corps
//! opaque —, sa frontière, et la couleur qu'elle prend.
//!
//! La loi des régions et l'ordre où les champs se posent sont dans le noyau
//! ([`glucose_core::membrane_forme::Regions`]) ; ici, ce qu'une membrane de Glucose y met.
//!
//! # Le corps : le ton d'une région, opaque
//!
//! Le corps d'une membrane a le ton qu'elle aurait **seule sur le canevas** — sa teinte posée à la
//! force [`force`] sur le fond —, et il est opaque : aucune mère ne s'y mêle. La teinte et la clarté
//! de la couleur choisie restent intactes, et gardent le sens qu'il leur a donné le 9/11/2026 (la
//! proximité des domaines, l'accessibilité) ; la profondeur se lit aux frontières et aux titres.
//!
//! # Sans couleur choisie, neutre
//!
//! Une membrane sans couleur prenait le bleu de Glucose Tauri, `#60a5fa`, qui ne dit rien de ce
//! qu'elle porte : sur la carte de Wikipédia, le grand cadre de la racine devenait la surface la
//! plus vive de l'écran (fiche 104). Elle prend désormais le **gris de même clarté** — sa parole du
//! 11/11/2026, *« oui aussi »*. Ce qui n'a pas de sens ne prend pas de couleur (*« il faut que TOUT
//! ait une signification »*).

use super::{MembraneVue, MEMBRANE_GLOW};
use crate::renderer::pass::SELECTION_RING;
use glucose_core::couleur::Oklch;
use glucose_core::membrane_forme::{Arrondi, Bord, Membrane};

/// Le bleu des membranes de Glucose Tauri : la **clarté** d'une membrane sans couleur.
pub(crate) const BLEU_DE_TAURI: (u8, u8, u8) = (96, 165, 250);

/// L'opacité du fond translucide d'une membrane chez Glucose Tauri, sur 255 : avec ses deux halos,
/// ce qui fait le voile d'une membrane seule.
const MEMBRANE_FILL_ALPHA: u8 = 8;
/// L'opacité de son contour au repos, sur 255.
const MEMBRANE_BORDER_ALPHA: u8 = 115;
/// L'opacité de son contour quand elle est sélectionnée, sur 255.
const MEMBRANE_SELECTED_ALPHA: u8 = 235;

/// **La teinte d'une membrane** : sa couleur, ou le gris neutre d'une membrane qui n'en a pas — ou
/// dont la couleur ne se lit pas.
pub(crate) fn teinte_de_membrane(couleur: Option<&str>) -> (u8, u8, u8) {
    couleur
        .and_then(crate::renderer::parse_hex_rgb)
        .unwrap_or_else(sans_couleur)
}

/// **Le gris d'une membrane sans couleur** : la clarté perçue du bleu de Tauri, sans chroma.
pub(crate) fn sans_couleur() -> (u8, u8, u8) {
    Oklch {
        c: 0.0,
        ..Oklch::des_octets(BLEU_DE_TAURI)
    }
    .octets()
}

/// **La force d'une région** : la part de sa teinte que son corps pose sur le canevas — **le voile
/// d'une membrane seule chez Glucose Tauri** (ses deux halos et son fond, tous trois sur son
/// intérieur), écrit avec ses constantes, sans en ajouter : le ton exact qu'avait une membrane
/// posée sur le canevas dans la référence.
///
/// # Un nombre de ressenti, ancré à sa parole
///
/// Aucune lecture ne la borne depuis que le papier d'un texte est toujours le gris des blocs
/// (PAPIER-1) ; elle suit donc ce qu'il a regardé. Sa parole du 11/11/2026 au soir, devant les
/// halos de la parenté posés sur les régions : *« les membranes ne sont pas encore assez
/// transparentes »*. La perception lui donne raison : une grande surface veut une couleur sourde et
/// une petite marque une couleur vive (Stone ; Szafir, 2018). Mesuré sur sa carte, au fond de
/// « Eau » : la lueur cyan d'une famille garde sa vivacité (Oklab 0,055 → 0,058), le sol orangé de
/// sa région la perd de moitié (0,027 → 0,016) — la famille s'en détache 3,8 fois au lieu de 2. Ce
/// n'est pas une loi : une membrane d'une couleur poussée au bord de l'écran noie encore le halo
/// d'une teinte opposée, à toute force qui garde la carte lisible de loin.
///
/// Le journal des valeurs regardées sur la carte de Wikipédia, par la carte graphique : 23 % (la
/// fiche 104) — approuvé (11/11) ; 21,3 % (le voile posé deux fois) — la même chose à l'œil (fiche
/// 105), puis « pas encore assez transparentes » ; **11,3 % (le voile simple)** — deux fois plus
/// transparent, les régions se lisent encore de loin, les familles ressortent, et le bord de
/// chaque région, que ses halos éclairent, se dessine mieux que son corps (fiche 107) ; sa parole
/// devant les trois forces : *« c'est vraiment vraiment parfait, 11 % ! on garde »* ; 8 % et 6 %
/// — de loin, les régions s'éteignent et la toile des flèches domine (fiche 107). Le « s'éteint »
/// que la fiche 103 prêtait à 11,3 % jugeait un prototype où le papier des textes prenait encore la
/// teinte de sa région.
///
/// L'opacité d'une membrane, quand le modèle la portera (l'appartenance, sa parole du 9/11), la
/// multipliera — jamais au-dessus.
fn force() -> f32 {
    let alpha = |a: u8| f32::from(a) / 255.0;
    1.0 - (1.0 - alpha(MEMBRANE_FILL_ALPHA))
        * (1.0 - alpha(MEMBRANE_GLOW[0].1))
        * (1.0 - alpha(MEMBRANE_GLOW[1].1))
}

impl MembraneVue<'_> {
    /// **Les champs de la membrane** : son sol (ses halos, puis son corps opaque, du ton qu'elle
    /// aurait seule sur `canevas`), et sa frontière.
    pub(super) fn champs(&self, canevas: [f32; 3]) -> ([Membrane; 2], Membrane) {
        let fond = self.fond();
        let [r, g, b] = [self.teinte.0, self.teinte.1, self.teinte.2];
        let teinte = [r, g, b].map(|c| f32::from(c) / 255.0);
        let f = force();
        let corps = [0, 1, 2].map(|i| canevas[i] + f * (teinte[i] - canevas[i]));
        let halos = Membrane {
            remplissages: [
                self.halo(MEMBRANE_GLOW[0]),
                self.halo(MEMBRANE_GLOW[1]),
                (fond, 0.0),
            ],
            bord: sans_bord(fond),
            teinte,
        };
        let frontiere = Membrane {
            remplissages: [(fond, 0.0); 3],
            bord: self.bord(fond),
            teinte,
        };
        ([halos, Membrane::plein(fond, corps, 1.0)], frontiere)
    }

    /// Le rectangle arrondi de la membrane, à l'écran.
    fn fond(&self) -> Arrondi {
        let (l, (x, y)) = (&self.layout, self.coin);
        Arrondi::nouveau(x, y, l.width, l.height, l.radius)
    }

    /// Un halo : la membrane débordée de `pad_monde`, à l'opacité `alpha` sur 255.
    ///
    /// Le halo appartient au cadre, donc il le suit, à toute distance : il se cachait de loin avec
    /// le titre, et aucun titre ne se cache plus (TITRE-1, GRIS-2).
    fn halo(&self, (pad_monde, alpha): (f32, u8)) -> (Arrondi, f32) {
        let (l, (x, y)) = (&self.layout, self.coin);
        let pad = self.scale.world(pad_monde);
        let forme = Arrondi::nouveau(
            x - pad,
            y - pad,
            l.width + pad * 2.0,
            l.height + pad * 2.0,
            l.radius + pad * 0.5,
        );
        (forme, f32::from(alpha) / 255.0)
    }

    /// Le contour d'une membrane : pointillé au repos, plein quand elle est prise.
    ///
    /// # Le pointillé s'arrête où le pixel s'arrête
    ///
    /// Un tiret plus fin qu'un pixel ne se voit pas comme un tiret : l'œil n'y lit qu'un trait
    /// continu, à moitié moins dense puisque la moitié du parcours est vide. On dessine donc
    /// exactement cela — un trait plein, d'opacité moitié. Le seuil n'est pas choisi : c'est
    /// le pixel, la plus petite chose qu'un écran sache montrer.
    fn bord(&self, forme: Arrondi) -> Bord {
        let dash = self.layout.dash;
        let pointille = !self.selectionnee && dash >= 1.0;
        let (largeur, opacite) = if self.selectionnee {
            (self.scale.screen(SELECTION_RING), MEMBRANE_SELECTED_ALPHA)
        } else if pointille {
            (self.layout.border, MEMBRANE_BORDER_ALPHA)
        } else {
            (self.layout.border, MEMBRANE_BORDER_ALPHA / 2)
        };
        Bord {
            forme,
            demi_largeur: largeur / 2.0,
            pointille: pointille.then(|| forme.pointille(dash)),
            alpha: f32::from(opacite) / 255.0,
        }
    }
}

/// Un bord qui ne pose rien.
fn sans_bord(forme: Arrondi) -> Bord {
    Bord {
        forme,
        demi_largeur: 0.0,
        pointille: None,
        alpha: 0.0,
    }
}

#[cfg(test)]
mod tests;
