//! Ce que l'historique d'une saisie promet : un mot s'annule d'un coup, et jamais deux gestes
//! de nature différente ne se retrouvent dans la même entrée.

use super::*;

fn instant(texte: &str) -> Instant {
    Instant {
        texte: texte.to_string(),
        selection: Selection::at(texte.chars().count()),
    }
}

/// Taper un texte lettre à lettre note une entrée **par mot**, pas par lettre.
///
/// La fiche 05 § 3.6 le demande dans ces termes : « vingt caractères tapés d'affilée, c'est
/// une entrée ». Sans ce regroupement, annuler une phrase demanderait autant de `Ctrl+Z` que
/// de touches, ce qui est exactement ce qu'un traitement de texte ne fait pas.
#[test]
fn test_un_mot_tape_lettre_a_lettre_ne_fait_qu_une_entree() {
    let mut h = Historique::default();
    let mut texte = String::new();
    for c in "Bonjour le monde".chars() {
        h.noter(instant(&texte), Nature::du_texte(&c.to_string()));
        texte.push(c);
    }
    assert_eq!(
        h.profondeur(),
        3,
        "trois mots, trois entrees -- et non seize"
    );
}

/// Un espace reste collé au mot qui le précède : annuler retire un mot **entier**.
#[test]
fn test_l_espace_reste_colle_au_mot_qui_le_precede() {
    let mut h = Historique::default();
    h.noter(instant(""), Nature::Mot);
    h.noter(instant("Bonjour"), Nature::Separateur);
    assert_eq!(h.profondeur(), 1, "le mot et son espace vont ensemble");
    h.noter(instant("Bonjour "), Nature::Mot);
    assert_eq!(h.profondeur(), 2, "le mot suivant ouvre une entree");
}

/// Écrire puis effacer puis réécrire fait **trois** entrées : la nature change deux fois.
#[test]
fn test_changer_de_nature_coupe_l_entree() {
    let mut h = Historique::default();
    h.noter(instant("ab"), Nature::Mot);
    h.noter(instant("abc"), Nature::Mot);
    assert_eq!(h.profondeur(), 1);
    h.noter(instant("abcd"), Nature::Effacement);
    assert_eq!(h.profondeur(), 2, "effacer n'est pas ecrire");
    h.noter(instant("abc"), Nature::Mot);
    assert_eq!(h.profondeur(), 3, "et reecrire n'est pas effacer");
}

/// Un collage ne se groupe avec rien, ni avant ni après.
#[test]
fn test_un_geste_isole_ne_se_groupe_avec_rien() {
    let mut h = Historique::default();
    h.noter(instant("a"), Nature::Mot);
    h.noter(instant("ab"), Nature::Isole);
    h.noter(instant("ab-colle"), Nature::Mot);
    assert_eq!(h.profondeur(), 3);
}

/// **Annuler puis rétablir revient exactement au même texte, curseur compris.**
///
/// Le curseur en fait partie : le reposer ailleurs ferait reprendre la frappe suivante au
/// mauvais endroit, donc l'annulation créerait un défaut au lieu d'en réparer un.
#[test]
fn test_annuler_puis_retablir_revient_au_meme_etat() {
    let mut h = Historique::default();
    h.noter(instant("Bonjour "), Nature::Mot);
    let courant = instant("Bonjour le");

    let annule = h.annuler(courant.clone()).expect("une entree a annuler");
    assert_eq!(annule, instant("Bonjour "));

    let retabli = h.retablir(annule).expect("une entree a retablir");
    assert_eq!(retabli, courant, "le texte ET le curseur reviennent");
}

/// Une frappe après une annulation **referme l'avenir** : on ne rétablit plus une branche
/// qu'on vient de quitter. C'est ce que fait tout éditeur, et l'ignorer ferait réapparaître
/// du texte qu'on avait remplacé.
#[test]
fn test_une_frappe_apres_une_annulation_referme_l_avenir() {
    let mut h = Historique::default();
    h.noter(instant("ab"), Nature::Mot);
    let apres = h.annuler(instant("abc")).expect("une entree");
    h.noter(apres, Nature::Mot);
    assert!(
        h.retablir(instant("abX")).is_none(),
        "l'avenir devait etre referme"
    );
}

/// Sans rien à annuler, on ne rend rien — et surtout on ne panique pas.
#[test]
fn test_un_historique_vide_n_annule_rien() {
    let mut h = Historique::default();
    assert!(h.annuler(instant("a")).is_none());
    assert!(h.retablir(instant("a")).is_none());
}
