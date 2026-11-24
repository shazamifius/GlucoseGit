//! TEXT-FIT-2 — un texte qui grandit pousse sa membrane (fiche 124).

use super::*;

fn texte(id: &str, (x, y, l, h): (f64, f64, f64, f64), dans: Option<&str>) -> Annotation {
    let mut a = Annotation::text(id, x, y, id);
    if let Annotation::Text { width, height, .. } = &mut a {
        (*width, *height) = (Some(l), Some(h));
    }
    a.set_membrane_id(dans.map(str::to_string));
    a
}

fn membrane(id: &str, (x, y, l, h): (f64, f64, f64, f64), dans: Option<&str>) -> Annotation {
    let mut a = Annotation::membrane(id, x, y, l, h);
    a.set_membrane_id(dans.map(str::to_string));
    a
}

fn bas_de(annotations: &[Annotation], id: &str) -> f64 {
    annotations
        .iter()
        .find(|a| a.id() == id)
        .and_then(Annotation::rect)
        .expect("une boîte")
        .bottom()
}

/// Les poussées appliquées, comme le chargement les applique.
fn pousser(annotations: &mut [Annotation], id: &str, bas: f64) {
    for (rang, bas) in membranes_poussees(annotations, id, bas) {
        poser_le_bas(&mut annotations[rang], bas);
    }
}

/// **Un texte qui grandit pousse le bas de sa membrane**, jusqu'à son bas plus l'écart du cadre ;
/// rien d'autre ne bouge. Preuve à l'envers : sans la loi, le texte grandi sortait de 50 unités.
#[test]
fn test_text_fit_2_un_texte_qui_grandit_pousse_le_bas_de_sa_membrane() {
    let mut a = vec![
        membrane("m", (0.0, 0.0, 500.0, 400.0), None),
        texte("t", (30.0, 42.0, 440.0, 300.0), Some("m")),
    ];
    let avant = a[0].rect().expect("le cadre");
    // Le texte grandit jusqu'à 450 : l'ancienne loi laissait la membrane à 400.
    assert!(450.0 > avant.bottom(), "le cas n'est pas exercé");
    pousser(&mut a, "t", 450.0);
    let apres = a[0].rect().expect("le cadre");
    assert_eq!(apres.bottom(), 450.0 + ECART);
    assert_eq!(
        (apres.left, apres.top, apres.width),
        (avant.left, avant.top, avant.width),
        "seul le bas bouge"
    );
}

/// **Ce qui tient encore ne pousse rien** : un texte qui grandit sans atteindre l'écart du bas
/// laisse sa membrane telle quelle, au bit près.
#[test]
fn test_text_fit_2_ce_qui_tient_encore_ne_pousse_rien() {
    let a = vec![
        membrane("m", (0.0, 0.0, 500.0, 400.0), None),
        texte("t", (30.0, 42.0, 440.0, 300.0), Some("m")),
    ];
    assert!(membranes_poussees(&a, "t", 400.0 - ECART).is_empty());
    assert_eq!(
        membranes_poussees(&a, "t", 400.0 - ECART + 1.0),
        vec![(0, 400.0 + 1.0)]
    );
}

/// **MEMB-7 : celle qui la contient suit à son tour**, et la première qui contient déjà arrête
/// la cascade.
#[test]
fn test_text_fit_2_la_membrane_qui_contient_suit_a_son_tour() {
    let mut a = vec![
        membrane("grand-mere", (-100.0, -100.0, 900.0, 2000.0), None),
        membrane("p", (0.0, 0.0, 600.0, 500.0), Some("grand-mere")),
        membrane("m", (20.0, 60.0, 500.0, 400.0), Some("p")),
        texte("t", (50.0, 100.0, 400.0, 300.0), Some("m")),
    ];
    pousser(&mut a, "t", 480.0);
    assert_eq!(bas_de(&a, "m"), 480.0 + ECART);
    assert_eq!(bas_de(&a, "p"), 480.0 + 2.0 * ECART);
    assert_eq!(bas_de(&a, "grand-mere"), 1900.0, "elle contenait déjà");
}

/// **Un nœud sans membrane ne pousse rien**, et un cycle de membranes — qui ne doit pas exister —
/// ne fait pas tourner la cascade sans fin : chacune n'est poussée qu'une fois.
#[test]
fn test_text_fit_2_sans_membrane_rien_et_un_cycle_s_arrete() {
    let seul = vec![texte("t", (0.0, 0.0, 400.0, 300.0), None)];
    assert!(membranes_poussees(&seul, "t", 900.0).is_empty());
    let cycle = vec![
        membrane("a", (0.0, 0.0, 500.0, 400.0), Some("b")),
        membrane("b", (0.0, 0.0, 500.0, 400.0), Some("a")),
        texte("t", (30.0, 42.0, 440.0, 300.0), Some("a")),
    ];
    // Sur un fil, avec un délai : une garde cassée fait tomber l'épreuve, elle ne la bloque pas.
    let (envoi, recu) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = envoi.send(membranes_poussees(&cycle, "t", 900.0));
    });
    let poussees = recu
        .recv_timeout(std::time::Duration::from_secs(1))
        .expect("un cycle de membranes a tourné sans fin");
    assert_eq!(poussees.len(), 2, "{poussees:?}");
}

/// **Par le journal** : la saisie qui fait grandir un texte et la poussée de sa membrane sont un
/// seul geste — un `Ctrl+Z` rend les deux hauteurs d'avant.
#[test]
fn test_text_fit_2_par_le_journal_un_seul_ctrl_z_defait_les_deux() {
    let mut store = Store::new("essai");
    let board = store.project.active_board_id.clone();
    // Un haut qui n'est pas zéro : la hauteur posée est le bas moins le haut, pas le bas.
    store.add_annotation(&board, membrane("m", (0.0, 10.0, 500.0, 390.0), None));
    store.add_annotation(&board, texte("t", (30.0, 42.0, 440.0, 300.0), None));
    let lire = |s: &Store, id: &str| bas_de(&s.active_board().expect("le tableau").annotations, id);
    assert_eq!(
        store
            .active_board()
            .and_then(|b| b.annotations.iter().find(|a| a.id() == "t"))
            .and_then(Annotation::membrane_id),
        Some("m"),
        "posé dans la membrane, il lui appartient (MEMB-1)"
    );
    store.begin_live_edit();
    store.set_annotation_rect(&board, "t", Rect::new(30.0, 42.0, 440.0, 600.0));
    store.pousser_les_membranes(&board, "t", 642.0);
    store.end_live_edit();
    assert_eq!(lire(&store, "m"), 642.0 + ECART);
    assert!(store.undo());
    assert_eq!(lire(&store, "m"), 400.0);
    assert_eq!(lire(&store, "t"), 342.0);
}

/// Les voisins poussés de `id`, son bas à `bas` : (id, haut neuf).
fn pousses(annotations: &[Annotation], id: &str, bas: f64) -> Vec<(String, f64)> {
    (voisins_pousses(annotations, id, bas).into_iter())
        .map(|(rang, haut)| (annotations[rang].id().to_string(), haut))
        .collect()
}

/// **Fiche 145** : le voisin dessous ne bouge qu'au contact, du strict nécessaire, et le suivant
/// à son tour — l'ordre gardé ; à côté ou déjà chevauché, rien ne bouge.
#[test]
fn test_pousser_pendant_l_ecriture_le_voisin_dessous_s_ecarte_et_pousse_le_suivant() {
    let a = [
        texte("t", (0.0, 0.0, 200.0, 100.0), None),
        texte("dessous", (50.0, 140.0, 200.0, 60.0), None),
        texte("encore", (100.0, 240.0, 100.0, 50.0), None),
        texte("a-cote", (300.0, 120.0, 100.0, 50.0), None),
        texte("deja", (0.0, 80.0, 100.0, 50.0), None),
    ];
    assert!(pousses(&a, "t", 100.0).is_empty(), "rien n'a grandi");
    assert!(
        pousses(&a, "t", 110.0).is_empty(),
        "l'écart de 30 tient encore"
    );
    assert_eq!(
        pousses(&a, "t", 150.0),
        vec![("dessous".into(), 180.0), ("encore".into(), 270.0)]
    );
}

/// **Effacer fait remonter** : une fonction de l'état de départ, sans dérive.
#[test]
fn test_pousser_pendant_l_ecriture_effacer_fait_remonter() {
    let a = [
        texte("t", (0.0, 0.0, 200.0, 100.0), None),
        texte("dessous", (0.0, 110.0, 200.0, 60.0), None),
    ];
    assert_eq!(pousses(&a, "t", 200.0), vec![("dessous".into(), 210.0)]);
    assert_eq!(pousses(&a, "t", 120.0), vec![("dessous".into(), 130.0)]);
    assert!(pousses(&a, "t", 100.0).is_empty(), "revenu à sa place");
}

/// **Seul son niveau** : ce qui est dans une autre membrane ne bouge pas avec lui.
#[test]
fn test_pousser_pendant_l_ecriture_seul_son_niveau() {
    let a = [
        texte("t", (0.0, 0.0, 200.0, 100.0), Some("m")),
        texte("ailleurs", (0.0, 120.0, 200.0, 60.0), None),
        texte("frere", (0.0, 130.0, 200.0, 60.0), Some("m")),
    ];
    assert_eq!(pousses(&a, "t", 150.0), vec![("frere".into(), 180.0)]);
}

/// **L'étage du dessus** : le voisin poussé sort du bas de la membrane — elle grandit jusqu'à lui,
/// et pousse à son tour ce qui était sous elle.
#[test]
fn test_pousser_pendant_l_ecriture_la_membrane_grandit_et_pousse_les_siens() {
    let a = [
        membrane("m", (0.0, 0.0, 300.0, 230.0), None),
        texte("t", (10.0, 10.0, 200.0, 100.0), Some("m")),
        texte("frere", (10.0, 140.0, 200.0, 60.0), Some("m")),
        texte("sous-m", (0.0, 260.0, 300.0, 50.0), None),
    ];
    let faits = pousser_en_ecrivant(&a, "t", 150.0);
    assert_eq!(
        faits,
        vec![
            (2, Bord::Haut, 180.0),
            (0, Bord::Bas, 270.0),
            (3, Bord::Haut, 300.0)
        ]
    );
    assert!(
        pousser_en_ecrivant(&a, "t", 110.0).is_empty(),
        "rien n'atteint rien"
    );
}

/// **Par le journal** (fiche 145) : à chaque lettre, le voisin s'écarte ; effacer le fait remonter
/// à sa place ; un seul `Ctrl+Z` rend tout.
#[test]
fn test_pousser_pendant_l_ecriture_par_le_journal() {
    let mut store = Store::new("essai");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, texte("t", (0.0, 0.0, 200.0, 100.0), None));
    store.add_annotation(&board, texte("dessous", (0.0, 130.0, 200.0, 60.0), None));
    let depart = store
        .active_board()
        .expect("le tableau")
        .annotations
        .clone();
    let haut = |s: &Store| {
        let b = s.active_board().expect("le tableau");
        bas_de(&b.annotations, "dessous") - 60.0
    };
    let mut deja = Vec::new();
    store.begin_live_edit();
    for bas in [150.0, 200.0] {
        store.set_annotation_rect(&board, "t", Rect::new(0.0, 0.0, 200.0, bas));
        store.pousser_en_ecrivant(&board, &depart, &mut deja, "t", bas);
    }
    assert_eq!(haut(&store), 230.0, "au contact, l'écart gardé");
    store.set_annotation_rect(&board, "t", Rect::new(0.0, 0.0, 200.0, 120.0));
    store.pousser_en_ecrivant(&board, &depart, &mut deja, "t", 120.0);
    assert_eq!(haut(&store), 150.0, "effacer le fait remonter");
    store.set_annotation_rect(&board, "t", Rect::new(0.0, 0.0, 200.0, 100.0));
    store.pousser_en_ecrivant(&board, &depart, &mut deja, "t", 100.0);
    assert_eq!(haut(&store), 130.0, "jusqu'à sa place de départ");
    store.set_annotation_rect(&board, "t", Rect::new(0.0, 0.0, 200.0, 180.0));
    store.pousser_en_ecrivant(&board, &depart, &mut deja, "t", 180.0);
    store.end_live_edit();
    assert_eq!(haut(&store), 210.0);
    assert!(store.undo());
    assert_eq!(haut(&store), 130.0, "un seul Ctrl+Z");
}
