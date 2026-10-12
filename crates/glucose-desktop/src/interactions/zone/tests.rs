//! PRESET-1 — prendre une zone à la vraie souris : par son bandeau, par un coin, par sa
//! croix ; et son intérieur, qui reste au canevas.

use crate::app::GlucoseApp;
use glucose_core::types::{BoardImage, BoardZone, Viewport};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Une application vue à l'échelle 1 depuis l'origine — un point de l'écran est le même point
/// du monde —, et une zone de 300 × 400 en (200, 200).
fn app() -> (GlucoseApp, String) {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    let board = app.store.project.active_board_id.clone();
    app.store.set_viewport(
        &board,
        Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        },
    );
    app.store.poser_les_zones(
        &board,
        vec![BoardZone::new("props/ref", 200.0, 200.0, 300.0, 400.0)],
    );
    app.store.journal.clear();
    (app, board)
}

/// Appuie en `de`, glisse jusqu'à `a`, relâche.
fn glisser(app: &mut GlucoseApp, de: (f64, f64), a: (f64, f64)) {
    app.mouse_pos = de;
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_cursor_moved(PhysicalPosition::new(a.0, a.1));
    app.handle_mouse_up(MouseButton::Left);
}

fn zone(app: &GlucoseApp, board: &str) -> (f64, f64, f64, f64) {
    let z = &app.store.zones(board)[0];
    (z.x, z.y, z.width, z.height)
}

/// **Le bandeau prend la zone** : elle est choisie et suit la main, d'un seul geste — une
/// seule entrée du journal.
#[test]
fn test_preset_1_le_bandeau_prend_et_glisse_la_zone() {
    let (mut app, board) = app();
    glisser(&mut app, (300.0, 215.0), (360.0, 265.0));
    assert_eq!(zone(&app, &board), (260.0, 250.0, 300.0, 400.0));
    assert_eq!(app.ui.zones.choisie.as_deref(), Some("props/ref"));
    assert!(app.store.undo(), "il y a quelque chose à annuler");
    assert_eq!(zone(&app, &board), (200.0, 200.0, 300.0, 400.0));
    assert!(!app.store.undo(), "un seul geste");
}

/// **L'intérieur reste au canevas** : glisser dans le corps d'une zone ne la prend pas, et ne
/// la choisit pas — c'est le rectangle de sélection, comme partout.
#[test]
fn test_preset_1_l_interieur_reste_au_canevas() {
    let (mut app, board) = app();
    glisser(&mut app, (300.0, 400.0), (360.0, 460.0));
    assert_eq!(zone(&app, &board), (200.0, 200.0, 300.0, 400.0));
    assert_eq!(app.ui.zones.choisie, None);
}

/// **Un coin redimensionne la zone choisie**, jusqu'à sa taille minimale et pas en deçà.
#[test]
fn test_preset_1_un_coin_redimensionne_la_zone_choisie() {
    let (mut app, board) = app();
    glisser(&mut app, (300.0, 215.0), (300.0, 215.0));
    glisser(&mut app, (500.0, 600.0), (550.0, 650.0));
    assert_eq!(zone(&app, &board), (200.0, 200.0, 350.0, 450.0));
    glisser(&mut app, (550.0, 650.0), (0.0, 0.0));
    let (_, _, w, h) = zone(&app, &board);
    assert_eq!((w, h), (100.0, 80.0), "la taille minimale");
}

/// **La croix retire la zone choisie**, et Suppr aussi ; chaque fois, un `Ctrl+Z` la rend.
#[test]
fn test_preset_1_la_croix_et_suppr_retirent_la_zone() {
    let (mut app, board) = app();
    glisser(&mut app, (300.0, 215.0), (300.0, 215.0));
    let ecran = (200.0, 200.0, 300.0, 400.0);
    let (cx, cy, cw, ch) = crate::renderer::zone::croix(ecran, app.ui.scale());
    glisser(
        &mut app,
        (f64::from(cx + cw / 2.0), f64::from(cy + ch / 2.0)),
        (f64::from(cx + cw / 2.0), f64::from(cy + ch / 2.0)),
    );
    assert!(app.store.zones(&board).is_empty(), "la croix");
    assert!(app.store.undo());
    assert_eq!(app.store.zones(&board).len(), 1);

    glisser(&mut app, (300.0, 215.0), (300.0, 215.0));
    app.delete_selection();
    assert!(app.store.zones(&board).is_empty(), "Suppr");
}

/// **Une photo posée sur le bandeau se prend avant lui** : les nœuds passent devant les zones.
#[test]
fn test_preset_1_une_photo_sur_le_bandeau_passe_devant() {
    let (mut app, board) = app();
    app.store
        .add_image(&board, BoardImage::new("i", 300.0, 215.0, 60.0, 40.0));
    app.store.clear_selection();
    glisser(&mut app, (300.0, 215.0), (300.0, 215.0));
    assert_eq!(app.store.selected_image_ids, vec!["i".to_string()]);
    assert_eq!(app.ui.zones.choisie, None);
}

/// **La zone choisie montre ses prises** : à son coin, la poignée se peint — la vraie peinture,
/// choisie puis non choisie.
#[test]
fn test_preset_1_la_zone_choisie_montre_ses_prises() {
    let (mut app, _) = app();
    let coin = |app: &mut GlucoseApp| {
        app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
        app.pixmap
            .as_ref()
            .expect("une image")
            .pixel(200, 200)
            .expect("dans l'image")
    };
    let sans = coin(&mut app);
    glisser(&mut app, (300.0, 215.0), (300.0, 215.0));
    let avec = coin(&mut app);
    assert_ne!(avec, sans, "la poignée du coin");
}
