//! Dossiers de canevas et boards enfants.
//!
//! INVARIANT FOLD-1 — un dossier possède un board enfant, et les éléments capturés à sa
//! création y sont stockés en coordonnées RELATIVES au coin haut-gauche du dossier. C'est ce
//! qui permet de déplacer un dossier sans toucher à son contenu.

use super::images::translate_annotation;
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
    pub fn create_folder(&mut self, parent_board_id: &str, mut folder: CanvasFolder) {
        self.push_undo();
        if folder.id.is_empty() {
            folder.id = self.generate_id("folder");
        }
        let child_board_id = self.generate_id("board");
        folder.child_board_id = child_board_id.clone();

        let captured = self.capture_content_under(parent_board_id, &folder);
        if let Some(par) = self.project.boards.iter_mut().find(|b| b.id == parent_board_id) {
            par.folders.push(folder.clone());
        }

        let mut child_board = Board::new(&child_board_id, &folder.name);
        child_board.images = captured.images;
        child_board.annotations = captured.annotations;
        self.project.boards.push(child_board);
    }

    /// Retire du board parent tout ce qui tombe dans l'emprise du dossier et le rend en
    /// coordonnées relatives (FOLD-1).
    fn capture_content_under(
        &mut self,
        parent_board_id: &str,
        folder: &CanvasFolder,
    ) -> CapturedContent {
        let (fx0, fy0) = (folder.x, folder.y);
        let (fx1, fy1) = (folder.x + folder.width, folder.y + folder.height);
        let inside = |cx: f64, cy: f64| cx >= fx0 && cx <= fx1 && cy >= fy0 && cy <= fy1;

        let mut out = CapturedContent::default();
        let Some(par) = self.project.boards.iter_mut().find(|b| b.id == parent_board_id) else {
            return out;
        };

        let img_ids: HashSet<String> =
            par.images.iter().filter(|i| inside(i.x, i.y)).map(|i| i.id.clone()).collect();
        let ann_ids: HashSet<String> = par
            .annotations
            .iter()
            .filter(|a| inside(a.x(), a.y()))
            .map(|a| a.id().to_string())
            .collect();

        par.images.retain(|img| {
            if !img_ids.contains(&img.id) {
                return true;
            }
            let mut moved = img.clone();
            moved.x -= fx0;
            moved.y -= fy0;
            out.images.push(moved);
            false
        });
        par.annotations.retain(|ann| {
            if !ann_ids.contains(ann.id()) {
                return true;
            }
            let mut moved = ann.clone();
            translate_annotation(&mut moved, -fx0, -fy0);
            out.annotations.push(moved);
            false
        });
        out
    }

    pub fn create_folder_with_content(
        &mut self,
        parent_board_id: &str,
        mut folder: CanvasFolder,
        seed_annotations: Vec<Annotation>,
    ) -> String {
        self.push_undo();
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
        self.project.boards.push(child_board);

        if let Some(par) = self.project.boards.iter_mut().find(|b| b.id == parent_board_id) {
            par.folders.push(folder);
        }

        folder_id
    }

    pub fn update_folder<F: FnOnce(&mut CanvasFolder)>(
        &mut self,
        parent_board_id: &str,
        id: &str,
        f: F,
    ) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == parent_board_id) {
            if let Some(fld) = b.folders.iter_mut().find(|f| f.id == id) {
                f(fld);
            }
        }
    }

    pub fn remove_folders(&mut self, parent_board_id: &str, ids: &[&str]) {
        self.push_undo();
        let id_set: HashSet<&str> = ids.iter().copied().collect();
        let mut child_board_ids = Vec::new();

        if let Some(par) = self.project.boards.iter_mut().find(|b| b.id == parent_board_id) {
            for f in &par.folders {
                if id_set.contains(f.id.as_str()) {
                    child_board_ids.push(f.child_board_id.clone());
                }
            }
            par.folders.retain(|f| !id_set.contains(f.id.as_str()));
        }

        self.project.boards.retain(|b| !child_board_ids.contains(&b.id));
        if child_board_ids.contains(&self.project.active_board_id) {
            self.project.active_board_id = parent_board_id.to_string();
        }
    }

    /// Crée un miroir de dossier, ou dit pourquoi il n'a pas pu (R-35).
    pub fn try_mirror_folder(
        &mut self,
        parent_board_id: &str,
        folder_id: &str,
        x: f64,
        y: f64,
    ) -> CoreResult<String> {
        self.push_undo();
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

        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == parent_board_id) {
            b.folders.push(mirrored);
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
        self.try_mirror_folder(parent_board_id, folder_id, x, y).ok()
    }
}
