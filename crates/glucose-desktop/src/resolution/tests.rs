//! Ce que la résolution adaptative doit garantir — et la garantie qui compte le plus est
//! qu'elle ne peut pas osciller.

use super::*;

const BUDGET: Duration = Duration::from_millis(10);

/// Ce que l'œil tolère, quand ce n'est pas lui qu'on teste.
///
/// Les tests de ce fichier portent sur le **budget** : ce dont on a besoin. Le plafond de la
/// perception — ce qui est licite — a ses propres tests dans [`crate::perception`], et le
/// neutraliser ici garde chaque loi vérifiée pour elle-même.
const SANS_PLAFOND: u32 = u32::MAX;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Une image dont **tout** le coût vient de la scène : le cas d'école, et le seul que
/// l'ancienne version savait lire.
fn scene_seule(duree: Duration) -> Mesure {
    Mesure {
        image: duree,
        scene: duree,
    }
}

/// Une image qui tient dans son budget ne se réduit pas : la netteté est l'état par défaut,
/// pas une récompense.
#[test]
fn une_image_qui_tient_reste_nette() {
    let mut r = Resolution::nette();
    r.observer(scene_seule(ms(4)), BUDGET, true, SANS_PLAFOND);
    assert_eq!(r.facteur(), 1);
    assert!(!r.reduite());
}

/// Cent vingt millisecondes pour dix de budget : il faut diviser la durée par douze, donc la
/// surface par douze, donc le côté par trois et demi — le palier au-dessus est quatre.
#[test]
fn une_image_douze_fois_trop_chere_se_rend_quatre_fois_plus_petite() {
    let mut r = Resolution::nette();
    r.observer(scene_seule(ms(120)), BUDGET, true, SANS_PLAFOND);
    assert_eq!(r.facteur(), 4);
}

/// **La garantie centrale.** Une fois réduite, la scène devient rapide — et c'est précisément
/// là qu'une adaptation naïve rétablirait la pleine résolution, pour la voir redevenir lente.
///
/// On ne retient jamais la durée observée mais ce qu'elle dit du coût à pleine résolution, qui
/// ne dépend pas du facteur. Le point fixe est donc atteint dès la seconde image.
#[test]
fn la_resolution_ne_peut_pas_osciller() {
    let cout_a_pleine_resolution = 120.0_f64;
    let mut r = Resolution::nette();
    let mut vus = Vec::new();

    for _ in 0..12 {
        let f = f64::from(r.facteur());
        // Ce que coûte vraiment une image rendue à ce facteur : la loi de la surface.
        let observe = Duration::from_secs_f64(cout_a_pleine_resolution / 1000.0 / (f * f));
        r.observer(scene_seule(observe), BUDGET, true, SANS_PLAFOND);
        vus.push(r.facteur());
    }

    let stable = vus[vus.len() - 1];
    assert!(
        vus.iter().skip(1).all(|f| *f == stable),
        "le facteur doit se fixer des la seconde image : {vus:?}"
    );
}

/// **Le cercle que le modèle `scène × f²` produisait, et qu'il ne produit plus.**
///
/// Le test au-dessus simule `coût / f²` — c'est-à-dire l'hypothèse même du modèle. Il ne
/// pouvait donc pas la mettre en défaut ; il vérifiait que le calcul est cohérent avec ce
/// qu'on lui donne, ce qui est vrai et sans intérêt.
///
/// La réalité mesurée est autre : **rendre la scène réduite désactive le cache de tuiles**,
/// donc une scène à `f = 4` ne coûte pas le seizième d'une scène nette. Elle coûte souvent
/// DAVANTAGE par pixel, puisqu'elle repeint ce que la grille aurait repris. Multipliée par
/// `f²`, elle paraissait alors gigantesque, et le modèle réduisait encore.
///
/// Sur le terrain : 47 % des images à un facteur moyen 3,42, pendant que la scène nette
/// tenait dans le budget. Ce test rejoue exactement cette forme.
#[test]
fn une_scene_reduite_ne_sert_pas_de_reference_a_sa_propre_reduction() {
    let mut r = Resolution::nette();

    // Une image nette qui tient largement : c'est la référence, et elle vaut 4 ms.
    r.observer(scene_seule(ms(4)), BUDGET, true, SANS_PLAFOND);
    assert_eq!(r.facteur(), 1, "une scene a 4 ms tient dans 10 ms");

    // Une pointe passagère fait réduire -- un pic de tuiles au franchissement d'octave.
    r.observer(scene_seule(ms(40)), BUDGET, true, SANS_PLAFOND);
    let apres_la_pointe = r.facteur();
    assert!(apres_la_pointe > 1, "une scene a 40 ms doit faire reduire");

    // Et maintenant le cas qui compte : reduite, la scene a PERDU les tuiles, donc elle
    // coute encore 12 ms au lieu des 40/f² que l'ancien modele attendait.
    for _ in 0..8 {
        r.observer(scene_seule(ms(12)), BUDGET, true, SANS_PLAFOND);
    }
    assert!(
        r.facteur() <= apres_la_pointe,
        "le facteur s'emballe : {} apres etre monte a {apres_la_pointe}",
        r.facteur()
    );

    // LA PREUVE QUE CE TEST ATTRAPE QUELQUE CHOSE. L'ancien modele lisait la scene reduite
    // comme `9 ms x f²`, et concluait un facteur bien plus haut. On rejoue son calcul.
    let f = f64::from(apres_la_pointe);
    let cru_par_l_ancien = 12.0 * f * f;
    assert!(
        cru_par_l_ancien > 40.0,
        "l'ancien modele voyait {cru_par_l_ancien} ms la ou la scene nette en coute 40 : \
         s'il n'est plus plus pessimiste, le cas ne reproduit plus le cercle"
    );

    // Et la nettete revient entierement des que la main s'arrete.
    for _ in 0..8 {
        r.observer(scene_seule(ms(12)), BUDGET, false, SANS_PLAFOND);
    }
    assert_eq!(r.facteur(), 1, "a l'arret, la scene redevient nette");
}

/// La netteté revient par moitiés : d'un coup, elle rendrait l'image chère juste au moment où
/// l'œil se pose dessus.
#[test]
fn la_nettete_revient_par_moities_et_finit_par_revenir() {
    let mut r = Resolution::nette();
    r.observer(scene_seule(ms(500)), BUDGET, true, SANS_PLAFOND);
    assert_eq!(r.facteur(), 8, "une image cinquante fois trop chere");

    let mut paliers = Vec::new();
    for _ in 0..5 {
        r.observer(scene_seule(ms(1)), BUDGET, false, SANS_PLAFOND);
        paliers.push(r.facteur());
    }
    assert_eq!(paliers, vec![4, 2, 1, 1, 1]);
}

/// Une durée absurde ne doit pas produire un facteur qu'aucun tampon ne pourrait porter.
#[test]
fn une_duree_insensee_retombe_sur_le_dernier_palier() {
    let mut r = Resolution::nette();
    r.observer(
        scene_seule(Duration::from_secs(3600)),
        BUDGET,
        true,
        SANS_PLAFOND,
    );
    assert_eq!(r.facteur(), 8);

    // Un budget nul ne se tient par AUCUN facteur. La bonne reponse n'est donc pas de
    // rapetisser au maximum -- ce serait perdre la nettete sans rien gagner -- mais de rendre
    // net et de laisser la mesure dire ou est vraiment le temps.
    let mut r = Resolution::nette();
    r.observer(scene_seule(ms(120)), Duration::ZERO, true, SANS_PLAFOND);
    assert_eq!(
        r.facteur(),
        1,
        "un budget intenable ne s'achete pas en pixels"
    );
}

/// Le facteur ne quitte jamais les paliers : l'agrandissement au plus proche par un entier
/// donne des blocs exacts, et tout autre valeur rééchantillonnerait pour rien.
#[test]
fn le_facteur_reste_une_puissance_de_deux() {
    let mut r = Resolution::nette();
    for duree in [1u64, 7, 11, 23, 50, 99, 137, 400, 900] {
        r.observer(scene_seule(ms(duree)), BUDGET, true, SANS_PLAFOND);
        assert!(
            PALIERS.contains(&r.facteur()),
            "{duree} ms a donne un facteur hors palier : {}",
            r.facteur()
        );
    }
}

/// **L'erreur qui a rendu l'application illisible, et que rien n'interdisait.**
///
/// Une image passée pour l'essentiel hors de la scène — interface, panneaux, téléversement —
/// ne se répare pas en rapetissant la scène. L'ancienne version appliquait pourtant la loi de
/// la surface à la durée entière : elle divisait une part qui ne bougeait pas, constatait que
/// le budget n'était toujours pas tenu, et divisait encore, jusqu'au dernier palier. Mesuré
/// sur une vraie session : facteur moyen 7,83 sur 98 % des images, pour une cadence *plus
/// basse* qu'avant.
#[test]
fn une_image_dont_le_cout_n_est_pas_dans_la_scene_reste_nette() {
    let mut r = Resolution::nette();
    // Douze millisecondes d'image, dont une seule de scène : le reste ne se divise pas.
    r.observer(
        Mesure {
            image: ms(12),
            scene: ms(1),
        },
        BUDGET,
        true,
        SANS_PLAFOND,
    );
    assert_eq!(
        r.facteur(),
        1,
        "reduire une scene qui ne coute rien abime l'image sans rien gagner"
    );
}

/// Et le verrouillage lui-même : quel que soit le facteur déjà atteint, une image dominée par
/// son terme fixe doit **revenir** à la pleine résolution, pas y rester bloquée.
#[test]
fn un_facteur_deja_haut_redescend_quand_la_scene_n_est_pas_en_cause() {
    let mut r = Resolution::nette();
    r.observer(scene_seule(ms(500)), BUDGET, true, SANS_PLAFOND);
    assert_eq!(r.facteur(), 8, "la scene coutait vraiment, au depart");

    for _ in 0..3 {
        // La scene ne coute plus rien ; le reste de l'image, si.
        r.observer(
            Mesure {
                image: ms(12),
                scene: Duration::from_micros(200),
            },
            BUDGET,
            true,
            SANS_PLAFOND,
        );
    }
    assert_eq!(
        r.facteur(),
        1,
        "le modele doit savoir revenir sur sa decision"
    );
}

/// Le cas mixte : le terme fixe tient dans le budget, et ce qui reste dicte le facteur.
#[test]
fn seul_ce_qui_reste_au_budget_apres_le_fixe_dicte_le_facteur() {
    let mut r = Resolution::nette();
    // 8 ms de fixe sur 10 de budget : il reste 2 ms pour une scene qui en coute 30.
    // Il faut diviser la surface par quinze, donc le cote par ~3,9 : le palier est quatre.
    r.observer(
        Mesure {
            image: ms(38),
            scene: ms(30),
        },
        BUDGET,
        true,
        SANS_PLAFOND,
    );
    assert_eq!(r.facteur(), 4);
}

/// **Le budget qu'on donne à ce module décide de ce que l'utilisateur voit**, et c'est ce
/// qu'un test doit dire avant qu'une capture d'écran ne le dise.
///
/// L'application lui passait la *cible de coût* — deux millisecondes et demie, ce qu'une
/// image vise pour laisser du temps au travail de fond. La réduction se déclenchait donc dès
/// qu'une image dépassait 2,5 ms, c'est-à-dire presque toujours : 22 % des images d'une
/// session réelle rendues à facteur 1,98, en gros blocs illisibles, sans que la cadence y
/// gagne quoi que ce soit.
///
/// Le seuil légitime est le **plancher de la charte**. Ce test le verrouille par sa
/// conséquence : une scène qui tient largement dans le plancher reste nette, quand bien même
/// elle dépasse la cible.
#[test]
fn une_scene_qui_tient_dans_le_plancher_reste_nette_meme_au_dela_de_la_cible() {
    let plancher = crate::cadence::BUDGET_TOTAL;
    let cible = Duration::from_micros(2_500);
    assert!(cible < plancher, "la cible est plus serree que le plancher");

    let mut r = Resolution::nette();
    // Six millisecondes de scène : deux fois la cible, mais bien en deçà du plancher.
    for _ in 0..10 {
        r.observer(scene_seule(ms(6)), plancher, true, SANS_PLAFOND);
    }
    assert_eq!(
        r.facteur(),
        1,
        "une image de 6 ms tient dans les dix du plancher : rien ne justifie de l'abimer"
    );

    // Et la preuve que ce n'était pas une insensibilité du module : avec l'ancien budget,
    // la même scène part en morceaux.
    let mut avec_la_cible = Resolution::nette();
    avec_la_cible.observer(scene_seule(ms(6)), cible, true, SANS_PLAFOND);
    assert!(
        avec_la_cible.facteur() > 1,
        "c'est bien le budget qui decidait, et non la scene"
    );
}
