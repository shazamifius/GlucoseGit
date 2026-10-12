//! **Ce que les épreuves partagent** : un dossier à elles, qui ne reste jamais derrière.
//!
//! # Pourquoi un seul
//!
//! Huit modules d'épreuves avaient chacun le leur, recopié — et deux qui l'avaient écrit
//! autrement, sans le retirer à la fin, ont laissé près de mille huit cents dossiers dans le
//! dossier temporaire de sa machine (vu le 12/10). Deux autres, lancés ensemble, se vidaient
//! leur dossier parce qu'ils lui donnaient le même nom (vu le 07/10).
//!
//! Le nom porte donc le processus **et un compteur** : deux épreuves ne partagent jamais un
//! dossier, quel que soit le nom qu'elles lui donnent. Et il se retire quand l'épreuve finit,
//! qu'elle réussisse ou qu'elle tombe.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Un dossier d'épreuve, neuf et vide, retiré quand il sort de portée.
pub(crate) struct Dossier(PathBuf);

impl Dossier {
    /// `glucose-<nom>-<processus>-<rang>`, dans le dossier temporaire du système.
    pub(crate) fn nouveau(nom: &str) -> Self {
        static RANG: AtomicUsize = AtomicUsize::new(0);
        let rang = RANG.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("glucose-{nom}-{}-{rang}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).expect("un dossier d'épreuve");
        Self(p)
    }
}

impl std::ops::Deref for Dossier {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for Dossier {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for Dossier {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
