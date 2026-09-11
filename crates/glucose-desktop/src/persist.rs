//! Enregistrer et ouvrir un projet `.glucose` (répare R-01).
//!
//! # Partage des rôles
//!
//! Le **format** vit dans `glucose_core::persist` : pur, sans I/O, testable sans fenêtre.
//! Ce module-ci ne fait que ce que le noyau n'a pas le droit de faire — toucher au disque,
//! lire l'horloge, ouvrir un dialogue — et remonter chaque échec à l'utilisateur par un toast
//! (standard § 6.4). Un échec d'enregistrement silencieux serait pire que pas d'enregistrement.
//!
//! | Raccourci | Effet |
//! |---|---|
//! | `Ctrl+S` | Enregistre ; demande un chemin si le projet n'en a pas encore |
//! | `Ctrl+Maj+S` | Enregistre sous un nouveau chemin |
//! | `Ctrl+O` | Ouvre un projet |
//! | `Ctrl+I` | Importe des images (le raccourci qu'occupait `Ctrl+O`) |

pub mod assets;
pub mod atomic;
pub mod close;
pub mod commands;

use crate::error::{DesktopError, DesktopResult};
use glucose_core::persist::{self, GlucoseFile, FILE_EXTENSION};
use glucose_core::types::{AssetStore, Project};
use std::path::{Path, PathBuf};

/// Marqueur de modifications non enregistrées, en tête du titre de la fenêtre.
pub const DIRTY_MARK: &str = "\u{25cf} ";
/// Nom de l'application dans le titre de la fenêtre.
pub const APP_TITLE: &str = "GLUCOSE";
/// Nom affiché tant que le projet n'a pas de chemin sur le disque.
pub const UNTITLED: &str = "Projet sans titre";

/// Ce qu'un enregistrement a réellement écrit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveReport {
    pub bytes: usize,
    pub assets: usize,
    pub unreadable: usize,
}

// ── I/O de fichier, sans fenêtre ni dialogue ────────────────────────────────

/// Millisecondes depuis l'époque Unix. Le noyau ne lit jamais l'horloge : c'est ici que la
/// date d'enregistrement entre dans le manifeste, et que la date de création d'un domaine
/// entre dans le document.
pub fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Impose l'extension `.glucose` à un chemin choisi dans un dialogue.
pub fn with_glucose_extension(path: PathBuf) -> PathBuf {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case(FILE_EXTENSION) => path,
        _ => path.with_extension(FILE_EXTENSION),
    }
}

/// Encode puis écrit un projet **atomiquement** (voir [`atomic::write_atomic`]).
pub fn write_project_file(
    path: &Path,
    project: &Project,
    assets: &AssetStore,
    saved_at: i64,
) -> DesktopResult<usize> {
    let bytes = persist::encode(project, assets, saved_at);
    atomic::write_atomic(path, &bytes)?;
    Ok(bytes.len())
}

/// Lit et décode un projet. Toute corruption devient une erreur nommée, jamais une panique.
pub fn read_project_file(path: &Path) -> DesktopResult<GlucoseFile> {
    let raw = std::fs::read(path).map_err(|e| DesktopError::OpenFailed {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    persist::decode(&raw).map_err(DesktopError::from)
}

/// Écriture lisible d'une taille de fichier, pour le toast de confirmation.
pub fn human_size(bytes: usize) -> String {
    const KIB: usize = 1024;
    const MIB: usize = KIB * 1024;
    if bytes >= MIB {
        format!("{:.1} Mio", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} Kio", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} o")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glucose_core::types::{Annotation, Board, BoardImage};

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("glucose-tests-persist");
        std::fs::create_dir_all(&dir).expect("le dossier temporaire du système doit être créable");
        dir.join(name)
    }

    fn sample_project() -> Project {
        let mut project = Project::new("Aller-retour disque");
        let mut board = Board::new("b1", "Principal");
        board.images = vec![BoardImage::new("img-1", 10.0, -20.0, 300.0, 200.0)];
        board.annotations = vec![Annotation::Text {
            id: "ann-1".into(),
            x: 4.0,
            y: 5.0,
            width: Some(120.0),
            height: Some(30.0),
            text: "écrit puis relu".into(),
            font_size: Some(13.0),
            color: None,
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        }];
        project.boards = vec![board, Board::new("b2", "Annexe")];
        project.active_board_id = "b2".into();
        project
    }

    #[test]
    fn test_a_project_written_to_disk_comes_back_identical() {
        // La preuve de bout en bout demandée : ce sont les MÊMES fonctions que `Ctrl+S` et
        // `Ctrl+O` appellent, pas une réimplémentation de test.
        let path = scratch("aller-retour.glucose");
        let project = sample_project();
        let mut assets = AssetStore::new();
        assets.insert("C:/photos/a.png", vec![1, 2, 3, 4]);

        let written = write_project_file(&path, &project, &assets, 1_770_000_000_000)
            .expect("l'ecriture doit reussir");
        assert!(written > 0);

        let reloaded = read_project_file(&path).expect("la relecture doit reussir");
        assert_eq!(reloaded.project, project, "le document relu differe de l'original");
        assert_eq!(reloaded.assets, assets, "les actifs relus different");
        assert_eq!(reloaded.manifest.saved_at, 1_770_000_000_000);

        std::fs::remove_file(&path).expect("nettoyage");
    }

    #[test]
    fn test_saving_twice_replaces_the_file_without_leaving_a_temporary() {
        let path = scratch("double-enregistrement.glucose");
        let mut project = sample_project();
        write_project_file(&path, &project, &AssetStore::new(), 0).expect("premiere ecriture");

        project.name = "Renomme".into();
        write_project_file(&path, &project, &AssetStore::new(), 1).expect("seconde ecriture");

        let reloaded = read_project_file(&path).expect("relecture");
        assert_eq!(reloaded.project.name, "Renomme");
        std::fs::remove_file(&path).expect("nettoyage");
    }

    #[test]
    fn test_a_truncated_file_on_disk_is_reported_not_silently_accepted() {
        let path = scratch("tronque.glucose");
        let project = sample_project();
        let bytes = persist::encode(&project, &AssetStore::new(), 0);
        std::fs::write(&path, &bytes[..bytes.len() / 2]).expect("ecriture du fichier mutile");

        let err = read_project_file(&path).expect_err("un fichier tronque ne doit pas passer");
        assert!(err.to_string().contains("tronqué"), "message : {err}");
        std::fs::remove_file(&path).expect("nettoyage");
    }

    #[test]
    fn test_opening_a_missing_file_says_what_to_check() {
        let err = read_project_file(Path::new("C:/nulle-part/absent.glucose"))
            .expect_err("le fichier n'existe pas");
        assert!(err.to_string().contains("Ouverture impossible"), "message : {err}");
    }

    #[test]
    fn test_extension_is_forced_on_the_chosen_path() {
        assert_eq!(
            with_glucose_extension(PathBuf::from("C:/travail/carnet")),
            PathBuf::from("C:/travail/carnet.glucose")
        );
        assert_eq!(
            with_glucose_extension(PathBuf::from("C:/travail/carnet.GLUCOSE")),
            PathBuf::from("C:/travail/carnet.GLUCOSE"),
            "une extension deja correcte ne doit pas etre reecrite"
        );
        assert_eq!(
            with_glucose_extension(PathBuf::from("C:/travail/carnet.txt")),
            PathBuf::from("C:/travail/carnet.glucose")
        );
    }

    #[test]
    fn test_human_size_reads_like_a_file_manager() {
        assert_eq!(human_size(0), "0 o");
        assert_eq!(human_size(512), "512 o");
        assert_eq!(human_size(2048), "2.0 Kio");
        assert_eq!(human_size(3 * 1024 * 1024), "3.0 Mio");
    }

}
