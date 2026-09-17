//! Ce que la salissure promet — et surtout ce qu'elle promet de ne jamais faire.

use super::*;

fn rect(left: f64, top: f64, width: f64, height: f64) -> Rect {
    Rect {
        left,
        top,
        width,
        height,
    }
}

fn vue(x: f64, y: f64, scale: f64) -> Viewport {
    Viewport { x, y, scale }
}

#[test]
fn test_par_defaut_tout_est_sale() {
    // La règle de sûreté, et elle passe avant l'efficacité : un état neuf dont personne n'a
    // rien déclaré doit redessiner l'écran, pas le garder tel quel.
    assert_eq!(Salissure::default(), Salissure::Tout);
    assert!(!Salissure::default().est_propre());
}

#[test]
fn test_une_zone_precise_ne_reduit_jamais_une_salissure_existante() {
    // C'est la propriété qui rend l'ordre des déclarations indifférent : un geste qui salit
    // tout, suivi d'un geste qui salit un petit rectangle, doit toujours donner « tout ».
    // Sans elle, il suffirait qu'un appelant précis passe après un appelant vague pour qu'une
    // partie de l'écran reste périmée.
    let precis = rect(10.0, 10.0, 20.0, 20.0);
    assert_eq!(Salissure::Tout.avec(precis), Salissure::Tout);
    assert_eq!(
        Salissure::Tout.union(Salissure::Zone(precis)),
        Salissure::Tout
    );
    assert_eq!(
        Salissure::Zone(precis).union(Salissure::Tout),
        Salissure::Tout
    );
}

#[test]
fn test_deux_zones_donnent_leur_englobant() {
    let a = rect(0.0, 0.0, 10.0, 10.0);
    let b = rect(20.0, 30.0, 10.0, 10.0);
    let Salissure::Zone(u) = Salissure::Zone(a).avec(b) else {
        panic!("deux zones donnent une zone");
    };
    assert_eq!((u.left, u.top), (0.0, 0.0));
    assert_eq!((u.width, u.height), (30.0, 40.0));
}

#[test]
fn test_rien_est_le_neutre_de_l_union() {
    let a = rect(5.0, 5.0, 5.0, 5.0);
    assert_eq!(
        Salissure::Rien.union(Salissure::Zone(a)),
        Salissure::Zone(a)
    );
    assert_eq!(
        Salissure::Zone(a).union(Salissure::Rien),
        Salissure::Zone(a)
    );
    assert_eq!(Salissure::Rien.union(Salissure::Rien), Salissure::Rien);
    assert_eq!(Salissure::Rien.avec(a), Salissure::Zone(a));
}

#[test]
fn test_la_region_couvre_la_zone_marge_comprise() {
    // Un nœud de 100 × 50 posé à l'origine du monde, vue sans décalage ni zoom.
    let s = Salissure::Zone(rect(100.0, 100.0, 100.0, 50.0));
    let region = s
        .region(&vue(0.0, 0.0, 1.0), (1280, 800), 30.0)
        .expect("une zone petite donne une région");

    // La marge s'ajoute des quatre côtés : 70 = 100 − 30, et 230 = 100 + 100 + 30.
    assert_eq!((region.x, region.y), (70, 70));
    assert_eq!((region.largeur, region.hauteur), (160, 110));
}

#[test]
fn test_la_region_suit_la_vue() {
    let s = Salissure::Zone(rect(0.0, 0.0, 100.0, 100.0));
    let sans_marge = |vp| s.region(&vp, (1280, 800), 0.0).expect("région");

    // Déplacer la vue déplace la région d'autant : `world_to_screen` vaut monde × échelle + vp.
    let a = sans_marge(vue(200.0, 150.0, 1.0));
    assert_eq!((a.x, a.y), (200, 150));
    assert_eq!((a.largeur, a.hauteur), (100, 100));

    // Zoomer la grandit dans la même proportion.
    let b = sans_marge(vue(0.0, 0.0, 2.0));
    assert_eq!((b.largeur, b.hauteur), (200, 200));
}

#[test]
fn test_une_region_trop_grande_vaut_toute_la_fenetre() {
    // Rendre à part puis reporter coûte l'aire deux fois : au-delà de la moitié de la
    // fenêtre, le détour coûte plus cher que le rendu complet qu'il remplace. Le seuil est le
    // point où les deux s'égalent, il n'est pas choisi.
    let grande = Salissure::Zone(rect(0.0, 0.0, 1000.0, 700.0));
    assert!(
        grande
            .region(&vue(0.0, 0.0, 1.0), (1280, 800), 0.0)
            .is_none(),
        "une zone qui couvre plus de la moitié doit rendre la fenêtre entière"
    );

    let petite = Salissure::Zone(rect(0.0, 0.0, 300.0, 300.0));
    assert!(petite
        .region(&vue(0.0, 0.0, 1.0), (1280, 800), 0.0)
        .is_some());
}

#[test]
fn test_une_zone_hors_fenetre_ne_donne_rien_a_redessiner() {
    // Un nœud déplacé très loin de la vue : il a changé, mais aucun pixel de l'écran n'en
    // dépend. Redessiner « sa » région reviendrait à redessiner un rectangle vide.
    let loin = Salissure::Zone(rect(-5000.0, -5000.0, 100.0, 100.0));
    let region = loin
        .region(&vue(0.0, 0.0, 1.0), (1280, 800), 30.0)
        .expect("une région, fût-elle vide");
    assert!(region.est_vide());
}

#[test]
fn test_la_region_est_bornee_par_la_fenetre() {
    // Une zone à cheval sur le bord ne doit jamais produire de coordonnées hors du pixmap :
    // ce serait un débordement à l'écriture, pas un simple gaspillage.
    let a_cheval = Salissure::Zone(rect(1200.0, 750.0, 200.0, 200.0));
    let region = a_cheval
        .region(&vue(0.0, 0.0, 1.0), (1280, 800), 30.0)
        .expect("région");
    assert!(region.x + region.largeur <= 1280);
    assert!(region.y + region.hauteur <= 800);
}

#[test]
fn test_la_region_sait_si_elle_touche_la_chrome() {
    // La chrome se place en coordonnées écran : elle ne peut pas se rendre décalée. Savoir si
    // la zone sale la touche décide s'il faut la redessiner entière.
    let haut = Region {
        x: 0,
        y: 0,
        largeur: 100,
        hauteur: 100,
    };
    let bas = Region {
        x: 0,
        y: 400,
        largeur: 100,
        hauteur: 100,
    };
    assert!(haut.touche_le_haut(56.0));
    assert!(!bas.touche_le_haut(56.0));
}
