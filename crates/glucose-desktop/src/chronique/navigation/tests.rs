//! Ce que la mesure de navigation doit garantir pour qu'on puisse s'y fier.

use super::*;

#[test]
fn une_navigation_neuve_n_a_rien_a_dire() {
    let mut nav = Navigation::nouvelle();
    assert_eq!(nav.comptes(), Comptes::default());
    assert_eq!(nav.mesurees(), (0, 0));
    assert!(
        nav.image_presentee().is_none(),
        "une image qui ne montre aucun geste neuf n'a pas de latence"
    );
}

/// **La latence est celle du plus ancien geste non montré, pas du dernier.**
///
/// Une rafale de pavé tactile envoie des dizaines d'événements entre deux images. Mesurer le
/// dernier donnerait une latence quasi nulle et masquerait exactement ce qu'on cherche : le
/// temps que le premier a attendu.
#[test]
fn la_latence_est_celle_du_plus_ancien_geste_en_attente() {
    let mut nav = Navigation::nouvelle();
    nav.evenement(Decision::Pan);
    std::thread::sleep(Duration::from_millis(12));
    nav.evenement(Decision::Pan);

    let latence = nav.image_presentee().expect("deux gestes attendaient");
    assert!(
        latence >= Duration::from_millis(12),
        "la latence {latence:?} devrait remonter au premier geste"
    );
}

/// Une image qui ne montre rien de neuf ne compte pas : la compter tirerait la distribution
/// vers zéro et ferait paraître fluide une application qui ne l'est pas.
#[test]
fn une_image_sans_geste_neuf_ne_compte_pas() {
    let mut nav = Navigation::nouvelle();
    nav.evenement(Decision::Zoom);
    assert!(nav.image_presentee().is_some());
    assert_eq!(nav.mesurees().0, 1);

    assert!(nav.image_presentee().is_none());
    assert_eq!(nav.mesurees().0, 1, "rien de neuf, rien de mesure");
}

#[test]
fn chaque_nature_de_geste_se_compte_a_part() {
    let mut nav = Navigation::nouvelle();
    nav.evenement(Decision::Zoom);
    nav.evenement(Decision::Zoom);
    nav.evenement(Decision::Pan);
    nav.evenement(Decision::CranDeSouris);
    nav.evenement(Decision::Pincement);
    assert_eq!(
        nav.comptes(),
        Comptes {
            pincements: 1,
            zooms: 2,
            pans: 1,
            crans: 1
        }
    );
    assert_eq!(nav.comptes().total(), 5);
}

/// **Un pincement n'est pas un zoom clavier.** Les confondre reviendrait à perdre la seule
/// grandeur qui dise si le geste du pavé tactile arrive jusqu'ici.
#[test]
fn un_pincement_ne_grossit_pas_le_compte_des_zooms_clavier() {
    let mut nav = Navigation::nouvelle();
    nav.evenement(Decision::Pincement);
    assert_eq!(nav.comptes().zooms, 0);
    assert_eq!(nav.comptes().pincements, 1);
}

/// Le centile se lit sur la distribution, comme pour les images.
#[test]
fn le_centile_suit_les_latences_observees() {
    let mut nav = Navigation::nouvelle();
    for _ in 0..100 {
        nav.evenement(Decision::Pan);
        nav.image_presentee();
    }
    let (mesurees, pire) = nav.mesurees();
    assert_eq!(mesurees, 100);
    assert!(nav.centile(0.99) <= pire.max(1) * 2);
}

/// Le découpage en tranches est celui de la chronique : une durée tombe dans la tranche dont
/// elle ne dépasse pas la borne haute.
#[test]
fn une_duree_tombe_dans_une_tranche_qui_la_contient() {
    for us in [1u32, 7, 100, 1_000, 16_666, 500_000] {
        let i = tranche(us);
        assert!(
            borne_haute(i) >= us,
            "{us} us tombe dans une tranche bornee a {}",
            borne_haute(i)
        );
    }
}
