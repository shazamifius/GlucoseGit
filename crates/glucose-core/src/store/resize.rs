//! Mutation réversible de la boîte d'un nœud (RESIZE-1).
//!
//! Le geste de redimensionnement produit un rectangle **monde, origine haut-gauche** (§ 2.3)
//! et ne veut rien savoir de la convention de rangement de chaque type. C'est ici, et ici
//! seulement, qu'une image reconvertit ce rectangle en centre, et qu'une carte ou un
//! pense-bête range ses dimensions optionnelles.

use super::journal::{Edit, Slot};
use super::Store;
use crate::smart_align::AlignRect;
use crate::types::Annotation;

impl Store {
    /// Pose la boîte d'une image. `x`/`y` deviennent son centre.
    ///
    /// Rend `false` si l'image n'existe pas ; rien n'est alors écrit, ni dans le document ni
    /// dans la pile d'undo.
    pub fn set_image_rect(&mut self, board_id: &str, id: &str, rect: AlignRect) -> bool {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return false;
        };
        let Some(i) = b.images.iter().position(|img| img.id == id) else {
            return false;
        };
        let before = b.images[i].clone();
        let img = &mut b.images[i];
        img.x = rect.left + rect.width / 2.0;
        img.y = rect.top + rect.height / 2.0;
        img.width = rect.width;
        img.height = rect.height;
        let after = b.images[i].clone();

        self.record_edit(Edit::Image {
            board: board_id.to_string(),
            slot: Slot::changed(i, before, after),
        });
        true
    }

    /// Pose la boîte d'une carte, d'un pense-bête ou d'une membrane. Une flèche n'a pas de
    /// boîte : elle rend `false` sans rien écrire.
    pub fn set_annotation_rect(&mut self, board_id: &str, id: &str, rect: AlignRect) -> bool {
        let sized = |a: &&Annotation| a.id() == id && !matches!(a, Annotation::Arrow { .. });
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return false;
        };
        // Le predicat ecarte deja les fleches : l'index trouve designe forcement une boite.
        let Some(i) = b.annotations.iter().position(|a| sized(&a)) else {
            return false;
        };
        let before = b.annotations[i].clone();
        let ann = &mut b.annotations[i];
        let ok = match ann {
            Annotation::Text {
                x,
                y,
                width,
                height,
                ..
            }
            | Annotation::Sticky {
                x,
                y,
                width,
                height,
                ..
            } => {
                *x = rect.left;
                *y = rect.top;
                *width = Some(rect.width);
                *height = Some(rect.height);
                true
            }
            Annotation::Membrane {
                x,
                y,
                width,
                height,
                ..
            } => {
                *x = rect.left;
                *y = rect.top;
                *width = rect.width;
                *height = rect.height;
                true
            }
            Annotation::Arrow { .. } => false,
        };
        if !ok {
            return false;
        }
        let after = b.annotations[i].clone();
        self.record_edit(Edit::Annotation {
            board: board_id.to_string(),
            slot: Slot::changed(i, before, after),
        });
        true
    }

    /// Pose la boîte d'un dossier. Son contenu est rangé en coordonnées relatives (FOLD-1),
    /// donc déplacer son origine ne le touche pas.
    pub fn set_folder_rect(&mut self, board_id: &str, id: &str, rect: AlignRect) -> bool {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return false;
        };
        let Some(i) = b.folders.iter().position(|f| f.id == id) else {
            return false;
        };
        let before = b.folders[i].clone();
        let f = &mut b.folders[i];
        f.x = rect.left;
        f.y = rect.top;
        f.width = rect.width;
        f.height = rect.height;
        let after = b.folders[i].clone();

        self.record_edit(Edit::Folder {
            board: board_id.to_string(),
            slot: Slot::changed(i, before, after),
        });
        true
    }
}
