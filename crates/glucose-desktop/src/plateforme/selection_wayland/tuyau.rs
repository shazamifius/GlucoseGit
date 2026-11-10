//! **Les octets d'un transfert Wayland, par un tuyau** — lus ou écrits avec une patience bornée.
//!
//! Sous Wayland, rien ne transite par le compositeur : celui qui colle crée un tuyau, en donne le
//! bout d'écriture à celui qui a copié (`wl_data_offer.receive`), et lit l'autre bout jusqu'à la
//! fin. Une application figée ne doit jamais figer Glucose avec elle : chaque attente se borne,
//! entre deux morceaux, par `poll` — la patience de Qt, comme sous X11
//! ([`super::super::selection_x11`]).

use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd};
use std::time::Duration;

/// **Combien de temps on attend une application qui ne répond plus**, entre deux morceaux : cinq
/// secondes, la patience de Qt (`QXcbClipboard::clipboardTimeout`).
pub(super) const PATIENCE: Duration = Duration::from_secs(5);

/// Attend que ce descripteur soit prêt (`POLLIN` ou `POLLOUT`), au plus `patience`.
fn attendre(fd: &impl AsFd, evenement: libc::c_short, patience: Duration) -> Result<(), String> {
    let mut attente = libc::pollfd {
        fd: fd.as_fd().as_raw_fd(),
        events: evenement,
        revents: 0,
    };
    let ms = libc::c_int::try_from(patience.as_millis()).unwrap_or(libc::c_int::MAX);
    loop {
        // SAFETY : un seul `pollfd`, vivant pendant l'appel, sur un descripteur ouvert.
        let n = unsafe { libc::poll(&mut attente, 1, ms) };
        match n {
            1.. => return Ok(()),
            0 => return Err("l'autre application ne répond plus".into()),
            _ if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted => {}
            _ => return Err(std::io::Error::last_os_error().to_string()),
        }
    }
}

/// **Lit tout ce que l'autre bout envoie**, jusqu'à ce qu'il ferme — au plus `plafond` octets, et
/// jamais plus de [`PATIENCE`] sans un morceau.
pub(super) fn lire_tout(
    mut lecteur: std::io::PipeReader,
    plafond: usize,
) -> Result<Vec<u8>, String> {
    let mut tout = Vec::new();
    let mut morceau = vec![0u8; 64 * 1024];
    loop {
        attendre(&lecteur, libc::POLLIN, PATIENCE)?;
        match lecteur.read(&mut morceau) {
            Ok(0) => return Ok(tout),
            Ok(n) => {
                if tout.len() + n > plafond {
                    return Err(format!("plus de {plafond} octets"));
                }
                tout.extend_from_slice(&morceau[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e.to_string()),
        }
    }
}

/// **Écrit ces octets à celui qui les a demandés**, puis ferme : la fermeture est la fin du
/// transfert. Un lecteur parti fait une erreur (`EPIPE` — Rust ignore `SIGPIPE`), jamais un gel.
pub(super) fn ecrire_tout(fd: std::os::fd::OwnedFd, octets: &[u8]) -> Result<(), String> {
    // Non bloquant : chaque attente passe par `poll`, et se borne.
    // SAFETY : `fcntl` sur un descripteur que nous possédons.
    unsafe {
        let drapeaux = libc::fcntl(fd.as_raw_fd(), libc::F_GETFL);
        libc::fcntl(fd.as_raw_fd(), libc::F_SETFL, drapeaux | libc::O_NONBLOCK);
    }
    let mut fichier = std::fs::File::from(fd);
    let mut reste = octets;
    while !reste.is_empty() {
        attendre(&fichier, libc::POLLOUT, PATIENCE)?;
        match fichier.write(reste) {
            Ok(n) => reste = &reste[n..],
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock
                ) => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Ce qui s'écrit d'un bout se relit de l'autre, à l'octet près**, plus gros qu'un tampon de
    /// tuyau (64 Kio sous Linux) : l'écriture attend la lecture au lieu de se figer ou d'échouer.
    #[test]
    fn test_un_transfert_plus_gros_qu_un_tuyau_arrive_entier() {
        let (lecteur, ecrivain) = std::io::pipe().expect("un tuyau");
        let octets: Vec<u8> = (0..1_000_003u32).map(|i| (i * 7 % 251) as u8).collect();
        let a_ecrire = octets.clone();
        let fil = std::thread::spawn(move || ecrire_tout(ecrivain.into(), &a_ecrire));
        assert_eq!(lire_tout(lecteur, usize::MAX).as_deref(), Ok(&octets[..]));
        assert_eq!(fil.join().expect("le fil"), Ok(()));
    }

    /// **Un transfert plus gros que le plafond s'arrête** au lieu de remplir la mémoire.
    #[test]
    fn test_le_plafond_arrete_la_lecture() {
        let (lecteur, ecrivain) = std::io::pipe().expect("un tuyau");
        let fil = std::thread::spawn(move || ecrire_tout(ecrivain.into(), &[1u8; 300_000]));
        assert!(lire_tout(lecteur, 100_000).is_err());
        let _ = fil.join();
    }

    /// **Un lecteur parti n'arrête pas Glucose** : l'écriture rend une erreur, sans `SIGPIPE`.
    #[test]
    fn test_un_lecteur_parti_rend_une_erreur() {
        let (lecteur, ecrivain) = std::io::pipe().expect("un tuyau");
        drop(lecteur);
        assert!(ecrire_tout(ecrivain.into(), &[1u8; 300_000]).is_err());
    }
}
