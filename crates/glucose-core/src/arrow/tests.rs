//! ARROW-1 — la géométrie d'une flèche, tenue sans écran.

use super::*;
use crate::types::Point2D;

fn fleche(id: &str, a: (f64, f64), b: (f64, f64)) -> Annotation {
    Annotation::arrow(id, a.0, a.1, b.0, b.1)
}

fn coudee(id: &str, a: (f64, f64), par: &[(f64, f64)], b: (f64, f64)) -> Annotation {
    let mut f = fleche(id, a, b);
    if let Annotation::Arrow { waypoints, .. } = &mut f {
        *waypoints = par.iter().map(|(x, y)| Point2D { x: *x, y: *y }).collect();
    }
    f
}

/// Le chemin part de l'origine, passe par les étapes dans l'ordre, et finit à la pointe.
#[test]
fn test_arrow_1_the_path_goes_from_tail_to_head_through_its_waypoints() {
    let f = coudee("a", (0.0, 0.0), &[(10.0, 5.0), (20.0, 5.0)], (30.0, 0.0));
    assert_eq!(
        path(&f).expect("une flèche"),
        vec![(0.0, 0.0), (10.0, 5.0), (20.0, 5.0), (30.0, 0.0)]
    );
}

/// Ce qui n'est pas une flèche n'a pas de chemin — et ce n'est pas un chemin vide.
#[test]
fn test_arrow_1_what_is_not_an_arrow_has_no_path() {
    assert_eq!(path(&Annotation::text("t", 0.0, 0.0, "x")), None);
    assert_eq!(
        distance_to(&Annotation::text("t", 0.0, 0.0, "x"), (0.0, 0.0)),
        None
    );
}

/// La distance à un segment : au milieu, en face d'une extrémité, et au-delà.
#[test]
fn test_arrow_1_the_distance_falls_back_on_the_ends() {
    let f = fleche("a", (0.0, 0.0), (100.0, 0.0));
    // En face du milieu : la perpendiculaire.
    assert!((distance_to(&f, (50.0, 10.0)).expect("d") - 10.0).abs() < 1e-9);
    // Sur le trait : rien.
    assert!(distance_to(&f, (50.0, 0.0)).expect("d") < 1e-9);
    // Au-delà de la pointe : c'est la pointe qui est le point le plus proche, pas la droite.
    assert!((distance_to(&f, (130.0, 0.0)).expect("d") - 30.0).abs() < 1e-9);
    assert!((distance_to(&f, (-30.0, 0.0)).expect("d") - 30.0).abs() < 1e-9);
}

/// Une flèche réduite à un point ne divise pas par zéro.
#[test]
fn test_arrow_1_a_degenerate_arrow_is_still_measurable() {
    let f = fleche("a", (7.0, 7.0), (7.0, 7.0));
    assert!((distance_to(&f, (7.0, 10.0)).expect("d") - 3.0).abs() < 1e-9);
}

/// Un coude se mesure sur le tronçon le plus proche, pas sur la corde.
#[test]
fn test_arrow_1_a_bend_is_measured_on_its_nearest_leg() {
    let f = coudee("a", (0.0, 0.0), &[(50.0, 50.0)], (100.0, 0.0));
    // Le point est à cinq unités sous le sommet du coude.
    assert!(distance_to(&f, (50.0, 45.0)).expect("d") <= 5.0 + 1e-9);
    // Le milieu de la corde, lui, est loin des deux tronçons.
    assert!(distance_to(&f, (50.0, 0.0)).expect("d") > 30.0);
}

/// La bande garde une épaisseur **écran** : viser une flèche ne devient pas plus dur au
/// dézoom, ce qui est exactement ce que `non-scaling-stroke` fait côté Tauri.
#[test]
fn test_arrow_1_the_band_keeps_its_screen_width() {
    let fleches = [fleche("a", (0.0, 0.0), (100.0, 0.0))];
    // À l'échelle 1, la bande vaut 24 px : on attrape jusqu'à 12 unités monde.
    assert!(at(&fleches, (50.0, 11.0), 1.0).is_some());
    assert!(at(&fleches, (50.0, 13.0), 1.0).is_none());
    // Au demi-zoom, la même bande écran couvre deux fois plus de monde.
    assert!(at(&fleches, (50.0, 23.0), 0.5).is_some());
    assert!(at(&fleches, (50.0, 25.0), 0.5).is_none());
}

/// Entre deux flèches qui se croisent, la plus proche gagne ; à égalité, celle du dessus.
#[test]
fn test_arrow_1_the_nearest_wins_and_ties_go_to_the_top_one() {
    let fleches = [
        fleche("dessous", (0.0, 0.0), (100.0, 0.0)),
        fleche("dessus", (0.0, 0.0), (100.0, 0.0)),
    ];
    let (gagnante, _) = at(&fleches, (50.0, 1.0), 1.0).expect("une flèche");
    assert_eq!(
        gagnante.id(),
        "dessus",
        "à égalité, celle dessinée au-dessus"
    );

    let fleches = [
        fleche("loin", (0.0, 20.0), (100.0, 20.0)),
        fleche("pres", (0.0, 0.0), (100.0, 0.0)),
    ];
    let (gagnante, _) = at(&fleches, (50.0, 2.0), 1.0).expect("une flèche");
    assert_eq!(gagnante.id(), "pres");
}

/// Rien à portée : rien. Une flèche lointaine ne se sélectionne pas par accident.
#[test]
fn test_arrow_1_nothing_within_reach_selects_nothing() {
    let fleches = [fleche("a", (0.0, 0.0), (100.0, 0.0))];
    assert!(at(&fleches, (50.0, 400.0), 1.0).is_none());
    assert!(at(&[], (0.0, 0.0), 1.0).is_none());
}
