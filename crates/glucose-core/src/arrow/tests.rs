//! ARROW-1 — la géométrie d'une flèche, tenue sans écran.

use super::*;
use crate::types::Point2D;

/// Une flèche qui ne s'accroche à rien : le résolveur ne connaît aucun nœud.
fn libre(_: &str) -> Option<crate::geometry::Rect> {
    None
}

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
    assert!(at(&fleches, libre, (50.0, 11.0), 1.0).is_some());
    assert!(at(&fleches, libre, (50.0, 13.0), 1.0).is_none());
    // Au demi-zoom, la même bande écran couvre deux fois plus de monde.
    assert!(at(&fleches, libre, (50.0, 23.0), 0.5).is_some());
    assert!(at(&fleches, libre, (50.0, 25.0), 0.5).is_none());
}

/// Entre deux flèches qui se croisent, la plus proche gagne ; à égalité, celle du dessus.
#[test]
fn test_arrow_1_the_nearest_wins_and_ties_go_to_the_top_one() {
    let fleches = [
        fleche("dessous", (0.0, 0.0), (100.0, 0.0)),
        fleche("dessus", (0.0, 0.0), (100.0, 0.0)),
    ];
    let (gagnante, _) = at(&fleches, libre, (50.0, 1.0), 1.0).expect("une flèche");
    assert_eq!(
        gagnante.id(),
        "dessus",
        "à égalité, celle dessinée au-dessus"
    );

    let fleches = [
        fleche("loin", (0.0, 20.0), (100.0, 20.0)),
        fleche("pres", (0.0, 0.0), (100.0, 0.0)),
    ];
    let (gagnante, _) = at(&fleches, libre, (50.0, 2.0), 1.0).expect("une flèche");
    assert_eq!(gagnante.id(), "pres");
}

/// Rien à portée : rien. Une flèche lointaine ne se sélectionne pas par accident.
#[test]
fn test_arrow_1_nothing_within_reach_selects_nothing() {
    let fleches = [fleche("a", (0.0, 0.0), (100.0, 0.0))];
    assert!(at(&fleches, libre, (50.0, 400.0), 1.0).is_none());
    assert!(at(&[], libre, (0.0, 0.0), 1.0).is_none());
}

// ── L'ancrage : une flèche s'arrête sur le bord de ce qu'elle vise ─────────

use crate::geometry::Rect;

fn ancree(id: &str, de: Option<&str>, vers: Option<&str>) -> Annotation {
    let mut f = fleche(id, (0.0, 0.0), (400.0, 0.0));
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut f
    {
        *source_id = de.map(str::to_string);
        *target_id = vers.map(str::to_string);
    }
    f
}

/// Une flèche qui vise un nœud s'arrête sur son **bord**, pas sur son centre.
///
/// C'est ce que `arrow_anchor` savait faire depuis toujours — cent soixante-dix-sept lignes
/// écrites, testées, et que personne n'appelait.
#[test]
fn test_arrow_1_an_anchored_arrow_stops_at_the_border() {
    // Une boîte de 200 de large centrée en (400, 0) : son bord gauche est à 300.
    let resolve = |id: &str| (id == "cible").then(|| Rect::new(300.0, -50.0, 200.0, 100.0));
    let f = ancree("a", None, Some("cible"));

    let points = path_with(&f, resolve).expect("un chemin");
    let (fin_x, fin_y) = *points.last().expect("une pointe");
    assert!(
        fin_x < 300.0,
        "la pointe s'arrête avant le bord gauche ({fin_x}), pas au centre"
    );
    assert!(fin_y.abs() < 1.0, "elle reste sur l'axe");

    // Sans ancrage, elle irait jusqu'à son point brut.
    let brut = path(&f).expect("un chemin");
    assert_eq!(brut.last(), Some(&(400.0, 0.0)));
}

/// Une flèche sans ancre garde exactement son tracé : l'ancrage ne coûte rien à qui n'en veut
/// pas, et `path_with` rend alors la même chose que `path`.
#[test]
fn test_arrow_1_an_unanchored_arrow_keeps_its_raw_path() {
    let f = fleche("a", (0.0, 0.0), (400.0, 0.0));
    assert_eq!(path_with(&f, libre), path(&f));
}

/// Un nœud introuvable ne fait pas disparaître la flèche : elle retombe sur son point brut.
#[test]
fn test_arrow_1_a_missing_node_falls_back_on_the_raw_point() {
    let f = ancree("a", None, Some("fantome"));
    assert_eq!(path_with(&f, libre), path(&f));
}

/// Et le clic suit l'ancrage : on vise la flèche là où elle se dessine.
#[test]
fn test_arrow_1_the_click_follows_the_anchor() {
    let resolve = |id: &str| (id == "cible").then(|| Rect::new(300.0, -50.0, 200.0, 100.0));
    let fleches = [ancree("a", None, Some("cible"))];
    // Un point bien à l'intérieur de la boîte visée : le trait n'y va plus.
    assert!(at(&fleches, resolve, (450.0, 0.0), 1.0).is_none());
    // Alors qu'avant le bord, il est toujours là.
    assert!(at(&fleches, resolve, (200.0, 0.0), 1.0).is_some());
}
