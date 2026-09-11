//! Pile Undo/Redo infinie et transactions live.
//!
//! INVARIANT UNDO-1 — la caméra et le board actif sont préservés à travers un undo/redo :
//! annuler un déplacement ne doit jamais téléporter l'utilisateur ailleurs dans le document.

use super::{build_folder_stack, Store};
use crate::types::Project;

/// Préserve la caméra et le board actif lors d'un Undo ou Redo.
pub fn preserve_view(restored: &mut Project, cur: &Project) {
    if restored.boards.iter().any(|b| b.id == cur.active_board_id) {
        restored.active_board_id = cur.active_board_id.clone();
    } else if let Some(first) = restored.boards.first() {
        restored.active_board_id = first.id.clone();
    }
    for b in &mut restored.boards {
        if let Some(cb) = cur.boards.iter().find(|x| x.id == b.id) {
            b.viewport = cb.viewport;
        }
    }
}

impl Store {
    pub fn push_undo(&mut self) {
        if self.in_live_edit {
            return;
        }
        self.undo_stack.push_back(self.project.clone());
        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.pop_front();
        }
        self.redo_stack.clear();
        self.bump_version();
    }

    /// Ouvre une transaction : l'état d'avant est mis de côté, et tout `push_undo` jusqu'à
    /// `end_live_edit` est absorbé.
    ///
    /// La version n'avance pas ici : rien n'a encore changé. C'est `end_live_edit` qui la
    /// fait avancer, une fois le geste écrit ; et un geste abandonné (`cancel_live_edit`)
    /// ou resté immobile ne la touche pas — le document n'est pas « modifié » pour un clic.
    pub fn begin_live_edit(&mut self) {
        if self.in_live_edit {
            return;
        }
        self.undo_stack.push_back(self.project.clone());
        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.pop_front();
        }
        self.redo_stack.clear();
        self.in_live_edit = true;
    }

    pub fn end_live_edit(&mut self) {
        self.in_live_edit = false;
        self.bump_version();
    }

    /// Abandonne la transaction live en cours (`Échap` pendant un geste).
    ///
    /// Le document revient à l'état d'avant `begin_live_edit`, et l'entrée d'undo posée à
    /// l'ouverture disparaît avec lui : un geste annulé ne laisse **aucune** trace dans la
    /// pile, ni à annuler ni à rétablir. La sélection est conservée — les nœuds existent
    /// toujours — et la caméra aussi (UNDO-1). La version ne bouge pas : le document est
    /// exactement celui de la version courante, que les index connaissent déjà. Rend `false`
    /// hors transaction.
    pub fn cancel_live_edit(&mut self) -> bool {
        if !self.in_live_edit {
            return false;
        }
        self.in_live_edit = false;
        let Some(mut before) = self.undo_stack.pop_back() else {
            return false;
        };
        preserve_view(&mut before, &self.project);
        self.project = before;
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        let Some(mut prev) = self.undo_stack.pop_back() else {
            return false;
        };
        let current = self.project.clone();
        preserve_view(&mut prev, &current);
        self.redo_stack.push_back(current);
        self.adopt_restored_project(prev);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(mut next) = self.redo_stack.pop_back() else {
            return false;
        };
        let current = self.project.clone();
        preserve_view(&mut next, &current);
        self.undo_stack.push_back(current);
        self.adopt_restored_project(next);
        true
    }

    /// Partie commune de `undo` et `redo` : le document restauré redevient le document courant.
    fn adopt_restored_project(&mut self, project: Project) {
        self.project = project;
        self.clear_selection();
        self.folder_stack = build_folder_stack(&self.project.boards, &self.project.active_board_id);
        self.in_live_edit = false;
        self.bump_version();
    }
}
