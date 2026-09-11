//! Index spatial — invariants SPAT-1 (incrémental) et SPAT-2 (sans allocation par requête).
//! Ces tests prouvent que le chemin réellement emprunté par le renderer (`index_board` à chaque
//! changement de `store.version`, donc à chaque événement souris) ne reconstruit plus l'index.

use glucose_core::quadtree::SpatialHash;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Board, BoardImage};

fn mk_text(id: &str, x: f64, y: f64) -> Annotation {
    Annotation::Text {
        id: id.to_string(),
        x,
        y,
        width: Some(200.0),
        height: Some(50.0),
        text: "Hello".into(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

fn mk_image(id: &str, x: f64, y: f64) -> BoardImage {
    let mut img = BoardImage::new(id, 0.0, 0.0, 200.0, 200.0);
    img.x = x;
    img.y = y;
    img
}

#[test]
fn test_index_board_place_images_et_annotations() {
    let mut board = Board::new("board-1", "Test Board");
    board.images.push(mk_image("img-1", 200.0, 200.0));
    board.annotations.push(mk_text("text-1", 3000.0, 3000.0));

    let mut sh = SpatialHash::new(1000.0);
    sh.index_board(&board);

    let near = sh.query_rect(0.0, 0.0, 500.0, 500.0, 50.0);
    assert!(near.contains("img-1"));
    assert!(!near.contains("text-1"));

    let far = sh.query_rect(2800.0, 2800.0, 3200.0, 3200.0, 50.0);
    assert!(!far.contains("img-1"));
    assert!(far.contains("text-1"));
}

/// SPAT-1 — Le cas exact de R-39 : un drag émet ~100 `CursorMoved` par seconde, chacun
/// incrémentant `store.version`, donc chacun rappelant `index_board`. Après 200 pas d'un
/// pixel sur un board de 2 000 nœuds, l'index doit être exact SANS aucune reconstruction.
#[test]
fn test_drag_de_200_pas_nindexe_jamais_tout_le_board() {
    let mut board = Board::new("b", "Drag");
    for i in 0..2000 {
        board.images.push(mk_image(&format!("img-{}", i), (i % 50) as f64 * 300.0, (i / 50) as f64 * 300.0));
    }

    // Le nœud mobile est placé au cœur d'une cellule : 200 pas d'1 px ne la quittent pas.
    board.images[0].x = 400.0;
    board.images[0].y = 400.0;

    let mut sh = SpatialHash::new(1000.0);
    sh.index_board(&board);
    let rebuilds_apres_indexation = sh.rebuild_count();
    let ecritures_apres_indexation = sh.cell_write_count();
    assert_eq!(sh.len(), 2000);

    // 200 déplacements d'un seul nœud, de 1 px chacun (largement dans sa cellule de 1 000 px).
    for _ in 0..200 {
        board.images[0].x += 1.0;
        sh.index_board(&board);
    }

    assert_eq!(
        sh.rebuild_count(),
        rebuilds_apres_indexation,
        "SPAT-1 violé : index_board a reconstruit l'index pendant le drag"
    );
    assert_eq!(
        sh.cell_write_count(),
        ecritures_apres_indexation,
        "SPAT-1 violé : un déplacement intra-cellule ne doit provoquer aucune écriture de grille"
    );

    // L'index reste exact.
    assert_eq!(sh.len(), 2000);
    let visible = sh.query_rect_refs(300.0, 300.0, 700.0, 700.0, 0.0);
    assert!(visible.contains("img-0"));
    assert!(!visible.contains("img-1999"));
}

/// SPAT-1 — Un déplacement qui traverse une frontière de cellule met bien l'index à jour,
/// et seules les cellules concernées sont réécrites.
#[test]
fn test_traversee_de_cellule_met_a_jour_lindex_localement() {
    let mut board = Board::new("b", "Cross");
    for i in 0..500 {
        board.images.push(mk_image(&format!("img-{}", i), 5000.0 + i as f64, 5000.0));
    }
    board.images.push(mk_image("mobile", 100.0, 100.0));

    let mut sh = SpatialHash::new(1000.0);
    sh.index_board(&board);
    let avant = sh.cell_write_count();

    // Traversée franche vers une cellule éloignée.
    if let Some(img) = board.images.iter_mut().find(|i| i.id == "mobile") {
        img.x = 20_100.0;
        img.y = 20_100.0;
    }
    sh.index_board(&board);

    let ecritures = sh.cell_write_count() - avant;
    assert!(
        ecritures <= 8,
        "un seul nœud a bougé : {} écritures de cellule, attendu <= 8",
        ecritures
    );
    assert!(sh.query_rect_refs(0.0, 0.0, 500.0, 500.0, 0.0).is_empty());
    assert!(sh.query_rect_refs(20_000.0, 20_000.0, 20_500.0, 20_500.0, 0.0).contains("mobile"));
    assert_eq!(sh.rebuild_count(), 0);
}

/// SPAT-1 — Le balayage retire les nœuds supprimés du board sans reconstruction.
#[test]
fn test_suppression_dun_noeud_est_propagee_sans_reconstruction() {
    let mut board = Board::new("b", "Sweep");
    board.images.push(mk_image("a", 100.0, 100.0));
    board.images.push(mk_image("b", 100.0, 100.0));
    board.annotations.push(mk_text("t", 100.0, 100.0));

    let mut sh = SpatialHash::new(1000.0);
    sh.index_board(&board);
    assert_eq!(sh.len(), 3);

    board.images.retain(|i| i.id != "a");
    board.annotations.clear();
    sh.index_board(&board);

    assert_eq!(sh.len(), 1);
    assert!(!sh.contains("a"));
    assert!(!sh.contains("t"));
    assert!(sh.contains("b"));
    assert_eq!(sh.query_rect_refs(0.0, 0.0, 500.0, 500.0, 0.0).len(), 1);
    assert_eq!(sh.rebuild_count(), 0, "aucune reconstruction ne doit avoir eu lieu");
}

/// SPAT-1 — Changer de board actif réutilise le même index : les nœuds de l'ancien board
/// disparaissent par balayage, pas par `clear`.
#[test]
fn test_changement_de_board_est_une_synchronisation() {
    let mut b1 = Board::new("b1", "Un");
    b1.images.push(mk_image("x", 100.0, 100.0));
    let mut b2 = Board::new("b2", "Deux");
    b2.images.push(mk_image("y", 100.0, 100.0));

    let mut sh = SpatialHash::new(1000.0);
    sh.index_board(&b1);
    sh.index_board(&b2);

    assert!(sh.contains("y"));
    assert!(!sh.contains("x"));
    assert_eq!(sh.rebuild_count(), 0, "index_board ne doit jamais appeler clear()");
}

/// L'index reste cohérent avec le `Store` réel après une suite de mutations applicatives :
/// c'est le chemin qu'emprunte `renderer.rs` (index_board(store.active_board())).
#[test]
fn test_index_suit_les_mutations_du_store() {
    let mut store = Store::new("P");
    store.add_annotation("main", mk_text("t1", 0.0, 0.0));
    store.add_annotation("main", mk_text("t2", 4000.0, 4000.0));

    let mut sh = SpatialHash::new(1000.0);
    let board = store.active_board().expect("board actif créé par Store::new");
    sh.index_board(board);
    assert_eq!(sh.len(), 2);

    store.select_annotation("t1".into(), false);
    store.begin_live_edit();
    for _ in 0..50 {
        store.move_selected("main", 2.0, 2.0);
        let board = store.active_board().expect("board actif toujours présent");
        sh.index_board(board);
    }
    store.end_live_edit();

    assert_eq!(sh.rebuild_count(), 0);
    assert_eq!(sh.len(), 2);
    assert!(sh.query_rect_refs(0.0, 0.0, 300.0, 300.0, 0.0).contains("t1"));

    store.delete_selected("main");
    let board = store.active_board().expect("board actif toujours présent");
    sh.index_board(board);
    assert!(!sh.contains("t1"));
    assert!(sh.contains("t2"));
}

/// SPAT-1 — Le cache de résolution positionnel ne doit jamais mentir : si le board réordonne
/// ses nœuds (z-order, duplication, suppression au milieu), l'index doit rester exact.
#[test]
fn test_reordonner_le_board_ne_corrompt_pas_lindex() {
    let mut board = Board::new("b", "Reorder");
    for i in 0..10 {
        board.images.push(mk_image(&format!("img-{}", i), i as f64 * 2000.0, 0.0));
    }

    let mut sh = SpatialHash::new(1000.0);
    sh.index_board(&board);

    board.images.reverse();
    sh.index_board(&board);
    assert_eq!(sh.len(), 10);
    for i in 0..10 {
        let id = format!("img-{}", i);
        assert!(sh.contains(&id), "{} a disparu apres reordonnancement", id);
        let x = i as f64 * 2000.0;
        assert!(sh.query_rect_refs(x - 50.0, -50.0, x + 50.0, 50.0, 0.0).contains(id.as_str()));
    }

    // Suppression au milieu, puis insertion d'un nouveau nœud : décalage de toutes les
    // positions suivantes, le cache positionnel doit se resynchroniser.
    board.images.remove(4);
    board.images.insert(2, mk_image("neuf", 50_000.0, 0.0));
    sh.index_board(&board);
    assert_eq!(sh.len(), 10);
    assert!(sh.contains("neuf"));
    assert_eq!(sh.rebuild_count(), 0);
    for img in &board.images {
        assert!(sh.contains(&img.id), "{} absent de l index", img.id);
    }
}
