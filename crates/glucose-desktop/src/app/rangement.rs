//! **Ranger** : le panneau ORDONNER appliqué au tableau.
//!
//! # ORDONNER-2 — un rangement par la couleur attend ses couleurs, sans geler
//!
//! Trier par teinte ou par clarté demande la couleur de chaque image : le sommet de sa
//! pyramide (MOYENNE-1), que le magasin tient pour toute image qu'il connaît. Une image qu'il
//! ne connaît pas encore est demandée à l'atelier, et le rangement **attend** — l'écran
//! continue de vivre, et tout s'applique d'un bloc, en une seule entrée d'annulation, à l'image
//! où la dernière couleur arrive. C'est le chemin du `Ctrl+B` qui attend ses originaux.
//!
//! Glucose Tauri attendait chaque image 1,2 seconde, puis lui donnait une luminosité de 0,5 en
//! silence. Ici une image qui ne se lira jamais n'a pas de couleur : elle va à la fin, et le
//! compte-rendu le dit.

use super::GlucoseApp;
use crate::dock::{apply_organize_layout, OrganizeState};
use crate::renderer::magasin::Moyenne;
use glucose_core::couleur::{self, Oklch, Tri};
use glucose_core::types::BoardImage;
use std::collections::HashMap;

/// **Un rangement par la couleur qui attend ses couleurs** : ce qu'il range, comment, et ce
/// qu'il a déjà lu.
pub struct RangementEnAttente {
    board: String,
    ids: Vec<String>,
    etat: OrganizeState,
    tri: Tri,
    /// La couleur de chaque fichier déjà lue — `None` pour un fichier qui n'en aura jamais.
    /// Gardée ici, une image lue ne se redemande pas si le magasin la rend ensuite.
    lues: HashMap<String, Option<Oklch>>,
}

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
        let Some(tri) = state.sort_by.tri_par_couleur() else {
            self.ranger(&board_id, &a_ranger, state, 0);
            return;
        };
        self.attentes.rangement = Some(RangementEnAttente {
            board: board_id,
            ids: a_ranger.into_iter().map(|i| i.id).collect(),
            etat: state.clone(),
            tri,
            lues: HashMap::new(),
        });
        self.poursuivre_les_attentes();
    }

    /// **Applique le rangement en attente dès que toutes ses couleurs sont lues** — appelée à
    /// chaque image, et au geste lui-même.
    pub(crate) fn poursuivre_le_rangement(&mut self) {
        let Some(attente) = self.attentes.rangement.as_mut() else {
            return;
        };
        let magasin = &mut self.renderer.magasin;
        let mut en_chemin = 0usize;
        for id in &attente.ids {
            let Some(img) = self.store.image(&attente.board, id) else {
                continue;
            };
            let Some(src) = img.src.as_deref() else {
                continue;
            };
            if attente.lues.contains_key(src) {
                continue;
            }
            match magasin.moyenne(src) {
                Moyenne::Connue(pixel) => {
                    attente.lues.insert(src.to_string(), Oklch::du_pixel(pixel));
                }
                Moyenne::Illisible => {
                    attente.lues.insert(src.to_string(), None);
                }
                Moyenne::EnChemin => en_chemin += 1,
            }
        }
        if en_chemin > 0 {
            return;
        }
        if let Some(attente) = self.attentes.rangement.take() {
            self.ranger_par_la_couleur(attente);
        }
    }

    /// Range les images dans l'ordre de leurs couleurs ; celles qui n'en ont pas, à la fin.
    fn ranger_par_la_couleur(&mut self, attente: RangementEnAttente) {
        let images: Vec<BoardImage> = attente
            .ids
            .iter()
            .filter_map(|id| self.store.image(&attente.board, id).cloned())
            .collect();
        let couleurs: Vec<Option<Oklch>> = images
            .iter()
            .map(|img| *img.src.as_deref().and_then(|s| attente.lues.get(s))?)
            .collect();
        let sans_couleur = couleurs.iter().filter(|c| c.is_none()).count();
        let rangees: Vec<BoardImage> = couleur::ordonner(&couleurs, attente.tri)
            .into_iter()
            .map(|k| images[k].clone())
            .collect();
        self.ranger(&attente.board, &rangees, &attente.etat, sans_couleur);
    }

    /// Pose les images dans la disposition, en un geste annulable, et dit ce qui a été rangé.
    /// Le toast dit si c'était la sélection ou tout le tableau : un rangement qui ne dit pas ce
    /// qu'il a touché laisse chercher.
    fn ranger(
        &mut self,
        board_id: &str,
        images: &[BoardImage],
        state: &OrganizeState,
        sans_couleur: usize,
    ) {
        let combien = self.store.selected_image_ids.len();
        // Le cliche se prend autour du geste, et non apres lui : pris apres coup, il capturait
        // l'etat deja range, et Ctrl+Z ne defaisait rien.
        self.store.mutate_board_layout(board_id, |board| {
            for res in apply_organize_layout(images, state) {
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
        let mut message = format!("{} : {quoi} rangé(es)", state.layout.title());
        if sans_couleur > 0 {
            message.push_str(&format!(" — {sans_couleur} sans couleur, à la fin"));
        }
        self.ui.show_toast(message);
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
