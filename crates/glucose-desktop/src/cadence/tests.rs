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

/// Le budget d'une image reste court même sur un écran lent — c'est ce qui laisse du temps
/// au travail de fond.
///
/// Sur 60 Hz, la période fait 16,7 ms ; viser 16,7 ms de rendu ne laisserait rien pour
/// avancer une tranche, et la moindre action lourde se verrait.
#[test]
fn test_the_render_budget_stays_short_even_on_a_slow_screen() {
    let lent = Cadence::depuis_millihertz(60_000);
    assert_eq!(lent.budget_rendu(), BUDGET_RENDU);
    assert!(lent
        .temps_libre(BUDGET_RENDU)
        .is_some_and(|l| l > Duration::from_millis(13)));
}

/// Sur un écran très rapide, le budget ne prend jamais toute la période.
#[test]
fn test_on_a_very_fast_screen_the_budget_never_eats_the_whole_period() {
    let rapide = Cadence::depuis_millihertz(500_000);
    assert!(rapide.budget_rendu() < rapide.periode());
    assert!(rapide.budget_rendu() < BUDGET_RENDU);
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
