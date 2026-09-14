//! Ce que la rotation garantit — et les entrées dégénérées qu'elle ne doit pas prendre pour
//! des directions.

use super::*;
use crate::resize::Handle;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

const CENTRE: (f64, f64) = (100.0, 100.0);
/// Tolérance d'égalité d'angles, en radians — très en deçà d'un pixel sur un nœud d'un mètre.
const EPS: f64 = 1e-9;

fn proche(a: f64, b: f64) -> bool {
    (normalize(a - b)).abs() < EPS
}

/// ROT-1 — l'angle appliqué est la différence des azimuts, ajoutée à celui de départ.
#[test]
fn test_rot_1_un_quart_de_tour_de_la_main_fait_un_quart_de_tour_au_noeud() {
    // Saisi à droite du centre, emmené au-dessus : un quart de tour.
    let a = rotation_from_drag(CENTRE, (200.0, 100.0), (100.0, 200.0), 0.0, false);
    assert!(proche(a, FRAC_PI_2), "{a}");

    // Et cela s'ajoute à l'angle que le nœud avait déjà.
    let b = rotation_from_drag(CENTRE, (200.0, 100.0), (100.0, 200.0), FRAC_PI_4, false);
    assert!(proche(b, FRAC_PI_4 + FRAC_PI_2), "{b}");
}

/// Le geste ne s'accumule pas : le résultat ne dépend que de **où est** le curseur, jamais du
/// chemin qu'il a pris ni du nombre d'événements reçus.
#[test]
fn test_le_resultat_ne_depend_pas_du_nombre_devenements() {
    let saisi = (200.0, 100.0);
    let arrivee = (100.0, 0.0);
    let direct = rotation_from_drag(CENTRE, saisi, arrivee, 0.0, false);

    // Le même geste, « rejoué » depuis l'origine à chaque étape intermédiaire.
    let mut dernier = 0.0;
    for k in 1..=20 {
        let t = k as f64 / 20.0;
        let inter = (
            saisi.0 + (arrivee.0 - saisi.0) * t,
            saisi.1 + (arrivee.1 - saisi.1) * t,
        );
        dernier = rotation_from_drag(CENTRE, saisi, inter, 0.0, false);
    }
    assert!(proche(dernier, direct), "{dernier} au lieu de {direct}");
}

/// La contrainte de `Maj` verrouille sur les huit directions des poignées, et sur rien d'autre.
#[test]
fn test_la_contrainte_tombe_sur_les_huit_directions_des_poignees() {
    let cran = TAU / OCTANTS as f64;
    for k in 0..40 {
        // Des azimuts pris tout autour, y compris entre deux crans.
        let theta = k as f64 * TAU / 40.0;
        let pointer = (CENTRE.0 + 50.0 * theta.cos(), CENTRE.1 + 50.0 * theta.sin());
        let a = rotation_from_drag(CENTRE, (CENTRE.0 + 50.0, CENTRE.1), pointer, 0.0, true);
        let reste = normalize(a).rem_euclid(cran);
        assert!(
            reste < EPS || (cran - reste) < EPS,
            "θ={theta} donne {a}, qui n'est pas un multiple de {cran}"
        );
    }
}

/// Et il y en a bien huit, autant que de poignées : les deux nombres ne peuvent pas diverger.
#[test]
fn test_il_y_a_un_cran_par_poignee() {
    assert_eq!(OCTANTS, Handle::ALL.len());
}

/// Saisir le centre même n'a pas de direction : le nœud garde son angle plutôt que d'en
/// prendre un au hasard.
#[test]
fn test_saisir_le_centre_ne_fait_rien_tourner() {
    assert_eq!(
        rotation_from_drag(CENTRE, CENTRE, (200.0, 100.0), 0.3, false),
        0.3
    );
    assert_eq!(
        rotation_from_drag(CENTRE, (200.0, 100.0), CENTRE, 0.3, false),
        0.3
    );
}

#[test]
fn test_normalize_ramene_dans_un_tour_sans_changer_lorientation() {
    for tours in -3..=3 {
        for base in [0.0, FRAC_PI_4, FRAC_PI_2, 2.0, -2.0, PI - 1e-6] {
            let a = base + tours as f64 * TAU;
            let n = normalize(a);
            assert!((-PI..PI).contains(&n), "{a} → {n} sort de [-π, π)");
            assert!(proche(n, a), "{a} → {n} a changé d'orientation");
        }
    }
}

/// Un angle qui n'est pas un nombre ne doit pas se propager dans le document : le repliement
/// le ramène à zéro plutôt que d'écrire un `NaN` sur le disque.
#[test]
fn test_normalize_refuse_de_propager_un_angle_qui_nen_est_pas_un() {
    assert_eq!(normalize(f64::NAN), 0.0);
    assert_eq!(normalize(f64::INFINITY), 0.0);
    assert_eq!(normalize(f64::NEG_INFINITY), 0.0);
}

/// Un tour complet ramène le nœud là où il était, à la normalisation près.
#[test]
fn test_un_tour_complet_revient_au_point_de_depart() {
    let saisi = (200.0, 100.0);
    let a = rotation_from_drag(CENTRE, saisi, saisi, 1.2, false);
    assert!(proche(a, 1.2), "{a}");
}
