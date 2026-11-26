//! **Les documents jamais ouverts, trouvés sur ce PC** (RECENTS-1, fiche 174) : un fil qui
//! parcourt le dossier personnel à la recherche des `.glucose`, sans rien lire d'autre que les
//! noms — Glucose ne fouillait pas le disque (fiche 88) ; sa demande du 25/11 l'y invite, pour
//! montrer aussi ce qu'il n'a jamais ouvert.
//!
//! # Ce qu'il ne parcourt pas
//!
//! Ce que personne n'y range à la main : les dossiers cachés (un point devant le nom, ou l'attribut
//! caché de Windows — `AppData`), ceux qui se déclarent cache (`CACHEDIR.TAG`, la convention que
//! Cargo, Bazel ou ccache suivent : bford.info/cachedir), les liens symboliques et, sous Linux, ce qui
//! est monté d'un autre disque (comme `find -xdev`). Aucune borne de nombre ni de profondeur : il
//! cède au fil qui dessine, et ce qu'il trouve paraît au fur et à mesure.
//!
//! Ni au téléphone, où Glucose ne voit que ses propres documents, ni sur le Mac, qui demanderait à
//! l'utilisateur la permission d'entrer dans chacun de ses dossiers (Documents, Bureau…) : là, la
//! liste montre les récents et leurs voisins.

use super::Trouve;
use crate::plateforme::Reveil;
use glucose_core::persist::FILE_EXTENSION;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// **La fouille** : où elle cherche, ce qu'elle a trouvé jusqu'ici, et si elle cherche encore.
/// Sans [`Fouille::brancher`] (une épreuve, un banc), elle ne cherche nulle part : seul le vrai
/// lancement la fait entrer chez l'utilisateur.
#[derive(Default)]
pub struct Fouille {
    partage: Arc<Partage>,
    racine: Option<PathBuf>,
    reveil: Option<Reveil>,
}

#[derive(Default)]
struct Partage {
    trouves: Mutex<Vec<Trouve>>,
    en_cours: AtomicBool,
    /// Combien de fois la liste a grandi : la palette se recompose quand il change.
    version: AtomicU64,
}

impl Fouille {
    /// **Où chercher, et qui réveiller** : au vrai lancement.
    pub fn brancher(&mut self, racine: Option<PathBuf>, reveil: Reveil) {
        self.racine = racine;
        self.reveil = Some(reveil);
    }

    /// **Cherche**, sauf si une fouille cherche déjà ; chaque trouvaille réveille la boucle. Une
    /// fouille finie se relance : un document posé depuis paraît.
    pub fn lancer(&self) {
        let Some(racine) = self.racine.clone() else {
            return;
        };
        if self.partage.en_cours.swap(true, Ordering::AcqRel) {
            return;
        }
        let (partage, reveil) = (Arc::clone(&self.partage), self.reveil.clone());
        let lance = std::thread::Builder::new()
            .name("fouille".into())
            .spawn(move || {
                crate::plateforme::priorite::ceder_au_fil_qui_dessine();
                let mut deja: HashSet<PathBuf> = partage.trouves.lock().map_or_else(
                    |_| HashSet::new(),
                    |t| t.iter().map(|(p, _)| p.clone()).collect(),
                );
                parcourir(&racine, &mut |chemin| {
                    if deja.insert(chemin.to_path_buf()) {
                        let numero = crate::plateforme::numero::de(chemin);
                        if let Ok(mut t) = partage.trouves.lock() {
                            t.push((chemin.to_path_buf(), numero));
                        }
                        partage.version.fetch_add(1, Ordering::AcqRel);
                        if let Some(reveil) = &reveil {
                            reveil();
                        }
                    }
                });
                // Finie, elle le dit : la palette cesse d'annoncer qu'elle cherche.
                partage.en_cours.store(false, Ordering::Release);
                partage.version.fetch_add(1, Ordering::AcqRel);
                if let Some(reveil) = &reveil {
                    reveil();
                }
            });
        if lance.is_err() {
            self.partage.en_cours.store(false, Ordering::Release);
        }
    }

    /// Ce qu'elle a trouvé jusqu'ici.
    pub fn trouves(&self) -> Vec<Trouve> {
        self.partage
            .trouves
            .lock()
            .map(|t| t.clone())
            .unwrap_or_default()
    }

    /// Combien de fois la liste a grandi.
    pub fn version(&self) -> u64 {
        self.partage.version.load(Ordering::Acquire)
    }

    /// Cherche-t-elle encore ?
    pub fn en_cours(&self) -> bool {
        self.partage.en_cours.load(Ordering::Acquire)
    }
}

/// **Où chercher** : le dossier personnel, sur un bureau qui le permet sans rien demander.
pub fn racine() -> Option<PathBuf> {
    if cfg!(any(target_os = "android", target_os = "macos")) {
        return None;
    }
    #[cfg(windows)]
    let personnel = std::env::var_os("USERPROFILE");
    #[cfg(not(windows))]
    let personnel = std::env::var_os("HOME");
    personnel.map(PathBuf::from).filter(|p| p.is_dir())
}

/// Un document de Glucose, à son extension.
pub fn est_un_document(chemin: &Path) -> bool {
    chemin
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(FILE_EXTENSION))
        && chemin.is_file()
}

/// **Parcourt `racine`** et donne chaque document trouvé. En largeur, par une pile : aucune
/// profondeur ne fait déborder la pile du fil.
pub fn parcourir(racine: &Path, trouve: &mut dyn FnMut(&Path)) {
    let disque = disque_de(racine);
    let mut a_voir = vec![racine.to_path_buf()];
    while let Some(dossier) = a_voir.pop() {
        let Ok(entrees) = std::fs::read_dir(&dossier) else {
            continue;
        };
        let mut sous = Vec::new();
        let mut cache = false;
        for entree in entrees.flatten() {
            let nom = entree.file_name();
            cache |= nom == "CACHEDIR.TAG";
            let Ok(sorte) = entree.file_type() else {
                continue;
            };
            let chemin = entree.path();
            if sorte.is_dir() && !est_cache(&entree, &nom) {
                sous.push(chemin);
            } else if sorte.is_file() && est_un_document(&chemin) {
                trouve(&chemin);
            }
        }
        if !cache {
            a_voir.extend(sous.into_iter().filter(|d| disque_de(d) == disque));
        }
    }
}

/// Un dossier caché : un point devant son nom, ou l'attribut caché de Windows.
fn est_cache(entree: &std::fs::DirEntry, nom: &std::ffi::OsStr) -> bool {
    if nom.to_string_lossy().starts_with('.') {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const CACHE: u32 = 0x2; // FILE_ATTRIBUTE_HIDDEN
        const SYSTEME: u32 = 0x4; // FILE_ATTRIBUTE_SYSTEM
        if let Ok(m) = entree.metadata() {
            return m.file_attributes() & (CACHE | SYSTEME) != 0;
        }
    }
    let _ = entree;
    false
}

/// Le disque qui porte ce dossier (Unix) : on ne passe pas sur un autre.
#[cfg(unix)]
fn disque_de(chemin: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    std::fs::symlink_metadata(chemin).ok().map(|m| m.dev())
}

#[cfg(not(unix))]
fn disque_de(_: &Path) -> Option<u64> {
    None
}
