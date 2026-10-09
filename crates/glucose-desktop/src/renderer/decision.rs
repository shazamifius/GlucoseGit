//! **Ce qui attend une décision, posé par-dessus les panneaux** (DECISION-1, fiche 59).
//!
//! # L'ordre des couches, écrit une fois
//!
//! Le canevas, puis la chrome permanente (la bande, le fil d'Ariane, la minimap, le rail), puis
//! les panneaux, puis **ce qui attend une décision** : la barre d'action, les options de flèche,
//! la fenêtre des ancres, le toast, le menu, la question. C'est l'ordre même dans lequel le clic
//! descend (`interactions::mouse::handle_left_down`) : ce qu'on voit au-dessus est ce qu'on
//! touche.
//!
//! Les deux peintres appellent cette méthode **après** les panneaux — la voie processeur dans
//! son tampon, la voie graphique dans la couche du dessus ou, quand un panneau est ouvert, dans
//! un tampon que la carte pose après eux (`app::decision`). `render`, `rendre_reduit` et
//! `rendre_les_couches` ne la posent pas : elles s'arrêtent à la chrome permanente.

use super::Renderer;
use crate::params::Pointer;
use crate::ui::UiState;
use glucose_core::store::Store;
use tiny_skia::PixmapMut;

impl Renderer {
    /// Pose ce qui attend une décision, dans l'ordre où le clic le trouve.
    pub fn poser_ce_qui_attend_une_decision(
        &mut self,
        pixmap: &mut PixmapMut,
        store: &Store,
        ui: &UiState,
        pointer: Pointer,
    ) {
        let kit = (&self.typography, &self.theme);
        crate::ui::decision::poser_les_barres(pixmap, store, ui, kit);
        self.poser_la_fenetre_d_ancrage(pixmap, store, ui);
        let kit = (&self.typography, &self.theme);
        crate::ui::decision::poser_les_reponses_attendues(pixmap, store, ui, kit, pointer);
        // **Ce que `ui` nomme** : ce qui ne paraît que sur décision de l'utilisateur. La marque
        // se posait autrefois chez l'appelant, après la bande, le fil d'Ariane et la minimap :
        // un poste qui nomme quatre choses ne désigne rien.
        crate::perf::stage("ui");
    }
}
