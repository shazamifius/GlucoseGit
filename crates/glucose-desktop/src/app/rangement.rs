//! **Ranger** : le panneau ORDONNER appliqué au tableau.

use super::GlucoseApp;
use crate::dock::{apply_organize_layout, OrganizeState};

impl GlucoseApp {
    /// Applique la réorganisation issue du panneau ORDONNER (Masonry, Grille, Même Hauteur, etc.)
    ///
    /// # ORDONNER-1 — on range **ce qui est sélectionné**, et rien d'autre
    ///
    /// Le panneau rangeait toutes les images du tableau, quoi qu'on ait sélectionné. Choisir
    /// douze images pour les aligner et voir les quatre cents autres se réarranger avec elles
    /// n'est pas une maladresse d'ergonomie : c'est une fonction qui détruit un travail qu'on
    /// ne lui avait pas confié, et l'annulation est le seul recours.
    ///
    /// Une sélection vide veut dire « tout le tableau » — sinon le bouton ne ferait rien du
    /// tout, ce qui serait pire, et c'est le geste qu'on attend d'un rangement global.
    ///
    /// Le toast dit lequel des deux a eu lieu : un rangement qui ne dit pas ce qu'il a touché
    /// laisse chercher.
    pub fn apply_dock_layout(&mut self, state: &OrganizeState) {
        let board_id = self.store.project.active_board_id.clone();
        if self
            .store
            .active_board()
            .is_some_and(|b| b.images.is_empty())
        {
            self.ui.show_toast("Aucune image sur le canvas");
            return;
        }
        // **Le Store dit ce qu'il faut ranger**, parce que « une selection vide veut dire tout
        // le tableau » est une regle metier et non une commodite d'affichage.
        let a_ranger = self.store.images_a_organiser();
        let combien = self.store.selected_image_ids.len();
        // Le cliche se prend autour du geste, et non apres lui : pris apres coup, il capturait
        // l'etat deja range, et Ctrl+Z ne defaisait rien.
        self.store.mutate_board_layout(&board_id, |board| {
            for res in apply_organize_layout(&a_ranger, state) {
                if let Some(img) = board.images.iter_mut().find(|i| i.id == res.id) {
                    img.x = res.x;
                    img.y = res.y;
                    img.width = res.width;
                    img.height = res.height;
                }
            }
        });
        let quoi = if combien == 0 {
            "tout le canvas".to_string()
        } else {
            format!("{combien} image(s)")
        };
        self.ui
            .show_toast(format!("{} : {quoi} rangé(es)", state.layout.title()));
        self.mark_dirty();
    }
}
