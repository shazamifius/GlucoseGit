//! Ce que la cadence garantit, quel que soit l'écran qu'on lui donne.

use super::*;

/// L'écran de 240 Hz sur lequel le défaut a été constaté donne bien 4,16 ms.
///
/// C'est le chiffre que tout le projet ignorait en raisonnant sur « 100 fps, donc 10 ms ».
#[test]
fn test_a_240_hz_screen_gives_a_4_16_ms_period() {
    let c = Cadence::depuis_millihertz(240_000);
    assert!((c.fps() - 240.0).abs() < 0.001);
    let ms = c.periode().as_secs_f64() * 1000.0;
    assert!((ms - 4.1667).abs() < 0.001, "période de {ms} ms");
}

/// Un écran muet ne fait pas viser au hasard : on retombe sur le plancher de la charte.
#[test]
fn test_a_silent_screen_falls_back_to_the_charter_floor() {
    for muet in [Cadence::depuis_millihertz(0), Cadence::inconnue()] {
        assert!((muet.fps() - FPS_PLANCHER).abs() < 0.001);
    }
}

/// Une image qui a mangé sa période ne laisse rien — et le dit.
///
/// Faire avancer une tranche par-dessus une image déjà en retard ne ferait que creuser le
/// retard, et c'est exactement ce que la règle interdit : le déplacement passe avant.
#[test]
fn test_a_frame_that_ate_its_period_leaves_nothing() {
    let c = Cadence::depuis_millihertz(240_000);
    assert!(c.temps_libre(c.periode()).is_none());
    assert!(c.temps_libre(c.periode() * 3).is_none());
}

/// Le temps libre décroît exactement de ce que l'image a coûté.
#[test]
fn test_the_free_time_shrinks_by_exactly_what_the_frame_cost() {
    let c = Cadence::depuis_millihertz(120_000);
    let court = c.temps_libre(Duration::from_millis(1)).expect("du temps");
    let long = c.temps_libre(Duration::from_millis(3)).expect("du temps");
    let ecart = court.saturating_sub(long);
    assert!(
        (ecart.as_secs_f64() * 1000.0 - 2.0).abs() < 0.001,
        "écart de {ecart:?} pour deux millisecondes de rendu en plus"
    );
}

// ── La tranche de fond ──────────────────────────────────────────────────────

/// **Une image chère ne finance pas un gros chantier.** C'est la rétroaction positive qui a
/// coûté le plus cher sur le terrain : la version d'avant accordait au fond autant que
/// l'image venait de coûter, si bien qu'une image à 72 ms en offrait 72 — et l'image suivante
/// était plus chère encore.
#[test]
fn une_image_chere_ne_finance_plus_un_chantier_plus_cher() {
    let c = Cadence::depuis_millihertz(240_000);
    for cout in [10, 20, 40, 72, 500] {
        let tranche = c.tranche_de_fond(Duration::from_millis(cout));
        assert_eq!(
            tranche,
            Duration::ZERO,
            "une image de {cout} ms a deja mange le plancher : le fond se tait"
        );
    }
}

/// **La tranche décroît quand l'image grossit** — jamais l'inverse. C'est la propriété qui
/// distingue un investissement d'une fuite, et elle vaut sur toute la plage.
#[test]
fn la_tranche_ne_grandit_jamais_avec_le_cout_de_l_image() {
    let c = Cadence::depuis_millihertz(240_000);
    let mut precedente = c.tranche_de_fond(Duration::ZERO);
    for us in (0..12_000).step_by(250) {
        let tranche = c.tranche_de_fond(Duration::from_micros(us));
        assert!(
            tranche <= precedente,
            "a {us} us l'image coute plus et la tranche a grandi : {tranche:?} apres {precedente:?}"
        );
        precedente = tranche;
    }
}

/// **Une image et son fond tiennent ensemble dans le plancher de la charte.** Cent images par
/// seconde, « quoi qu'il se passe » — l'investissement n'a pas le droit de faire descendre en
/// dessous, puisque c'est la cadence qui ne se négocie pas.
#[test]
fn l_image_et_son_fond_tiennent_dans_le_plancher_de_la_charte() {
    for hz in [60_000, 120_000, 240_000, 500_000] {
        let c = Cadence::depuis_millihertz(hz);
        for us in (0..9_000).step_by(500) {
            let rendu = Duration::from_micros(us);
            let total = rendu + c.tranche_de_fond(rendu);
            // Sur un écran lent la période dépasse le plancher : le temps y est réellement
            // libre, et le fond a le droit de le prendre — il ne coûte aucune image.
            let permis = BUDGET_TOTAL.max(c.periode());
            assert!(
                total <= permis,
                "a {hz} mHz, {rendu:?} de rendu donnent {total:?} au total"
            );
        }
    }
}

/// **Sur un écran lent, le vrai temps mort reste disponible.** Une période de 16,7 ms laisse
/// plus que le plancher : le refuser reviendrait à dormir alors qu'il y a du travail.
#[test]
fn un_ecran_lent_offre_son_temps_mort_entier() {
    let c = Cadence::depuis_millihertz(60_000);
    let tranche = c.tranche_de_fond(Duration::from_millis(2));
    assert!(
        tranche > Duration::from_millis(10),
        "16,7 ms de periode moins 2 ms de rendu : {tranche:?}"
    );
}
