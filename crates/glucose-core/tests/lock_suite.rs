//! Fiche 08 § 1.3 — le verrouillage d'images : ce qu'il empêche, et ce qu'il n'empêche pas.

use glucose_core::store::Store;
use glucose_core::types::BoardImage;

/// Un board portant `n` images, aucune verrouillée.
fn store_with(n: usize) -> (Store, String) {
    let mut store = Store::new("verrou");
    let board = store.project.active_board_id.clone();
    for k in 0..n {
        store.add_image(
            &board,
            BoardImage::new(format!("i{k}"), 0.0, 0.0, 100.0, 100.0),
        );
    }
    store.journal.clear();
    (store, board)
}

fn est_verrouillee(store: &Store, id: &str) -> bool {
    store
        .active_board()
        .and_then(|b| b.images.iter().find(|i| i.id == id))
        .map(|i| i.locked)
        .unwrap_or(false)
}

fn select(store: &mut Store, ids: &[&str]) {
    store.set_selected_image_ids(ids.iter().map(|s| s.to_string()).collect());
}

#[test]
fn test_le_verrou_est_une_bascule() {
    let (mut store, b) = store_with(2);
    select(&mut store, &["i0"]);

    assert_eq!(store.toggle_lock_selection(&b), Some(true));
    assert!(est_verrouillee(&store, "i0"));
    assert!(!est_verrouillee(&store, "i1"), "et seulement la sélection");

    assert_eq!(store.toggle_lock_selection(&b), Some(false));
    assert!(!est_verrouillee(&store, "i0"));
}

/// Une bascule **de groupe** : tant qu'il reste une image libre, tout se ferme. Basculer
/// chaque image pour son compte scinderait une sélection mixte en deux moitiés qui
/// s'inversent à chaque appui, et le raccourci ne voudrait plus rien dire.
#[test]
fn test_une_selection_mixte_se_verrouille_entierement_avant_de_s_ouvrir() {
    let (mut store, b) = store_with(3);
    select(&mut store, &["i0"]);
    store.toggle_lock_selection(&b);

    select(&mut store, &["i0", "i1", "i2"]);
    assert_eq!(
        store.toggle_lock_selection(&b),
        Some(true),
        "deux libres sur trois : tout se verrouille"
    );
    assert!((0..3).all(|k| est_verrouillee(&store, &format!("i{k}"))));

    assert_eq!(
        store.toggle_lock_selection(&b),
        Some(false),
        "toutes fermées : le geste suivant libère"
    );
    assert!((0..3).all(|k| !est_verrouillee(&store, &format!("i{k}"))));
}

#[test]
fn test_verrouiller_est_un_seul_geste_annulable() {
    let (mut store, b) = store_with(4);
    select(&mut store, &["i0", "i1", "i2", "i3"]);
    store.toggle_lock_selection(&b);
    assert_eq!(store.undo_depth(), 1, "quatre images, un geste");

    assert!(store.undo());
    assert!((0..4).all(|k| !est_verrouillee(&store, &format!("i{k}"))));
    assert!(store.redo());
    assert!((0..4).all(|k| est_verrouillee(&store, &format!("i{k}"))));
}

/// Un geste sans effet n'entre pas dans l'historique : reverrouiller ce qui l'est déjà ne
/// doit pas coûter un `Ctrl+Z` de plus à l'utilisateur.
#[test]
fn test_reverrouiller_ce_qui_l_est_deja_n_inscrit_rien() {
    let (mut store, b) = store_with(2);
    select(&mut store, &["i0", "i1"]);
    store.toggle_lock_selection(&b);
    let apres_premier = store.undo_depth();

    // La sélection est entièrement verrouillée : le geste suivant ouvre, celui d'après ferme.
    store.toggle_lock_selection(&b);
    store.toggle_lock_selection(&b);
    assert_eq!(
        store.undo_depth(),
        apres_premier + 2,
        "deux gestes réels, deux entrées"
    );
}

#[test]
fn test_sans_image_selectionnee_le_verrou_ne_dit_rien() {
    let (mut store, b) = store_with(2);
    store.clear_selection();
    assert_eq!(
        store.toggle_lock_selection(&b),
        None,
        "une carte ou un dossier ne porte pas de verrou : la référence n'en donne qu'aux images"
    );
    assert_eq!(store.undo_depth(), 0);
}

/// Le verrou tient ce qu'il promet : l'image ne bouge plus.
#[test]
fn test_une_image_verrouillee_ne_se_deplace_plus() {
    let (mut store, b) = store_with(2);
    select(&mut store, &["i0"]);
    store.toggle_lock_selection(&b);

    select(&mut store, &["i0", "i1"]);
    store.move_selected(&b, 50.0, 25.0);

    let board = store.active_board().expect("un board");
    let i0 = board.images.iter().find(|i| i.id == "i0").expect("i0");
    let i1 = board.images.iter().find(|i| i.id == "i1").expect("i1");
    assert_eq!((i0.x, i0.y), (0.0, 0.0), "la verrouillée n'a pas bougé");
    assert_eq!((i1.x, i1.y), (50.0, 25.0), "l'autre, si");
}
