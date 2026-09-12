//! Dossiers de canevas et boards enfants.
//!
//! INVARIANT FOLD-1 — un dossier possède un board enfant, et les éléments capturés à sa
//! création y sont stockés en coordonnées RELATIVES au coin haut-gauche du dossier. C'est ce
//! qui permet de déplacer un dossier sans toucher à son contenu.

use super::images::translate_annotation;
use super::journal::{Edit, Slot, Whole};
use super::Store;
use crate::error::{CoreError, CoreResult};
use crate::types::{Annotation, Board, BoardImage, CanvasFolder};
use std::collections::HashSet;

/// Contenu extrait du board parent au moment où un dossier est créé par-dessus lui.
#[derive(Default)]
struct CapturedContent {
    images: Vec<BoardImage>,
    annotations: Vec<Annotation>,
}

impl Store {
    /// Crée un dossier et y aspire ce qui tombe sous son emprise.
    ///
    /// **Site migré vers le journal.** Le geste est composite — du contenu quitte le board
    /// parent, un dossier y apparaît, un sous-board naît — et forme une seule entrée. Les
    /// éditions sont consignées dans l'ordre où elles sont faites ; l'inversion les rejoue
    /// à l'envers, ce qui remet le contenu à sa place avant que le dossier ne reparte.
    pub fn create_folder(&mut self, parent_board_id: &str, mut folder: CanvasFolder) {
        if folder.id.is_empty() {
            folder.id = self.generate_id("folder");
        }
        let child_board_id = self.generate_id("board");
        folder.child_board_id = child_board_id.clone();

        let mut edits = Vec::new();
        let captured = self.capture_content_under(parent_board_id, &folder, &mut edits);

        if let Some(par) = self
            .project
            .boards
            .iter_mut()
            .find(|b| b.id == parent_board_id)
        {
            let index = par.folders.len();
            par.folders.push(folder.clone());
            edits.push(Edit::Folder {
                board: parent_board_id.to_string(),
                slot: Slot::inserted(index, folder.clone()),
            });
        }

        let mut child_board = Board::new(&child_board_id, &folder.name);
        child_board.images = captured.images;
        child_board.annotations = captured.annotations;
        let at = self.project.boards.len();
        self.project.boards.push(child_board.clone());
        edits.push(Edit::Board { slot: Slot::inserted(at, child_board) });

        self.record_as_one_gesture(edits);
    }

    /// Retire du board parent tout ce qui tombe dans l'emprise du dossier et le rend en
    /// coordonnées relatives (FOLD-1).
    fn capture_content_under(
        &mut self,
        parent_board_id: &str,
        folder: &CanvasFolder,
        edits: &mut Vec<Edit>,
    ) -> CapturedContent {
        let (fx0, fy0) = (folder.x, folder.y);
        let (fx1, fy1) = (folder.x + folder.width, folder.y + folder.height);
        let inside = |cx: f64, cy: f64| cx >= fx0 && cx <= fx1 && cy >= fy0 && cy <= fy1;

        let mut out = CapturedContent::default();
        let Some(par) = self
            .project
            .boards
            .iter_mut()
            .find(|b| b.id == parent_board_id)
        else {
            return out;
        };

        let img_ids: HashSet<String> = par
            .images
            .iter()
            .filter(|i| inside(i.x, i.y))
            .map(|i| i.id.clone())
            .collect();
        let ann_ids: HashSet<String> = par
            .annotations
            .iter()
            .filter(|a| inside(a.x(), a.y()))
            .map(|a| a.id().to_string())
            .collect();

        // Parcours arrière : les index consignés restent valides à la réinsertion, comme
        // pour `remove_images`. Le contenu capturé est ensuite remis à l'endroit, puisque
        // `out` doit rester dans l'ordre d'origine.
        for i in (0..par.images.len()).rev() {
            if !img_ids.contains(&par.images[i].id) {
                continue;
            }
            let original = par.images.remove(i);
            edits.push(Edit::Image {
                board: parent_board_id.to_string(),
                slot: Slot::removed(i, original.clone()),
            });
            let mut moved = original;
            moved.x -= fx0;
            moved.y -= fy0;
            out.images.push(moved);
        }
        for i in (0..par.annotations.len()).rev() {
            if !ann_ids.contains(par.annotations[i].id()) {
                continue;
            }
            let original = par.annotations.remove(i);
            edits.push(Edit::Annotation {
                board: parent_board_id.to_string(),
                slot: Slot::removed(i, original.clone()),
            });
            let mut moved = original;
            translate_annotation(&mut moved, -fx0, -fy0);
            out.annotations.push(moved);
        }
        out.images.reverse();
        out.annotations.reverse();
        out
    }

    pub fn create_folder_with_content(
        &mut self,
        parent_board_id: &str,
        mut folder: CanvasFolder,
        seed_annotations: Vec<Annotation>,
    ) -> String {
        let folder_id = if folder.id.is_empty() {
            self.generate_id("folder")
        } else {
            folder.id.clone()
        };
        let child_board_id = self.generate_id("board");
        folder.id = folder_id.clone();
        folder.child_board_id = child_board_id.clone();

        let mut child_board = Board::new(&child_board_id, &folder.name);
        child_board.annotations = seed_annotations;
        let at = self.project.boards.len();
        self.project.boards.push(child_board.clone());
        let mut edits = vec![Edit::Board { slot: Slot::inserted(at, child_board) }];

        if let Some(par) = self
            .project
            .boards
            .iter_mut()
            .find(|b| b.id == parent_board_id)
        {
            let index = par.folders.len();
            par.folders.push(folder.clone());
            edits.push(Edit::Folder {
                board: parent_board_id.to_string(),
                slot: Slot::inserted(index, folder),
            });
        }

        self.record_as_one_gesture(edits);
        folder_id
    }

    pub fn update_folder<F: FnOnce(&mut CanvasFolder)>(
        &mut self,
        parent_board_id: &str,
        id: &str,
        f: F,
    ) {
        let Some(b) = self
            .project
            .boards
            .iter_mut()
            .find(|b| b.id == parent_board_id)
        else {
            return;
        };
        let Some(i) = b.folders.iter().position(|fld| fld.id == id) else {
            return;
        };
        let before = b.folders[i].clone();
        f(&mut b.folders[i]);
        let after = b.folders[i].clone();
        self.record_edit(Edit::Folder {
            board: parent_board_id.to_string(),
            slot: Slot::changed(i, before, after),
        });
    }

    pub fn remove_folders(&mut self, parent_board_id: &str, ids: &[&str]) {
        let id_set: HashSet<&str> = ids.iter().copied().collect();
        let mut child_board_ids = Vec::new();
        let mut edits = Vec::new();

        if let Some(par) = self
            .project
            .boards
            .iter_mut()
            .find(|b| b.id == parent_board_id)
        {
            for i in (0..par.folders.len()).rev() {
                if !id_set.contains(par.folders[i].id.as_str()) {
                    continue;
                }
                let removed = par.folders.remove(i);
                child_board_ids.push(removed.child_board_id.clone());
                edits.push(Edit::Folder {
                    board: parent_board_id.to_string(),
                    slot: Slot::removed(i, removed),
                });
            }
        }

        // Un sous-board supprimé emporte tout son contenu : l'entrée est lourde, et c'est
        // exactement la taille de ce que le geste détruit (JRN-1).
        for i in (0..self.project.boards.len()).rev() {
            if !child_board_ids.contains(&self.project.boards[i].id) {
                continue;
            }
            let removed = self.project.boards.remove(i);
            edits.push(Edit::Board { slot: Slot::removed(i, removed) });
        }

        if child_board_ids.contains(&self.project.active_board_id) {
            let before = self.project.active_board_id.clone();
            self.project.active_board_id = parent_board_id.to_string();
            edits.push(Edit::ActiveBoard {
                whole: Whole::new(before, parent_board_id.to_string()),
            });
        }

        self.record_as_one_gesture(edits);
    }

    /// Crée un miroir de dossier, ou dit pourquoi il n'a pas pu (R-35).
    pub fn try_mirror_folder(
        &mut self,
        parent_board_id: &str,
        folder_id: &str,
        x: f64,
        y: f64,
    ) -> CoreResult<String> {
        let source = self
            .project
            .boards
            .iter()
            .find(|b| b.id == parent_board_id)
            .ok_or_else(|| CoreError::BoardNotFound(parent_board_id.to_string()))?
            .folders
            .iter()
            .find(|f| f.id == folder_id)
            .ok_or_else(|| CoreError::FolderNotFound(folder_id.to_string()))?
            .clone();

        let mid = self.generate_id("mirror-folder");
        let mut mirrored = source;
        mirrored.id = mid.clone();
        mirrored.x = x;
        mirrored.y = y;
        mirrored.mirror_of = Some(folder_id.to_string());

        if let Some(b) = self
            .project
            .boards
            .iter_mut()
            .find(|b| b.id == parent_board_id)
        {
            let index = b.folders.len();
            b.folders.push(mirrored.clone());
            self.record_edit(Edit::Folder {
                board: parent_board_id.to_string(),
                slot: Slot::inserted(index, mirrored),
            });
        }
        Ok(mid)
    }

    /// Enveloppe silencieuse de [`Store::try_mirror_folder`].
    #[deprecated(note = "R-35 : l'échec est silencieux. Utiliser `try_mirror_folder`.")]
    pub fn mirror_folder(
        &mut self,
        parent_board_id: &str,
        folder_id: &str,
        x: f64,
        y: f64,
    ) -> Option<String> {
        self.try_mirror_folder(parent_board_id, folder_id, x, y)
            .ok()
    }
}
