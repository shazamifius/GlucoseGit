//! **Poser un gabarit** (PRESET-1, fiche 62) : choisi dans le panneau des Presets, il suit le
//! curseur — une zone fantôme par catégorie —, et un clic sur le canevas le pose là, centré ;
//! Échap y renonce. C'est le geste de Glucose Tauri (« mode de placement » : le panneau se
//! ferme, un aperçu suit la souris, un clic pose).
//!
//! **Une seule fonction dit où le gabarit se poserait** (`glucose_core::store::zones::rangee`) :
//! le fantôme la lit au mouvement, le clic la lit au clic — il se pose là où on l'a vu.

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use crate::dock::preset::GabaritIntent;
use crate::dock::TabId;
use glucose_core::gabarits::a_choisir;
use glucose_core::geometry::Rect;
use glucose_core::store::zones::rangee;
use glucose_core::types::BoardZone;

impl GlucoseApp {
    /// **Arme ce gabarit** : il suivra le curseur jusqu'au clic qui le pose.
    fn armer_le_gabarit(&mut self, id: String) {
        self.ui.placement.gabarit = Some(id);
        self.mark_dirty();
    }

    /// **Ce que demande le panneau des Presets** : armer un gabarit — le panneau se ferme, comme
    /// dans Tauri, pour laisser voir le canevas où il va se poser —, ou retirer celui du tableau.
    pub(crate) fn suivre_le_panneau_des_presets(&mut self, intent: GabaritIntent) {
        match intent {
            GabaritIntent::Armer(id) => {
                self.dock_manager.toggle_tab(TabId::Preset);
                self.armer_le_gabarit(id);
            }
            GabaritIntent::Retirer => {
                let board = self.store.project.active_board_id.clone();
                self.store.retirer_le_gabarit(&board);
                self.mark_dirty();
            }
        }
    }

    /// Les zones du gabarit armé, centrées sous le curseur ; aucune s'il n'y en a pas d'armé.
    fn rangee_sous_le_curseur(&self) -> Option<Vec<BoardZone>> {
        let id = self.ui.placement.gabarit.as_deref()?;
        let choix = a_choisir(&self.store.project)
            .into_iter()
            .find(|c| c.id == id)?;
        let categories: Vec<&str> = choix.categories.iter().map(|c| c.0).collect();
        let vp = self.store.viewport();
        let centre = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        Some(rangee((choix.id, &categories), centre))
    }

    /// **Le fantôme du gabarit armé**, une boîte par zone, en unités monde.
    pub(crate) fn fantome_du_gabarit(&self) -> Vec<Rect> {
        self.rangee_sous_le_curseur()
            .unwrap_or_default()
            .iter()
            .map(|z| Rect::new(z.x, z.y, z.width, z.height))
            .collect()
    }

    /// **Pose le gabarit armé là où son fantôme est**, s'il y en a un : le clic est pris, et
    /// le gabarit cesse de suivre le curseur. Un seul `Ctrl+Z` le retire.
    pub(crate) fn poser_le_gabarit_arme(&mut self) -> bool {
        let Some(zones) = self.rangee_sous_le_curseur() else {
            return false;
        };
        let board = self.store.project.active_board_id.clone();
        self.store.poser_les_zones(&board, zones);
        self.ui.placement.gabarit = None;
        self.mark_dirty();
        true
    }

    /// **Renonce au gabarit armé** — Échap. Rend vrai s'il y en avait un.
    pub(crate) fn renoncer_au_gabarit(&mut self) -> bool {
        if self.ui.placement.gabarit.take().is_none() {
            return false;
        }
        self.mark_dirty();
        true
    }

    /// Le curseur bouge : le fantôme du gabarit armé le suit.
    pub(crate) fn suivre_le_gabarit(&mut self) {
        if self.ui.placement.gabarit.is_some() {
            self.mark_dirty();
        }
    }
}

#[cfg(test)]
mod tests;
