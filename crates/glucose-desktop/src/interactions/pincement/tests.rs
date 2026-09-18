//! Ce que le pont doit garantir pour qu'on ose s'y fier.
//!
//! L'état lu ici est **global** — c'est le prix d'un crochet que la plateforme appelle sans
//! rien nous passer. Un seul test le manipule donc, de bout en bout, plutôt que plusieurs qui
//! se marcheraient dessus en parallèle.

use super::*;

/// Le décodage complet, sur un message fabriqué à la main.
///
/// Le tampon est construit depuis les **offsets de l'ABI**, pas depuis la structure Rust : si
/// celle-ci dérivait, ce test la prendrait en défaut au lieu de dériver avec elle.
#[cfg(target_os = "windows")]
#[test]
fn le_pont_lit_la_marque_du_systeme_et_rien_d_autre() {
    const MOLETTE_VERTICALE: u32 = 0x020A;
    const DEPLACEMENT_SOURIS: u32 = 0x0200;
    const CONTROLE: usize = 0x0008;

    let avant = marques();

    plateforme::lire(message(MOLETTE_VERTICALE, CONTROLE).as_ptr().cast());
    assert!(zoom_du_systeme(), "le bit de controle est la marque de zoom");
    assert_eq!(marques(), avant + 1, "une marque vue se compte");

    // **La marque se consomme.** Sans cela, un message qui n'est pas passe par le crochet --
    // et Windows a des boucles internes qui en pompent -- heriterait de celle du precedent.
    // Un glissement entier devenait alors un zoom de plusieurs octaves.
    assert!(
        !zoom_du_systeme(),
        "une marque deja lue ne doit plus servir a personne"
    );

    plateforme::lire(message(MOLETTE_VERTICALE, 0).as_ptr().cast());
    assert!(
        !zoom_du_systeme(),
        "un defilement nu ne doit pas heriter de la marque du precedent"
    );
    assert_eq!(marques(), avant + 1, "un defilement nu ne se compte pas");

    // Un message qui n'est pas un défilement ne dit rien du zoom : le laisser écrire
    // effacerait la marque entre le crochet et l'événement qu'elle qualifie.
    plateforme::lire(message(MOLETTE_VERTICALE, CONTROLE).as_ptr().cast());
    plateforme::lire(message(DEPLACEMENT_SOURIS, 0).as_ptr().cast());
    assert!(
        zoom_du_systeme(),
        "seuls les messages de defilement touchent la marque"
    );
}

/// Un `MSG` Win32 en octets, placé selon l'ABI : `hwnd`, puis `message`, puis `wParam`,
/// chacun aligné sur la taille d'un pointeur.
#[cfg(target_os = "windows")]
fn message(code: u32, haut: usize) -> Vec<u8> {
    let mot = size_of::<usize>();
    let mut octets = vec![0u8; 6 * mot];
    octets[mot..mot + 4].copy_from_slice(&code.to_ne_bytes());
    octets[2 * mot..3 * mot].copy_from_slice(&haut.to_ne_bytes());
    octets
}

/// Ailleurs, le pont se tait — et surtout il ne prétend pas avoir vu quoi que ce soit.
#[cfg(not(target_os = "windows"))]
#[test]
fn sans_convention_de_plateforme_le_pont_ne_dit_rien() {
    assert!(!zoom_du_systeme());
    assert_eq!(marques(), 0);
}
