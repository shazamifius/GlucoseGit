//! **DATE-1 — dater des nœuds** (fiche 113) : la temporalité de Glucose Tauri (`Maj+T`,
//! `TemporalAnchorPrompt.tsx`), dont le modèle et ses lectures (`timeline.rs`) attendaient un
//! geste. Un document de Tauri qui datait ses nœuds les gardait datés dans Glucose Rust — lus,
//! écrits, et jamais montrés.
//!
//! * **Une date** est une année, une plage, ou une ère nommée (`timeline::parse_anchor` : « 1789 »,
//!   « 1789-1799 », « -500 » ou « 500 av. J.-C. », « Renaissance », « 10 ka », « 1,5 Ma ») ; la
//!   convention des historiens, sans an zéro.
//! * **Un geste** : la poser ou la retirer sur tout ce qui est choisi, textes, post-it, flèches,
//!   membranes et images — un `Ctrl+Z` défait tout. Ce qui la porte déjà ne change pas.
//! * **Le même geste pour un agent** (`glucose_dater`) : un nom qui n'est pas un nœud du tableau
//!   est refusé, plutôt qu'ignoré — l'humain date ce qu'il a choisi, qui existe ; un agent peut se
//!   tromper de nom, et doit le savoir.

use super::agents::Refus;
use super::Store;
use crate::types::TemporalAnchor;

impl Store {
    /// **Pose `ancre` sur ces nœuds de ce tableau, ou la retire** (`None`). Ce qui la porte déjà,
    /// ou n'est pas un nœud du tableau, est laissé ; rend combien ont changé. L'appelant en fait
    /// **un** geste.
    pub fn ancrer_dans_le_temps(
        &mut self,
        board_id: &str,
        ids: &[String],
        ancre: Option<&TemporalAnchor>,
    ) -> usize {
        let mut changes = 0;
        for id in ids {
            if let Some(a) = self.project.annotation(board_id, id) {
                if a.temporal_anchor() == ancre {
                    continue;
                }
                self.update_annotation(board_id, id, |ann| {
                    *ann.temporal_anchor_mut() = ancre.cloned();
                });
            } else if let Some(img) = self.image(board_id, id) {
                if img.temporal_anchor.as_ref() == ancre {
                    continue;
                }
                self.update_image(board_id, id, |img| img.temporal_anchor = ancre.cloned());
            } else {
                continue;
            }
            changes += 1;
        }
        changes
    }

    /// **Date ces nœuds pour un agent**, ou retire leur date : le geste de `Maj+T`, après avoir
    /// vérifié que chaque nom est un nœud de ce tableau. Rend combien ont changé.
    pub fn dater_pour_un_agent(
        &mut self,
        board_id: &str,
        ids: &[String],
        ancre: Option<&TemporalAnchor>,
    ) -> Result<usize, Refus> {
        self.tableau_existe(board_id)?;
        let inconnu = |id: &&String| {
            self.project.annotation(board_id, id).is_none() && self.image(board_id, id).is_none()
        };
        if let Some(id) = ids.iter().find(inconnu) {
            return Err(Refus::NoeudInconnu(id.clone()));
        }
        Ok(self.ancrer_dans_le_temps(board_id, ids, ancre))
    }

    /// **Les dates du tableau actif** — celles de ses nœuds, puis de ses photos : ce que la frise
    /// du temps répartit et cadre (fiche 114).
    pub fn dates_du_tableau(&self) -> impl Iterator<Item = &TemporalAnchor> {
        let board = self.active_board();
        let notes = board
            .into_iter()
            .flat_map(|b| b.annotations.iter().filter_map(|a| a.temporal_anchor()));
        let images = board
            .into_iter()
            .flat_map(|b| b.images.iter().filter_map(|i| i.temporal_anchor.as_ref()));
        notes.chain(images)
    }

    /// **Les dates de ce qui est choisi**, nœuds puis images, dans l'ordre du choix : ce que
    /// l'invite propose d'abord, et ce qui dit si « Retirer la date » a un objet.
    pub fn dates_choisies(&self) -> Vec<Option<&TemporalAnchor>> {
        let board = &self.project.active_board_id;
        let notes = self
            .selected_annotation_ids
            .iter()
            .filter_map(|id| self.project.annotation(board, id))
            .map(|a| a.temporal_anchor());
        let images = self
            .selected_image_ids
            .iter()
            .filter_map(|id| self.image(board, id))
            .map(|i| i.temporal_anchor.as_ref());
        notes.chain(images).collect()
    }
}

#[cfg(test)]
mod tests;
