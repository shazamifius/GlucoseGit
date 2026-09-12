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
