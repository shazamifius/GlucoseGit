//! **Transformer une sélection entière** (fiche 53 § 10) : autour de l'origine du groupe ou de
//! celle de chaque nœud, flèches accrochées comprises, en un seul geste annulable.

use glucose_core::groupe::{Origine, Transformation};
use glucose_core::store::Store;
use glucose_core::types::{Annotation, BoardImage};
use std::f64::consts::FRAC_PI_2;

/// Deux images et un texte choisis, une image verrouillée choisie, et une flèche **non**
/// choisie qui part de l'image `a` vers un nœud hors de la sélection.
fn document() -> (Store, String) {
    let mut store = Store::new("P");
    let board = store.project.active_board_id.clone();
    store.add_image(&board, BoardImage::new("a", 100.0, 100.0, 40.0, 20.0));
    store.add_image(&board, BoardImage::new("b", 300.0, 100.0, 40.0, 20.0));
    let mut fixe = BoardImage::new("fixe", 500.0, 500.0, 40.0, 20.0);
    fixe.locked = true;
    store.add_image(&board, fixe);
    store.add_image(&board, BoardImage::new("hors", 900.0, 900.0, 40.0, 20.0));
    let mut texte = Annotation::text("t", 180.0, 180.0, "un mot");
    if let Annotation::Text { width, height, .. } = &mut texte {
        (*width, *height) = (Some(40.0), Some(40.0));
    }
    store.add_annotation(&board, texte);
    let mut fleche = Annotation::arrow("f", 100.0, 100.0, 900.0, 900.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut fleche
    {
        (*source_id, *target_id) = (Some("a".into()), Some("hors".into()));
    }
    store.add_annotation(&board, fleche);
    store.set_selected_image_ids(vec!["a".into(), "b".into(), "fixe".into()]);
    store.selected_annotation_ids = vec!["t".into()];
    store.journal.clear();
    (store, board)
}

fn centre(store: &Store, board: &str, id: &str) -> (f64, f64) {
    let i = store.image(board, id).expect("l'image");
    (i.x, i.y)
}

fn fleche(store: &Store) -> ((f64, f64), (f64, f64)) {
    match store
        .project
        .annotation(&store.project.active_board_id, "f")
    {
        Some(Annotation::Arrow { x, y, x2, y2, .. }) => ((*x, *y), (*x2, *y2)),
        _ => panic!("la flèche"),
    }
}

fn geste(store: &mut Store, board: &str, t: Transformation, origine: Origine) {
    let departs = store.depart_de_la_selection(board);
    store.begin_live_edit();
    store.transformer_la_selection(board, &departs, t, origine);
    store.end_live_edit();
}

/// **Origine commune** : tout s'éloigne de l'ancre du facteur, tailles comprises ; le bout de
/// flèche accroché à `a` suit `a` ; l'image verrouillée ne bouge pas ; rien hors de la
/// sélection ne bouge.
#[test]
fn test_l_origine_commune_met_le_groupe_a_l_echelle() {
    let (mut store, board) = document();
    let t = Transformation::Echelle {
        facteur: 2.0,
        ancre: (0.0, 0.0),
    };
    geste(&mut store, &board, t, Origine::Commune);
    assert_eq!(centre(&store, &board, "a"), (200.0, 200.0));
    assert_eq!(centre(&store, &board, "b"), (600.0, 200.0));
    let a = store.image(&board, "a").expect("a");
    assert_eq!((a.width, a.height), (80.0, 40.0));
    let r = store
        .project
        .annotation(&board, "t")
        .and_then(|t| t.rect())
        .expect("t");
    assert_eq!(
        (r.left, r.top, r.width, r.height),
        (360.0, 360.0, 80.0, 80.0)
    );
    assert_eq!(
        fleche(&store),
        ((200.0, 200.0), (900.0, 900.0)),
        "le bout de a suit a"
    );
    assert_eq!(
        centre(&store, &board, "fixe"),
        (500.0, 500.0),
        "verrouillée"
    );
    assert_eq!(
        centre(&store, &board, "hors"),
        (900.0, 900.0),
        "hors de la sélection"
    );
}

/// **Origines individuelles** : chacun grandit sur place ; le bout accroché à `a` s'écarte du
/// centre de `a`, pas de l'ancre du groupe.
#[test]
fn test_les_origines_individuelles_mettent_chacun_a_l_echelle_sur_place() {
    let (mut store, board) = document();
    let t = Transformation::Echelle {
        facteur: 2.0,
        ancre: (0.0, 0.0),
    };
    geste(&mut store, &board, t, Origine::Individuelle);
    assert_eq!(centre(&store, &board, "a"), (100.0, 100.0));
    assert_eq!(centre(&store, &board, "b"), (300.0, 100.0));
    let b = store.image(&board, "b").expect("b");
    assert_eq!((b.width, b.height), (80.0, 40.0));
    let r = store
        .project
        .annotation(&board, "t")
        .and_then(|t| t.rect())
        .expect("t");
    assert_eq!(
        (r.left, r.top, r.width, r.height),
        (160.0, 160.0, 80.0, 80.0)
    );
    assert_eq!(
        fleche(&store).0,
        (100.0, 100.0),
        "le bout au centre de a ne bouge pas"
    );
}

/// **Rotation commune** : les centres tournent autour du pivot, l'angle des images s'ajoute ;
/// un texte, qui ne tourne pas, voit seulement son centre tourner.
#[test]
fn test_la_rotation_commune_tourne_le_groupe() {
    let (mut store, board) = document();
    let t = Transformation::Rotation {
        angle: FRAC_PI_2,
        pivot: (200.0, 100.0),
    };
    geste(&mut store, &board, t, Origine::Commune);
    let (ax, ay) = centre(&store, &board, "a");
    assert!(
        (ax - 200.0).abs() < 1e-9 && (ay - 0.0).abs() < 1e-9,
        "({ax}, {ay})"
    );
    let a = store.image(&board, "a").expect("a");
    assert!((a.rotation - FRAC_PI_2).abs() < 1e-9);
    let r = store
        .project
        .annotation(&board, "t")
        .and_then(|t| t.rect())
        .expect("t");
    assert_eq!((r.width, r.height), (40.0, 40.0), "un texte ne tourne pas");
}

/// **Un seul geste** : un `Ctrl+Z` rend tout, les flèches comprises.
#[test]
fn test_la_transformation_se_defait_d_un_coup() {
    let (mut store, board) = document();
    let avant = store.project.clone();
    let t = Transformation::Rotation {
        angle: 1.0,
        pivot: (0.0, 0.0),
    };
    geste(&mut store, &board, t, Origine::Commune);
    assert_ne!(store.project, avant);
    assert!(store.undo());
    assert_eq!(store.project, avant);
}
