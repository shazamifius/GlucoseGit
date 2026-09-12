//! Les boutons de la barre d'outils, joués sur le **vrai** `GlucoseApp`, par le chemin de la
//! souris.
//!
//! Un bouton dont la fonction n'existe pas encore a deux façons honnêtes d'exister : ne pas
//! être là, ou dire qu'il ne fait rien. Il n'en a aucune d'annoncer ce qu'il n'a pas fait.
//! Ces tests tiennent la seconde pour les fonctions que la fiche 09 attend encore — export
//! (§ 5), storyboard (§ 8), collaboration (§ 9) — et tomberont le jour où l'une arrive, ce
//! qui est le but : ils sont la liste de ce qui manque, exécutable.

use super::*;
use crate::dock::{compute_panel_layouts, layout_plugins_panel, layout_storyboard_panel, TabId};
use crate::ui::layout_topbar;
use winit::event::MouseButton;

const SCREEN: (f32, f32) = (1440.0, 900.0);

/// Clique au centre du bouton de la barre d'outils qui porte `action`.
fn click_topbar(app: &mut GlucoseApp, action: UiAction) {
    let img_count = app.store.active_board().map(|b| b.images.len()).unwrap_or(0);
    let layout = layout_topbar(SCREEN.0, &app.ui, &app.renderer.typography, img_count);
    let btn = layout
        .buttons
        .iter()
        .find(|b| b.action == action)
        .unwrap_or_else(|| panic!("aucun bouton pour {action:?}"));
    let (cx, cy) = ((btn.x + btn.w / 2.0) as f64, (btn.y + btn.h / 2.0) as f64);
    app.handle_cursor_moved(PhysicalPosition::new(cx, cy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_mouse_up(MouseButton::Left);
}

#[test]
fn test_the_export_button_does_not_claim_to_have_exported() {
    let mut app = GlucoseApp::new();
    let before = app.store.version;
    click_topbar(&mut app, UiAction::ExportMenu);
    assert_eq!(app.ui.toast_message(), Some(NOT_YET_EXPORT));
    assert_eq!(app.store.version, before, "et rien n'a touché le document");
}

#[test]
fn test_the_collab_button_does_not_claim_to_be_connected() {
    let mut app = GlucoseApp::new();
    click_topbar(&mut app, UiAction::ToggleCollab);
    assert_eq!(app.ui.toast_message(), Some(crate::ui::NOT_YET_COLLAB));
    assert!(!app.ui.collab_active, "le bouton ne s'allume pas");
    click_topbar(&mut app, UiAction::ToggleCollab);
    assert!(!app.ui.collab_active, "même au second clic");
}

#[test]
fn test_the_storyboard_activate_button_does_not_stay_lit() {
    let mut app = GlucoseApp::new();
    click_topbar(&mut app, UiAction::ToggleStoryboard);
    assert!(app.dock_manager.is_open(TabId::Storyboard), "le panneau s'ouvre");

    // Le bouton « Activer » du panneau, là où il est dessiné.
    let s = crate::theme::clamp_ui_scale(app.ui.scale());
    let frame = compute_panel_layouts(&app.dock_manager, SCREEN.0, SCREEN.1, app.ui.header_height(), s)
        .into_iter()
        .find(|b| b.tab == TabId::Storyboard)
        .expect("le panneau ouvert a un cadre");
    let layout = layout_storyboard_panel(frame.x, frame.y, frame.width, frame.height, s);
    let (cx, cy) = (
        (layout.activate_button.x + layout.activate_button.w / 2.0) as f64,
        (layout.activate_button.y + layout.activate_button.h / 2.0) as f64,
    );
    app.handle_cursor_moved(PhysicalPosition::new(cx, cy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_mouse_up(MouseButton::Left);

    assert_eq!(app.ui.toast_message(), Some(NOT_YET_STORYBOARD));
    assert!(!app.dock_manager.storyboard.active, "rien n'est « activé »");
    assert!(app.store.active_board().unwrap().panels.is_empty(), "et aucun panneau n'est né");
}

#[test]
fn test_the_download_model_button_does_not_pretend_to_download() {
    let mut app = GlucoseApp::new();
    click_topbar(&mut app, UiAction::TogglePlugins);
    assert!(app.dock_manager.is_open(TabId::Plugins), "le panneau s'ouvre");

    let s = crate::theme::clamp_ui_scale(app.ui.scale());
    let frame = compute_panel_layouts(&app.dock_manager, SCREEN.0, SCREEN.1, app.ui.header_height(), s)
        .into_iter()
        .find(|b| b.tab == TabId::Plugins)
        .expect("le panneau ouvert a un cadre");
    let layout = layout_plugins_panel(frame.x, frame.y, frame.width, frame.height, s);
    let (cx, cy) = (
        (layout.download_button.x + layout.download_button.w / 2.0) as f64,
        (layout.download_button.y + layout.download_button.h / 2.0) as f64,
    );
    app.handle_cursor_moved(PhysicalPosition::new(cx, cy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_mouse_up(MouseButton::Left);

    assert_eq!(app.ui.toast_message(), Some(NOT_YET_AI));
    assert_eq!(crate::dock::OLLAMA_STATUS, "Ollama : non détecté", "et le panneau ne dit pas « actif »");
}

// ── Fiche 07 — la gestuelle, chiffre par chiffre, sur le vrai GlucoseApp ────────────

use crate::canvas::world_to_screen;
use glucose_core::hit_priority::pick_consts;
use glucose_core::types::BoardImage;

fn render(app: &mut GlucoseApp) {
    let mut pixmap = tiny_skia::Pixmap::new(SCREEN.0 as u32, SCREEN.1 as u32).expect("pixmap");
    let mut view = pixmap.as_mut();
    let overlay = crate::params::SceneOverlay { guides: &app.active_guides, selection_box: None, editing: None };
    app.renderer.render(&mut view, &app.store, &mut app.ui, overlay, crate::params::Pointer { x: 0.0, y: 0.0 });
}

fn canvas_app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.viewport.x = 400.0;
        b.viewport.y = 300.0;
    }
    app
}

fn screen_of(app: &GlucoseApp, wx: f64, wy: f64) -> (f64, f64) {
    let vp = app.store.active_board().map(|b| b.viewport).unwrap_or_default();
    world_to_screen(wx, wy, &vp)
}

fn click_at(app: &mut GlucoseApp, sx: f64, sy: f64) {
    app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_mouse_up(MouseButton::Left);
}

/// § 1 — « DOUBLE_CLICK_WINDOW » : deux clics sur une carte de texte à moins de 350 ms
/// ouvrent l'édition ; à plus de 350 ms, non. Une seule fenêtre, celle du noyau.
#[test]
fn test_a_double_click_opens_the_editor_inside_350_ms_and_not_beyond() {
    let mut app = canvas_app();
    let board = app.store.project.active_board_id.clone();
    app.store.add_annotation(
        &board,
        glucose_core::types::Annotation::Text {
            id: "T1".into(),
            x: 0.0,
            y: 0.0,
            width: Some(240.0),
            height: Some(48.0),
            text: "bonjour".into(),
            font_size: Some(14.0),
            color: None,
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        },
    );
    render(&mut app);
    // (x, y) d'une annotation est son coin haut-gauche : le centre d'une carte de 240 × 48
    // posée à l'origine est en (120, 24). Sur le coin, une poignée gagnerait — à bon droit.
    let (sx, sy) = screen_of(&app, 120.0, 24.0);

    for (elapsed_ms, opens) in [(pick_consts::DBLCLICK_MS as u64 - 1, true), (pick_consts::DBLCLICK_MS as u64 + 1, false)] {
        app.editing_session = None;
        app.last_click = None;
        click_at(&mut app, sx, sy);
        // Le second clic arrive `elapsed_ms` plus tard : on recule l'horloge du premier.
        if let Some(lc) = app.last_click.as_mut() {
            lc.time = std::time::Instant::now() - std::time::Duration::from_millis(elapsed_ms);
        }
        click_at(&mut app, sx, sy);
        assert_eq!(app.editing_session.is_some(), opens, "second clic à {elapsed_ms} ms");
    }
}

/// § 7.3 — un clic maintenu sur le vide ne devient une sélection élastique qu'au-delà de
/// 4 px de déplacement écran ; en deçà, rien n'est sélectionné. Et la sélection est en
/// boîtes englobantes : un nœud que le rectangle touche est pris.
#[test]
fn test_the_rubberband_needs_more_than_four_pixels_and_selects_by_bounding_box() {
    let mut app = canvas_app();
    let board = app.store.project.active_board_id.clone();
    app.store.add_image(&board, BoardImage::new("I1", 300.0, 0.0, 100.0, 100.0));
    app.store.clear_selection();
    render(&mut app);
    let (ex, ey) = screen_of(&app, -400.0, -250.0); // du vide, loin de l'image

    // 4 px tout juste : pas une sélection.
    app.handle_cursor_moved(PhysicalPosition::new(ex, ey));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_cursor_moved(PhysicalPosition::new(ex + 4.0, ey + 4.0));
    app.handle_mouse_up(MouseButton::Left);
    assert!(app.store.selected_image_ids.is_empty(), "4 px : un clic, pas un lasso");

    // Un rectangle qui effleure le coin de l'image la prend.
    let (cx, cy) = screen_of(&app, 251.0, -49.0); // juste dans le coin haut-gauche de I1
    app.handle_cursor_moved(PhysicalPosition::new(ex, ey));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_cursor_moved(PhysicalPosition::new(cx, cy));
    app.handle_mouse_up(MouseButton::Left);
    assert_eq!(app.store.selected_image_ids, vec!["I1".to_string()], "sélection par boîte englobante");
}

/// § 7.1 — la molette ne dépasse ni ×20 ni ×50 dézoomé (0,02), bornes du geste, plus
/// étroites que celles du modèle.
#[test]
fn test_the_wheel_zoom_is_bounded_between_0_02_and_20() {
    use winit::event::MouseScrollDelta;
    let mut app = canvas_app();
    assert_eq!(crate::interactions::pan_zoom::WHEEL_SCALE_RANGE, (0.02, 20.0));
    let scale = |app: &GlucoseApp| app.store.active_board().unwrap().viewport.scale;

    for _ in 0..200 {
        app.handle_mouse_wheel(MouseScrollDelta::LineDelta(0.0, 1.0));
    }
    assert_eq!(scale(&app), 20.0, "zoom avant borné");
    for _ in 0..400 {
        app.handle_mouse_wheel(MouseScrollDelta::LineDelta(0.0, -1.0));
    }
    assert_eq!(scale(&app), 0.02, "zoom arrière borné");
}

// ── Les dossiers : créer, entrer, remonter ───────────────────────────────────

/// Termine sur-le-champ le vol de caméra en cours, s'il y en a un.
///
/// Entrer dans un dossier plonge la caméra pendant 400 ms avant de basculer (fiche 07 § 1) :
/// un test qui vérifie l'arrivée doit faire passer ce temps, et le faire passer d'un coup
/// plutôt que d'attendre pour de vrai.
fn finish_flight(app: &mut GlucoseApp) {
    app.animator.skip(&mut app.store);
}

/// Pose le curseur au point monde `(wx, wy)` et clique.
fn click_world(app: &mut GlucoseApp, wx: f64, wy: f64) {
    let vp = app.store.active_board().map(|b| b.viewport).unwrap_or_default();
    let (sx, sy) = crate::canvas::world_to_screen(wx, wy, &vp);
    app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_mouse_up(MouseButton::Left);
}

/// **L'outil Dossier crée un dossier.** Il se contentait d'un toast « Dossier » : un bouton qui
/// annonce ce qu'il n'a pas fait, ce que la fiche 11 § A.2 interdit.
#[test]
fn test_the_folder_tool_actually_creates_a_folder() {
    let mut app = GlucoseApp::new();
    let avant = app.store.active_board().map(|b| b.folders.len()).unwrap_or(0);
    let boards_avant = app.store.project.boards.len();

    app.ui.active_tool = ActiveTool::Folder;
    click_world(&mut app, 600.0, 400.0);

    let board = app.store.active_board().expect("un tableau");
    assert_eq!(board.folders.len(), avant + 1, "le dossier existe");
    let f = board.folders.last().expect("le dossier");
    assert_eq!((f.x, f.y), (600.0, 400.0), "posé sous le curseur");
    assert_eq!((f.width, f.height), FOLDER_DEFAULT_SIZE);
    assert!(!f.child_board_id.is_empty(), "et son tableau enfant est créé");
    assert_eq!(app.store.project.boards.len(), boards_avant + 1);

    // R1 — le geste est annulable, et il ne laisse rien derrière lui.
    assert!(app.store.undo(), "la création s'annule");
    assert_eq!(app.store.active_board().map(|b| b.folders.len()), Some(avant));
}

/// **Un double-clic sur un dossier y entre.** Le store savait le faire — `try_enter_folder`,
/// testé — et aucun geste ne l'appelait.
#[test]
fn test_double_clicking_a_folder_enters_it() {
    let mut app = GlucoseApp::new();
    app.ui.active_tool = ActiveTool::Folder;
    click_world(&mut app, 600.0, 400.0);
    let (folder_id, child) = {
        let f = app.store.active_board().and_then(|b| b.folders.last()).expect("le dossier");
        (f.id.clone(), f.child_board_id.clone())
    };
    let racine = app.store.project.active_board_id.clone();

    // Un premier clic au milieu du dossier sélectionne, il n'entre pas.
    click_world(&mut app, 700.0, 500.0);
    assert_eq!(app.store.selected_folder_id.as_deref(), Some(folder_id.as_str()));
    assert_eq!(app.store.project.active_board_id, racine, "un seul clic n'ouvre rien");

    // Le second, au même endroit et dans les temps, lance la plongée.
    click_world(&mut app, 700.0, 500.0);
    assert!(app.animator.is_running(), "la caméra plonge avant de basculer");
    assert_eq!(app.store.project.active_board_id, racine, "on est encore dans le parent");
    finish_flight(&mut app);
    assert_eq!(app.store.project.active_board_id, child, "le tableau enfant est actif");
    assert_eq!(app.store.folder_path().len(), 2, "on est descendu d'un cran");
}

/// Entrer dans un dossier n'est pas une modification du document : la navigation ne touche
/// jamais à la pile d'annulation (loi UNDO-1).
#[test]
fn test_entering_a_folder_is_not_an_undoable_edit() {
    let mut app = GlucoseApp::new();
    app.ui.active_tool = ActiveTool::Folder;
    click_world(&mut app, 600.0, 400.0);
    let profondeur_pile = app.store.journal.depth();

    click_world(&mut app, 700.0, 500.0);
    click_world(&mut app, 700.0, 500.0);
    finish_flight(&mut app);
    assert_eq!(app.store.folder_path().len(), 2, "on est bien entré");
    assert_eq!(
        app.store.journal.depth(),
        profondeur_pile,
        "et la pile n'a pas bougé"
    );
}

/// **Cliquer le fil d'Ariane ramène en arrière.** Sans lui, entrer dans un dossier était un
/// aller sans retour visible.
#[test]
fn test_clicking_the_breadcrumb_goes_back_up() {
    let mut app = GlucoseApp::new();
    app.ui.active_tool = ActiveTool::Folder;
    click_world(&mut app, 600.0, 400.0);
    let racine = app.store.project.active_board_id.clone();
    click_world(&mut app, 700.0, 500.0);
    click_world(&mut app, 700.0, 500.0);
    finish_flight(&mut app);
    assert_ne!(app.store.project.active_board_id, racine, "on est entré");

    // Le premier segment du fil : la racine du projet.
    let segs = crate::ui::breadcrumb::layout_breadcrumb(
        &app.store,
        &app.renderer.typography,
        app.ui.header_height(),
        app.ui.scale_factor,
    );
    let seg = segs.first().expect("le fil est affiché");
    let (cx, cy) = (
        (seg.rect.0 + seg.rect.2 / 2.0) as f64,
        (seg.rect.1 + seg.rect.3 / 2.0) as f64,
    );
    app.handle_cursor_moved(PhysicalPosition::new(cx, cy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_mouse_up(MouseButton::Left);

    assert_eq!(app.store.project.active_board_id, racine, "on est remonté à la racine");
    assert_eq!(app.store.folder_path().len(), 1);
}

/// **Un clic pendant la plongée abrège l'animation.** Quelqu'un de pressé arrive tout de
/// suite ; le clic est consommé plutôt que de viser au hasard dans le tableau d'arrivée.
#[test]
fn test_a_click_during_the_dive_cuts_it_short() {
    let mut app = GlucoseApp::new();
    app.ui.active_tool = ActiveTool::Folder;
    click_world(&mut app, 600.0, 400.0);
    let child = app
        .store
        .active_board()
        .and_then(|b| b.folders.last())
        .map(|f| f.child_board_id.clone())
        .expect("le dossier");

    click_world(&mut app, 700.0, 500.0);
    click_world(&mut app, 700.0, 500.0);
    assert!(app.animator.is_running(), "la plongée est en cours");

    // Un clic n'importe où pendant la plongée.
    app.handle_cursor_moved(PhysicalPosition::new(50.0, 700.0));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_mouse_up(MouseButton::Left);

    assert!(!app.animator.is_running(), "elle est finie");
    assert_eq!(app.store.project.active_board_id, child, "et l'on est arrivé");
    assert!(
        app.store.selected_image_ids.is_empty() && app.store.selected_annotation_ids.is_empty(),
        "le clic n'a rien sélectionné dans le tableau d'arrivée"
    );
}
