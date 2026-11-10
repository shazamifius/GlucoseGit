//! **RETOUR-2 — `Alt+Z` est le `Ctrl+Z` de la caméra**, par les vrais chemins : la molette et la
//! main passent par l'élan, et c'est l'image qui dit quand la vue bouge et quand elle s'arrête.

use crate::app::GlucoseApp;
use crate::commandes::Commande;
use glucose_core::types::Viewport;
use winit::event::MouseScrollDelta;

/// Une application au tableau vide, l'accueil éteint, la vue posée en un lieu connu, et une image
/// au repos pour que la vue du repos soit celle-là.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let tableau = app.store.project.active_board_id.clone();
    app.store.set_viewport(&tableau, depart());
    image(&mut app);
    app
}

fn depart() -> Viewport {
    Viewport {
        x: 40.0,
        y: 30.0,
        scale: 1.0,
    }
}

/// Une image comme la boucle la joue, une période d'écran à 240 Hz après la précédente.
fn image(app: &mut GlucoseApp) {
    let periode = std::time::Duration::from_micros(4_167);
    app.horloge.accorder(periode);
    let t = app
        .horloge
        .derniere_presentation()
        .unwrap_or_else(std::time::Instant::now);
    app.horloge.presentee(t + periode, true);
    app.appliquer_l_elan(800, 600);
}

/// Des images jusqu'à ce que la vue s'arrête, et une de plus, immobile : le geste est fini.
fn jusqu_au_repos(app: &mut GlucoseApp) {
    for _ in 0..2_000 {
        image(app);
        if !app.elan.bouge() && !app.vol.en_cours() {
            image(app);
            return;
        }
    }
    panic!("la vue ne s'arrête pas");
}

/// Un cran de molette, et l'image qui le montre.
fn cran(app: &mut GlucoseApp) {
    app.handle_mouse_wheel(MouseScrollDelta::LineDelta(0.0, 1.0));
    image(app);
}

fn alt_z(app: &mut GlucoseApp) -> bool {
    app.executer(Commande::VuePrecedente)
}

/// **Un tour de molette se défait** : `Alt+Z` rend la vue d'avant, au bit près.
#[test]
fn test_retour_2_un_tour_de_molette_se_defait() {
    let mut app = app();
    cran(&mut app);
    jusqu_au_repos(&mut app);
    assert_ne!(app.store.viewport(), depart(), "la molette a zoomé");
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart());
}

/// **Une rafale ne fait qu'un pas** : trois crans sans image immobile entre eux — ce que la main
/// fait en tournant la molette — se défont d'un seul `Alt+Z` ; il n'y a rien avant.
#[test]
fn test_retour_2_une_rafale_ne_fait_qu_un_pas() {
    let mut app = app();
    for _ in 0..3 {
        cran(&mut app);
    }
    jusqu_au_repos(&mut app);
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart());
    assert!(!alt_z(&mut app), "un seul pas");
}

/// **Deux gestes séparés par un arrêt, deux pas** : la molette, puis la main qui déplace la vue.
#[test]
fn test_retour_2_deux_gestes_deux_pas() {
    let mut app = app();
    cran(&mut app);
    jusqu_au_repos(&mut app);
    let apres_la_molette = app.store.viewport();
    app.is_panning = true;
    app.handle_pan_move(30.0, 12.0);
    image(&mut app);
    app.is_panning = false;
    jusqu_au_repos(&mut app);
    assert_ne!(
        app.store.viewport(),
        apres_la_molette,
        "la main a déplacé la vue"
    );
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), apres_la_molette, "d'abord le glisser");
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart(), "puis la molette");
    assert!(app.executer(Commande::VueSuivante));
    assert_eq!(app.store.viewport(), apres_la_molette, "Alt+Maj+Z repart");
}

/// **Un vol ne se note pas deux fois** : son départ est déjà retenu (RETOUR-1).
#[test]
fn test_retour_2_un_vol_ne_se_note_pas_deux_fois() {
    let mut app = app();
    app.vol.voler_vers(Viewport {
        x: 900.0,
        y: 30.0,
        scale: 1.0,
    });
    jusqu_au_repos(&mut app);
    assert!(alt_z(&mut app));
    assert_eq!(app.store.viewport(), depart());
    assert!(!alt_z(&mut app), "un seul pas");
}
