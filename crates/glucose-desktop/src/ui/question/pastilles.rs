//! **PASTILLES-1 — choisir une couleur dans une question** (Q19, fiche 140).
//!
//! Sa réponse du 23/11 : *« on demande le pseudo ainsi que la couleur d'affichage »*. Une couleur se
//! choisit en la voyant : dix boutons qui porteraient des noms de couleur ne la montreraient pas.
//! La question porte donc, sous son champ, une rangée de pastilles — une par couleur — ; celle qui
//! est choisie est cernée du filet de la sélection. Un clic, un doigt, ou les flèches gauche et
//! droite en changent ; Entrée répond. La cible d'un doigt est la case entière autour de la
//! pastille, pas son seul carré.

use super::{cerner, dans, Rangee};
use tiny_skia::{Color, PixmapMut, Rect};

/// **Les couleurs proposées**, et celle qui est choisie.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pastilles {
    pub couleurs: Vec<[u8; 3]>,
    pub choisie: usize,
}

impl Pastilles {
    /// La choisie avance de `pas`, et revient au début après la dernière.
    pub fn avancer(&mut self, pas: isize) {
        let n = self.couleurs.len() as isize;
        if n > 0 {
            self.choisie = (self.choisie as isize + pas).rem_euclid(n) as usize;
        }
    }
}

/// **Les pastilles, placées** : la case de chacune (la cible), sa couleur, et la choisie.
#[derive(Clone, Debug, PartialEq)]
pub struct Placees {
    pub cases: Vec<Rangee>,
    pub couleurs: Vec<[u8; 3]>,
    pub choisie: usize,
}

/// **Une case par couleur**, côte à côte sur la rangée donnée.
pub fn placer(p: &Pastilles, (gauche, haut, largeur, hauteur): Rangee) -> Placees {
    let pas = largeur / p.couleurs.len().max(1) as f32;
    let cases = (0..p.couleurs.len())
        .map(|i| (gauche + pas * i as f32, haut, pas, hauteur))
        .collect();
    Placees {
        cases,
        couleurs: p.couleurs.clone(),
        choisie: p.choisie,
    }
}

/// La pastille dont la case contient ce point, s'il y en a une.
pub fn sous(p: &Placees, px: f32, py: f32) -> Option<usize> {
    p.cases.iter().position(|c| dans(*c, px, py))
}

/// **Dessine les pastilles** : un carré plein au centre de chaque case, sur la grille de pixels ;
/// la choisie, cernée du filet de la sélection.
pub fn dessiner(pixmap: &mut PixmapMut, p: &Placees, cadre: Color, s: f32) {
    for (i, (&(x, y, w, h), [r, g, b])) in p.cases.iter().zip(&p.couleurs).enumerate() {
        let cote = (w.min(h) * 0.6).round().max(1.0);
        let (gauche, haut) = (
            (x + (w - cote) / 2.0).round(),
            (y + (h - cote) / 2.0).round(),
        );
        if let Some(carre) = Rect::from_xywh(gauche, haut, cote, cote) {
            crate::renderer::scale::fill_crisp(pixmap, carre, Color::from_rgba8(*r, *g, *b, 255));
        }
        if i == p.choisie {
            let marge = (3.0 * s).round();
            let autour = (
                gauche - marge,
                haut - marge,
                cote + 2.0 * marge,
                cote + 2.0 * marge,
            );
            cerner(pixmap, autour, 0.0, cadre);
        }
    }
}
