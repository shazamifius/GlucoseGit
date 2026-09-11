//! Mutation réversible de la boîte d'un nœud (RESIZE-1).
//!
//! Le geste de redimensionnement produit un rectangle **monde, origine haut-gauche** (§ 2.3)
//! et ne veut rien savoir de la convention de rangement de chaque type. C'est ici, et ici
//! seulement, qu'une image reconvertit ce rectangle en centre, et qu'une carte ou un
//! pense-bête range ses dimensions optionnelles.

use super::Store;
use crate::smart_align::AlignRect;
use crate::types::Annotation;

impl Store {
    /// Pose la boîte d'une image. `x`/`y` deviennent son centre.
    ///
    /// Rend `false` si l'image n'existe pas ; rien n'est alors écrit, ni dans le document ni
    /// dans la pile d'undo.
    pub fn set_image_rect(&mut self, board_id: &str, id: &str, rect: AlignRect) -> bool {
        if !self.board_has(board_id, |b| b.images.iter().any(|i| i.id == id)) {
            return false;
        }
        self.push_undo();
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return false;
        };
        let Some(img) = b.images.iter_mut().find(|i| i.id == id) else {
            return false;
        };
        img.x = rect.left + rect.width / 2.0;
        img.y = rect.top + rect.height / 2.0;
        img.width = rect.width;
        img.height = rect.height;
        true
    }

    /// Pose la boîte d'une carte, d'un pense-bête ou d'une membrane. Une flèche n'a pas de
    /// boîte : elle rend `false` sans rien écrire.
    pub fn set_annotation_rect(&mut self, board_id: &str, id: &str, rect: AlignRect) -> bool {
        let sized = |a: &Annotation| a.id() == id && !matches!(a, Annotation::Arrow { .. });
        if !self.board_has(board_id, |b| b.annotations.iter().any(sized)) {
            return false;
        }
        self.push_undo();
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return false;
        };
        let Some(ann) = b.annotations.iter_mut().find(|a| a.id() == id) else {
            return false;
        };
        match ann {
            Annotation::Text { x, y, width, height, .. } | Annotation::Sticky { x, y, width, height, .. } => {
                *x = rect.left;
                *y = rect.top;
                *width = Some(rect.width);
                *height = Some(rect.height);
                true
            }
            Annotation::Membrane { x, y, width, height, .. } => {
                *x = rect.left;
                *y = rect.top;
                *width = rect.width;
                *height = rect.height;
                true
            }
            Annotation::Arrow { .. } => false,
        }
    }

    /// Pose la boîte d'un dossier. Son contenu est rangé en coordonnées relatives (FOLD-1),
    /// donc déplacer son origine ne le touche pas.
    pub fn set_folder_rect(&mut self, board_id: &str, id: &str, rect: AlignRect) -> bool {
        if !self.board_has(board_id, |b| b.folders.iter().any(|f| f.id == id)) {
            return false;
        }
        self.push_undo();
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return false;
        };
        let Some(f) = b.folders.iter_mut().find(|f| f.id == id) else {
            return false;
        };
        f.x = rect.left;
        f.y = rect.top;
        f.width = rect.width;
        f.height = rect.height;
        true
    }

    fn board_has(&self, board_id: &str, pred: impl Fn(&crate::types::Board) -> bool) -> bool {
        self.project.boards.iter().find(|b| b.id == board_id).is_some_and(pred)
    }
}
