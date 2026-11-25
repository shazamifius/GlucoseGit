//! **Où sont les octets d'une vidéo** (VIDEO-1, fiche 169) : ce que le lecteur du système lit.

use crate::persist::objets::{Objets, Source};
use std::path::PathBuf;
use std::sync::Arc;

/// **Où sont les octets d'une vidéo** : un fichier (entier, ou la tranche d'un document), ou la
/// mémoire. Le lecteur les lit là, sans copie.
pub(crate) enum Lieu {
    Fichier {
        chemin: PathBuf,
        debut: u64,
        /// `None` : jusqu'au bout du fichier.
        longueur: Option<u64>,
    },
    Octets(Arc<[u8]>),
}

impl Lieu {
    /// **Le lieu des octets de cette clé** — la même table que [`Objets::lire`], sans rien lire.
    /// L'empreinte d'une tranche ne se vérifie pas ici (il faudrait tout lire avant la première
    /// image) : une tranche abîmée fait échouer le décodeur, jamais montrer autre chose.
    pub(crate) fn de(src: &str, objets: Option<&Objets>) -> Option<Self> {
        let tranche = |chemin, debut, longueur| Self::Fichier {
            chemin,
            debut,
            longueur: Some(longueur),
        };
        let entier = |chemin| Self::Fichier {
            chemin,
            debut: 0,
            longueur: None,
        };
        let Some(objets) = objets else {
            return Some(entier(PathBuf::from(src)));
        };
        Some(match objets.source(src) {
            Some(Source::Tranche {
                offset, longueur, ..
            }) => tranche(objets.document()?, offset, longueur),
            Some(Source::Ailleurs {
                fichier,
                offset,
                longueur,
                ..
            }) => tranche(fichier, offset, longueur),
            Some(Source::Fichier(chemin)) => entier(chemin),
            Some(Source::Memoire(octets)) => Self::Octets(octets.as_slice().into()),
            Some(Source::Promise(p)) => Self::Octets(p.deja()?.as_slice().into()),
            None => entier(PathBuf::from(src)),
        })
    }
}
