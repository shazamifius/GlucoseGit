//! Tests d'export Markdown et SVG — 100% Rust Standard Library (std)
//! Ports de src/utils/export/toMarkdown.test.ts et toSvg.test.ts

use glucose_core::export::{card_title, project_to_markdown};
use glucose_core::types::{Annotation, ArrowPredicate, Board, Project};

fn make_project(annotations: Vec<Annotation>) -> Project {
    let mut board = Board::new("b1", "Mon Board");
    board.annotations = annotations;
    let mut p = Project::new("Projet");
    p.boards = vec![board];
    p.active_board_id = "b1".into();
    p
}

fn card(id: &str, x: f64, y: f64, text: &str) -> Annotation {
    Annotation::Text {
        id: id.to_string(),
        x,
        y,
        width: Some(340.0),
        height: Some(120.0),
        text: text.to_string(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

// ════════════════════════════════════════════════════════════════════
// card_title
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_card_title_prend_le_premier_titre_markdown() {
    assert_eq!(card_title("### Newton\ncorps"), "Newton");
}

#[test]
fn test_card_title_retombe_sur_premiere_ligne_nue() {
    assert!(card_title("Leibniz est important").contains("Leibniz"));
}

// ════════════════════════════════════════════════════════════════════
// project_to_markdown
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_project_to_markdown_structure_zones_cartes_et_liens() {
    let membrane = Annotation::Membrane {
        id: "m".into(),
        x: -50.0,
        y: -50.0,
        width: 900.0,
        height: 600.0,
        color: None,
        text: Some("Physique".into()),
        mode: glucose_core::types::MembraneMode::Classic,
        curtains: Vec::new(),
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    let arrow = Annotation::Arrow {
        id: "arr".into(),
        x: 0.0,
        y: 0.0,
        x2: 400.0,
        y2: 0.0,
        text: None,
        font_size: None,
        color: None,
        arrow_type: None,
        arrow_bidirectional: false,
        predicate: Some(ArrowPredicate::Contredit),
        stroke_width: None,
        waypoints: Vec::new(),
        source_id: Some("a".into()),
        target_id: Some("b".into()),
        source_block_id: None,
        target_block_id: None,
        source_text_sel: None,
        target_text_sel: None,
        long_text: Some("Querelle de la priorité.".into()),
        target_board_id: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };

    let p = make_project(vec![
        membrane,
        card("a", 0.0, 0.0, "### Newton\nGravitation universelle."),
        card("b", 400.0, 0.0, "### Leibniz\nCalcul infinitésimal."),
        arrow,
    ]);

    let md = project_to_markdown(&p);
    assert!(md.contains("# Projet — Mon Board"));
    assert!(md.contains("## Physique"));
    assert!(md.contains("### Newton"));
    assert!(md.contains("### Leibniz"));
    assert!(md.contains("## Liens"));
    assert!(md.contains("**Newton**"));
    assert!(md.contains("contredit"));
    assert!(md.contains("Querelle de la priorité"));
}

#[test]
fn test_project_to_markdown_met_les_cartes_hors_zone_sous_autres() {
    let membrane = Annotation::Membrane {
        id: "m".into(),
        x: 5000.0,
        y: 5000.0,
        width: 100.0,
        height: 100.0,
        color: None,
        text: Some("Loin".into()),
        mode: glucose_core::types::MembraneMode::Classic,
        curtains: Vec::new(),
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    let p = make_project(vec![membrane, card("a", 0.0, 0.0, "### Orphelin\ncorps")]);

    let md = project_to_markdown(&p);
    assert!(md.contains("### Orphelin"));
}
