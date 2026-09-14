//! Les gestes de la vague 1.B joués au clavier, sur le **vrai** `GlucoseApp` : verrouiller,
//! empiler, pousser d'un cran.
//!
//! Le noyau les tient déjà sans écran (`lock_suite`, `store::order::tests`). Ce qui se vérifie
//! ici est l'autre moitié : que la touche arrive, qu'elle vise le board actif, et qu'aucune
//! ne se fait voler par une famille de raccourcis voisine — `L` par un outil, `Ctrl+[` par
//! l'édition.

use super::*;
use glucose_core::types::BoardImage;
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState, NamedKey, SmolStr};

/// Un board de `n` images superposées à l'origine, toutes sélectionnées.
fn app_with(n: usize) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    for k in 0..n {
        app.store.add_image(
            &board,
            BoardImage::new(format!("i{k}"), 0.0, 0.0, 100.0, 100.0),
        );
    }
    app.store
        .set_selected_image_ids((0..n).map(|k| format!("i{k}")).collect());
    app.store.journal.clear();
    app
}

fn touche(app: &mut GlucoseApp, c: &str, mods: ModifiersState) {
    app.modifiers = mods;
    app.handle_shortcut_input(&Key::Character(SmolStr::new(c)), ElementState::Pressed);
}

fn fleche(app: &mut GlucoseApp, key: NamedKey, mods: ModifiersState) {
    app.modifiers = mods;
    app.handle_shortcut_input(&Key::Named(key), ElementState::Pressed);
}

fn image(app: &GlucoseApp, id: &str) -> BoardImage {
    app.store
        .active_board()
        .and_then(|b| b.images.iter().find(|i| i.id == id).cloned())
        .expect("l'image existe")
}

fn ordre(app: &GlucoseApp) -> Vec<String> {
    app.store
        .active_board()
        .map(|b| b.images.iter().map(|i| i.id.clone()).collect())
        .unwrap_or_default()
}

#[test]
fn test_l_verrouille_et_deverrouille_la_selection() {
    let mut app = app_with(2);
    touche(&mut app, "l", ModifiersState::empty());
    assert!(image(&app, "i0").locked && image(&app, "i1").locked);
    assert_eq!(app.ui.toast_message(), Some("Images verrouillées"));

    touche(&mut app, "l", ModifiersState::empty());
    assert!(!image(&app, "i0").locked);
    assert_eq!(app.ui.toast_message(), Some("Images déverrouillées"));
}

/// `L` est une lettre nue : tenue par un modificateur, elle ne doit pas verrouiller.
#[test]
fn test_ctrl_l_ne_verrouille_rien() {
    let mut app = app_with(1);
    touche(&mut app, "l", ModifiersState::CONTROL);
    assert!(!image(&app, "i0").locked);
}

#[test]
fn test_ctrl_crochets_portent_la_selection_au_premier_et_au_dernier_plan() {
    let mut app = app_with(3);
    app.store.set_selected_image_ids(vec!["i0".into()]);

    touche(&mut app, "]", ModifiersState::CONTROL);
    assert_eq!(ordre(&app), ["i1", "i2", "i0"]);

    touche(&mut app, "[", ModifiersState::CONTROL);
    assert_eq!(ordre(&app), ["i0", "i1", "i2"]);
}

/// Sans `Ctrl`, les crochets ne sont pas un geste : ils ne doivent pas non plus tomber dans
/// le sélecteur d'outils et changer l'outil actif en silence.
#[test]
fn test_les_crochets_nus_ne_font_rien() {
    let mut app = app_with(3);
    app.store.set_selected_image_ids(vec!["i0".into()]);
    let outil = app.ui.active_tool;

    touche(&mut app, "]", ModifiersState::empty());
    assert_eq!(ordre(&app), ["i0", "i1", "i2"]);
    assert_eq!(app.ui.active_tool, outil);
}

#[test]
fn test_les_fleches_poussent_la_selection_dun_cran() {
    let mut app = app_with(1);
    fleche(&mut app, NamedKey::ArrowRight, ModifiersState::empty());
    assert_eq!((image(&app, "i0").x, image(&app, "i0").y), (1.0, 0.0));

    fleche(&mut app, NamedKey::ArrowDown, ModifiersState::empty());
    assert_eq!((image(&app, "i0").x, image(&app, "i0").y), (1.0, 1.0));

    fleche(&mut app, NamedKey::ArrowLeft, ModifiersState::empty());
    fleche(&mut app, NamedKey::ArrowUp, ModifiersState::empty());
    assert_eq!(
        (image(&app, "i0").x, image(&app, "i0").y),
        (0.0, 0.0),
        "les quatre directions se compensent"
    );
}

#[test]
fn test_maj_fleche_pousse_de_dix_crans() {
    let mut app = app_with(1);
    fleche(&mut app, NamedKey::ArrowRight, ModifiersState::SHIFT);
    assert_eq!(image(&app, "i0").x, NUDGE_DECADE);
}

/// Ce que le verrou promet, par le chemin du clavier : une image fermée ne bouge plus, même
/// si on insiste.
#[test]
fn test_une_image_verrouillee_ignore_les_fleches() {
    let mut app = app_with(1);
    touche(&mut app, "l", ModifiersState::empty());
    fleche(&mut app, NamedKey::ArrowRight, ModifiersState::empty());
    fleche(&mut app, NamedKey::ArrowRight, ModifiersState::SHIFT);
    assert_eq!(image(&app, "i0").x, 0.0);
}
