use super::*;

/// La plus grande distance d'un point de la courbe à la ligne brisée, sur un échantillonnage
/// bien plus fin que celle-ci.
fn ecart_maximal(m: &Morceau, brisee: &[Point]) -> f64 {
    let distance = |p: Point| {
        brisee
            .windows(2)
            .map(|s| crate::geometry::distance_to_segment(p, s[0], s[1]))
            .fold(f64::INFINITY, f64::min)
    };
    (0..=2000)
        .map(|k| distance(m.point(f64::from(k) / 2000.0)))
        .fold(0.0, f64::max)
}

/// **Aplatie, une courbe ne s'écarte nulle part de plus que la tolérance** — la promesse de
/// la formule de Wang, vérifiée sur une courbe ample et sur une courbe serrée.
#[test]
fn test_fleche_1_la_ligne_brisee_tient_la_tolerance() {
    let courbes = morceaux(
        &[(0.0, 0.0), (400.0, 300.0), (800.0, -100.0), (900.0, 600.0)],
        true,
    );
    for tolerance in [4.0, 1.0, 0.25] {
        for m in &courbes {
            let brisee = aplatir(std::slice::from_ref(m), tolerance);
            let ecart = ecart_maximal(m, &brisee);
            assert!(
                ecart <= tolerance,
                "tolérance {tolerance} : écart {ecart} sur {} tronçons",
                brisee.len() - 1
            );
        }
    }
}

/// **La formule ne gaspille pas** : pour une tolérance quatre fois plus fine, deux fois plus
/// de tronçons — `n` suit `1/√ε`, pas `1/ε`.
#[test]
fn test_fleche_1_le_nombre_de_troncons_suit_la_racine_de_la_tolerance() {
    let m = morceaux(&[(0.0, 0.0), (300.0, 400.0), (900.0, 0.0)], true)[0];
    let (grossier, fin) = (m.troncons(1.0), m.troncons(0.25));
    assert!(
        fin >= 2 * grossier - 1 && fin <= 2 * grossier + 1,
        "{grossier} puis {fin}"
    );
}

/// **La courbe passe par les points de la flèche**, et elle part et arrive dans l'axe de
/// ses tronçons extrêmes — les reflets de Tauri.
#[test]
fn test_fleche_1_la_courbe_passe_par_ses_points() {
    let points = [(0.0, 0.0), (100.0, 50.0), (200.0, 0.0)];
    let m = morceaux(&points, true);
    assert_eq!(m.len(), 2);
    assert_eq!(m[0].depart(), points[0]);
    assert_eq!(m[0].arrivee(), points[1]);
    assert_eq!(m[1].arrivee(), points[2]);
    let Morceau::Courbe { de, c1, .. } = m[0] else {
        panic!("une courbe");
    };
    // Le reflet de (100, 50) par (0, 0) est (−100, −50) : le premier contrôle regarde
    // (100, 50) − (−100, −50), donc dans l'axe du premier tronçon.
    let (dx, dy) = (c1.0 - de.0, c1.1 - de.1);
    assert!(
        (dx * 50.0 - dy * 100.0).abs() < 1e-9,
        "dans l'axe : ({dx}, {dy})"
    );
}

/// **Deux points font un segment**, courbe demandée ou non — il n'y a rien à courber.
#[test]
fn test_fleche_1_deux_points_font_un_segment() {
    let m = morceaux(&[(0.0, 0.0), (10.0, 0.0)], true);
    assert_eq!(
        m,
        vec![Morceau::Droit {
            de: (0.0, 0.0),
            a: (10.0, 0.0)
        }]
    );
    assert_eq!(m[0].troncons(0.001), 1);
}

/// **Le milieu d'un morceau courbe est sur la courbe** — pas au milieu de sa corde.
#[test]
fn test_fleche_1_le_milieu_d_une_courbe_est_sur_elle() {
    let m = morceaux(&[(0.0, 0.0), (100.0, 100.0), (200.0, 0.0)], true)[0];
    let milieu = m.milieu();
    let corde = (50.0, 50.0);
    assert!(
        (milieu.0 - corde.0).hypot(milieu.1 - corde.1) > 1.0,
        "{milieu:?} ne doit pas être le milieu de la corde"
    );
    assert_eq!(milieu, m.point(0.5));
}
