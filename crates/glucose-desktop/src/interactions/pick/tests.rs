//! PICK-1 § 3.3 — le cycle de profondeur, joué sur le **vrai** `GlucoseApp`, par le chemin de
//! la souris.
//!
//! Ces tests ne dorment pas. L'origine du temps des clics est un champ de l'application
//! ([`GlucoseApp::click_epoch`]) : les reculer d'une seconde vieillit le clic précédent
//! exactement comme une seconde d'attente, et le résultat ne dépend plus de la charge de la
//! machine.

use super::*;
use crate::canvas::world_to_screen;
use glucose_core::hit_priority::pick_consts;
use std::time::Duration;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

const SCREEN: (f32, f32) = (1440.0, 900.0);

/// Deux membranes empilées au même endroit, la seconde par-dessus la première.
///
/// Des membranes, et non des cartes : le cycle s'arrête sur un nœud `terminal` — un texte,
/// une note —, qui est le fond de la pile. Il faut donc deux nœuds qui ne le soient pas pour
/// voir le cycle descendre.
fn app_with_stack() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.viewport.x = 400.0;
        b.viewport.y = 300.0;
    }
    app.store.add_annotation(
        &board,
        Annotation::membrane("dessous", 0.0, 0.0, 400.0, 300.0),
    );
    app.store.add_annotation(
        &board,
        Annotation::membrane("dessus", 20.0, 20.0, 360.0, 260.0),
    );
    app.store.clear_selection();
    app
}

/// Le centre commun des deux membranes, en coordonnées écran.
fn centre(app: &GlucoseApp) -> (f64, f64) {
    let vp = app
        .store
        .active_board()
        .map(|b| b.viewport)
        .unwrap_or_default();
    world_to_screen(200.0, 150.0, &vp)
}

fn click_at(app: &mut GlucoseApp, (sx, sy): (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_mouse_up(MouseButton::Left);
}

/// Fait vieillir le dernier clic de `ms`, sans attendre.
fn plus_tard(app: &mut GlucoseApp, ms: u64) {
    app.click_epoch -= Duration::from_millis(ms);
}

fn selection(app: &GlucoseApp) -> Vec<String> {
    app.store.selected_annotation_ids.to_vec()
}

/// L'espacement d'un re-clic : au-delà de la fenêtre du double-clic, en deçà du TTL du cycle.
const RECLIC_MS: u64 = (pick_consts::DBLCLICK_MS as u64 + pick_consts::CYCLE_TTL_MS as u64) / 2;

#[test]
fn test_pick_1_a_slow_re_click_at_the_same_spot_reaches_the_node_underneath() {
    let mut app = app_with_stack();
    let at = centre(&app);

    click_at(&mut app, at);
    assert_eq!(
        selection(&app),
        vec!["dessus"],
        "le premier clic prend le sommet"
    );

    plus_tard(&mut app, RECLIC_MS);
    click_at(&mut app, at);
    assert_eq!(
        selection(&app),
        vec!["dessous"],
        "le re-clic descend d'un cran"
    );

    // Le cycle est circulaire : au bout de la pile, il remonte au sommet.
    plus_tard(&mut app, RECLIC_MS);
    click_at(&mut app, at);
    assert_eq!(selection(&app), vec!["dessus"], "et il boucle");
}

#[test]
fn test_pick_1_a_double_click_does_not_advance_the_cycle() {
    let mut app = app_with_stack();
    let at = centre(&app);

    click_at(&mut app, at);
    // Deux clics dans la même fenêtre : c'est un double-clic, pas un re-clic.
    plus_tard(&mut app, pick_consts::DBLCLICK_MS as u64 - 50);
    click_at(&mut app, at);
    assert_eq!(
        selection(&app),
        vec!["dessus"],
        "un double-clic ouvre ce qu'il vise, il ne cherche pas dessous"
    );
}

#[test]
fn test_pick_1_the_cycle_expires_and_starts_over_at_the_top() {
    let mut app = app_with_stack();
    let at = centre(&app);

    click_at(&mut app, at);
    plus_tard(&mut app, pick_consts::CYCLE_TTL_MS as u64 + 100);
    click_at(&mut app, at);
    assert_eq!(
        selection(&app),
        vec!["dessus"],
        "passé {} ms, la pile est oubliée et on repart du sommet",
        pick_consts::CYCLE_TTL_MS
    );
}

#[test]
fn test_pick_1_moving_away_starts_a_new_stack() {
    let mut app = app_with_stack();
    let at = centre(&app);

    click_at(&mut app, at);
    plus_tard(&mut app, RECLIC_MS);
    // Au-delà du rayon du cycle, ce n'est plus le même endroit.
    let ailleurs = (at.0 + pick_consts::CYCLE_RADIUS_PX + 2.0, at.1);
    click_at(&mut app, ailleurs);
    assert_eq!(
        selection(&app),
        vec!["dessus"],
        "un clic ailleurs repart du sommet"
    );
}

#[test]
fn test_pick_1_a_drag_never_advances_the_cycle() {
    let mut app = app_with_stack();
    let at = centre(&app);

    click_at(&mut app, at);
    plus_tard(&mut app, RECLIC_MS);

    // Un re-clic, mais qui **déplace** : on manipulait le nœud, on ne cherchait pas celui du
    // dessous. C'est la raison d'être du « au relâchement » de la fiche 07 § 3.3.
    app.handle_cursor_moved(PhysicalPosition::new(at.0, at.1));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    app.handle_cursor_moved(PhysicalPosition::new(at.0 + 60.0, at.1 + 40.0));
    app.handle_mouse_up(MouseButton::Left);

    assert_eq!(
        selection(&app),
        vec!["dessus"],
        "le glisser garde le nœud qu'il déplaçait"
    );
}
