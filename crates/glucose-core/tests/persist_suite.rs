//! INVARIANT PERSIST-1 — l'aller-retour est l'identité.
//!
//! Un projet non trivial sérialisé puis relu doit être **strictement égal** à l'original
//! (standard § 7.5). C'est le seul test qui prouve que R-01 est réparé : tout le reste
//! (conteneur, sommes de contrôle, atomicité) ne sert à rien si le document revient différent.

mod persist_fixture;

use glucose_core::persist;
use glucose_core::store::journal::Slot;
use glucose_core::types::{
    Annotation, ArrowPredicate, AssetStore, Board, CurtainEditable, CurtainVisibility,
    FolderSortMode, Grille, MembraneMode, Plan, Project, StickyOperator, Storyboard,
    StoryboardPanel, Viewport, STORYBOARD_D_AVANT,
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

    // La fixture d'aujourd'hui, ramenée à ce qu'elle était en v1 : sans recadrage, sans
    // grille de storyboard (schéma 3), et des cases sans durée, plan ni champs (schéma 4).
    let mut attendu = sans_grille(sans_la_lecture(d_avant_les_sequences(rich_project())));
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

/// **Un document au schéma v2 se lit encore, et ses tableaux n'ont pas de grille de
/// storyboard** (STORYBOARD-2) : ses cases vont au storyboard d'avant, à la grille de Tauri
/// (SEQUENCE-1, fiche 68).
///
/// Le témoin a été écrit par la build de `58ff58c`, la dernière à produire du v2, depuis la
/// même fixture — avant qu'une grille n'y soit posée ; comme celui du v1, il n'est pas
/// régénérable par cette build.
#[test]
fn test_un_document_v2_se_lit_encore_sans_grille_de_storyboard() {
    let temoin = include_bytes!("persist_fixture/riche-v2.glucose");
    let relu = persist::decode(temoin).expect("un document v2 doit se lire");
    assert_eq!(
        relu.manifest.document_version, 2,
        "le témoin est bien un v2"
    );

    let attendu = sans_grille(sans_la_lecture(d_avant_les_sequences(rich_project())));
    assert_eq!(
        relu.project, attendu,
        "tout ce qui existait en v2 revient à l'identique, sans grille"
    );

    let reecrit = persist::encode(&relu.project, &AssetStore::new(), SAVED_AT);
    let remonte = persist::decode(&reecrit).expect("le document migré se relit");
    assert_eq!(remonte.manifest.document_version, persist::DOCUMENT_VERSION);
    assert_eq!(remonte.project, relu.project);
}

/// **La fixture ramenée à ce qu'un schéma d'avant le 4 pouvait dire** : ses cases sans durée,
/// sans plan, sans champs (fiche 67).
fn sans_la_lecture(mut p: Project) -> Project {
    for board in &mut p.boards {
        for case in &mut board.panels {
            (case.duree_ms, case.plan, case.textes) = (None, None, Vec::new());
        }
    }
    p
}

/// **La fixture ramenée à ce qu'un schéma d'avant le 5 pouvait dire** (fiche 68) : un seul
/// storyboard par tableau — le premier —, sous le nom d'avant, et seulement ses cases.
fn d_avant_les_sequences(mut p: Project) -> Project {
    for board in &mut p.boards {
        let Some(premier) = board.storyboards.first().map(|s| s.id.clone()) else {
            continue;
        };
        board.storyboards.truncate(1);
        board.storyboards[0].id = STORYBOARD_D_AVANT.into();
        board.panels.retain(|c| c.storyboard == premier);
        for case in &mut board.panels {
            case.storyboard = STORYBOARD_D_AVANT.into();
        }
    }
    p
}

/// **La fixture ramenée à ce qu'un schéma d'avant le 3 pouvait dire** : aucune grille — ses
/// cases vont au storyboard d'avant, né à la grille de Tauri (SEQUENCE-1).
fn sans_grille(mut p: Project) -> Project {
    for board in &mut p.boards {
        for s in &mut board.storyboards {
            *s = Storyboard::neuf(STORYBOARD_D_AVANT, Grille::default());
        }
    }
    p
}

/// **Un document au schéma v3 se lit encore** : son storyboard garde sa grille et prend les
/// réglages de naissance — deux secondes par case, Dialogue, Action, Notes —, ses cases n'ont
/// ni durée à elles, ni plan, ni texte (fiche 67).
///
/// Le témoin a été écrit par la build de `ea8a690`, la dernière à produire du v3, depuis la
/// même fixture — avant que les cases ne portent leur lecture.
#[test]
fn test_un_document_v3_se_lit_encore_aux_reglages_de_naissance() {
    let temoin = include_bytes!("persist_fixture/riche-v3.glucose");
    let relu = persist::decode(temoin).expect("un document v3 doit se lire");
    assert_eq!(
        relu.manifest.document_version, 3,
        "le témoin est bien un v3"
    );

    let mut attendu = sans_la_lecture(d_avant_les_sequences(rich_project()));
    for board in &mut attendu.boards {
        for s in &mut board.storyboards {
            *s = Storyboard::neuf(STORYBOARD_D_AVANT, s.grille);
        }
    }
    assert_eq!(
        relu.project, attendu,
        "tout ce qui existait en v3 revient à l'identique, aux réglages de naissance"
    );
    let aujourd_hui = rich_project();
    let board = &aujourd_hui.boards[0];
    assert!(
        board
            .panels
            .iter()
            .any(StoryboardPanel::porte_quelque_chose)
            && board
                .storyboards
                .iter()
                .all(|s| *s != Storyboard::neuf(s.id.as_str(), s.grille)),
        "la fixture d'aujourd'hui porte une lecture : sans elle, l'épreuve ne verrait rien"
    );

    let reecrit = persist::encode(&relu.project, &AssetStore::new(), SAVED_AT);
    let remonte = persist::decode(&reecrit).expect("le document migré se relit");
    assert_eq!(remonte.manifest.document_version, persist::DOCUMENT_VERSION);
    assert_eq!(remonte.project, relu.project);
}

/// **Un document au schéma v4 se lit encore** (fiche 68) : il n'a vécu qu'une après-midi,
/// mais une build l'a écrit chez lui. Son unique storyboard garde tout — grille, durée, champs
/// — sous le nom d'avant, et ses cases aussi.
///
/// Le témoin a été écrit par la build de `5bb82ec`, la seule à produire du v4, depuis la même
/// fixture — avant qu'un tableau ne porte deux storyboards.
#[test]
fn test_un_document_v4_se_lit_encore_sous_le_storyboard_d_avant() {
    let temoin = include_bytes!("persist_fixture/riche-v4.glucose");
    let relu = persist::decode(temoin).expect("un document v4 doit se lire");
    assert_eq!(
        relu.manifest.document_version, 4,
        "le témoin est bien un v4"
    );
    assert_eq!(
        relu.project,
        d_avant_les_sequences(rich_project()),
        "tout ce qui existait en v4 revient à l'identique"
    );
    assert!(
        rich_project().boards[0].storyboards.len() > 1,
        "la fixture d'aujourd'hui porte deux storyboards : sans eux, l'épreuve ne verrait rien"
    );

    let reecrit = persist::encode(&relu.project, &AssetStore::new(), SAVED_AT);
    let remonte = persist::decode(&reecrit).expect("le document migré se relit");
    assert_eq!(remonte.manifest.document_version, persist::DOCUMENT_VERSION);
    assert_eq!(remonte.project, relu.project);
}

/// Ce qu'un geste du témoin porte : ses cases, et ses storyboards au journal.
fn cases_et_storyboards(
    geste: &persist::histoire::Geste,
) -> (Vec<StoryboardPanel>, Vec<Slot<Storyboard>>) {
    use glucose_core::store::journal::Edit;
    let (mut cases, mut storyboards) = (Vec::new(), Vec::new());
    for edit in &geste.transaction.edits {
        match edit {
            Edit::Panel { slot, .. } => cases.extend(slot.after.as_deref().cloned()),
            Edit::Storyboard { slot, .. } => storyboards.push(slot.clone()),
            autre => panic!("le témoin ne porte que des cases et un storyboard : {autre:?}"),
        }
    }
    (cases, storyboards)
}

/// **Un geste de l'histoire écrit au schéma v3 se relit** : chaque entrée garde son schéma, et
/// celles d'un document d'avant le 4 portent des cases et un storyboard au v3 — le storyboard,
/// comme la case de rang 0 de la liste d'aujourd'hui (fiche 68).
///
/// Le témoin a été écrit par la build de `ea8a690` : un storyboard posé (sa grille, huit cases
/// insérées), puis la deuxième case décrite — tous les gestes du storyboard qu'un v3 écrivait.
#[test]
fn test_un_geste_v3_du_storyboard_se_relit() {
    let temoin = include_bytes!("persist_fixture/storyboard-v3.geste");
    let geste = persist::histoire::lire_geste(temoin).expect("un geste v3 doit se lire");
    let (cases, storyboards) = cases_et_storyboards(&geste);
    assert_eq!(cases.len(), 9, "huit cases posées, puis une décrite");
    assert!(cases.iter().all(|c| c.duree_ms.is_none()
        && c.plan.is_none()
        && c.textes.is_empty()
        && c.storyboard == STORYBOARD_D_AVANT));
    assert_eq!(cases[8].description, "une case décrite");
    assert_eq!(
        storyboards,
        vec![Slot::inserted(
            0,
            Storyboard::neuf(STORYBOARD_D_AVANT, Grille::default())
        )],
        "la grille de Tauri, aux réglages de naissance, au rang 0"
    );

    // Réécrit par cette build, il passe au schéma du jour sans rien perdre.
    let reecrit = persist::histoire::contenu_geste(&geste);
    assert_eq!(
        persist::histoire::lire_geste(&reecrit).expect("relu"),
        geste
    );
}

/// **Un geste de l'histoire écrit au schéma v4 se relit** (fiche 68) : un storyboard posé avec
/// sa durée et un champ de plus, et une case qui dit tout ce qu'une case du v4 disait.
///
/// Le témoin a été écrit par la build de `5bb82ec`.
#[test]
fn test_un_geste_v4_du_storyboard_se_relit() {
    let temoin = include_bytes!("persist_fixture/storyboard-v4.geste");
    let geste = persist::histoire::lire_geste(temoin).expect("un geste v4 doit se lire");
    let (cases, storyboards) = cases_et_storyboards(&geste);
    let mut attendu = Storyboard::neuf(STORYBOARD_D_AVANT, Grille::default());
    attendu.duree_ms = 3500;
    attendu.champs.push("Son".into());
    assert_eq!(storyboards, vec![Slot::inserted(0, attendu)]);
    let mut case = StoryboardPanel::new(
        "case-v4",
        STORYBOARD_D_AVANT,
        0,
        (10.0, 20.0),
        (280.0, 157.5),
    );
    case.description = "une case lue".into();
    case.duree_ms = Some(1200);
    case.plan = Some(Plan::Gros);
    case.ecrire(1, "il court");
    assert_eq!(cases, vec![case]);

    let reecrit = persist::histoire::contenu_geste(&geste);
    assert_eq!(
        persist::histoire::lire_geste(&reecrit).expect("relu"),
        geste
    );
}

/// **La version se lit dans le manifeste, jamais dans les octets** : relire un document en
/// croyant qu'il est d'un schéma plus ancien échoue, au lieu de rendre un document à moitié
/// juste.
#[test]
fn test_lire_un_schema_comme_un_plus_ancien_echoue_au_lieu_de_deviner() {
    let nu = persist::encode_document(&rich_project());
    for ancien in 1..persist::DOCUMENT_VERSION {
        assert!(
            persist::decode_document_v(&nu, ancien).is_err(),
            "les octets d'aujourd'hui ne peuvent pas passer pour un v{ancien}"
        );
    }
    assert!(persist::decode_document_v(&nu, persist::DOCUMENT_VERSION).is_ok());
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
    assert_eq!(board.panels.len(), 2);
    assert_eq!(board.storyboards.len(), 2);
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
