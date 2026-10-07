//! La transformation d'un groupe, autour de son origine ou de celle de chaque nœud.

use super::*;

fn pose(centre: (f64, f64), taille: (f64, f64)) -> Pose {
    Pose {
        centre,
        taille,
        rotation: 0.0,
    }
}

fn proches(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9
}

/// **Origine commune, échelle** : le groupe grandit autour de son ancre — chaque centre
/// s'éloigne d'elle du facteur, chaque taille aussi.
#[test]
fn test_l_origine_commune_ecarte_les_centres() {
    let t = Transformation::Echelle {
        facteur: 2.0,
        ancre: (0.0, 0.0),
    };
    let p = transformer(pose((10.0, 5.0), (4.0, 2.0)), t, Origine::Commune);
    assert_eq!(p.centre, (20.0, 10.0));
    assert_eq!(p.taille, (8.0, 4.0));
}

/// **Origines individuelles, échelle** : chaque nœud grandit sur place.
#[test]
fn test_les_origines_individuelles_gardent_les_centres() {
    let t = Transformation::Echelle {
        facteur: 2.0,
        ancre: (0.0, 0.0),
    };
    let p = transformer(pose((10.0, 5.0), (4.0, 2.0)), t, Origine::Individuelle);
    assert_eq!(p.centre, (10.0, 5.0));
    assert_eq!(p.taille, (8.0, 4.0));
}

/// **Rotation** : en commun, le centre tourne autour du pivot et l'angle s'ajoute ; chacun
/// sur soi, seul l'angle change.
#[test]
fn test_la_rotation_tourne_les_centres_ou_les_garde() {
    let t = Transformation::Rotation {
        angle: std::f64::consts::FRAC_PI_2,
        pivot: (0.0, 0.0),
    };
    let commune = transformer(pose((10.0, 0.0), (4.0, 2.0)), t, Origine::Commune);
    assert!(proches(commune.centre, (0.0, 10.0)), "{:?}", commune.centre);
    assert_eq!(commune.taille, (4.0, 2.0));
    assert!((commune.rotation - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    let seule = transformer(pose((10.0, 0.0), (4.0, 2.0)), t, Origine::Individuelle);
    assert_eq!(seule.centre, (10.0, 0.0));
    assert!((seule.rotation - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
}

/// **Tirer un coin garde le coin opposé immobile**, et le facteur suit le pointeur, rapport
/// gardé : le coin bas-droit tiré de 100 sur un groupe de 200 × 100 double le groupe.
#[test]
fn test_le_coin_tire_garde_le_coin_oppose() {
    let groupe = Rect::new(0.0, 0.0, 200.0, 100.0);
    let t = echelle_du_coin(groupe, Handle::BottomRight, (200.0, 100.0));
    let Transformation::Echelle { facteur, ancre } = t else {
        panic!("une échelle")
    };
    assert!((facteur - 2.0).abs() < 1e-9, "{facteur}");
    assert_eq!(ancre, (0.0, 0.0), "le coin haut-gauche");
    let t = echelle_du_coin(groupe, Handle::TopLeft, (-200.0, -100.0));
    let Transformation::Echelle { ancre, .. } = t else {
        panic!("une échelle")
    };
    assert_eq!(ancre, (200.0, 100.0), "le coin bas-droit");
    // Le coin opposé, transformé, ne bouge pas.
    assert_eq!(deplacer_le_point(t, (200.0, 100.0)), (200.0, 100.0));
}

/// **Le groupe ne s'écrase pas** : réduire au-delà de rien s'arrête à la taille minimale.
#[test]
fn test_le_groupe_ne_s_ecrase_pas() {
    let groupe = Rect::new(0.0, 0.0, 200.0, 100.0);
    let Transformation::Echelle { facteur, .. } =
        echelle_du_coin(groupe, Handle::BottomRight, (-500.0, -500.0))
    else {
        panic!("une échelle")
    };
    assert!(
        facteur > 0.0 && facteur * 100.0 >= MIN_IMAGE_SIDE - 1e-9,
        "{facteur}"
    );
}

/// **La rotation du coin se fait autour du centre du groupe**, de l'angle que le pointeur a
/// parcouru depuis la prise.
#[test]
fn test_la_rotation_du_coin_tourne_autour_du_centre() {
    let groupe = Rect::new(0.0, 0.0, 200.0, 200.0);
    let t = rotation_du_coin(groupe, (200.0, 100.0), (100.0, 200.0), false);
    let Transformation::Rotation { angle, pivot } = t else {
        panic!("une rotation")
    };
    assert_eq!(pivot, (100.0, 100.0));
    assert!(
        (angle - std::f64::consts::FRAC_PI_2).abs() < 1e-9,
        "{angle}"
    );
}
