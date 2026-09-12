//! `Store` — invariants ID-1 (identifiants O(1), R-22) et R-35 (les erreurs ne sont plus
//! silencieuses), plus le suivi des flèches lors d'un déplacement (R-12).

use glucose_core::error::CoreError;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Board, BoardImage, CanvasFolder, Domain, Project, Viewport};

fn mk_text(id: &str, x: f64, y: f64) -> Annotation {
    Annotation::Text {
        id: id.to_string(),
        x,
        y,
        width: Some(100.0),
        height: Some(50.0),
        text: "T".into(),
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

fn mk_arrow(id: &str, source: &str, target: &str, x: f64, y: f64, x2: f64, y2: f64) -> Annotation {
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
        source_id: Some(source.to_string()),
        target_id: Some(target.to_string()),
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

// ── R-12 — les flèches suivent leurs extrémités ─────────────────────────────

#[test]
fn test_arrow_follows_source_and_target_on_move() {
    let mut store = Store::new("Test");
    store.add_annotation("main", mk_text("card1", 10.0, 20.0));
    store.add_annotation("main", mk_text("card2", 200.0, 200.0));
    store.add_annotation(
        "main",
        mk_arrow("arr1", "card1", "card2", 10.0, 20.0, 200.0, 200.0),
    );

    store.select_annotation("card2".into(), false);
    store.move_selected("main", 50.0, 30.0);

    let board = store.active_board().expect("board actif");
    let arr = board
        .annotations
        .iter()
        .find(|a| a.id() == "arr1")
        .expect("la flèche existe");
    match arr {
        Annotation::Arrow { x, y, x2, y2, .. } => {
            assert_eq!((*x, *y), (10.0, 20.0));
            assert_eq!((*x2, *y2), (250.0, 230.0));
        }
        _ => panic!("Expected arrow"),
    }
}

// ── R-22 / ID-1 — génération d'identifiants ─────────────────────────────────

#[test]
fn test_generate_id_ne_collisionne_jamais_sur_un_projet_neuf() {
    let mut store = Store::new("P");
    let mut seen = std::collections::HashSet::new();
    for _ in 0..5_000 {
        let id = store.generate_id("img");
        assert!(seen.insert(id.clone()), "identifiant dupliqué : {}", id);
        assert!(!store.id_exists(&id));
    }
}

/// ID-1 — un projet venu du disque contient des identifiants que ce `Store` n'a pas produits.
/// `load_project` doit recaler `next_id` au-dessus du plus grand suffixe rencontré, sinon la
/// première création entre en collision avec un élément existant.
#[test]
fn test_load_project_recale_next_id_au_dessus_des_ids_existants() {
    let mut project = Project::new("Chargé");
    let mut board = Board::new("board-900", "Chargé");
    board
        .images
        .push(BoardImage::new("img-4242", 0.0, 0.0, 10.0, 10.0));
    board.annotations.push(mk_text("ann-77", 0.0, 0.0));
    board
        .folders
        .push(CanvasFolder::new("folder-88", "F", "board-901"));
    project.boards.push(board);
    project.active_board_id = "board-900".into();

    let mut store = Store::new("P");
    store.load_project(project);

    for _ in 0..50 {
        let id = store.generate_id("img");
        assert!(!store.id_exists(&id), "collision après chargement : {}", id);
    }
    let next = store.generate_id("ann");
    assert!(!store.id_exists(&next));
    assert!(
        store.next_id > 4242,
        "next_id doit dominer le plus grand suffixe chargé"
    );
}

/// R-22 — la génération est O(1). L'implémentation d'origine appelait `id_exists` (scan
/// complet du projet) pour CHAQUE identifiant : ce test prenait alors plusieurs minutes.
#[test]
fn test_generation_didentifiants_reste_lineaire() {
    let mut store = Store::new("P");
    for i in 0..10_000 {
        let id = store.generate_id("img");
        let mut img = BoardImage::new(&id, 0.0, 0.0, 10.0, 10.0);
        img.x = i as f64;
        if let Some(b) = store.project.boards.first_mut() {
            b.images.push(img);
        }
    }

    let start = std::time::Instant::now();
    for _ in 0..10_000 {
        let id = store.generate_id("img");
        assert!(!id.is_empty());
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "R-22 : 10 000 identifiants sur un projet de 10 000 nœuds ont pris {:?}",
        elapsed
    );
}

// ── R-35 — les erreurs ne sont plus silencieuses ────────────────────────────

#[test]
fn test_try_set_active_board_id_signale_un_tableau_inconnu() {
    let mut store = Store::new("P");
    assert_eq!(
        store.try_set_active_board_id("fantome"),
        Err(CoreError::BoardNotFound("fantome".into()))
    );
    assert_eq!(store.project.active_board_id, "main");
    assert!(store.try_set_active_board_id("main").is_ok());
}

#[test]
fn test_try_enter_folder_signale_un_dossier_inconnu() {
    let mut store = Store::new("P");
    assert_eq!(
        store.try_enter_folder("fantome"),
        Err(CoreError::FolderNotFound("fantome".into()))
    );
    assert!(store.folder_stack.is_empty());

    store.create_folder("main", CanvasFolder::new("F", "F", ""));
    assert!(store.try_enter_folder("F").is_ok());
    assert_eq!(store.folder_stack.len(), 1);
}

#[test]
fn test_try_remove_board_refuse_le_dernier_tableau_en_disant_quoi_faire() {
    let mut store = Store::new("P");
    let err = store
        .try_remove_board("main")
        .expect_err("le dernier tableau est protégé");
    match err {
        CoreError::InvalidOperation(msg) => {
            assert!(
                msg.contains("dernier tableau"),
                "message peu utile : {}",
                msg
            );
            assert!(
                msg.contains("créer un autre"),
                "le message doit dire quoi faire : {}",
                msg
            );
        }
        other => panic!("variante inattendue : {:?}", other),
    }
    assert_eq!(store.project.boards.len(), 1);

    let other = store.add_board("Deux");
    assert!(store.try_remove_board(&other).is_ok());
    assert_eq!(
        store.try_remove_board("fantome"),
        Err(CoreError::BoardNotFound("fantome".into()))
    );
}

#[test]
fn test_try_mirror_signale_une_source_introuvable() {
    let mut store = Store::new("P");
    assert_eq!(
        store.try_mirror_annotation("main", "fantome", 0.0, 0.0),
        Err(CoreError::AnnotationNotFound("fantome".into()))
    );
    assert_eq!(
        store.try_mirror_annotation("board-fantome", "x", 0.0, 0.0),
        Err(CoreError::BoardNotFound("board-fantome".into()))
    );
    assert_eq!(
        store.try_mirror_folder("main", "fantome", 0.0, 0.0),
        Err(CoreError::FolderNotFound("fantome".into()))
    );

    store.add_annotation("main", mk_text("O", 0.0, 0.0));
    let mid = store
        .try_mirror_annotation("main", "O", 50.0, 50.0)
        .expect("O existe");
    let board = store.active_board().expect("board actif");
    assert!(board.annotations.iter().any(|a| a.id() == mid));
}

#[test]
fn test_try_assign_domain_to_node_signale_un_noeud_introuvable() {
    let mut store = Store::new("P");
    store
        .try_add_domain(Domain {
            id: "D1".into(),
            name: "Science".into(),
            color: "#60a5fa".into(),
            icon: "SCI".into(),
            created_at: 0,
        })
        .expect("catalogue vide");

    assert_eq!(
        store.try_assign_domain_to_node("main", "fantome", "D1", 1.0),
        Err(CoreError::NodeNotFound("fantome".into()))
    );
    assert_eq!(
        store.try_assign_domain_to_node("board-fantome", "n", "D1", 1.0),
        Err(CoreError::BoardNotFound("board-fantome".into()))
    );

    store.add_annotation("main", mk_text("T", 0.0, 0.0));
    // Un domaine absent du catalogue est refusé : c'est ce refus qui empêche la création
    // d'une référence orpheline dès l'assignation (DOM-1).
    assert_eq!(
        store.try_assign_domain_to_node("main", "T", "inconnu", 0.5),
        Err(CoreError::DomainNotFound("inconnu".into()))
    );

    assert!(store
        .try_assign_domain_to_node("main", "T", "D1", 0.5)
        .is_ok());
    let assigned = store.node_domains("main", "T").expect("T existe");
    assert_eq!(assigned.len(), 1);
    assert_eq!(assigned[0].domain_id, "D1");
}

/// Compatibilité : les enveloppes historiques restent muettes et ne paniquent pas.
/// C'est exactement le comportement que `try_*` remplace — il est testé pour que la
/// migration des appelants puisse se faire sans surprise.
#[test]
#[allow(deprecated)]
fn test_les_enveloppes_historiques_restent_silencieuses() {
    let mut store = Store::new("P");
    store.enter_folder("fantome");
    assert!(store.folder_stack.is_empty());

    assert!(!store.remove_board("main"));
    assert_eq!(store.project.boards.len(), 1);

    assert_eq!(store.mirror_annotation("main", "fantome", 0.0, 0.0), None);
    assert_eq!(store.mirror_folder("main", "fantome", 0.0, 0.0), None);

    store.set_active_board_id("fantome");
    assert_eq!(store.project.active_board_id, "main");
}

// ── Fiche 09 § 1 — bornes numériques du modèle : l'échelle ───────────────────

/// « Échelle de zoom bornée entre 0.005 (×200 dézoomé) et 50.0 (×50 zoomé). »
///
/// Les chiffres eux-mêmes, puis la propriété : **tout** viewport qui entre dans le store est
/// dans ce domaine, quel que soit le chemin — pose directe, zoom, chargement d'un fichier.
#[test]
fn test_l_echelle_du_modele_est_bornee_a_0_005_et_50() {
    assert_eq!(Viewport::MIN_SCALE, 0.005, "fiche 09 § 1 : ×200 dézoomé");
    assert_eq!(Viewport::MAX_SCALE, 50.0, "fiche 09 § 1 : ×50 zoomé");

    let mut store = Store::new("P");
    let scale = |store: &Store| store.active_board().expect("main").viewport.scale;

    store.set_viewport("main", Viewport { x: 0.0, y: 0.0, scale: 0.0001 });
    assert_eq!(scale(&store), 0.005, "en dessous, rabattu sur la borne basse");
    store.set_viewport("main", Viewport { x: 0.0, y: 0.0, scale: 1_000.0 });
    assert_eq!(scale(&store), 50.0, "au-dessus, rabattu sur la borne haute");

    store.set_viewport("main", Viewport::default());
    store.zoom(1e9, 0.0, 0.0, Viewport::SCALE_RANGE);
    assert_eq!(scale(&store), 50.0, "un zoom ne franchit pas la borne haute");
    store.zoom(1e-9, 0.0, 0.0, Viewport::SCALE_RANGE);
    assert_eq!(scale(&store), 0.005, "ni la basse");

    // L'appelant peut demander plus étroit — jamais plus large.
    store.set_viewport("main", Viewport::default());
    store.zoom(1e9, 0.0, 0.0, (0.02, 20.0));
    assert_eq!(scale(&store), 20.0, "la borne du geste s'ajoute à celle du modèle");
    store.zoom(1e9, 0.0, 0.0, (0.0, 1e9));
    assert_eq!(scale(&store), 50.0, "et ne peut pas l'élargir");
}

/// Un fichier abîmé peut porter une caméra à `NaN` ou à l'infini. `clamp` laisse passer
/// `NaN`, et une caméra à `NaN` rend chaque pixel invisible sans faire tomber le programme —
/// un écran vide qui ne dit pas pourquoi. Le chargement ramène donc la caméra dans le modèle.
#[test]
fn test_charger_un_projet_ramene_la_camera_dans_le_modele() {
    let mut project = Project::new("Abîmé");
    project.boards[0].viewport = Viewport { x: f64::NAN, y: f64::INFINITY, scale: f64::NAN };
    let mut extra = Board::new("b2", "Trop zoomé");
    extra.viewport = Viewport { x: 10.0, y: 20.0, scale: 999.0 };
    project.boards.push(extra);
    let mut zero = Board::new("b3", "Échelle nulle");
    zero.viewport = Viewport { x: 0.0, y: 0.0, scale: 0.0 };
    project.boards.push(zero);

    let mut store = Store::new("P");
    store.load_project(project);

    let main = store.project.boards[0].viewport;
    assert_eq!((main.x, main.y, main.scale), (0.0, 0.0, 1.0), "non-nombres → caméra neutre");
    let b2 = store.project.boards[1].viewport;
    assert_eq!((b2.x, b2.y, b2.scale), (10.0, 20.0, 50.0), "hors borne → rabattu, le reste intact");
    // Zéro est un nombre : il est rabattu sur la borne basse, comme dans la version de
    // référence — c'est `NaN`, et lui seul, qui redevient 1.
    assert_eq!(store.project.boards[2].viewport.scale, 0.005);

    // Et un zoom sur une caméra qui aurait échappé à la normalisation ne produit pas de NaN.
    let mut vp = Viewport { x: 0.0, y: 0.0, scale: 0.0 };
    vp.zoom_at(2.0, 100.0, 100.0, Viewport::SCALE_RANGE);
    assert!(vp.scale.is_finite() && vp.x.is_finite() && vp.y.is_finite(), "{vp:?}");
}

// ── Fiche 08 § 1.3 — la duplication ──────────────────────────────────────────

/// « Clone immédiatement les images sélectionnées avec un décalage » — de **20 px** en X et
/// en Y : c'est ce que fait Glucose Tauri (`OFFSET = 20` dans `duplicateSelected`), et la
/// fiche disait 24. La cible fait foi. Le clone est sélectionné à la place de l'original.
#[test]
fn test_duplicate_offsets_the_clone_by_twenty_pixels_and_selects_it() {
    let mut store = Store::new("P");
    store.add_image("main", BoardImage::new("img-1", 100.0, 50.0, 200.0, 150.0));
    store.add_annotation("main", mk_text("t-1", -30.0, 70.0));
    store.set_selected_image_ids(vec!["img-1".into()]);
    store.set_selected_annotation_ids(vec!["t-1".into()]);

    store.duplicate_selected("main");

    let board = store.active_board().expect("main");
    let clone = board.images.iter().find(|i| i.id != "img-1").expect("le clone de l'image");
    assert_eq!((clone.x, clone.y), (120.0, 70.0), "+20 px en X et en Y");
    assert_eq!((clone.width, clone.height), (200.0, 150.0), "même taille");
    let text_clone = board.annotations.iter().find(|a| a.id() != "t-1").expect("le clone du texte");
    assert_eq!((text_clone.x(), text_clone.y()), (-10.0, 90.0));
    assert_eq!(store.selected_image_ids, vec![clone.id.clone()], "le clone prend la sélection");
    assert_eq!(store.selected_annotation_ids, vec![text_clone.id().to_string()]);
}

// ── content_bounds : où est le contenu d'un tableau ─────────────────────────

/// La boîte englobante couvre les trois familles d'objets — images, dossiers, annotations —
/// et pas seulement celle qu'on a en tête en écrivant le code.
#[test]
fn test_les_bornes_du_contenu_couvrent_les_trois_familles() {
    let mut store = Store::new("P");
    let board = store.project.active_board_id.clone();

    store.add_image(&board, BoardImage::new("i", 100.0, 200.0, 50.0, 40.0));
    let mut dossier = CanvasFolder::new("f", "D", String::new());
    dossier.x = -300.0;
    dossier.y = 0.0;
    dossier.width = 100.0;
    dossier.height = 100.0;
    store.create_folder(&board, dossier);
    store.add_annotation(&board, Annotation::membrane("m", 0.0, 400.0, 200.0, 150.0));

    let b = store.content_bounds(&board).expect("des bornes");
    assert_eq!(b.left, -300.0, "le dossier tire la borne gauche");
    assert_eq!(b.top, 0.0, "le dossier tire la borne haute");
    assert_eq!(b.left + b.width, 200.0, "l'image et la membrane tirent la droite");
    assert_eq!(b.top + b.height, 550.0, "la membrane tire le bas");
}

/// **Une flèche compte par ses deux extrémités.** Son point d'ancrage ne dit rien de l'endroit
/// qu'elle occupe — et une flèche qui remonte vers la gauche a sa pointe avant son origine.
#[test]
fn test_une_fleche_compte_par_ses_deux_extremites() {
    let mut store = Store::new("P");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::arrow("a", 500.0, 500.0, -100.0, -200.0));

    let b = store.content_bounds(&board).expect("des bornes");
    assert_eq!((b.left, b.top), (-100.0, -200.0));
    assert_eq!((b.left + b.width, b.top + b.height), (500.0, 500.0));
}

/// Un tableau vide n'a pas de bornes, et un tableau inconnu non plus. Rendre un rectangle nul
/// serait pire : un appelant cadrerait sur un point.
#[test]
fn test_un_tableau_vide_n_a_pas_de_bornes() {
    let store = Store::new("P");
    let board = store.project.active_board_id.clone();
    assert!(store.content_bounds(&board).is_none());
    assert!(store.content_bounds("jamais-vu").is_none());
}

/// Une carte sans taille explicite compte pour son point, pas pour une taille inventée.
#[test]
fn test_une_carte_sans_taille_compte_pour_son_point() {
    let mut store = Store::new("P");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text("t", 40.0, 60.0, "sans dimension"));

    let b = store.content_bounds(&board).expect("des bornes");
    assert_eq!((b.left, b.top, b.width, b.height), (40.0, 60.0, 0.0, 0.0));
}
