//! Création, renommage et suppression de tableaux.

use super::Store;
use crate::error::{CoreError, CoreResult};
use crate::types::{Annotation, Board};

impl Store {
    pub fn rename_board(&mut self, board_id: &str, name: impl Into<String>) {
        self.push_undo();
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.name = name.into();
        }
    }

    pub fn add_board(&mut self, name: impl Into<String>) -> String {
        self.push_undo();
        let id = self.generate_id("board");
        self.project.boards.push(Board::new(&id, name));
        id
    }

    /// Supprime un tableau, ou dit pourquoi il ne l'a pas fait (R-35).
    ///
    /// INVARIANT BOARD-1 — un projet contient toujours au moins un tableau : supprimer le
    /// dernier laisserait l'application sans surface de dessin et sans board actif valide.
    pub fn try_remove_board(&mut self, id: &str) -> CoreResult<()> {
        if !self.project.boards.iter().any(|b| b.id == id) {
            return Err(CoreError::BoardNotFound(id.to_string()));
        }
        if self.project.boards.len() <= 1 {
            return Err(CoreError::InvalidOperation(format!(
                "impossible de supprimer '{}' : c'est le dernier tableau du projet — en créer un autre d'abord",
                id
            )));
        }
        self.push_undo();
        self.project.boards.retain(|b| b.id != id);
        if self.project.active_board_id == id {
            if let Some(first) = self.project.boards.first() {
                self.project.active_board_id = first.id.clone();
            }
        }
        self.clear_portal_arrows_to(id);
        Ok(())
    }

    /// Enveloppe silencieuse de [`Store::try_remove_board`]. Rend `false` en cas d'échec, sans
    /// dire lequel des deux (tableau inexistant / dernier tableau) s'est produit.
    #[deprecated(note = "R-35 : l'échec est muet. Utiliser `try_remove_board`.")]
    pub fn remove_board(&mut self, id: &str) -> bool {
        self.try_remove_board(id).is_ok()
    }

    /// Neutralise les flèches portails qui pointaient vers un tableau supprimé.
    fn clear_portal_arrows_to(&mut self, removed_board_id: &str) {
        for b in &mut self.project.boards {
            for a in &mut b.annotations {
                if let Annotation::Arrow { target_board_id, .. } = a {
                    if target_board_id.as_deref() == Some(removed_board_id) {
                        *target_board_id = None;
                    }
                }
            }
        }
    }
}
