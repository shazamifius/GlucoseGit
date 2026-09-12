//! Écriture atomique : un plantage pendant la sauvegarde ne détruit jamais le fichier
//! précédent.
//!
//! # INVARIANT SAVE-1 — la destination n'est jamais ouverte en écriture
//!
//! Le contenu part dans un fichier **temporaire voisin**, il est poussé jusqu'au disque par
//! `sync_all()`, et c'est seulement ensuite qu'un `rename` le met à la place de la
//! destination. Les trois étapes comptent, et chacune répare un scénario précis :
//!
//! | Étape | Ce qu'elle empêche |
//! |---|---|
//! | Écrire dans un temporaire | Une coupure en plein `write_all` laisse le `.glucose` d'hier intact, pas un fichier à moitié écrit |
//! | `sync_all()` **avant** le renommage | Une coupure de courant juste après le renommage : sans `fsync`, le nom pointerait un contenu encore dans le cache du système, donc perdu |
//! | `rename` | L'opération de remplacement est atomique pour le système de fichiers — un lecteur voit l'ancien fichier ou le nouveau, jamais un entre-deux |
//!
//! Le temporaire est dans le **même dossier** que la destination : un `rename` entre volumes
//! n'est pas atomique (et échoue purement et simplement sous Windows).

use crate::error::{DesktopError, DesktopResult};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn failed(path: &Path, reason: impl std::fmt::Display) -> DesktopError {
    DesktopError::SaveFailed {
        path: path.display().to_string(),
        reason: reason.to_string(),
    }
}

/// Nom du fichier temporaire : voisin de la destination, caché, et unique par processus et par
/// instant, pour que deux enregistrements concurrents ne se marchent pas dessus.
fn temp_sibling(path: &Path) -> PathBuf {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let stem = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "projet.glucose".to_string());
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    parent.join(format!(".{}.{}.{}.tmp", stem, std::process::id(), nanos))
}

/// Pousse le dossier parent jusqu'au disque, pour que le renommage lui-même survive à une
/// coupure. Seul un système POSIX permet d'ouvrir un dossier ; sous Windows, `MoveFileEx`
/// publie déjà l'entrée de répertoire de façon durable.
#[cfg(unix)]
fn sync_parent(path: &Path) -> DesktopResult<()> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    File::open(parent)
        .and_then(|dir| dir.sync_all())
        .map_err(|e| failed(path, e))
}

#[cfg(not(unix))]
fn sync_parent(_path: &Path) -> DesktopResult<()> {
    Ok(())
}

/// Écrit `bytes` à `path` sans jamais mettre en danger le contenu qui s'y trouve déjà.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> DesktopResult<()> {
    let temp = temp_sibling(path);

    if let Err(err) = fill(&temp, bytes) {
        discard(&temp);
        return Err(failed(path, err));
    }

    if let Err(err) = std::fs::rename(&temp, path) {
        discard(&temp);
        return Err(failed(path, err));
    }

    sync_parent(path)
}

/// Écrit le temporaire et le pousse jusqu'au disque.
fn fill(temp: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(temp)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// Efface un temporaire abandonné. Un échec ici ne peut rien corriger et ne doit surtout pas
/// masquer l'erreur d'origine, qui est celle que l'utilisateur doit lire.
fn discard(temp: &Path) {
    if temp.exists() {
        match std::fs::remove_file(temp) {
            Ok(()) => {}
            Err(err) => eprintln!(
                "[Glucose] temporaire d'enregistrement laissé sur le disque : {} ({err})",
                temp.display()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chaque cas a SON dossier : `cargo test` les exécute en parallèle, et
    /// `test_write_atomic_leaves_no_temporary_behind` inspecte le dossier entier — il verrait
    /// sinon le temporaire d'un autre cas en cours d'écriture et échouerait au hasard.
    fn scratch(case: &str, name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("glucose-tests-atomic").join(case);
        std::fs::create_dir_all(&dir).expect("le dossier temporaire du système doit être créable");
        dir.join(name)
    }

    #[test]
    fn test_write_atomic_creates_then_replaces() {
        let path = scratch("remplacement", "remplacement.glucose");
        write_atomic(&path, b"premiere version").expect("premiere ecriture");
        assert_eq!(std::fs::read(&path).expect("relecture"), b"premiere version");

        write_atomic(&path, b"seconde").expect("seconde ecriture");
        assert_eq!(std::fs::read(&path).expect("relecture"), b"seconde");

        std::fs::remove_file(&path).expect("nettoyage");
    }

    /// Fiche 09 § 3.3 — « si l'écriture échoue, le fichier original n'est jamais corrompu ».
    ///
    /// On fait échouer le remplacement de façon déterministe : la destination est un
    /// **dossier** non vide, qu'aucun `rename` ne peut écraser. Le temporaire a été écrit en
    /// entier ; l'échec survient à la dernière étape, celle qui touche la destination. Elle
    /// doit ressortir intacte, et le temporaire ne doit pas rester.
    ///
    /// Ce test remplace `test_a_failed_write_preserves_the_previous_file`, qui écrivait un
    /// fichier A, faisait échouer une écriture vers un chemin B *différent*, et constatait
    /// que A n'avait pas bougé — ce qui ne disait rien de SAVE-1.
    #[test]
    fn test_a_failed_replacement_leaves_the_destination_untouched() {
        let inside = scratch("remplacement-rate/destination", "temoin.txt");
        std::fs::write(&inside, b"contenu d'hier").expect("temoin");
        let destination = inside.parent().expect("le dossier destination").to_path_buf();

        let err = write_atomic(&destination, b"nouveau contenu").expect_err("un dossier ne se remplace pas");
        assert!(matches!(err, DesktopError::SaveFailed { .. }), "{err:?}");
        assert!(err.to_string().contains("intact"), "le message le dit : {err}");

        assert!(destination.is_dir(), "la destination est toujours le dossier d'hier");
        assert_eq!(std::fs::read(&inside).expect("relecture"), b"contenu d'hier");
        let leftovers: Vec<String> = std::fs::read_dir(destination.parent().expect("parent"))
            .expect("lecture du dossier")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temporaires restants : {leftovers:?}");

        std::fs::remove_dir_all(destination.parent().expect("parent")).expect("nettoyage");
    }

    #[test]
    fn test_write_atomic_leaves_no_temporary_behind() {
        let path = scratch("sans-residu", "sans-residu.glucose");
        write_atomic(&path, b"contenu").expect("ecriture");

        let parent = path.parent().expect("le fichier a un dossier parent");
        let leftovers: Vec<String> = std::fs::read_dir(parent)
            .expect("lecture du dossier")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temporaires restants : {leftovers:?}");

        std::fs::remove_file(&path).expect("nettoyage");
    }

    #[test]
    fn test_temp_sibling_stays_in_the_destination_directory() {
        let path = scratch("voisin", "voisin.glucose");
        let temp = temp_sibling(&path);
        assert_eq!(temp.parent(), path.parent(), "le temporaire a change de volume");
        assert!(
            temp.to_string_lossy().ends_with(".tmp"),
            "le temporaire doit etre reconnaissable : {}",
            temp.display()
        );
    }
}
