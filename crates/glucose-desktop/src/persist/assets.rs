//! Incorporation et restitution des octets d'image autour du magasin d'actifs.
//!
//! Le modèle ne connaît qu'un `src` : un chemin de fichier sur la machine où l'image a été
//! importée. Tel quel, un `.glucose` déplacé sur une autre machine s'ouvre avec des cadres
//! vides — R-01 ne serait réparé qu'à moitié.
//!
//! Ces deux fonctions ferment le circuit, et **elles seules** touchent au disque des images :
//!
//! - [`collect`] lit les octets de chaque `src` encore présent et remplit le magasin d'actifs.
//!   Il est **reconstruit à chaque enregistrement**, ce qui fait du ramasse-miettes du § 8 une
//!   conséquence et non un traitement à part : un actif que plus aucune image ne référence
//!   n'est simplement pas recollecté.
//! - [`restore`] fait l'inverse à l'ouverture : un `src` qui n'existe plus sur cette machine
//!   est réécrit vers le cache d'actifs, où le contenu vient d'être déposé sous son sha256.
//!
//! Le renderer, lui, ne change pas : il continue de lire un chemin de fichier (R-29 reste
//! entier, il sera traité quand le chargement d'image quittera la boucle de rendu).

use glucose_core::hash::{hex_of, sha256};
use glucose_core::types::{AssetStore, Project};
use std::path::{Path, PathBuf};

/// Ce qu'un enregistrement a pu — ou n'a pas pu — emporter avec lui.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Collected {
    pub store: AssetStore,
    /// Images dont le `src` n'est plus lisible : elles restent dans le document, mais leurs
    /// octets ne voyagent pas. L'utilisateur doit le savoir (standard § 6.4).
    pub unreadable: Vec<String>,
}

/// Un `src` ne désigne un fichier à incorporer que s'il est un chemin local.
fn is_local_path(src: &str) -> bool {
    !src.is_empty()
        && !src.starts_with("data:")
        && !src.starts_with("asset:")
        && !src.starts_with("http://")
        && !src.starts_with("https://")
}

/// Lit les octets de toutes les images référencées par le projet.
pub fn collect(project: &Project) -> Collected {
    let mut out = Collected::default();
    for board in &project.boards {
        for image in &board.images {
            let Some(src) = image.src.as_deref() else {
                continue;
            };
            if !is_local_path(src) || out.store.blobs.contains_key(src) {
                continue;
            }
            match std::fs::read(src) {
                Ok(bytes) => out.store.insert(src, bytes),
                Err(_) => {
                    if !out.unreadable.iter().any(|s| s == src) {
                        out.unreadable.push(src.to_string());
                    }
                }
            }
        }
    }
    out
}

/// Dossier où sont déposés les actifs restitués, hors du projet lui-même.
fn cache_dir() -> PathBuf {
    std::env::temp_dir().join("glucose-assets")
}

/// Nom de fichier d'un actif restitué : son empreinte, plus l'extension d'origine pour que le
/// décodeur d'image choisisse le bon format. Content-addressed, donc écrit une seule fois.
fn cache_name(src: &str, bytes: &[u8]) -> String {
    let digest = hex_of(&sha256(bytes));
    match Path::new(src).extension().and_then(|e| e.to_str()) {
        Some(ext) if !ext.is_empty() => format!("{digest}.{ext}"),
        _ => digest,
    }
}

/// Réécrit vers le cache d'actifs les `src` qui n'existent plus sur cette machine.
///
/// Rend le nombre d'images remises sur pied. Une image dont le `src` existe encore n'est
/// **jamais** touchée : ouvrir un projet sur sa machine d'origine ne déplace rien.
pub fn restore(project: &mut Project, assets: &AssetStore) -> usize {
    if assets.is_empty() {
        return 0;
    }
    let dir = cache_dir();
    if let Err(err) = std::fs::create_dir_all(&dir) {
        eprintln!("[Glucose] cache d'actifs inaccessible : {err}");
        return 0;
    }

    let mut restored = 0;
    for board in &mut project.boards {
        for image in &mut board.images {
            let Some(src) = image.src.clone() else {
                continue;
            };
            if Path::new(&src).is_file() {
                continue;
            }
            let Some(bytes) = assets.get(&src) else {
                continue;
            };
            let Some(target) = place_in_cache(&dir, &src, bytes) else {
                continue;
            };
            image.src = Some(target.to_string_lossy().to_string());
            restored += 1;
        }
    }
    restored
}

/// Dépose un contenu dans le cache et rend son chemin. Un contenu déjà présent n'est pas
/// réécrit : le nom étant son empreinte, le fichier qui porte ce nom porte déjà ces octets.
fn place_in_cache(dir: &Path, src: &str, bytes: &[u8]) -> Option<PathBuf> {
    let target = dir.join(cache_name(src, bytes));
    if target.is_file() {
        return Some(target);
    }
    match std::fs::write(&target, bytes) {
        Ok(()) => Some(target),
        Err(err) => {
            eprintln!("[Glucose] actif non restitué ({}) : {err}", target.display());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glucose_core::types::{Board, BoardImage};

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("glucose-tests-assets");
        std::fs::create_dir_all(&dir).expect("le dossier temporaire du système doit être créable");
        dir.join(name)
    }

    fn project_with_srcs(srcs: &[&str]) -> Project {
        let mut project = Project::new("actifs");
        let mut board = Board::new("b", "B");
        for (index, src) in srcs.iter().enumerate() {
            let mut image = BoardImage::new(format!("img-{index}"), 0.0, 0.0, 10.0, 10.0);
            image.src = Some((*src).to_string());
            board.images.push(image);
        }
        project.boards = vec![board];
        project.active_board_id = "b".into();
        project
    }

    #[test]
    fn test_collect_reads_each_referenced_file_once() {
        let path = scratch("collecte.png");
        std::fs::write(&path, b"des octets d'image").expect("ecriture du fichier d'essai");
        let src = path.to_string_lossy().to_string();

        let collected = collect(&project_with_srcs(&[&src, &src]));
        assert_eq!(collected.store.len(), 1, "deux images du meme fichier = une entree");
        assert_eq!(collected.store.get(&src), Some(&b"des octets d'image"[..]));
        assert!(collected.unreadable.is_empty());

        std::fs::remove_file(&path).expect("nettoyage");
    }

    #[test]
    fn test_collect_reports_a_missing_file_instead_of_failing() {
        let collected = collect(&project_with_srcs(&["C:/nulle-part/absente.png"]));
        assert!(collected.store.is_empty());
        assert_eq!(collected.unreadable, vec!["C:/nulle-part/absente.png".to_string()]);
    }

    #[test]
    fn test_collect_ignores_non_file_sources() {
        let collected = collect(&project_with_srcs(&[
            "data:image/png;base64,AAAA",
            "https://exemple.invalide/a.png",
            "asset:deja-incorpore.png",
            "",
        ]));
        assert!(collected.store.is_empty());
        assert!(collected.unreadable.is_empty(), "aucune n'est un chemin local");
    }

    #[test]
    fn test_collect_drops_assets_of_deleted_images() {
        // Ramasse-miettes du § 8 : le magasin est reconstruit, donc une image supprimée du
        // document emporte son actif sans traitement dédié.
        let path = scratch("ramasse-miettes.png");
        std::fs::write(&path, b"contenu").expect("ecriture");
        let src = path.to_string_lossy().to_string();

        let mut project = project_with_srcs(&[&src]);
        assert_eq!(collect(&project).store.len(), 1);
        project.boards[0].images.clear();
        assert_eq!(collect(&project).store.len(), 0);

        std::fs::remove_file(&path).expect("nettoyage");
    }

    #[test]
    fn test_restore_rewrites_only_the_sources_that_vanished() {
        let mut assets = AssetStore::new();
        assets.insert("C:/machine-absente/photo.png", b"octets restitues".to_vec());

        let present = scratch("toujours-la.png");
        std::fs::write(&present, b"inchange").expect("ecriture");
        let present_src = present.to_string_lossy().to_string();
        assets.insert(present_src.clone(), b"inchange".to_vec());

        let mut project = project_with_srcs(&["C:/machine-absente/photo.png", &present_src]);
        assert_eq!(restore(&mut project, &assets), 1, "une seule image a restituer");

        let restored_src = project.boards[0].images[0].src.clone().expect("src restitue");
        assert!(Path::new(&restored_src).is_file(), "le fichier restitue doit exister");
        assert_eq!(
            std::fs::read(&restored_src).expect("relecture"),
            b"octets restitues"
        );
        assert_eq!(
            project.boards[0].images[1].src.as_deref(),
            Some(present_src.as_str()),
            "une image encore presente sur le disque ne doit pas etre deplacee"
        );

        std::fs::remove_file(&present).expect("nettoyage");
        std::fs::remove_file(&restored_src).expect("nettoyage");
    }

    #[test]
    fn test_cache_name_is_content_addressed() {
        let a = cache_name("C:/photos/nom-a.png", b"memes octets");
        let b = cache_name("D:/ailleurs/nom-b.png", b"memes octets");
        assert_eq!(a, b, "deux chemins, un seul contenu, un seul nom de cache");
        assert!(a.ends_with(".png"));
        assert_ne!(a, cache_name("C:/photos/nom-a.png", b"autres octets"));
    }
}
