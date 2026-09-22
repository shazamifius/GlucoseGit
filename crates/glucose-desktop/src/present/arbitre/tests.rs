//! Ce que l'arbitre promet : il essaie, il regarde, il garde la meilleure — et il ne peut
//! pas osciller.
//!
//! La moitié la plus utile de ce fichier rejoue **la session de terrain du 22/09 au soir**,
//! celle où l'arbitre n'a rien fait pendant que la carte gelait quatre fois. Les deux défauts
//! y sont séparés, et chacun est montré en train de conclure l'inverse.

use super::*;
use crate::cadence::ECHANTILLON;

/// Un `present` qui tient largement le plancher — la médiane du terrain au repos.
const SAIN: u32 = 1_020;
/// Le plancher de la charte, en microsecondes.
const PLANCHER: u32 = 10_000;

/// Un arbitre qui a fini son échauffement : c'est dans ce régime que tout se joue.
fn chauffe(depart: Preference) -> Arbitre {
    let mut a = Arbitre::nouveau(depart);
    for _ in 0..ECHANTILLON {
        assert_eq!(a.observer(SAIN), Verdict::Continuer);
    }
    a
}

/// Joue `combien` images saines, et rend le dernier verdict rencontré.
fn saines(a: &mut Arbitre, combien: u32) -> Verdict {
    let mut dernier = Verdict::Continuer;
    for _ in 0..combien {
        let v = a.observer(SAIN);
        if v != Verdict::Continuer {
            dernier = v;
        }
    }
    dernier
}

// --------------------------------------------------------------------------------------
// La session du terrain, rejouée
// --------------------------------------------------------------------------------------

/// Quarante-huit secondes, 2 546 images, et les quatre gels de `present` que la chronique a
/// retenus, chacun à sa seconde.
///
/// **C'est un minorant, et c'est voulu** : la table des images lentes n'en garde que douze, et
/// la fiche 17 § 3.2 rappelle que c'est un échantillon biaisé. Il a pu y avoir d'autres gels
/// entre dix et cent cinquante millisecondes que personne n'a vus. Une loi qui bascule sur ces
/// quatre-là basculera a fortiori sur davantage.
const IMAGES: u32 = 2_546;
const GELS: [(u32, u32); 4] = [
    // (image, ce que `present` a coûté en µs) — l'image se déduit de la seconde : t / 48 s.
    (833, 194_210),   // 15,7 s, au repos
    (1_204, 156_120), // 22,7 s, en déplaçant la vue
    (1_437, 478_760), // 27,1 s, en déplaçant la vue
    (1_782, 221_710), // 33,6 s, en déplaçant la vue
];

/// Ce que `present` a coûté à l'image `i` de cette session.
fn terrain(i: u32) -> u32 {
    GELS.iter()
        .find_map(|&(quand, us)| (quand == i).then_some(us))
        .unwrap_or(SAIN)
}

/// **La session où l'arbitre n'a rien fait le fait maintenant basculer**, et dès le premier
/// gel.
///
/// Le premier gel tombe à la 833ᵉ image : 194,21 ms, soit 18,4 images entièrement perdues, là
/// où la tolérance en permet deux par échantillon. La conclusion est acquise sans attendre la
/// fin de l'échantillon — attendre ne ferait que prolonger les gels.
#[test]
fn test_la_session_du_terrain_fait_basculer_des_le_premier_gel() {
    let mut a = Arbitre::nouveau(Preference::Econome);
    let mut bascule = None;
    for i in 0..IMAGES {
        if a.observer(terrain(i)) == Verdict::Essayer(Preference::Rapide) {
            bascule = Some(i);
            break;
        }
    }
    assert_eq!(
        bascule,
        Some(833),
        "l'arbitre doit essayer l'autre carte au premier gel, pas au quatrieme"
    );
    assert_eq!(a.courante(), Preference::Rapide);
}

/// **Premier défaut, rejoué : compter des images gelées ne pouvait pas déclencher.**
///
/// L'ancienne loi comptait combien d'images dépassaient le plancher dans `present`. Sur cette
/// session : quatre sur 2 546, soit **0,157 %**, sous le pour cent toléré. La nouvelle grandeur
/// — le temps perdu ramené en images — en donne **3,97 %**, vingt-cinq fois plus.
///
/// Les deux nombres sont calculés ici sur les mêmes données, et c'est tout l'écart entre une
/// loi aveugle et une loi qui voit.
#[test]
fn test_l_ancienne_grandeur_concluait_l_inverse_sur_les_memes_images() {
    let gelees = GELS.len() as f64;
    let part_ancienne = gelees / f64::from(IMAGES);
    assert!(
        part_ancienne <= crate::cadence::PART_TOLEREE,
        "l'ancienne loi tolerait ces gels : {:.3} % <= {:.0} %",
        part_ancienne * 100.0,
        crate::cadence::PART_TOLEREE * 100.0
    );

    let mut bilan = Bilan::default();
    for i in 0..IMAGES {
        bilan.noter(terrain(i));
    }
    let part = bilan.part_perdue();
    assert!(
        part > crate::cadence::PART_TOLEREE * 3.0,
        "le temps perdu doit depasser franchement la tolerance, et il vaut {:.2} %",
        part * 100.0
    );
    // 101 images perdues sur 2 546 : le chiffre exact, pour qu'une dérive se voie.
    assert!(
        (bilan.images_perdues() - 101.1).abs() < 0.5,
        "images perdues : {:.1}",
        bilan.images_perdues()
    );
}

/// **Second défaut, rejoué : la porte se fermait douze secondes avant le premier gel.**
///
/// L'ancienne loi jugeait une fois, au bout d'une seconde d'écran — 240 images à 240 Hz — puis
/// déclarait la question close. Le premier gel de la session tombe à la 833ᵉ image. **Même
/// avec la bonne grandeur, l'arbitre n'aurait rien vu**, et c'est ce défaut-là qui décidait.
#[test]
fn test_l_ancienne_fenetre_se_fermait_avant_le_premier_gel() {
    const ANCIENNE_FENETRE: u32 = 240; // une seconde d'un ecran a 240 Hz
    let premier_gel = GELS[0].0;
    assert!(
        ANCIENNE_FENETRE < premier_gel,
        "la porte se fermait a l'image {ANCIENNE_FENETRE}, le premier gel tombe a la {premier_gel}"
    );

    // Et l'arbitre d'aujourd'hui, lui, regarde encore à cette image-là.
    let mut a = chauffe(Preference::Econome);
    saines(&mut a, premier_gel);
    assert!(
        !a.a_tranche(),
        "la question ne doit pas etre close tant qu'une carte reste a essayer"
    );
    assert_eq!(a.observer(GELS[0].1), Verdict::Essayer(Preference::Rapide));
}

// --------------------------------------------------------------------------------------
// Les propriétés
// --------------------------------------------------------------------------------------

/// **Une carte qui tient ne fait jamais rien bouger**, même après dix mille images.
///
/// La version précédente prouvait « la question est close » au bout d'une seconde ; c'est
/// précisément ce qui l'a rendue aveugle. La promesse utile est plus forte : sur une machine
/// saine, l'arbitre ne bascule **jamais**, quelle que soit la durée de la session.
#[test]
fn test_une_carte_qui_tient_ne_fait_jamais_rien() {
    let mut a = Arbitre::nouveau(Preference::Econome);
    assert_eq!(saines(&mut a, 10_000), Verdict::Continuer);
    assert_eq!(a.courante(), Preference::Econome);
    assert_eq!(a.bilan().perdu_us, 0);
}

/// **Un gel de 194 ms suffit**, parce qu'il vaut dix-huit images entières.
#[test]
fn test_un_seul_gel_franc_suffit() {
    let mut a = chauffe(Preference::Econome);
    assert_eq!(a.observer(194_210), Verdict::Essayer(Preference::Rapide));
}

/// **Ce que la tolérance permet ne déclenche rien** : deux images perdues par échantillon.
///
/// Sans cette borne, l'arbitre changerait de carte au premier hoquet — et un hoquet, toute
/// machine en a.
#[test]
fn test_ce_que_la_tolerance_permet_ne_declenche_rien() {
    let mut a = chauffe(Preference::Econome);
    // Deux images perdues tout rond : le plancher, deux fois de trop.
    assert_eq!(a.observer(PLANCHER * 2), Verdict::Continuer);
    assert_eq!(a.observer(PLANCHER * 2), Verdict::Continuer);
    assert_eq!(a.courante(), Preference::Econome);
    // La troisième passe au-dessus, et là seulement il bouge.
    assert_eq!(
        a.observer(PLANCHER * 2),
        Verdict::Essayer(Preference::Rapide)
    );
}

/// **L'échantillon se renouvelle** : des pertes étalées sur toute une session ne s'additionnent
/// pas jusqu'à déclencher.
///
/// Une image perdue toutes les cent images est exactement la tolérance ; mille images de ce
/// régime ne doivent rien faire bouger, alors qu'un cumul sans renouvellement en compterait dix
/// et basculerait.
#[test]
fn test_l_echantillon_se_renouvelle_et_la_tolerance_reste_une_part() {
    let mut a = chauffe(Preference::Econome);
    for i in 0..1_000 {
        let v = a.observer(if i % 100 == 0 { PLANCHER * 2 } else { SAIN });
        assert_eq!(v, Verdict::Continuer, "a l'image {i}");
    }
    assert_eq!(a.courante(), Preference::Econome);
}

/// **Le démarrage ne compte pas** : un pilote qui s'initialise ne dit rien de sa carte.
///
/// La fiche 22 § 6 mesure `present` à 30 ms sur les seize premières images. Sans échauffement,
/// toute machine du monde basculerait au lancement — ce qui est exactement le choix par
/// étiquette que la charte refuse, sous un autre nom.
#[test]
fn test_le_demarrage_ne_fait_pas_basculer() {
    let mut a = Arbitre::nouveau(Preference::Econome);
    for _ in 0..16 {
        assert_eq!(a.observer(30_090), Verdict::Continuer);
    }
    assert_eq!(saines(&mut a, ECHANTILLON), Verdict::Continuer);
    assert_eq!(a.courante(), Preference::Econome);
    assert_eq!(
        a.bilan().perdu_us,
        0,
        "les images du demarrage ne doivent pas etre au bilan de la carte"
    );
}

/// **Si les deux gèlent, on revient à la moins pire — et on ne bouge plus jamais.**
///
/// C'est le garde-fou contre le cercle vicieux : ce dépôt en a écrit cinq, et leur forme est
/// toujours la même. Une carte n'est essayée qu'une fois ; quand il n'en reste plus, la
/// question est close pour de bon, quelle que soit la charge de la machine.
#[test]
fn test_si_les_deux_gelent_on_revient_a_la_moins_pire_et_on_ne_bouge_plus() {
    let mut a = chauffe(Preference::Econome);
    // L'économe perd une image sur cinquante : au-dessus de la tolérance, sans plus.
    for i in 0..1_000 {
        if a.observer(if i % 50 == 0 { PLANCHER * 2 } else { SAIN }) != Verdict::Continuer {
            break;
        }
    }
    assert_eq!(
        a.courante(),
        Preference::Rapide,
        "l'autre doit etre essayee"
    );

    // La rapide gèle franchement : bien pire.
    for _ in 0..ECHANTILLON {
        a.observer(SAIN);
    }
    assert_eq!(a.observer(478_760), Verdict::Revenir(Preference::Econome));
    assert_eq!(a.courante(), Preference::Econome);
    assert!(a.a_tranche());

    // Et mille images de gels de plus ne la font plus bouger.
    for _ in 0..1_000 {
        assert_eq!(a.observer(478_760), Verdict::Continuer);
    }
    assert_eq!(a.courante(), Preference::Econome);
}

/// **Si la seconde tient, on la garde** — et elle a droit au même échauffement, parce que
/// rouvrir une carte lui fait tout reconstruire.
#[test]
fn test_si_la_seconde_tient_on_la_garde() {
    let mut a = chauffe(Preference::Econome);
    assert_eq!(a.observer(478_760), Verdict::Essayer(Preference::Rapide));
    assert_eq!(saines(&mut a, 10_000), Verdict::Continuer);
    assert_eq!(a.courante(), Preference::Rapide);
    assert!(
        !a.a_tranche(),
        "rien ne l'oblige a clore : il observe encore"
    );
}

/// **Une durée d'image longue pour une autre raison ne dit rien de la carte.**
///
/// L'arbitre lit le poste `present` seul. Une image lente parce qu'une texture se rendait — la
/// session du terrain en porte une à 226 ms — n'a aucune raison de faire changer de carte.
#[test]
fn test_seul_le_poste_present_compte() {
    let mut a = chauffe(Preference::Econome);
    assert_eq!(saines(&mut a, 1_000), Verdict::Continuer);
    assert_eq!(a.courante(), Preference::Econome);
    assert_eq!(a.bilan().perdu_us, 0);
}

/// **Le bilan compare des parts, pas des comptes** : deux cartes observées inégalement se
/// départagent quand même.
#[test]
fn test_deux_cartes_se_departagent_sur_la_part_et_non_sur_le_compte() {
    let peu_vue = Bilan {
        observees: 100,
        perdu_us: 100_000, // dix images perdues sur cent : 10 %
    };
    let beaucoup_vue = Bilan {
        observees: 10_000,
        perdu_us: 500_000, // cinquante sur dix mille : 0,5 %
    };
    assert!(beaucoup_vue.vaut_mieux_que(peu_vue));
    assert!(!peu_vue.vaut_mieux_que(beaucoup_vue));
}

/// **L'essai forcé attend l'échauffement**, puis bascule — et une seule fois.
///
/// C'est un instrument : il existe parce que la bascule ne s'était jamais exécutée, et que le
/// jour où elle l'a pu, elle a planté. Un chemin qu'on ne peut emprunter qu'en priant pour un
/// gel est un chemin qu'on ne vérifie jamais.
///
/// Mais il doit vérifier **ce chemin-là**, et pas un autre : il attend donc l'échauffement,
/// comme la loi qu'il remplace.
#[test]
fn test_l_essai_force_attend_l_echauffement_puis_bascule_une_seule_fois() {
    let mut a = Arbitre::nouveau(Preference::Econome);
    a.essai_force = true;
    // Pendant l'échauffement, il ne fait rien : l'application se charge encore, et basculer
    // là mettrait le chemin dans un état qu'aucune bascule réelle ne rencontre.
    assert_eq!(saines(&mut a, ECHANTILLON), Verdict::Continuer);
    assert_eq!(a.courante(), Preference::Econome);
    assert_eq!(a.observer(SAIN), Verdict::Essayer(Preference::Rapide));
    assert_eq!(a.courante(), Preference::Rapide);
    // Et il ne se redéclenche pas : la carte suivante fait ses preuves normalement.
    assert_eq!(saines(&mut a, 10_000), Verdict::Continuer);
    assert_eq!(a.courante(), Preference::Rapide);
}
