//! **La flèche sous la souris, et ce qu'elle désigne** (FLECHE-4).
//!
//! Tauri faisait briller, au survol d'une flèche, les passages qu'elle ancre dans ses deux
//! cartes (`glucose:hover-arrow`). Ici la souris suit la flèche qu'elle survole ; l'image
//! demande ensuite quels passages éclairer — calculés au dessin, jamais gardés : une édition
//! de la carte déplace le passage, et rien n'est à invalider.

use crate::app::GlucoseApp;
use crate::params::Eclairage;
use crate::ui::ActiveTool;
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

    /// **Suit le nœud dont une flèche partirait**, sous l'outil Flèche armé : l'image ne se
    /// redessine que quand il change.
    pub(crate) fn suivre_le_noeud_pressenti(&mut self) {
        let pressenti = (self.ui.active_tool == ActiveTool::Arrow)
            .then(|| {
                let vp = self.store.viewport();
                let point = crate::canvas::screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
                let board = self.store.active_board()?;
                glucose_core::arrow::snap_to_nearest(board, point, &[]).node
            })
            .flatten();
        if pressenti != self.ui.noeud_pressenti {
            self.ui.noeud_pressenti = pressenti;
            self.mark_dirty();
        }
    }

    /// **Suit les cartes désignées, et rend leur vivacité à cet instant** (LUEUR-2) : ce que
    /// l'image montre, et ce qui dit au réveil qu'une lueur glisse encore.
    pub(crate) fn suivre_la_designation(&mut self) -> Vec<(String, f32)> {
        let maintenant = self.now_ms() as f64;
        let designees = self.cartes_designees();
        self.designation.suivre(&designees, maintenant);
        self.designation.vivacites(maintenant)
    }

    /// **Les cartes dont la lueur s'avive dans cette image** (LUEUR-1) — ce à quoi une flèche
    /// va se lier, ou se lie :
    ///
    /// * la flèche qui naît sous la main : ce que sa pointe vise ;
    /// * l'outil Flèche armé : le nœud dont elle partirait ;
    /// * une flèche survolée : ceux de ses bouts qui ne désignent pas de passage — un passage
    ///   ancré brille à sa place, lui seul.
    pub(crate) fn cartes_designees(&self) -> Vec<String> {
        let Some(board) = self.store.active_board() else {
            return Vec::new();
        };
        if let Some(session) = &self.draw_session {
            let visee = match self.store.project.annotation(&board.id, &session.id) {
                Some(Annotation::Arrow { target_id, .. }) => target_id.clone(),
                _ => None,
            };
            return visee.into_iter().collect();
        }
        // L'outil rendu, ce qu'il pressentait ne vaut plus — même avant que la souris bouge.
        if self.ui.active_tool == ActiveTool::Arrow {
            return self.ui.noeud_pressenti.iter().cloned().collect();
        }
        let Some(Annotation::Arrow {
            source_id,
            target_id,
            source_text_sel,
            target_text_sel,
            ..
        }) = self
            .ui
            .fleche_survolee
            .as_ref()
            .and_then(|id| self.store.project.annotation(&board.id, id))
        else {
            return Vec::new();
        };
        [(source_id, source_text_sel), (target_id, target_text_sel)]
            .into_iter()
            .filter(|(_, passage)| {
                !glucose_core::text_anchors::has_text_selection(passage.as_ref())
            })
            .filter_map(|(bout, _)| bout.clone())
            .collect()
    }

    /// **Les passages à faire briller dans cette image** : ceux que la flèche survolée ancre,
    /// dans sa source et dans sa cible, chacun à la couleur de sa carte.
    pub(crate) fn eclairages(&self) -> Vec<Eclairage> {
        // L'éditeur d'ancres ouvert, c'est ce qu'on choisit qui brille.
        if let Some(choisi) = self.eclairages_de_l_ancrage() {
            return choisi;
        }
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
