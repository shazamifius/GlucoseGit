//! **La barre d'options des flèches** : où elle apparaît, ce qui s'y allume, ce qu'un clic
//! demande.

use super::*;
use glucose_core::types::Annotation;

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Un tableau : deux flèches droites, une courbe, une carte. Rien de sélectionné.
fn tableau() -> Store {
    let mut store = Store::new("options");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::arrow("f1", 0.0, 0.0, 100.0, 0.0));
    store.add_annotation(&board, Annotation::arrow("f2", 0.0, 50.0, 100.0, 50.0));
    let mut courbe = Annotation::arrow("f3", 0.0, 90.0, 100.0, 90.0);
    if let Annotation::Arrow { arrow_type, .. } = &mut courbe {
        *arrow_type = Some("curved".to_string());
    }
    store.add_annotation(&board, courbe);
    store.add_annotation(&board, Annotation::sticky("c", 200.0, 0.0, "une carte"));
    store.clear_selection();
    store
}

fn choisir(store: &mut Store, ids: &[&str]) {
    store.clear_selection();
    for id in ids {
        store.select_annotation((*id).to_string(), true);
    }
}

fn barre(store: &Store) -> Option<OptionsDeFleche> {
    layout_options_de_fleche(store, &Typography::new(), ECRAN, 1.0)
}

fn bouton(b: &OptionsDeFleche, contenu: Contenu) -> &Bouton {
    b.boutons
        .iter()
        .find(|x| x.contenu == contenu)
        .expect("le bouton")
}

/// **Sans flèche sélectionnée, pas de barre** — une carte seule n'en a pas.
#[test]
fn test_fleche_3_sans_fleche_pas_de_barre() {
    let mut store = tableau();
    assert!(barre(&store).is_none());
    choisir(&mut store, &["c"]);
    assert!(barre(&store).is_none());
}

/// **Elle se pose au-dessus de la barre d'action**, et ses boutons tiennent en elle sans se
/// chevaucher.
#[test]
fn test_fleche_3_elle_surmonte_la_barre_d_action() {
    let mut store = tableau();
    choisir(&mut store, &["f1"]);
    let b = barre(&store).expect("une barre");
    let action = crate::ui::action_bar::layout_action_bar(&store, &Typography::new(), ECRAN, 1.0)
        .expect("la barre d'action");
    assert!(b.rect.1 + b.rect.3 < action.rect.1, "au-dessus");
    let (x, y, w, h) = b.rect;
    for (i, bt) in b.boutons.iter().enumerate() {
        let (bx, by, bw, bh) = bt.rect;
        assert!(
            bx >= x && by >= y && bx + bw <= x + w && by + bh <= y + h,
            "{i} dedans"
        );
        if let Some(suivant) = b.boutons.get(i + 1) {
            assert!(bx + bw <= suivant.rect.0, "{i} ne chevauche pas le suivant");
        }
    }
}

/// **Un réglage s'allume quand toutes les flèches le partagent** — et aucun quand elles
/// diffèrent.
#[test]
fn test_fleche_3_ce_qui_s_allume() {
    let mut store = tableau();
    choisir(&mut store, &["f1", "f2"]);
    let b = barre(&store).expect("une barre");
    assert!(bouton(&b, Contenu::Texte("Droite")).allume);
    assert!(!bouton(&b, Contenu::Texte("Courbe")).allume);
    assert!(
        bouton(&b, Contenu::Texte("2")).allume,
        "l'épaisseur par défaut"
    );
    choisir(&mut store, &["f1", "f3"]);
    let b = barre(&store).expect("une barre");
    assert!(!bouton(&b, Contenu::Texte("Droite")).allume);
    assert!(!bouton(&b, Contenu::Texte("Courbe")).allume);
}

/// **Ce qu'un clic demande** : poser un réglage — et, sur une relation déjà portée par toutes,
/// la retirer ; sur « Double sens » allumé, l'éteindre.
#[test]
fn test_fleche_3_ce_qu_un_clic_demande() {
    let mut store = tableau();
    let board = store.project.active_board_id.clone();
    choisir(&mut store, &["f1"]);
    let b = barre(&store).expect("une barre");
    let centre = |bt: &Bouton| (bt.rect.0 + bt.rect.2 / 2.0, bt.rect.1 + bt.rect.3 / 2.0);
    let (x, y) = centre(bouton(&b, Contenu::Texte("Courbe")));
    assert_eq!(
        action_sous(&b, x, y),
        Some(Action::Regler(Reglage::Courbe(true)))
    );
    let (x, y) = centre(bouton(&b, Contenu::Sigle(ArrowPredicate::Inspire)));
    assert_eq!(
        action_sous(&b, x, y),
        Some(Action::Regler(Reglage::Relation(Some(
            ArrowPredicate::Inspire
        ))))
    );

    let ids = vec!["f1".to_string()];
    store.regler_les_fleches(
        &board,
        &ids,
        Reglage::Relation(Some(ArrowPredicate::Inspire)),
    );
    store.regler_les_fleches(&board, &ids, Reglage::DoubleSens(true));
    let b = barre(&store).expect("une barre");
    let (x, y) = centre(bouton(&b, Contenu::Sigle(ArrowPredicate::Inspire)));
    assert_eq!(
        action_sous(&b, x, y),
        Some(Action::Regler(Reglage::Relation(None)))
    );
    let (x, y) = centre(bouton(&b, Contenu::Texte("Double sens")));
    assert_eq!(
        action_sous(&b, x, y),
        Some(Action::Regler(Reglage::DoubleSens(false)))
    );
    // Entre deux boutons, rien — mais la barre couvre le clic.
    let premier = &b.boutons[0];
    let entre = (premier.rect.0 + premier.rect.2 + 1.0, premier.rect.1 + 1.0);
    assert!(couvre(&b, entre.0, entre.1));
}

/// **Par les vraies entrées de l'application** : un clic sur « Courbe » rend la flèche
/// sélectionnée courbe, `Ctrl+Z` la redresse — un geste —, et un clic entre deux boutons ne
/// désélectionne rien : la barre prend tout ce qui tombe sur elle.
#[test]
fn test_fleche_3_un_clic_de_la_souris_regle_la_fleche() {
    use winit::dpi::PhysicalPosition;
    use winit::event::MouseButton;
    let mut app = crate::app::GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::arrow("f", 100.0, 100.0, 400.0, 100.0));
    app.store.journal.clear();
    choisir(&mut app.store, &["f"]);
    let clic = |app: &mut crate::app::GlucoseApp, (x, y): (f32, f32)| {
        app.handle_cursor_moved(PhysicalPosition::new(f64::from(x), f64::from(y)));
        app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
        app.handle_mouse_up(MouseButton::Left);
    };
    let b = layout_options_de_fleche(
        &app.store,
        &app.renderer.typography,
        ECRAN,
        app.ui.scale_factor,
    )
    .expect("la barre");
    let courbe = bouton(&b, Contenu::Texte("Courbe")).rect;
    clic(
        &mut app,
        (courbe.0 + courbe.2 / 2.0, courbe.1 + courbe.3 / 2.0),
    );
    let est_courbe = |app: &crate::app::GlucoseApp| {
        glucose_core::arrow::est_courbe(app.store.selected_arrows()[0])
    };
    assert!(est_courbe(&app), "le clic l'a rendue courbe");
    let premier = &b.boutons[0];
    clic(
        &mut app,
        (premier.rect.0 + premier.rect.2 + 1.0, premier.rect.1 + 1.0),
    );
    assert_eq!(
        app.store.selected_arrows().len(),
        1,
        "toujours sélectionnée"
    );
    assert!(app.store.undo(), "un geste");
    assert!(!est_courbe(&app), "Ctrl+Z la redresse");
}
