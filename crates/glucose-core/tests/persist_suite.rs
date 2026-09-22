//! INVARIANT PERSIST-1 — l'aller-retour est l'identité.
//!
//! Un projet non trivial sérialisé puis relu doit être **strictement égal** à l'original
//! (standard § 7.5). C'est le seul test qui prouve que R-01 est réparé : tout le reste
//! (conteneur, sommes de contrôle, atomicité) ne sert à rien si le document revient différent.

mod persist_fixture;

use glucose_core::persist;
use glucose_core::types::{
    Annotation, ArrowPredicate, AssetStore, Board, CurtainEditable, CurtainVisibility,
    FolderSortMode, MembraneMode, Project, StickyOperator, Viewport,
};
use persist_fixture::rich_project;

const SAVED_AT: i64 = 1_770_000_123_456;

/// **Un document au schéma v1 se lit encore, et ses images sont entières** (RECADRAGE-1).
///
/// C'est la première migration chaînée du format, et elle se prouve sur de vrais octets : ce
/// témoin a été écrit par la build du commit `9ffb376`, la dernière à produire du v1, depuis
/// la même fixture — avant qu'un recadrage n'y soit posé. Il n'est pas régénérable par cette
/// build, et c'est tout son intérêt : un test qui écrirait le v1 lui-même ne prouverait que sa
/// propre idée du v1.
///
/// Ce qu'on vérifie : le manifeste dit v1, tout ce qui existait en v1 revient à l'identique,
/// et chaque image porte le recadrage neutre — l'état qu'elle avait avant que le geste
/// n'existe.
#[test]
fn test_un_document_v1_se_lit_encore_et_ses_images_sont_entieres() {
    let temoin = include_bytes!("persist_fixture/riche-v1.glucose");
    let relu = persist::decode(temoin).expect("un document v1 doit se lire");
    assert_eq!(
        relu.manifest.document_version, 1,
        "le temoin est bien un v1"
    );

    // La fixture d'aujourd'hui, ramenée à ce qu'elle était en v1 : sans recadrage.
    let mut attendu = rich_project();
    for board in &mut attendu.boards {
        for img in &mut board.images {
            img.crop = glucose_core::types::Recadrage::ENTIER;
        }
    }
    assert_eq!(
        relu.project, attendu,
        "tout ce qui existait en v1 doit revenir a l'identique, et les images entieres"
    );

    // Et réécrit par cette build, il monte au schéma du jour sans rien perdre.
    let reecrit = persist::encode(&relu.project, &AssetStore::new(), SAVED_AT);
    let remonte = persist::decode(&reecrit).expect("le document migre se relit");
    assert_eq!(remonte.manifest.document_version, persist::DOCUMENT_VERSION);
    assert_eq!(remonte.project, relu.project);
}

/// **La version se lit dans le manifeste, jamais dans les octets** : relire un v2 en croyant
/// que c'est un v1 échoue, au lieu de rendre un document à moitié juste.
#[test]
fn test_lire_un_v2_comme_un_v1_echoue_au_lieu_de_deviner() {
    let nu = persist::encode_document(&rich_project());
    assert!(
        persist::decode_document_v(&nu, 1).is_err(),
        "les octets du recadrage ne peuvent pas passer pour la suite du document"
    );
    assert!(persist::decode_document_v(&nu, 2).is_ok());
}

#[test]
fn test_round_trip_of_a_rich_project_is_the_identity() {
    let project = rich_project();
    let file = persist::encode(&project, &AssetStore::new(), SAVED_AT);
    let reloaded =
        persist::decode(&file).expect("un fichier que l'on vient d'écrire doit se relire");

    assert_eq!(
        reloaded.project, project,
        "le document relu diffère de l'original"
    );
    assert_eq!(reloaded.manifest.project_name, project.name);
    assert_eq!(reloaded.manifest.saved_at, SAVED_AT);
    assert_eq!(
        reloaded.manifest.document_version,
        persist::DOCUMENT_VERSION
    );
}

#[test]
fn test_document_alone_round_trips_without_the_container() {
    let project = rich_project();
    let payload = persist::encode_document(&project);
    assert_eq!(
        persist::decode_document(&payload).expect("le document nu doit se relire"),
        project
    );
}

#[test]
fn test_every_board_member_survives_the_round_trip() {
    let project = rich_project();
    let file = persist::encode(&project, &AssetStore::new(), SAVED_AT);
    let board = &persist::decode(&file).expect("relecture").project.boards[0];

    assert_eq!(board.images.len(), 2);
    assert_eq!(board.annotations.len(), 4);
    assert_eq!(board.folders.len(), 2);
    assert_eq!(board.panels.len(), 1);
    assert_eq!(board.zones.len(), 1);
    assert_eq!(board.bookmarks.len(), 2);
    assert!(matches!(board.annotations[0], Annotation::Text { .. }));
    assert!(matches!(board.annotations[1], Annotation::Sticky { .. }));
    assert!(matches!(board.annotations[2], Annotation::Arrow { .. }));
    assert!(matches!(board.annotations[3], Annotation::Membrane { .. }));
}

#[test]
fn test_negative_zero_and_extreme_floats_survive_bit_for_bit() {
    // `-0.0 == 0.0` : seule une comparaison de bits attrape une écriture décimale fautive.
    let mut project = Project::new("flottants");
    let mut board = Board::new("b", "B");
    board.viewport = Viewport {
        x: -0.0,
        y: f64::MIN_POSITIVE,
        scale: 1.0 / 3.0,
    };
    project.boards = vec![board];
    project.active_board_id = "b".into();

    let reloaded =
        persist::decode_document(&persist::encode_document(&project)).expect("relecture");
    let vp = reloaded.boards[0].viewport;
    assert_eq!(vp.x.to_bits(), (-0.0f64).to_bits(), "-0.0 est devenu +0.0");
    assert_eq!(vp.y, f64::MIN_POSITIVE);
    assert_eq!(vp.scale, 1.0 / 3.0);
}

#[test]
fn test_encoding_is_deterministic_even_with_unordered_bookmarks() {
    // Les signets vivent dans une `HashMap`, dont l'ordre d'itération varie d'une exécution à
    // l'autre. Deux encodages du même projet doivent malgré tout donner les mêmes octets.
    let project = rich_project();
    let first = persist::encode(&project, &AssetStore::new(), SAVED_AT);
    let second = persist::encode(&project, &AssetStore::new(), SAVED_AT);
    assert_eq!(first, second, "l'encodage n'est pas reproductible");
}

#[test]
fn test_empty_project_round_trips() {
    let project = Project::new("vide");
    let file = persist::encode(&project, &AssetStore::new(), 0);
    assert_eq!(persist::decode(&file).expect("relecture").project, project);
}

#[test]
fn test_every_arrow_predicate_round_trips() {
    for predicate in [
        ArrowPredicate::EstPrecurseur,
        ArrowPredicate::Contredit,
        ArrowPredicate::HeriteDe,
        ArrowPredicate::Inspire,
        ArrowPredicate::DependDe,
        ArrowPredicate::Illustre,
    ] {
        let project = project_with(arrow_with_predicate(predicate));
        let reloaded =
            persist::decode_document(&persist::encode_document(&project)).expect("relecture");
        assert_eq!(reloaded, project, "prédicat {}", predicate.as_str());
    }
}

#[test]
fn test_every_sticky_operator_round_trips() {
    for operator in [
        StickyOperator::And,
        StickyOperator::Or,
        StickyOperator::But,
        StickyOperator::Because,
    ] {
        let project = project_with(sticky_with_operator(operator));
        let reloaded =
            persist::decode_document(&persist::encode_document(&project)).expect("relecture");
        assert_eq!(reloaded, project, "opérateur {operator:?}");
    }
}

#[test]
fn test_every_membrane_mode_and_curtain_flag_round_trips() {
    for mode in [
        MembraneMode::Classic,
        MembraneMode::Minimized,
        MembraneMode::Stretched,
    ] {
        for visibility in [CurtainVisibility::Private, CurtainVisibility::Shared] {
            for editable in [CurtainEditable::Owner, CurtainEditable::Everyone] {
                let project = project_with(membrane_with(mode, visibility, editable));
                let reloaded = persist::decode_document(&persist::encode_document(&project))
                    .expect("relecture");
                assert_eq!(
                    reloaded, project,
                    "{mode:?} / {visibility:?} / {editable:?}"
                );
            }
        }
    }
}

#[test]
fn test_every_folder_sort_mode_round_trips() {
    use glucose_core::types::{CanvasFolder, FolderMirrorSource};

    for sort_by in [
        FolderSortMode::NameAsc,
        FolderSortMode::NameDesc,
        FolderSortMode::Type,
        FolderSortMode::SizeDesc,
        FolderSortMode::SizeAsc,
        FolderSortMode::ModifiedDesc,
        FolderSortMode::ModifiedAsc,
    ] {
        let mut folder = CanvasFolder::new("f", "F", "child");
        folder.mirror_source = Some(FolderMirrorSource {
            root_path: "/racine".into(),
            mode: "snapshot".into(),
            last_scanned_at: 0,
            pattern: None,
            recursive: false,
            sort_by: Some(sort_by),
            pending_scan: false,
        });

        let mut project = Project::new("tri");
        let mut board = Board::new("b", "B");
        board.folders = vec![folder];
        project.boards = vec![board];
        project.active_board_id = "b".into();

        let reloaded =
            persist::decode_document(&persist::encode_document(&project)).expect("relecture");
        assert_eq!(reloaded, project, "tri {sort_by:?}");
    }
}

// ── Fabriques locales ───────────────────────────────────────────────────────

fn project_with(annotation: Annotation) -> Project {
    let mut project = Project::new("variantes");
    let mut board = Board::new("b", "B");
    board.annotations = vec![annotation];
    project.boards = vec![board];
    project.active_board_id = "b".into();
    project
}

fn arrow_with_predicate(predicate: ArrowPredicate) -> Annotation {
    Annotation::Arrow {
        id: "a".into(),
        x: 0.0,
        y: 0.0,
        x2: 1.0,
        y2: 1.0,
        text: None,
        font_size: None,
        color: None,
        arrow_type: None,
        arrow_bidirectional: false,
        predicate: Some(predicate),
        stroke_width: None,
        waypoints: Vec::new(),
        source_id: None,
        target_id: None,
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

fn sticky_with_operator(operator: StickyOperator) -> Annotation {
    Annotation::Sticky {
        id: "s".into(),
        x: 0.0,
        y: 0.0,
        width: None,
        height: None,
        text: String::new(),
        font_size: None,
        color: None,
        bg_color: None,
        cursor_pos: None,
        operator: Some(operator),
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

fn membrane_with(
    mode: MembraneMode,
    visibility: CurtainVisibility,
    editable: CurtainEditable,
) -> Annotation {
    use glucose_core::types::MembraneCurtain;

    Annotation::Membrane {
        id: "m".into(),
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
        color: None,
        text: None,
        mode,
        curtains: vec![MembraneCurtain {
            id: "c".into(),
            owner_id: "u".into(),
            owner_name: "U".into(),
            owner_color: "#fff".into(),
            visibility,
            editable,
            collapsed_ratio: None,
            expanded_ratio: None,
            board_id: None,
            notes: Vec::new(),
            created_at: 0,
        }],
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}
