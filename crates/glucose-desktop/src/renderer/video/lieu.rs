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

#[cfg(target_os = "android")]
impl Lieu {
    /// Le lieu, tel que la porte d'Android le reçoit.
    pub(crate) fn ou(&self) -> crate::plateforme::video::Ou<'_> {
        use crate::plateforme::video::Ou;
        match self {
            Self::Fichier {
                chemin,
                debut,
                longueur,
            } => Ou::Fichier {
                chemin,
                debut: *debut,
                longueur: *longueur,
            },
            Self::Octets(octets) => Ou::Octets(octets),
        }
    }
}

/// **Les octets d'un lieu, ouverts** : lus à une position, là où ils sont, par les lecteurs qui les
/// demandent eux-mêmes (le flot de Windows, le chargeur du Mac).
#[cfg(any(windows, target_os = "macos"))]
pub(crate) struct Octets {
    source: SourceOuverte,
    longueur: u64,
}

#[cfg(any(windows, target_os = "macos"))]
enum SourceOuverte {
    Fichier { fichier: std::fs::File, debut: u64 },
    Memoire(Arc<[u8]>),
}

#[cfg(any(windows, target_os = "macos"))]
impl Lieu {
    /// **Ouvre ses octets** ; `None` si le fichier ne s'ouvre pas.
    pub(crate) fn ouvrir(&self) -> Option<Octets> {
        Some(match self {
            Self::Fichier {
                chemin,
                debut,
                longueur,
            } => {
                let fichier = std::fs::File::open(chemin).ok()?;
                let longueur = match longueur {
                    Some(l) => *l,
                    None => fichier.metadata().ok()?.len().saturating_sub(*debut),
                };
                let debut = *debut;
                Octets {
                    source: SourceOuverte::Fichier { fichier, debut },
                    longueur,
                }
            }
            Self::Octets(octets) => Octets {
                source: SourceOuverte::Memoire(octets.clone()),
                longueur: octets.len() as u64,
            },
        })
    }
}

#[cfg(any(windows, target_os = "macos"))]
impl Octets {
    pub(crate) fn longueur(&self) -> u64 {
        self.longueur
    }

    /// **Lit à partir de `a`**, tant qu'il y a de la place et des octets ; rend le nombre lu.
    pub(crate) fn lire(&self, a: u64, tampon: &mut [u8]) -> std::io::Result<usize> {
        let n = (tampon.len() as u64).min(self.longueur.saturating_sub(a)) as usize;
        let tampon = &mut tampon[..n];
        match &self.source {
            SourceOuverte::Memoire(octets) => {
                tampon.copy_from_slice(&octets[a as usize..a as usize + n]);
                Ok(n)
            }
            SourceOuverte::Fichier { fichier, debut } => {
                let mut lus = 0;
                while lus < n {
                    match lire_a(fichier, &mut tampon[lus..], debut + a + lus as u64)? {
                        0 => break,
                        k => lus += k,
                    }
                }
                Ok(lus)
            }
        }
    }

    /// **L'extension que sa signature dit** (`mp4`, `mov`…) : le lecteur du système choisit son
    /// démultiplexeur par elle.
    pub(crate) fn extension(&self) -> Option<&'static str> {
        let mut tete = [0; super::signature::LONGUEUR];
        let lus = self.lire(0, &mut tete).ok()?;
        super::signature::extension(&tete[..lus])
    }
}

/// Lit à une position du fichier, sans curseur partagé.
#[cfg(windows)]
fn lire_a(fichier: &std::fs::File, tampon: &mut [u8], a: u64) -> std::io::Result<usize> {
    std::os::windows::fs::FileExt::seek_read(fichier, tampon, a)
}

#[cfg(target_os = "macos")]
fn lire_a(fichier: &std::fs::File, tampon: &mut [u8], a: u64) -> std::io::Result<usize> {
    std::os::unix::fs::FileExt::read_at(fichier, tampon, a)
}
