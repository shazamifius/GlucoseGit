//! RENOMMER-1 : nommer un dossier (fiche 120).

use crate::store::Store;
use crate::types::CanvasFolder;

fn un_dossier_choisi() -> (Store, String, String) {
    let mut store = Store::new("renommer");
    let board = store.project.active_board_id.clone();
    store.create_folder(&board, CanvasFolder::new("", "Dossier", ""));
    let id = store.active_board().expect("un tableau").folders[0]
        .id
        .clone();
    store.selected_folder_ids = vec![id.clone()].into();
    (store, board, id)
}

fn nom(store: &Store) -> String {
    store.active_board().expect("un tableau").folders[0]
        .name
        .clone()
}

#[test]
fn test_renommer_1_le_dossier_choisi_se_nomme_et_ctrl_z_le_defait() {
    let (mut store, board, id) = un_dossier_choisi();
    assert_eq!(
        store.dossier_choisi_et_son_nom(),
        Some((id.as_str(), "Dossier"))
    );
    assert!(store.renommer_le_dossier(&board, &id, "  Références de Mary  "));
    assert_eq!(nom(&store), "Références de Mary", "le nom, sans ses blancs");
    assert!(store.undo(), "un seul geste");
    assert_eq!(nom(&store), "Dossier");
    assert!(store.redo());
    assert_eq!(nom(&store), "Références de Mary");
}

#[test]
fn test_renommer_1_un_nom_vide_ou_le_meme_ne_nomme_rien() {
    let (mut store, board, id) = un_dossier_choisi();
    let version = store.version;
    assert!(!store.renommer_le_dossier(&board, &id, "   "));
    assert!(!store.renommer_le_dossier(&board, &id, "Dossier"));
    assert!(!store.renommer_le_dossier(&board, "un-autre", "Nom"));
    assert_eq!(nom(&store), "Dossier");
    assert_eq!(store.version, version, "rien n'a été fait : aucun geste");
}

#[test]
fn test_renommer_1_sans_dossier_choisi_rien_n_est_propose() {
    let (mut store, _, _) = un_dossier_choisi();
    store.selected_folder_ids.clear();
    assert_eq!(store.dossier_choisi_et_son_nom(), None);
}

/// **DOSSIER-2 — deux dossiers choisis, rien à renommer** : le nom demandé serait celui d'un seul,
/// au hasard de l'ordre (fiche 131).
#[test]
fn test_dossier_2_deux_dossiers_choisis_rien_a_renommer() {
    let (mut store, board, id) = un_dossier_choisi();
    store.create_folder(&board, CanvasFolder::new("", "Autre", ""));
    let autre = store.active_board().expect("un tableau").folders[1]
        .id
        .clone();
    store.select_folder(autre, true);
    assert_eq!(store.selected_folder_ids.len(), 2);
    assert_eq!(store.dossier_choisi_et_son_nom(), None);
    store.select_folder(id.clone(), false);
    assert_eq!(
        store.dossier_choisi_et_son_nom(),
        Some((id.as_str(), "Dossier"))
    );
}

/// **DOSSIER-2 — choisir un dossier seul efface la sélection d'avant** (un clic sans Maj) ; avec
/// Maj, il s'y ajoute.
#[test]
fn test_dossier_2_choisir_seul_efface_la_selection_d_avant() {
    let (mut store, board, id) = un_dossier_choisi();
    let t = crate::types::Annotation::text("t", 0.0, 0.0, "un texte");
    store.add_annotation(&board, t);
    store.select_annotation("t".into(), true);
    store.select_folder(id.clone(), true);
    assert_eq!(
        (
            store.selected_annotation_ids.len(),
            store.selected_folder_ids.len()
        ),
        (1, 1)
    );
    store.select_folder(id, false);
    assert_eq!(
        (
            store.selected_annotation_ids.len(),
            store.selected_folder_ids.len()
        ),
        (0, 1)
    );
}
