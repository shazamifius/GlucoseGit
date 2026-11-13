//! **ZEN-1 — les gestes du mode zen** (fiche 115) : `F11`, la palette, le menu du vide et
//! l'indicateur le basculent ; `Échap` le referme. L'état vit dans [`crate::ui::UiState::zen`] ; ce
//! qu'il cache, `UiState::sans_interface` le dit à tout ce qui se dessine et se clique.

use crate::app::GlucoseApp;
use crate::params::{Pointer, ScreenFrame};

impl GlucoseApp {
    /// **Entre dans le mode zen, ou en sort.** La fenêtre ne change pas (ZEN-1).
    pub(crate) fn basculer_le_mode_zen(&mut self) {
        self.ui.zen = !self.ui.zen;
        self.ui.context_menu_at = None;
        // La bande part ou revient : toute la vue change de cadre.
        self.mark_dirty();
    }

    /// **L'indicateur « ZEN · F11 », cliqué** : le mode se referme, et le clic s'arrête là.
    pub(crate) fn click_zen(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        if !self.ui.zen {
            return false;
        }
        let ecran = (screen.width, screen.height);
        let indicateur = crate::ui::zen::placer(&self.ui, &self.renderer.typography, ecran);
        if !crate::ui::zen::sous_la_main(&indicateur, (pointer.x, pointer.y)) {
            return false;
        }
        self.basculer_le_mode_zen();
        true
    }
}

#[cfg(test)]
mod tests;
