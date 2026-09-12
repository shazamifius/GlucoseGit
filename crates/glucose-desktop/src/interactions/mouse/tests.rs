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
