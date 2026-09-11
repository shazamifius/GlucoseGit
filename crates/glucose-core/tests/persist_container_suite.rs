//! INVARIANT PERSIST-3 — un fichier abîmé est détecté, jamais accepté en silence.
//!
//! Ces tests attaquent le conteneur `.glucose` v2 par tous les bouts : troncature, somme de
//! contrôle falsifiée, signature étrangère, numéro de version hors de portée, section absente.
//! Aucun ne doit paniquer ; chacun doit rendre un message qui dit à l'utilisateur quoi faire.

use glucose_core::persist::{self, container, manifest};
use glucose_core::types::{AssetStore, Board, BoardImage, Project};

fn small_project() -> Project {
    let mut project = Project::new("conteneur");
    let mut board = Board::new("b", "B");
    board.images = vec![BoardImage::new("img-1", 1.0, 2.0, 3.0, 4.0)];
    project.boards = vec![board];
    project.active_board_id = "b".into();
    project
}

fn message_of(file: &[u8]) -> String {
    persist::decode(file)
        .expect_err("ce fichier ne doit pas être accepté")
        .to_string()
}

// ── Troncature ──────────────────────────────────────────────────────────────

#[test]
fn test_a_file_truncated_in_its_payload_is_rejected() {
    let file = persist::encode(&small_project(), &AssetStore::new(), 0);
    let cut = &file[..file.len() - 8];
    let msg = message_of(cut);
    assert!(msg.contains("tronqué"), "message inattendu : {msg}");
}

#[test]
fn test_a_file_truncated_inside_its_section_table_is_rejected() {
    let file = persist::encode(&small_project(), &AssetStore::new(), 0);
    let cut = &file[..container::HEADER_LEN + 10];
    let msg = message_of(cut);
    assert!(msg.contains("tronqué"), "message inattendu : {msg}");
}

#[test]
fn test_an_empty_file_is_rejected_without_panicking() {
    let msg = message_of(&[]);
    assert!(msg.contains("tronqué"), "message inattendu : {msg}");
}

#[test]
fn test_every_truncation_length_is_rejected_and_none_panics() {
    // Le cas réel de la coupure de courant : le fichier s'arrête à un offset quelconque.
    // Aucune longueur intermédiaire ne doit être acceptée, et aucune ne doit paniquer.
    let file = persist::encode(&small_project(), &AssetStore::new(), 0);
    for len in 0..file.len() {
        assert!(
            persist::decode(&file[..len]).is_err(),
            "un fichier de {len} octets sur {} a été accepté",
            file.len()
        );
    }
    assert!(persist::decode(&file).is_ok(), "le fichier complet doit passer");
}

// ── Altération ──────────────────────────────────────────────────────────────

#[test]
fn test_a_flipped_bit_in_the_document_breaks_its_checksum() {
    let mut file = persist::encode(&small_project(), &AssetStore::new(), 0);
    let last = file.len() - 1;
    file[last] ^= 0x01;
    let msg = message_of(&file);
    assert!(msg.contains("somme de contrôle"), "message inattendu : {msg}");
}

#[test]
fn test_a_forged_checksum_in_the_table_is_detected() {
    let mut file = persist::encode(&small_project(), &AssetStore::new(), 0);
    // Empreinte de la première entrée de table : octets 24..56 de l'entrée.
    file[container::HEADER_LEN + 24] ^= 0xff;
    let msg = message_of(&file);
    assert!(msg.contains("somme de contrôle"), "message inattendu : {msg}");
}

#[test]
fn test_reserved_bytes_must_stay_zero() {
    let mut file = persist::encode(&small_project(), &AssetStore::new(), 0);
    file[container::HEADER_LEN + 3] = 0x42;
    let msg = message_of(&file);
    assert!(msg.contains("réservés"), "message inattendu : {msg}");
}

#[test]
fn test_a_foreign_file_is_not_mistaken_for_a_project() {
    let msg = message_of(b"PK\x03\x04 ceci est une archive zip, pas un projet Glucose");
    assert!(msg.contains("signature"), "message inattendu : {msg}");
}

// ── Numéros de version ──────────────────────────────────────────────────────

#[test]
fn test_a_container_from_a_newer_glucose_says_to_update() {
    let mut file = persist::encode(&small_project(), &AssetStore::new(), 0);
    file[8..10].copy_from_slice(&99u16.to_le_bytes());
    let msg = message_of(&file);
    assert!(msg.contains("v99"), "message inattendu : {msg}");
    assert!(msg.contains("mets Glucose à jour"), "message inattendu : {msg}");
}

#[test]
fn test_a_v1_container_names_the_missing_importer() {
    let mut file = persist::encode(&small_project(), &AssetStore::new(), 0);
    file[8..10].copy_from_slice(&1u16.to_le_bytes());
    let msg = message_of(&file);
    assert!(msg.contains("v1"), "message inattendu : {msg}");
    assert!(msg.contains("importeur"), "message inattendu : {msg}");
}

#[test]
fn test_unknown_header_flags_are_refused_rather_than_ignored() {
    let mut file = persist::encode(&small_project(), &AssetStore::new(), 0);
    file[10..12].copy_from_slice(&0b10u16.to_le_bytes());
    let msg = message_of(&file);
    assert!(msg.contains("fanions"), "message inattendu : {msg}");
}

#[test]
fn test_a_newer_document_schema_says_to_update() {
    // Conteneur v2 parfaitement valide, mais dont le manifeste annonce un schéma futur.
    let project = small_project();
    let future = manifest::Manifest {
        document_version: persist::DOCUMENT_VERSION + 7,
        project_name: project.name.clone(),
        saved_at: 0,
        assets: Vec::new(),
    };
    let file = container::assemble(&[
        container::Section {
            kind: container::KIND_MANIFEST,
            payload: manifest::encode(&future),
        },
        container::Section {
            kind: container::KIND_DOCUMENT,
            payload: persist::encode_document(&project),
        },
    ]);

    let msg = message_of(&file);
    assert!(msg.contains("schéma"), "message inattendu : {msg}");
    assert!(msg.contains("mets Glucose à jour"), "message inattendu : {msg}");
}

// ── Sections ────────────────────────────────────────────────────────────────

#[test]
fn test_a_file_without_its_document_section_is_rejected() {
    let file = container::assemble(&[container::Section {
        kind: container::KIND_MANIFEST,
        payload: manifest::encode(&manifest::Manifest {
            document_version: persist::DOCUMENT_VERSION,
            project_name: "orphelin".into(),
            saved_at: 0,
            assets: Vec::new(),
        }),
    }]);
    let msg = message_of(&file);
    assert!(msg.contains("document"), "message inattendu : {msg}");
    assert!(msg.contains("absent"), "message inattendu : {msg}");
}

#[test]
fn test_a_journal_section_is_skipped_not_refused() {
    // Le journal incrémental du § 8 n'est pas livré. Un fichier écrit par une build qui
    // l'embarque doit malgré tout s'ouvrir ici, sans la reprise qu'il portait.
    let project = small_project();
    let file = container::assemble(&[
        container::Section {
            kind: container::KIND_MANIFEST,
            payload: manifest::encode(&manifest::Manifest {
                document_version: persist::DOCUMENT_VERSION,
                project_name: project.name.clone(),
                saved_at: 0,
                assets: Vec::new(),
            }),
        },
        container::Section {
            kind: container::KIND_DOCUMENT,
            payload: persist::encode_document(&project),
        },
        container::Section {
            kind: container::KIND_JOURNAL,
            payload: b"des commandes que cette version ne sait pas rejouer".to_vec(),
        },
    ]);

    let reloaded = persist::decode(&file).expect("le journal doit être ignoré, pas refuser le fichier");
    assert_eq!(reloaded.project, project);
}

#[test]
fn test_an_asset_announced_but_absent_is_reported_by_name() {
    let project = small_project();
    let file = container::assemble(&[
        container::Section {
            kind: container::KIND_MANIFEST,
            payload: manifest::encode(&manifest::Manifest {
                document_version: persist::DOCUMENT_VERSION,
                project_name: project.name.clone(),
                saved_at: 0,
                assets: vec![manifest::AssetEntry {
                    key: "C:/photos/disparue.png".into(),
                    digest: [3u8; 32],
                    size: 12,
                }],
            }),
        },
        container::Section {
            kind: container::KIND_DOCUMENT,
            payload: persist::encode_document(&project),
        },
    ]);

    let msg = message_of(&file);
    assert!(msg.contains("disparue.png"), "message inattendu : {msg}");
}

// ── Actifs adressés par contenu ─────────────────────────────────────────────

#[test]
fn test_a_duplicated_asset_writes_a_single_blob() {
    let mut assets = AssetStore::new();
    let picture = vec![0xAAu8; 4096];
    assets.insert("C:/photos/a.png", picture.clone());
    assets.insert("C:/photos/copie-de-a.png", picture.clone());
    assets.insert("D:/sauvegarde/a.png", picture);
    assets.insert("C:/photos/b.png", vec![0xBBu8; 2048]);

    let file = persist::encode(&small_project(), &assets, 0);
    assert_eq!(
        persist::asset_section_count(&file).expect("le conteneur doit se lire"),
        2,
        "3 clés portant les mêmes octets doivent partager une seule section"
    );

    // Le fichier reste plus petit que la somme naïve des quatre contenus.
    assert!(
        file.len() < 4096 * 3 + 2048,
        "la déduplication n'a pas réduit la taille du fichier ({} octets)",
        file.len()
    );

    let reloaded = persist::decode(&file).expect("relecture");
    assert_eq!(reloaded.assets.len(), 4, "les 4 clés doivent revenir");
    assert_eq!(reloaded.assets.get("C:/photos/a.png"), reloaded.assets.get("D:/sauvegarde/a.png"));
    assert_eq!(reloaded.assets.get("C:/photos/b.png").map(<[u8]>::len), Some(2048));
}

#[test]
fn test_assets_round_trip_byte_for_byte() {
    let mut assets = AssetStore::new();
    assets.insert("vide", Vec::new());
    assets.insert("octets", (0u8..=255).collect::<Vec<u8>>());
    assets.insert("clé accentuée é.png", vec![1, 2, 3]);

    let file = persist::encode(&small_project(), &assets, 42);
    let reloaded = persist::decode(&file).expect("relecture");
    assert_eq!(reloaded.assets, assets);
}

#[test]
fn test_an_asset_whose_bytes_were_swapped_is_detected() {
    let mut assets = AssetStore::new();
    assets.insert("photo", vec![7u8; 64]);
    let mut file = persist::encode(&small_project(), &assets, 0);
    let last = file.len() - 1;
    file[last] ^= 0xff;
    let msg = message_of(&file);
    assert!(msg.contains("somme de contrôle"), "message inattendu : {msg}");
}
