//! **Le groupe se transforme par ses coins** (fiche 53 § 10), à la vraie souris : appui sur un
//! coin du cadre du groupe, mouvement, relâchement — autour de l'origine commune ou de celles
//! de chaque nœud.

use super::tests::{app, drag_by, image_box, press_handle, release, with_image};
use super::*;
use glucose_core::groupe::Origine;
use winit::keyboard::ModifiersState;

/// Deux images de 100 × 100 centrées en (−200, 0) et (200, 0), choisies ensemble : le
/// groupe va de (−250, −50) à (250, 50).
fn deux_images() -> GlucoseApp {
    let mut app = app();
    with_image(&mut app, "a", -200.0, 0.0, 100.0, 100.0);
    with_image(&mut app, "b", 200.0, 0.0, 100.0, 100.0);
    app.store
        .set_selected_image_ids(vec!["a".into(), "b".into()]);
    app.store.journal.clear();
    app
}

fn centre(app: &GlucoseApp, id: &str) -> (f64, f64) {
    let r = image_box(app, id);
    (r.left + r.width / 2.0, r.top + r.height / 2.0)
}

fn proches(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6
}

/// **Origine commune** : tirer le coin bas-droit du groupe de 500 le double — le coin
/// haut-gauche reste, chaque image double et s'éloigne de lui ; et c'est un seul geste.
#[test]
fn test_tirer_le_coin_du_groupe_le_met_a_l_echelle_autour_du_coin_oppose() {
    let mut app = deux_images();
    let groupe = app.emprise_du_groupe().expect("un groupe");
    assert_eq!(groupe, AlignRect::new(-250.0, -50.0, 500.0, 100.0));
    press_handle(&mut app, groupe, Handle::BottomRight);
    assert!(
        matches!(
            app.resize_session.as_ref().map(|s| &s.target),
            Some(ResizeTarget::Groupe { .. })
        ),
        "le coin du groupe ouvre le geste du groupe"
    );
    drag_by(&mut app, 500.0, 100.0, 5);
    release(&mut app);

    assert!(
        proches(centre(&app, "a"), (-150.0, 50.0)),
        "{:?}",
        centre(&app, "a")
    );
    assert!(
        proches(centre(&app, "b"), (650.0, 50.0)),
        "{:?}",
        centre(&app, "b")
    );
    assert_eq!(image_box(&app, "a").width, 200.0);
    assert_eq!(app.store.undo_depth(), 1, "un seul geste");
}

/// **Origines individuelles** : le même geste double chaque image sur place.
#[test]
fn test_en_origines_individuelles_chaque_image_grandit_sur_place() {
    let mut app = deux_images();
    app.ui.origine_du_groupe = Origine::Individuelle;
    let groupe = app.emprise_du_groupe().expect("un groupe");
    press_handle(&mut app, groupe, Handle::BottomRight);
    drag_by(&mut app, 500.0, 100.0, 5);
    release(&mut app);

    assert!(proches(centre(&app, "a"), (-200.0, 0.0)));
    assert!(proches(centre(&app, "b"), (200.0, 0.0)));
    assert_eq!(image_box(&app, "b").width, 200.0);
}

/// **`Alt` + coin du groupe le fait tourner** autour de son centre : en commun, les images
/// tournent autour de lui ; en individuel, chacune tourne sur place.
#[test]
fn test_alt_sur_un_coin_du_groupe_le_fait_tourner() {
    // La prise (250, −50) amenée en (−50, −250) : le même quart de tour, autour du centre
    // (0, 0), qui envoie (x, y) en (y, −x) — donc `a`, en (−200, 0), en (0, 200).
    for (origine, attendu_a) in [
        (Origine::Commune, (0.0, 200.0)),
        (Origine::Individuelle, (-200.0, 0.0)),
    ] {
        let mut app = deux_images();
        app.ui.origine_du_groupe = origine;
        let groupe = app.emprise_du_groupe().expect("un groupe");
        app.modifiers = ModifiersState::ALT;
        press_handle(&mut app, groupe, Handle::TopRight);
        drag_by(&mut app, -300.0, -200.0, 6);
        release(&mut app);
        app.modifiers = ModifiersState::empty();
        let a = app
            .store
            .active_board()
            .and_then(|b| b.images.iter().find(|i| i.id == "a"))
            .expect("a")
            .clone();
        assert!(
            (a.rotation.abs() - std::f64::consts::FRAC_PI_2).abs() < 1e-6,
            "{origine:?} : un quart de tour, {}",
            a.rotation
        );
        let c = (a.x, a.y);
        assert!(
            (c.0 - attendu_a.0).abs() < 1e-6 && (c.1 - attendu_a.1).abs() < 1e-6,
            "{origine:?} : {c:?}"
        );
    }
}

/// **Une image seule garde ses propres poignées** : il n'y a pas de groupe d'un.
#[test]
fn test_une_image_seule_n_a_pas_de_groupe() {
    let mut app = deux_images();
    app.store.set_selected_image_ids(vec!["a".into()]);
    assert_eq!(app.emprise_du_groupe(), None);
}
