//! Les épreuves de la loi, et de son instrument processeur.
//!
//! La loi se vérifie contre une **construction indépendante** : le bord échantillonné point par
//! point, ses distances et ses abscisses mesurées à la force brute. L'instrument se vérifie
//! contre la loi elle-même, évaluée en **chaque** pixel : le découpage en segments ne doit rien
//! changer, au bit près.

use super::*;
use crate::report::Pixel;
use std::f64::consts::PI;

/// Le bord d'un rectangle arrondi, échantillonné dans l'ordre du parcours : `(x, y, s)`, en
/// `f64`, construit sans rien emprunter au module.
fn bord_echantillonne(f: &Arrondi, pas: f64) -> Vec<(f64, f64, f64)> {
    let (g, h, d, b, r) = (
        f64::from(f.gauche),
        f64::from(f.haut),
        f64::from(f.droite),
        f64::from(f.bas),
        f64::from(f.rayon),
    );
    let mut points = Vec::new();
    let mut s = 0.0;
    let droit = |x0: f64, y0: f64, x1: f64, y1: f64, s: &mut f64, pts: &mut Vec<_>| {
        let l = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        let n = (l / pas).ceil().max(1.0) as usize;
        for i in 0..n {
            let t = i as f64 / n as f64;
            pts.push((x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, *s + l * t));
        }
        *s += l;
    };
    let arc = |cx: f64, cy: f64, a0: f64, s: &mut f64, pts: &mut Vec<_>| {
        let l = r * PI / 2.0;
        let n = (l / pas).ceil().max(1.0) as usize;
        for i in 0..n {
            let t = i as f64 / n as f64;
            let a = a0 + t * PI / 2.0;
            pts.push((cx + r * a.cos(), cy + r * a.sin(), *s + l * t));
        }
        *s += l;
    };
    droit(g + r, h, d - r, h, &mut s, &mut points);
    arc(d - r, h + r, -PI / 2.0, &mut s, &mut points);
    droit(d, h + r, d, b - r, &mut s, &mut points);
    arc(d - r, b - r, 0.0, &mut s, &mut points);
    droit(d - r, b, g + r, b, &mut s, &mut points);
    arc(g + r, b - r, PI / 2.0, &mut s, &mut points);
    droit(g, b - r, g, h + r, &mut s, &mut points);
    arc(g + r, h + r, PI, &mut s, &mut points);
    points
}

/// Le point du bord le plus proche, à la force brute : sa distance et son abscisse.
fn plus_proche(bord: &[(f64, f64, f64)], x: f64, y: f64) -> (f64, f64) {
    bord.iter()
        .map(|&(bx, by, s)| (((bx - x).powi(2) + (by - y).powi(2)).sqrt(), s))
        .fold((f64::INFINITY, 0.0), |a, b| if b.0 < a.0 { b } else { a })
}

/// Des points pseudo-aléatoires, reproductibles, dans une boîte.
fn points(n: usize, boite: (f32, f32, f32, f32)) -> Vec<(f32, f32)> {
    let mut graine = 0x9e37_79b9_u32;
    let mut suivant = move || {
        graine ^= graine << 13;
        graine ^= graine >> 17;
        graine ^= graine << 5;
        graine as f32 / u32::MAX as f32
    };
    (0..n)
        .map(|_| {
            (
                boite.0 + (boite.2 - boite.0) * suivant(),
                boite.1 + (boite.3 - boite.1) * suivant(),
            )
        })
        .collect()
}

fn forme() -> Arrondi {
    Arrondi::nouveau(40.0, 30.0, 300.0, 180.0, 60.0)
}

/// **La distance est exacte** : celle du point du bord le plus proche, au pas de
/// l'échantillonnage près, avec le signe de l'intérieur.
#[test]
fn test_la_distance_est_celle_du_bord_le_plus_proche() {
    let f = forme();
    let bord = bord_echantillonne(&f, 0.01);
    for (x, y) in points(2_000, (0.0, 0.0, 400.0, 260.0)) {
        let (brute, _) = plus_proche(&bord, f64::from(x), f64::from(y));
        let d = f64::from(f.distance(x, y));
        assert!(
            (d.abs() - brute).abs() < 0.02,
            "en ({x}, {y}) : distance {d}, le bord est a {brute}"
        );
        let dedans = x > f.gauche && x < f.droite && y > f.haut && y < f.bas;
        if d < -0.02 {
            assert!(dedans, "({x}, {y}) se dit dedans et ne l'est pas");
        }
    }
}

/// **Repousser le bord est exact**, y compris rentré au-delà du rayon, où les coins
/// deviennent vifs.
#[test]
fn test_dilater_retire_exactement_la_distance() {
    let f = forme();
    for d in [-80.0_f32, -60.0, -20.0, -0.5, 0.5, 12.0] {
        let g = f.dilate(d);
        for (x, y) in points(500, (0.0, 0.0, 400.0, 260.0)) {
            let attendu = f.distance(x, y) - d;
            // Rentré au-delà du rayon, l'égalité ne vaut que dedans : dehors, la forme rentrée
            // a des coins vifs dont la distance n'est plus celle du bord d'origine.
            if d < -f.rayon && attendu > 0.0 {
                continue;
            }
            let lu = g.distance(x, y);
            assert!(
                (lu - attendu).abs() < 1e-3,
                "dilate({d}) en ({x}, {y}) : {lu} au lieu de {attendu}"
            );
        }
    }
}

/// **L'étendue d'une ligne est l'ensemble où la distance est négative.**
#[test]
fn test_l_etendue_d_une_ligne_est_celle_de_la_distance() {
    let f = forme();
    for i in 0..=260 {
        let y = i as f32 + 0.5;
        match f.etendue(y) {
            None => {
                for x in 0..400 {
                    assert!(f.distance(x as f32, y) > -1e-3, "({x}, {y}) oublie");
                }
            }
            Some((x0, x1)) => {
                assert!(
                    f.distance(x0, y).abs() < 1e-3,
                    "bord gauche de la ligne {y}"
                );
                assert!(f.distance(x1, y).abs() < 1e-3, "bord droit de la ligne {y}");
                assert!(f.distance((x0 + x1) / 2.0, y) <= 1e-3);
            }
        }
    }
}

/// **L'abscisse est celle du point du bord le plus proche**, mesurée le long du parcours.
#[test]
fn test_l_abscisse_est_celle_du_bord_le_plus_proche() {
    let f = forme();
    let bord = bord_echantillonne(&f, 0.01);
    let longueurs = f.longueurs();
    let debuts: Vec<f64> = longueurs
        .iter()
        .scan(0.0, |s, l| {
            let d = *s;
            *s += l;
            Some(d)
        })
        .collect();
    let perimetre: f64 = longueurs.iter().sum();
    for (x, y) in points(2_000, (0.0, 0.0, 400.0, 260.0)) {
        let (_, brute) = plus_proche(&bord, f64::from(x), f64::from(y));
        let (k, local) = f.sur_le_bord(x, y);
        let s = debuts[k] + f64::from(local);
        let ecart = (s - brute).abs().min(perimetre - (s - brute).abs());
        // Au centre, plusieurs points du bord sont à égale distance : l'abscisse n'y est pas
        // définie, et seule compte la distance.
        let d = f64::from(f.distance(x, y));
        let equidistant = d < -f64::from(f.bas - f.haut) / 2.0 + 1.0;
        assert!(
            equidistant || ecart < 0.05,
            "en ({x}, {y}) : abscisse {s}, le bord le plus proche est a {brute}"
        );
    }
}

/// **Le pointillé alterne tiret et vide depuis le départ, et se soude au bout du tour.**
#[test]
fn test_le_pointille_alterne_et_se_soude_au_bout_du_tour() {
    let f = forme();
    let tiret = 10.0;
    let p = f.pointille(tiret);
    let bord = bord_echantillonne(&f, 0.25);
    let perimetre: f64 = f.longueurs().iter().sum();
    for &(x, y, s) in &bord {
        let (k, local) = f.sur_le_bord(x as f32, y as f32);
        let ecart = f64::from(p.ecart(k, local));
        let dans_la_periode = s.rem_euclid(2.0 * f64::from(tiret));
        let loin_des_bouts = (dans_la_periode - f64::from(tiret)).abs() > 0.1
            && dans_la_periode > 0.1
            && dans_la_periode < 2.0 * f64::from(tiret) - 0.1
            && s < perimetre - 2.0 * f64::from(tiret);
        if loin_des_bouts && dans_la_periode < f64::from(tiret) {
            assert_eq!(ecart, 0.0, "s = {s} est sur un tiret");
        } else if loin_des_bouts {
            assert!(ecart > 0.0, "s = {s} est dans un vide");
        }
    }
    // Juste avant la fin du tour, dans un vide : le prochain tiret est le premier.
    let reste_au_bout = perimetre.rem_euclid(2.0 * f64::from(tiret));
    if reste_au_bout > f64::from(tiret) {
        let (k, local) = f.sur_le_bord(f.gauche + f.rayon - 0.01, f.haut);
        let ecart = p.ecart(k, local);
        assert!(
            ecart < 0.1,
            "au bout du tour, le premier tiret est a {ecart}"
        );
    }
}

/// Une membrane d'épreuve : les couches et les opacités de Glucose, à l'échelle `e`.
fn membrane(x: f32, y: f32, l: f32, h: f32, e: f32, selection: bool) -> Membrane {
    let r = 60.0 * e;
    let fond = Arrondi::nouveau(x, y, l, h, r);
    let halo = |pad: f32| {
        Arrondi::nouveau(
            x - pad,
            y - pad,
            l + 2.0 * pad,
            h + 2.0 * pad,
            r + pad / 2.0,
        )
    };
    let tiret = 10.0 * e;
    Membrane {
        remplissages: [
            (halo(20.0 * e), 8.0 / 255.0),
            (halo(10.0 * e), 14.0 / 255.0),
            (fond, 8.0 / 255.0),
        ],
        bord: Bord {
            forme: fond,
            demi_largeur: if selection { 1.0 } else { e },
            pointille: (!selection && tiret >= 1.0).then(|| fond.pointille(tiret)),
            alpha: if selection {
                235.0 / 255.0
            } else {
                115.0 / 255.0
            },
        },
        teinte: [96.0 / 255.0, 165.0 / 255.0, 250.0 / 255.0],
    }
}

/// Une image opaque, sombre, qui n'est pas uniforme : une erreur de composition sur une
/// destination variée ne se cache pas derrière un fond constant.
fn image(l: u32, h: u32) -> Vec<Pixel> {
    (0..l * h)
        .map(|i| {
            let (x, y) = (i % l, i / l);
            [
                (x % 37) as u8 + 11,
                (y % 23) as u8 + 7,
                ((x + y) % 29) as u8 + 13,
                255,
            ]
        })
        .collect()
}

/// La référence : la loi évaluée en **chaque** pixel, composée comme l'instrument compose.
fn reference(m: &Membrane, image: &mut [Pixel], l: u32) {
    for (i, pixel) in image.iter_mut().enumerate() {
        let (x, y) = ((i as u32 % l) as f32 + 0.5, (i as u32 / l) as f32 + 0.5);
        let a = m.alpha(x, y);
        if a > 0.0 {
            peindre::Melange::de(m.teinte, a).poser(pixel);
        }
    }
}

/// **Le découpage en segments ne change rien, au bit près**, sur toutes les formes qui
/// éprouvent ses recoins : entière, débordant de l'image, vue de très près par son coin,
/// sélectionnée, au trait plus fin qu'un pixel, sans halos, sans rayon.
#[test]
fn test_les_segments_rendent_exactement_la_loi_en_chaque_pixel() {
    let (l, h) = (320_u32, 200_u32);
    let mut sans_halos = membrane(30.0, 40.0, 200.0, 120.0, 1.0, false);
    sans_halos.remplissages[0].1 = 0.0;
    sans_halos.remplissages[1].1 = 0.0;
    let cas = [
        ("entiere", membrane(30.5, 25.25, 240.0, 140.0, 1.0, false)),
        (
            "debordante",
            membrane(-80.0, -50.0, 300.0, 400.0, 1.0, false),
        ),
        (
            "par son coin, de pres",
            membrane(90.0, 60.0, 50_000.0, 40_000.0, 40.0, false),
        ),
        (
            "selectionnee",
            membrane(20.0, 20.0, 260.0, 150.0, 0.7, true),
        ),
        ("trait fin", membrane(10.0, 10.0, 250.0, 170.0, 0.2, false)),
        ("sans halos", sans_halos),
        (
            "de tres loin, selectionnee",
            membrane(30.0, 25.0, 300.0, 200.0, 0.02, true),
        ),
        ("sans rayon", {
            let mut m = membrane(50.0, 50.0, 150.0, 90.0, 1.0, false);
            m.bord.forme.rayon = 0.0;
            m.remplissages.iter_mut().for_each(|(f, _)| f.rayon = 0.0);
            m
        }),
    ];
    for (nom, m) in cas {
        let mut segments = image(l, h);
        let mut chaque_pixel = segments.clone();
        let encre = peindre(&m, &mut segments, l, h);
        reference(&m, &mut chaque_pixel, l);
        let differents = segments
            .iter()
            .zip(&chaque_pixel)
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            differents, 0,
            "{nom} : {differents} pixels different de la loi"
        );
        assert!(encre, "{nom} : rien n'a ete pose");
        assert_ne!(segments, image(l, h), "{nom} : l'image n'a pas change");
    }
}

/// **Loin du coin d'une très grande membrane, le bord reste net en `f32`** : sa place est
/// juste au centième de pixel près, là où un calcul depuis le centre tremblerait d'un quart.
#[test]
fn test_le_bord_d_une_membrane_immense_reste_a_sa_place() {
    let f = Arrondi::nouveau(100.25, 80.75, 3.0e6, 2.0e6, 60.0 * 400.0);
    // Sur le bord gauche, loin des coins : la distance est l'écart horizontal, exactement.
    for dx in [-3.0_f32, -0.5, 0.0, 0.25, 2.0] {
        let d = f.distance(100.25 + dx, 80.75 + 30_000.0);
        assert!((d + dx).abs() < 0.01, "a {dx} du bord : {d}");
    }
}

/// **Les couvertures sont celles du filtre-boîte** : entière, nulle, et la moitié au bord.
#[test]
fn test_les_couvertures_sont_celles_du_filtre_boite() {
    assert_eq!(couverture_d_un_plein(-3.0), 1.0);
    assert_eq!(couverture_d_un_plein(3.0), 0.0);
    assert_eq!(couverture_d_un_plein(0.0), 0.5);
    assert_eq!(couverture_d_un_trait(0.0, 2.0), 1.0);
    assert_eq!(couverture_d_un_trait(2.0, 2.0), 0.5);
    assert_eq!(couverture_d_un_trait(3.0, 2.0), 0.0);
    // Un trait d'un quart de pixel ne couvre jamais qu'un quart du pixel.
    assert!((couverture_d_un_trait(0.0, 0.125) - 0.25).abs() < 1e-6);
}

/// **Une image qui ment sur sa taille, ou une membrane hors de l'image, ne dessine rien.**
#[test]
fn test_rien_ne_se_pose_hors_de_l_image_ni_sur_une_taille_fausse() {
    let m = membrane(10.0, 10.0, 100.0, 60.0, 1.0, false);
    let mut img = image(64, 48);
    assert!(!peindre(&m, &mut img, 64, 47));
    let loin = membrane(1_000.0, 1_000.0, 100.0, 60.0, 1.0, false);
    assert!(!peindre(&loin, &mut img, 64, 48));
    assert_eq!(img, image(64, 48));
}
