//! **La flèche sous la souris, et ce qu'elle désigne** (FLECHE-4).
//!
//! Tauri faisait briller, au survol d'une flèche, les passages qu'elle ancre dans ses deux
//! cartes (`glucose:hover-arrow`). Ici la souris suit la flèche qu'elle survole ; l'image
//! demande ensuite quels passages éclairer — calculés au dessin, jamais gardés : une édition
//! de la carte déplace le passage, et rien n'est à invalider.

use crate::app::GlucoseApp;
use crate::params::Eclairage;
use glucose_core::hit_priority::PickKind;
use glucose_core::text_anchors::resolve_text_sel;
use glucose_core::types::Annotation;

impl GlucoseApp {
    /// **Suit la flèche survolée** : l'image ne se redessine que quand elle change.
    pub(crate) fn suivre_la_fleche_survolee(&mut self) {
        let survolee = self.fleche_sous_la_souris();
        if survolee != self.ui.fleche_survolee {
            self.ui.fleche_survolee = survolee;
            self.mark_dirty();
        }
    }

    /// La flèche que le clic prendrait sous la souris — l'arbitre de clic est seul juge de ce
    /// qu'elle vise, pour que ce qui brille soit ce qu'un clic sélectionnerait.
    fn fleche_sous_la_souris(&self) -> Option<String> {
        let vp = self.store.viewport();
        let (wx, wy) = crate::canvas::screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        let candidat = self.pick_candidate_at(wx, wy)?;
        (candidat.kind == PickKind::Arrow).then_some(candidat.id)
    }

    /// **Les passages à faire briller dans cette image** : ceux que la flèche survolée ancre,
    /// dans sa source et dans sa cible, chacun à la couleur de sa carte.
    pub(crate) fn eclairages(&self) -> Vec<Eclairage> {
        let (Some(id), Some(board)) = (&self.ui.fleche_survolee, self.store.active_board()) else {
            return Vec::new();
        };
        let Some(Annotation::Arrow {
            source_id,
            target_id,
            source_text_sel,
            target_text_sel,
            ..
        }) = self.store.project.annotation(&board.id, id)
        else {
            return Vec::new();
        };
        [(source_id, source_text_sel), (target_id, target_text_sel)]
            .into_iter()
            .filter_map(|(carte, passage)| {
                let carte = carte.as_ref()?;
                let texte = self
                    .store
                    .project
                    .annotation(&board.id, carte)?
                    .own_text()?;
                let plages: Vec<(usize, usize)> = resolve_text_sel(&texte, passage.as_ref())
                    .into_iter()
                    .map(|p| (p.start, p.end))
                    .collect();
                (!plages.is_empty()).then(|| Eclairage {
                    carte: carte.clone(),
                    plages,
                    teinte: None,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
