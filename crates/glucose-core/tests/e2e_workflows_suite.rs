//! Scénarios E2E : Workflows utilisateur complets simulés via Store — 100% Rust Standard Library (std).
//! Port complet de src/e2e-workflows.test.ts

use glucose_core::store::Store;
use glucose_core::types::{
    Annotation, BoardImage, CanvasFolder, Domain, Preset, PresetSlot,
    StickyOperator, TemporalAnchor,
};

fn mk_text(id: &str, x: f64, y: f64, text: &str) -> Annotation {
    Annotation::Text {
        id: id.to_string(),
        x,
        y,
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

fn mk_sticky(id: &str, x: f64, y: f64, text: &str, op: Option<StickyOperator>) -> Annotation {
    Annotation::Sticky {
        id: id.to_string(),
        x,
        y,
        width: Some(160.0),
        height: Some(120.0),
        text: text.to_string(),
        font_size: None,
        color: None,
        bg_color: None,
        cursor_pos: None,
        operator: op,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

fn mk_arrow(id: &str, source_id: Option<&str>, target_id: Option<&str>, x: f64, y: f64, x2: f64, y2: f64) -> Annotation {
    Annotation::Arrow {
        id: id.to_string(),
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

fn mk_image(id: &str, x: f64, y: f64) -> BoardImage {
    BoardImage::new(id, x, y, 100.0, 100.0)
}

fn mk_folder(id: &str, name: &str, x: f64, y: f64, width: f64, height: f64) -> CanvasFolder {
    let mut f = CanvasFolder::new(id, name, "");
    f.x = x;
    f.y = y;
    f.width = width;
    f.height = height;
    f
}

// ─────────── Workflow 1 — Folder lifecycle complet ───────────────

#[test]
fn test_workflow_folder_cycle_complet() {
    let mut store = Store::new("test");

    // 1. Crée du contenu dans le board parent
    store.add_image("main", mk_image("img1", 50.0, 50.0));
    store.add_annotation("main", mk_text("T1", 80.0, 80.0, "Inside"));

    // 2. Crée folder par-dessus (capture spatiale)
    store.create_folder("main", mk_folder("F", "F", 0.0, 0.0, 200.0, 200.0));

    // Vérifie que le contenu a été capturé
    assert_eq!(store.active_board().unwrap().images.len(), 0);
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);
    assert_eq!(store.active_board().unwrap().folders.len(), 1);

    // 3. Entre dans le folder
    store.try_enter_folder("F").expect("le dossier existe");
    assert_eq!(store.active_board().unwrap().images.len(), 1);
    assert_eq!(store.active_board().unwrap().annotations.len(), 1);

    // 4. Édite : ajoute un nouveau sticky
    let child_id = store.project.active_board_id.clone();
    store.add_annotation(&child_id, mk_sticky("S1", 10.0, 10.0, "Added in folder", None));
    assert_eq!(store.active_board().unwrap().annotations.len(), 2);

    // 5. Sort
    store.exit_folder();
    assert_eq!(store.project.active_board_id, "main");

    // 6. Undo défait le sticky qu'on vient d'ajouter
    assert!(store.undo());
    store.add_image("main", mk_image("new_img", 0.0, 0.0));
    assert_eq!(store.active_board().unwrap().images.len(), 1);
}

#[test]
fn test_workflow_creer_folder_vide_entrer_sortir_supprimer() {
    let mut store = Store::new("test");
    store.create_folder("main", mk_folder("F1", "F", 0.0, 0.0, 100.0, 100.0));
    store.try_enter_folder("F1").expect("le dossier existe");
    store.exit_folder();
    store.remove_folders("main", &["F1"]);
    assert_eq!(store.project.boards.len(), 1);
}

#[test]
fn test_workflow_nested_folders() {
    let mut store = Store::new("test");
    store.create_folder("main", mk_folder("F", "F", 0.0, 0.0, 100.0, 100.0));
    store.try_enter_folder("F").expect("le dossier existe");
    let child_f = store.project.active_board_id.clone();

    store.create_folder(&child_f, mk_folder("G", "G", 0.0, 0.0, 50.0, 50.0));
    store.try_enter_folder("G").expect("le dossier existe");
    assert_eq!(store.folder_stack.len(), 2);

    store.exit_folder();
    assert_eq!(store.folder_stack.len(), 1);
    assert_eq!(store.project.active_board_id, child_f);

    store.exit_folder();
    assert_eq!(store.project.active_board_id, "main");
}

#[test]
fn test_workflow_renommer_folder_ne_casse_pas_la_navigation() {
    let mut store = Store::new("test");
    store.create_folder("main", mk_folder("F", "Old", 0.0, 0.0, 100.0, 100.0));
    store.update_folder("main", "F", |f| f.name = "New".into());
    store.try_enter_folder("F").expect("le dossier existe");
    store.exit_folder();
    assert_eq!(store.project.active_board_id, "main");
}

#[test]
fn test_workflow_mirror_folder() {
    let mut store = Store::new("test");
    store.create_folder("main", mk_folder("F", "F", 0.0, 0.0, 100.0, 100.0));
    store.try_enter_folder("F").expect("le dossier existe");
    let f_child = store.project.active_board_id.clone();
    store.add_annotation(&f_child, mk_text("T-in-F", 0.0, 0.0, "hello"));
    store.exit_folder();

    let mid = store.try_mirror_folder("main", "F", 300.0, 300.0).expect("le dossier F existe");
    store.try_enter_folder(&mid).expect("le miroir de dossier est navigable");

    // Le miroir partage le child_board_id : le contenu doit être présent
    let child = store.active_board().unwrap();
    assert!(child.annotations.iter().any(|a| a.id() == "T-in-F"));
}

// ─────────── Workflow 2 — Flèches lifecycle ───────────────────

#[test]
fn test_workflow_fleches_deplacement_source_la_fleche_suit() {
    let mut store = Store::new("test");
    store.add_annotation("main", mk_text("T1", 0.0, 0.0, "node 1"));
    store.add_annotation("main", mk_text("T2", 200.0, 0.0, "node 2"));
    store.add_annotation("main", mk_arrow("A1", Some("T1"), Some("T2"), 0.0, 0.0, 200.0, 0.0));

    store.update_annotation("main", "T1", |a| {
        if let Annotation::Text { x, y, .. } = a {
            *x = 100.0;
            *y = 100.0;
        }
    });

    let updated = store
        .active_board()
        .unwrap()
        .annotations
        .iter()
        .find(|a| a.id() == "A1")
        .unwrap();

    assert_eq!(updated.x(), 100.0);
    assert_eq!(updated.y(), 100.0);
}

#[test]
fn test_workflow_supprimer_la_source_supprime_la_fleche() {
    let mut store = Store::new("test");
    store.add_annotation("main", mk_text("T", 0.0, 0.0, "text"));
    store.add_annotation("main", mk_arrow("A", Some("T"), None, 0.0, 0.0, 100.0, 100.0));

    store.remove_annotations("main", &["T"]);
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);
}

#[test]
fn test_workflow_fleche_portail_vers_autre_board() {
    let mut store = Store::new("test");
    let other = store.add_board("Other");
    store.set_active_board_id("main");

    let mut arrow = mk_arrow("PA", None, None, 0.0, 0.0, 100.0, 0.0);
    if let Annotation::Arrow { ref mut target_board_id, .. } = arrow {
        *target_board_id = Some(other.clone());
    }
    store.add_annotation("main", arrow);

    let ann = &store.active_board().unwrap().annotations[0];
    if let Annotation::Arrow { target_board_id, .. } = ann {
        assert_eq!(target_board_id.as_deref(), Some(other.as_str()));
    }

    store.try_remove_board(&other).expect("tableau existant, et pas le dernier du projet");
    let ann_after = &store.active_board().unwrap().annotations[0];
    if let Annotation::Arrow { target_board_id, .. } = ann_after {
        assert_eq!(*target_board_id, None);
    }
}

// ─────────── Workflow 3 — Undo/Redo intensif ───────────

#[test]
fn test_workflow_100_mutations_100_undos_100_redos() {
    let mut store = Store::new("test");
    for i in 0..100 {
        store.add_image("main", mk_image(&format!("img-{}", i), i as f64, i as f64));
    }
    for _ in 0..100 {
        store.undo();
    }
    for _ in 0..100 {
        store.redo();
    }
    assert!(!store.active_board().unwrap().images.is_empty());
}

#[test]
fn test_workflow_undo_apres_create_folder_restaure_le_contenu_pre_capture() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("IMG", 50.0, 50.0));
    store.create_folder("main", mk_folder("F", "F", 0.0, 0.0, 200.0, 200.0));

    // Après création folder, image dans child board
    assert_eq!(store.active_board().unwrap().images.len(), 0);

    store.undo();
    // Après undo, image revient dans main
    assert_eq!(store.active_board().unwrap().images.len(), 1);
}

// ─────────── Workflow 4 — Duplication et Miroir ────────

#[test]
fn test_workflow_duplicate_selection_mixte() {
    let mut store = Store::new("test");
    store.add_image("main", mk_image("I", 50.0, 50.0));
    store.add_annotation("main", mk_text("T", 0.0, 0.0, "hello"));
    store.add_annotation("main", mk_sticky("S", 100.0, 100.0, "note", None));

    store.set_selected_image_ids(vec!["I".into()]);
    store.set_selected_annotation_ids(vec!["T".into(), "S".into()]);
    store.duplicate_selected("main");

    let b = store.active_board().unwrap();
    assert_eq!(b.images.len(), 2);
    assert_eq!(b.annotations.len(), 4);
}

#[test]
fn test_workflow_mirror_annotation() {
    let mut store = Store::new("test");
    store.add_annotation("main", mk_text("O", 0.0, 0.0, "hello"));
    let mid = store.try_mirror_annotation("main", "O", 50.0, 50.0).expect("l'annotation O existe");

    let b = store.active_board().unwrap();
    let mirror = b.annotations.iter().find(|a| a.id() == mid).unwrap();
    match mirror {
        Annotation::Text { mirror_of, .. } => {
            assert_eq!(mirror_of.as_deref(), Some("O"));
        }
        _ => panic!("Expected text annotation"),
    }
}

// ─────────── Workflow 5 — Domaines + Temporel ────────

#[test]
fn test_workflow_domains_et_temporal() {
    let mut store = Store::new("test");
    let d = Domain {
        id: "D1".into(),
        name: "Sci".into(),
        color: "#60a5fa".into(),
        icon: "SCI".into(),
        created_at: 0,
    };
    store.try_add_domain(d).expect("catalogue vide");

    let mut t1 = mk_text("T1", 0.0, 0.0, "Newton");
    if let Annotation::Text { ref mut temporal_anchor, .. } = t1 {
        *temporal_anchor = Some(TemporalAnchor { start: 1643, end: 1727, label: None });
    }
    let mut t2 = mk_text("T2", 0.0, 0.0, "Einstein");
    if let Annotation::Text { ref mut temporal_anchor, .. } = t2 {
        *temporal_anchor = Some(TemporalAnchor { start: 1879, end: 1955, label: None });
    }
    store.add_annotation("main", t1);
    store.add_annotation("main", t2);

    store.try_assign_domain_to_node("main", "T1", "D1", 0.7).expect("T1 existe");
    store.try_assign_domain_to_node("main", "T2", "D1", 0.5).expect("T2 existe");

    store.set_temporal_filter(Some(TemporalAnchor { start: 1800, end: 2000, label: None }));
    assert!(store.temporal_filter.is_some());

    store.set_temporal_filter(None);
    assert!(store.temporal_filter.is_none());
}

// ─────────── Workflow 6 — Presets ────────────────────────

#[test]
fn test_workflow_presets() {
    let mut store = Store::new("test");
    store.add_preset(Preset {
        id: "P1".into(),
        name: "Char".into(),
        description: "".into(),
        slots: vec![PresetSlot {
            id: "X".into(),
            name: "X".into(),
            color: "#000".into(),
            description: "".into(),
            order: 0,
        }],
        is_builtin: false,
        created_at: 0,
    });

    store.apply_preset_to_board("main", Some("P1"));
    assert_eq!(store.active_board().unwrap().zones.len(), 1);

    store.apply_preset_to_board("main", None);
    assert_eq!(store.active_board().unwrap().zones.len(), 0);
}

// ─────────── Workflow 7 — Chemin chaud "Le bug folder" ──

#[test]
fn test_workflow_folder_avec_markdown_et_latex() {
    let mut store = Store::new("test");
    store.add_annotation(
        "main",
        mk_text("MD", 0.0, 0.0, "# Titre\n\n- liste\n\n$E = mc^2$\n\n**bold**"),
    );

    store.create_folder("main", mk_folder("F", "F", -50.0, -50.0, 300.0, 300.0));
    assert_eq!(store.active_board().unwrap().annotations.len(), 0);
    assert_eq!(store.active_board().unwrap().folders.len(), 1);

    store.try_enter_folder("F").expect("le dossier existe");
    let child = store.active_board().unwrap();
    assert!(child.annotations.iter().any(|a| a.id() == "MD"));

    store.exit_folder();
    assert_eq!(store.project.active_board_id, "main");
}

#[test]
fn test_workflow_sticky_avec_operateur() {
    let mut store = Store::new("test");
    store.add_annotation(
        "main",
        mk_sticky("OP", 50.0, 50.0, "Formule", Some(StickyOperator::Because)),
    );
    store.create_folder("main", mk_folder("F", "F", 0.0, 0.0, 200.0, 200.0));
    store.try_enter_folder("F").expect("le dossier existe");
    store.exit_folder();
    assert_eq!(store.project.active_board_id, "main");
}

// ─────────── Workflow 8 — Projet réaliste 100+ items ──

#[test]
fn test_workflow_gros_volume_100_images_50_annotations() {
    let mut store = Store::new("test");
    for i in 0..100 {
        store.add_image(
            "main",
            mk_image(
                &format!("img-{}", i),
                ((i % 10) * 50) as f64,
                ((i / 10) * 50) as f64,
            ),
        );
    }
    for i in 0..50 {
        store.add_annotation(
            "main",
            mk_text(
                &format!("txt-{}", i),
                ((i % 5) * 30) as f64,
                (i * 10) as f64,
                "note",
            ),
        );
    }

    store.create_folder("main", mk_folder("F", "F", 0.0, 0.0, 200.0, 200.0));
    assert!(store.project.boards.len() >= 2);
}

#[test]
fn test_r13_no_duplicate_id_on_repeated_clones() {
    let mut store = Store::new("test");
    store.add_annotation("main", mk_text("orig", 0.0, 0.0, "Original"));

    // Dupliquer une première fois
    store.select_annotation("orig".to_string(), false);
    store.duplicate_selected("main");
    let dup1_id = store.selected_annotation_ids[0].clone();
    assert_ne!(dup1_id, "orig");

    // Resélectionner l'original et dupliquer à nouveau -> doit avoir un id DIFFÉRENT de dup1
    store.select_annotation("orig".to_string(), false);
    store.duplicate_selected("main");
    let dup2_id = store.selected_annotation_ids[0].clone();
    assert_ne!(dup2_id, "orig");
    assert_ne!(dup2_id, dup1_id, "R-13: duplicate IDs must never collide!");

    let board = store.active_board().unwrap();
    let mut ids = std::collections::HashSet::new();
    for ann in &board.annotations {
        assert!(ids.insert(ann.id().to_string()), "Duplicate annotation ID found: {}", ann.id());
    }
}

#[test]
fn test_r14_no_board_id_collision_after_deletion() {
    let mut store = Store::new("test");
    let b2 = store.add_board("Board 2");
    let b3 = store.add_board("Board 3");
    assert_eq!(store.project.boards.len(), 3);

    // Supprimer Board 2
    store.try_remove_board(&b2).expect("tableau existant, et pas le dernier du projet");
    assert_eq!(store.project.boards.len(), 2);

    // Créer un nouveau board -> ne doit jamais entrer en collision avec b3
    let b_new = store.add_board("New Board");
    assert_ne!(b_new, b3, "R-14: newly created board ID must never collide with existing boards!");
    assert_ne!(b_new, b2);

    let mut board_ids = std::collections::HashSet::new();
    for b in &store.project.boards {
        assert!(board_ids.insert(b.id.clone()), "Duplicate board ID found: {}", b.id);
    }
}
