//! Navigation : board actif, viewport, pan/zoom, dossiers.
//!
//! INVARIANT UNDO-1 — aucune de ces opérations ne pousse d'entrée d'undo (règle 3.5 des
//! standards) : la navigation n'est pas une mutation du document.

use super::Store;
use crate::error::{CoreError, CoreResult};
use crate::types::{Board, FolderTreeNode, TemporalAnchor, Viewport};

/// Reconstruit la pile de dossiers UI menant au board actif.
pub fn build_folder_stack(boards: &[Board], active_board_id: &str) -> Vec<(String, String)> {
    let mut parent_map = std::collections::HashMap::new();
    for b in boards {
        for f in &b.folders {
            parent_map.insert(f.child_board_id.clone(), (b.id.clone(), f.id.clone()));
        }
    }
    let mut stack = Vec::new();
    let mut curr = active_board_id.to_string();
    let mut guard = 256;
    while let Some(parent) = parent_map.get(&curr) {
        if guard == 0 {
            break;
        }
        guard -= 1;
        stack.insert(0, parent.clone());
        curr = parent.0.clone();
    }
    stack
}

impl Store {
    pub fn active_board(&self) -> Option<&Board> {
        self.project
            .boards
            .iter()
            .find(|b| b.id == self.project.active_board_id)
    }

    pub fn active_board_mut(&mut self) -> Option<&mut Board> {
        let id = self.project.active_board_id.clone();
        self.project.boards.iter_mut().find(|b| b.id == id)
    }

    /// Change de board actif, ou dit pourquoi il ne l'a pas fait (R-35).
    pub fn try_set_active_board_id(&mut self, board_id: impl Into<String>) -> CoreResult<()> {
        let bid = board_id.into();
        if !self.project.boards.iter().any(|b| b.id == bid) {
            return Err(CoreError::BoardNotFound(bid));
        }
        self.project.active_board_id = bid.clone();
        self.folder_stack = build_folder_stack(&self.project.boards, &bid);
        self.clear_selection();
        Ok(())
    }

    /// Enveloppe silencieuse de [`Store::try_set_active_board_id`].
    ///
    /// NON marquée `#[deprecated]` — contrairement aux autres enveloppes R-35 — parce que
    /// `glucose-desktop` l'appelle et que le dépôt exige `clippy -D warnings` à zéro
    /// avertissement. À migrer vers `try_set_active_board_id` en même temps que ses appelants.
    pub fn set_active_board_id(&mut self, board_id: impl Into<String>) {
        drop(self.try_set_active_board_id(board_id));
    }

    pub fn set_viewport(&mut self, board_id: &str, vp: Viewport) {
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.viewport = vp;
        }
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        if let Some(b) = self.active_board_mut() {
            b.viewport.x += dx;
            b.viewport.y += dy;
        }
    }

    pub fn zoom(&mut self, factor: f64, cursor_x: f64, cursor_y: f64) {
        if let Some(b) = self.active_board_mut() {
            let old_scale = b.viewport.scale;
            let new_scale = (old_scale * factor).clamp(0.01, 50.0);
            b.viewport.x = cursor_x - (cursor_x - b.viewport.x) * (new_scale / old_scale);
            b.viewport.y = cursor_y - (cursor_y - b.viewport.y) * (new_scale / old_scale);
            b.viewport.scale = new_scale;
        }
    }

    /// Entre dans un dossier du board actif, ou dit pourquoi il n'a pas pu (R-35).
    pub fn try_enter_folder(&mut self, folder_id: &str) -> CoreResult<()> {
        let current_board_id = self.project.active_board_id.clone();
        let target = self
            .active_board()
            .and_then(|b| b.folders.iter().find(|f| f.id == folder_id))
            .map(|f| f.child_board_id.clone())
            .ok_or_else(|| CoreError::FolderNotFound(folder_id.to_string()))?;

        self.folder_stack
            .push((current_board_id, folder_id.to_string()));
        self.project.active_board_id = target;
        self.clear_selection();
        Ok(())
    }

    /// Enveloppe silencieuse de [`Store::try_enter_folder`].
    #[deprecated(note = "R-35 : l'échec est silencieux. Utiliser `try_enter_folder`.")]
    pub fn enter_folder(&mut self, folder_id: &str) {
        drop(self.try_enter_folder(folder_id));
    }

    pub fn exit_folder(&mut self) {
        if let Some((parent_board_id, _)) = self.folder_stack.pop() {
            self.project.active_board_id = parent_board_id;
            self.clear_selection();
        }
    }

    pub fn exit_to_root(&mut self) {
        if let Some((root_board_id, _)) = self.folder_stack.first().cloned() {
            self.folder_stack.clear();
            self.project.active_board_id = root_board_id;
            self.clear_selection();
        }
    }

    pub fn expand_folder(
        &mut self,
        _parent_board_id: &str,
        folder_id: &str,
        level: FolderTreeNode,
    ) {
        let child_board_id = self
            .project
            .boards
            .iter()
            .find_map(|b| b.folders.iter().find(|f| f.id == folder_id))
            .map(|f| f.child_board_id.clone());

        if let Some(child_id) = child_board_id {
            if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == child_id) {
                b.annotations.extend(level.annotations);
                b.images.extend(level.images);
            }
        }
    }

    pub fn set_temporal_filter(&mut self, filter: Option<TemporalAnchor>) {
        self.temporal_filter = filter;
    }
}
