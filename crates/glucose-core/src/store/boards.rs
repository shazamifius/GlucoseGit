//! Création, renommage et suppression de tableaux.

use super::journal::{Edit, Slot, Whole};
use super::Store;
use crate::error::{CoreError, CoreResult};
use crate::types::{Annotation, Board};

impl Store {
    /// **Site migré vers le journal.** Renommer ne clone pas le board : l'entrée ne porte
    /// que les deux noms.
    pub fn rename_board(&mut self, board_id: &str, name: impl Into<String>) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let before = b.name.clone();
        b.name = name.into();
        let after = b.name.clone();
        self.record_edit(Edit::BoardName {
            board: board_id.to_string(),
            whole: Whole::new(before, after),
        });
    }

    /// **Site migré vers le journal.**
    pub fn add_board(&mut self, name: impl Into<String>) -> String {
        let id = self.generate_id("board");
        let board = Board::new(&id, name);
        let index = self.project.boards.len();
        self.project.boards.push(board.clone());
        self.record_edit(Edit::Board { slot: Slot::inserted(index, board) });
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
        // Un seul geste : le board part, le board actif bascule, et les flèches portails qui
        // le visaient sont neutralisées. Un Ctrl+Z remet le tout.
        let Some(index) = self.project.boards.iter().position(|b| b.id == id) else {
            return Err(CoreError::BoardNotFound(id.to_string()));
        };
        let removed = self.project.boards.remove(index);
        let mut edits = vec![Edit::Board { slot: Slot::removed(index, removed) }];

        if self.project.active_board_id == id {
            if let Some(first) = self.project.boards.first() {
                let before = self.project.active_board_id.clone();
                let after = first.id.clone();
                self.project.active_board_id = after.clone();
                edits.push(Edit::ActiveBoard { whole: Whole::new(before, after) });
            }
        }
        edits.extend(self.clear_portal_arrows_to(id));

        self.record_as_one_gesture(edits);
        Ok(())
    }

    /// Enveloppe silencieuse de [`Store::try_remove_board`]. Rend `false` en cas d'échec, sans
    /// dire lequel des deux (tableau inexistant / dernier tableau) s'est produit.
    #[deprecated(note = "R-35 : l'échec est muet. Utiliser `try_remove_board`.")]
    pub fn remove_board(&mut self, id: &str) -> bool {
        self.try_remove_board(id).is_ok()
    }

    /// Neutralise les flèches portails qui pointaient vers un tableau supprimé, et rend les
    /// éditions correspondantes.
    ///
    /// Seule une flèche réellement concernée est clonée : le coût suit le nombre de portails
    /// cassés, pas le nombre d'annotations du projet.
    fn clear_portal_arrows_to(&mut self, removed_board_id: &str) -> Vec<Edit> {
        let mut edits = Vec::new();
        for b in &mut self.project.boards {
            for (i, a) in b.annotations.iter_mut().enumerate() {
                let concerned = matches!(
                    a,
                    Annotation::Arrow { target_board_id, .. }
                        if target_board_id.as_deref() == Some(removed_board_id)
                );
                if !concerned {
                    continue;
                }
                let before = a.clone();
                if let Annotation::Arrow { target_board_id, .. } = a {
                    *target_board_id = None;
                }
                edits.push(Edit::Annotation {
                    board: b.id.clone(),
                    slot: Slot::changed(i, before, a.clone()),
                });
            }
        }
        edits
    }
}
