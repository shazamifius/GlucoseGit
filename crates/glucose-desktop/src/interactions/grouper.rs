//! **GROUPER-1 — grouper la sélection dans une membrane neuve** (fiche 98) : `Ctrl+G`, le menu
//! d'une sélection, la palette. La loi est celle du noyau ([`glucose_core::store::Store`],
//! `grouper_dans_une_membrane`) ; ici, le geste qui l'appelle.

use crate::app::GlucoseApp;

impl GlucoseApp {
    /// Groupe ce qui est choisi dans une membrane neuve, en **un** geste ; la membrane est choisie
    /// ensuite — un double-clic la nomme. Rend `false` si rien de choisi n'a de boîte : l'accord
    /// passe alors à la suivante de sa liste.
    ///
    /// **Sans message** : la membrane paraît autour de ce qu'on a choisi.
    pub(crate) fn grouper_la_selection(&mut self) -> bool {
        self.store.begin_live_edit();
        let groupe = self.store.grouper_dans_une_membrane();
        self.store.end_live_edit();
        self.mark_dirty();
        groupe.is_some()
    }
}

#[cfg(test)]
mod tests;
