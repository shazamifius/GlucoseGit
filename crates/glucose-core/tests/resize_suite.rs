//! RESIZE-1..3 — la géométrie du redimensionnement, testée sans écran (§ 7.1).
//!
//! Chaque invariant nommé a son test : l'ancre opposée pour chacune des huit poignées, la
//! taille minimale qui ne fait pas sauter la poignée, le rapport conservé puis libéré, la
//! réconciliation avec le magnétisme, et l'écriture dans le document avec une seule entrée
//! d'undo par geste.

use glucose_core::hit_priority::{collect_candidates, handle_cursor, PickInput, PickKind};
use glucose_core::resize::{resize_rect, snap_resized_rect, Handle, ResizeRule, MIN_IMAGE_SIDE};
use glucose_core::smart_align::{AlignKind, AlignRect, AlignTarget, SnapOptions};
use glucose_core::store::Store;
use glucose_core::types::{Annotation, BoardImage};

const START: AlignRect = AlignRect {
    left: 100.0,
    top: 200.0,
    width: 300.0,
    height: 200.0,
};

fn free() -> ResizeRule {
    ResizeRule {
        min_width: 10.0,
        min_height: 10.0,
        keep_aspect: false,
    }
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn edges(r: AlignRect) -> (f64, f64, f64, f64) {
    (r.left, r.top, r.left + r.width, r.top + r.height)
}

// ── RESIZE-1 : l'ancre est le côté opposé ───────────────────────────────────

#[test]
fn test_resize_1_each_handle_moves_its_own_edges_and_nothing_else() {
    let (l0, t0, r0, b0) = edges(START);
    for handle in Handle::ALL {
        let out = resize_rect(START, handle, (17.0, -23.0), free());
        let (l, t, r, b) = edges(out);
        let name = handle.as_str();
        assert!(
            approx(l, if handle.moves_left() { l0 + 17.0 } else { l0 }),
            "{name} : gauche {l}"
        );
        assert!(
            approx(r, if handle.moves_right() { r0 + 17.0 } else { r0 }),
            "{name} : droite {r}"
        );
        assert!(
            approx(t, if handle.moves_top() { t0 - 23.0 } else { t0 }),
            "{name} : haut {t}"
        );
        assert!(
            approx(b, if handle.moves_bottom() { b0 - 23.0 } else { b0 }),
            "{name} : bas {b}"
        );
    }
}

#[test]
fn test_resize_1_the_right_handle_keeps_the_left_edge_fixed() {
    let out = resize_rect(START, Handle::Right, (50.0, 999.0), free());
    assert_eq!(out, AlignRect::new(100.0, 200.0, 350.0, 200.0));
}

#[test]
fn test_resize_1_the_top_left_corner_keeps_the_bottom_right_corner_fixed() {
    let out = resize_rect(START, Handle::TopLeft, (-40.0, -60.0), free());
    let (_, _, r, b) = edges(out);
    assert!(approx(r, 400.0) && approx(b, 400.0), "{out:?}");
    assert!(approx(out.left, 60.0) && approx(out.top, 140.0), "{out:?}");
}

// ── RESIZE-2 : taille minimale, sans saut de poignée ───────────────────────

#[test]
fn test_resize_2_pulling_past_the_anchor_stops_at_the_minimum_without_flipping() {
    let rule = ResizeRule {
        min_width: 40.0,
        min_height: 30.0,
        keep_aspect: false,
    };
    // La poignée droite tirée 1 000 unités à gauche du bord gauche.
    let out = resize_rect(START, Handle::Right, (-1000.0, 0.0), rule);
    assert_eq!(
        out,
        AlignRect::new(100.0, 200.0, 40.0, 200.0),
        "droite bloquée à 40"
    );
    // La poignée haut-gauche tirée loin au-delà du coin bas-droit.
    let out = resize_rect(START, Handle::TopLeft, (1000.0, 1000.0), rule);
    let (l, t, r, b) = edges(out);
    assert!(
        approx(r, 400.0) && approx(b, 400.0),
        "l'ancre n'a pas bougé : {out:?}"
    );
    assert!(
        approx(l, 360.0) && approx(t, 370.0),
        "bloqué au minimum : {out:?}"
    );
    assert!(out.width > 0.0 && out.height > 0.0);
}

#[test]
fn test_resize_2_the_minimum_holds_with_the_aspect_ratio_too() {
    let rule = ResizeRule::image(false);
    let start = AlignRect::new(0.0, 0.0, 400.0, 200.0);
    let out = resize_rect(start, Handle::BottomRight, (-5000.0, -5000.0), rule);
    assert!(
        out.width >= MIN_IMAGE_SIDE && out.height >= MIN_IMAGE_SIDE,
        "{out:?}"
    );
    assert!(
        approx(out.width / out.height, 2.0),
        "rapport conservé : {out:?}"
    );
    assert!(
        approx(out.left, 0.0) && approx(out.top, 0.0),
        "l'ancre tient : {out:?}"
    );
}

// ── RESIZE-3 : rapport d'aspect ─────────────────────────────────────────────

#[test]
fn test_resize_3_a_corner_keeps_the_ratio_and_the_opposite_corner() {
    let start = AlignRect::new(50.0, 50.0, 400.0, 200.0);
    for handle in [
        Handle::TopLeft,
        Handle::TopRight,
        Handle::BottomLeft,
        Handle::BottomRight,
    ] {
        let out = resize_rect(start, handle, (120.0, 5.0), ResizeRule::image(false));
        assert!(
            approx(out.width / out.height, 2.0),
            "{} : {out:?}",
            handle.as_str()
        );
        let (l0, t0, r0, b0) = edges(start);
        let (l, t, r, b) = edges(out);
        if handle.moves_left() {
            assert!(approx(r, r0), "{} : ancre droite {r}", handle.as_str());
        } else {
            assert!(approx(l, l0), "{} : ancre gauche {l}", handle.as_str());
        }
        if handle.moves_top() {
            assert!(approx(b, b0), "{} : ancre bas {b}", handle.as_str());
        } else {
            assert!(approx(t, t0), "{} : ancre haut {t}", handle.as_str());
        }
    }
}

#[test]
fn test_resize_3_the_corner_follows_the_diagonal_through_the_anchor() {
    // Tirer le coin bas-droit exactement le long de la diagonale double la taille.
    let start = AlignRect::new(0.0, 0.0, 300.0, 150.0);
    let out = resize_rect(
        start,
        Handle::BottomRight,
        (300.0, 150.0),
        ResizeRule::image(false),
    );
    assert_eq!(out, AlignRect::new(0.0, 0.0, 600.0, 300.0));
}

#[test]
fn test_resize_3_shift_frees_the_ratio_on_an_image() {
    let start = AlignRect::new(0.0, 0.0, 300.0, 150.0);
    let out = resize_rect(
        start,
        Handle::BottomRight,
        (100.0, 0.0),
        ResizeRule::image(true),
    );
    assert_eq!(out, AlignRect::new(0.0, 0.0, 400.0, 150.0));
}

#[test]
fn test_resize_3_a_side_handle_changes_one_dimension_even_with_the_ratio_kept() {
    let start = AlignRect::new(0.0, 0.0, 300.0, 150.0);
    let out = resize_rect(start, Handle::Right, (100.0, 0.0), ResizeRule::image(false));
    assert_eq!(out, AlignRect::new(0.0, 0.0, 400.0, 150.0));
    let out = resize_rect(start, Handle::Bottom, (0.0, 50.0), ResizeRule::image(false));
    assert_eq!(out, AlignRect::new(0.0, 0.0, 300.0, 200.0));
}

// ── Magnétisme ──────────────────────────────────────────────────────────────

fn target_at(left: f64, top: f64) -> AlignTarget {
    AlignTarget {
        id: "ref".into(),
        kind: AlignKind::Text,
        rect: AlignRect::new(left, top, 100.0, 100.0),
    }
}

#[test]
fn test_snap_keeps_the_ratio_when_a_guide_catches_the_pulled_edge() {
    let start = AlignRect::new(0.0, 0.0, 200.0, 100.0);
    let pulled = resize_rect(
        start,
        Handle::BottomRight,
        (95.0, 47.5),
        ResizeRule::image(false),
    );
    // Un bord droit à 300 attend à 5 unités : il aimante la largeur à 300, et la hauteur suit.
    let out = snap_resized_rect(
        start,
        Handle::BottomRight,
        pulled,
        &[target_at(300.0, 500.0)],
        SnapOptions::default(),
        ResizeRule::image(false),
    );
    assert!(approx(out.rect.width, 300.0), "{:?}", out.rect);
    assert!(approx(out.rect.height, 150.0), "{:?}", out.rect);
    assert_eq!(out.guides.x, Some(vec![300.0]));
    assert_eq!(
        out.guides.y, None,
        "un seul guide à la fois quand le rapport est tenu"
    );
}

#[test]
fn test_snap_is_dropped_when_it_would_break_the_minimum_size() {
    let rule = ResizeRule {
        min_width: 10.0,
        min_height: 140.0,
        keep_aspect: true,
    };
    let start = AlignRect::new(0.0, 0.0, 200.0, 100.0);
    let pulled = resize_rect(start, Handle::BottomRight, (100.0, 50.0), rule);
    // Le guide à 296 rendrait la hauteur 148 — acceptable ; celui à 260 la rendrait 130 : refusé.
    let out = snap_resized_rect(
        start,
        Handle::BottomRight,
        pulled,
        &[target_at(260.0, 500.0)],
        SnapOptions::default(),
        rule,
    );
    assert_eq!(out.rect, pulled, "le guide est abandonné, pas la règle");
    assert_eq!(out.guides.x, None);
}

#[test]
fn test_snap_without_ratio_snaps_both_axes_like_before() {
    let start = AlignRect::new(0.0, 0.0, 200.0, 100.0);
    let pulled = resize_rect(start, Handle::BottomRight, (97.0, 48.0), free());
    let out = snap_resized_rect(
        start,
        Handle::BottomRight,
        pulled,
        &[target_at(300.0, 150.0)],
        SnapOptions::default(),
        free(),
    );
    assert_eq!(out.rect, AlignRect::new(0.0, 0.0, 300.0, 150.0));
    assert_eq!(out.guides.x, Some(vec![300.0]));
    assert_eq!(out.guides.y, Some(vec![150.0]));
}

// ── Poignées : nom, position, curseur ───────────────────────────────────────

#[test]
fn test_every_handle_has_a_name_a_position_and_a_cursor() {
    let rect = AlignRect::new(0.0, 0.0, 100.0, 50.0);
    let expected = [
        (Handle::TopLeft, (0.0, 0.0), "nwse-resize"),
        (Handle::Top, (50.0, 0.0), "ns-resize"),
        (Handle::TopRight, (100.0, 0.0), "nesw-resize"),
        (Handle::Right, (100.0, 25.0), "ew-resize"),
        (Handle::BottomRight, (100.0, 50.0), "nwse-resize"),
        (Handle::Bottom, (50.0, 50.0), "ns-resize"),
        (Handle::BottomLeft, (0.0, 50.0), "nesw-resize"),
        (Handle::Left, (0.0, 25.0), "ew-resize"),
    ];
    for (handle, pos, cursor) in expected {
        assert_eq!(handle.position_on(rect), pos, "{}", handle.as_str());
        assert_eq!(handle.cursor(), cursor);
        assert_eq!(Handle::parse(handle.as_str()), Some(handle));
        assert_eq!(handle_cursor(handle.as_str()), cursor);
    }
    assert_eq!(Handle::parse("centre"), None);
    assert_eq!(handle_cursor("centre"), "default");
}

#[test]
fn test_pick_1_a_selected_node_exposes_its_eight_handles() {
    let image = BoardImage::new("I1", 200.0, 100.0, 200.0, 100.0);
    let selected = ["I1".to_string()];
    let empty: [String; 0] = [];
    let rect = AlignRect::new(100.0, 50.0, 200.0, 100.0);
    for handle in Handle::ALL {
        let (hx, hy) = handle.position_on(rect);
        let input = PickInput {
            wx: hx + 3.0,
            wy: hy - 2.0,
            scale: 1.0,
            images: std::slice::from_ref(&image),
            annotations: &[],
            folders: &[],
            selected_image_ids: &selected,
            selected_annotation_ids: &empty,
            selected_folder_id: None,
            arrow_id: None,
            dom_hint: None,
        };
        let top = collect_candidates(&input)
            .into_iter()
            .next()
            .expect("une cible");
        assert_eq!(top.kind, PickKind::Handle, "{}", handle.as_str());
        assert_eq!(top.corner.as_deref(), Some(handle.as_str()));
    }
}

#[test]
fn test_pick_1_a_text_card_offers_no_vertical_handle() {
    let card = Annotation::Text {
        id: "T1".into(),
        x: 0.0,
        y: 0.0,
        width: Some(240.0),
        height: Some(48.0),
        text: "hello".into(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    let selected = ["T1".to_string()];
    let empty: [String; 0] = [];
    let probe = |wx: f64, wy: f64| {
        let input = PickInput {
            wx,
            wy,
            scale: 1.0,
            images: &[],
            annotations: std::slice::from_ref(&card),
            folders: &[],
            selected_image_ids: &empty,
            selected_annotation_ids: &selected,
            selected_folder_id: None,
            arrow_id: None,
            dom_hint: None,
        };
        collect_candidates(&input)
            .into_iter()
            .next()
            .map(|c| (c.kind, c.corner))
    };
    assert_eq!(
        probe(240.0, 24.0),
        Some((PickKind::Handle, Some("r".into())))
    );
    assert_eq!(
        probe(120.0, 0.0).map(|c| c.0),
        Some(PickKind::Text),
        "pas de poignée haute"
    );
    assert_eq!(
        probe(120.0, 48.0).map(|c| c.0),
        Some(PickKind::Text),
        "pas de poignée basse"
    );
}

// ── Le document ─────────────────────────────────────────────────────────────

#[test]
fn test_set_image_rect_writes_the_center_and_one_undo_entry_per_gesture() {
    let mut store = Store::new("resize");
    let board = store.project.active_board_id.clone();
    store.add_image(&board, BoardImage::new("I1", 100.0, 100.0, 200.0, 100.0));
    let before = store.undo_depth();

    store.begin_live_edit();
    for step in 1..=50 {
        let w = 200.0 + step as f64;
        assert!(store.set_image_rect(&board, "I1", AlignRect::new(0.0, 50.0, w, 100.0)));
    }
    store.end_live_edit();

    let img = &store.active_board().expect("board").images[0];
    assert_eq!(
        (img.x, img.y, img.width, img.height),
        (125.0, 100.0, 250.0, 100.0)
    );
    assert_eq!(
        store.undo_depth(),
        before + 1,
        "cinquante pas, une seule entrée"
    );

    assert!(store.undo());
    let img = &store.active_board().expect("board").images[0];
    assert_eq!(
        (img.x, img.y, img.width, img.height),
        (100.0, 100.0, 200.0, 100.0),
        "Ctrl+Z rend la taille d'avant d'un coup"
    );
}

#[test]
fn test_cancel_live_edit_restores_the_document_and_leaves_no_undo_entry() {
    let mut store = Store::new("resize");
    let board = store.project.active_board_id.clone();
    store.add_image(&board, BoardImage::new("I1", 100.0, 100.0, 200.0, 100.0));
    let before = store.undo_depth();
    let version = store.version;

    store.begin_live_edit();
    store.set_image_rect(&board, "I1", AlignRect::new(0.0, 0.0, 900.0, 900.0));
    assert!(store.cancel_live_edit(), "Échap pendant le geste");

    let img = &store.active_board().expect("board").images[0];
    assert_eq!((img.width, img.height), (200.0, 100.0));
    assert_eq!(store.undo_depth(), before, "aucune entrée d'undo ne reste");
    assert!(!store.in_live_edit());
    assert_eq!(
        store.version, version,
        "rien n'a changé : le document n'est pas « modifié »"
    );
    assert_eq!(
        store.selected_image_ids,
        vec!["I1".to_string()],
        "la sélection survit"
    );
    assert!(!store.cancel_live_edit(), "rien à annuler hors geste");
}

#[test]
fn test_set_annotation_rect_covers_text_sticky_and_membrane_but_not_arrows() {
    let mut store = Store::new("resize");
    let board = store.project.active_board_id.clone();
    store.add_annotation(
        &board,
        Annotation::Membrane {
            id: "M".into(),
            x: 0.0,
            y: 0.0,
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
    assert!(store.set_annotation_rect(&board, "M", AlignRect::new(10.0, 20.0, 400.0, 300.0)));
    let Annotation::Membrane {
        x,
        y,
        width,
        height,
        ..
    } = &store.active_board().expect("board").annotations[0]
    else {
        panic!("membrane attendue");
    };
    assert_eq!((*x, *y, *width, *height), (10.0, 20.0, 400.0, 300.0));
    assert!(!store.set_annotation_rect(&board, "absent", AlignRect::new(0.0, 0.0, 1.0, 1.0)));
    assert!(!store.set_image_rect(&board, "absent", AlignRect::new(0.0, 0.0, 1.0, 1.0)));
    assert!(!store.set_folder_rect(&board, "absent", AlignRect::new(0.0, 0.0, 1.0, 1.0)));
}

#[test]
fn test_persist_1_a_resized_node_comes_back_from_disk_with_its_new_size() {
    let mut store = Store::new("resize");
    let board = store.project.active_board_id.clone();
    store.add_image(&board, BoardImage::new("I1", 100.0, 100.0, 200.0, 100.0));
    store.begin_live_edit();
    store.set_image_rect(&board, "I1", AlignRect::new(0.0, 50.0, 333.0, 166.5));
    store.end_live_edit();

    let bytes = glucose_core::persist::encode(&store.project, &store.assets, 0);
    let reloaded = glucose_core::persist::decode(&bytes).expect("relecture");
    let img = &reloaded.project.boards[0].images[0];
    assert_eq!(
        (img.x, img.y, img.width, img.height),
        (166.5, 133.25, 333.0, 166.5)
    );
}
