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

/// **OPERATEUR-1 — un post-it devient un opérateur, à la taille de son mot, et le redevient
/// papier ; ce qui n'est pas un post-it est ignoré ; un seul `Ctrl+Z` défait tout** (fiche 98).
#[test]
fn test_operateur_1_un_post_it_devient_un_operateur_et_se_defait() {
    let (mut store, board, mut ids) = tableau();
    let mut grand = Annotation::sticky("d", 400.0, 0.0, "un autre");
    if let Annotation::Sticky { width, height, .. } = &mut grand {
        (*width, *height) = (Some(320.0), Some(260.0));
    }
    store.add_annotation(&board, grand);
    ids.push("d".to_string());
    let avant = store.project.clone();

    store.begin_live_edit();
    let changes = store.poser_l_operateur(&board, &ids, Some(StickyOperator::Because));
    store.end_live_edit();
    assert_eq!(changes, 2, "les deux post-it, pas les flèches");
    for id in ["c", "d"] {
        let n = fleche(&store, id);
        let Annotation::Sticky { operator, text, .. } = n else {
            panic!("un post-it")
        };
        assert_eq!(*operator, Some(StickyOperator::Because));
        assert!(!text.is_empty(), "le texte n'est pas touché");
        assert_eq!(
            n.size(),
            Some((
                StickyOperator::Because.default_width(),
                crate::types::DEFAULT_OPERATOR_HEIGHT
            )),
            "un opérateur a la taille de son mot"
        );
    }
    assert_eq!(
        store.poser_l_operateur(&board, &ids, Some(StickyOperator::Because)),
        0,
        "le même opérateur ne change rien"
    );
    assert!(store.undo(), "un seul geste");
    assert_eq!(
        store.project, avant,
        "Ctrl+Z rend les papiers et leur taille"
    );

    store.poser_l_operateur(&board, &ids, Some(StickyOperator::And));
    store.poser_l_operateur(&board, &ids, None);
    let Annotation::Sticky { operator, .. } = fleche(&store, "d") else {
        panic!("un post-it")
    };
    assert_eq!(*operator, None, "Alt+0 : le papier revient");
}

/// **COULEUR-1 — colorer touche la couleur de chaque sorte, ignore les flèches, se défait d'un
/// geste, et `None` rend la couleur par défaut** (fiche 98).
#[test]
fn test_couleur_1_colorer_chaque_sorte_et_se_defait() {
    let (mut store, board, mut ids) = tableau();
    store.add_annotation(&board, Annotation::text("t", 0.0, 300.0, "un texte"));
    store.add_annotation(&board, Annotation::membrane("m", 0.0, 600.0, 300.0, 200.0));
    store.add_annotation(&board, Annotation::sticky("o", 0.0, 900.0, "car"));
    store.poser_l_operateur(&board, &["o".to_string()], Some(StickyOperator::But));
    ids.extend(["t".to_string(), "m".to_string(), "o".to_string()]);
    let avant = store.project.clone();

    store.begin_live_edit();
    assert_eq!(
        store.colorer(&board, &ids, Some("#f472b6")),
        3,
        "sans les flèches ni l'opérateur, dont la couleur dit le sens"
    );
    store.end_live_edit();
    let rose = Some("#f472b6".to_string());
    match fleche(&store, "c") {
        Annotation::Sticky {
            bg_color, color, ..
        } => {
            assert_eq!(*bg_color, rose, "le papier d'un post-it");
            assert_eq!(*color, None, "son encre ne change pas");
        }
        _ => panic!("un post-it"),
    }
    assert!(matches!(fleche(&store, "t"), Annotation::Text { color, .. } if *color == rose));
    assert!(matches!(fleche(&store, "m"), Annotation::Membrane { color, .. } if *color == rose));
    assert!(matches!(
        fleche(&store, "f1"),
        Annotation::Arrow { color: None, .. }
    ));
    assert_eq!(store.colorer(&board, &ids, Some("#f472b6")), 0, "déjà rose");
    assert!(store.undo(), "un seul geste");
    assert_eq!(store.project, avant);

    store.colorer(&board, &ids, Some("#60a5fa"));
    store.colorer(&board, &ids, None);
    assert_eq!(
        store.project, avant,
        "par défaut : la couleur d'avant toute couleur"
    );
}
