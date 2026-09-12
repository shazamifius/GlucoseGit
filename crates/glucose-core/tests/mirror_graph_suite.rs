//! Tests portés fidèlement de src/store/mirrorGraph.test.ts

use glucose_core::mirror_graph::{find_board_containing_folder, would_create_mirror_cycle};
use glucose_core::types::{Board, CanvasFolder};

fn make_folder(id: &str, child_board_id: &str) -> CanvasFolder {
    CanvasFolder {
        id: id.into(),
        name: id.into(),
        color: "#fff".into(),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        child_board_id: child_board_id.into(),
        mirror_of: None,
        mirror_source: None,
    }
}

fn make_board(id: &str, folders: &[(&str, &str)]) -> Board {
    let mut b = Board::new(id, id);
    b.folders = folders
        .iter()
        .map(|&(fid, cid)| make_folder(fid, cid))
        .collect();
    b
}

#[test]
fn test_ghost_folder_returns_false() {
    let boards = [make_board("main", &[])];
    assert!(!would_create_mirror_cycle(&boards, "ghost-folder", "main"));
}

#[test]
fn test_mirror_in_own_child_detected() {
    let boards = [
        make_board("main", &[("folderA", "childA")]),
        make_board("childA", &[]),
    ];
    assert!(would_create_mirror_cycle(&boards, "folderA", "childA"));
}

#[test]
fn test_indirect_cycle_detected() {
    let boards = [
        make_board("main", &[("folderA", "childA")]),
        make_board("childA", &[("folderB", "childB")]),
        make_board("childB", &[]),
    ];
    assert!(would_create_mirror_cycle(&boards, "folderA", "childB"));
}

#[test]
fn test_independent_siblings_no_cycle() {
    let boards = [
        make_board("main", &[("folderA", "childA"), ("folderB", "childB")]),
        make_board("childA", &[]),
        make_board("childB", &[]),
    ];
    assert!(!would_create_mirror_cycle(&boards, "folderA", "childB"));
}

#[test]
fn test_deep_cycle_detected() {
    let boards = [
        make_board("main", &[("fA", "childA")]),
        make_board("childA", &[("fB", "childB")]),
        make_board("childB", &[("fC", "childC")]),
        make_board("childC", &[]),
    ];
    assert!(would_create_mirror_cycle(&boards, "fA", "childC"));
}

#[test]
fn test_pre_cyclic_graph_visited_set_prevents_loop() {
    let boards = [
        make_board("main", &[("fA", "childA"), ("fB", "childB")]),
        make_board("childA", &[("fB-mirror", "childB")]),
        make_board("childB", &[("fA-mirror", "childA")]),
    ];
    let res = would_create_mirror_cycle(&boards, "fA", "childA");
    assert!(res);
}

#[test]
fn test_find_board_containing_folder() {
    let boards = [
        make_board("b1", &[("f1", "b2")]),
        make_board("b2", &[("f2", "b3")]),
    ];
    assert_eq!(
        find_board_containing_folder(&boards, "f1").map(|b| b.id.as_str()),
        Some("b1")
    );
    assert_eq!(
        find_board_containing_folder(&boards, "f2").map(|b| b.id.as_str()),
        Some("b2")
    );
    assert_eq!(find_board_containing_folder(&boards, "ghost"), None);
}

/// Fiche 08 § 6.2 — la fiche parlait d'une « limite de profondeur de chaîne fixée à 16 ».
/// Le parcours en largeur avec ensemble des visités n'a pas besoin de limite : un cycle est
/// détecté à toute profondeur, et un graphe sain de toute profondeur est accepté. La
/// constante a disparu au lieu d'être réglée ; ce test le tient à 40 niveaux.
#[test]
fn test_a_cycle_is_detected_at_any_depth_without_a_limit() {
    let depth = 40;
    let mut boards = Vec::new();
    for i in 0..depth {
        let folder = format!("f{i}");
        let child = format!("b{}", i + 1);
        boards.push(make_board(&format!("b{i}"), &[(folder.as_str(), child.as_str())]));
    }
    boards.push(make_board(&format!("b{depth}"), &[]));

    assert!(
        would_create_mirror_cycle(&boards, "f0", &format!("b{depth}")),
        "un miroir de f0 tout au fond de sa propre descendance boucle"
    );
    assert!(
        !would_create_mirror_cycle(&boards, &format!("f{}", depth - 1), "b0"),
        "un miroir du dernier dossier à la racine ne boucle pas : b0 n'est pas sous lui"
    );
}
