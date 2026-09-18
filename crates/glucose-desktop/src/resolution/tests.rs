//! Ce que la résolution adaptative doit garantir — et la garantie qui compte le plus est
//! qu'elle ne peut pas osciller.

use super::*;

const BUDGET: Duration = Duration::from_millis(10);

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Une image qui tient dans son budget ne se réduit pas : la netteté est l'état par défaut,
/// pas une récompense.
#[test]
fn une_image_qui_tient_reste_nette() {
    let mut r = Resolution::nette();
    r.observer(ms(4), BUDGET, true);
    assert_eq!(r.facteur(), 1);
    assert!(!r.reduite());
}

/// Cent vingt millisecondes pour dix de budget : il faut diviser la durée par douze, donc la
/// surface par douze, donc le côté par trois et demi — le palier au-dessus est quatre.
#[test]
fn une_image_douze_fois_trop_chere_se_rend_quatre_fois_plus_petite() {
    let mut r = Resolution::nette();
    r.observer(ms(120), BUDGET, true);
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
        r.observer(observe, BUDGET, true);
        vus.push(r.facteur());
    }

    let stable = vus[vus.len() - 1];
    assert!(
        vus.iter().skip(1).all(|f| *f == stable),
        "le facteur doit se fixer des la seconde image : {vus:?}"
    );
}

/// La netteté revient par moitiés : d'un coup, elle rendrait l'image chère juste au moment où
/// l'œil se pose dessus.
#[test]
fn la_nettete_revient_par_moities_et_finit_par_revenir() {
    let mut r = Resolution::nette();
    r.observer(ms(500), BUDGET, true);
    assert_eq!(r.facteur(), 8, "une image cinquante fois trop chere");

    let mut paliers = Vec::new();
    for _ in 0..5 {
        r.observer(ms(1), BUDGET, false);
        paliers.push(r.facteur());
    }
    assert_eq!(paliers, vec![4, 2, 1, 1, 1]);
}

/// Une durée absurde ne doit pas produire un facteur qu'aucun tampon ne pourrait porter.
#[test]
fn une_duree_insensee_retombe_sur_le_dernier_palier() {
    let mut r = Resolution::nette();
    r.observer(Duration::from_secs(3600), BUDGET, true);
    assert_eq!(r.facteur(), 8);

    let mut r = Resolution::nette();
    r.observer(ms(120), Duration::ZERO, true);
    assert_eq!(r.facteur(), 8, "un budget nul ne divise pas par zero");
}

/// Le facteur ne quitte jamais les paliers : l'agrandissement au plus proche par un entier
/// donne des blocs exacts, et tout autre valeur rééchantillonnerait pour rien.
#[test]
fn le_facteur_reste_une_puissance_de_deux() {
    let mut r = Resolution::nette();
    for duree in [1u64, 7, 11, 23, 50, 99, 137, 400, 900] {
        r.observer(ms(duree), BUDGET, true);
        assert!(
            PALIERS.contains(&r.facteur()),
            "{duree} ms a donne un facteur hors palier : {}",
            r.facteur()
        );
    }
}
