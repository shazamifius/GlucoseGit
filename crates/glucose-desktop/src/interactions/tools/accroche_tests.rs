//! SNAP-3 : un bout de flèche s'accroche parmi ce que l'index trouve autour de la souris — et
//! c'est **le même choix** que le parcours du tableau entier.

use crate::app::GlucoseApp;
use crate::canvas::world_to_screen;
use crate::ui::ActiveTool;
use glucose_core::types::{Annotation, BoardImage, CanvasFolder};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

/// Un tirage déterministe : le même tableau à chaque exécution.
struct Hasard(u64);

impl Hasard {
    fn entre(&mut self, a: f64, b: f64) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        a + (b - a) * ((self.0 >> 11) as f64 / (1u64 << 53) as f64)
    }
}

fn carte(id: &str, (x, y): (f64, f64), (w, h): (f64, f64)) -> Annotation {
    let mut a = Annotation::text(id, x, y, "carte");
    if let Annotation::Text { width, height, .. } = &mut a {
        *width = Some(w);
        *height = Some(h);
    }
    a
}

/// Une application au tableau vide, l'accueil éteint.
fn vide() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.folders.clear();
    }
    app
}

/// **Le même choix que le parcours du tableau entier**, en cinq cents points tirés au hasard
/// sur un tableau de photos, de cartes et de dossiers — dont des égalités exactes entre boîtes
/// jointives.
#[test]
fn test_snap_3_l_index_choisit_comme_le_tableau_entier() {
    let mut app = vide();
    let board = app.store.project.active_board_id.clone();
    let mut h = Hasard(7);
    for k in 0..150 {
        let (x, y) = (h.entre(-3000.0, 3000.0), h.entre(-2000.0, 2000.0));
        let (w, hh) = (h.entre(20.0, 300.0), h.entre(20.0, 200.0));
        app.store
            .add_image(&board, BoardImage::new(format!("p{k}"), x, y, w, hh));
        let (x, y) = (h.entre(-3000.0, 3000.0), h.entre(-2000.0, 2000.0));
        app.store
            .add_annotation(&board, carte(&format!("c{k}"), (x, y), (w, hh)));
    }
    for k in 0..10 {
        let mut f = CanvasFolder::new(format!("d{k}"), "dossier", format!("sous{k}"));
        (f.x, f.y) = (h.entre(-3000.0, 3000.0), h.entre(-2000.0, 2000.0));
        app.store.create_folder(&board, f);
    }
    // Deux cartes jointives : un point entre elles est à égale distance des deux.
    app.store
        .add_annotation(&board, carte("gauche", (5000.0, 0.0), (100.0, 100.0)));
    app.store
        .add_annotation(&board, carte("droite", (5200.0, 0.0), (100.0, 100.0)));
    app.une_image_sans_fenetre((1200, 800));

    let mut points: Vec<(f64, f64)> = (0..500)
        .map(|_| (h.entre(-3200.0, 3200.0), h.entre(-2200.0, 2200.0)))
        .collect();
    points.push((5150.0, 50.0));
    let mut accroches = 0;
    for p in points {
        let par_l_index = app.snap_for_arrow(p, &[]);
        let board = app.store.active_board().expect("un tableau");
        let par_le_tableau = glucose_core::arrow::snap_to_nearest(board, p, &[]);
        assert_eq!(par_l_index, par_le_tableau, "au point {p:?}");
        accroches += usize::from(par_l_index.node.is_some());
    }
    assert!(
        accroches > 50,
        "les points s'accrochent vraiment : {accroches}"
    );
}

/// **La pointe n'accroche jamais sa source**, par la vraie souris : tirée juste à côté de la
/// carte dont elle part, elle reste libre ; tirée près d'une autre carte, elle s'y accroche.
#[test]
fn test_snap_3_la_pointe_n_accroche_pas_sa_source() {
    let mut app = vide();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, carte("source", (0.0, 0.0), (100.0, 60.0)));
    app.store
        .add_annotation(&board, carte("autre", (400.0, 0.0), (100.0, 60.0)));
    app.store.clear_selection();
    app.une_image_sans_fenetre((1200, 800));

    let tirer = |app: &mut GlucoseApp, vers: (f64, f64)| -> Option<String> {
        app.ui.active_tool = ActiveTool::Arrow;
        let vp = app.store.viewport();
        let (sx, sy) = world_to_screen(50.0, 30.0, &vp);
        app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
        app.handle_mouse_down(MouseButton::Left, 1200.0, 800.0);
        let (tx, ty) = world_to_screen(vers.0, vers.1, &vp);
        app.handle_cursor_moved(PhysicalPosition::new(tx, ty));
        app.handle_mouse_up(MouseButton::Left);
        app.store
            .active_board()
            .and_then(|b| {
                b.annotations.iter().rev().find_map(|a| match a {
                    Annotation::Arrow { target_id, .. } => Some(target_id.clone()),
                    _ => None,
                })
            })
            .flatten()
    };
    assert_eq!(tirer(&mut app, (130.0, 30.0)), None, "pas sa source");
    assert_eq!(
        tirer(&mut app, (380.0, 30.0)).as_deref(),
        Some("autre"),
        "une autre carte, si"
    );
}
