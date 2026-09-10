//! Tests portés fidèlement de src/canvas/membraneFocus.test.ts

use glucose_core::geometry::Rect;
use glucose_core::membrane_focus::{
    annotation_visible_under_focus, arrow_visible_under_focus, coverage, fit_viewport,
    focus_background, focus_box, focus_consts, focus_decision, focus_frame_of, screen_center_world,
    visible_under_focus, FocusAction, FocusInput, FocusState, ScreenSize,
};
use glucose_core::membrane_space::{resolve_items, ResolveOptions, SpaceItem, SpaceItemKind};
use glucose_core::types::{Annotation, MembraneMode, Viewport};
use std::collections::HashSet;

const SCREEN: ScreenSize = ScreenSize {
    width: 1000.0,
    height: 800.0,
};

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-5
}

fn memb(id: &str, x: f64, y: f64, w: f64, h: f64, mode: MembraneMode, membrane_id: Option<&str>) -> SpaceItem {
    SpaceItem {
        id: id.to_string(),
        kind: SpaceItemKind::Membrane,
        x,
        y,
        width: w,
        height: h,
        mode: Some(mode),
        membrane_id: membrane_id.map(String::from),
    }
}

fn box_item(id: &str, x: f64, y: f64, w: f64, h: f64, membrane_id: Option<&str>) -> SpaceItem {
    SpaceItem {
        id: id.to_string(),
        kind: SpaceItemKind::Image,
        x,
        y,
        width: w,
        height: h,
        mode: None,
        membrane_id: membrane_id.map(String::from),
    }
}

fn centered_on(cx: f64, cy: f64, scale: f64) -> Viewport {
    Viewport {
        scale,
        x: SCREEN.width / 2.0 - cx * scale,
        y: SCREEN.height / 2.0 - cy * scale,
    }
}

fn decide(items: &[SpaceItem], vp: Viewport, state: FocusState, now: i64) -> FocusAction {
    let resolved = resolve_items(items, ResolveOptions::default());
    focus_decision(FocusInput {
        items,
        resolved: &resolved,
        vp,
        screen: SCREEN,
        state,
        now,
    })
}

#[test]
fn test_fit_viewport_and_coverage() {
    let vp = fit_viewport(Rect::new(0.0, 0.0, 500.0, 400.0), SCREEN, focus_consts::FIT_PADDING);
    assert!(approx_eq(vp.scale, 1.76));

    let c = screen_center_world(vp, SCREEN);
    assert!(approx_eq(c.x, 250.0));
    assert!(approx_eq(c.y, 200.0));

    let m = memb("M", 0.0, 0.0, 100.0, 100.0, MembraneMode::Minimized, None);
    let enfant = box_item("I", 0.0, 0.0, 1000.0, 1000.0, Some("M"));
    assert_eq!(focus_box(&m, &[&enfant]), Rect::new(0.0, 0.0, 1000.0, 1000.0));

    let m_large = memb("M", 0.0, 0.0, 900.0, 900.0, MembraneMode::Classic, None);
    let enfant_small = box_item("I", 0.0, 0.0, 100.0, 100.0, Some("M"));
    assert_eq!(focus_box(&m_large, &[&enfant_small]), Rect::new(0.0, 0.0, 900.0, 900.0));

    assert!(approx_eq(
        coverage(Rect::new(0.0, 0.0, 500.0, 400.0), centered_on(250.0, 200.0, 2.0), SCREEN),
        1.0
    ));
}

#[test]
fn test_enter_focus() {
    let m = memb("M", 0.0, 0.0, 500.0, 400.0, MembraneMode::Classic, None);
    let items = [m.clone()];

    let r = decide(&items, centered_on(250.0, 200.0, 2.0), FocusState::default(), 10_000);
    if let FocusAction::Enter { membrane_id, fit, state } = r {
        assert_eq!(membrane_id, "M");
        assert!(approx_eq(fit.scale, 1.76));
        assert!(approx_eq(state.enter_scale, 1.76));
    } else {
        panic!("Should have entered focus");
    }

    // Pas assez zoomé -> reste
    let r_zoom = decide(&items, centered_on(250.0, 200.0, 1.5), FocusState::default(), 10_000);
    assert!(matches!(r_zoom, FocusAction::Stay(_)));

    // Centre hors membrane -> reste
    let r_off = decide(&items, centered_on(5000.0, 5000.0, 2.0), FocusState::default(), 10_000);
    assert!(matches!(r_off, FocusAction::Stay(_)));

    // Plus petite membrane qui remplit l'écran
    let grande = memb("GRANDE", 0.0, 0.0, 2000.0, 1600.0, MembraneMode::Classic, None);
    let petite = memb("PETITE", 200.0, 100.0, 500.0, 400.0, MembraneMode::Classic, Some("GRANDE"));
    let r_nest = decide(&[grande, petite], centered_on(450.0, 300.0, 2.0), FocusState::default(), 10_000);
    if let FocusAction::Enter { membrane_id, .. } = r_nest {
        assert_eq!(membrane_id, "PETITE");
    } else {
        panic!("Should have entered PETITE");
    }
}

#[test]
fn test_no_oscillation_after_cadrage() {
    let m = memb("M", 0.0, 0.0, 500.0, 400.0, MembraneMode::Classic, None);
    let items = [m.clone()];

    let fit = fit_viewport(focus_box(&m, &[]), SCREEN, focus_consts::FIT_PADDING);
    let cov = coverage(Rect::new(0.0, 0.0, 500.0, 400.0), fit, SCREEN);
    assert!(cov < focus_consts::ENTER_COVERAGE);
    assert!(approx_eq(cov, 0.88 * 0.88));

    let enter = decide(&items, centered_on(250.0, 200.0, 2.0), FocusState::default(), 10_000);
    if let FocusAction::Enter { state, fit, .. } = enter {
        let after = decide(&items, fit, state, 10_000 + focus_consts::COOLDOWN_MS + 1);
        assert!(matches!(after, FocusAction::Stay(_)));
    } else {
        panic!("Expected Enter");
    }
}

#[test]
fn test_exit_focus() {
    let m = memb("M", 0.0, 0.0, 500.0, 400.0, MembraneMode::Classic, None);
    let items = [m];

    let state = FocusState {
        membrane_id: Some("M".into()),
        enter_scale: 1.76,
        t: 10_000,
    };

    // Dézoom sous le seuil
    let trop = 1.76 * focus_consts::EXIT_SCALE_RATIO - 0.01;
    let r = decide(&items, centered_on(250.0, 200.0, trop), state.clone(), 20_000);
    assert!(matches!(r, FocusAction::Exit(_)));

    // Dézoom léger -> reste
    let peu = 1.76 * focus_consts::EXIT_SCALE_RATIO + 0.01;
    let r_peu = decide(&items, centered_on(250.0, 200.0, peu), state.clone(), 20_000);
    assert!(matches!(r_peu, FocusAction::Stay(_)));

    // Pan loin -> sort
    let loin = centered_on(5000.0, 200.0, 1.76);
    let r_loin = decide(&items, loin, state.clone(), 20_000);
    assert!(matches!(r_loin, FocusAction::Exit(_)));

    // Membrane disparue -> sort
    let r_disp = decide(&[], centered_on(250.0, 200.0, 1.76), state, 20_000);
    assert!(matches!(r_disp, FocusAction::Exit(_)));
}

#[test]
fn test_visible_under_focus() {
    assert_eq!(visible_under_focus(&[memb("M", 0.0, 0.0, 10.0, 10.0, MembraneMode::Classic, None)], None), None);

    let items = [
        memb("M", 0.0, 0.0, 400.0, 400.0, MembraneMode::Minimized, None),
        box_item("direct", 0.0, 0.0, 50.0, 50.0, Some("M")),
        memb("SOUS", 0.0, 0.0, 100.0, 100.0, MembraneMode::Minimized, Some("M")),
        box_item("profond", 0.0, 0.0, 20.0, 20.0, Some("SOUS")),
        box_item("dehors", 5000.0, 5000.0, 50.0, 50.0, None),
    ];
    let v = visible_under_focus(&items, Some("M")).unwrap();
    assert!(v.contains("M"));
    assert!(v.contains("SOUS"));
    assert!(v.contains("direct"));
    assert!(v.contains("profond"));
    assert!(!v.contains("dehors"));

    // Legacy unparented inside membrane is visible
    let legacy_items = [
        memb("M", 0.0, 0.0, 400.0, 400.0, MembraneMode::Classic, None),
        box_item("legacy", 100.0, 100.0, 50.0, 50.0, None),
    ];
    let v_leg = visible_under_focus(&legacy_items, Some("M")).unwrap();
    assert!(v_leg.contains("legacy"));

    // Belonging to another is not pulled in
    let other_items = [
        memb("M", 0.0, 0.0, 400.0, 400.0, MembraneMode::Classic, None),
        memb("AUTRE", 0.0, 0.0, 400.0, 400.0, MembraneMode::Classic, None),
        box_item("a-autrui", 100.0, 100.0, 50.0, 50.0, Some("AUTRE")),
    ];
    let v_oth = visible_under_focus(&other_items, Some("M")).unwrap();
    assert!(!v_oth.contains("a-autrui"));
}

#[test]
fn test_arrow_and_annotation_visible_under_focus() {
    let fb = Some(Rect::new(0.0, 0.0, 400.0, 400.0));
    let mut visible = HashSet::new();
    visible.insert("A".to_string());
    visible.insert("B".to_string());
    visible.insert("dedans".to_string());

    let make_arrow = |src: Option<&str>, tgt: Option<&str>, x: f64, y: f64, x2: f64, y2: f64| Annotation::Arrow {
        id: "fl".into(),
        x,
        y,
        x2,
        y2,
        text: None,
        font_size: None,
        color: None,
        arrow_type: None,
        arrow_bidirectional: false,
        predicate: None,
        stroke_width: None,
        waypoints: Vec::new(),
        source_id: src.map(String::from),
        target_id: tgt.map(String::from),
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

    // Hors focus
    assert!(arrow_visible_under_focus(&make_arrow(None, None, 10.0, 10.0, 90.0, 90.0), None, None));

    // Attachée : visible ssi tous ses nœuds le sont
    assert!(arrow_visible_under_focus(&make_arrow(Some("A"), Some("B"), 0.0, 0.0, 0.0, 0.0), Some(&visible), fb));
    assert!(!arrow_visible_under_focus(&make_arrow(Some("A"), Some("Z"), 0.0, 0.0, 0.0, 0.0), Some(&visible), fb));

    // Libre : dans le cadre
    assert!(arrow_visible_under_focus(&make_arrow(None, None, 10.0, 10.0, 90.0, 90.0), Some(&visible), fb));
    assert!(!arrow_visible_under_focus(&make_arrow(None, None, 10.0, 10.0, 9000.0, 9000.0), Some(&visible), fb));

    // Text annotations
    let text_in = Annotation::Text {
        id: "dedans".into(),
        x: 0.0,
        y: 0.0,
        width: Some(10.0),
        height: Some(10.0),
        text: "a".into(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    let text_out = Annotation::Text {
        id: "dehors".into(),
        x: 0.0,
        y: 0.0,
        width: Some(10.0),
        height: Some(10.0),
        text: "a".into(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    assert!(annotation_visible_under_focus(&text_in, Some(&visible), fb));
    assert!(!annotation_visible_under_focus(&text_out, Some(&visible), fb));

    // Unmeasured text judged by anchor
    let text_unmeasured = Annotation::Text {
        id: "neuf".into(),
        x: 50.0,
        y: 50.0,
        width: None,
        height: None,
        text: "".into(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    assert!(annotation_visible_under_focus(&text_unmeasured, Some(&visible), fb));
}

#[test]
fn test_focus_background() {
    let bg = focus_background(Some("#60a5fa"), None, None);
    assert_eq!(bg.len(), 7);
    assert!(bg.starts_with('#'));
    assert_ne!(bg, "#60a5fa");

    assert_eq!(focus_background(Some("#ffffff"), Some("#000000"), Some(1.0)), "#ffffff");
    assert_eq!(focus_background(Some("#abc"), Some("#000000"), Some(1.0)), "#aabbcc");

    assert_eq!(focus_background(None, None, None), "#0d0d0d");
    assert_eq!(focus_background(Some("rebeccapurple"), None, None), "#0d0d0d");
    assert_eq!(focus_background(Some(""), None, None), "#0d0d0d");
}

#[test]
fn test_focus_frame_of() {
    let items = [
        memb("M", 0.0, 0.0, 100.0, 100.0, MembraneMode::Minimized, None),
        box_item("I", 0.0, 0.0, 800.0, 600.0, Some("M")),
    ];
    assert_eq!(focus_frame_of(&items, Some("M")), Some(Rect::new(0.0, 0.0, 800.0, 600.0)));
    assert_eq!(focus_frame_of(&items, None), None);
    assert_eq!(focus_frame_of(&items, Some("FANTOME")), None);
}
