//! Tests portés fidèlement de src/canvas/arrowAnchor.test.ts

use glucose_core::arrow_anchor::{arrow_endpoints, ArrowAnchor, ANCHOR_MARGIN};
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
    assert!(ep.end.y < 500.0); // au-dessus du haut de B (y=500)
    assert!(approx_eq(ep.start.x, 100.0)); // pile sous le centre
    assert!(approx_eq(ep.end.x, 100.0));

    // Sort par la droite quand la cible est à droite
    let ep_horiz = arrow_endpoints(
        note(0.0, 0.0, 200.0, 100.0),
        note(500.0, 0.0, 200.0, 100.0),
        &[],
    );
    assert!(ep_horiz.start.x > 200.0);
    assert!(ep_horiz.end.x < 500.0);
    assert!(approx_eq(ep_horiz.start.y, 50.0));

    // Décolle du bloc de 12px quand la place le permet
    assert!(approx_eq(ep.start.y, 112.0)); // bas de A (100) + marge (12)
}

#[test]
fn test_arrow_never_inverts() {
    // Ne croise pas les pointes sur des blocs quasi collés (< 24px d'écart)
    let a = note(0.0, 0.0, 200.0, 100.0); // bas à y=100
    let b = note(0.0, 110.0, 200.0, 100.0); // haut à y=110 - 10px d'écart
    let ep = arrow_endpoints(a, b, &[]);
    assert!(ep.end.y >= ep.start.y); // sens conservé

    // Garde le sens pour tout écart de 0 à 40px vertical
    for gap in 0..=40 {
        let ep_v = arrow_endpoints(
            note(0.0, 0.0, 200.0, 100.0),
            note(0.0, 100.0 + (gap as f64), 200.0, 100.0),
            &[],
        );
        assert!(ep_v.end.y >= ep_v.start.y, "failed at gap {}", gap);
    }

    // Garde le sens à l'horizontale aussi
    for gap in 0..=40 {
        let ep_h = arrow_endpoints(
            note(0.0, 0.0, 200.0, 100.0),
            note(200.0 + (gap as f64), 0.0, 200.0, 100.0),
            &[],
        );
        assert!(ep_h.end.x >= ep_h.start.x, "failed at gap {}", gap);
    }

    // Ne consomme jamais plus que la place disponible
    for &gap in &[0.0, 5.0, 10.0, 23.0, 24.0, 25.0, 100.0] {
        let ep_gap = arrow_endpoints(
            note(0.0, 0.0, 200.0, 100.0),
            note(0.0, 100.0 + gap, 200.0, 100.0),
            &[],
        );
        let d = ep_gap.start.distance_to(ep_gap.end);
        assert!(d <= gap + 0.001, "failed at gap {}", gap);
    }
}

#[test]
fn test_arrow_waypoints_and_free_ends() {
    // Vise le point de passage
    let wp = [Point::new(-300.0, 50.0)];
    let ep_wp = arrow_endpoints(
        note(0.0, 0.0, 200.0, 100.0),
        note(500.0, 0.0, 200.0, 100.0),
        &wp,
    );
    assert!(ep_wp.start.x < 0.0);

    // Borne la marge sur un point de passage très proche
    let close_wp = [Point::new(100.0, 105.0)]; // 5px sous le bas de A
    let ep_close = arrow_endpoints(
        note(0.0, 0.0, 200.0, 100.0),
        note(0.0, 500.0, 200.0, 100.0),
        &close_wp,
    );
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

/// Une page longue (100 × 800) dont le passage est à 600, et ce qu'elle relie loin au-dessus et
/// à gauche : la flèche arrive en biais.
fn page_au_passage() -> (ArrowAnchor, Point) {
    let page = ArrowAnchor::with_box(50.0, 600.0, 0.0, 100.0, 0.0, 800.0).au_passage();
    (page, Point::new(-100.0, -3000.0))
}

#[test]
fn fleche_6_a_cote_elle_arrive_a_la_hauteur_du_passage_par_le_cote_qui_fait_face() {
    let (page, loin) = page_au_passage();
    let bout = arrow_endpoints(page, ArrowAnchor::point(loin.x, loin.y), &[]).start;
    // Le côté gauche, à la hauteur du passage — la marge l'écarte du bord, vers ce qu'elle relie.
    assert!(bout.x <= 0.0, "sur le côté gauche ou au-delà : {:?}", bout);
    assert!(
        (bout.y - 600.0).abs() <= ANCHOR_MARGIN,
        "à la hauteur du passage : {:?}",
        bout
    );
    // La preuve à l'envers : la loi d'avant (la même ancre, sans le passage) touchait le bord du
    // haut, loin au-dessus de la citation.
    let avant = arrow_endpoints(
        ArrowAnchor {
            passage: false,
            ..page
        },
        ArrowAnchor::point(loin.x, loin.y),
        &[],
    )
    .start;
    assert!(
        avant.y < 0.0,
        "l'ancienne loi sortait par le haut : {:?}",
        avant
    );
    // À droite, le côté droit.
    let droite = arrow_endpoints(page, ArrowAnchor::point(900.0, -2000.0), &[]).start;
    assert!(
        droite.x >= 100.0 && (droite.y - 600.0).abs() <= ANCHOR_MARGIN,
        "{:?}",
        droite
    );
}

#[test]
fn fleche_6_juste_au_dessus_rien_ne_change() {
    let (page, _) = page_au_passage();
    let dessus = ArrowAnchor::point(60.0, -2000.0);
    let avec = arrow_endpoints(page, dessus, &[]);
    let sans = arrow_endpoints(
        ArrowAnchor {
            passage: false,
            ..page
        },
        dessus,
        &[],
    );
    assert_eq!(avec, sans, "sans côté qui fasse face, la loi d'avant");
    // Et une ancre sans passage ne change jamais.
    let note = ArrowAnchor::with_box(50.0, 400.0, 0.0, 100.0, 0.0, 800.0);
    let loin = ArrowAnchor::point(-100.0, -3000.0);
    assert!(arrow_endpoints(note, loin, &[]).start.y < 0.0);
}
