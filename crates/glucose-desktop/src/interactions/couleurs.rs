//! **COULEUR-1 — colorer la sélection** (fiche 98) : une teinte du nuancier, ou la couleur par
//! défaut, sur chaque texte, post-it et membrane choisi — en un geste. La barre de couleur
//! ([`crate::ui::couleurs`]), la palette et le clavier y passent tous : c'est la commande
//! `couleur.<teinte>`.

use crate::app::GlucoseApp;
use crate::params::{Pointer, ScreenFrame};
use crate::ui::couleurs::{couvre, layout_barre_de_couleur, teinte, teinte_sous};

impl GlucoseApp {
    /// Pose la teinte de rang `rang` du nuancier — ou rend la couleur par défaut (`None`) — sur
    /// tout ce qui est choisi et se colore, en **un** geste. Rend `false` si rien ne s'y prête.
    pub(crate) fn colorer_la_selection(&mut self, rang: Option<u8>) -> bool {
        let couleur = match rang {
            Some(r) => match teinte(r) {
                Some(hex) => Some(hex),
                None => return false,
            },
            None => None,
        };
        let ids: Vec<String> = self
            .store
            .selected_colorables()
            .iter()
            .map(|a| a.id().to_string())
            .collect();
        if ids.is_empty() {
            return false;
        }
        let board = self.store.project.active_board_id.clone();
        self.store.begin_live_edit();
        self.store.colorer(&board, &ids, couleur);
        self.store.end_live_edit();
        self.mark_dirty();
        true
    }

    /// **La barre de couleur** : elle prend tout ce qui tombe sur elle, comme ses voisines.
    pub fn click_barre_de_couleur(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        // Pendant l'édition des ancres, c'est son panneau qui parle (FLECHE-4).
        if self.ui.ancrage.is_some() {
            return false;
        }
        let Some(barre) = layout_barre_de_couleur(
            &self.store,
            &self.renderer.typography,
            self.ui.ecran_visible((screen.width, screen.height)),
            screen.scale,
        ) else {
            return false;
        };
        if !couvre(&barre, pointer.x, pointer.y) {
            return false;
        }
        if let Some(rang) = teinte_sous(&barre, pointer.x, pointer.y) {
            self.colorer_la_selection(rang);
        }
        true
    }
}

#[cfg(test)]
mod tests;
