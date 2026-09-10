//! Tests portés fidèlement de src/canvas/arrowAnchor.test.ts

use glucose_core::arrow_anchor::{arrow_endpoints, ArrowAnchor};
use glucose_core::geometry::Point;

fn note(x: f64, y: f64, w: f64, h: f64) -> ArrowAnchor {
    ArrowAnchor::with_box(x + w / 2.0, y + h / 2.0, x, x + w, y, y + h)
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-5
}

#[test]
fn test_arrow_endpoints_exit_facing_side() {
    let a = note(0.0, 0.0, 200.0, 100.0);
    let b = note(0.0, 500.0, 200.0, 100.0);
    let ep = arrow_endpoints(a, b, &[]);

    assert!(ep.start.y > 100.0); // sous le bas de A (y=100)
    assert!(ep.end.y < 500.0);   // au-dessus du haut de B (y=500)
    assert!(approx_eq(ep.start.x, 100.0)); // pile sous le centre
    assert!(approx_eq(ep.end.x, 100.0));

    // Sort par la droite quand la cible est à droite
    let ep_horiz = arrow_endpoints(note(0.0, 0.0, 200.0, 100.0), note(500.0, 0.0, 200.0, 100.0), &[]);
    assert!(ep_horiz.start.x > 200.0);
    assert!(ep_horiz.end.x < 500.0);
    assert!(approx_eq(ep_horiz.start.y, 50.0));

    // Décolle du bloc de 12px quand la place le permet
    assert!(approx_eq(ep.start.y, 112.0)); // bas de A (100) + marge (12)
}

#[test]
fn test_arrow_never_inverts() {
    // Ne croise pas les pointes sur des blocs quasi collés (< 24px d'écart)
    let a = note(0.0, 0.0, 200.0, 100.0);   // bas à y=100
    let b = note(0.0, 110.0, 200.0, 100.0); // haut à y=110 - 10px d'écart
    let ep = arrow_endpoints(a, b, &[]);
    assert!(ep.end.y >= ep.start.y); // sens conservé

    // Garde le sens pour tout écart de 0 à 40px vertical
    for gap in 0..=40 {
        let ep_v = arrow_endpoints(note(0.0, 0.0, 200.0, 100.0), note(0.0, 100.0 + (gap as f64), 200.0, 100.0), &[]);
        assert!(ep_v.end.y >= ep_v.start.y, "failed at gap {}", gap);
    }

    // Garde le sens à l'horizontale aussi
    for gap in 0..=40 {
        let ep_h = arrow_endpoints(note(0.0, 0.0, 200.0, 100.0), note(200.0 + (gap as f64), 0.0, 200.0, 100.0), &[]);
        assert!(ep_h.end.x >= ep_h.start.x, "failed at gap {}", gap);
    }

    // Ne consomme jamais plus que la place disponible
    for &gap in &[0.0, 5.0, 10.0, 23.0, 24.0, 25.0, 100.0] {
        let ep_gap = arrow_endpoints(note(0.0, 0.0, 200.0, 100.0), note(0.0, 100.0 + gap, 200.0, 100.0), &[]);
        let d = ep_gap.start.distance_to(ep_gap.end);
        assert!(d <= gap + 0.001, "failed at gap {}", gap);
    }
}

#[test]
fn test_arrow_waypoints_and_free_ends() {
    // Vise le point de passage
    let wp = [Point::new(-300.0, 50.0)];
    let ep_wp = arrow_endpoints(note(0.0, 0.0, 200.0, 100.0), note(500.0, 0.0, 200.0, 100.0), &wp);
    assert!(ep_wp.start.x < 0.0);

    // Borne la marge sur un point de passage très proche
    let close_wp = [Point::new(100.0, 105.0)]; // 5px sous le bas de A
    let ep_close = arrow_endpoints(note(0.0, 0.0, 200.0, 100.0), note(0.0, 500.0, 200.0, 100.0), &close_wp);
    assert!(ep_close.start.y <= close_wp[0].y);

    // Extrémité libre sans boîte
    let libre = ArrowAnchor::point(42.0, 7.0);
    let ep_libre = arrow_endpoints(libre, note(500.0, 0.0, 200.0, 100.0), &[]);
    assert_eq!(ep_libre.start, Point::new(42.0, 7.0));

    // Ne produit pas de NaN quand confondues
    let a = note(0.0, 0.0, 200.0, 100.0);
    let ep_same = arrow_endpoints(a, a, &[]);
    assert!(ep_same.start.x.is_finite());
    assert!(ep_same.start.y.is_finite());
    assert!(ep_same.end.x.is_finite());
    assert!(ep_same.end.y.is_finite());
}
