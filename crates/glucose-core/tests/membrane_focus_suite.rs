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
    let vp = fit_viewport(
        Rect::new(0.0, 0.0, 500.0, 400.0),
        SCREEN,
        focus_consts::FIT_PADDING,
    );
    assert!(approx_eq(vp.scale, 1.76));

    let c = screen_center_world(vp, SCREEN);
    assert!(approx_eq(c.x, 250.0));
    assert!(approx_eq(c.y, 200.0));

    let m = memb("M", 0.0, 0.0, 100.0, 100.0, MembraneMode::Minimized, None);
    let enfant = box_item("I", 0.0, 0.0, 1000.0, 1000.0, Some("M"));
    assert_eq!(
        focus_box(&m, &[&enfant]),
        Rect::new(0.0, 0.0, 1000.0, 1000.0)
    );

    let m_large = memb("M", 0.0, 0.0, 900.0, 900.0, MembraneMode::Classic, None);
    let enfant_small = box_item("I", 0.0, 0.0, 100.0, 100.0, Some("M"));
    assert_eq!(
        focus_box(&m_large, &[&enfant_small]),
        Rect::new(0.0, 0.0, 900.0, 900.0)
    );

    assert!(approx_eq(
        coverage(
            Rect::new(0.0, 0.0, 500.0, 400.0),
            centered_on(250.0, 200.0, 2.0),
            SCREEN
        ),
        1.0
    ));
}

#[test]
fn test_enter_focus() {
    let m = memb("M", 0.0, 0.0, 500.0, 400.0, MembraneMode::Classic, None);
    let items = [m.clone()];

    let r = decide(
        &items,
        centered_on(250.0, 200.0, 2.0),
        FocusState::default(),
        10_000,
    );
    if let FocusAction::Enter {
        membrane_id,
        fit,
        state,
    } = r
    {
        assert_eq!(membrane_id, "M");
        assert!(approx_eq(fit.scale, 1.76));
        assert!(approx_eq(state.enter_scale, 1.76));
    } else {
        panic!("Should have entered focus");
    }

    // Pas assez zoomé -> reste
    let r_zoom = decide(
        &items,
        centered_on(250.0, 200.0, 1.5),
        FocusState::default(),
        10_000,
    );
    assert!(matches!(r_zoom, FocusAction::Stay(_)));

    // Centre hors membrane -> reste
    let r_off = decide(
        &items,
        centered_on(5000.0, 5000.0, 2.0),
        FocusState::default(),
        10_000,
    );
    assert!(matches!(r_off, FocusAction::Stay(_)));

    // Plus petite membrane qui remplit l'écran
    let grande = memb(
        "GRANDE",
        0.0,
        0.0,
        2000.0,
        1600.0,
        MembraneMode::Classic,
        None,
    );
    let petite = memb(
        "PETITE",
        200.0,
        100.0,
        500.0,
        400.0,
        MembraneMode::Classic,
        Some("GRANDE"),
    );
    let r_nest = decide(
        &[grande, petite],
        centered_on(450.0, 300.0, 2.0),
        FocusState::default(),
        10_000,
    );
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

    let enter = decide(
        &items,
        centered_on(250.0, 200.0, 2.0),
        FocusState::default(),
        10_000,
    );
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
    let r = decide(
        &items,
        centered_on(250.0, 200.0, trop),
        state.clone(),
        20_000,
    );
    assert!(matches!(r, FocusAction::Exit(_)));

    // Dézoom léger -> reste
    let peu = 1.76 * focus_consts::EXIT_SCALE_RATIO + 0.01;
    let r_peu = decide(
        &items,
        centered_on(250.0, 200.0, peu),
        state.clone(),
        20_000,
    );
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
    assert_eq!(
        visible_under_focus(
            &[memb("M", 0.0, 0.0, 10.0, 10.0, MembraneMode::Classic, None)],
            None
        ),
        None
    );

    let items = [
        memb("M", 0.0, 0.0, 400.0, 400.0, MembraneMode::Minimized, None),
        box_item("direct", 0.0, 0.0, 50.0, 50.0, Some("M")),
        memb(
            "SOUS",
            0.0,
            0.0,
            100.0,
            100.0,
            MembraneMode::Minimized,
            Some("M"),
        ),
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

    let make_arrow = |src: Option<&str>, tgt: Option<&str>, x: f64, y: f64, x2: f64, y2: f64| {
        Annotation::Arrow {
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
        }
    };

    // Hors focus
    assert!(arrow_visible_under_focus(
        &make_arrow(None, None, 10.0, 10.0, 90.0, 90.0),
        None,
        None
    ));

    // Attachée : visible ssi tous ses nœuds le sont
    assert!(arrow_visible_under_focus(
        &make_arrow(Some("A"), Some("B"), 0.0, 0.0, 0.0, 0.0),
        Some(&visible),
        fb
    ));
    assert!(!arrow_visible_under_focus(
        &make_arrow(Some("A"), Some("Z"), 0.0, 0.0, 0.0, 0.0),
        Some(&visible),
        fb
    ));

    // Libre : dans le cadre
    assert!(arrow_visible_under_focus(
        &make_arrow(None, None, 10.0, 10.0, 90.0, 90.0),
        Some(&visible),
        fb
    ));
    assert!(!arrow_visible_under_focus(
        &make_arrow(None, None, 10.0, 10.0, 9000.0, 9000.0),
        Some(&visible),
        fb
    ));

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
    assert!(!annotation_visible_under_focus(
        &text_out,
        Some(&visible),
        fb
    ));

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
    assert!(annotation_visible_under_focus(
        &text_unmeasured,
        Some(&visible),
        fb
    ));
}

#[test]
fn test_focus_background() {
    let bg = focus_background(Some("#60a5fa"), None, None);
    assert_eq!(bg.len(), 7);
    assert!(bg.starts_with('#'));
    assert_ne!(bg, "#60a5fa");

    assert_eq!(
        focus_background(Some("#ffffff"), Some("#000000"), Some(1.0)),
        "#ffffff"
    );
    assert_eq!(
        focus_background(Some("#abc"), Some("#000000"), Some(1.0)),
        "#aabbcc"
    );

    assert_eq!(focus_background(None, None, None), "#0d0d0d");
    assert_eq!(
        focus_background(Some("rebeccapurple"), None, None),
        "#0d0d0d"
    );
    assert_eq!(focus_background(Some(""), None, None), "#0d0d0d");
}

#[test]
fn test_focus_frame_of() {
    let items = [
        memb("M", 0.0, 0.0, 100.0, 100.0, MembraneMode::Minimized, None),
        box_item("I", 0.0, 0.0, 800.0, 600.0, Some("M")),
    ];
    assert_eq!(
        focus_frame_of(&items, Some("M")),
        Some(Rect::new(0.0, 0.0, 800.0, 600.0))
    );
    assert_eq!(focus_frame_of(&items, None), None);
    assert_eq!(focus_frame_of(&items, Some("FANTOME")), None);
}

/// Fiche 07 § 5.3 — le cadrage du mode focus garde une marge de 6 % autour de la membrane
/// (`FOCUS.FIT_PADDING = 0.06` dans la référence ; la fiche disait 10 %), en 320 ms.
#[test]
fn test_the_focus_fit_padding_is_six_percent() {
    use glucose_core::membrane_focus::focus_consts;
    assert_eq!(focus_consts::FIT_PADDING, 0.06);
    assert_eq!(focus_consts::FIT_ANIM_MS, 320);
}

/// **Le masque du focus** (MEMB-2) : par rang — images, puis annotations, puis dossiers, dans
/// l'ordre de l'index —, la membrane et ce qu'elle porte se voient ; une flèche qui en sort, un
/// dossier, ce qui est ailleurs, non.
#[test]
fn test_le_masque_du_focus_suit_l_ordre_des_rangs() {
    use glucose_core::membrane_focus::masque_du_focus;
    use glucose_core::store::Store;
    use glucose_core::types::{Annotation, BoardImage, CanvasFolder};

    let mut s = Store::new("focus");
    let b = s.project.active_board_id.clone();
    if let Some(board) = s.active_board_mut() {
        board.annotations.clear();
        board.images.clear();
    }
    s.add_annotation(&b, Annotation::membrane("M", 0.0, 0.0, 1000.0, 600.0));
    s.add_image(&b, BoardImage::new("dedans", 500.0, 300.0, 100.0, 80.0));
    s.add_image(&b, BoardImage::new("dehors", 5000.0, 300.0, 100.0, 80.0));
    let fleche = |id: &str, de: &str, vers: &str| {
        let mut f = Annotation::arrow(id, 0.0, 0.0, 1.0, 1.0);
        if let Annotation::Arrow {
            source_id,
            target_id,
            ..
        } = &mut f
        {
            *source_id = Some(de.into());
            *target_id = Some(vers.into());
        }
        f
    };
    s.add_annotation(&b, fleche("sortante", "dedans", "dehors"));
    s.add_annotation(&b, fleche("interieure", "dedans", "M"));
    // Un dossier capture ce qui est sous lui : il naît loin, et vide.
    let mut dossier = CanvasFolder::new("", "Dossier", "");
    dossier.x = 20_000.0;
    dossier.y = 20_000.0;
    s.create_folder(&b, dossier);

    let board = s.active_board().expect("le tableau");
    let m = masque_du_focus(board, "M").expect("M est une membrane");
    // Rangs : dedans, dehors | M, sortante, interieure | le dossier.
    assert_eq!(m.permis, vec![true, false, true, false, true, false]);
    assert!(m.laisse_voir("interieure") && !m.laisse_voir("sortante"));
    assert!(
        masque_du_focus(board, "dedans").is_none(),
        "une image n'est pas une membrane"
    );
}

/// **Le masque en un passage rend exactement ce que rendait la description complète** : la
/// preuve de la réécriture, sur un tableau fabriqué pour en éprouver les recoins —
/// membranes imbriquées et qui se chevauchent, une appartenance en boucle, des appartenances
/// vers ce qui n'est pas une membrane, des cartes mesurées ou non, des flèches accrochées ou
/// libres —, et pour chaque membrane focalisée.
#[test]
fn test_le_masque_rend_ce_que_rendait_la_description_complete() {
    use glucose_core::membrane_focus::{focus_frame_of, masque_du_focus};
    use glucose_core::membrane_space::items_of_board;
    use glucose_core::types::{Board, BoardImage, CanvasFolder};

    let mut graine = 0x2545_f491_4f6c_dd1du64;
    let mut hasard = |n: u64| {
        graine ^= graine << 13;
        graine ^= graine >> 7;
        graine ^= graine << 17;
        graine % n
    };
    let mut b = Board::new("b", "b");
    let membranes = ["m0", "m1", "m2", "m3", "m4"];
    for (k, id) in membranes.iter().enumerate() {
        let x = (k as f64) * 300.0;
        b.annotations.push(Annotation::membrane(
            *id,
            x,
            0.0,
            900.0 - 100.0 * k as f64,
            700.0,
        ));
    }
    // Une boucle : m3 dans m4, m4 dans m3 ; et m1 dans m0.
    b.annotations[3].set_membrane_id(Some("m4".into()));
    b.annotations[4].set_membrane_id(Some("m3".into()));
    b.annotations[1].set_membrane_id(Some("m0".into()));
    let parents = ["m0", "m1", "m2", "m3", "m4", "i3", "absente"];
    for i in 0..80 {
        let mut img = BoardImage::new(
            format!("i{i}"),
            hasard(2400) as f64 - 200.0,
            hasard(900) as f64 - 100.0,
            40.0 + hasard(200) as f64,
            30.0 + hasard(150) as f64,
        );
        if hasard(3) > 0 {
            img.membrane_id = Some(parents[hasard(parents.len() as u64) as usize].into());
        }
        b.images.push(img);
    }
    for t in 0..25 {
        let mut carte = Annotation::text(
            format!("t{t}"),
            hasard(2400) as f64 - 200.0,
            hasard(900) as f64,
            "une idée",
        );
        if hasard(2) == 0 {
            if let Annotation::Text { width, height, .. } = &mut carte {
                *width = Some(120.0);
                *height = Some(60.0);
            }
        }
        if hasard(2) == 0 {
            carte.set_membrane_id(Some(parents[hasard(5) as usize].into()));
        }
        b.annotations.push(carte);
    }
    for f in 0..20 {
        let bout = |h: u64| (h < 90).then(|| format!("i{h}"));
        let mut fl = Annotation::arrow(
            format!("f{f}"),
            hasard(2400) as f64,
            hasard(900) as f64,
            hasard(2400) as f64,
            hasard(900) as f64,
        );
        if let Annotation::Arrow {
            source_id,
            target_id,
            ..
        } = &mut fl
        {
            *source_id = bout(hasard(120));
            *target_id = bout(hasard(120));
        }
        b.annotations.push(fl);
    }
    b.folders.push(CanvasFolder::new("d", "Dossier", "enfant"));

    let items = items_of_board(&b);
    for m in membranes {
        let vis = visible_under_focus(&items, Some(m)).expect("une membrane");
        let cadre = focus_frame_of(&items, Some(m));
        let attendu: Vec<bool> = b
            .images
            .iter()
            .map(|i| vis.contains(&i.id))
            .chain(
                b.annotations
                    .iter()
                    .map(|a| annotation_visible_under_focus(a, Some(&vis), cadre)),
            )
            .chain(b.folders.iter().map(|_| false))
            .collect();
        let masque = masque_du_focus(&b, m).expect("une membrane");
        assert_eq!(masque.permis, attendu, "focus sur {m}");
        assert!(
            attendu.iter().filter(|v| **v).count() > 1,
            "le tableau éprouve quelque chose pour {m}"
        );
    }
}
