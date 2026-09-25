//! **La fenêtre de l'éditeur du texte lié, posée par-dessus l'interface** (ANCRE-UX).
//!
//! Elle vit ici et non dans l'interface pour une seule raison : elle met en page le texte d'une
//! carte, et seul le moteur de rendu a sous la main ce qu'il faut — les formules, et la teinte
//! de la carte, qui dépend de ses voisines.

use super::{PaintKit, Renderer};
use crate::ui::UiState;
use glucose_core::store::Store;
use tiny_skia::PixmapMut;

impl Renderer {
    /// Pose la fenêtre de l'éditeur, si elle est ouverte — par-dessus tout le reste : elle
    /// attend qu'on choisisse.
    pub(super) fn poser_la_fenetre_d_ancrage(
        &mut self,
        pixmap: &mut PixmapMut,
        store: &Store,
        ui: &UiState,
    ) {
        let Some(ancrage) = &ui.ancrage else {
            return;
        };
        let (Some(board), Some(carte)) = (store.active_board(), ancrage.carte()) else {
            return;
        };
        let Some(ann) = store.project.annotation(&board.id, carte) else {
            return;
        };
        let Some(texte) = ann.own_text() else {
            return;
        };
        let (_, teinte) = self
            .hue_cache
            .get_or_compute(ann, &self.spatial_hash, board);
        let ecran = (pixmap.width() as f32, pixmap.height() as f32);
        let fenetre = crate::ui::ancrage::layout_ancrage(
            ancrage,
            &texte,
            (&self.typography, &self.math),
            ecran,
            ui.scale_factor,
        );
        let kit = PaintKit {
            typography: &self.typography,
            math: &self.math,
            tints: &self.domain_tints,
            theme: &self.theme,
        };
        crate::ui::ancrage::dessiner(pixmap, (&fenetre, ancrage), (&texte, teinte), kit);
    }
}
