//! GESTES-1 — les gestes à plusieurs doigts (fiche 58), joués sur le vrai [`GlucoseApp`] par
//! les mêmes évènements que `winit` livre sur un téléphone.

use crate::app::GlucoseApp;
use glucose_core::types::BoardImage;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, Touch, TouchPhase};

fn doigt(app: &mut GlucoseApp, id: u64, phase: TouchPhase, (x, y): (f64, f64)) {
    app.toucher(&Touch {
        device_id: DeviceId::dummy(),
        phase,
        location: PhysicalPosition::new(x, y),
        force: None,
        id,
    });
}

/// Une application vide, et deux images — où chacune tombe à l'écran.
fn deux_images() -> (GlucoseApp, (f64, f64), (f64, f64)) {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        // La vue décalée : les images tombent au milieu de l'écran, loin de la barre.
        b.viewport.x = 400.0;
        b.viewport.y = 300.0;
    }
    app.store
        .add_image(&board, BoardImage::new("a", 0.0, 0.0, 100.0, 100.0));
    app.store
        .add_image(&board, BoardImage::new("b", 300.0, 0.0, 100.0, 100.0));
    app.store.clear_selection();
    let vp = app.store.viewport();
    let a = crate::canvas::world_to_screen(50.0, 50.0, &vp);
    let b = crate::canvas::world_to_screen(350.0, 50.0, &vp);
    (app, a, b)
}

/// L'appui long du doigt posé prend : son échéance est atteinte.
fn l_appui_prend(app: &mut GlucoseApp) {
    if let Some(appui) = &mut app.toucher.appui {
        appui.echeance = std::time::Instant::now();
    }
    assert_eq!(app.attente_de_l_appui(), None, "l'appui a pris");
}

/// Des doigts touchés ensemble, sans bouger.
fn toucher_a(app: &mut GlucoseApp, n: u64) {
    let ou = |i: u64| (200.0 + 120.0 * i as f64, 500.0);
    for i in 0..n {
        doigt(app, 10 + i, TouchPhase::Started, ou(i));
    }
    for i in 0..n {
        doigt(app, 10 + i, TouchPhase::Ended, ou(i));
    }
}

/// **Deux doigts touchés annulent, trois rétablissent** — comme Procreate.
#[test]
fn test_gestes_1_deux_doigts_annulent_trois_retablissent() {
    let (mut app, ..) = deux_images();
    assert_eq!(app.store.active_board().unwrap().images.len(), 2);
    toucher_a(&mut app, 2);
    assert_eq!(
        app.store.active_board().unwrap().images.len(),
        1,
        "la dernière image posée est annulée"
    );
    toucher_a(&mut app, 3);
    assert_eq!(
        app.store.active_board().unwrap().images.len(),
        2,
        "et rétablie"
    );
}

/// **Deux doigts qui bougent pincent, et n'annulent rien** ; deux doigts qui tiennent au-delà
/// de l'appui long non plus.
#[test]
fn test_gestes_1_pincer_ou_tenir_n_annule_rien() {
    let (mut app, ..) = deux_images();
    doigt(&mut app, 1, TouchPhase::Started, (200.0, 500.0));
    doigt(&mut app, 2, TouchPhase::Started, (400.0, 500.0));
    doigt(&mut app, 2, TouchPhase::Moved, (480.0, 500.0));
    doigt(&mut app, 1, TouchPhase::Ended, (200.0, 500.0));
    doigt(&mut app, 2, TouchPhase::Ended, (480.0, 500.0));
    assert_eq!(app.store.active_board().unwrap().images.len(), 2, "pincé");

    doigt(&mut app, 1, TouchPhase::Started, (200.0, 500.0));
    doigt(&mut app, 2, TouchPhase::Started, (400.0, 500.0));
    if let Some(b) = &mut app.toucher.bref {
        b.debut -= crate::plateforme::doigt::appui_long();
    }
    doigt(&mut app, 1, TouchPhase::Ended, (200.0, 500.0));
    doigt(&mut app, 2, TouchPhase::Ended, (400.0, 500.0));
    assert_eq!(app.store.active_board().unwrap().images.len(), 2, "tenu");
}

/// **Appui long puis glisser : un rectangle de sélection** — les deux images, et aucun menu.
#[test]
fn test_gestes_1_appui_long_puis_glisser_trace_un_rectangle() {
    let (mut app, a, b) = deux_images();
    let depart = (a.0 - 80.0, a.1 - 80.0);
    doigt(&mut app, 1, TouchPhase::Started, depart);
    l_appui_prend(&mut app);
    doigt(&mut app, 1, TouchPhase::Moved, (b.0 + 80.0, b.1 + 80.0));
    assert!(app.selection_box.is_some(), "le rectangle se trace");
    doigt(&mut app, 1, TouchPhase::Ended, (b.0 + 80.0, b.1 + 80.0));
    let mut choisies = app.store.selected_image_ids.clone();
    choisies.sort();
    assert_eq!(choisies, ["a", "b"]);
    assert_eq!(app.ui.context_menu_at, None, "aucun menu");
}

/// **Appui long sur un nœud, puis un autre doigt touche l'autre : les deux sont choisis** — et
/// le doigt qui tient, levé, n'ouvre pas de menu. Un toucher sur le vide n'y change rien.
#[test]
fn test_gestes_1_un_doigt_tient_un_autre_ajoute() {
    let (mut app, a, b) = deux_images();
    doigt(&mut app, 1, TouchPhase::Started, a);
    l_appui_prend(&mut app);
    assert_eq!(app.store.selected_image_ids, ["a"]);
    doigt(&mut app, 2, TouchPhase::Started, b);
    doigt(&mut app, 2, TouchPhase::Ended, b);
    doigt(&mut app, 3, TouchPhase::Started, (b.0, b.1 + 400.0));
    doigt(&mut app, 3, TouchPhase::Ended, (b.0, b.1 + 400.0));
    doigt(&mut app, 1, TouchPhase::Ended, a);
    let mut choisies = app.store.selected_image_ids.clone();
    choisies.sort();
    assert_eq!(choisies, ["a", "b"], "le vide n'a rien vidé");
    assert_eq!(app.ui.context_menu_at, None, "aucun menu");
    assert_eq!(
        app.store.active_board().unwrap().images.len(),
        2,
        "rien d'annulé"
    );
}
