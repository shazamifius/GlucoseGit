//! **Les signaux de fin, sous Linux** (FIN-SESSION-1) — la voie de Linux de [`super`].
//!
//! `SIGTERM` (l'extinction, la déconnexion, `kill`), `SIGINT` (`Ctrl+C` dans un terminal) et
//! `SIGHUP` (le terminal qui se ferme) tuaient Glucose sans qu'il finisse.
//!
//! # La forme : un tuyau
//!
//! Un gestionnaire de signal n'a le droit d'appeler qu'une poignée de fonctions, dont `write` —
//! ni verrou, ni allocation, ni canal. Le nôtre écrit donc un octet, le numéro du signal, dans
//! un tuyau ; un fil le lit, et c'est lui qui décide ([`Demande::recevoir`]) et réveille la
//! boucle. Pas de masque de signaux : il passerait aux processus que Glucose lance (un
//! navigateur ouvert par un lien), qui ne mourraient plus à l'extinction — un gestionnaire, lui,
//! ne survit pas à `exec`.

use super::{Demande, Issue};
use std::sync::atomic::{AtomicI32, Ordering};

/// Le bout du tuyau où le gestionnaire écrit ; `-1` tant qu'il n'existe pas.
static TUYAU: AtomicI32 = AtomicI32::new(-1);

/// Les signaux qui demandent la fin.
const SIGNAUX: [libc::c_int; 3] = [libc::SIGTERM, libc::SIGINT, libc::SIGHUP];

/// **Le gestionnaire** : un octet dans le tuyau, et rien d'autre.
extern "C" fn recu(signal: libc::c_int) {
    let bout = TUYAU.load(Ordering::Relaxed);
    if bout >= 0 {
        let octet = signal as u8;
        // SAFETY : `write` est sûr dans un gestionnaire de signal (POSIX, signal-safety(7)) ;
        // l'octet vit sur la pile le temps de l'appel.
        let _ = unsafe { libc::write(bout, (&octet as *const u8).cast(), 1) };
    }
}

/// **Pose le tuyau, le fil qui le lit, et les gestionnaires.** Rend `None` si le système refuse
/// le tuyau ou le fil : Glucose est alors tué par ces signaux, comme avant.
pub(super) fn ecouter(demande: Demande) -> Option<Demande> {
    let mut bouts = [0 as libc::c_int; 2];
    // SAFETY : `bouts` a la place des deux descripteurs. `O_CLOEXEC` : un processus que Glucose
    // lance n'hérite pas du tuyau.
    if unsafe { libc::pipe2(bouts.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
        return None;
    }
    let [lire, ecrire] = bouts;
    let pour_le_fil = demande.clone();
    std::thread::Builder::new()
        .name("glucose-signaux".into())
        .spawn(move || lire_le_tuyau(lire, &pour_le_fil))
        .ok()?;
    TUYAU.store(ecrire, Ordering::Relaxed);
    for signal in SIGNAUX {
        // SAFETY : une structure mise à zéro est une action valide, que l'on complète ;
        // `SA_RESTART` reprend les appels système que le signal a interrompus ailleurs.
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = recu as extern "C" fn(libc::c_int) as libc::sighandler_t;
            action.sa_flags = libc::SA_RESTART;
            libc::sigemptyset(&mut action.sa_mask);
            libc::sigaction(signal, &action, std::ptr::null_mut());
        }
    }
    Some(demande)
}

/// **Le fil des signaux** : chaque octet lu est un signal reçu.
fn lire_le_tuyau(bout: libc::c_int, demande: &Demande) {
    loop {
        let mut octet = 0u8;
        // SAFETY : un octet à remplir, sur la pile ; le bout reste ouvert toute la vie du
        // processus.
        let lu = unsafe { libc::read(bout, (&mut octet as *mut u8).cast(), 1) };
        if lu == 1 {
            match demande.recevoir(i32::from(octet)) {
                Issue::Finir => demande.reveiller(),
                Issue::Partir(code) => std::process::exit(code),
            }
        } else if lu == 0
            || std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted
        {
            return;
        }
    }
}
