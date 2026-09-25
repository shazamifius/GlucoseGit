use super::*;

/// Deux flèches et une carte sur le tableau principal, toutes sélectionnées.
fn tableau() -> (Store, String, Vec<String>) {
    let mut store = Store::new("fleches");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::arrow("f1", 0.0, 0.0, 100.0, 0.0));
    store.add_annotation(&board, Annotation::arrow("f2", 0.0, 50.0, 100.0, 50.0));
    store.add_annotation(&board, Annotation::sticky("c", 200.0, 0.0, "une carte"));
    let ids = vec!["f1".to_string(), "f2".to_string(), "c".to_string()];
    (store, board, ids)
}

fn fleche<'a>(store: &'a Store, id: &str) -> &'a Annotation {
    store
        .active_board()
        .and_then(|b| b.annotations.iter().find(|a| a.id() == id))
        .expect("la flèche")
}

/// **Un réglage touche les flèches, ignore le reste, et se défait d'un geste** (FLECHE-3).
#[test]
fn test_fleche_3_un_reglage_touche_les_fleches_et_se_defait() {
    let (mut store, board, ids) = tableau();
    let avant = store.project.clone();
    for reglage in [
        Reglage::Courbe(true),
        Reglage::DoubleSens(true),
        Reglage::Epaisseur(5.0),
        Reglage::Relation(Some(ArrowPredicate::Inspire)),
    ] {
        store.begin_live_edit();
        assert_eq!(
            store.regler_les_fleches(&board, &ids, reglage),
            2,
            "{reglage:?}"
        );
        store.end_live_edit();
    }
    let Annotation::Arrow {
        arrow_type,
        arrow_bidirectional,
        stroke_width,
        predicate,
        ..
    } = fleche(&store, "f2")
    else {
        panic!("une flèche");
    };
    assert_eq!(arrow_type.as_deref(), Some("curved"));
    assert!(*arrow_bidirectional);
    assert_eq!(*stroke_width, Some(5.0));
    assert_eq!(*predicate, Some(ArrowPredicate::Inspire));
    for _ in 0..4 {
        assert!(store.undo(), "un geste par réglage");
    }
    assert!(store.project == avant, "tout est défait");
}

/// **Droite s'écrit comme Tauri l'écrit** : `"straight"`, pas l'absence de valeur — un document
/// écrit ici se relit là-bas à l'identique.
#[test]
fn test_fleche_3_droite_s_ecrit_comme_tauri() {
    let (mut store, board, ids) = tableau();
    store.regler_les_fleches(&board, &ids, Reglage::Courbe(false));
    assert!(matches!(
        fleche(&store, "f1"),
        Annotation::Arrow { arrow_type: Some(t), .. } if t == "straight"
    ));
}
