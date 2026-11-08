//! **Disposer une membrane en colonnes** (COLONNES-1, fiche 92) : l'arbre de ce qu'elle porte, lu
//! dans le document ; la disposition, pure ([`crate::layout::colonnes`]) ; puis la boîte finale de
//! chaque nœud, posée telle quelle.
//!
//! **L'arbre ne change pas** : l'appartenance est écrite (`membrane_id`, MEMB-1), et chaque nœud
//! reste dans sa membrane, qui le contient toujours. Rien ne se rattache, rien ne se recalcule par
//! la géométrie. Les flèches ne se rangent pas : un bout accroché suit son nœud.
//!
//! **L'ordre de lecture** est celui où les nœuds se trouvent : de haut en bas, puis de gauche à
//! droite — un article posé par un agent, empilé, se lit ainsi dans l'ordre où il a été écrit.
//!
//! Le geste n'est pas ouvert ici : l'humain l'enveloppe dans le sien, un agent dans
//! `geste_d_agent` (AGENT-1). Un seul `Ctrl+Z` défait tout.

use super::Store;
use crate::geometry::Rect;
use crate::layout::colonnes::{disposer, Bloc, Marges};
use crate::membrane_titre::porte_un_titre;
use crate::types::Annotation;

impl Store {
    /// **Dispose en colonnes ce que porte la membrane**, ses sous-membranes comprises, pour que la
    /// sienne approche `proportion` (largeur sur hauteur, celle de l'écran). Son coin haut-gauche ne
    /// bouge pas. Rend le nombre de nœuds rangés ; zéro si elle ne porte rien, ou n'existe pas.
    pub fn disposer_en_colonnes(
        &mut self,
        board_id: &str,
        membrane: &str,
        proportion: f64,
    ) -> usize {
        let Some(cadre) = self
            .project
            .annotation(board_id, membrane)
            .and_then(Annotation::rect)
        else {
            return 0;
        };
        let Some(bloc) = self.bloc_de(board_id, membrane, cadre) else {
            return 0;
        };
        if bloc.enfants.is_empty() || !proportion.is_finite() || proportion <= 0.0 {
            return 0;
        }
        let marges = Marges {
            ecart: super::agents::ECART,
            titre: super::agents::TITRE_DE_MEMBRANE,
        };
        let boites = disposer(&bloc, proportion, marges);
        let n = boites.len() - 1;
        for (id, r) in boites {
            let r = Rect::new(r.left + cadre.left, r.top + cadre.top, r.width, r.height);
            self.poser_la_boite(board_id, &id, r);
        }
        n
    }

    /// **Les membranes qu'un « Disposer en colonnes » range**, sur le tableau regardé : celles qui
    /// sont choisies, sinon celles qui portent les nœuds choisis — comme `F` cadre ce qu'on désigne.
    /// Vide si rien de choisi n'est ni une membrane ni dans une membrane.
    pub fn membranes_de_la_selection(&self) -> Vec<String> {
        let board = &self.project.active_board_id;
        let annotation = |id: &String| self.project.annotation(board, id);
        let choisies: Vec<String> = self
            .selected_annotation_ids
            .iter()
            .filter(|id| matches!(annotation(id), Some(Annotation::Membrane { .. })))
            .cloned()
            .collect();
        if !choisies.is_empty() {
            return choisies;
        }
        let mut porteuses: Vec<String> = Vec::new();
        let des_annotations = self
            .selected_annotation_ids
            .iter()
            .filter_map(|id| annotation(id).and_then(Annotation::membrane_id));
        let des_images = self
            .selected_image_ids
            .iter()
            .filter_map(|id| self.image(board, id).and_then(|i| i.membrane_id.as_deref()));
        for m in des_annotations.chain(des_images) {
            if !porteuses.iter().any(|p| p == m) {
                porteuses.push(m.to_string());
            }
        }
        porteuses
    }

    /// **Le bloc d'un nœud** : sa boîte ; pour une membrane, ce qu'elle porte, dans l'ordre de lecture.
    fn bloc_de(&self, board_id: &str, id: &str, boite: Rect) -> Option<Bloc> {
        let board = self.project.boards.iter().find(|b| b.id == board_id)?;
        let mut enfants: Vec<(Rect, String, bool)> = board
            .annotations
            .iter()
            .filter(|a| a.membrane_id() == Some(id) && !matches!(a, Annotation::Arrow { .. }))
            .map(|a| {
                let membrane = matches!(a, Annotation::Membrane { .. });
                (a.bounds(), a.id().to_string(), membrane)
            })
            .chain(
                board
                    .images
                    .iter()
                    .filter(|i| i.membrane_id.as_deref() == Some(id))
                    .map(|i| (i.rect(), i.id.clone(), false)),
            )
            .collect();
        enfants.sort_by(|a, b| {
            a.0.top
                .total_cmp(&b.0.top)
                .then(a.0.left.total_cmp(&b.0.left))
        });
        let enfants = enfants
            .into_iter()
            .map(|(r, e, membrane)| match membrane {
                true => self.bloc_de(board_id, &e, r),
                false => Some(feuille(e, r)),
            })
            .collect::<Option<Vec<Bloc>>>()?;
        let titree = match self.project.annotation(board_id, id) {
            Some(Annotation::Membrane { text, .. }) => porte_un_titre(text.as_deref()),
            _ => false,
        };
        Some(Bloc {
            id: id.to_string(),
            largeur: boite.width,
            hauteur: boite.height,
            enfants,
            titree,
        })
    }

    /// Pose ce nœud dans cette boîte : une membrane en prend la taille, un autre nœud y est déplacé.
    fn poser_la_boite(&mut self, board_id: &str, id: &str, r: Rect) {
        if let Some(i) = self.image(board_id, id) {
            let avant = i.rect();
            let (dx, dy) = (r.left - avant.left, r.top - avant.top);
            self.update_image(board_id, id, |i| {
                i.x += dx;
                i.y += dy;
            });
            return;
        }
        self.update_annotation(board_id, id, |a| match a {
            Annotation::Membrane {
                x,
                y,
                width,
                height,
                ..
            } => (*x, *y, *width, *height) = (r.left, r.top, r.width, r.height),
            autre => {
                let avant = autre.bounds();
                autre.translate(r.left - avant.left, r.top - avant.top);
            }
        });
    }
}

fn feuille(id: String, r: Rect) -> Bloc {
    Bloc {
        id,
        largeur: r.width,
        hauteur: r.height,
        enfants: Vec::new(),
        titree: false,
    }
}

#[cfg(test)]
mod tests;
