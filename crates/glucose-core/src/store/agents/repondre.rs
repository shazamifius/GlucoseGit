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
    /// dessous, à sa gauche ou au-dessus, glissée au-delà de chaque nœud qu'elle chevaucherait ;
    /// la plus proche des quatre, dans cet ordre à égalité (on lit de gauche à droite, de haut en
    /// bas). Les quatre, pas deux : le 27/11, à droite et dessous seulement, deux réponses sont
    /// tombées à 5 900 unités de sa note — une membrane de 6 000 de large à droite, une colonne
    /// pleine dessous, et le vide juste au-dessus. Les membranes qui entourent la cible ne gênent
    /// pas : la réponse y entre. Aucune distance à régler : l'écart est celui de tout ce qu'une IA
    /// pose (`ECART`).
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
        let (l, h) = (largeur, hauteur);
        let voies = [
            glisser(
                &obstacles,
                Rect::new(t.right() + ECART, t.top, l, h),
                |o, r| {
                    r.left = o.right() + ECART;
                },
            ),
            glisser(
                &obstacles,
                Rect::new(t.left, t.bottom() + ECART, l, h),
                |o, r| {
                    r.top = o.bottom() + ECART;
                },
            ),
            glisser(
                &obstacles,
                Rect::new(t.left - ECART - l, t.top, l, h),
                |o, r| {
                    r.left = o.left - ECART - r.width;
                },
            ),
            glisser(
                &obstacles,
                Rect::new(t.left, t.top - ECART - h, l, h),
                |o, r| {
                    r.top = o.top - ECART - r.height;
                },
            ),
        ];
        // **Aucun chevauchement neuf** : ni la réponse, ni ses membranes qui grandiront pour elle.
        // Si les quatre voies en créent un, la plus proche quand même — comme avant.
        let chaine = self.cadres_autour_de(board_id, noeud.membrane.as_deref());
        let libres: Vec<Rect> = (voies.iter().copied())
            .filter(|r| croissance_libre(&chaine, *r, &obstacles))
            .collect();
        let candidates = if libres.is_empty() {
            voies.to_vec()
        } else {
            libres
        };
        // `min_by` rend la première des plus proches : l'ordre des voies tranche l'égalité.
        let place = (candidates.into_iter())
            .min_by(|a, b| ecart(t, *a).total_cmp(&ecart(t, *b)))
            .unwrap_or(t);
        Ok(PlaceDeReponse {
            x: place.left,
            y: place.top,
            membrane: noeud.membrane.clone(),
        })
    }
}

impl Store {
    /// Les cadres de `membrane` et de chacune de celles qui la contiennent, de la plus proche à
    /// la plus large : ceux qu'`agrandir_la_membrane_pour` fera grandir.
    fn cadres_autour_de(&self, board_id: &str, membrane: Option<&str>) -> Vec<Rect> {
        let (mut cadres, mut vues) = (Vec::new(), Vec::new());
        let mut suivante = membrane.map(str::to_string);
        while let Some(m) = suivante.filter(|m| !vues.contains(m)) {
            let Ok(cadre) = self.boite_d_un_noeud(board_id, &m) else {
                break;
            };
            cadres.push(cadre);
            suivante = (self.project.annotation(board_id, &m))
                .and_then(crate::types::Annotation::membrane_id)
                .map(str::to_string);
            vues.push(m);
        }
        cadres
    }
}

/// **Les membranes peuvent-elles grandir pour `r` sans rien recouvrir de neuf ?** Chacune grandit
/// comme `agrandir_la_membrane_pour` le fera (la loi du cadre, `cadre_autour`) ; un obstacle
/// qu'elle ne touchait pas et que son cadre grandi toucherait la ferme. Ceux qu'elle touchait déjà
/// — son contenu, ou un désordre d'avant — ne comptent pas : on ne juge que ce que la réponse
/// change.
fn croissance_libre(chaine: &[Rect], r: Rect, obstacles: &[Rect]) -> bool {
    let mut boite = r;
    for cadre in chaine {
        let neuf = cadre.union(Store::cadre_autour(boite));
        if neuf == *cadre {
            return true;
        }
        if (obstacles.iter()).any(|o| neuf.overlaps(*o) && !cadre.overlaps(*o)) {
            return false;
        }
        boite = neuf;
    }
    true
}

/// **L'écart entre deux boîtes** : la largeur du vide qui les sépare, nul si elles se touchent.
fn ecart(a: Rect, b: Rect) -> f64 {
    let horizontal = (b.left - a.right()).max(a.left - b.right());
    let vertical = (b.top - a.bottom()).max(a.top - b.bottom());
    horizontal.max(vertical).max(0.0)
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
