//! DATE-1 : une date posée en un geste, sur les nœuds et les images ; défaite d'un `Ctrl+Z`.

use super::*;
use crate::timeline::{format_anchor, parse_anchor, DEFAULT_ERAS};
use crate::types::{Annotation, BoardImage};

fn scene() -> (Store, String) {
    let mut store = Store::new("temps");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text("a", 0.0, 0.0, "la Révolution"));
    store.add_annotation(&board, Annotation::sticky("p", 0.0, 300.0, "un post-it"));
    store.add_image(&board, BoardImage::new("i", 600.0, 0.0, 100.0, 100.0));
    store.journal.clear();
    (store, board)
}

fn date(texte: &str) -> TemporalAnchor {
    parse_anchor(texte, DEFAULT_ERAS).expect("une date")
}

/// **Un geste date un texte, un post-it et une image** ; un seul `Ctrl+Z` les rend tous trois
/// sans date ; ce qui la porte déjà ne compte pas.
#[test]
fn temps_1_un_geste_date_les_noeuds_et_les_images() {
    let (mut store, board) = scene();
    let ids: Vec<String> = ["a", "p", "i"].map(String::from).to_vec();
    let revolution = date("1789-1799");
    store.begin_live_edit();
    assert_eq!(
        store.ancrer_dans_le_temps(&board, &ids, Some(&revolution)),
        3
    );
    store.end_live_edit();
    assert_eq!(
        store
            .project
            .annotation(&board, "p")
            .unwrap()
            .temporal_anchor(),
        Some(&revolution)
    );
    assert_eq!(
        store.image(&board, "i").unwrap().temporal_anchor.as_ref(),
        Some(&revolution)
    );
    assert_eq!(format_anchor(&revolution), "1789 – 1799");
    assert_eq!(
        store.ancrer_dans_le_temps(&board, &ids, Some(&revolution)),
        0,
        "déjà portée"
    );
    assert!(store.undo(), "un geste");
    assert!(store
        .project
        .annotation(&board, "a")
        .unwrap()
        .temporal_anchor()
        .is_none());
    assert!(store.image(&board, "i").unwrap().temporal_anchor.is_none());
    assert!(!store.can_undo(), "un seul geste, défait d'un coup");
}

/// **La retirer** : `None` efface la date de ce qui en porte une, et ne compte pas le reste.
#[test]
fn temps_1_retirer_la_date() {
    let (mut store, board) = scene();
    let renaissance = date("Renaissance");
    store.ancrer_dans_le_temps(&board, &["a".to_string()], Some(&renaissance));
    let ids: Vec<String> = ["a", "p"].map(String::from).to_vec();
    assert_eq!(store.ancrer_dans_le_temps(&board, &ids, None), 1);
    assert!(store
        .project
        .annotation(&board, "a")
        .unwrap()
        .temporal_anchor()
        .is_none());
}

/// **Les dates de la sélection**, nœuds puis images : ce que l'invite propose d'abord.
#[test]
fn temps_1_les_dates_de_la_selection() {
    let (mut store, board) = scene();
    let antiquite = date("-500");
    store.ancrer_dans_le_temps(&board, &["i".to_string()], Some(&antiquite));
    store.set_selected_annotation_ids(vec!["a".into()]);
    store.set_selected_image_ids(vec!["i".into()]);
    assert_eq!(store.dates_choisies(), vec![None, Some(&antiquite)]);
    assert_eq!(
        antiquite,
        date("500 av. J.-C."),
        "sans an zéro, comme les historiens"
    );
}

/// **Un export ignore la frise chronologique** : sa copie du document naît sans période gardée —
/// la frise est une façon de regarder, pas le document (ESTOMPE-1, fiche 114).
#[test]
fn estompe_1_la_copie_d_un_export_ignore_la_frise() {
    let (mut store, _) = scene();
    store.set_temporal_filter(Some(crate::timeline::TemporalFilter {
        start: 1914,
        end: 1918,
    }));
    assert_eq!(store.copie_a_exporter(false).temporal_filter, None);
    assert_eq!(store.copie_a_exporter(true).temporal_filter, None);
}
