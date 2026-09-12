//! La preuve de complétude du pont : un tableau qui entre et ressort identique.

use super::*;
use crate::types::{
    Annotation, ArrowPredicate, AssetRef, Board, BoardImage, CanvasFolder, CurtainEditable,
    CurtainVisibility, DomainAssignment, FolderMirrorSource, FolderSortMode, MembraneCurtain,
    MembraneMode, Point2D, StickyOperator, StoryboardPanel, TemporalAnchor, TextAnchor,
    TextSelection, Viewport,
};

/// Un tableau où **tout** est rempli : les quatre genres d'annotation, une image, un dossier,
/// un panneau, et chacun de leurs champs à une valeur non triviale.
///
/// Les coordonnées sont des multiples d'un 256e de pixel et les flottants sont exacts en `f32`
/// là où l'arène les range en `f32` : ce test mesure la complétude du pont, pas la précision de
/// [`Fx`], qui a ses propres tests.
fn tableau_complet() -> Board {
    let mut b = Board::new("board-1", "Tableau complet");
    b.viewport = Viewport { x: 12.0, y: -34.0, scale: 2.0 };
    b.bookmarks.insert("repère".into(), Viewport { x: 1.0, y: 2.0, scale: 0.5 });
    b.created_at = 1_700_000_000;
    b.updated_at = 1_700_000_001;

    let domaines = vec![
        DomainAssignment { domain_id: "science".into(), weight: 0.75 },
        DomainAssignment { domain_id: "histoire".into(), weight: 0.25 },
    ];
    let ancre = TemporalAnchor { start: -3000, end: 1789, label: Some("longue durée".into()) };

    b.images.push(BoardImage {
        id: "img-1".into(),
        membrane_id: Some("mem-1".into()),
        asset: Some(AssetRef::Embed {
            sha256: "abc123".into(),
            mime: "image/png".into(),
            size_bytes: Some(4096),
        }),
        src: Some("fichier.png".into()),
        x: 100.5,
        y: 200.25,
        width: 640.0,
        height: 480.0,
        rotation: 45.0,
        locked: true,
        tags: vec!["une".into(), "deux".into()],
        slot_id: Some("slot-a".into()),
        source_url: Some("https://exemple.test/i.png".into()),
        original_width: 1920.0,
        original_height: 1080.0,
        is_video: true,
        fit: Some("cover".into()),
        domains: domaines.clone(),
        mirror_of: Some("img-2".into()),
        temporal_anchor: Some(ancre.clone()),
    });
    b.images.push(BoardImage::new("img-2", -50.0, -60.0, 10.0, 20.0));

    b.annotations.push(Annotation::Text {
        id: "txt-1".into(),
        x: 10.0,
        y: 20.0,
        width: Some(300.0),
        height: Some(120.0),
        text: "Un texte avec des accents, des émojis 🌱 et des retours\nà la ligne".into(),
        font_size: Some(14.0),
        color: Some("#eab308".into()),
        cursor_pos: Some(7),
        source_file: Some("notes.md".into()),
        membrane_id: Some("mem-1".into()),
        domains: domaines.clone(),
        mirror_of: Some("txt-2".into()),
        temporal_anchor: Some(ancre.clone()),
    });
    // Une carte dont la taille est calculée : c'est le `None` que les drapeaux AUTO portent.
    b.annotations.push(Annotation::Text {
        id: "txt-2".into(),
        x: -1000.0,
        y: 3000.5,
        width: None,
        height: None,
        text: String::new(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    });
    b.annotations.push(Annotation::Sticky {
        id: "s-1".into(),
        x: 500.0,
        y: 500.0,
        width: Some(200.0),
        height: None,
        text: "une note".into(),
        font_size: Some(13.0),
        color: Some("#111111".into()),
        bg_color: Some("#f5c542".into()),
        cursor_pos: Some(3),
        operator: Some(StickyOperator::Because),
        source_file: Some("carnet.md".into()),
        membrane_id: None,
        domains: domaines.clone(),
        mirror_of: None,
        temporal_anchor: None,
    });
    b.annotations.push(Annotation::Arrow {
        id: "arr-1".into(),
        x: 900.0,
        y: 100.0,
        x2: 200.0,
        y2: 700.0,
        text: Some("étiquette".into()),
        font_size: Some(11.0),
        color: Some("#60a5fa".into()),
        arrow_type: Some("courbe".into()),
        arrow_bidirectional: true,
        predicate: Some(ArrowPredicate::Contredit),
        stroke_width: Some(2.5),
        waypoints: vec![Point2D { x: 700.0, y: 300.0 }, Point2D { x: 400.0, y: 500.0 }],
        source_id: Some("txt-1".into()),
        target_id: Some("img-1".into()),
        source_block_id: Some("bloc-a".into()),
        target_block_id: Some("bloc-b".into()),
        source_text_sel: Some(TextSelection::Legacy("ancienne".into())),
        target_text_sel: Some(TextSelection::Anchors(vec![TextAnchor {
            start: 3,
            end: 9,
            quote: "extrait".into(),
            prefix: Some("avant".into()),
            suffix: Some("après".into()),
        }])),
        long_text: Some("un long développement".into()),
        target_board_id: Some("board-2".into()),
        membrane_id: Some("mem-1".into()),
        domains: domaines.clone(),
        mirror_of: None,
        temporal_anchor: Some(ancre.clone()),
    });
    b.annotations.push(Annotation::Membrane {
        id: "mem-1".into(),
        x: -200.0,
        y: -200.0,
        width: 2000.0,
        height: 1500.0,
        color: Some("#2a2a2a".into()),
        text: Some("Une membrane".into()),
        mode: MembraneMode::Stretched,
        curtains: vec![MembraneCurtain {
            id: "cur-1".into(),
            owner_id: "u-1".into(),
            owner_name: "Quelqu'un".into(),
            owner_color: "#38bdf8".into(),
            visibility: CurtainVisibility::Shared,
            editable: CurtainEditable::Everyone,
            collapsed_ratio: Some(0.1),
            expanded_ratio: Some(0.9),
            board_id: Some("board-3".into()),
            notes: Vec::new(),
            created_at: 42,
        }],
        membrane_id: None,
        domains: domaines.clone(),
        mirror_of: None,
        temporal_anchor: None,
    });

    b.folders.push(CanvasFolder {
        id: "fold-1".into(),
        name: "Un dossier".into(),
        color: "#888888".into(),
        x: 1000.0,
        y: -1000.0,
        width: 200.0,
        height: 150.0,
        child_board_id: "board-enfant".into(),
        mirror_of: Some("fold-2".into()),
        mirror_source: Some(FolderMirrorSource {
            root_path: "/un/chemin".into(),
            mode: "live".into(),
            last_scanned_at: 99,
            pattern: Some("*.md".into()),
            recursive: true,
            sort_by: Some(FolderSortMode::ModifiedDesc),
            pending_scan: true,
        }),
    });
    b.folders.push(CanvasFolder::new("fold-2", "Second", "board-enfant-2"));

    b.panels.push(StoryboardPanel {
        id: "pan-1".into(),
        order: 3,
        description: "Un panneau".into(),
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 180.0,
    });
    b
}

/// **La preuve de complétude.** Un tableau dont chaque champ est rempli traverse le pont et
/// ressort identique, à la virgule près. Si un champ n'était pas transporté, cette égalité
/// tomberait — c'est ce qui rend une perte silencieuse impossible.
#[test]
fn test_un_tableau_traverse_le_pont_sans_rien_perdre() {
    let avant = tableau_complet();
    let pont = Bridge::from_board(&avant);
    pont.check().unwrap();
    let apres = pont.to_board_like(&avant);
    assert_eq!(apres, avant);
}

/// Le pont est **idempotent** : refaire le trajet ne change plus rien. Un aller-retour qui
/// dérive à chaque passage serait un aller-retour qui ment au premier.
#[test]
fn test_refaire_le_trajet_ne_change_plus_rien() {
    let depart = tableau_complet();
    let un = Bridge::from_board(&depart).to_board_like(&depart);
    let deux = Bridge::from_board(&un).to_board_like(&un);
    assert_eq!(deux, un);
}

/// Les trois normalisations annoncées, et rien d'autre : la casse d'une couleur, la forme
/// courte d'un hexadécimal, et un texte optionnel vide.
#[test]
fn test_les_normalisations_sont_celles_qui_sont_annoncees() {
    let mut b = Board::new("b", "b");
    b.annotations.push(Annotation::Membrane {
        id: "m".into(),
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
        color: Some("#ABC".into()),
        text: Some(String::new()),
        mode: MembraneMode::Classic,
        curtains: Vec::new(),
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    });
    let apres = Bridge::from_board(&b).to_board_like(&b);
    let Annotation::Membrane { color, text, .. } = &apres.annotations[0] else {
        panic!("membrane attendue");
    };
    assert_eq!(color.as_deref(), Some("#aabbcc"), "la forme courte est développée");
    assert_eq!(*text, None, "un libellé vide ressort absent");
}

/// Une couleur que l'arène ne sait pas lire est conservée à la lettre, pas perdue.
#[test]
fn test_une_couleur_illisible_est_conservee_telle_quelle() {
    let mut b = Board::new("b", "b");
    b.annotations.push(Annotation::Sticky {
        id: "s".into(),
        x: 0.0,
        y: 0.0,
        width: Some(10.0),
        height: Some(10.0),
        text: String::new(),
        font_size: None,
        color: Some("rebeccapurple".into()),
        bg_color: Some("rgb(1, 2, 3)".into()),
        cursor_pos: None,
        operator: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    });
    let pont = Bridge::from_board(&b);
    pont.check().unwrap();
    let apres = pont.to_board_like(&b);
    assert_eq!(apres, b, "ni la couleur de trait ni celle de fond ne se perdent");
}

/// Une référence vers un nœud absent — ce qu'un fichier incomplet porte — ouvre le document
/// avec un lien rompu plutôt qu'une erreur.
#[test]
fn test_une_reference_rompue_ouvre_le_document_quand_meme() {
    let mut b = Board::new("b", "b");
    let mut img = BoardImage::new("i", 0.0, 0.0, 10.0, 10.0);
    img.membrane_id = Some("membrane-qui-n-existe-pas".into());
    img.mirror_of = Some("fantome".into());
    b.images.push(img);

    let pont = Bridge::from_board(&b);
    pont.check().unwrap();
    let apres = pont.to_board_like(&b);
    assert_eq!(apres.images[0].membrane_id, None, "le parent introuvable disparaît");
    assert_eq!(apres.images[0].mirror_of, None, "le miroir introuvable aussi");
}

/// Les quatre orientations d'une flèche traversent le pont, y compris la dégénérée : c'est le
/// même invariant que celui de l'arène, vérifié de bout en bout cette fois.
#[test]
fn test_les_quatre_orientations_d_une_fleche_traversent_le_pont() {
    for (x, y, x2, y2) in [
        (0.0, 0.0, 300.0, 200.0),
        (300.0, 200.0, 0.0, 0.0),
        (300.0, 0.0, 0.0, 200.0),
        (0.0, 200.0, 300.0, 0.0),
        (7.0, 7.0, 7.0, 7.0),
    ] {
        let mut b = Board::new("b", "b");
        b.annotations.push(Annotation::Arrow {
            id: "a".into(),
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
        });
        let apres = Bridge::from_board(&b).to_board_like(&b);
        assert_eq!(apres, b, "flèche ({x},{y}) → ({x2},{y2})");
    }
}

/// Un tableau vide traverse le pont sans cas particulier.
#[test]
fn test_un_tableau_vide_traverse_le_pont() {
    let b = Board::new("vide", "Rien");
    let pont = Bridge::from_board(&b);
    assert!(pont.is_empty());
    assert_eq!(pont.len(), 0);
    pont.check().unwrap();
    assert_eq!(pont.to_board_like(&b), b);
}

/// Le document sur l'arène pèse une fraction de ce que pèse le modèle historique, sur le même
/// contenu — et ce contenu-ci est le plus défavorable qui soit, puisque chaque champ est
/// rempli.
#[test]
fn test_le_document_pese_moins_que_le_modele_meme_tout_rempli() {
    let b = tableau_complet();
    let pont = Bridge::from_board(&b);
    let n = pont.len();
    let modele = b.images.len() * std::mem::size_of::<BoardImage>()
        + b.annotations.len() * std::mem::size_of::<Annotation>();
    assert!(
        pont.doc.bytes() < modele,
        "{} octets sur l'arène contre {modele} pour {n} nœuds (pile seule)",
        pont.doc.bytes()
    );
}
