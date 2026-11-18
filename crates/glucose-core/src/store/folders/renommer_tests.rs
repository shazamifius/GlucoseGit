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
    store.selected_folder_id = Some(id.clone());
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
    store.selected_folder_id = None;
    assert_eq!(store.dossier_choisi_et_son_nom(), None);
}
