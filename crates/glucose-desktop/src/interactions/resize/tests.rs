//! Le redimensionnement joué sur le **vrai** `GlucoseApp`, par le chemin de la souris.
//!
//! C'est la règle § 7.6 : le noyau a beau savoir redimensionner, il ne vaut rien tant
//! qu'aucun test ne prouve qu'un clic sur une poignée l'atteint. Chaque test part d'un appui
//! là où la poignée est dessinée, déplace le pointeur, relâche, et lit le document, la pile
//! d'undo et le curseur.

use super::*;
use crate::canvas::world_to_screen;
use crate::params::{Pointer, SceneOverlay};
use glucose_core::types::BoardImage;
use tiny_skia::Pixmap;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{Key, ModifiersState, NamedKey};

const SCREEN: (f32, f32) = (1440.0, 900.0);

/// Rend une frame : c'est ce qui remplit l'index spatial dont le test de clic dépend.
pub(super) fn render_frame(app: &mut GlucoseApp) -> Pixmap {
    let mut pixmap = Pixmap::new(SCREEN.0 as u32, SCREEN.1 as u32).expect("pixmap");
    let mut view = pixmap.as_mut();
    let overlay = SceneOverlay { guides: &app.active_guides, selection_box: None, editing: None };
    app.renderer.render(&mut view, &app.store, &mut app.ui, overlay, Pointer { x: 0.0, y: 0.0 });
    pixmap
}

fn render(app: &mut GlucoseApp) {
    render_frame(app);
}

/// Une application dont la caméra place l'origine au milieu du canevas.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.viewport.x = 400.0;
        b.viewport.y = 300.0;
    }
    app
}

fn with_image(app: &mut GlucoseApp, id: &str, cx: f64, cy: f64, w: f64, h: f64) {
    let board = app.store.project.active_board_id.clone();
    app.store.add_image(&board, BoardImage::new(id, cx, cy, w, h));
    render(app);
}

fn image_box(app: &GlucoseApp, id: &str) -> AlignRect {
    let img = app.store.active_board().and_then(|b| b.images.iter().find(|i| i.id == id)).expect("image");
    rect_of_image(img)
}

pub(super) fn text_card(id: &str, x: f64, y: f64, width: f64, text: &str) -> Annotation {
    Annotation::Text {
        id: id.into(),
        x,
        y,
        width: Some(width),
        height: Some(48.0),
        text: text.into(),
        font_size: Some(14.0),
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

fn ann_box(app: &GlucoseApp, id: &str) -> AlignRect {
    let ann = app.store.active_board().and_then(|b| b.annotations.iter().find(|a| a.id() == id)).expect("annotation");
    rect_of_annotation(ann).expect("boîte")
}

/// Appuie exactement sur `handle` du rectangle monde `rect`.
pub(super) fn press_handle(app: &mut GlucoseApp, rect: AlignRect, handle: Handle) {
    let vp = app.store.active_board().map(|b| b.viewport).unwrap_or_default();
    let (hx, hy) = handle.position_on(rect);
    let (sx, sy) = world_to_screen(hx, hy, &vp);
    app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
}

/// Déplace le pointeur de `(dx, dy)` unités monde, en `steps` événements.
pub(super) fn drag_by(app: &mut GlucoseApp, dx: f64, dy: f64, steps: usize) {
    let vp = app.store.active_board().map(|b| b.viewport).unwrap_or_default();
    let (x0, y0) = app.mouse_pos;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let pos = PhysicalPosition::new(x0 + dx * t * vp.scale, y0 + dy * t * vp.scale);
        app.handle_cursor_moved(pos);
    }
}

pub(super) fn release(app: &mut GlucoseApp) {
    app.handle_mouse_up(MouseButton::Left);
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

// ── Images ──────────────────────────────────────────────────────────────────

#[test]
fn test_resize_1_the_right_handle_of_an_image_keeps_its_left_edge_fixed() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0, 200.0, 100.0);
    let start = image_box(&app, "I1");
    assert_eq!(start, AlignRect::new(-100.0, -50.0, 200.0, 100.0));

    press_handle(&mut app, start, Handle::Right);
    assert!(app.resize_session.is_some(), "l'appui sur la poignée ouvre le geste");
    assert!(!app.is_dragging_item, "et n'ouvre pas un déplacement");
    drag_by(&mut app, 60.0, 0.0, 6);
    release(&mut app);

    let after = image_box(&app, "I1");
    assert!(approx(after.left, -100.0), "bord gauche fixe : {after:?}");
    assert!(approx(after.width, 260.0), "{after:?}");
    assert!(approx(after.height, 100.0), "un côté ne change qu'une dimension : {after:?}");
    assert!(app.resize_session.is_none());
    assert!(app.is_dirty(), "redimensionner modifie le document");
}

#[test]
fn test_resize_1_every_handle_of_an_image_anchors_the_opposite_side() {
    for handle in Handle::ALL {
        let mut app = app();
        with_image(&mut app, "I1", 0.0, 0.0, 200.0, 100.0);
        let start = image_box(&app, "I1");
        // Shift : rapport libre, pour lire chaque bord indépendamment.
        app.modifiers = ModifiersState::SHIFT;
        press_handle(&mut app, start, handle);
        drag_by(&mut app, 20.0, -10.0, 4);
        release(&mut app);
        let after = image_box(&app, "I1");
        let (l0, t0, r0, b0) = (start.left, start.top, start.left + start.width, start.top + start.height);
        let (l, t, r, b) = (after.left, after.top, after.left + after.width, after.top + after.height);
        let name = handle.as_str();
        assert!(approx(l, if handle.moves_left() { l0 + 20.0 } else { l0 }), "{name} gauche {l}");
        assert!(approx(r, if handle.moves_right() { r0 + 20.0 } else { r0 }), "{name} droite {r}");
        assert!(approx(t, if handle.moves_top() { t0 - 10.0 } else { t0 }), "{name} haut {t}");
        assert!(approx(b, if handle.moves_bottom() { b0 - 10.0 } else { b0 }), "{name} bas {b}");
    }
}

#[test]
fn test_resize_3_an_image_corner_keeps_its_ratio_and_shift_frees_it() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0, 200.0, 100.0);
    let start = image_box(&app, "I1");

    press_handle(&mut app, start, Handle::BottomRight);
    drag_by(&mut app, 100.0, 0.0, 5);
    release(&mut app);
    let kept = image_box(&app, "I1");
    assert!(approx(kept.width / kept.height, 2.0), "rapport conservé : {kept:?}");
    assert!(kept.width > 200.0 && kept.height > 100.0, "{kept:?}");
    assert!(approx(kept.left, -100.0) && approx(kept.top, -50.0), "ancre haut-gauche : {kept:?}");

    render(&mut app);
    app.modifiers = ModifiersState::SHIFT;
    press_handle(&mut app, kept, Handle::BottomRight);
    drag_by(&mut app, 50.0, 0.0, 5);
    release(&mut app);
    let freed = image_box(&app, "I1");
    assert!(approx(freed.width, kept.width + 50.0), "{freed:?}");
    assert!(approx(freed.height, kept.height), "Shift libère le rapport : {freed:?}");
}

#[test]
fn test_resize_2_an_image_cannot_be_pulled_below_its_minimum_nor_flipped() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0, 200.0, 100.0);
    let start = image_box(&app, "I1");
    app.modifiers = ModifiersState::SHIFT;
    press_handle(&mut app, start, Handle::BottomRight);
    drag_by(&mut app, -900.0, -900.0, 9);
    release(&mut app);
    let after = image_box(&app, "I1");
    assert!(approx(after.width, glucose_core::resize::MIN_IMAGE_SIDE), "{after:?}");
    assert!(approx(after.height, glucose_core::resize::MIN_IMAGE_SIDE), "{after:?}");
    assert!(approx(after.left, -100.0) && approx(after.top, -50.0), "l'ancre tient : {after:?}");
}

// ── Undo, Échap ─────────────────────────────────────────────────────────────

#[test]
fn test_undo_1_a_gesture_of_many_moves_is_one_undo_entry() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0, 200.0, 100.0);
    let entries = app.store.undo_depth();
    let start = image_box(&app, "I1");

    press_handle(&mut app, start, Handle::Right);
    drag_by(&mut app, 80.0, 0.0, 40);
    release(&mut app);
    assert!(approx(image_box(&app, "I1").width, 280.0));
    assert_eq!(app.store.undo_depth(), entries + 1, "quarante mouvements, une entrée");

    app.modifiers = ModifiersState::CONTROL;
    app.handle_shortcut_input(&Key::Character("z".into()), ElementState::Pressed);
    assert_eq!(image_box(&app, "I1"), start, "Ctrl+Z rend la taille d'avant, d'un coup");
}

#[test]
fn test_a_click_on_a_handle_without_moving_leaves_no_undo_entry() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0, 200.0, 100.0);
    let entries = app.store.undo_depth();
    let version = app.store.version;
    let rect = image_box(&app, "I1");
    press_handle(&mut app, rect, Handle::Left);
    release(&mut app);
    assert_eq!(app.store.undo_depth(), entries);
    assert_eq!(app.store.version, version, "rien n'a changé : pas d'entrée d'undo, pas de « modifié »");
}

#[test]
fn test_escape_during_the_gesture_restores_the_start_size_and_leaves_no_entry() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0, 200.0, 100.0);
    let entries = app.store.undo_depth();
    let start = image_box(&app, "I1");

    press_handle(&mut app, start, Handle::BottomRight);
    drag_by(&mut app, 100.0, 100.0, 5);
    assert!(image_box(&app, "I1").width > 200.0, "le nœud suit pendant le geste");

    app.handle_shortcut_input(&Key::Named(NamedKey::Escape), ElementState::Pressed);
    assert_eq!(image_box(&app, "I1"), start, "Échap rend la taille de départ");
    assert!(app.resize_session.is_none());
    assert_eq!(app.store.undo_depth(), entries, "aucune entrée d'undo");
    assert!(app.store.can_undo() || entries == 0);

    // Le relâchement qui suit ne rouvre rien et n'écrit rien.
    drag_by(&mut app, 50.0, 50.0, 2);
    release(&mut app);
    assert_eq!(image_box(&app, "I1"), start);
    assert_eq!(app.store.undo_depth(), entries);
}

// ── Curseur ─────────────────────────────────────────────────────────────────

#[test]
fn test_the_cursor_announces_the_gesture_over_a_handle_before_any_click() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0, 200.0, 100.0);
    let rect = image_box(&app, "I1");
    let vp = app.store.active_board().map(|b| b.viewport).unwrap_or_default();
    for (handle, expected) in [
        (Handle::Right, CursorIcon::EwResize),
        (Handle::Top, CursorIcon::NsResize),
        (Handle::TopLeft, CursorIcon::NwseResize),
        (Handle::BottomLeft, CursorIcon::NeswResize),
    ] {
        let (hx, hy) = handle.position_on(rect);
        let (sx, sy) = world_to_screen(hx, hy, &vp);
        app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
        assert_eq!(app.hovered_handle(), Some(handle));
        assert_eq!(app.current_cursor(), expected, "{}", handle.as_str());
    }
    // Au milieu de l'image : la flèche.
    let (sx, sy) = world_to_screen(0.0, 0.0, &vp);
    app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
    assert_eq!(app.current_cursor(), CursorIcon::Default);
    // Et pendant le geste, le curseur reste celui de la poignée tenue.
    press_handle(&mut app, rect, Handle::Bottom);
    assert_eq!(app.current_cursor(), CursorIcon::NsResize);
    release(&mut app);
}

#[test]
fn test_pick_1_a_handle_wins_over_the_node_and_the_node_over_the_canvas() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0, 200.0, 100.0);
    let rect = image_box(&app, "I1");
    // Clic au centre : déplacement, pas redimensionnement.
    let vp = app.store.active_board().map(|b| b.viewport).unwrap_or_default();
    let (sx, sy) = world_to_screen(0.0, 0.0, &vp);
    app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    assert!(app.is_dragging_item && app.resize_session.is_none());
    release(&mut app);
    // Clic sur la poignée : redimensionnement.
    press_handle(&mut app, rect, Handle::TopRight);
    assert!(app.resize_session.is_some() && !app.is_dragging_item);
    release(&mut app);
    // Clic dans le vide : sélection élastique.
    let (sx, sy) = world_to_screen(500.0, 500.0, &vp);
    app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
    app.handle_mouse_down(MouseButton::Left, SCREEN.0, SCREEN.1);
    assert!(app.selection_box.is_some() && app.resize_session.is_none());
    release(&mut app);
}

// ── Cartes, pense-bêtes, membranes ──────────────────────────────────────────

#[test]
fn test_text_fit_1_narrowing_a_card_reflows_its_text_and_its_height_follows() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    let text = "Une carte de texte redimensionnée en largeur reflue son texte, et sa hauteur suit.";
    app.store.add_annotation(&board, text_card("T1", -100.0, -40.0, 400.0, text));
    app.fit_text_card_height("T1");
    render(&mut app);
    let start = ann_box(&app, "T1");
    assert!(approx(start.width, 400.0));

    press_handle(&mut app, start, Handle::Right);
    drag_by(&mut app, -220.0, 300.0, 6);
    release(&mut app);
    let after = ann_box(&app, "T1");
    assert!(approx(after.width, 180.0), "{after:?}");
    assert!(after.height > start.height, "plus de lignes, carte plus haute : {after:?} vs {start:?}");
    assert!(approx(after.top, start.top), "la hauteur suit le texte, le haut ne bouge pas");
    let expected = text_card_fit_height(&app.renderer.typography, text, 180.0);
    assert!(approx(after.height, expected), "hauteur du document = hauteur reflué");
}

#[test]
fn test_text_fit_1_a_text_card_has_no_top_or_bottom_handle() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.store.add_annotation(&board, text_card("T1", -100.0, -40.0, 240.0, "hello"));
    app.fit_text_card_height("T1");
    render(&mut app);
    let rect = ann_box(&app, "T1");
    press_handle(&mut app, rect, Handle::Bottom);
    assert!(app.resize_session.is_none(), "pas de poignée basse sur une carte");
    release(&mut app);
    press_handle(&mut app, rect, Handle::Left);
    assert!(app.resize_session.is_some(), "mais une poignée gauche");
    release(&mut app);
}

#[test]
fn test_text_fit_1_committing_a_text_edit_writes_the_fitted_height() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.store.add_annotation(&board, text_card("T1", 0.0, 0.0, 240.0, "une ligne"));
    app.fit_text_card_height("T1");
    let one_line = ann_box(&app, "T1").height;
    let entries = app.store.undo_depth();

    app.start_text_edit("T1".into(), "une ligne\ndeux\ntrois".into());
    app.commit_editing();
    let three_lines = ann_box(&app, "T1").height;
    assert!(three_lines > one_line, "{three_lines} <= {one_line}");
    assert_eq!(app.store.undo_depth(), entries + 1, "une saisie = une entrée d'undo");
    assert!(app.store.undo());
    assert!(approx(ann_box(&app, "T1").height, one_line), "Ctrl+Z rend le texte ET la hauteur d'avant");
}

#[test]
fn test_a_sticky_and_a_membrane_resize_from_any_side() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.store.add_annotation(
        &board,
        Annotation::Sticky {
            id: "S1".into(),
            x: -300.0,
            y: -200.0,
            width: Some(160.0),
            height: Some(120.0),
            text: "note".into(),
            font_size: None,
            color: None,
            bg_color: None,
            cursor_pos: None,
            operator: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        },
    );
    app.store.add_annotation(
        &board,
        Annotation::Membrane {
            id: "M1".into(),
            x: 100.0,
            y: 50.0,
            width: 320.0,
            height: 240.0,
            color: None,
            text: None,
            mode: glucose_core::types::MembraneMode::Classic,
            curtains: Vec::new(),
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        },
    );
    app.store.select_annotation("S1".into(), false);
    render(&mut app);
    let rect = ann_box(&app, "S1");
    press_handle(&mut app, rect, Handle::Top);
    drag_by(&mut app, 0.0, -30.0, 3);
    release(&mut app);
    let sticky = ann_box(&app, "S1");
    assert_eq!(sticky, AlignRect::new(-300.0, -230.0, 160.0, 150.0), "le bas reste, le haut monte");

    app.store.select_annotation("M1".into(), false);
    render(&mut app);
    let rect = ann_box(&app, "M1");
    press_handle(&mut app, rect, Handle::BottomLeft);
    drag_by(&mut app, -40.0, 60.0, 4);
    release(&mut app);
    let membrane = ann_box(&app, "M1");
    assert_eq!(membrane, AlignRect::new(60.0, 50.0, 360.0, 300.0), "le coin haut-droit reste");
}

// ── Magnétisme ──────────────────────────────────────────────────────────────

#[test]
fn test_snap_1_the_pulled_edge_snaps_to_a_neighbour_and_shows_a_guide() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0, 200.0, 100.0);
    with_image(&mut app, "REF", 400.0, 300.0, 100.0, 100.0); // bord gauche à 350
    app.store.select_image("I1".into(), false);
    render(&mut app);
    let start = image_box(&app, "I1");
    app.modifiers = ModifiersState::SHIFT;
    press_handle(&mut app, start, Handle::Right);
    // Le bord droit va de 100 à 346 : à 4 unités du bord de REF, il s'y colle.
    drag_by(&mut app, 246.0, 0.0, 3);
    let during = image_box(&app, "I1");
    assert!(approx(during.left + during.width, 350.0), "{during:?}");
    assert_eq!(app.active_guides.x, Some(vec![350.0]), "le guide est visible pendant le geste");
    release(&mut app);
    assert_eq!(app.active_guides, SnapGuides::default(), "et disparaît après");

    app.ui.smart_align = false;
    render(&mut app);
    let start = image_box(&app, "I1");
    press_handle(&mut app, start, Handle::Right);
    drag_by(&mut app, -4.0, 0.0, 2);
    release(&mut app);
    let free = image_box(&app, "I1");
    assert!(approx(free.left + free.width, 346.0), "aimant coupé : {free:?}");
}

// ── Enregistrement ──────────────────────────────────────────────────────────

#[test]
fn test_persist_1_a_resized_image_survives_save_and_reopen() {
    let mut app = app();
    with_image(&mut app, "I1", 0.0, 0.0, 200.0, 100.0);
    let rect = image_box(&app, "I1");
    press_handle(&mut app, rect, Handle::BottomRight);
    drag_by(&mut app, 100.0, 50.0, 4);
    release(&mut app);
    let resized = image_box(&app, "I1");
    assert!(approx(resized.width, 300.0) && approx(resized.height, 150.0), "{resized:?}");

    let dir = std::env::temp_dir().join(format!("glucose-resize-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dossier temporaire");
    let path = dir.join("resized.glucose");
    app.save_to(path.clone());
    assert!(!app.is_dirty(), "enregistré");

    let mut reopened = GlucoseApp::new();
    reopened.open_from(path);
    assert_eq!(image_box(&reopened, "I1"), resized, "la taille est là au réveil");
    std::fs::remove_dir_all(&dir).expect("nettoyage");
}

#[test]
fn test_text_fit_1_an_older_document_gets_its_card_heights_fitted_on_open() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    // Une carte « d'avant » : quatre lignes rangées dans 48 unités de haut.
    app.store.add_annotation(&board, text_card("T1", 0.0, 0.0, 240.0, "un\ndeux\ntrois\nquatre"));
    let dir = std::env::temp_dir().join(format!("glucose-fit-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dossier temporaire");
    let path = dir.join("old.glucose");
    app.save_to(path.clone());

    let mut reopened = GlucoseApp::new();
    reopened.open_from(path);
    let fitted = ann_box(&reopened, "T1").height;
    assert!(approx(fitted, text_card_fit_height(&reopened.renderer.typography, "un\ndeux\ntrois\nquatre", 240.0)));
    assert!(fitted > 48.0);
    assert!(!reopened.is_dirty(), "recaler à l'ouverture n'est pas une modification");
    std::fs::remove_dir_all(&dir).expect("nettoyage");
}
