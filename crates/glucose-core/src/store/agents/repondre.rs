//! **Répondre à côté** (MCP-REPONDRE, sa liste du 24/11, `relais-rep-mcp` point 4 : *« un seul
//! geste qui pose ma réponse près de ta note, sans rien chevaucher, avec la flèche « réponse » »*) :
//! la place d'une réponse près du nœud auquel elle répond.

use super::{Refus, Sorte, ECART};
use crate::geometry::Rect;
use crate::store::Store;

/// **Où poser une réponse** : le coin haut gauche de sa boîte, et la membrane du nœud auquel elle
/// répond — elle y entre, et la membrane grandit pour elle.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaceDeReponse {
    pub x: f64,
    pub y: f64,
    pub membrane: Option<String>,
}

impl Store {
    /// **La place libre la plus proche** d'une réponse de `taille` au nœud `cible` : à sa droite,
    /// ou sous lui, glissée au-delà de chaque nœud qu'elle chevaucherait ; la plus proche des deux,
    /// la droite à égalité (on lit de gauche à droite). Les membranes qui entourent la cible ne
    /// gênent pas : la réponse y entre. Aucune distance à régler : l'écart est celui de tout ce
    /// qu'une IA pose (`ECART`).
    pub fn place_pour_une_reponse(
        &self,
        board_id: &str,
        cible: &str,
        (largeur, hauteur): (f64, f64),
    ) -> Result<PlaceDeReponse, Refus> {
        let decrits = self.decrire_pour_un_agent(board_id)?;
        let noeud = (decrits.iter())
            .find(|d| d.id == cible)
            .ok_or_else(|| Refus::NoeudInconnu(cible.to_string()))?;
        let t = noeud.boite;
        let obstacles: Vec<Rect> = (decrits.iter())
            .filter(|d| d.sorte != Sorte::Fleche)
            .filter(|d| !(d.sorte == Sorte::Membrane && d.boite.contains_rect(t)))
            .map(|d| d.boite)
            .collect();
        let a_droite = glisser(
            &obstacles,
            Rect::new(t.right() + ECART, t.top, largeur, hauteur),
            |o, r| {
                r.left = o.right() + ECART;
            },
        );
        let dessous = glisser(
            &obstacles,
            Rect::new(t.left, t.bottom() + ECART, largeur, hauteur),
            |o, r| {
                r.top = o.bottom() + ECART;
            },
        );
        let place = if dessous.top - t.bottom() < a_droite.left - t.right() {
            dessous
        } else {
            a_droite
        };
        Ok(PlaceDeReponse {
            x: place.left,
            y: place.top,
            membrane: noeud.membrane.clone(),
        })
    }
}

/// **Pousse `r` au-delà de chaque obstacle qu'il chevauche**, dans un seul sens : il n'y revient
/// jamais, et s'arrête au plus tard après les avoir tous passés.
fn glisser(obstacles: &[Rect], mut r: Rect, passer: impl Fn(&Rect, &mut Rect)) -> Rect {
    for _ in 0..=obstacles.len() {
        match obstacles.iter().find(|o| o.overlaps(r)) {
            Some(o) => passer(o, &mut r),
            None => break,
        }
    }
    r
}

#[cfg(test)]
#[path = "repondre_tests.rs"]
mod tests;
