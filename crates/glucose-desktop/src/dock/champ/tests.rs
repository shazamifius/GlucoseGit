//! CHAMP-1 — un champ de nombre : ses bornes, le glisser qui règle, la frappe qui remplace.

use super::*;

/// Un dock dont Ordonner est ouvert.
fn dock() -> DockManager {
    let mut dock = DockManager::new();
    dock.toggle_tab(TabId::Organize);
    dock.toggle_tab(TabId::Storyboard);
    dock
}

/// Un cadre de 250 points de large : la règle du glisser.
fn cadre() -> ScaledRect {
    ScaledRect {
        x: 0.0,
        y: 0.0,
        w: 250.0,
        h: 400.0,
        scale: 1.0,
    }
}

fn caractere(c: &str) -> Key {
    Key::Character(c.into())
}

/// **Ce qui s'écrit est toujours permis** : borné, entier ; ce qui n'est pas un nombre ne
/// change rien.
#[test]
fn test_champ_1_une_valeur_est_bornee_et_entiere() {
    let mut d = dock();
    Champ::OrdonnerTaille.ecrire(&mut d, 3000.0);
    assert_eq!(d.organize.size, 2000.0);
    Champ::OrdonnerTaille.ecrire(&mut d, 12.4);
    assert_eq!(d.organize.size, 50.0);
    Champ::OrdonnerEcart.ecrire(&mut d, 7.6);
    assert_eq!(d.organize.gap, 8.0);
    Champ::OrdonnerEcart.ecrire(&mut d, f64::NAN);
    assert_eq!(d.organize.gap, 8.0);
    Champ::StoryColonnes.ecrire(&mut d, 0.0);
    assert_eq!(d.storyboard.cols, 1, "au moins une colonne");
}

/// **Glisser règle** : sous le jeu de la main, rien ne bouge ; au-delà, la moitié du panneau
/// parcourt la moitié des valeurs, comptée depuis le bord du jeu.
#[test]
fn test_champ_1_glisser_parcourt_les_valeurs() {
    let mut d = dock();
    d.organize.size = 280.0;
    d.tenir_le_champ(Champ::OrdonnerTaille, cadre(), (100.0, 8.0));
    assert!(d.glisser_le_champ(105.0));
    assert_eq!(d.organize.size, 280.0, "dans le jeu, c'est encore un clic");
    assert!(d.glisser_le_champ(108.0 + 125.0));
    // La moitié de 1 950, au pas de 10 : 980 (97,5 pas arrondis à 98).
    assert_eq!(d.organize.size, 1260.0);
    assert!(d.glisser_le_champ(-5000.0));
    assert_eq!(d.organize.size, 50.0, "la borne basse");
    assert!(d.lacher_le_champ());
    assert_eq!(d.champ, None, "glissé : il garde sa valeur, sans frappe");
}

/// **Cliquer ouvre la frappe**, la valeur choisie entière : la première touche la remplace,
/// Entrée valide.
#[test]
fn test_champ_1_cliquer_puis_taper_remplace_la_valeur() {
    let mut d = dock();
    d.tenir_le_champ(Champ::OrdonnerEcart, cadre(), (100.0, 8.0));
    assert!(d.lacher_le_champ());
    assert!(matches!(
        d.champ,
        Some(Saisie::Frappe { entiere: true, .. })
    ));
    for c in ["2", "x", "4"] {
        assert!(
            d.frapper_le_champ(&caractere(c)),
            "toute touche est pour lui"
        );
    }
    assert_eq!(d.organize.gap, 16.0, "rien avant Entrée");
    d.frapper_le_champ(&Key::Named(NamedKey::Enter));
    assert_eq!(d.organize.gap, 24.0, "« x » n'est pas un chiffre");
    assert_eq!(d.champ, None);
    assert!(!d.frapper_le_champ(&caractere("5")), "le clavier est rendu");
}

/// **Échap renonce, les flèches montent d'un pas, Tab passe au champ suivant.**
#[test]
fn test_champ_1_echap_fleches_et_tab() {
    let mut d = dock();
    d.tenir_le_champ(Champ::OrdonnerTaille, cadre(), (0.0, 8.0));
    d.lacher_le_champ();
    d.frapper_le_champ(&caractere("9"));
    d.frapper_le_champ(&Key::Named(NamedKey::Escape));
    assert_eq!(d.organize.size, 280.0, "Échap renonce");

    d.tenir_le_champ(Champ::OrdonnerTaille, cadre(), (0.0, 8.0));
    d.lacher_le_champ();
    d.frapper_le_champ(&Key::Named(NamedKey::ArrowUp));
    d.frapper_le_champ(&Key::Named(NamedKey::Tab));
    assert_eq!(d.organize.size, 290.0, "un pas, validé par Tab");
    // Ordonner est en rangées compactes : pas de colonnes — Tab va à l'espacement.
    assert_eq!(
        d.champ.as_ref().map(Saisie::champ),
        Some(Champ::OrdonnerEcart)
    );
    d.frapper_le_champ(&Key::Named(NamedKey::Tab));
    assert_eq!(
        d.champ.as_ref().map(Saisie::champ),
        Some(Champ::OrdonnerTaille)
    );
}

/// **Le panneau refermé emporte sa frappe** : elle ne garde pas le clavier.
#[test]
fn test_champ_1_un_panneau_ferme_rend_le_clavier() {
    let mut d = dock();
    d.tenir_le_champ(Champ::StoryLargeur, cadre(), (0.0, 8.0));
    d.lacher_le_champ();
    d.toggle_tab(TabId::Storyboard);
    assert!(!d.frapper_le_champ(&caractere("1")));
    assert_eq!(d.champ, None);
}

/// **Les colonnes d'Ordonner n'existent qu'où elles comptent**, et la taille dit ce qu'elle
/// mesure : la hauteur d'une rangée, ou la largeur d'une colonne.
#[test]
fn test_champ_1_les_colonnes_et_le_libelle_suivent_la_disposition() {
    let mut d = dock();
    assert!(!Champ::OrdonnerColonnes.present(&d));
    assert_eq!(Champ::OrdonnerTaille.libelle(&d), "HAUTEUR");
    d.organize.layout = LayoutMode::Grid;
    assert!(Champ::OrdonnerColonnes.present(&d));
    assert_eq!(Champ::OrdonnerTaille.libelle(&d), "LARGEUR");
    assert_eq!(Champ::OrdonnerColonnes.montre(0.0), "auto");
}

/// **Une durée se lit et se tape en secondes** : « 2,5 s », la virgule ou le point ; elle se
/// garde en millisecondes.
#[test]
fn test_une_duree_se_lit_et_se_tape_en_secondes() {
    let c = Champ::CaseDuree;
    assert_eq!(c.montre(2000.0), "2,0 s");
    assert_eq!(c.montre(250.0), "0,25 s");
    assert_eq!(c.montre(14_000.0), "14,0 s");
    assert_eq!(c.a_taper(2500.0), "2,5");
    assert_eq!(c.lu("2,5"), Some(2500.0));
    assert_eq!(c.lu("0.04"), Some(40.0));
    assert_eq!(c.lu("deux"), None);
    assert!(c.admet(',') && c.admet('.') && c.admet('7') && !c.admet('a'));
    assert!(
        !Champ::StoryLargeur.admet(','),
        "la virgule n'est qu'aux durées"
    );
    assert_eq!(
        Champ::StoryLargeur.lu("280"),
        Some(280.0),
        "le reste se lit tel quel"
    );
}

/// **Deux chiffres significatifs** : la même finesse relative partout, sans pas choisi.
#[test]
fn test_une_duree_s_arrondit_a_deux_chiffres_significatifs() {
    for (v, attendu) in [
        (2487.0, 2500.0),
        (2449.0, 2400.0),
        (131.0, 130.0),
        (14_400.0, 14_000.0),
        (99.6, 100.0),
    ] {
        assert_eq!(significatifs(v, 2), attendu, "{v}");
    }
    let mut d = dock();
    Champ::CaseDuree.ecrire(&mut d, 2487.0);
    assert_eq!(d.storyboard.duree_case, 2500.0);
    Champ::CaseDuree.ecrire(&mut d, 20.0);
    assert_eq!(d.storyboard.duree_case, 100.0, "bornée en bas");
}

/// **La règle d'une durée est un rapport** : tout le panneau, du min au max multiplie par
/// `max/min` ; la moitié du panneau, par sa racine ; revenir au point d'appui rend la valeur
/// de départ. À l'envers : la règle linéaire, à cent millisecondes le pas, montait de 2 à 14 s
/// en soixante pixels d'un panneau de 260.
#[test]
fn test_la_regle_d_une_duree_est_logarithmique() {
    let mut d = dock();
    d.storyboard.duree_case = 1000.0;
    let regle = cadre();
    d.tenir_le_champ(Champ::CaseDuree, regle, (100.0, 0.0));
    let b = Champ::CaseDuree.bornes();
    d.glisser_le_champ(100.0 + regle.w / 2.0);
    let attendu = significatifs(1000.0 * (b.max / b.min).sqrt(), 2);
    assert_eq!(
        d.storyboard.duree_case, attendu,
        "un demi-panneau : la racine du rapport"
    );
    d.glisser_le_champ(100.0);
    assert_eq!(d.storyboard.duree_case, 1000.0, "revenue au départ");
    d.glisser_le_champ(100.0 + 60.0 * regle.w / 260.0);
    assert!(
        d.storyboard.duree_case < 5000.0,
        "soixante pixels ne mènent plus à 14 s : {}",
        d.storyboard.duree_case
    );
}

/// **Une flèche, un cran** : un dixième de plus ou de moins pour une durée, arrondi — chaque
/// cran bouge ; un pas pour le reste.
#[test]
fn test_un_cran_de_fleche() {
    let c = Champ::CaseDuree;
    assert_eq!(c.cran(2000.0, 1.0), 2200.0);
    assert_eq!(c.cran(2000.0, -1.0), 1800.0);
    assert_eq!(c.cran(1000.0, -1.0), 910.0);
    assert_eq!(c.cran(60_000.0, 1.0), 60_000.0, "borné");
    assert_eq!(Champ::StoryLargeur.cran(280.0, 1.0), 290.0);
}

/// **Tab ne passe jamais à un champ absent** (fiche 68) : sans case choisie, la durée d'une
/// case n'est pas dans le panneau — Tab, depuis la durée par case, revient à la largeur ; une
/// case choisie, il va à sa durée. Sans cette règle, la frappe s'ouvrirait sur un champ qu'on
/// ne voit pas, et taper réglerait une case qui n'existe pas.
#[test]
fn test_tab_ne_passe_jamais_a_un_champ_absent() {
    use crate::dock::storyboard::lecture::{Choisie, Lecture};
    let tab_depuis_la_duree_par_case = |case: Option<Choisie>| {
        let mut d = dock();
        d.storyboard.contenu.lecture = Some(Lecture {
            champs: Vec::new(),
            duree_ms: 2000,
            case,
        });
        d.tenir_le_champ(Champ::StoryDuree, cadre(), (0.0, 8.0));
        d.lacher_le_champ();
        d.frapper_le_champ(&Key::Named(NamedKey::Tab));
        d.champ.as_ref().map(Saisie::champ)
    };
    assert_eq!(
        tab_depuis_la_duree_par_case(None),
        Some(Champ::StoryLargeur),
        "sans case choisie, la durée d'une case n'est pas là"
    );
    let choisie = Choisie {
        id: "case-1".to_string(),
        numero: 1,
        description: String::new(),
        textes: Vec::new(),
        plan: None,
        duree_ms: None,
    };
    assert_eq!(
        tab_depuis_la_duree_par_case(Some(choisie)),
        Some(Champ::CaseDuree),
        "une case choisie : sa durée suit"
    );
}
