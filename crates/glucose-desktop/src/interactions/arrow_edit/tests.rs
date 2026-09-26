//! Les coudes d'une flèche : poser, déplacer, retirer — et ce que le clic doit atteindre.

use super::*;
use glucose_core::types::Annotation;

fn app_avec_fleche() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app.store
        .add_annotation(&board, Annotation::arrow("a", 0.0, 0.0, 400.0, 0.0));
    app.store.set_selected_annotation_ids(vec!["a".into()]);
    app.store.journal.clear();
    app
}

fn coudes(app: &GlucoseApp) -> Vec<(f64, f64)> {
    app.store
        .active_board()
        .and_then(|b| b.annotations.iter().find(|a| a.id() == "a").cloned())
        .map(|a| match a {
            Annotation::Arrow { waypoints, .. } => waypoints.iter().map(|p| (p.x, p.y)).collect(),
            _ => Vec::new(),
        })
        .unwrap_or_default()
}

/// Une flèche droite n'a qu'une poignée : le milieu de son unique tronçon.
#[test]
fn test_arrow_3_a_straight_arrow_offers_one_midpoint_and_no_bend() {
    let app = app_avec_fleche();
    let board = app.store.active_board().expect("un board");
    let fleche = &board.annotations[0];
    let poignees = arrow::handles(fleche, |id: &str| arrow::node_rect(board, id));

    assert_eq!(poignees.len(), 1);
    assert_eq!(poignees[0].kind, HandleKind::Midpoint(0));
    assert_eq!(poignees[0].at, (200.0, 0.0), "le milieu du tronçon");
}

/// Appuyer sur un milieu y pose un coude **et** ouvre le glisser dessus.
///
/// C'est DRAW-1 appliqué à un coude : l'objet naît sous la main et la suit. Glucose Tauri
/// demande de relâcher puis de reprendre ; ici un seul mouvement suffit.
#[test]
fn test_arrow_3_pressing_a_midpoint_inserts_a_bend_and_grabs_it() {
    let mut app = app_avec_fleche();
    assert!(app.begin_arrow_bend(200.0, 0.0), "le clic est consommé");

    assert_eq!(coudes(&app), vec![(200.0, 0.0)], "un coude au milieu");
    assert!(app.bend_session.is_some(), "et il est tenu sous la main");

    app.update_bend(200.0, 120.0);
    assert_eq!(coudes(&app), vec![(200.0, 120.0)], "il suit la main");

    app.finish_bend();
    assert!(app.bend_session.is_none());
}

/// Tout le geste — insertion et glisser — tient dans **une** entrée d'annulation.
#[test]
fn test_arrow_3_the_whole_gesture_undoes_in_one_step() {
    let mut app = app_avec_fleche();
    app.begin_arrow_bend(200.0, 0.0);
    app.update_bend(200.0, 50.0);
    app.update_bend(200.0, 90.0);
    app.update_bend(210.0, 120.0);
    app.finish_bend();
    assert_eq!(coudes(&app).len(), 1);

    app.store.undo();
    assert!(
        coudes(&app).is_empty(),
        "un seul Ctrl+Z doit défaire le coude entier, pas un pixel de son glisser"
    );
}

/// Un coude posé crée deux tronçons, donc deux nouveaux milieux : on peut plier encore.
#[test]
fn test_arrow_3_a_bend_splits_the_segment_into_two_offers() {
    let mut app = app_avec_fleche();
    app.begin_arrow_bend(200.0, 0.0);
    app.update_bend(200.0, 100.0);
    app.finish_bend();

    let board = app.store.active_board().expect("un board");
    let fleche = &board.annotations[0];
    let poignees = arrow::handles(fleche, |id: &str| arrow::node_rect(board, id));
    let milieux = poignees
        .iter()
        .filter(|h| matches!(h.kind, HandleKind::Midpoint(_)))
        .count();
    let plis = poignees
        .iter()
        .filter(|h| matches!(h.kind, HandleKind::Bend(_)))
        .count();
    assert_eq!((milieux, plis), (2, 1));
}

/// Un double-clic sur un coude le retire : on déplie du geste qui a plié.
///
/// Le double-clic porte sur **la même chose** : poser un coude puis cliquer dessus ne fait
/// pas deux clics sur ce coude, puisque le premier visait un milieu. Il faut donc bien
/// deux clics sur le coude lui-même, ce qui est exactement le geste qu'une main fait.
#[test]
fn test_arrow_3_a_double_click_on_a_bend_removes_it() {
    let mut app = app_avec_fleche();
    app.begin_arrow_bend(200.0, 0.0);
    app.finish_bend();
    assert_eq!(coudes(&app).len(), 1);

    // Premier clic du double : il reprend le coude.
    app.begin_arrow_bend(200.0, 0.0);
    app.finish_bend();
    assert_eq!(coudes(&app).len(), 1, "un clic seul ne retire rien");

    // Second clic, au même endroit et dans la foulée : il le retire.
    app.begin_arrow_bend(200.0, 0.0);
    assert!(coudes(&app).is_empty(), "le double-clic retire le coude");
    assert!(
        app.bend_session.is_none(),
        "et n'ouvre surtout pas un glisser sur un coude qui n'existe plus"
    );
}

/// Deux clics **éloignés dans le temps** ne sont pas un double-clic : le second reprend le
/// coude au lieu de le retirer.
#[test]
fn test_arrow_3_two_clicks_far_apart_in_time_do_not_remove_the_bend() {
    let mut app = app_avec_fleche();
    app.begin_arrow_bend(200.0, 0.0);
    app.finish_bend();

    // Reculer l'origine du temps vieillit le clic précédent sans faire dormir le test.
    app.click_epoch -= std::time::Duration::from_secs(5);
    app.begin_arrow_bend(200.0, 0.0);
    assert_eq!(coudes(&app).len(), 1, "le coude est repris, pas retiré");
    assert!(app.bend_session.is_some());
}

/// Loin de toute poignée, le clic n'est pas consommé : il retourne à la sélection.
#[test]
fn test_arrow_3_a_click_away_from_any_handle_is_not_taken() {
    let mut app = app_avec_fleche();
    assert!(!app.begin_arrow_bend(200.0, 400.0));
    assert!(app.bend_session.is_none());
    assert!(coudes(&app).is_empty());
}

/// Une flèche **non sélectionnée** n'a pas de poignée : rien ne s'attrape sur elle.
///
/// Sans quoi le canevas serait couvert de disques, et un clic près d'une flèche quelconque
/// la plierait au lieu de la prendre.
#[test]
fn test_arrow_3_an_unselected_arrow_offers_no_handle() {
    let mut app = app_avec_fleche();
    app.store.clear_selection();
    assert!(!app.begin_arrow_bend(200.0, 0.0));
    assert!(coudes(&app).is_empty());
}

/// La poignée d'un coude l'emporte sur celle d'un milieu, même quand elles se touchent.
///
/// Un coude fraîchement posé au milieu d'un tronçon est à zéro distance des deux milieux
/// qu'il vient de créer. Sans préférence pour le coude, le reprendre en insérerait un
/// second par-dessus le premier.
#[test]
fn test_arrow_3_a_bend_wins_over_a_midpoint_that_touches_it() {
    let mut app = app_avec_fleche();
    app.begin_arrow_bend(200.0, 0.0);
    app.finish_bend();
    app.click_epoch -= std::time::Duration::from_secs(5);

    app.begin_arrow_bend(200.0, 0.0);
    assert_eq!(
        coudes(&app).len(),
        1,
        "reprendre un coude ne doit pas en poser un second"
    );
    assert_eq!(
        app.bend_session.as_ref().map(|s| s.bend),
        Some(0),
        "et c'est bien le coude existant qui est tenu"
    );
}

/// **DPI-1 — à 150 %, une poignée de flèche se prend une fois et demie plus loin** : ses seize
/// pixels de prise sont logiques, et en valent vingt-quatre à l'écran. Un appui à vingt pixels
/// du milieu la manque à 100 %, et la prend à 150 %.
#[test]
fn test_dpi_1_la_prise_d_une_poignee_suit_la_densite() {
    for (densite, attendu) in [(1.0, false), (1.5, true)] {
        let mut app = app_avec_fleche();
        app.ui.scale_factor = densite;
        app.une_image_sans_fenetre((800, 600));
        let vp = app.store.viewport();
        // Le milieu est en (200, 0) dans le monde ; vingt pixels d'écran plus bas.
        let prise = app.arrow_handle_at(200.0, 20.0 / vp.scale);
        assert_eq!(prise.is_some(), attendu, "à la densité {densite}");
    }
}

/// **Son essai du 26/09 : un coude créé ne se déplace pas** — par le vrai chemin de la souris, à
/// 150 %, une membrane sous la flèche.
///
/// On prend la flèche d'un clic ; puis, plus lentement qu'un double-clic, on appuie sur son
/// losange : le coude naît et suit la main. Au relâchement, la flèche doit être **encore**
/// sélectionnée — sans quoi ses poignées disparaissent, et le coude reste là, sans prise. Puis
/// on reprend le coude, et il suit encore.
#[test]
fn test_fleche_un_coude_cree_se_reprend() {
    use winit::dpi::PhysicalPosition;
    use winit::event::MouseButton;
    const ECRAN: (f32, f32) = (1440.0, 900.0);
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.viewport.x = 300.0;
        b.viewport.y = 400.0;
    }
    app.ui.scale_factor = 1.5;
    app.store.add_annotation(
        &board,
        Annotation::membrane("m", -100.0, -150.0, 600.0, 300.0),
    );
    app.store
        .add_annotation(&board, Annotation::arrow("a", 0.0, 0.0, 400.0, 0.0));
    app.store.clear_selection();
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    let ecran = |app: &GlucoseApp, (x, y): (f64, f64)| {
        crate::canvas::world_to_screen(x, y, &app.store.viewport())
    };
    let aller = |app: &mut GlucoseApp, p: (f64, f64)| {
        app.handle_cursor_moved(PhysicalPosition::new(p.0, p.1));
    };
    let glisser = |app: &mut GlucoseApp, de: (f64, f64), a: (f64, f64)| {
        aller(app, de);
        app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
        aller(app, a);
        app.handle_mouse_up(MouseButton::Left);
    };

    // Un clic sur la flèche, près de son milieu : elle est prise.
    let pres_du_milieu = ecran(&app, (190.0, 0.0));
    glisser(&mut app, pres_du_milieu, pres_du_milieu);
    assert_eq!(app.store.selected_annotation_ids.to_vec(), vec!["a"]);

    // Plus tard, un appui sur le losange du milieu, tiré vers le bas.
    app.click_epoch -= std::time::Duration::from_millis(600);
    let milieu = ecran(&app, (200.0, 0.0));
    let plus_bas = ecran(&app, (200.0, 80.0));
    glisser(&mut app, milieu, plus_bas);
    assert_eq!(coudes(&app), vec![(200.0, 80.0)], "le coude suit la main");
    assert_eq!(
        app.store.selected_annotation_ids.to_vec(),
        vec!["a"],
        "la flèche doit rester prise : sans elle, ses poignées disparaissent"
    );

    // Bouton levé, la souris qui passe ne l'emporte plus : le geste s'est refermé.
    let ailleurs = ecran(&app, (320.0, 200.0));
    aller(&mut app, ailleurs);
    assert_eq!(coudes(&app), vec![(200.0, 80.0)], "le geste est refermé");

    // Et le coude se reprend.
    app.click_epoch -= std::time::Duration::from_millis(600);
    let encore = ecran(&app, (260.0, 120.0));
    glisser(&mut app, plus_bas, encore);
    assert_eq!(coudes(&app), vec![(260.0, 120.0)], "le coude se reprend");

    // Un simple clic sur un losange, sans bouger, plus lent qu'un double-clic : le coude naît
    // là, et la flèche reste prise — le relâchement au même endroit n'est pas un re-clic.
    app.click_epoch -= std::time::Duration::from_millis(600);
    let second_milieu = ecran(&app, (330.0, 60.0));
    glisser(&mut app, second_milieu, second_milieu);
    assert_eq!(coudes(&app).len(), 2, "un second coude");
    assert_eq!(
        app.store.selected_annotation_ids.to_vec(),
        vec!["a"],
        "le relachement d'un clic sur un losange ne change pas la selection"
    );
}

/// **Un clic sur un losange n'est pas un re-clic** : on prend la flèche d'un clic tout près de
/// son milieu, puis, plus lentement qu'un double-clic, on clique sur son losange sans bouger. Le
/// coude naît, et la flèche **reste prise** : le cycle du clic (PICK-2) ne descend qu'au
/// relâchement d'un appui que l'arbitre a armé, et un appui pris par une poignée ne passe pas par
/// lui. C'était ma première hypothèse sur « le coude qu'on ne peut pas déplacer » ; elle était
/// fausse, et cette épreuve garde le scénario.
#[test]
fn test_fleche_un_clic_sur_un_losange_n_est_pas_un_reclic() {
    use winit::dpi::PhysicalPosition;
    use winit::event::MouseButton;
    const ECRAN: (f32, f32) = (1440.0, 900.0);
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.viewport.x = 300.0;
        b.viewport.y = 400.0;
    }
    app.store.add_annotation(
        &board,
        Annotation::membrane("m", -100.0, -150.0, 600.0, 300.0),
    );
    app.store
        .add_annotation(&board, Annotation::arrow("a", 0.0, 0.0, 400.0, 0.0));
    app.store.clear_selection();
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    let clic = |app: &mut GlucoseApp, (x, y): (f64, f64)| {
        let (sx, sy) = crate::canvas::world_to_screen(x, y, &app.store.viewport());
        app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
        app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
        app.handle_mouse_up(MouseButton::Left);
    };
    clic(&mut app, (197.0, 0.0));
    assert_eq!(app.store.selected_annotation_ids.to_vec(), vec!["a"]);
    app.click_epoch -= std::time::Duration::from_millis(600);
    clic(&mut app, (200.0, 0.0));
    assert_eq!(coudes(&app), vec![(200.0, 0.0)], "le coude est né");
    assert_eq!(
        app.store.selected_annotation_ids.to_vec(),
        vec!["a"],
        "la fleche doit rester prise"
    );
}
