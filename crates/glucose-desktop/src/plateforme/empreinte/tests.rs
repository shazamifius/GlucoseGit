//! Ce que le relevé promet là où il existe : des nombres qui bougent dans le bon sens.
//!
//! Il n'y a pas grand-chose à prouver d'un appel système — c'est le système qui répond. Ce
//! qu'on vérifie est ce qui pourrait être **faux sans qu'on le voie** : une unité oubliée, une
//! recomposition de deux moitiés à l'envers, un compteur qui ne bouge jamais. Chacune de ces
//! trois fautes produit un nombre plausible et inutilisable.
//!
//! # Où la preuve se place, et pourquoi elle a dû déménager
//!
//! La première version prouvait l'unité **par une mesure de charge** : une boucle de cent vingt
//! millisecondes, et le compteur devait avoir avancé d'autant. Elle a échoué dans la suite
//! complète, pour une raison qui vaut d'être écrite : `GetProcessTimes` compte **tous les fils
//! du processus**, et `cargo test` en lance seize à la fois. La mesure ne parlait donc pas du
//! travail du test, mais de celui de la suite entière.
//!
//! Un test instable est pire qu'aucun test — la fiche 17 § 5 en nomme un qui a passé trois
//! sessions à échouer au hasard. La preuve de l'unité est donc descendue là où la faute vit :
//! dans la recomposition des deux moitiés, qui est une fonction **pure**. Ce qui reste ici du
//! système ne prouve plus que ce qu'il peut prouver sans ordonnanceur.

use super::*;

/// La plateforme de cette machine sait-elle répondre ? Ailleurs, ces preuves n'ont rien à dire.
fn ici() -> bool {
    cfg!(any(windows, target_os = "linux"))
}

/// **Sur une plateforme qui répond, la mémoire n'est ni nulle ni absurde.**
///
/// Zéro serait un compteur jamais rempli, et un zéro se lit comme une mesure (fiche 17 § 3.1).
/// Les bornes sont larges à dessein : ce test attrape une unité oubliée — des octets lus comme
/// des kibioctets, des pages comptées pour des octets — pas une dérive de quelques mébioctets.
#[test]
fn test_la_memoire_relevee_est_dans_un_ordre_de_grandeur_plausible() {
    let Some(vu) = relever() else {
        assert!(!ici(), "cette plateforme devrait savoir repondre");
        return;
    };
    let mo = vu.memoire_octets / (1024 * 1024);
    assert!(
        (1..=32_768).contains(&mo),
        "un binaire de test occupe entre un mebioctet et trente-deux gibioctets, pas {mo} Mo"
    );
}

/// **Le compteur de processeur avance quand on travaille**, et il ne recule jamais.
///
/// C'est tout ce qu'une mesure de charge peut promettre ici, et c'est déjà quelque chose : un
/// compteur figé — un champ qu'on aurait oublié de lire, une structure mal dimensionnée —
/// donnerait un sommeil parfait en permanence, et la section du rapport annoncerait qu'un
/// logiciel est économe sans jamais l'avoir mesuré.
///
/// **Ce qu'il ne promet pas** : de combien il avance. `GetProcessTimes` compte tous les fils du
/// processus, et la suite de tests en occupe seize.
#[test]
fn test_le_compteur_de_processeur_avance_quand_on_travaille() {
    let Some(avant) = relever() else {
        assert!(!ici(), "cette plateforme devrait savoir repondre");
        return;
    };
    let depart = std::time::Instant::now();
    // Un travail que le compilateur ne peut pas supprimer, sur un seul fil.
    let mut somme = 0u64;
    while depart.elapsed() < std::time::Duration::from_millis(80) {
        somme = somme
            .wrapping_add(somme ^ 0x9E37_79B9_7F4A_7C15)
            .wrapping_add(1);
    }
    assert_ne!(somme, u64::MAX, "le travail ne doit pas etre optimise");
    let apres = relever().expect("le releve a repondu une fois, il repond encore");
    assert!(
        apres.processeur_us > avant.processeur_us,
        "le compteur n'a pas bouge apres quatre-vingts millisecondes de calcul : {} puis {}",
        avant.processeur_us,
        apres.processeur_us
    );
}

/// **Les deux moitiés d'un `FILETIME` se recomposent dans le bon ordre**, et l'unité est la
/// microseconde.
///
/// C'est la faute qui produit un nombre positif, plausible, et faux d'un facteur quatre
/// milliards. **Vérifiée à l'envers** : en inversant `dwHighDateTime` et `dwLowDateTime` dans
/// le relevé, une boucle de cent vingt millisecondes annonçait **536 870 912 000 000 µs de
/// processeur**. Aucun regard posé sur ce nombre seul ne l'aurait démenti.
#[cfg(windows)]
#[test]
fn test_les_deux_moities_d_un_filetime_se_recomposent_dans_le_bon_ordre() {
    use windows::Win32::Foundation::FILETIME;

    // La moitié haute pèse exactement deux puissance trente-deux fois la basse.
    assert_eq!(
        natif::cent_ns(FILETIME {
            dwHighDateTime: 1,
            dwLowDateTime: 0,
        }),
        1u64 << 32,
        "la moitie haute est au mauvais bout"
    );
    assert_eq!(
        natif::cent_ns(FILETIME {
            dwHighDateTime: 0,
            dwLowDateTime: 7,
        }),
        7
    );
    // Et une seconde de processeur, telle que Windows la compte : dix millions d'intervalles
    // de cent nanosecondes.
    let une_seconde = 10_000_000u64;
    assert_eq!(
        natif::cent_ns(FILETIME {
            dwHighDateTime: (une_seconde >> 32) as u32,
            dwLowDateTime: (une_seconde & 0xFFFF_FFFF) as u32,
        }),
        une_seconde
    );
}
