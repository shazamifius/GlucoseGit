//! **Un mur est un seul obstacle** (FLECHE-6).
//!
//! Des boîtes dont les marges se chevauchent ne laissent passer aucune flèche entre elles : elles
//! font un seul obstacle, leur **amas**. La recherche découvrait une mosaïque serrée photo par
//! photo, cherchant à chaque tour à se faufiler entre deux photos qu'elle ne connaissait pas
//! encore — dix mille photos, 132 secondes. Quand un obstacle est découvert, son amas entier se
//! ramasse d'un coup, de voisin en voisin, par l'index.
//!
//! Et un chemin ne tourne qu'aux **coins extérieurs** d'un amas. Un coin qui tombe dans la marge
//! d'une voisine — ou sur son bord — n'est pas un coin convexe de leur union : le long du bord
//! d'un mur, l'union est droite ; au pied de deux photos superposées, elle rentre. Pour un mur
//! serré, il ne reste que ses quatre coins. Le coin de chaque boîte se décide au moment où elle
//! est ramassée : ses voisines sont exactement ce que l'index rend autour d'elle.
//!
//! Le seul cas perdu : deux boîtes qui ne se touchent que par un coin. Le passage entre elles est
//! de largeur nulle ; une flèche qui y passerait les toucherait toutes deux.
//!
//! # Seul le pourtour compte
//!
//! Une boîte dont les quatre bords sont recouverts par ses voisines est **dans** le mur : elle n'a
//! aucun coin extérieur, et un segment venu du dehors rencontre une boîte du pourtour avant de
//! l'atteindre. Elle n'est ni gardée ni étendue : le ramassage longe le pourtour, et son coût suit
//! le périmètre du mur, plus son aire. Mesuré : ramasser toute l'aire épuisait le budget dès
//! cent photos de côté.

use super::{Boite, Point, Recherche, BUDGET, ECART, LECTURE};
use crate::geometry::Rect;

impl Boite {
    /// Les deux boîtes se touchent-elles, bords compris ?
    pub(super) fn touche(&self, o: &Boite) -> bool {
        self.x0 <= o.x1 && o.x0 <= self.x1 && self.y0 <= o.y1 && o.y0 <= self.y1
    }

    /// Le point est-il dans la boîte **ou sur son bord**, au bruit près ?
    pub(super) fn couvre(&self, p: Point) -> bool {
        let e = self.bruit();
        p.0 >= self.x0 - e && p.0 <= self.x1 + e && p.1 >= self.y0 - e && p.1 <= self.y1 + e
    }

    /// La boîte qui englobe les deux.
    pub(super) fn union(&self, o: &Boite) -> Boite {
        Boite {
            x0: self.x0.min(o.x0),
            y0: self.y0.min(o.y0),
            x1: self.x1.max(o.x1),
            y1: self.y1.max(o.y1),
        }
    }

    /// Une clé exacte, pour ne ramasser une boîte qu'une fois.
    pub(super) fn cle(&self) -> [u64; 4] {
        [self.x0, self.y0, self.x1, self.y1].map(f64::to_bits)
    }

    /// La zone où chercher ce qui touche la boîte : une boîte d'origine touche la boîte gonflée
    /// si, gonflée à son tour, elle la touche.
    fn voisinage(&self) -> Rect {
        Rect::new(
            self.x0 - ECART,
            self.y0 - ECART,
            self.x1 - self.x0 + 2.0 * ECART,
            self.y1 - self.y0 + 2.0 * ECART,
        )
    }
}

/// **Un bord de `x` est-il à découvert ?** Pour chaque côté, les voisines qui recouvrent ce qui
/// est juste au-delà de la ligne du bord donnent des intervalles ; le bord est couvert si leur
/// union couvre toute sa longueur.
pub(super) fn a_un_bord_expose(x: &Boite, voisines: &[Boite]) -> bool {
    // (bord horizontal ?, ligne du bord, le dehors est-il du côté des coordonnées basses ?)
    let bords = [
        (true, x.y0, true),
        (true, x.y1, false),
        (false, x.x0, true),
        (false, x.x1, false),
    ];
    bords.iter().any(|&(horizontal, ligne, en_bas)| {
        let (lo, hi) = if horizontal {
            (x.x0, x.x1)
        } else {
            (x.y0, x.y1)
        };
        let mut morceaux: Vec<(f64, f64)> = voisines
            .iter()
            .filter_map(|o| {
                let (a0, a1, b0, b1) = if horizontal {
                    (o.y0, o.y1, o.x0, o.x1)
                } else {
                    (o.x0, o.x1, o.y0, o.y1)
                };
                let au_dela = if en_bas {
                    a0 < ligne && ligne <= a1
                } else {
                    a0 <= ligne && ligne < a1
                };
                au_dela.then_some((b0, b1))
            })
            .collect();
        morceaux.sort_by(|p, q| p.0.total_cmp(&q.0));
        let mut atteint = lo;
        for (debut, fin) in morceaux {
            if debut > atteint {
                break;
            }
            atteint = atteint.max(fin);
        }
        atteint < hi
    })
}

impl Recherche<'_> {
    /// **Ramasse l'amas de cette boîte** : de voisine en voisine, le long de son pourtour, tout
    /// ce qui la touche, sauf ce qui contient un bout. Chaque boîte du pourtour dit ses coins
    /// extérieurs. Rend rien si le budget s'épuise.
    pub(super) fn amasser(&mut self, depart: Boite) -> Option<()> {
        let debut = self.connues.len();
        let mut englobante: Option<Boite> = None;
        let mut pile = vec![depart];
        self.vues.insert(depart.cle());
        let mut voisines = Vec::new();
        while let Some(x) = pile.pop() {
            self.tampon.clear();
            (self.requete)(x.voisinage(), &mut self.tampon);
            self.travail += (self.tampon.len() + 1) * LECTURE;
            voisines.clear();
            let (a, b) = (self.a, self.b);
            voisines.extend(
                self.tampon
                    .drain(..)
                    .map(|r| Boite::de(r, ECART))
                    .filter(|o| *o != x && o.touche(&x) && !o.contient(a) && !o.contient(b)),
            );
            // Dans le mur : ni gardée, ni étendue.
            if !a_un_bord_expose(&x, &voisines) {
                continue;
            }
            for coin in x.coins() {
                if !voisines.iter().any(|o| o.couvre(coin)) {
                    self.coins.push(coin);
                }
            }
            self.connues.push(x);
            englobante = Some(englobante.map_or(x, |e| e.union(&x)));
            for o in &voisines {
                if self.vues.insert(o.cle()) {
                    pile.push(*o);
                }
            }
            if self.travail > BUDGET {
                return None;
            }
        }
        if let Some(englobante) = englobante {
            self.amas.push((englobante, debut..self.connues.len()));
        }
        Some(())
    }
}
