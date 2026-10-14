//! TRACE-1 — un clic pose à la taille de naissance, un glisser trace la taille : la membrane, le
//! dossier, le storyboard, à la vraie souris.

use crate::app::GlucoseApp;
use crate::interactions::tools::NEW_CONTAINER_SIZE;
use crate::ui::ActiveTool;
use glucose_core::store::storyboard::{Format, Grille};
use glucose_core::types::{Annotation, Viewport};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{Key, NamedKey};

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Une application vide, vue à l'échelle 1 depuis l'origine : un point de l'écran est le même
/// point du monde.
fn app() -> (GlucoseApp, String) {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    let board = app.store.project.active_board_id.clone();
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    app.store.set_viewport(&board, vue);
    app.store.clear_selection();
    app.store.journal.clear();
    (app, board)
}

fn appuyer(app: &mut GlucoseApp, (x, y): (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(x, y));
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
}

fn bouger(app: &mut GlucoseApp, (x, y): (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(x, y));
}

fn relacher(app: &mut GlucoseApp) {
    app.handle_mouse_up(MouseButton::Left);
}

/// Les membranes du tableau : leur boîte.
fn membranes(app: &GlucoseApp) -> Vec<(f64, f64, f64, f64)> {
    app.store
        .active_board()
        .map(|b| {
            b.annotations
                .iter()
                .filter_map(|a| match a {
                    Annotation::Membrane {
                        x,
                        y,
                        width,
                        height,
                        ..
                    } => Some((*x, *y, *width, *height)),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// **Un clic pose la membrane à sa taille de naissance**, comme toujours : sous le curseur.
#[test]
fn test_trace_1_un_clic_pose_la_membrane_a_sa_taille_de_naissance() {
    let (mut app, _) = app();
    app.ui.active_tool = ActiveTool::Membrane;
    appuyer(&mut app, (400.0, 300.0));
    assert!(membranes(&app).is_empty(), "rien n'est posé à l'appui");
    relacher(&mut app);
    let (w, h) = NEW_CONTAINER_SIZE;
    assert_eq!(membranes(&app), [(400.0, 300.0, w, h)]);
    assert_eq!(app.ui.active_tool, ActiveTool::Select, "l'outil se rend");
}

/// **Glisser trace la membrane** : son fantôme suit le rectangle pendant le geste, et le
/// relâchement la pose exactement là — vers le haut et la gauche aussi. Un geste.
#[test]
fn test_trace_1_glisser_trace_la_membrane() {
    let (mut app, _) = app();
    app.ui.active_tool = ActiveTool::Membrane;
    appuyer(&mut app, (700.0, 600.0));
    bouger(&mut app, (500.0, 450.0));
    bouger(&mut app, (300.0, 350.0));
    let fantome = app.fantome_montre().expect("le fantôme du rectangle");
    assert_eq!(
        (
            fantome.rect.left,
            fantome.rect.top,
            fantome.rect.width,
            fantome.rect.height
        ),
        (300.0, 350.0, 400.0, 250.0)
    );
    relacher(&mut app);
    assert_eq!(membranes(&app), [(300.0, 350.0, 400.0, 250.0)]);
    assert_eq!(app.fantome_montre(), None);
    assert!(app.store.undo());
    assert!(membranes(&app).is_empty(), "un seul Ctrl+Z");
}

/// **Le tremblement d'une main reste un clic** : quelques pixels ne tracent rien.
#[test]
fn test_trace_1_le_tremblement_reste_un_clic() {
    let (mut app, _) = app();
    app.ui.active_tool = ActiveTool::Membrane;
    appuyer(&mut app, (400.0, 300.0));
    bouger(&mut app, (404.0, 303.0));
    relacher(&mut app);
    let (w, h) = NEW_CONTAINER_SIZE;
    assert_eq!(membranes(&app), [(400.0, 300.0, w, h)]);
}

/// **Glisser trace aussi le dossier.**
#[test]
fn test_trace_1_glisser_trace_le_dossier() {
    let (mut app, board) = app();
    app.ui.active_tool = ActiveTool::Folder;
    appuyer(&mut app, (300.0, 300.0));
    bouger(&mut app, (900.0, 700.0));
    relacher(&mut app);
    let b = app
        .store
        .project
        .boards
        .iter()
        .find(|b| b.id == board)
        .expect("le tableau");
    let f = b.folders.first().expect("un dossier");
    assert_eq!((f.x, f.y, f.width, f.height), (300.0, 300.0, 600.0, 400.0));
}

/// **Échap pendant un tracé n'y pose rien**, et laisse l'outil armé ; un second Échap le rend.
#[test]
fn test_trace_1_echap_renonce_au_trace() {
    let (mut app, _) = app();
    app.ui.active_tool = ActiveTool::Membrane;
    appuyer(&mut app, (300.0, 300.0));
    bouger(&mut app, (600.0, 600.0));
    app.handle_shortcut_input(&Key::Named(NamedKey::Escape), ElementState::Pressed);
    relacher(&mut app);
    assert!(membranes(&app).is_empty());
    assert_eq!(app.ui.active_tool, ActiveTool::Membrane);
    app.handle_shortcut_input(&Key::Named(NamedKey::Escape), ElementState::Pressed);
    assert_eq!(app.ui.active_tool, ActiveTool::Select);
}

/// Des cases de 100 × 100 à 20 d'écart : 120 en largeur, 160 en hauteur d'une case à l'autre.
fn carree() -> Grille {
    Grille {
        format: Format::Carre,
        largeur: 100.0,
        colonnes: 4,
        ecart: 20.0,
    }
}

/// **Tracer le storyboard le remplit** : pendant le geste, le fantôme montre les cases qui
/// tiennent dans le rectangle — et le rectangle lui-même —, le relâchement les pose, et la
/// grille garde les colonnes tracées.
#[test]
fn test_trace_1_tracer_le_storyboard_le_remplit() {
    let (mut app, board) = app();
    app.ui.placement.storyboard = Some(carree());
    appuyer(&mut app, (200.0, 200.0));
    bouger(&mut app, (500.0, 300.0));
    assert_eq!(
        app.fantome_du_storyboard().len(),
        3,
        "300 × 100 : trois cases (340 est plus près de 300 que 220), une rangée"
    );
    bouger(&mut app, (800.0, 540.0));
    assert_eq!(
        app.fantome_du_storyboard().len(),
        10,
        "600 × 340 : cinq colonnes, deux rangées"
    );
    assert_eq!(
        app.rectangle_trace_a_l_ecran(),
        Some((200.0, 200.0, 800.0, 540.0)),
        "le rectangle tracé se dessine"
    );
    relacher(&mut app);
    let cases = crate::epreuve::les_cases(&app.store, &board);
    assert_eq!(cases.len(), 10);
    assert_eq!((cases[0].x, cases[0].y), (200.0, 200.0));
    assert_eq!(
        crate::epreuve::seul_storyboard(&app.store, &board)
            .and_then(|sb| app.store.grille_du_storyboard(&board, &sb))
            .map(|g| g.colonnes),
        Some(5)
    );
    assert_eq!(app.ui.placement.storyboard, None, "la grille est posée");
    assert_eq!(app.rectangle_trace_a_l_ecran(), None);
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    assert_eq!(
        app.dock_manager.storyboard.cols, 5,
        "le panneau dit les colonnes tracées"
    );
}

/// **FANTOME-1 — un clic sous la grille armée la pose par son coin** (fiche 66) : la première
/// case a son coin haut gauche là où la main a appuyé — le point même d'où un glisser aurait
/// tracé. Tauri la centrait ; le fantôme sautait alors d'une demi-grille quand la main
/// commençait à glisser.
#[test]
fn test_fantome_1_un_clic_pose_le_storyboard_par_son_coin() {
    let (mut app, board) = app();
    app.ui.placement.storyboard = Some(carree());
    appuyer(&mut app, (600.0, 500.0));
    relacher(&mut app);
    let cases = crate::epreuve::les_cases(&app.store, &board);
    assert_eq!(cases.len(), 8, "deux rangées de quatre");
    assert_eq!((cases[0].x, cases[0].y), (600.0, 500.0), "par son coin");
}

/// **FANTOME-1 — la membrane et le dossier aussi** : au survol, le coin haut gauche du fantôme
/// sous le curseur (loin de tout voisin, l'aimant ne tire rien), et le clic pose là.
#[test]
fn test_fantome_1_la_membrane_et_le_dossier_partent_du_curseur() {
    for outil in [ActiveTool::Membrane, ActiveTool::Folder] {
        let (mut app, board) = app();
        app.ui.active_tool = outil;
        bouger(&mut app, (600.0, 500.0));
        let fantome = app.fantome_montre().expect("un fantôme");
        assert_eq!(
            (fantome.rect.left, fantome.rect.top),
            (600.0, 500.0),
            "{outil:?} : au survol"
        );
        appuyer(&mut app, (600.0, 500.0));
        relacher(&mut app);
        let posee = match outil {
            ActiveTool::Membrane => membranes(&app).last().map(|m| (m.0, m.1)),
            _ => app
                .store
                .project
                .boards
                .iter()
                .find(|b| b.id == board)
                .and_then(|b| b.folders.last())
                .map(|f| (f.x, f.y)),
        };
        assert_eq!(posee, Some((600.0, 500.0)), "{outil:?} : le clic pose là");
    }
}

/// **Sous une membrane, le rectangle ne se dessine pas en plus** : son fantôme est déjà le
/// rectangle.
#[test]
fn test_trace_1_le_rectangle_ne_se_dessine_que_sous_la_grille() {
    let (mut app, _) = app();
    app.ui.active_tool = ActiveTool::Membrane;
    appuyer(&mut app, (300.0, 300.0));
    bouger(&mut app, (600.0, 600.0));
    assert_eq!(app.rectangle_trace_a_l_ecran(), None);
}
