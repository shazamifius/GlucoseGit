use super::*;
use std::f64::consts::PI;

/// Un nœud posé avant EXIF-1 sur une photo de 4 000 × 3 000 pixels couchés : sa boîte et ses
/// dimensions d'origine en paysage ; décodée aujourd'hui, la photo est debout, 3 000 × 4 000.
fn couche(rotation: f64, miroir: bool) -> BoardImage {
    let mut img = BoardImage::new("i", 50.0, 70.0, 400.0, 300.0);
    (img.original_width, img.original_height) = (4000.0, 3000.0);
    img.rotation = rotation;
    img.miroir = miroir;
    img
}

const DEBOUT: (u32, u32) = (3000, 4000);

/// La transformation `R(θ)·M^m` d'un nœud, en matrice : le miroir d'abord, puis la rotation.
fn matrice(rotation: f64, miroir: bool) -> [[f64; 2]; 2] {
    let (s, c) = rotation.sin_cos();
    let m = if miroir { -1.0 } else { 1.0 };
    [[c * m, -s], [s * m, c]]
}

fn produit(a: [[f64; 2]; 2], b: [[f64; 2]; 2]) -> [[f64; 2]; 2] {
    let case = |i: usize, j: usize| a[i][0] * b[0][j] + a[i][1] * b[1][j];
    [[case(0, 0), case(0, 1)], [case(1, 0), case(1, 1)]]
}

fn proches(a: [[f64; 2]; 2], b: [[f64; 2]; 2]) -> bool {
    a.iter()
        .flatten()
        .zip(b.iter().flatten())
        .all(|(x, y)| (x - y).abs() < 1e-6)
}

/// **Chaque nœud redressé montre ce qu'il doit** : ou sa rotation est restée la sienne (la photo
/// paraît enfin debout), ou ce qu'il voyait n'a pas changé — `R(θ')·M^m'·O = R(θ)·M^m` —, et il
/// garde des deux le moins tourné. Les huit miroirs et pentes, sur les quatre orientations qui
/// échangent largeur et hauteur.
#[test]
fn test_un_noeud_redresse_garde_la_lecture_la_moins_tournee() {
    let pentes = [
        0.0,
        0.3,
        FRAC_PI_2,
        -FRAC_PI_2,
        PI - 0.2,
        FRAC_PI_2 + 0.25,
        -2.0,
        1.0,
    ];
    for exif in 5..=8 {
        let (quarts, mu) = transformation(exif).expect("connue");
        let o = matrice(quarts as f64 * FRAC_PI_2, mu);
        for theta in pentes {
            for m in [false, true] {
                let mut img = couche(theta, m);
                assert!(img.redresser(exif, DEBOUT), "{exif} : couché");
                let garde = (img.rotation, img.miroir) == (theta, m);
                let voit_pareil = proches(
                    produit(matrice(img.rotation, img.miroir), o),
                    matrice(theta, m),
                );
                assert!(garde || voit_pareil, "{exif}, {theta}, {m}");
                let autre = if garde {
                    // L'autre lecture : le nœud qui verrait la même chose qu'avant — un quart de
                    // plus ou de moins, puisque l'orientation en échange un.
                    [normalize(theta + FRAC_PI_2), normalize(theta - FRAC_PI_2)]
                        .into_iter()
                        .filter(|&r| {
                            [false, true]
                                .iter()
                                .any(|&mm| proches(produit(matrice(r, mm), o), matrice(theta, m)))
                        })
                        .map(f64::abs)
                        .fold(f64::INFINITY, f64::min)
                } else {
                    normalize(theta).abs()
                };
                assert!(
                    normalize(img.rotation).abs() <= autre + 1e-9,
                    "{exif}, {theta}, {m} : le moins tourné ({} contre {autre})",
                    img.rotation
                );
            }
        }
    }
}

/// **Jamais tourné, il paraît debout** : sa boîte et ses dimensions d'origine passent en portrait
/// autour du même centre, sa rotation reste nulle.
#[test]
fn test_un_noeud_jamais_tourne_paraît_debout() {
    let mut img = couche(0.0, false);
    assert!(img.redresser(6, DEBOUT));
    assert_eq!(
        (img.x, img.y, img.width, img.height),
        (50.0, 70.0, 300.0, 400.0)
    );
    assert_eq!((img.original_width, img.original_height), (3000.0, 4000.0));
    assert_eq!((img.rotation, img.miroir), (0.0, false));
    assert!(
        !img.redresser(6, DEBOUT),
        "redressé une fois, il ne l'est plus"
    );
}

/// **Redressé à la main d'un quart de tour, il revient exactement à zéro** — même rangé en `f32`
/// par le document, où ce quart n'est pas `π/2` ; ce qu'il voyait ne change pas.
#[test]
fn test_un_noeud_redresse_a_la_main_revient_a_zero() {
    for exif in 5..=8 {
        let (quarts, miroir) = transformation(exif).expect("connue");
        let rangee = normalize(quarts as f64 * FRAC_PI_2) as f32 as f64;
        let mut img = couche(rangee, miroir);
        assert!(img.redresser(exif, DEBOUT));
        assert_eq!((img.rotation, img.miroir), (0.0, false), "{exif}");
    }
    // Redressé puis penché : il garde sa pente.
    let mut img = couche(FRAC_PI_2 + 0.1, false);
    img.redresser(6, DEBOUT);
    assert!((img.rotation - 0.1).abs() < 1e-9, "{}", img.rotation);
}

/// **Rien d'autre ne bouge** : une photo déjà debout, un carré, une orientation qui n'échange
/// rien, une orientation inconnue.
#[test]
fn test_seul_un_noeud_couche_se_redresse() {
    let tel_quel = |mut img: BoardImage, exif: u8, pixels: (u32, u32)| {
        let avant = img.clone();
        assert!(!img.redresser(exif, pixels), "{exif}");
        assert_eq!(img, avant);
    };
    let mut debout = couche(0.0, false);
    (debout.original_width, debout.original_height) = (3000.0, 4000.0);
    tel_quel(debout, 6, DEBOUT);
    let mut carre = couche(0.0, false);
    (carre.original_width, carre.original_height) = (3000.0, 3000.0);
    tel_quel(carre, 6, (3000, 3000));
    for exif in [0, 1, 2, 3, 4, 9] {
        tel_quel(couche(0.0, false), exif, DEBOUT);
    }
}

/// **Le recadrage suit les pixels** : sur une photo tournée d'un quart, ce qu'on retirait du bas
/// se retire à gauche ; chaque orientation suivie de son inverse rend le cadrage d'origine.
#[test]
fn test_le_recadrage_suit_ses_bords() {
    let cadre = Recadrage::depuis_les_marges(0.1, 0.2, 0.3, 0.05);
    assert_eq!(
        cadre.oriente(6).marges(),
        (0.05, 0.1, 0.2, 0.3),
        "le bas passe à gauche, la gauche en haut"
    );
    let inverse = |exif: u8| match exif {
        6 => 8,
        8 => 6,
        e => e,
    };
    for exif in 1..=8 {
        assert_eq!(cadre.oriente(exif).oriente(inverse(exif)), cadre, "{exif}");
    }
    let mut img = couche(0.0, false);
    img.crop = cadre;
    img.redresser(6, DEBOUT);
    assert_eq!(img.crop, cadre.oriente(6));
}
