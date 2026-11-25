//! **Un clic sur une vidéo** (VIDEO-1, fiche 169) : sur ▶, elle joue ou s'arrête ; sur l'icône du
//! son, il se rallume ou se coupe. Un clic qui n'a rien déplacé, et sur le nœud du dessus
//! seulement : un glisser déplace la vidéo, comme toute photo, et la sélectionne toujours.

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use crate::renderer::video::commandes::{touche, Touche};
use glucose_core::hit_priority::PickOwner;
use std::path::Path;

/// **La forme d'une vidéo avant sa première image** : 16:9, le format le plus courant ; elle
/// prend la sienne quand l'atelier l'a décodée (`Store::mesurer_les_videos`).
const FORME_D_ATTENTE: (f64, f64) = (1920.0, 1080.0);

impl GlucoseApp {
    /// **Pose une vidéo déposée** — si ce fichier en est une et que ce système sait la lire ;
    /// sinon `false`, et elle reste un lanceur, comme avant VIDEO-1.
    pub(crate) fn poser_une_video(&mut self, board: &str, path: &Path, ou: (f64, f64)) -> bool {
        use crate::renderer::video::{lisible_ici, signature};
        use std::io::Read;
        let mut debut = Vec::new();
        let lu = std::fs::File::open(path)
            .and_then(|f| f.take(signature::LONGUEUR as u64).read_to_end(&mut debut));
        if lu.is_err() || !signature::est_video(&debut) || !lisible_ici() {
            return false;
        }
        let Some(src) = path.to_str().map(str::to_string) else {
            return false;
        };
        let taille = crate::interactions::clipboard::taille_posee(FORME_D_ATTENTE);
        let id = self.store.generate_id("img");
        let mut img = glucose_core::types::BoardImage::new(id, ou.0, ou.1, taille.0, taille.1);
        img.src = Some(src);
        (img.original_width, img.original_height) = FORME_D_ATTENTE;
        img.is_video = true;
        self.store.add_image(board, img);
        true
    }

    /// **Le clic relâché touche-t-il une commande de vidéo ?** Rend `true` s'il en a actionné une.
    pub(crate) fn toucher_une_video(&mut self) -> bool {
        let vp = self.store.viewport();
        let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        let Some(dessus) = self.pick_candidates_at(wx, wy).into_iter().next() else {
            return false;
        };
        if dessus.owner != PickOwner::Image {
            return false;
        }
        let board = self.store.project.active_board_id.clone();
        let Some(img) = self.store.image(&board, &dessus.id).filter(|i| i.is_video) else {
            return false;
        };
        let Some(src) = img.src.clone() else {
            return false;
        };
        let magasin = &mut self.renderer.magasin;
        let lancee = magasin.etat_video(&src).is_some();
        match touche(img, self.mouse_pos, &vp, lancee) {
            Some(Touche::Lecture) => magasin.basculer_la_video(&src),
            Some(Touche::Son) => drop(magasin.basculer_le_son(&src)),
            None => return false,
        }
        self.mark_dirty();
        true
    }
}
