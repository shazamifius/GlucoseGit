//! **MIROIR-1 — retourner une image** (fiche 118) : un seul interrupteur, la rotation qui se
//! reflète, et un geste qui se défait. Sa capture du 17/11 : *« crée une option de flip pour
//! les images »*.

use glucose_core::store::Store;
use glucose_core::types::{BoardImage, Retournement};
use std::f64::consts::{FRAC_PI_6, PI};

/// **Ce que l'image fait à un point de son contenu**, à l'écran : `R(θ)·M^m` — le miroir
/// d'abord, la rotation ensuite —, en matrice 2 × 2 par lignes.
fn matrice(img: &BoardImage) -> [[f64; 2]; 2] {
    let (c, s) = (img.rotation.cos(), img.rotation.sin());
    let m = if img.miroir { -1.0 } else { 1.0 };
    [[c * m, -s], [s * m, c]]
}

fn egales(a: [[f64; 2]; 2], b: [[f64; 2]; 2]) -> bool {
    a.iter()
        .flatten()
        .zip(b.iter().flatten())
        .all(|(x, y)| (x - y).abs() < 1e-12)
}

/// **Retourner, c'est refléter l'image à l'écran** : de gauche à droite, chaque point passe de
/// `x` à `−x` ; de haut en bas, de `y` à `−y` — après tout ce que l'image faisait déjà, une
/// rotation et un miroir compris. Et deux fois le même sens rendent l'image.
#[test]
fn test_miroir_1_retourner_reflete_l_image_a_l_ecran() {
    for theta in [0.0, FRAC_PI_6, -2.0, PI / 2.0, PI] {
        for miroir in [false, true] {
            let mut img = BoardImage::new("i", 0.0, 0.0, 100.0, 50.0);
            (img.rotation, img.miroir) = (theta, miroir);
            let [[a, b], [c, d]] = matrice(&img);

            let mut h = img.clone();
            h.retourner(Retournement::Horizontal);
            assert!(
                egales(matrice(&h), [[-a, -b], [c, d]]),
                "de gauche à droite, à {theta} rad, miroir {miroir}"
            );
            let mut v = img.clone();
            v.retourner(Retournement::Vertical);
            assert!(
                egales(matrice(&v), [[a, b], [-c, -d]]),
                "de haut en bas, à {theta} rad, miroir {miroir}"
            );

            for (mut deux_fois, sens) in
                [(h, Retournement::Horizontal), (v, Retournement::Vertical)]
            {
                deux_fois.retourner(sens);
                assert_eq!(deux_fois.miroir, miroir, "deux fois {sens:?}");
                assert!(
                    egales(matrice(&deux_fois), matrice(&img)),
                    "deux fois {sens:?}"
                );
            }
        }
    }
}

/// **Le geste retourne chaque image choisie sur elle-même, épargne la verrouillée, et se
/// défait d'un `Ctrl+Z`** ; sa place ne bouge pas.
#[test]
fn test_miroir_1_le_geste_retourne_sauf_le_verrou_et_se_defait() {
    let mut store = Store::new("miroir");
    let board = store.project.active_board_id.clone();
    store.add_image(&board, BoardImage::new("a", 10.0, 20.0, 100.0, 50.0));
    let mut fermee = BoardImage::new("v", 300.0, 20.0, 100.0, 50.0);
    fermee.locked = true;
    store.add_image(&board, fermee);
    store.journal.clear();

    store.begin_live_edit();
    store.retourner_les_images(&board, &["a".into(), "v".into()], Retournement::Vertical);
    store.end_live_edit();
    let a = store.image(&board, "a").expect("a").clone();
    assert!(a.miroir, "retournée");
    assert_eq!(
        (a.x, a.y, a.width, a.height),
        (10.0, 20.0, 100.0, 50.0),
        "sur place"
    );
    assert!(
        !store.image(&board, "v").expect("v").miroir,
        "le verrou la garde"
    );

    store.undo();
    let a = store.image(&board, "a").expect("a");
    assert!(!a.miroir && a.rotation == 0.0, "un seul Ctrl+Z la rend");
}

/// **Recadrer une image retournée par le bord qu'on voit** : tirer son bord gauche à l'écran
/// rogne le bord DROIT de la photo, et la boîte se resserre par la gauche — comme pour une image
/// à l'endroit, vue dans un miroir.
#[test]
fn test_miroir_1_le_bord_vu_d_une_image_retournee_rogne_l_autre_cote_de_la_photo() {
    use glucose_core::resize::Handle;
    use glucose_core::types::Recadrage;
    let boite = (100.0, 50.0, 200.0, 80.0);
    let (droit, a_l_endroit) =
        Recadrage::ENTIER.en_tirant_le_bord_vu(Handle::Left, (20.0, 0.0), boite, false);
    let (retourne, vu) =
        Recadrage::ENTIER.en_tirant_le_bord_vu(Handle::Left, (20.0, 0.0), boite, true);
    let (g, _, d, _) = retourne.marges();
    assert!(
        g == 0.0 && (d - 0.1).abs() < 1e-12,
        "le bord droit de la photo : {g}, {d}"
    );
    assert_eq!(droit.marges().0, 0.1, "à l'endroit, son bord gauche");
    assert_eq!(
        vu, a_l_endroit,
        "la même boîte à l'écran, resserrée par la gauche"
    );
    assert_eq!(vu, (120.0, 50.0, 180.0, 80.0));
}
