//! **« Disposer en colonnes »** (COLONNES-1, fiche 92) : au menu d'une membrane choisie, à la
//! palette, et pour un agent (`glucose_disposer`, donc `glucose disposer …` au terminal). Le noyau
//! range ([`glucose_core::store::Store::disposer_en_colonnes`]) ; ici, la proportion visée — celle de
//! la fenêtre sous sa bande : la membrane rangée tient l'écran comme une page — et le geste.

use crate::app::GlucoseApp;

impl GlucoseApp {
    /// **La proportion de l'écran utile** : la largeur de la fenêtre sur sa hauteur sous la bande.
    pub(crate) fn proportion_de_l_ecran(&self) -> f64 {
        let (l, h) = self.taille_de_la_fenetre();
        let utile = (h - self.ui.header_height()).max(1.0);
        f64::from(l) / f64::from(utile)
    }

    /// **Range les membranes de la sélection** — choisies, sinon celles qui portent ce qu'on a choisi —
    /// en un seul geste (un `Ctrl+Z` défait tout). Le résultat se voit : aucun message de plus.
    pub(crate) fn disposer_la_selection_en_colonnes(&mut self) {
        let membranes = self.store.membranes_de_la_selection();
        if membranes.is_empty() {
            return;
        }
        let board = self.store.project.active_board_id.clone();
        let proportion = self.proportion_de_l_ecran();
        self.store.begin_live_edit();
        for m in &membranes {
            self.store.disposer_en_colonnes(&board, m, proportion);
        }
        self.store.end_live_edit();
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
