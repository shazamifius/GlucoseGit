//! L'échelle de l'interface (ECHELLE-1, fiche 121) : elle multiplie celle de l'écran, se retient,
//! ne s'offre pas quand elle ne changerait rien, et un souvenir abîmé ne la rend pas géante.

use super::*;
use crate::commandes::Commande;
use crate::persist::disque::tests::{application, dossier};

/// **Un cran multiplie l'échelle de l'écran** — l'interface grandit, l'écran ne change pas, et
/// « 100 % » la rend.
#[test]
fn test_echelle_1_un_cran_multiplie_l_echelle_de_l_ecran() {
    let mut app = application(&dossier("echelle-cran"));
    app.scale_factor = 1.25;
    app.poser_l_echelle();
    let bande = app.ui.header_height();
    assert!(app.executer(Commande::Echelle(Vers::PlusGrand)));
    assert_eq!(app.ui.echelle_choisie, 110);
    assert!(
        (app.ui.scale_factor - 1.375).abs() < 1e-6,
        "{}",
        app.ui.scale_factor
    );
    assert_eq!(app.scale_factor, 1.25, "l'échelle de l'écran ne bouge pas");
    assert!(app.ui.header_height() > bande, "la bande grandit");
    assert!(app.executer(Commande::Echelle(Vers::Cent)));
    assert_eq!(
        (app.ui.echelle_choisie, app.ui.header_height()),
        (100, bande)
    );
}

/// **Le choix se retient** : une autre application du même dossier le retrouve ; l'écran qui
/// change d'échelle le garde.
#[test]
fn test_echelle_1_le_choix_se_retient() {
    let d = dossier("echelle-souvenir");
    let mut app = application(&d);
    app.executer(Commande::Echelle(Vers::PlusPetit));
    app.executer(Commande::Echelle(Vers::PlusPetit));
    assert_eq!(app.ui.echelle_choisie, 80);
    let autre = application(&d);
    assert_eq!(autre.echelle_retenue(), 80);
    app.scale_factor = 2.0;
    app.poser_l_echelle();
    assert!((app.ui.scale_factor - 1.6).abs() < 1e-6);
}

/// **Ce qui ne changerait rien ne s'offre pas** : au bout des crans, à 100 % pour « 100 % », et
/// quand l'échelle bornée de l'interface est atteinte (un téléphone à 2,75 n'a pas de 150 %).
#[test]
fn test_echelle_1_ce_qui_ne_changerait_rien_ne_s_offre_pas() {
    let mut app = application(&dossier("echelle-bouts"));
    assert!(
        !app.s_applique(Commande::Echelle(Vers::Cent)),
        "déjà à 100 %"
    );
    app.ui.echelle_choisie = 300;
    app.poser_l_echelle();
    assert!(
        !app.s_applique(Commande::Echelle(Vers::PlusGrand)),
        "au bout des crans"
    );
    assert!(app.s_applique(Commande::Echelle(Vers::PlusPetit)));
    app.ui.echelle_choisie = 50;
    app.poser_l_echelle();
    assert!(
        !app.s_applique(Commande::Echelle(Vers::PlusPetit)),
        "au bout des crans"
    );
    app.scale_factor = 2.75;
    app.ui.echelle_choisie = 150;
    app.poser_l_echelle();
    assert!(
        !app.s_applique(Commande::Echelle(Vers::PlusGrand)),
        "l'échelle bornée est atteinte"
    );
    assert!(!app.executer(Commande::Echelle(Vers::PlusGrand)));
    assert_eq!(app.ui.echelle_choisie, 150);
}

/// **Un souvenir abîmé ne fait pas une interface géante** : ce qui n'est pas un cran vaut 100 %.
#[test]
fn test_echelle_1_un_souvenir_abime_vaut_cent() {
    let d = dossier("echelle-abime");
    let app = application(&d);
    for texte in ["99999", "abc", "", "125.5", "0"] {
        std::fs::create_dir_all(app.souvenir_de_la_fenetre(SOUVENIR).parent().unwrap()).ok();
        std::fs::write(app.souvenir_de_la_fenetre(SOUVENIR), texte).expect("écrire");
        assert_eq!(app.echelle_retenue(), 100, "« {texte} »");
    }
    std::fs::write(app.souvenir_de_la_fenetre(SOUVENIR), "125\n").expect("écrire");
    assert_eq!(app.echelle_retenue(), 125);
}

/// **Leurs identifiants sont stables** : une couche de raccourcis les écrit — le fichier de
/// l'utilisateur, un preset, un fork. Devenues une seule commande paramétrée (fiche 122), elles
/// gardent ceux de la fiche 121.
#[test]
fn test_echelle_1_les_identifiants_ne_changent_pas() {
    for (id, vers) in [
        ("interface.agrandir", Vers::PlusGrand),
        ("interface.reduire", Vers::PlusPetit),
        ("interface.cent", Vers::Cent),
    ] {
        assert_eq!(Commande::de_l_id(id), Some(Commande::Echelle(vers)), "{id}");
    }
}
