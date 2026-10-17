//! **La porte locale** : elle écoute sur un fil à elle, et sert chaque connexion sur le sien.
//!
//! Une connexion lit une ligne, la confie au [`Connexion`] du protocole, et répond. Un appel
//! d'outil part dans la boîte aux lettres, et la connexion attend la réponse de la boucle
//! avant de lire la ligne suivante : un agent qui attend l'accord de l'utilisateur attend, et
//! ses appels arrivent dans l'ordre où il les a faits.
//!
//! Chaque système a sa porte ([`windows`], [`unix`]) ; tout le reste est ici, une fois.

use super::protocole::{Connexion, Suite};
use super::{Boite, Demande};
use std::io::{BufRead, BufReader, Read, Write};

#[cfg(windows)]
mod tuyau;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::{connecter, nom_par_defaut, Porte};
/// Le flux d'une connexion à la porte, côté relais.
#[cfg(windows)]
pub type Flux = tuyau::Tuyau;

#[cfg(all(unix, not(target_os = "android")))]
mod unix;
#[cfg(all(unix, not(target_os = "android")))]
pub use self::unix::{connecter, nom_par_defaut, Porte};
#[cfg(all(unix, not(target_os = "android")))]
pub type Flux = std::os::unix::net::UnixStream;

/// Le téléphone n'a pas de porte : aucun agent n'y tourne à côté de Glucose.
#[cfg(target_os = "android")]
pub struct Porte;

#[cfg(target_os = "android")]
impl Porte {
    pub fn ouvrir(_nom: &str, _boite: Boite) -> Option<Self> {
        None
    }
}

#[cfg(target_os = "android")]
pub fn nom_par_defaut() -> Option<String> {
    None
}

/// La ligne la plus longue qu'une connexion accepte. Un texte d'un million de mots tient en
/// huit mégaoctets ; au-delà, ce n'est plus un message, c'est un fichier — et une ligne sans
/// fin ne doit pas pouvoir remplir la mémoire de Glucose.
pub const LIGNE_MAX: usize = 16 << 20;

/// **Lit une ligne**, bornée à [`LIGNE_MAX`] : `Ok(None)` à la fin du flux, une erreur au-delà
/// de la borne.
pub fn lire_une_ligne(lecteur: &mut impl BufRead) -> std::io::Result<Option<String>> {
    let mut octets = Vec::new();
    loop {
        let tampon = lecteur.fill_buf()?;
        if tampon.is_empty() {
            return Ok((!octets.is_empty()).then(|| String::from_utf8_lossy(&octets).into_owned()));
        }
        let (pris, fin) = match tampon.iter().position(|b| *b == b'\n') {
            Some(i) => (i + 1, true),
            None => (tampon.len(), false),
        };
        octets.extend_from_slice(&tampon[..pris]);
        lecteur.consume(pris);
        if octets.len() > LIGNE_MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "ligne trop longue",
            ));
        }
        if fin {
            octets.pop();
            if octets.last() == Some(&b'\r') {
                octets.pop();
            }
            return Ok(Some(String::from_utf8_lossy(&octets).into_owned()));
        }
    }
}

/// **Sert une connexion** jusqu'à ce que l'agent la ferme — ou que Glucose se ferme : la boucle
/// partie, la réponse n'arrive plus, et la connexion se termine au lieu d'attendre.
pub fn servir(lecteur: impl Read, mut ecrivain: impl Write, boite: &Boite) {
    let mut lecteur = BufReader::new(lecteur);
    let mut connexion = Connexion::default();
    while let Ok(Some(ligne)) = lire_une_ligne(&mut lecteur) {
        if ligne.trim().is_empty() {
            continue;
        }
        let reponse = match connexion.traiter(&ligne) {
            Suite::Rien => continue,
            Suite::Repondre(r) => r,
            Suite::Executer(appel) => {
                let (envoi, attente) = std::sync::mpsc::channel();
                boite.deposer(Demande {
                    appel,
                    reponse: envoi,
                });
                match attente.recv() {
                    Ok(r) => r,
                    Err(_) => return,
                }
            }
        };
        if writeln!(ecrivain, "{reponse}")
            .and_then(|()| ecrivain.flush())
            .is_err()
        {
            return;
        }
    }
}

#[cfg(test)]
mod tests;
