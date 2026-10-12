//! **Rendre pour un export** (EXPORT-PNG-1, fiche 62) : la scène à la vue qu'on lui donne, sans
//! les repères de la navigation, et **entière** — chaque image posée à sa finesse, attendue s'il
//! le faut.
//!
//! À l'écran, le rendu demande les images qui lui manquent sans jamais les attendre
//! (DECODE-1) : l'image suivante les montrera. Un export n'a pas d'image suivante. Il attend
//! donc le chantier du magasin et repasse, tant que la passe a demandé du travail — ce que fait
//! un banc pour sa capture, jusqu'au bout.

use super::{halo, Cadrage, Renderer};
use crate::params::SceneOverlay;
use crate::ui::UiState;
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::Viewport;
use tiny_skia::PixmapMut;

impl Renderer {
    /// **Rend la scène à cette vue, toutes ses images posées**, sans grille ni rien de ce qui
    /// n'est pas du contenu.
    pub fn rendre_pour_l_export(
        &mut self,
        pixmap: &mut PixmapMut,
        store: &Store,
        ui: &mut UiState,
        vue: Viewport,
    ) {
        let guides = SnapGuides::default();
        let cadrage = Cadrage {
            vue: Some(vue),
            ..Cadrage::plein()
        }
        .sans_reperes();
        loop {
            self.magasin.ouvrir();
            self.synchroniser_les_caches(store, ui, (pixmap.width(), pixmap.height()));
            let overlay = SceneOverlay::sans_rien(&guides);
            self.rendre_la_region(pixmap, store, ui, overlay, 0.0, cadrage);
            self.magasin.fermer();
            if self.magasin.en_travail() == 0 {
                return;
            }
            self.magasin.attendre_le_chantier();
        }
    }
}

/// **Jusqu'où le contenu déborde de ses boîtes**, en pixels, à cette échelle : la lueur au repos
/// d'une carte — sa dilatation, puis la portée de son flou (HALO-2). Au-delà, aucun pixel ne
/// change : c'est la marge exacte d'un export, sans rien de plus ni de moins.
pub fn debord(echelle: f64) -> f64 {
    f64::from(halo::debord_au_repos(echelle as f32))
}
