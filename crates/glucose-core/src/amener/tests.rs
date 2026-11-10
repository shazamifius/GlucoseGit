//! **LIEN-3 — la disposition de Bring & Go** : rien ne se chevauche, chaque voisin garde la
//! direction de sa vraie place, le plus proche d'abord, et chacun au plus près du centre.

use super::*;

const AIR: f64 = 24.0;

/// Un voisinage qui exerce la loi : des boîtes de tailles diverses, dans toutes les directions,
/// plusieurs dans la même, deux à la même place, une au centre même, et de très loin.
fn voisinage() -> (Rect, Vec<Rect>) {
    let centre = Rect::new(-150.0, -60.0, 300.0, 120.0);
    let mut voisins = Vec::new();
    for k in 0..40 {
        let angle = f64::from(k) * 2.399_963; // l'angle d'or : aucune direction répétée
        let loin = 500.0 + 9_000.0 * f64::from(k % 7);
        let (w, h) = (
            120.0 + 37.0 * f64::from(k % 5),
            40.0 + 23.0 * f64::from(k % 3),
        );
        voisins.push(Rect::new(
            loin * angle.cos() - w / 2.0,
            loin * angle.sin() - h / 2.0,
            w,
            h,
        ));
    }
    // Trois dans la même direction, à trois distances ; deux à la même place ; un au centre.
    for loin in [3_000.0, 1_000.0, 2_000.0, 2_000.0] {
        voisins.push(Rect::new(loin, -20.0, 200.0, 40.0));
    }
    voisins.push(Rect::new(-50.0, -20.0, 100.0, 40.0));
    (centre, voisins)
}

fn milieu_de(b: Rect) -> (f64, f64) {
    (b.left + b.width / 2.0, b.top + b.height / 2.0)
}

/// Les voisins accrochés en leur milieu, à leur vraie place, et l'origine au milieu du centre.
fn poser(centre: Rect, voisins: &[Rect]) -> Vec<Rect> {
    let v: Vec<Voisin> = voisins
        .iter()
        .map(|b| Voisin {
            boite: *b,
            accroche: milieu_de(*b),
            vraie_place: milieu_de(*b),
        })
        .collect();
    disposer(centre, milieu_de(centre), &v, AIR)
}

/// Deux boîtes se chevauchent-elles, l'air compris (le bord exact est permis) ?
fn chevauchent(a: Rect, b: Rect, air: f64) -> bool {
    let e = 1e-6;
    a.left < b.left + b.width + air - e
        && b.left < a.left + a.width + air - e
        && a.top < b.top + b.height + air - e
        && b.top < a.top + a.height + air - e
}

/// **Rien ne se chevauche**, ni entre amenés, ni avec le centre ; et chaque boîte garde sa taille.
#[test]
fn test_lien_3_rien_ne_se_chevauche() {
    let (centre, voisins) = voisinage();
    let places = poser(centre, &voisins);
    let toutes: Vec<Rect> = std::iter::once(centre)
        .chain(places.iter().copied())
        .collect();
    for (i, a) in toutes.iter().enumerate() {
        for b in &toutes[i + 1..] {
            assert!(!chevauchent(*a, *b, AIR), "{a:?} et {b:?}");
        }
    }
    for (p, v) in places.iter().zip(&voisins) {
        assert_eq!((p.width, p.height), (v.width, v.height));
    }
}

/// **Chacun garde la direction de sa vraie place** — à un milliardième de radian.
#[test]
fn test_lien_3_chacun_garde_la_direction_de_sa_vraie_place() {
    let (centre, voisins) = voisinage();
    let c = milieu_de(centre);
    let places = poser(centre, &voisins);
    for (p, v) in places.iter().zip(&voisins) {
        let (vrai, pose) = (milieu_de(*v), milieu_de(*p));
        if vrai == c {
            continue;
        }
        let a = (vrai.1 - c.1).atan2(vrai.0 - c.0);
        let b = (pose.1 - c.1).atan2(pose.0 - c.0);
        assert!((a - b).abs() < 1e-9, "{a} contre {b}");
    }
}

/// **Le plus proche d'abord** : dans une même direction, le plus proche est posé le plus près ; et
/// deux voisins à la même place se posent l'un derrière l'autre, sans se perdre.
#[test]
fn test_lien_3_le_plus_proche_d_abord() {
    let (centre, voisins) = voisinage();
    let places = poser(centre, &voisins);
    let n = voisins.len();
    let x = |i: usize| milieu_de(places[i]).0;
    let (trois_mille, mille, deux_mille, deux_mille_bis) = (n - 5, n - 4, n - 3, n - 2);
    assert!(x(mille) < x(deux_mille), "1 000 avant 2 000");
    assert!(
        x(deux_mille) < x(deux_mille_bis),
        "à égalité, l'ordre donné"
    );
    assert!(x(deux_mille_bis) < x(trois_mille), "2 000 avant 3 000");
}

/// **Au plus près** : chaque boîte reculée d'un millième vers le centre chevaucherait le centre ou
/// une boîte posée **avant** elle (le plus proche d'abord) — elle ne pouvait pas être plus près.
#[test]
fn test_lien_3_chacun_au_plus_pres_du_centre() {
    let (centre, voisins) = voisinage();
    let c = milieu_de(centre);
    let places = poser(centre, &voisins);
    let loin = |i: usize| {
        let m = milieu_de(voisins[i]);
        (m.0 - c.0).hypot(m.1 - c.1)
    };
    let mut ordre: Vec<usize> = (0..voisins.len()).collect();
    ordre.sort_by(|&a, &b| loin(a).total_cmp(&loin(b)).then(a.cmp(&b)));
    for (rang, &i) in ordre.iter().enumerate() {
        let p = places[i];
        let m = milieu_de(p);
        let (dx, dy) = (m.0 - c.0, m.1 - c.1);
        let r = dx.hypot(dy);
        let k = (r - 1e-3) / r;
        let recule = Rect::new(
            c.0 + dx * k - p.width / 2.0,
            c.1 + dy * k - p.height / 2.0,
            p.width,
            p.height,
        );
        let avant = std::iter::once(centre).chain(ordre[..rang].iter().map(|&j| places[j]));
        assert!(
            avant.into_iter().any(|b| chevauchent(recule, b, AIR)),
            "la boîte {i} pouvait se poser plus près"
        );
    }
}

/// **Une loi déterministe** : les mêmes voisins, la même disposition.
#[test]
fn test_lien_3_la_meme_disposition_a_chaque_fois() {
    let (centre, voisins) = voisinage();
    assert_eq!(poser(centre, &voisins), poser(centre, &voisins));
}

/// **Le point d'accroche glisse sur la demi-droite, pas le milieu de la boîte** : une boîte dont le
/// nœud est en bas (une membrane, son titre au-dessus) garde **son nœud** dans la direction de sa
/// vraie place ; et l'origine est le nœud du centre, pas le milieu de sa boîte.
#[test]
fn test_lien_3_le_point_d_accroche_garde_la_direction() {
    let centre = Rect::new(-150.0, -100.0, 300.0, 160.0);
    let origine = (0.0, 20.0);
    let voisins: Vec<Voisin> = [(4000.0, 900.0), (-2500.0, -3100.0), (10.0, 7000.0)]
        .into_iter()
        .map(|(x, y)| Voisin {
            boite: Rect::new(x - 120.0, y - 90.0, 240.0, 120.0),
            accroche: (x, y),
            vraie_place: (x, y),
        })
        .collect();
    let places = disposer(centre, origine, &voisins, AIR);
    for (p, v) in places.iter().zip(&voisins) {
        let accroche = (p.left + 120.0, p.top + 90.0);
        let a = (v.vraie_place.1 - origine.1).atan2(v.vraie_place.0 - origine.0);
        let b = (accroche.1 - origine.1).atan2(accroche.0 - origine.0);
        assert!((a - b).abs() < 1e-9, "{a} contre {b}");
        assert!(!chevauchent(*p, centre, AIR));
    }
}
