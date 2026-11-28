//! **PLACE-1 — poser et déplacer sans chevaucher** (fiche 185 ; plan du relais, point 3 ; sa note
//! du 26/11, `text-c80i-2483`, « c'est ma priorité »). Une seule loi pour tout ce qu'une IA met
//! quelque part : répondre (fiche 183), poser un texte, une membrane, une page, déplacer un nœud.
//!
//! Avant elle, seule la réponse cherchait sa place ; un texte posé à des coordonnées tombait où on
//! le disait, sur un voisin ou à cheval sur une membrane, et le contrôleur (fiche 184) le
//! constatait après coup. Désormais, la place demandée est **un départ** : si elle est libre, le
//! nœud y va ; sinon il glisse dans les quatre directions au-delà de ce qu'il chevauche, et la
//! plus proche des quatre places libres gagne. « Libre » se dit comme le contrôleur le juge : ni
//! chevauchement, ni membrane à cheval, ni membrane qui le contiendrait à l'œil sans être la
//! sienne, ni membrane qui grandirait pour lui sur un voisin.

use super::{Refus, Sorte, ECART};
use crate::geometry::Rect;
use crate::store::membranes::Emport;
use crate::store::Store;
use crate::types::Annotation;

/// **Où le nœud habitera** — ce qui décide quelles membranes il peut avoir autour de lui.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Maison<'a> {
    /// Cette membrane, nommée : elle et celles qui la contiennent grandiront pour lui ; aucune
    /// autre ne doit le contenir à l'œil.
    Dite(&'a str),
    /// Aucune : il reste hors de toute membrane (la réponse à un nœud libre).
    Aucune,
    /// Celle où il tombe, comme la main qui lâche (MEMB-1) : une membrane qui le contient tout
    /// entier devient la sienne ; une membrane à cheval, non.
    OuIlTombe,
}

/// **La place trouvée** : le coin haut gauche, et si elle est libre. Quand aucune des quatre
/// directions ne l'est, la plus proche quand même — et l'IA le sait.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub x: f64,
    pub y: f64,
    pub libre: bool,
}

/// Les quatre directions d'un glissement, dans l'ordre qui tranche l'égalité : on lit de gauche à
/// droite, de haut en bas.
#[derive(Debug, Clone, Copy)]
pub(super) enum Voie {
    Droite,
    Dessous,
    Gauche,
    Dessus,
}

impl Voie {
    pub(super) const TOUTES: [Voie; 4] = [Voie::Droite, Voie::Dessous, Voie::Gauche, Voie::Dessus];

    /// Pousse `r` juste au-delà de `o`, dans cette direction.
    fn passer(self, o: Rect, r: &mut Rect) {
        match self {
            Voie::Droite => r.left = o.right() + ECART,
            Voie::Dessous => r.top = o.bottom() + ECART,
            Voie::Gauche => r.left = o.left - ECART - r.width,
            Voie::Dessus => r.top = o.top - ECART - r.height,
        }
    }
}

/// Un nœud qui peut gêner : sa boîte, et s'il est une membrane.
#[derive(Debug, Clone, Copy)]
struct Obstacle {
    boite: Rect,
    membrane: bool,
}

/// **Ce qui gêne une place** : les nœuds du tableau (sans les flèches, ni ce qui bouge avec le
/// nœud), et la chaîne des membranes qui grandiront pour lui.
struct Terrain {
    obstacles: Vec<Obstacle>,
    chaine: Vec<Rect>,
    /// Une membrane qui contient la place tout entière l'accueille (`Maison::OuIlTombe`).
    accueille: bool,
}

impl Terrain {
    /// `o` gêne-t-il `r` ? Tout ce qu'il chevauche ; une membrane, sauf celle qui l'accueille.
    fn gene(&self, o: &Obstacle, r: Rect) -> bool {
        o.boite.overlaps(r) && !(o.membrane && self.accueille && o.boite.contains_rect(r))
    }

    /// Glisse `r` dans la direction `voie` au-delà de chaque obstacle qu'il chevauche : il n'y
    /// revient jamais, et s'arrête au plus tard après les avoir tous passés.
    fn glisser(&self, mut r: Rect, voie: Voie) -> Rect {
        for _ in 0..=self.obstacles.len() {
            match self.obstacles.iter().find(|o| self.gene(o, r)) {
                Some(o) => voie.passer(o.boite, &mut r),
                None => break,
            }
        }
        r
    }

    /// **Les membranes peuvent-elles grandir pour `r` sans rien recouvrir de neuf ?** Chacune
    /// grandit comme `agrandir_la_membrane_pour` le fera (la loi du cadre, `cadre_autour`) ; un
    /// obstacle qu'elle ne touchait pas et que son cadre grandi toucherait la ferme. Ceux qu'elle
    /// touchait déjà — son contenu, ou un désordre d'avant — ne comptent pas : on ne juge que ce
    /// que la pose change.
    fn croissance_libre(&self, r: Rect) -> bool {
        let mut boite = r;
        for cadre in &self.chaine {
            let neuf = cadre.union(Store::cadre_autour(boite));
            if neuf == *cadre {
                return true;
            }
            let touche = |o: &Obstacle| neuf.overlaps(o.boite) && !cadre.overlaps(o.boite);
            if self.obstacles.iter().any(touche) {
                return false;
            }
            boite = neuf;
        }
        true
    }

    fn libre(&self, r: Rect) -> bool {
        !self.obstacles.iter().any(|o| self.gene(o, r)) && self.croissance_libre(r)
    }
}

impl Store {
    /// **La place libre la plus proche de `voulue`** pour un nœud qui habitera `maison` : elle-même
    /// si elle est libre ; sinon, glissée à droite, dessous, à gauche ou au-dessus au-delà de ce
    /// qu'elle chevauche, la plus proche des quatre. `bougent` : les nœuds qui ne gênent pas, parce
    /// qu'ils sont celui qu'on place — et, pour une membrane, ce qu'elle emporte.
    pub fn place_libre(
        &self,
        board_id: &str,
        voulue: Rect,
        maison: Maison<'_>,
        bougent: &[&str],
    ) -> Result<Place, Refus> {
        let departs = Voie::TOUTES.map(|v| (voulue, v));
        self.meilleure_place(board_id, departs, maison, bougent)
    }

    /// **La meilleure des quatre places**, chacune partant de son départ et glissant dans sa
    /// direction : la plus proche de son départ parmi les libres, la première à égalité ; aucune
    /// libre, la plus proche quand même.
    pub(super) fn meilleure_place(
        &self,
        board_id: &str,
        departs: [(Rect, Voie); 4],
        maison: Maison<'_>,
        bougent: &[&str],
    ) -> Result<Place, Refus> {
        let terrain = self.terrain(board_id, maison, bougent)?;
        let places = departs.map(|(d, v)| (d, terrain.glisser(d, v)));
        let distance = |(d, r): &(Rect, Rect)| (r.left - d.left).abs() + (r.top - d.top).abs();
        let plus_proche = |candidates: Vec<(Rect, Rect)>| {
            // `min_by` rend la première des plus proches : l'ordre des voies tranche l'égalité.
            (candidates.into_iter()).min_by(|a, b| distance(a).total_cmp(&distance(b)))
        };
        let libres: Vec<_> = (places.iter().copied())
            .filter(|(_, r)| terrain.libre(*r))
            .collect();
        let libre = !libres.is_empty();
        let choisie = if libre {
            plus_proche(libres)
        } else {
            plus_proche(places.to_vec())
        };
        let r = choisie.map_or(departs[0].0, |(_, r)| r);
        Ok(Place {
            x: r.left,
            y: r.top,
            libre,
        })
    }

    /// Le terrain d'une pose sur ce tableau.
    fn terrain(
        &self,
        board_id: &str,
        maison: Maison<'_>,
        bougent: &[&str],
    ) -> Result<Terrain, Refus> {
        let decrits = self.decrire_pour_un_agent(board_id)?;
        let chaine = match maison {
            Maison::Dite(m) => self.cadres_autour_de(board_id, Some(m)),
            Maison::Aucune | Maison::OuIlTombe => Vec::new(),
        };
        let emportes = self.ce_que_bougent(board_id, bougent);
        let obstacles = (decrits.iter())
            .filter(|d| d.sorte != Sorte::Fleche && !emportes.contient(&d.id))
            .filter(|d| !chaine.iter().any(|(id, _)| *id == d.id))
            .map(|d| Obstacle {
                boite: d.boite,
                membrane: d.sorte == Sorte::Membrane,
            })
            .collect();
        Ok(Terrain {
            obstacles,
            chaine: chaine.into_iter().map(|(_, cadre)| cadre).collect(),
            accueille: maison == Maison::OuIlTombe,
        })
    }

    /// Ces nœuds, et tout ce que leurs membranes emportent.
    fn ce_que_bougent(&self, board_id: &str, bougent: &[&str]) -> Emport {
        let ids: Vec<String> = bougent.iter().map(|s| s.to_string()).collect();
        (self.project.boards.iter())
            .find(|b| b.id == board_id)
            .map(|b| Emport::de(b, &ids, &ids))
            .unwrap_or_default()
    }

    /// **Un cadre jamais plus serré que son contenu** (fiche 184 § 6, fiche 185 § 4) : la boîte
    /// `voulue` pour la membrane `membrane`, agrandie de ce qu'il faut pour contenir chacun de ses
    /// nœuds selon la loi du cadre (`cadre_autour` : l'écart sur les côtés, le titre au-dessus).
    /// Le 27/11, un cadre posé à la main avec 15 d'écart au lieu de 30 a fait grandir la bulle sur
    /// ses voisines à la pose suivante : un cadre trop serré se paie plus tard.
    pub fn cadre_tenu(&self, board_id: &str, membrane: &str, voulue: Rect) -> Result<Rect, Refus> {
        let contenu = (self.decrire_pour_un_agent(board_id)?.into_iter())
            .filter(|d| d.sorte != Sorte::Fleche && d.membrane.as_deref() == Some(membrane))
            .map(|d| d.boite)
            .reduce(|a, b| a.union(b));
        Ok(contenu.map_or(voulue, |c| voulue.union(Store::cadre_autour(c))))
    }

    /// Les cadres de `membrane` et de chacune de celles qui la contiennent, de la plus proche à
    /// la plus large : ceux qu'`agrandir_la_membrane_pour` fera grandir.
    pub(super) fn cadres_autour_de(
        &self,
        board_id: &str,
        membrane: Option<&str>,
    ) -> Vec<(String, Rect)> {
        let mut cadres: Vec<(String, Rect)> = Vec::new();
        let mut suivante = membrane.map(str::to_string);
        while let Some(m) = suivante.filter(|m| !cadres.iter().any(|(id, _)| id == m)) {
            let Ok(cadre) = self.boite_d_un_noeud(board_id, &m) else {
                break;
            };
            suivante = (self.project.annotation(board_id, &m))
                .and_then(crate::types::Annotation::membrane_id)
                .map(str::to_string);
            cadres.push((m, cadre));
        }
        cadres
    }
}

#[cfg(test)]
#[path = "place_tests.rs"]
mod tests;

impl Store {
    /// **Un texte prend la hauteur de son contenu** (TEXT-FIT-1) — que l'application mesure,
    /// le noyau ne connaissant pas les polices — et la membrane qui le contient grandit avec
    /// lui : un document qu'un agent allonge ne déborde jamais de sa bulle.
    pub fn tenir_la_hauteur_pour_un_agent(
        &mut self,
        board_id: &str,
        id: &str,
        hauteur: f64,
    ) -> Result<bool, Refus> {
        let Some(a) = self.project.annotation(board_id, id) else {
            return Err(Refus::NoeudInconnu(id.to_string()));
        };
        let (Annotation::Text { height, .. }, Some(boite)) = (a, a.rect()) else {
            return Ok(true);
        };
        let membrane = a.membrane_id().map(str::to_string);
        // **Allongé sur un voisin, ou sa membrane grandie sur une voisine, l'IA le sait** (PLACE-1,
        // fiche 185 § 5) : le 27/11, le plan du relais allongé a poussé sa bulle sur une note de
        // l'humain, sans un mot. Le texte reste où il est — c'est son contenu qui a grandi.
        let grandi = Rect::new(boite.left, boite.top, boite.width, hauteur);
        let maison = membrane.as_deref().map_or(Maison::OuIlTombe, Maison::Dite);
        let libre = hauteur <= boite.height || {
            let p = self.place_libre(board_id, grandi, maison, &[id])?;
            (p.x, p.y) == (grandi.left, grandi.top)
        };
        if *height != Some(hauteur) {
            self.update_annotation(board_id, id, |a| {
                if let Annotation::Text { height, .. } = a {
                    *height = Some(hauteur);
                }
            });
        }
        if let Some(m) = membrane {
            self.agrandir_la_membrane_pour(board_id, &m, grandi)?;
        }
        Ok(libre)
    }
}
