//! Tests portés fidèlement de src/canvas/smartAlign.test.ts

use glucose_core::smart_align::{
    collect_align_targets, rect_of_image, same_guides, snap_move, snap_point, snap_resize,
    union_rect, AlignKind, AlignRect, AlignTarget, SnapGuides, SnapOptions,
};
use glucose_core::types::{Annotation, Board, BoardImage, CanvasFolder, MembraneMode};
use std::collections::HashSet;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-5
}

fn r_target() -> AlignTarget {
    AlignTarget {
        id: "ref".into(),
        kind: AlignKind::Text,
        rect: AlignRect::new(0.0, 0.0, 100.0, 100.0),
    }
}

fn box_rect(left: f64, top: f64, width: f64, height: f64) -> AlignRect {
    AlignRect::new(left, top, width, height)
}

#[test]
fn test_snap_move_left_edge() {
    let r = snap_move(
        box_rect(3.0, 500.0, 40.0, 40.0),
        &[r_target()],
        SnapOptions::default(),
    );
    assert!(approx_eq(r.dx, -3.0));
    assert_eq!(r.guides.x, Some(vec![0.0]));
}

#[test]
fn test_snap_move_centers() {
    let r = snap_move(
        box_rect(27.0, 500.0, 40.0, 40.0),
        &[r_target()],
        SnapOptions::default(),
    );
    assert!(approx_eq(r.dx, 3.0));
    assert_eq!(r.guides.x, Some(vec![50.0]));
}

#[test]
fn test_snap_move_beyond_threshold() {
    let r = snap_move(
        box_rect(40.0, 500.0, 40.0, 40.0),
        &[r_target()],
        SnapOptions::default(),
    );
    assert_eq!(r.dx, 0.0);
    assert_eq!(r.guides.x, None);
}

#[test]
fn test_snap_move_independent_axes() {
    let r = snap_move(
        box_rect(2.0, 103.0, 40.0, 40.0),
        &[r_target()],
        SnapOptions::default(),
    );
    assert!(approx_eq(r.dx, -2.0));
    assert!(approx_eq(r.dy, -3.0));
    assert_eq!(r.guides.x, Some(vec![0.0]));
    assert_eq!(r.guides.y, Some(vec![100.0]));
}

#[test]
fn test_snap_move_scale_constant_on_screen() {
    let t = [r_target()];
    let p1 = snap_point(
        20.0,
        500.0,
        &t,
        SnapOptions {
            scale: 0.1,
            ..Default::default()
        },
    );
    assert!(approx_eq(p1.x, 0.0));

    let p2 = snap_point(
        20.0,
        500.0,
        &t,
        SnapOptions {
            scale: 4.0,
            ..Default::default()
        },
    );
    assert_eq!(p2.x, 20.0);
}

#[test]
fn test_snap_move_axis_disabled() {
    let r = snap_move(
        box_rect(3.0, 3.0, 40.0, 40.0),
        &[r_target()],
        SnapOptions {
            axis_x: false,
            ..Default::default()
        },
    );
    assert_eq!(r.dx, 0.0);
    assert_eq!(r.guides.x, None);
    assert!(approx_eq(r.dy, -3.0));
}

#[test]
fn test_snap_move_closest_target_wins() {
    let other = AlignTarget {
        id: "o".into(),
        kind: AlignKind::Image,
        rect: box_rect(6.0, 0.0, 10.0, 10.0),
    };
    let r = snap_move(
        box_rect(5.0, 500.0, 40.0, 40.0),
        &[r_target(), other],
        SnapOptions::default(),
    );
    assert_eq!(r.guides.x, Some(vec![6.0]));
}

#[test]
fn test_snap_resize_pulled_edge_only() {
    let far = snap_resize(
        box_rect(200.0, 200.0, 100.0, 100.0),
        "br",
        &[r_target()],
        SnapOptions::default(),
        1.0,
        1.0,
    );
    assert_eq!(far.rect, box_rect(200.0, 200.0, 100.0, 100.0));
    assert_eq!(far.guides.x, None);
}

#[test]
fn test_snap_resize_right_edge() {
    let r = snap_resize(
        box_rect(-60.0, 500.0, 157.0, 40.0),
        "br",
        &[r_target()],
        SnapOptions::default(),
        1.0,
        1.0,
    );
    assert_eq!(r.rect.left, -60.0);
    assert!(approx_eq(r.rect.width, 160.0));
    assert_eq!(r.guides.x, Some(vec![100.0]));
}

#[test]
fn test_snap_resize_left_edge() {
    let r = snap_resize(
        box_rect(3.0, 500.0, 200.0, 40.0),
        "tl",
        &[r_target()],
        SnapOptions::default(),
        1.0,
        1.0,
    );
    assert!(approx_eq(r.rect.left, 0.0));
    assert!(approx_eq(r.rect.width, 203.0));
    assert_eq!(r.guides.x, Some(vec![0.0]));
}

#[test]
fn test_snap_resize_non_pulled_edge_does_not_snap() {
    let r = snap_resize(
        box_rect(500.0, 3.0, 40.0, 300.0),
        "br",
        &[r_target()],
        SnapOptions::default(),
        1.0,
        1.0,
    );
    assert_eq!(r.rect.top, 3.0);
    assert_eq!(r.rect.height, 300.0);
    assert_eq!(r.guides.y, None);
}

#[test]
fn test_snap_resize_abandon_if_violates_min_size() {
    let r = snap_resize(
        box_rect(-60.0, 500.0, 157.0, 40.0),
        "br",
        &[r_target()],
        SnapOptions::default(),
        200.0, // min_width
        1.0,
    );
    assert_eq!(r.rect.width, 157.0);
    assert_eq!(r.guides.x, None);
}

#[test]
fn test_snap_point_both_axes() {
    let p = snap_point(4.0, 96.0, &[r_target()], SnapOptions::default());
    assert!(approx_eq(p.x, 0.0));
    assert!(approx_eq(p.y, 100.0));
    assert_eq!(p.guides.x, Some(vec![0.0]));
    assert_eq!(p.guides.y, Some(vec![100.0]));
}

#[test]
fn test_snap_point_beyond_threshold() {
    let p = snap_point(400.0, 400.0, &[r_target()], SnapOptions::default());
    assert_eq!(p.x, 400.0);
    assert_eq!(p.y, 400.0);
    assert_eq!(p.guides.x, None);
}

#[test]
fn test_collect_align_targets() {
    let img = BoardImage::new("i1", 100.0, 100.0, 40.0, 20.0);
    let text = Annotation::Text {
        id: "t1".into(),
        x: 0.0,
        y: 0.0,
        width: Some(50.0),
        height: Some(20.0),
        text: String::new(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    let memb = Annotation::Membrane {
        id: "m1".into(),
        x: 10.0,
        y: 10.0,
        width: 300.0,
        height: 200.0,
        color: Some("#fff".into()),
        text: None,
        mode: MembraneMode::Classic,
        curtains: Vec::new(),
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    let arrow = Annotation::Arrow {
        id: "a1".into(),
        x: 0.0,
        y: 0.0,
        x2: 5.0,
        y2: 5.0,
        text: None,
        font_size: None,
        color: None,
        arrow_type: None,
        arrow_bidirectional: false,
        predicate: None,
        stroke_width: None,
        waypoints: Vec::new(),
        source_id: None,
        target_id: None,
        source_block_id: None,
        target_block_id: None,
        source_text_sel: None,
        target_text_sel: None,
        long_text: None,
        target_board_id: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    let folder = CanvasFolder {
        id: "f1".into(),
        name: "D".into(),
        color: "#fff".into(),
        x: 400.0,
        y: 0.0,
        width: 200.0,
        height: 150.0,
        child_board_id: "b2".into(),
        mirror_of: None,
        mirror_source: None,
    };

    let mut board = Board::default();
    board.images.push(img);
    board.annotations.push(text);
    board.annotations.push(memb);
    board.annotations.push(arrow);
    board.folders.push(folder);

    // Includes images, texts, membranes, folders - never arrows
    let targets = collect_align_targets(&board, &HashSet::new());
    let mut kinds: Vec<&str> = targets
        .iter()
        .map(|t| match t.kind {
            AlignKind::Folder => "folder",
            AlignKind::Image => "image",
            AlignKind::Membrane => "membrane",
            AlignKind::Text => "text",
            AlignKind::Sticky => "sticky",
        })
        .collect();
    kinds.sort();
    assert_eq!(kinds, vec!["folder", "image", "membrane", "text"]);

    // Image center to top-left conversion
    let it = targets.iter().find(|t| t.id == "i1").unwrap();
    assert_eq!(it.rect, box_rect(80.0, 90.0, 40.0, 20.0));

    // Excludes manipulated elements
    let mut excl = HashSet::new();
    excl.insert("i1".into());
    excl.insert("f1".into());
    let filtered_targets = collect_align_targets(&board, &excl);
    let mut ids: Vec<&str> = filtered_targets.iter().map(|t| t.id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, vec!["m1", "t1"]);

    // Folder serves as target for other elements
    let mut excl_all_but_f = HashSet::new();
    excl_all_but_f.insert("t1".into());
    excl_all_but_f.insert("m1".into());
    excl_all_but_f.insert("i1".into());
    let targets_f = collect_align_targets(&board, &excl_all_but_f);
    let snap_res = snap_move(
        box_rect(403.0, 900.0, 40.0, 40.0),
        &targets_f,
        SnapOptions::default(),
    );
    assert_eq!(snap_res.guides.x, Some(vec![400.0]));
}

#[test]
fn test_union_rect_and_rect_of_image() {
    let u = union_rect(&[
        box_rect(0.0, 0.0, 10.0, 10.0),
        box_rect(90.0, 40.0, 10.0, 10.0),
    ])
    .unwrap();
    assert_eq!(u, box_rect(0.0, 0.0, 100.0, 50.0));

    assert_eq!(union_rect(&[]), None);

    let img = BoardImage::new("x", 0.0, 0.0, 10.0, 4.0);
    assert_eq!(rect_of_image(&img), box_rect(-5.0, -2.0, 10.0, 4.0));
}

#[test]
fn test_same_guides() {
    assert!(same_guides(None, None));
    assert!(same_guides(None, Some(&SnapGuides::default())));
    assert!(same_guides(
        Some(&SnapGuides {
            x: Some(vec![]),
            y: None
        }),
        None
    ));

    let g1 = SnapGuides {
        x: Some(vec![10.0]),
        y: None,
    };
    let g2 = SnapGuides {
        x: Some(vec![11.0]),
        y: None,
    };
    let g3 = SnapGuides {
        x: Some(vec![10.0]),
        y: None,
    };
    let g4 = SnapGuides {
        x: None,
        y: Some(vec![10.0]),
    };

    assert!(!same_guides(Some(&g1), Some(&g2)));
    assert!(same_guides(Some(&g1), Some(&g3)));
    assert!(!same_guides(Some(&g1), Some(&g4)));
}
