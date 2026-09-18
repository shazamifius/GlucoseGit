//! Ce que le modèle de coût doit garantir avant qu'on lui confie une décision.

use super::*;

#[test]
fn une_machine_qui_n_a_rien_fait_ne_prevoit_rien() {
    let cout = Cout::nouveau();
    assert!(cout.prevoir(Travail::PixelLisse, 1_000_000).is_none());
    assert!(cout.par_unite(Travail::PixelRepris).is_none());
}

/// **On ne dégrade pas ce qu'on n'a pas mesuré.**
#[test]
fn sans_prevision_la_scene_reste_lisse() {
    assert_eq!(finesse_pour(None), Finesse::Lisse);
}

#[test]
fn le_debit_observe_sert_a_prevoir() {
    let mut cout = Cout::nouveau();
    // Un million de pixels en dix millisecondes : dix nanosecondes le pixel.
    cout.observer(Travail::PixelLisse, 1_000_000, Duration::from_millis(10));
    assert_eq!(cout.par_unite(Travail::PixelLisse), Some(10.0));
    assert_eq!(
        cout.prevoir(Travail::PixelLisse, 2_000_000),
        Some(Duration::from_millis(20))
    );
}

/// Les natures ne se mélangent pas : elles diffèrent d'un ordre de grandeur.
#[test]
fn chaque_nature_garde_son_propre_debit() {
    let mut cout = Cout::nouveau();
    cout.observer(Travail::PixelRepris, 1_000_000, Duration::from_millis(1));
    cout.observer(Travail::PixelLisse, 1_000_000, Duration::from_millis(10));

    assert_eq!(cout.par_unite(Travail::PixelRepris), Some(1.0));
    assert_eq!(cout.par_unite(Travail::PixelLisse), Some(10.0));
    assert!(cout.par_unite(Travail::PixelPixelise).is_none());
}

/// Le **dernier** débit l'emporte : une machine qui se bride doit se voir tout de suite.
#[test]
fn le_dernier_debit_remplace_le_precedent() {
    let mut cout = Cout::nouveau();
    cout.observer(Travail::PixelLisse, 1_000_000, Duration::from_millis(5));
    assert_eq!(cout.par_unite(Travail::PixelLisse), Some(5.0));

    // La machine chauffe : le même travail coûte deux fois plus.
    cout.observer(Travail::PixelLisse, 1_000_000, Duration::from_millis(10));
    assert_eq!(
        cout.par_unite(Travail::PixelLisse),
        Some(10.0),
        "le modele doit suivre la machine, pas la moyenner"
    );
}

/// Une mesure sur zéro unité ne mesure que le bruit de l'horloge.
#[test]
fn une_mesure_sur_rien_n_apprend_rien() {
    let mut cout = Cout::nouveau();
    cout.observer(Travail::PixelLisse, 0, Duration::from_millis(5));
    assert!(cout.par_unite(Travail::PixelLisse).is_none());
}

/// **Une prévision partielle sous-estime, et une sous-estimation fait dépasser le budget.**
#[test]
fn une_nature_inconnue_rend_toute_la_prevision_inconnue() {
    let mut cout = Cout::nouveau();
    cout.observer(Travail::PixelRepris, 1_000_000, Duration::from_millis(1));

    let travaux = [(Travail::PixelRepris, 1_000_000), (Travail::PixelLisse, 10)];
    assert!(
        cout.prevoir_tout(&travaux).is_none(),
        "une nature jamais observee doit rendre la prevision entiere inconnue"
    );

    // Mais une nature inconnue de taille NULLE ne gêne pas : elle ne coûtera rien.
    let sans = [(Travail::PixelRepris, 1_000_000), (Travail::PixelLisse, 0)];
    assert_eq!(cout.prevoir_tout(&sans), Some(Duration::from_millis(1)));
}

#[test]
fn la_somme_de_deux_natures_s_additionne() {
    let mut cout = Cout::nouveau();
    cout.observer(Travail::PixelRepris, 1_000_000, Duration::from_millis(1));
    cout.observer(Travail::PixelLisse, 1_000_000, Duration::from_millis(10));

    let travaux = [
        (Travail::PixelRepris, 2_000_000),
        (Travail::PixelLisse, 500_000),
    ];
    assert_eq!(cout.prevoir_tout(&travaux), Some(Duration::from_millis(7)));
}

/// Le budget est un plancher de cadence, donc une borne de durée : au-delà, on pixelise.
#[test]
fn au_dela_du_budget_la_scene_se_pixelise() {
    assert_eq!(finesse_pour(Some(Duration::from_millis(9))), Finesse::Lisse);
    assert_eq!(finesse_pour(Some(BUDGET)), Finesse::Lisse);
    assert_eq!(
        finesse_pour(Some(Duration::from_millis(11))),
        Finesse::Pixelisee
    );
}

/// Cent images par seconde : le budget vaut exactement dix millisecondes.
#[test]
fn le_budget_vaut_cent_images_par_seconde() {
    assert_eq!(BUDGET.as_secs_f64() * 100.0, 1.0);
}
