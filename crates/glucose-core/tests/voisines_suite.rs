//! **SNAP-4 — l'aimant ne prend que les voisines** : la plus proche au-dessus, au-dessous, à
//! gauche, à droite, et la plus intime des boîtes qui entourent.
//!
//! Chaque scène est tirée deux fois : par les voisines, et par toutes les boîtes — l'aimant
//! d'avant. Là où les deux diffèrent, c'est la limitation qu'on éprouve.

use glucose_core::smart_align::{
    snap_move_sur, AlignKind, AlignRect, AlignTarget, Lignes, SnapOptions,
};

fn boite(left: f64, top: f64, width: f64, height: f64) -> AlignRect {
    AlignRect::new(left, top, width, height)
}

/// Ce que l'aimant fait de `rect` à l'échelle 1 : `(dx, dy)` parmi les voisines, puis parmi
/// toutes les boîtes.
fn tirer(rect: AlignRect, boites: &[AlignRect]) -> ((f64, f64), (f64, f64)) {
    let opts = SnapOptions::default();
    let voisines = Lignes::des_voisines(rect, boites, opts.seuil());
    let cibles: Vec<AlignTarget> = boites
        .iter()
        .map(|&rect| AlignTarget {
            id: String::new(),
            kind: AlignKind::Image,
            rect,
        })
        .collect();
    let par_les_voisines = snap_move_sur(rect, &voisines, opts);
    let par_toutes = snap_move_sur(rect, &Lignes::des(&cibles), opts);
    (
        (par_les_voisines.dx, par_les_voisines.dy),
        (par_toutes.dx, par_toutes.dy),
    )
}

/// **La voisine cache ce qui est derrière elle** : au-dessus de `rect`, A à trois unités
/// d'alignement, et plus haut B, à deux. Toutes les boîtes tiraient vers B ; seule A tire.
#[test]
fn test_snap_4_la_voisine_cache_ce_qui_est_derriere() {
    let rect = boite(3.0, 300.0, 100.0, 100.0);
    let a = boite(0.0, 150.0, 100.0, 100.0);
    let b = boite(5.0, 0.0, 100.0, 100.0);
    let (voisines, toutes) = tirer(rect, &[a, b]);
    assert_eq!(voisines, (-3.0, 0.0), "A, la voisine du dessus");
    assert_eq!(toutes, (2.0, 0.0), "l'aimant d'avant allait chercher B");
}

/// **Une voisine d'à côté ne prend pas la place de celle du dessous** : L touche la colonne de
/// `rect` au seuil près, mais c'est son bord droit qui fait face — pas son haut. B, dessous, à
/// trois unités d'alignement, reste la voisine du dessous ; si L lui prenait sa place,
/// l'aimant tirerait de cinq vers L.
#[test]
fn test_snap_4_une_voisine_d_a_cote_ne_prend_pas_la_place_de_celle_du_dessous() {
    let rect = boite(100.0, 100.0, 100.0, 100.0);
    let l = boite(0.0, 100.0, 95.0, 100.0);
    let b = boite(103.0, 210.0, 100.0, 100.0);
    let (voisines, _) = tirer(rect, &[l, b]);
    assert_eq!(voisines, (3.0, 0.0), "B tire, L aligne la rangée");
}

/// **À écart égal, la plus en face l'emporte** : une rangée de photos au-dessus, leurs bas
/// alignés — la grille. `rect` est surtout sous B : c'est B qui tire, pas A qui vient en
/// premier.
#[test]
fn test_snap_4_a_ecart_egal_la_plus_en_face_l_emporte() {
    let rect = boite(95.0, 300.0, 100.0, 100.0);
    let a = boite(0.0, 100.0, 100.0, 100.0);
    let b = boite(148.0, 100.0, 100.0, 100.0);
    let (voisines, _) = tirer(rect, &[a, b]);
    assert_eq!(voisines, (3.0, 0.0), "B, la plus en face");
}

/// **Ce qui ne fait que recouvrir n'aimante pas** : P chevauche `rect` sans qu'aucun de ses bords
/// fasse face à un bord de `rect`. L'aimant d'avant le prenait.
#[test]
fn test_snap_4_ce_qui_ne_fait_que_recouvrir_n_aimante_pas() {
    let rect = boite(3.0, 500.0, 100.0, 100.0);
    let p = boite(50.0, 450.0, 100.0, 100.0);
    let (voisines, toutes) = tirer(rect, &[p]);
    assert_eq!(voisines, (0.0, 0.0));
    assert_eq!(toutes, (-3.0, 0.0));
}

/// **La plus intime des boîtes qui entourent aimante, et elle seule** : on range dans une
/// membrane, et l'on s'aligne sur elle. M2, dans M1, entoure `rect` : son bord gauche tire ;
/// le milieu de M1, pourtant à une unité, ne tire pas.
#[test]
fn test_snap_4_la_plus_intime_des_boites_qui_entourent_aimante() {
    let rect = boite(10.0, 300.0, 100.0, 100.0);
    let m1 = boite(0.0, 0.0, 1000.0, 702.0);
    let m2 = boite(5.0, 290.0, 300.0, 300.0);
    let (voisines, toutes) = tirer(rect, &[m1, m2]);
    assert_eq!(voisines, (-5.0, 0.0), "le bord de M2");
    assert_eq!(
        toutes,
        (-5.0, 1.0),
        "l'aimant d'avant prenait aussi le milieu de M1"
    );
}

/// **L'aimant tire des deux côtés d'un bord** : le bord gauche de `rect` à cinq unités du bord
/// droit de A, au-dessus — qu'il le dépasse ou qu'il n'y soit pas encore. Sans la colonne élargie
/// du seuil, A ne serait sa voisine que d'un côté.
#[test]
fn test_snap_4_l_aimant_tire_des_deux_cotes_d_un_bord() {
    let a = boite(0.0, 0.0, 100.0, 100.0);
    let (en_deca, _) = tirer(boite(95.0, 150.0, 100.0, 100.0), &[a]);
    let (au_dela, _) = tirer(boite(105.0, 150.0, 100.0, 100.0), &[a]);
    assert_eq!((en_deca.0, au_dela.0), (5.0, -5.0));
}
