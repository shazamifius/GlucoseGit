//! Tests portés fidèlement de src/canvas/membraneSpace.test.ts

use glucose_core::membrane_space::{
    can_switch_mode, contained_in, content_extent, content_scale, has_scaling, items_of_board,
    origin_of, parent_map, project_board, reconcile_membership, resolve_items, scale_of,
    ResolveOptions, SpaceItem, SpaceItemKind, MIN_CONTENT_SCALE,
};
use glucose_core::membrane_stretch::stretch_plan;
use glucose_core::types::{Annotation, Board, BoardImage, MembraneMode, Point2D};

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

fn eff(
    items: &[SpaceItem],
    id: &str,
    focused_membrane_id: Option<&str>,
) -> glucose_core::membrane_space::ResolvedItem {
    let r = resolve_items(
        items,
        ResolveOptions {
            focused_membrane_id,
        },
    );
    r.get(id)
        .cloned()
        .unwrap_or_else(|| panic!("{} absent de la résolution", id))
}

#[test]
fn test_membership_invariant_reduced_content_not_lost() {
    let m = memb("M", 0.0, 0.0, 100.0, 100.0, MembraneMode::Minimized, None);
    let i = box_item("I", 0.0, 0.0, 1000.0, 1000.0, Some("M"));
    let items = [m, i];
    let pmap = parent_map(&items);
    assert_eq!(pmap.get("I"), Some(&"M".to_string()));

    let r = eff(&items, "I", None);
    assert_eq!(r.membrane_id.as_deref(), Some("M"));
    assert!(approx_eq(r.width, 100.0));
}

#[test]
fn test_membership_deleted_membrane_releases_element() {
    let items = [box_item("I", 0.0, 0.0, 10.0, 10.0, Some("DISPARUE"))];
    assert!(!parent_map(&items).contains_key("I"));
}

#[test]
fn test_membership_non_membrane_parent_ignored() {
    let items = [
        box_item("A", 0.0, 0.0, 10.0, 10.0, None),
        box_item("B", 0.0, 0.0, 10.0, 10.0, Some("A")),
    ];
    assert!(!parent_map(&items).contains_key("B"));
}

#[test]
fn test_membership_self_reference_ignored() {
    let items = [memb(
        "M",
        0.0,
        0.0,
        10.0,
        10.0,
        MembraneMode::Classic,
        Some("M"),
    )];
    assert!(!parent_map(&items).contains_key("M"));
}

#[test]
fn test_membership_cycle_broken_and_resolves() {
    let a = memb(
        "A",
        0.0,
        0.0,
        100.0,
        100.0,
        MembraneMode::Minimized,
        Some("B"),
    );
    let b = memb(
        "B",
        0.0,
        0.0,
        100.0,
        100.0,
        MembraneMode::Minimized,
        Some("A"),
    );
    let items = [a, b];
    assert_eq!(parent_map(&items).len(), 0);
    assert_eq!(resolve_items(&items, ResolveOptions::default()).len(), 2);
}

#[test]
fn test_contained_in_center_inside() {
    let m = memb("M", 0.0, 0.0, 400.0, 400.0, MembraneMode::Classic, None);
    let dedans = box_item("dedans", 100.0, 100.0, 50.0, 50.0, None);
    let dehors = box_item("dehors", 380.0, 100.0, 100.0, 50.0, None); // centre en x = 430
    let items = [m.clone(), dedans, dehors];
    let inside = contained_in(&items, &m);
    assert_eq!(inside.len(), 1);
    assert_eq!(inside[0].id, "dedans");

    let m_only = [m.clone()];
    assert_eq!(contained_in(&m_only, &m).len(), 0);
}

#[test]
fn test_content_scale_single_and_both_axes() {
    let contenu = box_item("I", 0.0, 0.0, 800.0, 600.0, Some("M"));
    let m = memb("M", 0.0, 0.0, 400.0, 300.0, MembraneMode::Minimized, None);
    let (ext_w, ext_h) = content_extent(m.rect(), &[&contenu]);
    assert!(approx_eq(
        content_scale(MembraneMode::Minimized, m.rect(), ext_w, ext_h),
        0.5
    ));
    assert!(approx_eq(
        eff(&[m.clone(), contenu.clone()], "I", None).width,
        400.0
    ));

    // Étirer un seul axe ne fait pas regrossir le contenu (min des deux)
    let large = memb("M", 0.0, 0.0, 800.0, 300.0, MembraneMode::Minimized, None);
    let (ext_lw, ext_lh) = content_extent(large.rect(), &[&contenu]);
    assert!(approx_eq(
        content_scale(MembraneMode::Minimized, large.rect(), ext_lw, ext_lh),
        0.5
    ));
    assert!(approx_eq(
        eff(&[large, contenu.clone()], "I", None).width,
        400.0
    ));

    // Étirer le second axe libère la croissance
    let carre = memb("M", 0.0, 0.0, 800.0, 600.0, MembraneMode::Minimized, None);
    let (ext_cw, ext_ch) = content_extent(carre.rect(), &[&contenu]);
    assert!(approx_eq(
        content_scale(MembraneMode::Minimized, carre.rect(), ext_cw, ext_ch),
        1.0
    ));
    assert!(approx_eq(
        eff(&[carre, contenu.clone()], "I", None).width,
        800.0
    ));

    // Plafonne à 1
    let vaste = memb("M", 0.0, 0.0, 1000.0, 800.0, MembraneMode::Minimized, None);
    let (ext_vw, ext_vh) = content_extent(vaste.rect(), &[&contenu]);
    assert!(approx_eq(
        content_scale(MembraneMode::Minimized, vaste.rect(), ext_vw, ext_vh),
        1.0
    ));

    // Plancher
    let minuscule = memb("M", 0.0, 0.0, 1.0, 1.0, MembraneMode::Minimized, None);
    assert!(approx_eq(
        content_scale(MembraneMode::Minimized, minuscule.rect(), 10000.0, 10000.0),
        MIN_CONTENT_SCALE
    ));

    // Classic & Stretched restent à 1
    let m_classic = memb("M", 0.0, 0.0, 400.0, 300.0, MembraneMode::Classic, None);
    assert!(approx_eq(
        content_scale(MembraneMode::Classic, m_classic.rect(), ext_w, ext_h),
        1.0
    ));
    assert!(approx_eq(
        content_scale(MembraneMode::Stretched, m_classic.rect(), ext_w, ext_h),
        1.0
    ));

    // Vide reste à 1
    assert!(approx_eq(
        content_scale(MembraneMode::Minimized, m.rect(), 0.0, 0.0),
        1.0
    ));
}

#[test]
fn test_focus_mode() {
    let m = memb("M", 0.0, 0.0, 400.0, 300.0, MembraneMode::Minimized, None);
    let i = box_item("I", 0.0, 0.0, 800.0, 600.0, Some("M"));
    let items = [m.clone(), i.clone()];

    assert!(approx_eq(eff(&items, "I", None).width, 400.0));
    assert!(approx_eq(eff(&items, "I", Some("M")).width, 800.0));
    assert!(approx_eq(eff(&items, "I", Some("M")).scale, 1.0));
    assert_eq!(i.width, 800.0);

    // Propagé aux membranes imbriquées
    let inner = memb(
        "IN",
        0.0,
        0.0,
        100.0,
        100.0,
        MembraneMode::Minimized,
        Some("M"),
    );
    let deep = box_item("D", 0.0, 0.0, 400.0, 400.0, Some("IN"));
    let items_nested = [m, inner, deep];
    assert!(approx_eq(eff(&items_nested, "D", Some("M")).scale, 1.0));

    // Focaliser une autre ne change rien ici
    let autre = memb(
        "AUTRE",
        5000.0,
        5000.0,
        100.0,
        100.0,
        MembraneMode::Classic,
        None,
    );
    let items_autre = [items[0].clone(), items[1].clone(), autre];
    assert!(approx_eq(
        eff(&items_autre, "I", Some("AUTRE")).width,
        400.0
    ));
}

#[test]
fn test_nested_membranes_compose_scale() {
    let outer = memb("O", 0.0, 0.0, 500.0, 500.0, MembraneMode::Minimized, None);
    let inner = memb(
        "IN",
        0.0,
        0.0,
        1000.0,
        1000.0,
        MembraneMode::Minimized,
        Some("O"),
    );
    let leaf = box_item("L", 0.0, 0.0, 2000.0, 2000.0, Some("IN"));
    let items = [outer, inner, leaf];

    assert!(approx_eq(eff(&items, "IN", None).scale, 0.5));
    assert!(approx_eq(eff(&items, "L", None).scale, 0.25));
    assert!(approx_eq(eff(&items, "L", None).width, 500.0));

    // Ancré sur le coin haut-gauche
    let m = memb(
        "M",
        100.0,
        100.0,
        200.0,
        200.0,
        MembraneMode::Minimized,
        None,
    );
    let a = box_item("A", 100.0, 100.0, 400.0, 400.0, Some("M"));
    assert!(approx_eq(eff(&[m, a], "A", None).x, 100.0));

    // Élément libre intact
    let libre = box_item("L_free", 0.0, 0.0, 400.0, 400.0, None);
    let m2 = memb("M2", 0.0, 0.0, 100.0, 100.0, MembraneMode::Minimized, None);
    let dedans = box_item("D", 0.0, 0.0, 400.0, 400.0, Some("M2"));
    let items_free = [m2, dedans, libre];
    assert!(approx_eq(eff(&items_free, "L_free", None).width, 400.0));
    assert!(approx_eq(eff(&items_free, "L_free", None).scale, 1.0));
}

#[test]
fn test_stretched_mode_plan() {
    let m = memb("M", 0.0, 0.0, 200.0, 200.0, MembraneMode::Stretched, None);
    let dedans = box_item("I", 0.0, 0.0, 300.0, 100.0, Some("M"));

    let plan = stretch_plan(&m, &[&dedans], &[]);
    assert!(approx_eq(
        plan.desired.width,
        300.0 + glucose_core::membrane_space::STRETCH_PADDING
    ));
    assert_eq!(plan.allowed, plan.desired);
    assert!(!plan.blocked);
    assert_eq!(plan.blockers.len(), 0);

    // Obstacle étranger bloque
    let etranger = box_item("ETR", 250.0, 0.0, 50.0, 50.0, None);
    let plan_blocked = stretch_plan(&m, &[&dedans], &[&etranger]);
    assert!(plan_blocked.blocked);
    assert_eq!(plan_blocked.blockers.len(), 1);
    assert_eq!(plan_blocked.blockers[0].id, "ETR");
    assert!(approx_eq(plan_blocked.allowed.width, 250.0));
    assert!(plan_blocked.allowed.width < plan_blocked.desired.width);
}

#[test]
fn test_can_switch_mode() {
    assert!(can_switch_mode(
        MembraneMode::Minimized,
        MembraneMode::Stretched
    ));
    assert!(can_switch_mode(
        MembraneMode::Stretched,
        MembraneMode::Minimized
    ));
    assert!(can_switch_mode(
        MembraneMode::Classic,
        MembraneMode::Minimized
    ));
    assert!(can_switch_mode(
        MembraneMode::Classic,
        MembraneMode::Stretched
    ));
    assert!(!can_switch_mode(
        MembraneMode::Minimized,
        MembraneMode::Classic
    ));
    assert!(!can_switch_mode(
        MembraneMode::Stretched,
        MembraneMode::Classic
    ));
    assert!(can_switch_mode(
        MembraneMode::Minimized,
        MembraneMode::Minimized
    ));
    assert!(can_switch_mode(
        MembraneMode::Classic,
        MembraneMode::Classic
    ));
}

#[test]
fn test_reconcile_membership() {
    let m = memb("M", 0.0, 0.0, 400.0, 400.0, MembraneMode::Classic, None);
    let autre = memb(
        "AUTRE",
        1000.0,
        0.0,
        400.0,
        400.0,
        MembraneMode::Classic,
        None,
    );

    // Lâché dedans rejoint la membrane
    let libre = box_item("I", 100.0, 100.0, 50.0, 50.0, None);
    let items = vec![m.clone(), libre];
    let res = resolve_items(&items, ResolveOptions::default());
    let ch = reconcile_membership(&items, &res, &["I".into()]);
    assert_eq!(ch.len(), 1);
    assert_eq!(ch[0].id, "I");
    assert_eq!(ch[0].membrane_id.as_deref(), Some("M"));

    // Sorti de la membrane redevient libre
    let parti = box_item("I", 5000.0, 5000.0, 50.0, 50.0, Some("M"));
    let items_parti = vec![m.clone(), parti];
    let res_parti = resolve_items(&items_parti, ResolveOptions::default());
    let ch_parti = reconcile_membership(&items_parti, &res_parti, &["I".into()]);
    assert_eq!(ch_parti.len(), 1);
    assert_eq!(ch_parti[0].membrane_id, None);

    // Migration directe
    let migre = box_item("I", 1100.0, 100.0, 50.0, 50.0, Some("M"));
    let items_migre = vec![m.clone(), autre.clone(), migre];
    let res_migre = resolve_items(&items_migre, ResolveOptions::default());
    let ch_migre = reconcile_membership(&items_migre, &res_migre, &["I".into()]);
    assert_eq!(ch_migre.len(), 1);
    assert_eq!(ch_migre[0].membrane_id.as_deref(), Some("AUTRE"));

    // Petite l'emporte
    let petite = memb(
        "PETITE",
        50.0,
        50.0,
        100.0,
        100.0,
        MembraneMode::Classic,
        None,
    );
    let it0 = box_item("I", 80.0, 80.0, 20.0, 20.0, None);
    let items_imb = vec![m.clone(), petite.clone(), it0];
    let res_imb = resolve_items(&items_imb, ResolveOptions::default());
    let ch_imb = reconcile_membership(&items_imb, &res_imb, &["I".into()]);
    assert_eq!(ch_imb.len(), 1);
    assert_eq!(ch_imb[0].membrane_id.as_deref(), Some("PETITE"));

    // Anti-cycle : grande ne peut pas entrer dans petite qui lui appartient
    let grande = memb(
        "GRANDE",
        0.0,
        0.0,
        400.0,
        400.0,
        MembraneMode::Classic,
        None,
    );
    let petite_child = memb(
        "PETITE_C",
        0.0,
        0.0,
        380.0,
        380.0,
        MembraneMode::Classic,
        Some("GRANDE"),
    );
    let items_cycle = vec![grande, petite_child];
    let res_cycle = resolve_items(&items_cycle, ResolveOptions::default());
    let ch_cycle = reconcile_membership(&items_cycle, &res_cycle, &["GRANDE".into()]);
    assert_eq!(ch_cycle.len(), 0);
}

#[test]
fn test_project_board_and_scaling() {
    let mut board = Board::default();
    let img = BoardImage::new("I", 100.0, 100.0, 50.0, 50.0);
    let m = Annotation::Membrane {
        id: "M".into(),
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 400.0,
        color: None,
        text: None,
        mode: MembraneMode::Classic,
        curtains: Vec::new(),
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    board.images.push(img);
    board.annotations.push(m);

    let items = items_of_board(&board);
    assert!(!has_scaling(&items));

    let proj = project_board(&board, None);
    assert_eq!(proj.images[0].x, 100.0);

    // Scaling triggered with minimized membrane
    let mut board_mini = Board::default();
    let mut img_mini = BoardImage::new("I", 400.0, 400.0, 800.0, 800.0);
    img_mini.membrane_id = Some("M".into());
    let m_mini = Annotation::Membrane {
        id: "M".into(),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 200.0,
        color: None,
        text: None,
        mode: MembraneMode::Minimized,
        curtains: Vec::new(),
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    board_mini.images.push(img_mini);
    board_mini.annotations.push(m_mini);

    let items_mini = items_of_board(&board_mini);
    assert!(has_scaling(&items_mini));

    let resolved = resolve_items(&items_mini, ResolveOptions::default());
    let out = project_board(&board_mini, Some(&resolved));
    assert!(approx_eq(out.images[0].x, 100.0));
    assert!(approx_eq(out.images[0].y, 100.0));
    assert!(approx_eq(out.images[0].width, 200.0));
    assert!(approx_eq(out.images[0].height, 200.0));

    // Data in source unchanged
    assert_eq!(board_mini.images[0].x, 400.0);

    // Scale and origin
    assert!(approx_eq(scale_of(Some(&resolved), "I"), 0.25));
    let orig = origin_of(Some(&resolved), "I").unwrap();
    assert!(approx_eq(orig.scale, 0.25));
    assert_eq!(scale_of(Some(&resolved), "M"), 1.0);
    assert_eq!(origin_of(Some(&resolved), "M"), None);
}

#[test]
fn test_project_arrow_in_membrane() {
    let mut board = Board::default();
    let mut a = BoardImage::new("A", 100.0, 100.0, 100.0, 100.0);
    a.membrane_id = Some("M".into());
    let mut b = BoardImage::new("B", 500.0, 500.0, 100.0, 100.0);
    b.membrane_id = Some("M".into());
    let mut etalon = BoardImage::new("ETALON", 750.0, 750.0, 100.0, 100.0);
    etalon.membrane_id = Some("M".into());

    let m = Annotation::Membrane {
        id: "M".into(),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 200.0,
        color: None,
        text: None,
        mode: MembraneMode::Minimized,
        curtains: Vec::new(),
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };
    let arrow = Annotation::Arrow {
        id: "F".into(),
        x: 100.0,
        y: 100.0,
        x2: 500.0,
        y2: 500.0,
        text: None,
        font_size: None,
        color: None,
        arrow_type: None,
        arrow_bidirectional: false,
        predicate: None,
        stroke_width: None,
        waypoints: vec![Point2D { x: 300.0, y: 200.0 }],
        source_id: Some("A".into()),
        target_id: Some("B".into()),
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

    board.images.push(etalon);
    board.images.push(a);
    board.images.push(b);
    board.annotations.push(m);
    board.annotations.push(arrow);

    let items = items_of_board(&board);
    let resolved = resolve_items(&items, ResolveOptions::default());
    let projected = project_board(&board, Some(&resolved));

    let f = projected
        .annotations
        .iter()
        .find(|ann| ann.id() == "F")
        .unwrap();
    if let Annotation::Arrow {
        x,
        y,
        x2,
        y2,
        waypoints,
        ..
    } = f
    {
        assert!(approx_eq(*x, 25.0));
        assert!(approx_eq(*y, 25.0));
        assert!(approx_eq(*x2, 125.0));
        assert!(approx_eq(*y2, 125.0));
        assert!(approx_eq(waypoints[0].x, 75.0));
        assert!(approx_eq(waypoints[0].y, 50.0));
    } else {
        panic!("F must be Arrow");
    }
}
