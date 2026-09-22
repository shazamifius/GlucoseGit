//! Ce que l'arbitre promet : il essaie, il regarde, il garde la meilleure — et il ne peut
//! pas osciller.

use super::*;

/// Un écran à 240 Hz : la fenêtre d'observation vaut alors 240 images.
fn a_240hz(depart: Preference) -> Arbitre {
    Arbitre::nouveau(depart, Duration::from_secs_f64(1.0 / 240.0))
}

/// Un `present` qui tient largement le plancher.
const SAIN: u32 = 1_000;
/// Un `present` qui gèle — quatre cents millisecondes, comme sur le terrain.
const GEL: u32 = 400_000;

/// Joue `combien` images dont `gels` gèlent, et rend le dernier verdict rencontré.
fn jouer(a: &mut Arbitre, combien: u64, gels: u64) -> Verdict {
    let mut dernier = Verdict::Continuer;
    for i in 0..combien {
        let v = a.observer(if i < gels { GEL } else { SAIN });
        if v != Verdict::Continuer {
            dernier = v;
        }
    }
    dernier
}

/// **Une carte qui tient est gardée**, et la décision ne se refait plus.
///
/// C'est la moitié qui empêche l'arbitre de devenir un mécanisme qui s'agite : sur une
/// machine saine il ne doit strictement rien faire, et cesser de se poser la question.
#[test]
fn test_une_carte_qui_tient_est_gardee_et_la_question_est_close() {
    let mut a = a_240hz(Preference::Econome);
    assert_eq!(jouer(&mut a, 240, 0), Verdict::Continuer);
    assert!(a.a_tranche(), "la question doit etre close");
    assert_eq!(a.courante(), Preference::Econome);
}

/// **Une carte qui gèle en fait essayer une autre.**
///
/// Le cas du terrain : sur l'Intel Arc, `present` dépasse le plancher sur bien plus d'une
/// image sur cent, et les douze images les plus lentes d'une session en sont faites.
#[test]
fn test_une_carte_qui_gele_en_fait_essayer_une_autre() {
    let mut a = a_240hz(Preference::Econome);
    // Dix pour cent d'images gelées : dix fois ce que la cadence tolère.
    assert_eq!(jouer(&mut a, 240, 24), Verdict::Essayer(Preference::Rapide));
    assert_eq!(a.courante(), Preference::Rapide, "la carte a change");
    assert!(!a.a_tranche(), "la nouvelle doit encore faire ses preuves");
}

/// **Une seule image sur cent ne suffit pas** : c'est exactement ce que la cadence tolère.
///
/// Sans cette borne, l'arbitre changerait de carte au premier hoquet — et un hoquet, toute
/// machine en a.
#[test]
fn test_la_part_toleree_ne_declenche_rien() {
    let mut a = a_240hz(Preference::Econome);
    // Deux gels sur 240, soit 0,83 % : sous la tolérance.
    assert_eq!(jouer(&mut a, 240, 2), Verdict::Continuer);
    assert!(a.a_tranche());
    assert_eq!(a.courante(), Preference::Econome);
}

/// **Il ne juge pas avant d'avoir regardé une seconde entière.**
///
/// La fenêtre se déduit de la cadence lue : à 240 Hz elle vaut 240 images. Juger sur dix
/// images ferait dépendre la décision du hasard des premières.
#[test]
fn test_il_ne_juge_pas_avant_d_avoir_regarde_une_seconde() {
    let mut a = a_240hz(Preference::Econome);
    assert_eq!(jouer(&mut a, 239, 239), Verdict::Continuer);
    assert!(
        !a.a_tranche(),
        "239 images ne font pas une seconde a 240 Hz"
    );
}

/// **La fenêtre suit l'écran, elle n'est pas choisie.** À 60 Hz, une seconde vaut 60 images.
#[test]
fn test_la_fenetre_suit_la_cadence_de_l_ecran() {
    let mut a = Arbitre::nouveau(Preference::Econome, Duration::from_secs_f64(1.0 / 60.0));
    assert_eq!(jouer(&mut a, 59, 59), Verdict::Continuer);
    assert_eq!(a.observer(GEL), Verdict::Essayer(Preference::Rapide));
}

/// **Si les deux gèlent, on revient à la moins pire — et on ne bouge plus jamais.**
///
/// C'est le garde-fou contre le cercle vicieux : ce dépôt en a écrit cinq, et leur forme est
/// toujours la même. Une carte n'est essayée qu'une fois ; quand il n'en reste plus, la
/// question est close pour de bon, quelle que soit la charge de la machine.
#[test]
fn test_si_les_deux_gelent_on_revient_a_la_moins_pire_et_on_ne_bouge_plus() {
    let mut a = a_240hz(Preference::Econome);
    // L'économe gèle une image sur dix.
    assert_eq!(jouer(&mut a, 240, 24), Verdict::Essayer(Preference::Rapide));
    // La rapide gèle une image sur quatre : pire encore.
    assert_eq!(
        jouer(&mut a, 240, 60),
        Verdict::Revenir(Preference::Econome)
    );
    assert_eq!(a.courante(), Preference::Econome);
    assert!(a.a_tranche());

    // Et mille images de gels de plus ne la font plus bouger.
    assert_eq!(jouer(&mut a, 1_000, 1_000), Verdict::Continuer);
    assert_eq!(a.courante(), Preference::Econome);
}

/// **Si la seconde tient, on la garde** — même si la première gelait moins qu'elle ne le
/// craignait.
#[test]
fn test_si_la_seconde_tient_on_la_garde() {
    let mut a = a_240hz(Preference::Econome);
    jouer(&mut a, 240, 24);
    assert_eq!(jouer(&mut a, 240, 0), Verdict::Continuer);
    assert_eq!(a.courante(), Preference::Rapide);
    assert!(a.a_tranche());
}

/// **Une durée d'image longue pour une autre raison ne dit rien de la carte.**
///
/// L'arbitre lit le poste `present` seul. Une image lente parce qu'une texture se rendait,
/// ou parce que la chrome s'est refaite, n'a aucune raison de faire changer de carte — et
/// confondre les deux ferait basculer sur le premier zoom un peu lourd.
#[test]
fn test_seul_le_poste_present_compte() {
    let mut a = a_240hz(Preference::Econome);
    // `present` reste sain : peu importe ce que le reste de l'image a coûté.
    assert_eq!(jouer(&mut a, 240, 0), Verdict::Continuer);
    assert_eq!(a.courante(), Preference::Econome);
    assert_eq!(a.bilan().gels, 0);
}
