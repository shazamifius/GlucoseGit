//! Tests portés fidèlement de src/canvas/membraneStretch.test.ts

use glucose_core::membrane_space::{SpaceItem, SpaceItemKind, STRETCH_PADDING};
use glucose_core::membrane_stretch::plan_board_stretch;
use glucose_core::types::MembraneMode;

const PAD: f64 = STRETCH_PADDING;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-5
}

fn memb(
    id: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    mode: MembraneMode,
    membrane_id: Option<&str>,
) -> SpaceItem {
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

#[test]
fn test_plan_board_stretch_grows_to_contain_content() {
    let items = [
        memb("M", 0.0, 0.0, 200.0, 200.0, MembraneMode::Stretched, None),
        box_item("I", 0.0, 0.0, 300.0, 100.0, Some("M")),
    ];
    let out = plan_board_stretch(&items);
    assert_eq!(out.len(), 1);
    let o = &out[0];
    assert_eq!(o.membrane_id, "M");
    assert!(o.grew);
    assert!(!o.blocked);
    assert_eq!(o.blocker_ids.len(), 0);
    assert!(approx_eq(o.width, 300.0 + PAD));
    assert!(approx_eq(o.height, 200.0));
}

#[test]
fn test_plan_board_stretch_already_large_enough_produces_nothing() {
    let items = [
        memb("M", 0.0, 0.0, 500.0, 500.0, MembraneMode::Stretched, None),
        box_item("I", 0.0, 0.0, 100.0, 100.0, Some("M")),
    ];
    assert_eq!(plan_board_stretch(&items).len(), 0);
}

#[test]
fn test_plan_board_stretch_classic_and_minimized_ignored() {
    let items = [
        memb("C", 0.0, 0.0, 50.0, 50.0, MembraneMode::Classic, None),
        box_item("A", 0.0, 0.0, 400.0, 400.0, Some("C")),
        memb("N", 1000.0, 0.0, 50.0, 50.0, MembraneMode::Minimized, None),
        box_item("B", 1000.0, 0.0, 400.0, 400.0, Some("N")),
    ];
    assert_eq!(plan_board_stretch(&items).len(), 0);
}

#[test]
fn test_plan_board_stretch_never_shrinks() {
    let items = [
        memb("M", 0.0, 0.0, 800.0, 800.0, MembraneMode::Stretched, None),
        box_item("I", 0.0, 0.0, 40.0, 40.0, Some("M")),
    ];
    assert_eq!(plan_board_stretch(&items).len(), 0);
}

#[test]
fn test_plan_board_stretch_empty_stretched_membrane_does_not_move() {
    let items = [memb(
        "M",
        0.0,
        0.0,
        200.0,
        200.0,
        MembraneMode::Stretched,
        None,
    )];
    assert_eq!(plan_board_stretch(&items).len(), 0);
}

#[test]
fn test_plan_board_stretch_free_element_blocks() {
    let items = [
        memb("M", 0.0, 0.0, 200.0, 200.0, MembraneMode::Stretched, None),
        box_item("I", 0.0, 0.0, 300.0, 100.0, Some("M")),
        box_item("ETR", 250.0, 0.0, 50.0, 50.0, None),
    ];
    let out = plan_board_stretch(&items);
    assert_eq!(out.len(), 1);
    assert!(out[0].blocked);
    assert_eq!(out[0].blocker_ids, vec!["ETR".to_string()]);
    assert!(approx_eq(out[0].width, 250.0));
    assert!(out[0].grew);
}

#[test]
fn test_plan_board_stretch_content_of_other_membrane_does_not_block() {
    let items = [
        memb("M", 0.0, 0.0, 200.0, 200.0, MembraneMode::Stretched, None),
        box_item("I", 0.0, 0.0, 300.0, 100.0, Some("M")),
        memb(
            "AUTRE",
            900.0,
            900.0,
            100.0,
            100.0,
            MembraneMode::Minimized,
            None,
        ),
        box_item("CACHE", 250.0, 0.0, 50.0, 50.0, Some("AUTRE")),
    ];
    let out = plan_board_stretch(&items);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].membrane_id, "M");
    assert!(!out[0].blocked);
    assert_eq!(out[0].blocker_ids.len(), 0);
    assert!(approx_eq(out[0].width, 300.0 + PAD));
}

#[test]
fn test_plan_board_stretch_own_content_never_blocks() {
    let items = [
        memb("M", 0.0, 0.0, 100.0, 100.0, MembraneMode::Stretched, None),
        box_item("A", 0.0, 0.0, 400.0, 50.0, Some("M")),
        box_item("B", 0.0, 200.0, 50.0, 50.0, Some("M")),
    ];
    let out = plan_board_stretch(&items);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].blocker_ids.len(), 0);
    assert!(!out[0].blocked);
}

#[test]
fn test_plan_board_stretch_parent_does_not_block_child() {
    let items = [
        memb("P", 0.0, 0.0, 1000.0, 1000.0, MembraneMode::Classic, None),
        memb(
            "M",
            0.0,
            0.0,
            200.0,
            200.0,
            MembraneMode::Stretched,
            Some("P"),
        ),
        box_item("I", 0.0, 0.0, 300.0, 100.0, Some("M")),
    ];
    let out = plan_board_stretch(&items);
    let m_res = out.iter().find(|o| o.membrane_id == "M").unwrap();
    assert!(!m_res.blocked);
    assert_eq!(m_res.blocker_ids.len(), 0);
}

#[test]
fn test_plan_board_stretch_sibling_blocks() {
    let items = [
        memb("P", 0.0, 0.0, 1000.0, 1000.0, MembraneMode::Classic, None),
        memb(
            "M",
            0.0,
            0.0,
            200.0,
            200.0,
            MembraneMode::Stretched,
            Some("P"),
        ),
        box_item("I", 0.0, 0.0, 300.0, 100.0, Some("M")),
        box_item("FRERE", 260.0, 0.0, 40.0, 40.0, None),
    ];
    // FRERE est à la racine, pas dans P : pas dans le repère de M
    let out1 = plan_board_stretch(&items);
    assert_eq!(
        out1.iter()
            .find(|o| o.membrane_id == "M")
            .unwrap()
            .blocker_ids
            .len(),
        0
    );

    // Si FRERE est dans P : il bloque M
    let items2 = [
        memb("P", 0.0, 0.0, 1000.0, 1000.0, MembraneMode::Classic, None),
        memb(
            "M",
            0.0,
            0.0,
            200.0,
            200.0,
            MembraneMode::Stretched,
            Some("P"),
        ),
        box_item("I", 0.0, 0.0, 300.0, 100.0, Some("M")),
        box_item("FRERE", 260.0, 0.0, 40.0, 40.0, Some("P")),
    ];
    let out2 = plan_board_stretch(&items2);
    assert_eq!(
        out2.iter()
            .find(|o| o.membrane_id == "M")
            .unwrap()
            .blocker_ids,
        vec!["FRERE".to_string()]
    );
}

#[test]
fn test_plan_board_stretch_nested_grow_before_parent() {
    let items = [
        memb(
            "PARENTE",
            0.0,
            0.0,
            150.0,
            150.0,
            MembraneMode::Stretched,
            None,
        ),
        memb(
            "FILLE",
            0.0,
            0.0,
            100.0,
            100.0,
            MembraneMode::Stretched,
            Some("PARENTE"),
        ),
        box_item("I", 0.0, 0.0, 300.0, 100.0, Some("FILLE")),
    ];
    let out = plan_board_stretch(&items);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].membrane_id, "FILLE");
    assert_eq!(out[1].membrane_id, "PARENTE");

    let fille = out.iter().find(|o| o.membrane_id == "FILLE").unwrap();
    let parente = out.iter().find(|o| o.membrane_id == "PARENTE").unwrap();
    assert!(approx_eq(fille.width, 300.0 + PAD));
    assert!(approx_eq(parente.width, 300.0 + PAD + PAD));
}

#[test]
fn test_plan_board_stretch_cycle_does_not_infinite_loop() {
    let items = [
        memb(
            "A",
            0.0,
            0.0,
            100.0,
            100.0,
            MembraneMode::Stretched,
            Some("B"),
        ),
        memb(
            "B",
            0.0,
            0.0,
            100.0,
            100.0,
            MembraneMode::Stretched,
            Some("A"),
        ),
    ];
    let _ = plan_board_stretch(&items);
}

/// Fiche 08 § 7.1 — le mode étiré englobe son contenu « plus une marge d'air de 32 px ».
#[test]
fn test_the_stretch_padding_is_thirty_two_pixels() {
    assert_eq!(glucose_core::membrane_space::STRETCH_PADDING, 32.0);
}
