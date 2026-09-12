//! Vérificateur acyclique pour les miroirs de dossiers (anti-inception) — 0 dépendance.

use crate::types::Board;
use std::collections::{HashMap, HashSet, VecDeque};

/// Renvoie `true` si placer un miroir du dossier `original_folder_id` à l'intérieur
/// du board `target_board_id` créerait un cycle d'imbrication infinie (Inception).
pub fn would_create_mirror_cycle(
    boards: &[Board],
    original_folder_id: &str,
    target_board_id: &str,
) -> bool {
    let mut board_by_id = HashMap::new();
    let mut folder_by_id = HashMap::new();

    for b in boards {
        board_by_id.insert(b.id.as_str(), b);
        for f in &b.folders {
            folder_by_id.insert(f.id.as_str(), f);
        }
    }

    let original = match folder_by_id.get(original_folder_id) {
        Some(&f) => f,
        None => return false,
    };

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(original.child_board_id.as_str());

    while let Some(board_id) = queue.pop_front() {
        if board_id == target_board_id {
            return true; // Cycle détecté !
        }
        if visited.contains(board_id) {
            continue;
        }
        visited.insert(board_id);

        if let Some(&b) = board_by_id.get(board_id) {
            for f in &b.folders {
                queue.push_back(f.child_board_id.as_str());
            }
        }
    }

    false
}

pub fn find_board_containing_folder<'a>(boards: &'a [Board], folder_id: &str) -> Option<&'a Board> {
    boards
        .iter()
        .find(|b| b.folders.iter().any(|f| f.id == folder_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CanvasFolder;

    fn folder(id: &str, child_board_id: &str) -> CanvasFolder {
        CanvasFolder {
            id: id.into(),
            name: id.into(),
            color: "#60a5fa".into(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            child_board_id: child_board_id.into(),
            mirror_of: None,
            mirror_source: None,
        }
    }

    fn board(id: &str, folders: Vec<CanvasFolder>) -> Board {
        let mut b = Board::new(id, id);
        b.folders = folders;
        b
    }

    #[test]
    fn test_detect_direct_cycle() {
        // Dossier A dans board1 mène vers board2.
        // Mettre un miroir de A dans board2 ferait board1 -> board2 -> board2 -> ...
        let b1 = board("b1", vec![folder("fA", "b2")]);
        let b2 = board("b2", vec![]);
        let boards = [b1, b2];

        assert!(would_create_mirror_cycle(&boards, "fA", "b2"));
    }

    #[test]
    fn test_detect_indirect_cycle() {
        // b1 -> fA(b2) -> fB(b3) -> fC(b1) -> boucle fermée
        let b1 = board("b1", vec![folder("fA", "b2")]);
        let b2 = board("b2", vec![folder("fB", "b3")]);
        let b3 = board("b3", vec![]);
        let boards = [b1, b2, b3];

        // Mettre un miroir de fA dans b3 fermerait la boucle
        assert!(would_create_mirror_cycle(&boards, "fA", "b3"));
        // Mettre un miroir de fB dans b1 ne crée pas de cycle vers un nouveau board neutre
        let b_neutral = board("b_neutral", vec![]);
        let boards_neutral = [
            boards[0].clone(),
            boards[1].clone(),
            boards[2].clone(),
            b_neutral,
        ];
        assert!(!would_create_mirror_cycle(
            &boards_neutral,
            "fA",
            "b_neutral"
        ));
    }
}
