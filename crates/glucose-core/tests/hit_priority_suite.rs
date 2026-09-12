//! Tests complets portés de src/canvas/hitPriority.test.ts.

use glucose_core::hit_priority::{
    advance_on_release, collect_candidates, pick_at_down, CycleState, PickInput, PickKind,
    PickOptions,
};
use glucose_core::types::{Annotation, BoardImage, CanvasFolder, MembraneMode};

fn img(id: &str, x: f64, y: f64, w: f64, h: f64, locked: bool) -> BoardImage {
    let mut i = BoardImage::new(id, x, y, w, h);
    i.locked = locked;
    i
}

fn membrane(id: &str, x: f64, y: f64, w: f64, h: f64, text: Option<&str>) -> Annotation {
    Annotation::Membrane {
        id: id.to_string(),
        x,
        y,
        width: w,
        height: h,
        color: None,
        text: text.map(String::from),
        mode: MembraneMode::Classic,
        curtains: Vec::new(),
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

fn text_ann(id: &str, x: f64, y: f64, w: f64, h: f64) -> Annotation {
    Annotation::Text {
        id: id.to_string(),
        x,
        y,
        width: Some(w),
        height: Some(h),
        text: "hello".to_string(),
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

fn sticky(id: &str, x: f64, y: f64, w: f64, h: f64) -> Annotation {
    Annotation::Sticky {
        id: id.to_string(),
        x,
        y,
        width: Some(w),
        height: Some(h),
        text: "note".to_string(),
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
    }
}

fn folder(id: &str, x: f64, y: f64, w: f64, h: f64) -> CanvasFolder {
    CanvasFolder {
        id: id.into(),
        name: id.into(),
        color: "#60a5fa".into(),
        x,
        y,
        width: w,
        height: h,
        child_board_id: format!("{}-child", id),
        mirror_of: None,
        mirror_source: None,
    }
}

fn order(input: &PickInput) -> Vec<String> {
    collect_candidates(input)
        .into_iter()
        .map(|c| format!("{}:{}", c.kind.as_str(), c.id))
        .collect()
}

#[test]
fn test_hit_priority_rules() {
    let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
    let images = [img("I1", 500.0, 400.0, 200.0, 150.0, false)];
    let annotations = [memb];
    let empty_folders = [];
    let empty: [String; 0] = [];

    // Clic centre image -> image gagne
    let input = PickInput {
        wx: 500.0,
        wy: 400.0,
        scale: 1.0,
        images: &images,
        annotations: &annotations,
        folders: &empty_folders,
        selected_image_ids: &empty,
        selected_annotation_ids: &empty,
        selected_folder_id: None,
        arrow_id: None,
        dom_hint: None,
    };
    let cands = collect_candidates(&input);
    assert_eq!(cands[0].id, "I1");
    assert_eq!(cands[0].kind, PickKind::Image);
    assert_eq!(cands[1].id, "M1");
    assert_eq!(cands[1].kind, PickKind::MembraneBody);

    // Clic pointillés membrane -> membrane-edge gagne
    let images2 = [
        img("I1", 5.0, 400.0, 200.0, 150.0, false),
        img("I2", 40.0, 400.0, 200.0, 150.0, false),
    ];
    let input2 = PickInput {
        wx: 5.0,
        wy: 400.0,
        images: &images2,
        ..input
    };
    let cands2 = collect_candidates(&input2);
    assert_eq!(cands2[0].kind, PickKind::MembraneEdge);
    assert_eq!(cands2[0].id, "M1");

    // Bord depuis l'extérieur de la membrane
    let input3 = PickInput {
        wx: -8.0,
        wy: 400.0,
        images: &[],
        ..input
    };
    assert_eq!(order(&input3), vec!["membrane-edge:M1"]);

    // Étiquette peinte au-dessus du bord haut
    let with_label = membrane("M2", 0.0, 0.0, 1000.0, 800.0, Some("Recherche"));
    let anns_label = [with_label];
    let input_label = PickInput {
        wx: 200.0,
        wy: -20.0,
        annotations: &anns_label,
        images: &[],
        ..input
    };
    assert_eq!(order(&input_label), vec!["membrane-edge:M2"]);

    // Clic texte posé dans membrane -> texte puis corps membrane
    let anns_text = [
        membrane("M1", 0.0, 0.0, 1000.0, 800.0, None),
        text_ann("T1", 400.0, 380.0, 200.0, 60.0),
    ];
    let input_text = PickInput {
        wx: 500.0,
        wy: 400.0,
        annotations: &anns_text,
        images: &[],
        ..input
    };
    let cands_text = collect_candidates(&input_text);
    assert_eq!(cands_text[0].kind, PickKind::Text);
    assert_eq!(cands_text[1].kind, PickKind::MembraneBody);
}

#[test]
fn test_nested_containers_smallest_wins() {
    let inner = membrane("PETITE", 400.0, 300.0, 200.0, 200.0, None);
    let outer = membrane("GRANDE", 0.0, 0.0, 1000.0, 800.0, None);
    let empty: [String; 0] = [];

    let input1 = PickInput {
        wx: 500.0,
        wy: 400.0,
        scale: 1.0,
        images: &[],
        annotations: &[outer.clone(), inner.clone()],
        folders: &[],
        selected_image_ids: &empty,
        selected_annotation_ids: &empty,
        selected_folder_id: None,
        arrow_id: None,
        dom_hint: None,
    };
    assert_eq!(order(&input1)[0], "membrane-body:PETITE");

    let input2 = PickInput {
        annotations: &[inner, outer],
        ..input1
    };
    assert_eq!(order(&input2)[0], "membrane-body:PETITE");

    // Dossier dans membrane : conteneur le plus petit gagne
    let f1 = folder("F1", 420.0, 330.0, 300.0, 200.0);
    let input_f = PickInput {
        annotations: &[membrane("M1", 0.0, 0.0, 1000.0, 800.0, None)],
        folders: &[f1],
        ..input1
    };
    let cands_f = collect_candidates(&input_f);
    assert_eq!(cands_f[0].id, "F1");
    assert_eq!(cands_f[1].id, "M1");
}

#[test]
fn test_text_is_last_among_contents() {
    let empty: [String; 0] = [];
    let t = text_ann("T1", 0.0, 0.0, 200.0, 100.0);
    let s = sticky("S1", 0.0, 0.0, 200.0, 100.0);
    let i = img("I1", 100.0, 50.0, 300.0, 200.0, false);

    let input = PickInput {
        wx: 100.0,
        wy: 50.0,
        scale: 1.0,
        images: &[i],
        annotations: &[t, s],
        folders: &[],
        selected_image_ids: &empty,
        selected_annotation_ids: &empty,
        selected_folder_id: None,
        arrow_id: None,
        dom_hint: None,
    };

    let cands = collect_candidates(&input);
    assert_eq!(cands[0].kind, PickKind::Image);
    assert_eq!(cands[1].kind, PickKind::Sticky);
    assert_eq!(cands[2].kind, PickKind::Text);

    // Flèche passe devant image
    let input_arr = PickInput {
        arrow_id: Some("A1"),
        annotations: &[],
        ..input
    };
    let cands_arr = collect_candidates(&input_arr);
    assert_eq!(cands_arr[0].kind, PickKind::Arrow);
    assert_eq!(cands_arr[1].kind, PickKind::Image);
}

#[test]
fn test_handles_absolute_priority() {
    let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
    let i = img("I1", 10.0, 10.0, 300.0, 300.0, false);
    let sel_m = ["M1".to_string()];
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 10.0,
        wy: 10.0,
        scale: 1.0,
        images: &[i],
        annotations: &[memb],
        folders: &[],
        selected_image_ids: &empty,
        selected_annotation_ids: &sel_m,
        selected_folder_id: None,
        arrow_id: None,
        dom_hint: None,
    };

    let cands = collect_candidates(&input);
    assert_eq!(cands[0].kind, PickKind::Handle);
    assert_eq!(cands[0].corner.as_deref(), Some("tl"));

    // Pas de sélection -> aucune poignée
    let input_nosel = PickInput {
        selected_annotation_ids: &empty,
        ..input
    };
    assert!(collect_candidates(&input_nosel)
        .iter()
        .all(|c| c.kind != PickKind::Handle));

    // Préhension constante à l'écran (grandit au dézoom)
    let far_scale1 = PickInput {
        wx: 50.0,
        wy: 0.0,
        scale: 1.0,
        ..input
    };
    assert_ne!(collect_candidates(&far_scale1)[0].kind, PickKind::Handle);

    let far_scale025 = PickInput {
        scale: 0.25,
        ..far_scale1
    };
    assert_eq!(collect_candidates(&far_scale025)[0].kind, PickKind::Handle);
}

#[test]
fn test_click_cycle_chain() {
    let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
    let t = text_ann("T1", 0.0, 380.0, 100.0, 40.0);
    let i = img("I1", 8.0, 400.0, 200.0, 150.0, false);
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 8.0,
        wy: 400.0,
        scale: 1.0,
        images: &[i],
        annotations: &[memb, t],
        folders: &[],
        selected_image_ids: &empty,
        selected_annotation_ids: &empty,
        selected_folder_id: None,
        arrow_id: None,
        dom_hint: None,
    };

    let cands = collect_candidates(&input);
    let mut cycle: Option<CycleState> = None;
    let mut time = 1000i64;
    let mut seen = Vec::new();

    for _ in 0..5 {
        let (down_picked, down_cycle) = pick_at_down(
            &cands,
            cycle.as_ref(),
            300.0,
            300.0,
            time,
            PickOptions::default(),
        );
        cycle = down_cycle;
        time += 60; // appui court
        let (up_picked, up_cycle) = advance_on_release(&cands, cycle.as_ref(), time);
        cycle = up_cycle;
        let final_picked = up_picked.or(down_picked);
        seen.push(format!(
            "{}:{}",
            final_picked.as_ref().unwrap().kind.as_str(),
            final_picked.unwrap().id
        ));
        time += 500; // pause au-delà du double-clic
    }

    // Chaque clic descend d'un cran puis s'arrête au texte (terminus)
    assert_eq!(
        seen,
        vec![
            "membrane-edge:M1",
            "image:I1",
            "text:T1",
            "text:T1",
            "text:T1",
        ]
    );
}

// ── Fiche 07 § 3 — les chiffres de l'arbitre de clic, un par un ─────────────

use glucose_core::hit_priority::pick_consts;

/// § 3.1 — l'échelle de priorité : poignée 0, bords de conteneur 10, flèche 20, image 30,
/// note 40, texte 50, corps de conteneur 60. Une membrane ne gagne jamais sur son contenu.
#[test]
fn test_the_pick_ranks_are_those_of_the_spec() {
    use glucose_core::hit_priority::*;
    assert_eq!(PICK_RANK_HANDLE, 0);
    assert_eq!(PICK_RANK_MEMBRANE_EDGE, 10);
    assert_eq!(PICK_RANK_FOLDER_EDGE, 10);
    assert_eq!(PICK_RANK_ARROW, 20);
    assert_eq!(PICK_RANK_IMAGE, 30);
    assert_eq!(PICK_RANK_STICKY, 40);
    assert_eq!(PICK_RANK_TEXT, 50);
    assert_eq!(PICK_RANK_MEMBRANE_BODY, 60);
    assert_eq!(PICK_RANK_FOLDER_BODY, 60);
}

/// § 3.1 et § 3.2 — bande de 14 px autour d'un contour ; rayon de saisie d'une poignée
/// 24 px, plafonné à 35 % du petit côté, plancher 6 px.
/// § 3.3 — re-clic au même endroit : moins de 8 px, moins de 2,5 s ; fenêtre du double-clic.
#[test]
fn test_the_pick_tolerances_are_those_of_the_spec() {
    assert_eq!(pick_consts::EDGE_BAND_PX, 14.0);
    assert_eq!(pick_consts::HANDLE_SLOP_PX, 24.0);
    assert_eq!(pick_consts::HANDLE_SLOP_MAX_RATIO, 0.35);
    assert_eq!(pick_consts::HANDLE_SLOP_MIN_PX, 6.0);
    assert_eq!(pick_consts::CYCLE_RADIUS_PX, 8.0);
    assert_eq!(pick_consts::CYCLE_TTL_MS, 2500);
    assert_eq!(pick_consts::DBLCLICK_MS, 350, "une seule fenêtre de double-clic");
}
