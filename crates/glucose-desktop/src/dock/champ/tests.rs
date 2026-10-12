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
