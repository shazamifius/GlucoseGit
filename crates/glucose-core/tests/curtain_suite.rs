//! Tests portés fidèlement de src/canvas/curtainPanel.test.ts et curtainModel.test.ts

use glucose_core::curtain_model::{
    can_edit, can_see, config_of, create_curtain, create_note, curtain_kind, detach_curtains,
    notes_to_annotations, sanitize_note_text, visible_curtains, CurtainKind, CurtainOwner,
    MAX_NOTE_LENGTH,
};
use glucose_core::curtain_panel::{
    advance, canvas_strip, curtain_consts, decide, is_over_panel, normalize_config, panel_rect,
    step, CurtainConfig, CurtainPhase, CurtainState,
};
use glucose_core::types::{CurtainEditable, CurtainVisibility};

const SCREEN_W: f64 = 1000.0;
const SCREEN_H: f64 = 800.0;

fn at(frac: f64) -> f64 {
    SCREEN_W * frac
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-5
}

// ── Tests curtainPanel ───────────────────────────────────────────────────────

#[test]
fn test_curtain_panel_normalize_config_invariants() {
    let cfg = normalize_config(None);
    assert_eq!(cfg.collapsed, 0.1);
    assert_eq!(cfg.expanded, 0.9);

    let cases = [
        CurtainConfig { collapsed: 0.0, expanded: 0.9 },
        CurtainConfig { collapsed: -5.0, expanded: 0.9 },
        CurtainConfig { collapsed: 0.1, expanded: 1.0 },
        CurtainConfig { collapsed: 0.1, expanded: 42.0 },
        CurtainConfig { collapsed: 0.9, expanded: 0.1 },
        CurtainConfig { collapsed: 0.5, expanded: 0.5 },
        CurtainConfig { collapsed: f64::NAN, expanded: f64::INFINITY },
    ];

    for c in cases {
        let n = normalize_config(Some(c));
        assert!(n.collapsed > 0.0);
        assert!(n.expanded < 1.0);
        assert!(n.expanded > n.collapsed);
    }

    let custom = normalize_config(Some(CurtainConfig { collapsed: 0.15, expanded: 0.5 }));
    assert_eq!(custom, CurtainConfig { collapsed: 0.15, expanded: 0.5 });

    let inverted = normalize_config(Some(CurtainConfig { collapsed: 0.3, expanded: 0.1 }));
    assert!(inverted.expanded >= inverted.collapsed + curtain_consts::MIN_SPAN);
}

#[test]
fn test_curtain_panel_geometry() {
    let p1 = panel_rect(0.1, SCREEN_W, SCREEN_H);
    assert_eq!(p1.left, 900.0);
    assert_eq!(p1.top, 0.0);
    assert_eq!(p1.width, 100.0);
    assert_eq!(p1.height, 800.0);

    let p2 = panel_rect(0.9, SCREEN_W, SCREEN_H);
    assert_eq!(p2.left, 100.0);
    assert_eq!(p2.width, 900.0);

    for r in [0.1, 0.9, 0.333, 0.15, 0.85] {
        let cs = canvas_strip(r, SCREEN_W, SCREEN_H);
        let pr = panel_rect(r, SCREEN_W, SCREEN_H);
        assert_eq!(cs.width + pr.width, SCREEN_W);
        assert_eq!(cs.width, pr.left);
    }

    assert!(is_over_panel(Some(at(0.95)), 0.1, SCREEN_W));
    assert!(!is_over_panel(Some(at(0.5)), 0.1, SCREEN_W));
    assert!(is_over_panel(Some(at(0.5)), 0.9, SCREEN_W));
    assert!(!is_over_panel(None, 0.9, SCREEN_W));
}

#[test]
fn test_curtain_panel_simulation_and_dwell() {
    let cfg = normalize_config(None);
    let mut state = CurtainState::new(cfg);

    // 80ms sur la languette (< 90ms dwell) puis repart -> reste replié
    state = decide(&state, Some(at(0.95)), SCREEN_W, 0);
    state = decide(&state, Some(at(0.95)), SCREEN_W, 80);
    assert_eq!(state.phase, CurtainPhase::Collapsed);

    state = decide(&state, Some(at(0.2)), SCREEN_W, 90);
    assert_eq!(state.phase, CurtainPhase::Collapsed);
    assert_eq!(state.pending, None);

    // Rester assez longtemps -> déploie
    let mut s2 = CurtainState::new(cfg);
    s2 = decide(&s2, Some(at(0.95)), SCREEN_W, 0);
    s2 = decide(&s2, Some(at(0.95)), SCREEN_W, curtain_consts::EXPAND_DWELL_MS + 1);
    assert_eq!(s2.phase, CurtainPhase::Expanded);

    // Repli est plus court que déploiement
    const { assert!(curtain_consts::COLLAPSE_DWELL_MS < curtain_consts::EXPAND_DWELL_MS) };

    // Souris quitte la fenêtre -> repli
    let mut ouvert = CurtainState {
        ratio: cfg.expanded,
        phase: CurtainPhase::Expanded,
        pending: None,
        pending_since: 0,
    };
    for i in 1..=200 {
        ouvert = step(&ouvert, None, SCREEN_W, &cfg, i * 16, 16.0);
    }
    assert_eq!(ouvert.phase, CurtainPhase::Collapsed);
    assert!(approx_eq(ouvert.ratio, cfg.collapsed));
}

#[test]
fn test_curtain_panel_animation_monotone() {
    let cfg = normalize_config(None);
    let mut s = CurtainState {
        ratio: cfg.collapsed,
        phase: CurtainPhase::Expanded,
        pending: None,
        pending_since: 0,
    };

    let mut prev_ratio = s.ratio;
    for _ in 0..100 {
        s = advance(&s, &cfg, 16.0);
        assert!(s.ratio >= prev_ratio);
        assert!(s.ratio <= cfg.expanded + 1e-9);
        prev_ratio = s.ratio;
    }
    assert_eq!(s.ratio, cfg.expanded);
}

// ── Tests curtainModel ───────────────────────────────────────────────────────

#[test]
fn test_curtain_model_creation() {
    let moi = CurtainOwner {
        id: "u-moi".into(),
        name: "Ada".into(),
        color: "#38bdf8".into(),
    };
    let c = create_curtain("c1", moi, 1000);
    assert_eq!(c.visibility, CurtainVisibility::Private);
    assert_eq!(c.editable, CurtainEditable::Owner);
    assert_eq!(c.notes.len(), 0);
    assert_eq!(c.owner_id, "u-moi");
    assert_eq!(c.owner_name, "Ada");
    assert_eq!(c.owner_color, "#38bdf8");
}

#[test]
fn test_curtain_model_notes_sanitization() {
    assert_eq!(sanitize_note_text("  salut  "), "salut");
    let long = "x".repeat(5000);
    assert_eq!(sanitize_note_text(&long).len(), MAX_NOTE_LENGTH);
    assert_eq!(sanitize_note_text("a\r\nb"), "a\nb");

    let note = create_note("n1", "un\ndeux", 500);
    assert_eq!(note.text, "un\ndeux");
}

#[test]
fn test_curtain_model_permissions() {
    let moi = CurtainOwner {
        id: "u-moi".into(),
        name: "Ada".into(),
        color: "#38bdf8".into(),
    };

    // Carnet
    let mut c_carnet = create_curtain("c1", moi.clone(), 1000);
    c_carnet.visibility = CurtainVisibility::Private;
    c_carnet.editable = CurtainEditable::Owner;
    assert_eq!(curtain_kind(&c_carnet), CurtainKind::Carnet);
    assert!(can_see(&c_carnet, "u-moi"));
    assert!(can_edit(&c_carnet, "u-moi"));
    assert!(!can_see(&c_carnet, "u-toi"));
    assert!(!can_edit(&c_carnet, "u-toi"));

    // Vitrine
    let mut c_vitrine = create_curtain("c2", moi.clone(), 1000);
    c_vitrine.visibility = CurtainVisibility::Shared;
    c_vitrine.editable = CurtainEditable::Owner;
    assert_eq!(curtain_kind(&c_vitrine), CurtainKind::Vitrine);
    assert!(can_see(&c_vitrine, "u-toi"));
    assert!(!can_edit(&c_vitrine, "u-toi"));
    assert!(can_edit(&c_vitrine, "u-moi"));

    // Atelier
    let mut c_atelier = create_curtain("c3", moi, 1000);
    c_atelier.visibility = CurtainVisibility::Shared;
    c_atelier.editable = CurtainEditable::Everyone;
    assert_eq!(curtain_kind(&c_atelier), CurtainKind::Atelier);
    assert!(can_see(&c_atelier, "u-toi"));
    assert!(can_edit(&c_atelier, "u-toi"));

    // Incohérent : privé + modifiable par tous -> non éditable par autrui
    let mut c_incoh = c_carnet.clone();
    c_incoh.visibility = CurtainVisibility::Private;
    c_incoh.editable = CurtainEditable::Everyone;
    assert!(!can_see(&c_incoh, "u-toi"));
    assert!(!can_edit(&c_incoh, "u-toi"));
}

#[test]
fn test_curtain_model_visible_curtains_sorting() {
    let u_moi = CurtainOwner {
        id: "u-moi".into(),
        name: "Ada".into(),
        color: "#38bdf8".into(),
    };
    let u_toi = CurtainOwner {
        id: "u-toi".into(),
        name: "Grace".into(),
        color: "#34d399".into(),
    };

    let mut c1 = create_curtain("c1", u_toi.clone(), 100);
    c1.visibility = CurtainVisibility::Shared;
    let mut c2 = create_curtain("c2", u_moi.clone(), 200);
    c2.visibility = CurtainVisibility::Private;
    let mut c3_private_toi = create_curtain("c3", u_toi, 50);
    c3_private_toi.visibility = CurtainVisibility::Private;

    let curtains = vec![c1, c2, c3_private_toi];
    let vis = visible_curtains(&curtains, "u-moi");

    // c3 n'est pas visible pour moi
    assert_eq!(vis.len(), 2);
    // Le mien passe en premier même s'il a été créé après
    assert_eq!(vis[0].id, "c2");
    assert_eq!(vis[1].id, "c1");
}

#[test]
fn test_curtain_model_detach() {
    let u = CurtainOwner {
        id: "u".into(),
        name: "U".into(),
        color: "#000".into(),
    };
    let mut c = create_curtain("c", u, 100);
    c.notes.push(create_note("n1", "hello", 100));

    let detached = detach_curtains(&[c.clone()]);
    assert_eq!(detached.len(), 1);
    assert_eq!(detached[0].notes.len(), 1);
    assert_eq!(detached[0].notes[0].text, "hello");
}

#[test]
fn test_curtain_model_config_of() {
    let cfg_none = config_of(None);
    assert_eq!(cfg_none.collapsed, 0.1);
    assert_eq!(cfg_none.expanded, 0.9);

    let u = CurtainOwner {
        id: "u".into(),
        name: "U".into(),
        color: "#000".into(),
    };
    let mut c = create_curtain("c", u, 100);
    c.collapsed_ratio = Some(0.15);
    c.expanded_ratio = Some(0.85);
    let cfg = config_of(Some(&c));
    assert_eq!(cfg.collapsed, 0.15);
    assert_eq!(cfg.expanded, 0.85);
}

#[test]
fn test_notes_to_annotations() {
    let notes = vec![
        create_note("n1", "First note", 100),
        create_note("n2", "Second note", 200),
        create_note("n3", "   ", 300), // Whitespace only is dropped
    ];
    let anns = notes_to_annotations(&notes);
    assert_eq!(anns.len(), 2);
    assert_eq!(anns[0].id(), "n1-b");
    assert_eq!(anns[1].id(), "n2-b");
}
