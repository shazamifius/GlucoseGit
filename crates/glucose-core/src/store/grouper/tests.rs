//! GROUPER-1 : la sélection entre dans une membrane neuve, à sa place dans l'arbre.

use super::*;

fn texte(store: &mut Store, id: &str, x: f64, y: f64) {
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text(id, x, y, id));
}

fn membrane_id(store: &Store, id: &str) -> Option<String> {
    membrane_de(store.active_board().expect("le tableau"), id).map(str::to_string)
}

fn boite(store: &Store, id: &str) -> Rect {
    boite_de(store.active_board().expect("le tableau"), id).expect("une boîte")
}

/// **Deux textes à la racine** : la membrane neuve les entoure par la loi du cadre, ils lui
/// appartiennent, elle est choisie, et un seul `Ctrl+Z` défait tout.
#[test]
fn grouper_1_deux_textes_entrent_dans_une_membrane_neuve() {
    let mut store = Store::new("grouper");
    texte(&mut store, "a", 0.0, 0.0);
    texte(&mut store, "b", 400.0, 300.0);
    let avant = store.project.clone();
    store.set_selected_annotation_ids(vec!["a".into(), "b".into()]);
    store.begin_live_edit();
    let g = store.grouper_dans_une_membrane().expect("une membrane");
    store.end_live_edit();
    assert_eq!(membrane_id(&store, "a").as_deref(), Some(g.as_str()));
    assert_eq!(membrane_id(&store, "b").as_deref(), Some(g.as_str()));
    assert_eq!(membrane_id(&store, &g), None, "à la racine");
    let contenu = boite(&store, "a").union(boite(&store, "b"));
    assert_eq!(
        boite(&store, &g),
        Store::cadre_autour(contenu),
        "la loi du cadre"
    );
    // Et la loi elle-même, sans elle : la place d'un titre au-dessus, un écart autour.
    let (cadre, ecart) = (boite(&store, &g), crate::store::agents::ECART);
    assert!(cadre.top <= contenu.top - crate::store::agents::TITRE_DE_MEMBRANE);
    assert!(cadre.left <= contenu.left - ecart);
    assert!(cadre.left + cadre.width >= contenu.left + contenu.width + ecart);
    assert!(cadre.top + cadre.height >= contenu.top + contenu.height + ecart);
    assert_eq!(
        store.selected_annotation_ids.len(),
        1,
        "elle seule est choisie"
    );
    assert!(
        store.selected_annotation_ids.contains(&g),
        "elle est choisie"
    );
    assert!(store.undo(), "un geste");
    assert_eq!(store.project, avant, "Ctrl+Z rend tout");
}

/// **Dans une membrane P, le groupe naît dans P**, et P grandit pour le contenir ; une membrane
/// choisie garde ce qu'elle contient (on emballe un pays avec ses villes).
#[test]
fn grouper_1_le_groupe_nait_dans_l_ancetre_commun_et_garde_les_villes() {
    let mut store = Store::new("grouper");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::membrane("p", 0.0, 0.0, 500.0, 400.0));
    store.add_annotation(&board, Annotation::membrane("m", 20.0, 60.0, 300.0, 250.0));
    texte(&mut store, "x", 40.0, 120.0);
    texte(&mut store, "y", 380.0, 300.0);
    assert_eq!(membrane_id(&store, "m").as_deref(), Some("p"), "le montage");
    assert_eq!(membrane_id(&store, "x").as_deref(), Some("m"), "le montage");
    assert_eq!(membrane_id(&store, "y").as_deref(), Some("p"), "le montage");
    store.set_selected_annotation_ids(vec!["m".into(), "x".into(), "y".into()]);
    let g = store.grouper_dans_une_membrane().expect("une membrane");
    assert_eq!(membrane_id(&store, &g).as_deref(), Some("p"), "dans P");
    assert_eq!(membrane_id(&store, "m").as_deref(), Some(g.as_str()));
    assert_eq!(membrane_id(&store, "y").as_deref(), Some(g.as_str()));
    assert_eq!(
        membrane_id(&store, "x").as_deref(),
        Some("m"),
        "la ville reste dans son pays"
    );
    let (p, cadre) = (boite(&store, "p"), boite(&store, &g));
    assert_eq!(
        p.union(Store::cadre_autour(cadre)),
        p,
        "P a grandi pour le contenir"
    );
}

/// **Rien à grouper** : ni une sélection vide, ni une flèche seule (elle n'a pas de boîte).
#[test]
fn grouper_1_rien_sans_boite() {
    let mut store = Store::new("grouper");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::arrow("f", 0.0, 0.0, 100.0, 0.0));
    store.clear_selection();
    let profondeur = store.undo_depth();
    assert_eq!(store.grouper_dans_une_membrane(), None);
    store.set_selected_annotation_ids(vec!["f".into()]);
    assert_eq!(store.grouper_dans_une_membrane(), None);
    assert_eq!(store.undo_depth(), profondeur, "rien n'est écrit");
}
