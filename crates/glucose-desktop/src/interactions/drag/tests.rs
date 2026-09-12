//! Le glisser joué sur le **vrai** `GlucoseApp`, par le chemin de la souris.
//!
//! Fiche 09 § 2.2 — « Au premier pixel bougé […] : ouverture d'une transaction ». Le noyau a
//! beau savoir tenir un geste continu, il ne vaut rien tant qu'aucun test ne prouve qu'un
//! appui, un déplacement et un relâchement l'atteignent — et surtout qu'un appui **sans**
//! déplacement ne l'atteint pas.

use super::*;
use crate::canvas::world_to_screen;
use crate::params::{Pointer, SceneOverlay};
use glucose_core::types::BoardImage;
use tiny_skia::Pixmap;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

const SCREEN: (f32, f32) = (1440.0, 900.0);

/// Rend une frame : c'est ce qui remplit l'index spatial dont le test de clic dépend.
fn render(app: &mut GlucoseApp) {
    let mut pixmap = Pixmap::new(SCREEN.0 as u32, SCREEN.1 as u32).expect("pixmap");
    let mut view = pixmap.as_mut();
    let overlay = SceneOverlay { guides: &app.active_guides, selection_box: None, editing: None };
    app.renderer.render(&mut view, &app.store, &mut app.ui, overlay, Pointer { x: 0.0, y: 0.0 });
}

/// Une application dont la caméra place l'origine du monde au milieu du canevas.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.viewport.x = 400.0;
        b.viewport.y = 300.0;
    }
    app
}

fn with_image(app: &mut GlucoseApp, id: &str, cx: f64, cy: f64) {
    let board = app.store.project.active_board_id.clone();
    app.store.add_image(&board, BoardImage::new(id, cx, cy, 200.0, 100.0));
    render(app);
}

fn image_x(app: &GlucoseApp, id: &str) -> f64 {
    app.store
        .active_board()
        .and_then(|b| b.images.iter().find(|i| i.id == id))
        .expect("image")
        .x
}

/// Appuie au centre de l'image `id`, là où elle est dessinée.
fn press_on(app: &mut GlucoseApp, id: &str) {
    let vp = app.store.active_board().map(|b| b.viewport).unwrap_or_default();
    let img = app
        .store
        .active_board()
        .and_then(|b| b.images.iter().find(|i| i.id == id))
        .expect("image");
    let (sx, sy) = world_to_screen(img.x, img.y, &vp);
    app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
}

fn drag_by(app: &mut GlucoseApp, dx: f64, steps: usize) {
    let vp = app.store.active_board().map(|b| b.viewport).unwrap_or_default();
    let (x0, y0) = app.mouse_pos;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        app.handle_cursor_moved(PhysicalPosition::new(x0 + dx * t * vp.scale, y0));
    }
}

fn release(app: &mut GlucoseApp) {
    app.handle_mouse_up(MouseButton::Left);
}

/// § 2.2 — un glisser complet, si long soit-il, ne fait qu'**une** entrée d'annulation.
#[test]
fn test_live_1_a_drag_however_long_is_a_single_undo_entry() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0);
    let entries = app.store.undo_depth();

    press_on(&mut app, "I1");
    drag_by(&mut app, 120.0, 40); // quarante événements de souris
    release(&mut app);

    assert!(image_x(&app, "I1") > 100.0, "l'image a bien suivi : {}", image_x(&app, "I1"));
    assert_eq!(app.store.undo_depth(), entries + 1, "quarante pas, une seule entrée");
    assert!(app.store.undo());
    assert_eq!(image_x(&app, "I1"), 0.0, "un seul Ctrl+Z rend la position de départ");
}

/// § 2.2 — **le test qui manquait.** Sélectionner n'est pas éditer : un appui suivi d'un
/// relâchement, sans le moindre pixel parcouru, ne laisse aucune trace — ni entrée
/// d'annulation, ni destruction de ce qui restait à rétablir.
///
/// C'est le comportement que la version de référence obtenait en n'ouvrant la transaction
/// qu'au premier pixel réellement parcouru, seuil de deux pixels compris. Ici l'ouverture
/// est à l'appui — elle ne coûte plus rien — et c'est l'écriture qui fait foi.
#[test]
fn test_live_2_clicking_a_card_without_moving_it_keeps_the_pending_redo() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0);
    with_image(&mut app, "I2", 400.0, 0.0);
    assert!(app.store.undo(), "on défait la création de I2");
    render(&mut app);
    assert_eq!(app.store.redo_depth(), 1);

    press_on(&mut app, "I1"); // un clic de sélection, pas un geste d'édition
    release(&mut app);

    assert_eq!(app.store.undo_depth(), 1, "aucune entrée d'annulation créée");
    assert_eq!(app.store.redo_depth(), 1, "et Ctrl+Y est toujours possible");
    assert!(app.store.redo());
    assert_eq!(app.store.active_board().unwrap().images.len(), 2);
}

/// Une application neuve n'a rien à défaire : la carte d'accueil est l'état de départ, pas
/// un geste de l'utilisateur. Sans cela, Ctrl+Z était actif dès le lancement.
#[test]
fn test_a_fresh_app_has_nothing_to_undo() {
    let app = GlucoseApp::new();
    assert!(!app.store.can_undo());
    assert!(!app.store.can_redo());
    assert_eq!(app.store.active_board().unwrap().annotations.len(), 1, "la carte d'accueil est bien là");
}

/// § 2.2 — « la rédaction d'un long mémo est annulée en un seul coup de Ctrl+Z ». Et elle
/// rend le **texte** d'avant, pas seulement la hauteur de la carte.
#[test]
fn test_live_3_undoing_a_text_edit_restores_the_previous_text() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.store.add_annotation(&board, glucose_core::types::Annotation::Text {
        id: "T1".into(),
        x: 0.0,
        y: 0.0,
        width: Some(240.0),
        height: Some(48.0),
        text: "ancien".into(),
        font_size: Some(14.0),
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    });
    let entries = app.store.undo_depth();

    app.start_text_edit("T1".into(), "nouveau, sur\nplusieurs\nlignes".into());
    app.commit_editing();

    let text_of = |app: &GlucoseApp| match app.store.active_board().unwrap().annotations.iter().find(|a| a.id() == "T1") {
        Some(glucose_core::types::Annotation::Text { text, .. }) => text.clone(),
        _ => panic!("T1 introuvable"),
    };
    assert_eq!(text_of(&app), "nouveau, sur\nplusieurs\nlignes");
    assert_eq!(app.store.undo_depth(), entries + 1, "une saisie = une entrée");
    assert!(app.store.undo());
    assert_eq!(text_of(&app), "ancien", "Ctrl+Z rend le texte d'avant");
    assert!(app.store.redo());
    assert_eq!(text_of(&app), "nouveau, sur\nplusieurs\nlignes", "et Ctrl+Y le nouveau");
}

/// Vider le texte d'une carte la supprime. Ce doit être un geste comme un autre : Ctrl+Z la
/// rend, et la pile qui précède reste intacte.
#[test]
fn test_live_4_emptying_a_text_card_deletes_it_undoably() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    with_image(&mut app, "I1", 0.0, 0.0);
    app.store.add_annotation(&board, glucose_core::types::Annotation::Text {
        id: "T1".into(),
        x: 300.0,
        y: 0.0,
        width: Some(240.0),
        height: Some(48.0),
        text: "à effacer".into(),
        font_size: Some(14.0),
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    });
    with_image(&mut app, "I2", 0.0, 300.0);
    let entries = app.store.undo_depth();
    let anns = app.store.active_board().unwrap().annotations.len();

    app.start_text_edit("T1".into(), "   ".into());
    app.commit_editing();

    assert_eq!(app.store.active_board().unwrap().annotations.len(), anns - 1, "la carte vide disparaît");
    assert_eq!(app.store.undo_depth(), entries + 1);
    assert!(app.store.undo(), "Ctrl+Z la rend");
    assert_eq!(app.store.active_board().unwrap().annotations.len(), anns);
    assert!(app.store.undo(), "et le geste précédent (I2) est toujours là");
    assert_eq!(app.store.active_board().unwrap().images.len(), 1);
    assert!(app.store.undo(), "et celui d'avant (T1)");
    assert!(app.store.undo(), "et I1");
    assert!(!app.store.can_undo(), "et c'est tout : rien n'a été perdu en route");
}
