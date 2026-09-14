//! Ce que l'ordre d'empilement garantit — et ce qu'il refuse d'inscrire dans l'historique.

use super::*;
use crate::types::{Annotation, BoardImage};

/// Un board portant `n` images nommées `i0`, `i1`, … dans cet ordre.
fn store_with(n: usize) -> Store {
    let mut store = Store::new("ordre");
    let board = store.project.active_board_id.clone();
    for k in 0..n {
        store.add_image(
            &board,
            BoardImage::new(format!("i{k}"), 0.0, 0.0, 100.0, 100.0),
        );
    }
    store.journal.clear();
    store
}

/// L'ordre des images du board actif, de l'arrière vers l'avant.
fn ordre(store: &Store) -> Vec<String> {
    store
        .active_board()
        .map(|b| b.images.iter().map(|i| i.id.clone()).collect())
        .unwrap_or_default()
}

fn board_id(store: &Store) -> String {
    store.project.active_board_id.clone()
}

fn select(store: &mut Store, ids: &[&str]) {
    store.set_selected_image_ids(ids.iter().map(|s| s.to_string()).collect());
}

#[test]
fn test_front_puts_the_selection_ahead_of_everything_else() {
    let mut store = store_with(5);
    let b = board_id(&store);
    select(&mut store, &["i1"]);
    assert_eq!(store.move_selection_in_stack(&b, StackMove::Front), 1);
    assert_eq!(ordre(&store), ["i0", "i2", "i3", "i4", "i1"]);
}

#[test]
fn test_back_puts_the_selection_behind_everything_else() {
    let mut store = store_with(5);
    let b = board_id(&store);
    select(&mut store, &["i3"]);
    assert_eq!(store.move_selection_in_stack(&b, StackMove::Back), 1);
    assert_eq!(ordre(&store), ["i3", "i0", "i1", "i2", "i4"]);
}

/// ORDER-1 — plusieurs nœuds montent ensemble sans se mélanger entre eux.
#[test]
fn test_order_1_a_multiple_selection_keeps_its_internal_order() {
    let mut store = store_with(6);
    let b = board_id(&store);
    select(&mut store, &["i1", "i4", "i0"]);
    assert_eq!(store.move_selection_in_stack(&b, StackMove::Front), 3);
    assert_eq!(
        ordre(&store),
        ["i2", "i3", "i5", "i0", "i1", "i4"],
        "i0 était devant i1, qui était devant i4 : ils le restent"
    );

    let mut store = store_with(6);
    let b = board_id(&store);
    select(&mut store, &["i1", "i4", "i0"]);
    assert_eq!(store.move_selection_in_stack(&b, StackMove::Back), 3);
    assert_eq!(ordre(&store), ["i0", "i1", "i4", "i2", "i3", "i5"]);
}

#[test]
fn test_undo_restores_the_exact_previous_order() {
    for mv in [StackMove::Front, StackMove::Back] {
        let mut store = store_with(6);
        let b = board_id(&store);
        let avant = ordre(&store);
        select(&mut store, &["i1", "i4"]);
        store.move_selection_in_stack(&b, mv);
        assert_ne!(ordre(&store), avant, "{mv:?} n'a rien fait");

        assert!(store.undo(), "{mv:?} : rien à défaire");
        assert_eq!(ordre(&store), avant, "{mv:?} : l'ordre n'est pas revenu");

        assert!(store.redo(), "{mv:?} : rien à refaire");
        assert_ne!(
            ordre(&store),
            avant,
            "{mv:?} : le rétablissement n'a rien fait"
        );
    }
}

/// Un geste sans effet n'entre pas dans l'historique : sinon `Ctrl+Z` serait à faire deux
/// fois pour annuler la dernière chose qui a vraiment changé.
#[test]
fn test_a_selection_already_at_the_edge_records_nothing() {
    let mut store = store_with(4);
    let b = board_id(&store);

    select(&mut store, &["i3"]);
    assert_eq!(store.move_selection_in_stack(&b, StackMove::Front), 0);
    assert_eq!(store.undo_depth(), 0);

    select(&mut store, &["i0", "i1"]);
    assert_eq!(store.move_selection_in_stack(&b, StackMove::Back), 0);
    assert_eq!(store.undo_depth(), 0);

    // En revanche, deux nœuds au bord mais **séparés** ont bien quelque chose à faire :
    // se rassembler.
    select(&mut store, &["i0", "i2"]);
    assert_eq!(store.move_selection_in_stack(&b, StackMove::Back), 2);
    assert_eq!(ordre(&store), ["i0", "i2", "i1", "i3"]);
}

#[test]
fn test_nothing_selected_changes_nothing() {
    let mut store = store_with(3);
    let b = board_id(&store);
    store.clear_selection();
    assert_eq!(store.move_selection_in_stack(&b, StackMove::Front), 0);
    assert_eq!(ordre(&store), ["i0", "i1", "i2"]);
    assert_eq!(store.undo_depth(), 0);
}

/// Le geste est **un** geste : une seule entrée d'historique, quel que soit le nombre de
/// nœuds déplacés (fiche 09 § 2, R1).
#[test]
fn test_moving_many_nodes_is_one_single_undo_step() {
    let mut store = store_with(8);
    let b = board_id(&store);
    select(&mut store, &["i0", "i2", "i4", "i6"]);
    store.move_selection_in_stack(&b, StackMove::Front);
    assert_eq!(store.undo_depth(), 1, "quatre nœuds, un geste");
    store.undo();
    assert_eq!(
        ordre(&store),
        ["i0", "i1", "i2", "i3", "i4", "i5", "i6", "i7"]
    );
}

/// Images et annotations sont deux couches : chacune se réordonne chez elle, et le geste
/// reste unique.
#[test]
fn test_images_and_annotations_restack_in_their_own_layer() {
    let mut store = store_with(3);
    let b = board_id(&store);
    for k in 0..3 {
        store.add_annotation(&b, Annotation::text(format!("a{k}"), 0.0, 0.0, "x"));
    }
    store.journal.clear();

    store.set_selected_image_ids(vec!["i0".into()]);
    store.set_selected_annotation_ids(vec!["a0".into()]);
    assert_eq!(store.move_selection_in_stack(&b, StackMove::Front), 2);

    let board = store.active_board().expect("un board");
    assert_eq!(
        board
            .images
            .iter()
            .map(|i| i.id.as_str())
            .collect::<Vec<_>>(),
        ["i1", "i2", "i0"]
    );
    assert_eq!(
        board.annotations.iter().map(|a| a.id()).collect::<Vec<_>>(),
        ["a1", "a2", "a0"]
    );
    assert_eq!(store.undo_depth(), 1, "les deux couches, un seul geste");
}
