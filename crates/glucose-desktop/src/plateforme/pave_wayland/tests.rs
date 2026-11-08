//! Le pincement de Wayland, pas à pas — sans compositeur.

use super::Pincement;
use crate::interactions::pave::Mouvement;

fn appliquer(m: &Mouvement, (x, y): (f64, f64)) -> (f64, f64) {
    (m.echelle * x + m.decalage.0, m.echelle * y + m.decalage.1)
}

/// **Le point sous les doigts y reste** : au fil des pas, ce qui était sous le centre des doigts au
/// début est, à la fin, sous le centre où ils ont glissé ; tout autre point a grandi de l'échelle
/// du geste autour de lui. En pixels : le facteur d'échelle de l'écran compte.
#[test]
fn le_point_sous_les_doigts_y_reste() {
    for k in [1.0, 1.25, 2.0] {
        let pointeur = (300.0, 200.0);
        let mut p = Pincement::commencer(pointeur);
        let pas = [
            (1.10, (2.0, -1.0)),
            (1.35, (0.5, 3.0)),
            (0.90, (-4.0, 0.0)),
            (1.60, (1.0, 1.0)),
        ];
        let (mut sous, mut autre) = ((pointeur.0 * k, pointeur.1 * k), (900.0, 50.0));
        let depart_autre = autre;
        let mut glisse = (0.0, 0.0);
        for (echelle, d) in pas {
            let m = p.pas(echelle, d, k).expect("un pas");
            sous = appliquer(&m, sous);
            autre = appliquer(&m, autre);
            glisse = (glisse.0 + d.0, glisse.1 + d.1);
        }
        let centre = ((pointeur.0 + glisse.0) * k, (pointeur.1 + glisse.1) * k);
        assert!(
            (sous.0 - centre.0).abs() < 1e-9 && (sous.1 - centre.1).abs() < 1e-9,
            "k {k} : {sous:?} au lieu de {centre:?}"
        );
        let debut = (pointeur.0 * k, pointeur.1 * k);
        let attendu = (
            1.6 * (depart_autre.0 - debut.0) + centre.0,
            1.6 * (depart_autre.1 - debut.1) + centre.1,
        );
        assert!(
            (autre.0 - attendu.0).abs() < 1e-9 && (autre.1 - attendu.1).abs() < 1e-9,
            "l'échelle du geste : {autre:?} au lieu de {attendu:?}"
        );
    }
}

/// **Une échelle absurde ne bouge rien** — nulle, négative, ou qui n'est pas un nombre.
#[test]
fn une_echelle_absurde_ne_bouge_rien() {
    let mut p = Pincement::commencer((10.0, 10.0));
    for e in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(p.pas(e, (1.0, 1.0), 1.0).is_none(), "{e}");
    }
    let m = p.pas(2.0, (0.0, 0.0), 1.0).expect("un vrai pas");
    assert_eq!(m.echelle, 2.0, "l'état n'a pas bougé : l'échelle part de 1");
}

/// **La similitude d'un pincement** (`Mouvement::pincement`) garde son centre, glissement compris.
#[test]
fn la_similitude_d_un_pincement_garde_son_centre() {
    let m = Mouvement::pincement(1.5, (100.0, 40.0), (3.0, -2.0));
    assert_eq!(appliquer(&m, (100.0, 40.0)), (103.0, 38.0));
}
