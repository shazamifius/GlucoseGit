//! PRESET-1 — poser un gabarit, changer une zone, et l'appartenance d'une image.

use super::*;
use crate::gabarits::integre;

fn store() -> (Store, String) {
    let store = Store::new("zones");
    let board = store.project.active_board_id.clone();
    (store, board)
}

/// Les identifiants des catégories d'un gabarit intégré.
fn categories(id: &str) -> (&'static str, Vec<&'static str>) {
    let g = integre(id).expect("un gabarit intégré");
    (g.id, g.categories.iter().map(|c| c.id).collect())
}

/// **Un gabarit se pose en rangée centrée là où on le pose**, une zone par catégorie, à la
/// taille et à l'écart de Glucose Tauri — et s'annule d'un seul `Ctrl+Z`.
#[test]
fn test_preset_1_un_gabarit_se_pose_centre_et_s_annule_d_un_coup() {
    let (mut store, board) = store();
    let (id, cats) = categories("props");
    store.poser_un_gabarit(&board, (id, &cats), (1000.0, 500.0));
    let zones = store.zones(&board);
    assert_eq!(zones.len(), 5);
    assert_eq!(zones[0].slot_id, "props/ref");
    let largeur = 5.0 * ZONE.0 + 4.0 * ECART;
    assert_eq!(
        (zones[0].x, zones[0].y),
        (1000.0 - largeur / 2.0, 500.0 - ZONE.1 / 2.0)
    );
    assert_eq!(zones[1].x - zones[0].x, ZONE.0 + ECART);
    let derniere = &zones[4];
    assert_eq!(
        derniere.x + derniere.width,
        1000.0 + largeur / 2.0,
        "centrée"
    );

    assert!(store.undo());
    assert!(store.zones(&board).is_empty(), "un seul geste");
}

/// Poser un autre gabarit remplace le premier ; le retirer retire toutes les zones.
#[test]
fn test_preset_1_un_tableau_suit_un_gabarit() {
    let (mut store, board) = store();
    let (a, ca) = categories("props");
    let (b, cb) = categories("storyboard");
    store.poser_un_gabarit(&board, (a, &ca), (0.0, 0.0));
    store.poser_un_gabarit(&board, (b, &cb), (0.0, 0.0));
    assert!(store
        .zones(&board)
        .iter()
        .all(|z| z.slot_id.starts_with("storyboard/")));
    store.retirer_le_gabarit(&board);
    assert!(store.zones(&board).is_empty());
    assert!(store.undo());
    assert_eq!(store.zones(&board).len(), 6);
}

/// **Une zone se déplace, se redimensionne, se retire**, seule, et chaque geste s'annule.
#[test]
fn test_preset_1_une_zone_change_seule() {
    let (mut store, board) = store();
    let (id, cats) = categories("props");
    store.poser_un_gabarit(&board, (id, &cats), (0.0, 0.0));
    let avant: Vec<BoardZone> = store.zones(&board).to_vec();

    store.placer_la_zone(&board, "props/sketch", (5.0, 6.0, 200.0, 300.0));
    let z = store.zones(&board)[1].clone();
    assert_eq!((z.x, z.y, z.width, z.height), (5.0, 6.0, 200.0, 300.0));
    assert_eq!(
        store.zones(&board)[0],
        avant[0],
        "les autres ne bougent pas"
    );

    store.retirer_la_zone(&board, "props/ref");
    assert_eq!(store.zones(&board).len(), 4);
    assert!(store.undo() && store.undo());
    assert_eq!(store.zones(&board), avant.as_slice());
}

/// **Une image appartient à la zone où son centre est posé** — et en change quand on la
/// déplace : rien n'est écrit sur l'image. Deux zones qui se recouvrent : la plus haute.
#[test]
fn test_preset_1_une_image_appartient_a_la_zone_de_son_centre() {
    let (mut store, board) = store();
    store.poser_les_zones(
        &board,
        vec![
            BoardZone::new("g/a", 0.0, 0.0, 100.0, 100.0),
            BoardZone::new("g/b", 50.0, 0.0, 100.0, 100.0),
        ],
    );
    let mut image = BoardImage::new("i", 20.0, 50.0, 30.0, 30.0);
    assert_eq!(
        store.zone_de(&board, &image).map(|z| z.slot_id.as_str()),
        Some("g/a")
    );
    image.x = 75.0;
    assert_eq!(
        store.zone_de(&board, &image).map(|z| z.slot_id.as_str()),
        Some("g/b"),
        "la plus haute des deux"
    );
    image.x = 400.0;
    assert_eq!(store.zone_de(&board, &image), None);
}
