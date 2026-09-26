//! Les chemins de KaTeX, lus.

use super::*;
use Commande::*;

/// **Tous les chemins nommés de KaTeX se lisent** : pas un ne tombe hors du vocabulaire, et
/// chacun commence par se poser quelque part. Une forme illisible ne se dessinerait pas — c'est
/// ici qu'on apprend qu'il en existe une.
#[test]
fn test_tous_les_chemins_de_katex_se_lisent() {
    let mut lus = 0;
    for (nom, d) in katex::svg_geometry::PATH_MAP.entries() {
        let commandes = lire(d).unwrap_or_else(|| panic!("le chemin {nom} ne se lit pas"));
        assert!(
            matches!(commandes.first(), Some(Aller(..))),
            "{nom} commence par se poser"
        );
        assert!(commandes.len() > 2, "{nom} trace quelque chose");
        lus += 1;
    }
    assert!(lus > 50, "le jeu de KaTeX est bien là : {lus} chemins");
}

/// Les nombres de SVG se collent : un second point, un signe commencent le nombre suivant.
#[test]
fn test_les_nombres_colles_se_separent() {
    assert_eq!(
        lire("M1.5.5L3-4"),
        Some(vec![Aller(1.5, 0.5), Ligne(3.0, -4.0)])
    );
    assert_eq!(lire("M1e-3,2E2"), Some(vec![Aller(0.001, 200.0)]));
    assert_eq!(lire("M-.5-.5"), Some(vec![Aller(-0.5, -0.5)]));
}

/// Les commandes relatives partent du point courant ; `H` et `V` ne bougent qu'un axe ; les
/// paires qui suivent un `M` sont des lignes ; `Z` ramène au départ du sous-chemin.
#[test]
fn test_relatif_horizontal_vertical_et_fermeture() {
    assert_eq!(
        lire("m10 10 5 0h5v5H0V0zl1 1"),
        Some(vec![
            Aller(10.0, 10.0),
            Ligne(15.0, 10.0),
            Ligne(20.0, 10.0),
            Ligne(20.0, 15.0),
            Ligne(0.0, 15.0),
            Ligne(0.0, 0.0),
            Fermer,
            Ligne(11.0, 11.0),
        ])
    );
}

/// `S` reflète le dernier point de contrôle autour du point courant ; sans cubique avant lui,
/// il part du point courant.
#[test]
fn test_la_cubique_lisse_reflete_son_controle() {
    assert_eq!(
        lire("M0 0C1 1 2 1 3 0S5-1 6 0"),
        Some(vec![
            Aller(0.0, 0.0),
            Cubique(1.0, 1.0, 2.0, 1.0, 3.0, 0.0),
            Cubique(4.0, -1.0, 5.0, -1.0, 6.0, 0.0),
        ])
    );
    assert_eq!(
        lire("M1 1s1 1 2 0"),
        Some(vec![Aller(1.0, 1.0), Cubique(1.0, 1.0, 2.0, 2.0, 3.0, 1.0)])
    );
}

/// Hors du vocabulaire, le lecteur refuse plutôt que de deviner.
#[test]
fn test_une_commande_inconnue_est_refusee() {
    assert_eq!(lire("M0 0Q1 1 2 2"), None);
    assert_eq!(lire("M0 0A1 1 0 0 1 2 2"), None);
    assert_eq!(lire("L1 1"), Some(vec![Ligne(1.0, 1.0)]));
    assert_eq!(lire("M0"), None, "une paire incomplète");
}

fn forme(boite_de_vue: (f64, f64, f64, f64), aspect_: &str) -> Forme {
    let (calage, ajustement) = aspect(Some(aspect_));
    Forme {
        commandes: vec![Aller(0.0, 0.0)],
        boite_de_vue,
        calage,
        ajustement,
        epaisseur: None,
    }
}

/// `slice` prend l'échelle la plus grande et cale au coin : un radical de 400 000 unités se
/// rogne à la longueur de sa boîte, à la hauteur exacte de celle-ci.
#[test]
fn test_couvre_prend_la_plus_grande_echelle() {
    let f = forme((0.0, 0.0, 400_000.0, 1000.0), "xMinYMin slice");
    let (sx, sy, dx, dy) = f.transformation((2.0, 1.0));
    assert_eq!((sx, sy, dx, dy), (0.001, 0.001, 0.0, 0.0));
}

/// `meet` prend la plus petite et centre le reste ; `none` étire chaque axe à sa mesure.
#[test]
fn test_contient_centre_et_etire_deforme() {
    let f = forme((0.0, 0.0, 100.0, 100.0), "xMidYMid meet");
    assert_eq!(f.transformation((200.0, 100.0)), (1.0, 1.0, 50.0, 0.0));
    let f = forme((0.0, 0.0, 100.0, 100.0), "xMaxYMax meet");
    assert_eq!(f.transformation((200.0, 100.0)), (1.0, 1.0, 100.0, 0.0));
    let f = forme((10.0, 0.0, 100.0, 50.0), "none");
    assert_eq!(f.transformation((200.0, 100.0)), (2.0, 2.0, -20.0, 0.0));
    let f = forme((0.0, 0.0, 100.0, 50.0), "none");
    assert_eq!(f.transformation((100.0, 100.0)), (1.0, 2.0, 0.0, 0.0));
}

/// Sans règle écrite, SVG centre et contient.
#[test]
fn test_l_aspect_par_defaut() {
    assert_eq!(
        aspect(None),
        ((Calage::Mid, Calage::Mid), Ajustement::Contient)
    );
    assert_eq!(
        aspect(Some("xMinYMax slice")),
        ((Calage::Min, Calage::Max), Ajustement::Couvre)
    );
    assert_eq!(
        boite_de_vue(Some("0 0 400000 1080")),
        Some((0.0, 0.0, 400_000.0, 1080.0))
    );
    assert_eq!(boite_de_vue(Some("0 0 1")), None);
}

/// Un `<line>` se lit en fractions de sa boîte ; un nombre nu, qui serait en unités de page,
/// est refusé.
#[test]
fn test_le_trait_d_une_rature() {
    let t = trait_de_ligne(["0", "100%", "100%", "0"], 0.046).expect("la rature de \\cancel");
    assert_eq!(t.commandes, vec![Aller(0.0, 1.0), Ligne(1.0, 0.0)]);
    assert_eq!(t.epaisseur, Some(0.046));
    assert_eq!(t.transformation((3.0, 2.0)), (3.0, 2.0, 0.0, 0.0));
    assert!(trait_de_ligne(["0", "5", "100%", "0"], 0.046).is_none());
}

/// L'encre d'une forme est rognée à sa boîte, et un trait déborde de sa demi-épaisseur.
#[test]
fn test_l_etendue_est_rognee_a_la_boite() {
    let mut f = forme((0.0, 0.0, 10.0, 10.0), "none");
    f.commandes = vec![Aller(2.0, 3.0), Ligne(20.0, 4.0), Ligne(5.0, 8.0), Fermer];
    assert_eq!(f.etendue((10.0, 10.0)), Some((2.0, 3.0, 10.0, 8.0)));
    let t = trait_de_ligne(["0", "100%", "100%", "0"], 0.5).expect("trait");
    assert_eq!(t.etendue((4.0, 2.0)), Some((0.0, 0.0, 4.0, 2.0)));
    let t = trait_de_ligne(["50%", "50%", "50%", "50%"], 0.5).expect("point");
    assert_eq!(t.etendue((4.0, 2.0)), Some((1.75, 0.75, 2.25, 1.25)));
}
