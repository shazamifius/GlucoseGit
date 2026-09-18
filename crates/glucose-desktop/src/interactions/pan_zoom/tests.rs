//! NAV-2 — les cas que `navigation.test.ts` tient côté Tauri, tenus ici aussi.

use super::*;
use winit::dpi::PhysicalPosition;

fn lignes(x: f32, y: f32) -> MouseScrollDelta {
    MouseScrollDelta::LineDelta(x, y)
}

fn pixels(x: f64, y: f64) -> MouseScrollDelta {
    MouseScrollDelta::PixelDelta(PhysicalPosition::new(x, y))
}

/// `Ctrl` + défilement : c'est un zoom, quel que soit le reste.
#[test]
fn test_nav_2_ctrl_is_a_zoom() {
    assert!(matches!(geste(lignes(0.0, 1.0), true, false), Geste::Zoom(_)));
    assert!(matches!(geste(pixels(3.0, -7.0), true, false), Geste::Zoom(_)));
}

/// Le pincement zoome **même sans `Ctrl` au clavier** : c'est le systeme qui le marque, et
/// c'était tout le défaut — le geste arrivait nu, donc il déplaçait la vue.
#[test]
fn test_nav_2_un_pincement_zoome_sans_ctrl_clavier() {
    assert!(matches!(geste(lignes(0.0, 0.07), false, true), Geste::Zoom(_)));
    assert!(matches!(geste(lignes(0.0, -0.07), false, true), Geste::Zoom(_)));
}

/// **Les deux gestes ont leur propre échelle, et c'est le fond de l'affaire.** Windows encode
/// le pincement et le glissement dans la même unité alors qu'ils mesurent deux choses
/// différentes — l'écartement des doigts d'un côté, leur course de l'autre.
///
/// Le test vérifie que les échelles sont bien **distinctes**, pas leur rapport : celui-ci est
/// un réglage de ressenti, il se juge à la main et doit pouvoir bouger sans casser une preuve.
#[test]
fn test_nav_2_un_pincement_a_sa_propre_echelle() {
    let Geste::Zoom(pince) = geste(lignes(0.0, 0.5), false, true) else {
        panic!("un pincement zoome");
    };
    let Geste::Zoom(molette) = geste(lignes(0.0, 0.5), true, false) else {
        panic!("ctrl zoome");
    };
    assert!(
        pince > molette,
        "{pince} octave(s) contre {molette} : un pincement ne se lit pas comme un cran"
    );
}

/// Un cran de souris — vertical pur, nombre entier de lignes — zoome.
#[test]
fn test_nav_2_a_mouse_notch_zooms() {
    assert!(matches!(geste(lignes(0.0, 1.0), false, false), Geste::Zoom(_)));
    assert!(matches!(geste(lignes(0.0, -1.0), false, false), Geste::Zoom(_)));
    assert!(matches!(geste(lignes(0.0, 3.0), false, false), Geste::Zoom(_)));
}

/// Deux doigts sur le pavé tactile déplacent la vue — **y compris vers le haut et le bas**.
///
/// C'est le cas que l'ancienne version rendait inatteignable : sous Windows, un pavé tactile
/// passe par `LineDelta` comme une souris, avec des fractions de ligne.
#[test]
fn test_nav_2_two_fingers_pan_in_every_direction() {
    assert!(matches!(geste(lignes(0.0, 0.42), false, false), Geste::Pan(_, _)));
    assert!(matches!(geste(lignes(0.0, -0.13), false, false), Geste::Pan(_, _)));
    assert!(matches!(geste(lignes(0.7, 0.0), false, false), Geste::Pan(_, _)));
    assert!(matches!(geste(pixels(0.0, 24.0), false, false), Geste::Pan(_, _)));
}

/// Une composante horizontale dénonce un pavé tactile, même si le vertical tombe juste.
#[test]
fn test_nav_2_a_whole_line_with_sideways_motion_is_still_a_pan() {
    assert!(matches!(geste(lignes(0.5, 1.0), false, false), Geste::Pan(_, _)));
}

/// Un événement vide ne fait rien plutôt que de zoomer par ×1.
#[test]
fn test_nav_2_an_empty_event_moves_nothing() {
    assert_eq!(geste(lignes(0.0, 0.0), false, false), Geste::Pan(0.0, 0.0));
}

/// Le sens : molette vers l'avant agrandit, vers soi réduit.
#[test]
fn test_nav_2_forward_grows_and_backward_shrinks() {
    let Geste::Zoom(avant) = geste(lignes(0.0, 1.0), false, false) else {
        panic!("un cran doit zoomer");
    };
    let Geste::Zoom(arriere) = geste(lignes(0.0, -1.0), false, false) else {
        panic!("un cran doit zoomer");
    };
    assert!(avant > 0.0, "vers l'avant, on agrandit : {avant} octave(s)");
    assert!(arriere < 0.0, "vers soi, on réduit : {arriere} octave(s)");
    // **Ce que l'octave fait gagner** : deux crans valent la somme de deux crans, et non le
    // carré d'un facteur. Le zoom devient additif, donc indépendant du découpage des
    // événements reçus -- et l'élan peut les accumuler sans rien trahir.
    let Geste::Zoom(deux) = geste(lignes(0.0, 2.0), false, false) else {
        panic!("un cran doit zoomer");
    };
    assert!((deux - 2.0 * avant).abs() < 1e-12);
}

/// **Huit crans doublent.** C'est ce que l'octave permet de dire, et de vérifier.
#[test]
fn test_nav_2_huit_crans_doublent_exactement() {
    let Geste::Zoom(un) = geste(lignes(0.0, 1.0), false, false) else {
        panic!("un cran doit zoomer");
    };
    assert!(
        (8.0 * un - 1.0).abs() < 1e-12,
        "huit crans doivent faire une octave pleine, ils font {}",
        8.0 * un
    );
}

/// Deux doigts vers le bas font descendre le contenu — le signe de winit est celui du monde.
#[test]
fn test_nav_2_the_content_follows_the_fingers() {
    let Geste::Pan(dx, dy) = geste(lignes(0.0, 0.5), false, false) else {
        panic!("un glissement doit paner");
    };
    assert_eq!(dx, 0.0);
    assert_eq!(dy, 0.5 * PAN_LIGNE_PX);
}
