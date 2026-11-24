//! MEMB-SEL-1 — sélectionner dans une membrane (sa note du 24/11, `text-g420-2316`).

use crate::app::GlucoseApp;
use glucose_core::types::{Annotation, Viewport};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Une membrane « M » (0, 0, 1000 × 800) qui porte un texte « t » en (600, 500), vue à
/// l'échelle 1 depuis l'origine : un point de l'écran est le même point du monde.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    app.store.set_viewport(&board, vue);
    app.store
        .add_annotation(&board, Annotation::membrane("M", 0.0, 0.0, 1000.0, 800.0));
    app.store
        .add_annotation(&board, Annotation::text("t", 600.0, 500.0, "Eau"));
    app.store.clear_selection();
    app
}

fn glisser(app: &mut GlucoseApp, de: (f64, f64), a: (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(de.0, de.1));
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_cursor_moved(PhysicalPosition::new(
        (de.0 + a.0) / 2.0,
        (de.1 + a.1) / 2.0,
    ));
    app.handle_cursor_moved(PhysicalPosition::new(a.0, a.1));
    app.handle_mouse_up(MouseButton::Left);
}

fn choisis(app: &GlucoseApp) -> Vec<String> {
    app.store.selected_annotation_ids.iter().cloned().collect()
}

fn x_de_la_membrane(app: &GlucoseApp) -> f64 {
    let b = app.store.active_board().expect("un tableau");
    match b.annotations.iter().find(|a| a.id() == "M") {
        Some(Annotation::Membrane { x, .. }) => *x,
        _ => panic!("la membrane a disparu"),
    }
}

/// **Glisser dans le vide d'une membrane qu'on n'a pas choisie trace le rectangle** : il prend
/// le texte qu'il croise, pas la membrane dans laquelle il est tracé, et elle ne bouge pas.
#[test]
fn test_memb_sel_1_le_vide_d_une_membrane_trace_le_rectangle() {
    let mut app = app();
    glisser(&mut app, (500.0, 400.0), (700.0, 600.0));
    assert_eq!(choisis(&app), vec!["t".to_string()]);
    assert_eq!(x_de_la_membrane(&app), 0.0, "la membrane a bougé");
}

/// **Un clic sans bouger dans son vide la choisit** ; choisie, son vide la déplace.
#[test]
fn test_memb_sel_1_un_clic_la_choisit_puis_son_vide_la_deplace() {
    let mut app = app();
    glisser(&mut app, (300.0, 300.0), (300.0, 300.0));
    assert_eq!(choisis(&app), vec!["M".to_string()]);
    // Ailleurs dans son vide : au même endroit, ce serait un double-clic.
    glisser(&mut app, (350.0, 350.0), (450.0, 350.0));
    assert_eq!(x_de_la_membrane(&app), 100.0, "choisie, son vide la glisse");
}
