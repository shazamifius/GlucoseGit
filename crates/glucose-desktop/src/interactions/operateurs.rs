//! **OPERATEUR-1 — poser un opérateur sur les post-it choisis** (fiche 98).
//!
//! ET, OU, MAIS, PARCE QUE : la grammaire des post-it de Glucose Tauri (`Alt+1` à `Alt+4`, et
//! `Alt+0` pour revenir au papier). Le modèle les portait, le rendu les dessinait en pilules
//! (fiche 06 § 5.3) — **aucun geste ne les posait** (INVENTAIRE § 3). Ils passent par la table des
//! commandes : un raccourci, une ligne de la palette, une entrée du menu d'un post-it, la même
//! commande pour les trois (P4) ; au téléphone, sans `Alt`, le menu.

use crate::app::GlucoseApp;
use glucose_core::types::StickyOperator;

impl GlucoseApp {
    /// Pose `voulu` sur chaque post-it choisi, ou le retire — **un** geste, défait d'un seul
    /// `Ctrl+Z`. Rend `false` sans post-it choisi : l'accord passe alors à la suivante de sa liste.
    ///
    /// **Sans message** : la pilule paraît, change de couleur ou redevient papier sous les yeux de
    /// celui qui vient d'appuyer — la règle des relations des flèches (PRED-1). Tauri en disait
    /// un ; un message qui décrit ce que l'œil voit est du bruit.
    pub(crate) fn poser_l_operateur(&mut self, voulu: Option<StickyOperator>) -> bool {
        // Les **post-it** de la sélection, demandés au modèle : sans eux, la transaction
        // s'ouvrirait pour rien, et le document se dirait modifié.
        let ids: Vec<String> = self
            .store
            .selected_stickies()
            .iter()
            .map(|a| a.id().to_string())
            .collect();
        if ids.is_empty() {
            return false;
        }
        let board = self.store.project.active_board_id.clone();
        self.store.begin_live_edit();
        self.store.poser_l_operateur(&board, &ids, voulu);
        self.store.end_live_edit();
        self.mark_dirty();
        true
    }
}

#[cfg(test)]
mod tests;
