//! Ce que la présentation note de chaque image, par le vrai chemin de l'application.

use super::*;

/// **Une image en retard sur le tempo est comptée comme ratée par la chronique** (TEMPO-2) —
/// par le vrai chemin : le tempo juge, la présentation le note, le relevé le recopie.
///
/// Les épreuves du tempo et de la chronique ne le prouvent pas à elles deux : un relevé qui
/// recopierait zéro les passait toutes, et le rapport aurait dit « aucune image n'a raté
/// son balayage » sur une session à trente images par seconde. C'est le sabotage qui l'a
/// montré.
#[test]
fn test_une_image_en_retard_est_comptee_ratee_par_la_chronique() {
    let mut app = GlucoseApp::new();
    let periode = std::time::Duration::from_millis(1);
    app.tempo.accorder(periode);
    // Le tempo ne règle que ce qui bouge : un élan en cours.
    app.elan
        .pousser_pan(10_000.0, 0.0, std::time::Instant::now());
    // La première image pose la grille.
    crate::perf::frame_begin();
    app.attendre_l_heure_de_soumettre();
    app.enregistrer_l_image(1_000, (800, 600));
    assert!(
        app.chronique.ratees().is_none(),
        "la premiere image n'a rien rate"
    );
    // La seconde arrive dix périodes plus tard, bien après sa cible.
    std::thread::sleep(periode * 10);
    crate::perf::frame_begin();
    app.attendre_l_heure_de_soumettre();
    app.enregistrer_l_image(1_000, (800, 600));
    let (ratees, _) = app.chronique.ratees().expect("une image ratee");
    assert_eq!(ratees.compte(), 1);
}
