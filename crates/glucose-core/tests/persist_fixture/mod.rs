//! Un projet d'essai volontairement non trivial, partagé par les suites de persistance.
//!
//! Chaque champ du modèle y reçoit une valeur **distinctive** : rien n'est laissé à sa valeur
//! par défaut, sinon un champ oublié par le sérialiseur passerait l'aller-retour sans qu'on
//! s'en aperçoive. C'est le pendant, côté données, de la déstructuration exhaustive côté code
//! (INVARIANT PERSIST-1).

use glucose_core::types::{
    Annotation, ArrowPredicate, AssetRef, Board, BoardImage, BoardZone, CanvasFolder, CurtainEditable,
    CurtainNote, CurtainVisibility, Domain, DomainAssignment, FolderMirrorSource, FolderSortMode,
    MembraneCurtain, MembraneMode, Point2D, Preset, PresetSlot, Project, StickyOperator,
    StoryboardPanel, TemporalAnchor, TextAnchor, TextSelection, Viewport,
};

fn anchor(start: i64, end: i64, label: &str) -> TemporalAnchor {
    TemporalAnchor {
        start,
        end,
        label: Some(label.to_string()),
    }
}

fn weights() -> Vec<DomainAssignment> {
    vec![
        DomainAssignment {
            domain_id: "dom-science".into(),
            weight: 0.75,
        },
        DomainAssignment {
            domain_id: "dom-art".into(),
            weight: 0.25,
        },
    ]
}

fn images() -> Vec<BoardImage> {
    let mut embedded = BoardImage::new("img-1", -120.5, 42.0, 320.0, 180.0);
    embedded.membrane_id = Some("memb-1".into());
    embedded.asset = Some(AssetRef::Embed {
        sha256: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".into(),
        mime: "image/png".into(),
        size_bytes: Some(4096),
    });
    embedded.src = Some("asset:croquis.png".into());
    embedded.rotation = -37.5;
    embedded.locked = true;
    embedded.tags = vec!["référence".into(), "étape 1".into()];
    embedded.slot_id = Some("slot-hero".into());
    embedded.source_url = Some("https://example.invalid/croquis.png".into());
    embedded.original_width = 1920.0;
    embedded.original_height = 1080.0;
    embedded.is_video = false;
    embedded.fit = Some("cover".into());
    embedded.domains = weights();
    embedded.mirror_of = Some("img-source".into());
    embedded.temporal_anchor = Some(anchor(-3000, -2500, "Âge du bronze"));

    let mut linked = BoardImage::new("img-2", 0.0, -0.0, 64.0, 64.0);
    linked.asset = Some(AssetRef::Link {
        href: "asset:planche.jpg".into(),
        sha256: None,
        size_bytes: None,
    });
    linked.src = Some("C:/photos/planche.jpg".into());
    linked.is_video = true;
    linked.tags = vec![];

    vec![embedded, linked]
}

fn text_annotation() -> Annotation {
    Annotation::Text {
        id: "ann-text-1".into(),
        x: 12.25,
        y: -48.75,
        width: Some(260.0),
        height: Some(48.0),
        text: "# Titre\n- puce\n- deuxième puce ✦".into(),
        font_size: Some(15.5),
        color: Some("#38bdf8".into()),
        cursor_pos: Some(7),
        source_file: Some("notes/chapitre-1.md".into()),
        membrane_id: Some("memb-1".into()),
        domains: weights(),
        mirror_of: Some("ann-text-origine".into()),
        temporal_anchor: Some(anchor(1789, 1799, "Révolution")),
    }
}

fn sticky_annotation() -> Annotation {
    Annotation::Sticky {
        id: "ann-sticky-1".into(),
        x: 300.0,
        y: 120.0,
        width: Some(180.0),
        height: None,
        text: "à relire".into(),
        font_size: None,
        color: Some("#0f172a".into()),
        bg_color: Some("#fde047".into()),
        cursor_pos: None,
        operator: Some(StickyOperator::Because),
        source_file: None,
        membrane_id: None,
        domains: vec![],
        mirror_of: None,
        temporal_anchor: None,
    }
}

fn arrow_annotation() -> Annotation {
    Annotation::Arrow {
        id: "ann-arrow-1".into(),
        x: 10.0,
        y: 20.0,
        x2: 410.0,
        y2: -220.5,
        text: Some("influence directe".into()),
        font_size: Some(11.0),
        color: Some("#f472b6".into()),
        arrow_type: Some("curved".into()),
        arrow_bidirectional: true,
        predicate: Some(ArrowPredicate::EstPrecurseur),
        stroke_width: Some(2.5),
        waypoints: vec![
            Point2D { x: 100.0, y: 0.0 },
            Point2D { x: 220.5, y: -110.25 },
        ],
        source_id: Some("ann-text-1".into()),
        target_id: Some("img-1".into()),
        source_block_id: Some("bloc-3".into()),
        target_block_id: Some("bloc-9".into()),
        source_text_sel: Some(TextSelection::Legacy("3:17".into())),
        target_text_sel: Some(TextSelection::Anchors(vec![
            TextAnchor {
                start: 0,
                end: 12,
                quote: "premier extrait".into(),
                prefix: Some("avant ".into()),
                suffix: None,
            },
            TextAnchor {
                start: 40,
                end: 55,
                quote: "second extrait".into(),
                prefix: None,
                suffix: Some(" après".into()),
            },
        ])),
        long_text: Some("Un paragraphe entier attaché à la flèche.".into()),
        target_board_id: Some("board-2".into()),
        membrane_id: Some("memb-1".into()),
        domains: weights(),
        mirror_of: None,
        temporal_anchor: Some(anchor(-44, -44, "Ides de mars")),
    }
}

fn membrane_annotation() -> Annotation {
    Annotation::Membrane {
        id: "memb-1".into(),
        x: -400.0,
        y: -300.0,
        width: 900.0,
        height: 700.0,
        color: Some("#22d3ee".into()),
        text: Some("Chapitre I".into()),
        mode: MembraneMode::Stretched,
        curtains: vec![
            MembraneCurtain {
                id: "curtain-1".into(),
                owner_id: "user-7".into(),
                owner_name: "Ada".into(),
                owner_color: "#a78bfa".into(),
                visibility: CurtainVisibility::Shared,
                editable: CurtainEditable::Everyone,
                collapsed_ratio: Some(0.15),
                expanded_ratio: Some(0.85),
                board_id: Some("board-2".into()),
                notes: vec![
                    CurtainNote {
                        id: "note-1".into(),
                        text: "première remarque".into(),
                        created_at: 1_700_000_000_000,
                    },
                    CurtainNote {
                        id: "note-2".into(),
                        text: "seconde remarque".into(),
                        created_at: 1_700_000_001_000,
                    },
                ],
                created_at: 1_699_999_999_000,
            },
            MembraneCurtain {
                id: "curtain-2".into(),
                owner_id: "user-8".into(),
                owner_name: "Grace".into(),
                owner_color: "#34d399".into(),
                visibility: CurtainVisibility::Private,
                editable: CurtainEditable::Owner,
                collapsed_ratio: None,
                expanded_ratio: None,
                board_id: None,
                notes: vec![],
                created_at: -1,
            },
        ],
        membrane_id: None,
        domains: vec![],
        mirror_of: Some("memb-origine".into()),
        temporal_anchor: None,
    }
}

fn folders() -> Vec<CanvasFolder> {
    let mut plain = CanvasFolder::new("folder-1", "Références", "board-3");
    plain.color = "#f97316".into();
    plain.x = 640.0;
    plain.y = -128.0;
    plain.width = 240.0;
    plain.height = 180.0;

    let mut mirrored = CanvasFolder::new("folder-2", "Disque local", "board-4");
    mirrored.mirror_of = Some("folder-1".into());
    mirrored.mirror_source = Some(FolderMirrorSource {
        root_path: "C:/Users/ada/Images".into(),
        mode: "live".into(),
        last_scanned_at: 1_770_000_000_000,
        pattern: Some("*.png".into()),
        recursive: true,
        sort_by: Some(FolderSortMode::ModifiedDesc),
        pending_scan: true,
    });

    vec![plain, mirrored]
}

fn main_board() -> Board {
    let mut board = Board::new("board-1", "Canevas principal");
    board.images = images();
    board.annotations = vec![
        text_annotation(),
        sticky_annotation(),
        arrow_annotation(),
        membrane_annotation(),
    ];
    board.folders = folders();
    board.panels = vec![{
        let mut panel = StoryboardPanel::new("panel-1", 3, 0.0, 0.0, 320.0, 180.0);
        panel.description = "plan d'ouverture".into();
        panel
    }];
    board.zones = vec![BoardZone::new("slot-hero", -50.0, -50.0, 400.0, 300.0)];
    board.viewport = Viewport {
        x: -220.5,
        y: 77.25,
        scale: 0.375,
    };
    board.bookmarks.insert(
        "vue-large".into(),
        Viewport {
            x: 0.0,
            y: 0.0,
            scale: 0.1,
        },
    );
    board.bookmarks.insert(
        "détail".into(),
        Viewport {
            x: 640.0,
            y: -120.0,
            scale: 3.0,
        },
    );
    board.created_at = 1_699_000_000_000;
    board.updated_at = 1_770_000_000_000;
    board
}

/// Le projet d'essai complet : 2 tableaux, 2 images, les 4 variantes d'annotation, 2 dossiers
/// dont un miroir de disque, un storyboard, une zone, 2 signets, 1 preset, 2 domaines.
pub fn rich_project() -> Project {
    let mut project = Project::new("Projet d'essai — ÉÀÜ, 日本語, 🍬");
    project.version = "2.0.0-essai".into();
    project.boards = vec![main_board(), Board::new("board-2", "Annexe")];
    project.active_board_id = "board-2".into();
    project.presets = vec![Preset {
        id: "preset-1".into(),
        name: "Planche contact".into(),
        description: "trois zones".into(),
        slots: vec![
            PresetSlot {
                id: "slot-hero".into(),
                name: "Héros".into(),
                color: "#ef4444".into(),
                description: "image principale".into(),
                order: 0,
            },
            PresetSlot {
                id: "slot-detail".into(),
                name: "Détail".into(),
                color: "#3b82f6".into(),
                description: "gros plan".into(),
                order: -1,
            },
        ],
        is_builtin: true,
        created_at: 1_600_000_000_000,
    }];
    project.domains = vec![
        Domain {
            id: "dom-science".into(),
            name: "Science".into(),
            color: "#22c55e".into(),
            icon: "🔬".into(),
            created_at: 1,
        },
        Domain {
            id: "dom-art".into(),
            name: "Art".into(),
            color: "#a855f7".into(),
            icon: "🎨".into(),
            created_at: -1,
        },
    ];
    project.collab_url = Some("wss://example.invalid/salle".into());
    project.asset_channel_url = Some("https://example.invalid/actifs".into());
    project.created_at = -86_400_000;
    project.updated_at = 1_770_000_123_456;
    project
}
