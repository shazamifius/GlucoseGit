//! PICK-1 — regles d'arbitrage du clic, testees depuis l'exterieur du crate.
//! (Anciens tests unitaires de `hit_priority.rs`, deplaces lors de son decoupage.)

use glucose_core::hit_priority::*;
use glucose_core::types::{Annotation, BoardImage};
fn img(id: &str, x: f64, y: f64, w: f64, h: f64, locked: bool) -> BoardImage {
    let mut img = BoardImage::new(id, x, y, w, h);
    img.locked = locked;
    img
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
        mode: glucose_core::types::MembraneMode::Classic,
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

#[test]
fn test_pick_priority_image_in_membrane() {
    let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
    let images = [img("I1", 500.0, 400.0, 200.0, 150.0, false)];
    let annotations = [memb];
    let folders = [];
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 500.0,
        wy: 400.0,
        scale: 1.0,
        images: &images,
        annotations: &annotations,
        folders: &folders,
        selected_image_ids: &empty,
        selected_annotation_ids: &empty,
        selected_folder_id: None,
        noeuds: None,
    };

    let cands = collect_candidates(&input);
    assert_eq!(cands[0].id, "I1");
    assert_eq!(cands[0].kind, PickKind::Image);
    assert_eq!(cands[1].id, "M1");
    assert_eq!(cands[1].kind, PickKind::MembraneBody);
}

#[test]
fn test_pick_membrane_edge() {
    let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
    let images = [img("I1", 5.0, 400.0, 200.0, 150.0, false)];
    let annotations = [memb];
    let folders = [];
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 5.0,
        wy: 400.0,
        scale: 1.0,
        images: &images,
        annotations: &annotations,
        folders: &folders,
        selected_image_ids: &empty,
        selected_annotation_ids: &empty,
        selected_folder_id: None,
        noeuds: None,
    };

    let cands = collect_candidates(&input);
    assert_eq!(cands[0].id, "M1");
    assert_eq!(cands[0].kind, PickKind::MembraneEdge);
}

#[test]
fn test_nested_membranes_smallest_wins() {
    let inner = membrane("PETITE", 400.0, 300.0, 200.0, 200.0, None);
    let outer = membrane("GRANDE", 0.0, 0.0, 1000.0, 800.0, None);
    let annotations = [outer, inner];
    let empty_images = [];
    let empty_folders = [];
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 500.0,
        wy: 400.0,
        scale: 1.0,
        images: &empty_images,
        annotations: &annotations,
        folders: &empty_folders,
        selected_image_ids: &empty,
        selected_annotation_ids: &empty,
        selected_folder_id: None,
        noeuds: None,
    };

    let cands = collect_candidates(&input);
    assert_eq!(cands[0].id, "PETITE");
}

/// **PICK-2 — un texte posé dans une membrane se prend en plein dedans**, et le re-clic
/// descend jusqu'à la membrane : le texte n'est plus le fond de la pile (registre de Tauri,
/// n° 8 : « en plein dans un texte posé dans une membrane → le texte » ; « recliquer sans
/// bouger passe à l'élément suivant dessous »).
#[test]
fn test_pick_2_un_texte_dans_une_membrane_se_prend_et_le_reclic_descend() {
    let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
    let t = text_ann("T1", 400.0, 380.0, 200.0, 60.0);
    let annotations = [memb, t];
    let empty_images = [];
    let empty_folders = [];
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 500.0,
        wy: 400.0,
        scale: 1.0,
        images: &empty_images,
        annotations: &annotations,
        folders: &empty_folders,
        selected_image_ids: &empty,
        selected_annotation_ids: &empty,
        selected_folder_id: None,
        noeuds: None,
    };

    let cands = collect_candidates(&input);
    assert_eq!(cands[0].id, "T1");
    assert_eq!(cands[1].id, "M1");

    // Un premier clic, puis un re-clic lent — au-delà de la fenêtre du double-clic — au même
    // endroit : au relâchement, la membrane.
    let (premier, cycle) = pick_at_down(&cands, None, 50.0, 50.0, 1_000, PickOptions::default());
    assert_eq!(premier.map(|c| c.id), Some("T1".to_string()));
    let (_, cycle) = advance_on_release(&cands, cycle.as_ref(), 1_060);
    let lent = 1_060 + pick_consts::DBLCLICK_MS + 100;
    let (_, cycle) = pick_at_down(
        &cands,
        cycle.as_ref(),
        50.0,
        50.0,
        lent,
        PickOptions::default(),
    );
    let (dessous, _) = advance_on_release(&cands, cycle.as_ref(), lent + 60);
    assert_eq!(
        dessous.map(|c| c.id),
        Some("M1".to_string()),
        "le re-clic lent sur un texte descend a ce qu'il y a dessous"
    );
}

#[test]
fn test_collect_candidates_indexed_matches_naive() {
    use glucose_core::quadtree::SpatialHash;

    let mut hash = SpatialHash::new(500.0);
    let mut images = Vec::new();
    let mut annotations = Vec::new();

    for i in 0..20 {
        let id = format!("I{}", i);
        let img_obj = img(&id, i as f64 * 1000.0, 0.0, 200.0, 200.0, false);
        hash.insert_image(&img_obj);
        images.push(img_obj);
    }

    let t = text_ann("T1", 5000.0, 0.0, 100.0, 40.0);
    hash.insert_annotation(&t);
    annotations.push(t);

    let empty_folders = [];
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 5020.0,
        wy: 10.0,
        scale: 1.0,
        images: &images,
        annotations: &annotations,
        folders: &empty_folders,
        selected_image_ids: &empty,
        selected_annotation_ids: &empty,
        selected_folder_id: None,
        noeuds: None,
    };

    let naive = collect_candidates(&input);
    let indexed = collect_candidates_indexed(&input, &hash);

    assert_eq!(naive.len(), indexed.len());
    assert_eq!(naive[0].id, indexed[0].id);
    // La carte est peinte au-dessus de l'image qu'elle recouvre : c'est elle qu'on prend
    // (PICK-2 ; l'ordre de Tauri rendait l'image).
    assert_eq!(naive[0].id, "T1");
    assert_eq!(naive[1].id, "I5");

    // Clic loin dans le vide
    let empty_input = PickInput {
        wx: 99999.0,
        wy: 99999.0,
        scale: 1.0,
        images: &images,
        annotations: &annotations,
        folders: &empty_folders,
        selected_image_ids: &empty,
        selected_annotation_ids: &empty,
        selected_folder_id: None,
        noeuds: None,
    };
    let empty_indexed = collect_candidates_indexed(&empty_input, &hash);
    assert!(empty_indexed.is_empty());
}
