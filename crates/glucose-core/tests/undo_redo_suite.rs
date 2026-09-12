//! Suite de tests UNDO-1 : Invariants Undo/Redo infinis pour TOUT.
//! Port complet de src/store/undo-redo.test.ts
//!
//! A. NAVIGATION TRANSPARENTE — zoomer, paner, changer de board, dossiers ne créent jamais d'undo ni ne détruisent le redo
//! B. ÉDITIONS RÉVERSIBLES — chaque mutation de contenu s'annule et se refait à l'identique (mutate -> undo -> redo)
//! C. ROBUSTESSE INTER-NAVIGATION — la caméra n'est jamais téléportée, on reste dans le dossier courant, le redo survit à la nav
//! D. TRANSACTIONS D'INTERACTION — drag, resize, tracé = 1 seule entrée undo (begin_live_edit / end_live_edit)

use glucose_core::store::{DomainPatch, Store};
use glucose_core::types::{
    Annotation, BoardImage, CanvasFolder, Domain, FolderTreeNode, Preset, StoryboardPanel, Viewport,
};

fn mk_text(id: &str, text: &str) -> Annotation {
    Annotation::Text {
        id: id.to_string(),
        x: 0.0,
        y: 0.0,
        width: Some(100.0),
        height: Some(40.0),
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

fn mk_sticky(id: &str, text: &str) -> Annotation {
    Annotation::Sticky {
        id: id.to_string(),
        x: 0.0,
        y: 0.0,
        width: Some(160.0),
        height: Some(120.0),
        text: text.to_string(),
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

fn mk_arrow(id: &str, source_id: Option<&str>, target_id: Option<&str>) -> Annotation {
    Annotation::Arrow {
        id: id.to_string(),
        x: 0.0,
        y: 0.0,
        x2: 100.0,
        y2: 0.0,
        text: None,
        font_size: None,
        color: None,
        arrow_type: None,
        arrow_bidirectional: false,
        predicate: None,
        stroke_width: None,
        waypoints: Vec::new(),
        source_id: source_id.map(|s| s.to_string()),
        target_id: target_id.map(|t| t.to_string()),
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
}

fn mk_membrane(id: &str) -> Annotation {
    Annotation::Membrane {
        id: id.to_string(),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 160.0,
        color: Some("#60a5fa".into()),
        text: Some("Membrane".into()),
        mode: glucose_core::types::MembraneMode::Classic,
        curtains: Vec::new(),
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

fn mk_image(id: &str) -> BoardImage {
    BoardImage::new(id, 0.0, 0.0, 100.0, 100.0)
}

fn mk_folder(id: &str, name: &str) -> CanvasFolder {
    CanvasFolder::new(id, name, "")
}

// ════════════════════════════════════════════════════════════════════
// A. NAVIGATION TRANSPARENTE — rien de tout ça ne touche l'undo/redo.
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_set_viewport_pan_zoom_ne_cree_aucune_entree_undo() {
    let mut store = Store::new("test");
    assert_eq!(store.undo_depth(), 0);

    store.set_viewport(
        "main",
        Viewport {
            x: 50.0,
            y: 60.0,
            scale: 2.0,
        },
    );
    store.set_viewport(
        "main",
        Viewport {
            x: 70.0,
            y: 80.0,
            scale: 3.0,
        },
    );
    assert_eq!(store.undo_depth(), 0);
}

#[test]
fn test_regression_20_pans_entre_action_et_ctrl_z_n_enterrent_pas_undo() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("img1"));
    for i in 0..20 {
        store.set_viewport(
            "main",
            Viewport {
                x: i as f64,
                y: i as f64,
                scale: 1.0,
            },
        );
    }
    assert_eq!(store.undo_depth(), 1); // 1 seul pas, pas 21
    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().images.len(), 0); // annulée du premier coup
}

#[test]
fn test_naviguer_apres_un_undo_ne_detruit_pas_le_redo() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("img1"));
    store.undo();
    assert_eq!(store.redo_depth(), 1); // redo armé

    store.set_viewport(
        "main",
        Viewport {
            x: 999.0,
            y: 999.0,
            scale: 4.0,
        },
    );
    store.set_active_board_id("main");
    assert_eq!(store.redo_depth(), 1); // toujours là malgré la nav
    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().images.len(), 1);
}

#[test]
fn test_enter_folder_exit_folder_ne_creent_aucune_entree_undo() {
    let mut store = Store::new("test");
    let fid = store.create_folder_with_content(
        "main",
        mk_folder("f1", "Dossier"),
        vec![mk_text("t1", "hi")],
    );
    let base = store.undo_depth(); // 1 = la création du dossier

    store.try_enter_folder(&fid).expect("le dossier existe");
    store.exit_folder();
    store.try_enter_folder(&fid).expect("le dossier existe");
    store.exit_to_root();
    assert_eq!(store.undo_depth(), base);
}

#[test]
fn test_set_active_board_id_ne_cree_aucune_entree_undo() {
    let mut store = Store::new("test");
    let other = store.add_board("Autre");
    let base = store.undo_depth();

    store.set_active_board_id("main");
    store.set_active_board_id(&other);
    assert_eq!(store.undo_depth(), base);
}

#[test]
fn test_expand_folder_ne_cree_aucune_entree_undo() {
    let mut store = Store::new("test");
    let fid = store.create_folder_with_content("main", mk_folder("f1", "Dossier"), Vec::new());
    let base = store.undo_depth();

    let level = FolderTreeNode {
        folder: mk_folder("f_sub", "Sub"),
        annotations: vec![mk_text("t1", "a"), mk_text("t2", "b")],
        images: Vec::new(),
        children: Vec::new(),
    };
    store.expand_folder("main", &fid, level);
    assert_eq!(store.undo_depth(), base); // navigation/scan, pas édition

    let folder = store
        .active_board()
        .unwrap()
        .folders
        .iter()
        .find(|f| f.id == fid)
        .unwrap();
    let child = store
        .project
        .boards
        .iter()
        .find(|b| b.id == folder.child_board_id)
        .unwrap();
    assert_eq!(child.annotations.len(), 2);
}

// ════════════════════════════════════════════════════════════════════
// B. ÉDITIONS RÉVERSIBLES — un round-trip mutate -> undo -> redo
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_add_image_roundtrip() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("img1"));
    assert_eq!(store.active_board().unwrap().images.len(), 1);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().images.len(), 0);

    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().images.len(), 1);
}

#[test]
fn test_update_image_roundtrip() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("img1"));

    store.update_image("main", "img1", |img| {
        img.x = 500.0;
        img.y = 300.0;
    });
    assert_eq!(store.active_board().unwrap().images[0].x, 500.0);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().images[0].x, 0.0);

    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().images[0].x, 500.0);
}

#[test]
fn test_remove_images_roundtrip() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("img1"));

    store.remove_images("main", &["img1"]);
    assert_eq!(store.active_board().unwrap().images.len(), 0);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().images.len(), 1);

    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().images.len(), 0);
}

#[test]
fn test_move_selected_roundtrip() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("img1"));
    store.set_selected_image_ids(vec!["img1".into()]);

    store.move_selected("main", 10.0, 20.0);
    assert_eq!(store.active_board().unwrap().images[0].y, 20.0);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().images[0].y, 0.0);

    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().images[0].y, 20.0);
}

#[test]
fn test_duplicate_selected_roundtrip() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("img1"));
    store.set_selected_image_ids(vec!["img1".into()]);

    store.duplicate_selected("main");
    assert_eq!(store.active_board().unwrap().images.len(), 2);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().images.len(), 1);

    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().images.len(), 2);
}

#[test]
fn test_annotation_text_roundtrip() {
    let mut store = Store::new("test");
    store.add_annotation("main", mk_text("t1", "hello"));
    assert_eq!(store.active_board().unwrap().annotations.len(), 1);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);

    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().annotations.len(), 1);
}

#[test]
fn test_update_annotation_roundtrip() {
    let mut store = Store::new("test");
    store.add_annotation("main", mk_text("t1", "avant"));

    store.update_annotation("main", "t1", |ann| {
        if let Annotation::Text { text, .. } = ann {
            *text = "après".into();
        }
    });
    if let Annotation::Text { text, .. } = &store.active_board().unwrap().annotations[0] {
        assert_eq!(text, "après");
    }

    assert!(store.undo());
    if let Annotation::Text { text, .. } = &store.active_board().unwrap().annotations[0] {
        assert_eq!(text, "avant");
    }

    assert!(store.redo());
    if let Annotation::Text { text, .. } = &store.active_board().unwrap().annotations[0] {
        assert_eq!(text, "après");
    }
}

#[test]
fn test_remove_annotations_roundtrip() {
    let mut store = Store::new("test");
    store.add_annotation("main", mk_sticky("s1", "note"));

    store.remove_annotations("main", &["s1"]);
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().annotations.len(), 1);

    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);
}

#[test]
fn test_arrow_cascade_delete_roundtrip() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("src1"));
    store.add_annotation("main", mk_arrow("arr1", Some("src1"), None));

    store.remove_images("main", &["src1"]);
    assert_eq!(store.active_board().unwrap().images.len(), 0);
    assert_eq!(store.active_board().unwrap().annotations.len(), 0); // flèche orpheline retirée

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().images.len(), 1);
    assert_eq!(store.active_board().unwrap().annotations.len(), 1); // restaurée

    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().images.len(), 0);
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);
}

#[test]
fn test_panel_roundtrip() {
    let mut store = Store::new("test");
    let panel = StoryboardPanel::new("p1", 0, 0.0, 0.0, 320.0, 180.0);
    store.add_panel("main", panel);
    assert_eq!(store.active_board().unwrap().panels.len(), 1);

    store.update_panel("main", "p1", "description x");
    assert_eq!(
        store.active_board().unwrap().panels[0].description,
        "description x"
    );

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().panels[0].description, "");

    assert!(store.redo());
    assert_eq!(
        store.active_board().unwrap().panels[0].description,
        "description x"
    );
}

#[test]
fn test_rename_board_roundtrip() {
    let mut store = Store::new("test");
    store.rename_board("main", "Renommé");
    assert_eq!(
        store
            .project
            .boards
            .iter()
            .find(|b| b.id == "main")
            .unwrap()
            .name,
        "Renommé"
    );

    assert!(store.undo());
    assert_eq!(
        store
            .project
            .boards
            .iter()
            .find(|b| b.id == "main")
            .unwrap()
            .name,
        "Canvas Principal"
    );

    assert!(store.redo());
    assert_eq!(
        store
            .project
            .boards
            .iter()
            .find(|b| b.id == "main")
            .unwrap()
            .name,
        "Renommé"
    );
}

#[test]
fn test_add_and_remove_board_roundtrip() {
    let mut store = Store::new("test");
    let other = store.add_board("Nouveau");
    assert_eq!(store.project.boards.len(), 2);

    assert!(store.undo());
    assert_eq!(store.project.boards.len(), 1);

    assert!(store.redo());
    assert_eq!(store.project.boards.len(), 2);

    store.set_active_board_id("main");
    store
        .try_remove_board(&other)
        .expect("tableau existant, et pas le dernier du projet");
    assert_eq!(store.project.boards.len(), 1);

    assert!(store.undo());
    assert_eq!(store.project.boards.len(), 2);

    assert!(store.redo());
    assert_eq!(store.project.boards.len(), 1);
}

#[test]
fn test_folder_with_content_roundtrip() {
    let mut store = Store::new("test");
    let fid = store.create_folder_with_content(
        "main",
        mk_folder("f1", "Dossier"),
        vec![mk_text("t1", "a"), mk_sticky("s1", "b")],
    );
    assert_eq!(store.active_board().unwrap().folders.len(), 1);
    assert_eq!(store.project.boards.len(), 2);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().folders.len(), 0);
    assert_eq!(store.project.boards.len(), 1);

    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().folders.len(), 1);
    assert_eq!(store.project.boards.len(), 2);

    store.remove_folders("main", &[&fid]);
    assert_eq!(store.active_board().unwrap().folders.len(), 0);
    assert_eq!(store.project.boards.len(), 1);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().folders.len(), 1);
    assert_eq!(store.project.boards.len(), 2);
}

#[test]
fn test_preset_and_domain_roundtrip() {
    let mut store = Store::new("test");
    let preset = Preset {
        id: "p1".into(),
        name: "Mien".into(),
        description: "".into(),
        slots: Vec::new(),
        is_builtin: false,
        created_at: 0,
    };
    store.add_preset(preset);
    assert_eq!(store.project.presets.len(), 1);

    store.update_preset("p1", "Modifié");
    assert_eq!(store.project.presets[0].name, "Modifié");

    assert!(store.undo());
    assert_eq!(store.project.presets[0].name, "Mien");

    assert!(store.redo());
    assert_eq!(store.project.presets[0].name, "Modifié");

    let dom = Domain {
        id: "d1".into(),
        name: "Maths".into(),
        color: "#60a5fa".into(),
        icon: "🔬".into(),
        created_at: 0,
    };
    store.try_add_domain(dom).expect("catalogue vide");
    assert_eq!(store.project.domains.len(), 1);

    store
        .try_update_domain("d1", DomainPatch::new().with_name("Algèbre"))
        .expect("d1 est au catalogue");
    assert_eq!(store.project.domains[0].name, "Algèbre");

    assert!(store.undo());
    assert_eq!(store.project.domains[0].name, "Maths");

    assert!(store.redo());
    assert_eq!(store.project.domains[0].name, "Algèbre");
}

#[test]
fn test_membrane_add_and_delete_roundtrip() {
    let mut store = Store::new("test");
    store.add_annotation("main", mk_membrane("m1"));
    assert_eq!(store.active_board().unwrap().annotations.len(), 1);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);

    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().annotations.len(), 1);

    store.set_selected_annotation_ids(vec!["m1".into()]);
    store.delete_selected("main");
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().annotations.len(), 1);
}

// ════════════════════════════════════════════════════════════════════
// C. ROBUSTESSE INTER-NAVIGATION
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_undo_ne_teleporte_pas_la_camera() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("img1"));
    store.set_viewport(
        "main",
        Viewport {
            x: 1234.0,
            y: 5678.0,
            scale: 3.0,
        },
    );

    assert!(store.undo());
    let vp = store.active_board().unwrap().viewport;
    assert_eq!(vp.x, 1234.0);
    assert_eq!(vp.y, 5678.0);
    assert_eq!(vp.scale, 3.0);
    assert_eq!(store.active_board().unwrap().images.len(), 0);
}

#[test]
fn test_redo_ne_teleporte_pas_la_camera() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("img1"));
    store.undo();
    store.set_viewport(
        "main",
        Viewport {
            x: 42.0,
            y: 42.0,
            scale: 2.0,
        },
    );

    assert!(store.redo());
    let vp = store.active_board().unwrap().viewport;
    assert_eq!(vp.x, 42.0);
    assert_eq!(vp.scale, 2.0);
    assert_eq!(store.active_board().unwrap().images.len(), 1);
}

#[test]
fn test_annuler_edition_dans_un_dossier_garde_dans_le_dossier() {
    let mut store = Store::new("test");
    let fid = store.create_folder_with_content("main", mk_folder("f1", "Dossier"), Vec::new());
    store.try_enter_folder(&fid).expect("le dossier existe");

    let child_id = store.project.active_board_id.clone();
    store.add_annotation(&child_id, mk_text("t1", "inside"));
    assert_eq!(store.active_board().unwrap().annotations.len(), 1);

    assert!(store.undo());
    assert_eq!(store.project.active_board_id, child_id); // toujours dans le dossier
    assert_eq!(store.folder_stack.len(), 1);
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);
}

#[test]
fn test_annuler_creation_dossier_ou_on_est_retombe_racine() {
    let mut store = Store::new("test");
    store.add_annotation("main", mk_text("t0", "root"));
    let fid = store.create_folder_with_content("main", mk_folder("f1", "Dossier"), Vec::new());
    store.try_enter_folder(&fid).expect("le dossier existe");

    let child_id = store.project.active_board_id.clone();
    assert!(store.undo()); // annule create_folder_with_content
    assert!(!store.project.boards.iter().any(|b| b.id == child_id));
    assert_eq!(store.project.active_board_id, "main");
    assert_eq!(store.folder_stack.len(), 0);
}

#[test]
fn test_undo_redo_renvoient_false_si_rien_a_faire() {
    let mut store = Store::new("test");
    assert!(!store.undo());
    assert!(!store.redo());

    store.add_image("main", mk_image("i1"));
    assert!(store.undo());
    assert!(!store.undo()); // pile vidée
    assert!(store.redo());
    assert!(!store.redo()); // plus rien
}

#[test]
fn test_nouvelle_edition_invalide_redo_mais_pas_navigation() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("i1"));
    store.undo();
    assert_eq!(store.redo_depth(), 1);

    store.add_annotation("main", mk_text("t1", "a")); // vraie édition -> vide le redo
    assert_eq!(store.redo_depth(), 0);
    assert!(!store.redo());
}

#[test]
fn test_sequence_longue_mixte_reste_coherente() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("i1")); // E1
    store.set_viewport(
        "main",
        Viewport {
            x: 10.0,
            y: 10.0,
            scale: 1.0,
        },
    );
    store.add_annotation("main", mk_text("t1", "txt")); // E2
    store.set_active_board_id("main");
    store.add_annotation("main", mk_sticky("s1", "stk")); // E3

    assert_eq!(store.undo_depth(), 3);
    assert!(store.undo()); // défait E3
    assert_eq!(store.active_board().unwrap().annotations.len(), 1);

    assert!(store.undo()); // défait E2
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);

    assert!(store.undo()); // défait E1
    assert_eq!(store.active_board().unwrap().images.len(), 0);

    assert!(!store.undo());
    assert!(store.redo()); // refait E1
    assert_eq!(store.active_board().unwrap().images.len(), 1);
}

// ════════════════════════════════════════════════════════════════════
// D. TRANSACTIONS D'INTERACTION (Live Edit)
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_30_moves_entre_begin_end_live_edit_egal_1_entree_undo() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("i1"));
    store.set_selected_image_ids(vec!["i1".into()]);
    let base = store.undo_depth();

    store.begin_live_edit();
    for _ in 0..30 {
        store.move_selected("main", 1.0, 0.0);
    }
    store.end_live_edit();

    assert_eq!(store.undo_depth(), base + 1); // +1, pas +30
    assert_eq!(store.active_board().unwrap().images[0].x, 30.0);

    assert!(store.undo());
    assert_eq!(store.active_board().unwrap().images[0].x, 0.0);

    assert!(store.redo());
    assert_eq!(store.active_board().unwrap().images[0].x, 30.0);
}

#[test]
fn test_begin_live_edit_est_idempotent() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("i1"));
    let base = store.undo_depth();

    store.begin_live_edit();
    store.begin_live_edit(); // idempotent
    store.add_annotation("main", mk_text("t1", "x"));
    store.update_annotation("main", "t1", |a| {
        if let Annotation::Text { text, .. } = a {
            *text = "y".into();
        }
    });
    store.end_live_edit();

    assert_eq!(store.undo_depth(), base + 1);
}

#[test]
fn test_end_live_edit_sans_begin_est_un_no_op() {
    let mut store = Store::new("test");
    store.end_live_edit();
    assert!(!store.in_live_edit());
    assert_eq!(store.undo_depth(), 0);
}

#[test]
fn test_demarrer_un_drag_vide_le_redo_en_attente() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("i1"));
    store.undo();
    assert_eq!(store.redo_depth(), 1);

    store.begin_live_edit();
    store.move_selected("main", 1.0, 1.0);
    store.end_live_edit();
    assert_eq!(store.redo_depth(), 0);
}

#[test]
fn test_regression_bug_texte_creer_frapper_commit_egal_1_entree() {
    let mut store = Store::new("test");
    let base = store.undo_depth();

    store.begin_live_edit();
    store.add_annotation("main", mk_text("t1", ""));
    for i in 0..6 {
        store.update_annotation("main", "t1", |a| {
            if let Annotation::Text { height, .. } = a {
                *height = Some(40.0 + (i as f64) * 16.0);
            }
        });
    }
    store.update_annotation("main", "t1", |a| {
        if let Annotation::Text { text, .. } = a {
            *text = "Bonjour le monde".into();
        }
    });
    store.end_live_edit();

    assert_eq!(store.undo_depth(), base + 1);
    assert!(store.undo()); // UN SEUL Ctrl+Z
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);
}

#[test]
fn test_sync_annotation_size_ne_cree_aucune_entree_undo() {
    let mut store = Store::new("test");
    store.add_annotation("main", mk_text("t1", ""));
    let base = store.undo_depth();

    store.sync_annotation_size("main", "t1", 250.0, 180.0);
    store.sync_annotation_size("main", "t1", 260.0, 200.0);
    assert_eq!(store.undo_depth(), base); // 0 entrée ajoutée
    assert_eq!(store.redo_depth(), 0);

    if let Annotation::Text { width, .. } = &store.active_board().unwrap().annotations[0] {
        assert_eq!(*width, Some(260.0));
    }
}
