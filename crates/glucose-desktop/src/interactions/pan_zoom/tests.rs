//! NAV-2 — les cas que `navigation.test.ts` tient côté Tauri, tenus ici aussi.

use super::*;
use winit::dpi::PhysicalPosition;

fn lignes(x: f32, y: f32) -> MouseScrollDelta {
    MouseScrollDelta::LineDelta(x, y)
}

fn pixels(x: f64, y: f64) -> MouseScrollDelta {
    MouseScrollDelta::PixelDelta(PhysicalPosition::new(x, y))
}

/// Le pincement arrive en `Ctrl` + défilement : c'est un zoom, quel que soit le reste.
#[test]
fn test_nav_2_pinch_is_a_zoom() {
    assert!(matches!(geste(lignes(0.0, 1.0), true), Geste::Zoom(_)));
    assert!(matches!(geste(pixels(3.0, -7.0), true), Geste::Zoom(_)));
}

/// Un cran de souris — vertical pur, nombre entier de lignes — zoome.
#[test]
fn test_nav_2_a_mouse_notch_zooms() {
    assert!(matches!(geste(lignes(0.0, 1.0), false), Geste::Zoom(_)));
    assert!(matches!(geste(lignes(0.0, -1.0), false), Geste::Zoom(_)));
    assert!(matches!(geste(lignes(0.0, 3.0), false), Geste::Zoom(_)));
}

/// Deux doigts sur le pavé tactile déplacent la vue — **y compris vers le haut et le bas**.
///
/// C'est le cas que l'ancienne version rendait inatteignable : sous Windows, un pavé tactile
/// passe par `LineDelta` comme une souris, avec des fractions de ligne.
#[test]
fn test_nav_2_two_fingers_pan_in_every_direction() {
    assert!(matches!(geste(lignes(0.0, 0.42), false), Geste::Pan(_, _)));
    assert!(matches!(geste(lignes(0.0, -0.13), false), Geste::Pan(_, _)));
    assert!(matches!(geste(lignes(0.7, 0.0), false), Geste::Pan(_, _)));
    assert!(matches!(geste(pixels(0.0, 24.0), false), Geste::Pan(_, _)));
}

/// Une composante horizontale dénonce un pavé tactile, même si le vertical tombe juste.
#[test]
fn test_nav_2_a_whole_line_with_sideways_motion_is_still_a_pan() {
    assert!(matches!(geste(lignes(0.5, 1.0), false), Geste::Pan(_, _)));
}

/// Un événement vide ne fait rien plutôt que de zoomer par ×1.
#[test]
fn test_nav_2_an_empty_event_moves_nothing() {
    assert_eq!(geste(lignes(0.0, 0.0), false), Geste::Pan(0.0, 0.0));
}

/// Le sens : molette vers l'avant agrandit, vers soi réduit.
#[test]
fn test_nav_2_forward_grows_and_backward_shrinks() {
    let Geste::Zoom(avant) = geste(lignes(0.0, 1.0), false) else {
        panic!("un cran doit zoomer");
    };
    let Geste::Zoom(arriere) = geste(lignes(0.0, -1.0), false) else {
        panic!("un cran doit zoomer");
    };
    assert!(avant > 1.0, "vers l'avant, on agrandit : {avant}");
    assert!(arriere < 1.0, "vers soi, on réduit : {arriere}");
    // Deux crans valent exactement deux fois un cran, pas davantage : le zoom est continu,
    // donc composable, donc indépendant du découpage des événements reçus.
    let Geste::Zoom(deux) = geste(lignes(0.0, 2.0), false) else {
        panic!("un cran doit zoomer");
    };
    assert!((deux - avant * avant).abs() < 1e-9);
}

/// Un cran de souris reste doux : la version précédente sautait de 12 % d'un coup.
#[test]
fn test_nav_2_a_notch_is_a_small_step() {
    let Geste::Zoom(f) = geste(lignes(0.0, 1.0), false) else {
        panic!("un cran doit zoomer");
    };
    assert!((1.03..1.05).contains(&f), "un cran vaut {f}");
}

/// Deux doigts vers le bas font descendre le contenu — le signe de winit est celui du monde.
#[test]
fn test_nav_2_the_content_follows_the_fingers() {
    let Geste::Pan(dx, dy) = geste(lignes(0.0, 0.5), false) else {
        panic!("un glissement doit paner");
    };
    assert_eq!(dx, 0.0);
    assert_eq!(dy, 0.5 * PAN_LIGNE_PX);
}
