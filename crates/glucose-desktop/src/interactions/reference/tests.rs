//! Le mode référence (fiche 51 § 5).

use super::*;
use crate::persist::disque::tests::{application, dossier};
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState};

/// **Les bords et les coins se reconnaissent**, à l'épaisseur près ; l'intérieur n'est aucun
/// bord.
#[test]
fn test_les_bords_et_les_coins() {
    let t = (800.0, 600.0);
    assert_eq!(bord((2.0, 300.0), t, 8.0), Some(ResizeDirection::West));
    assert_eq!(bord((795.0, 300.0), t, 8.0), Some(ResizeDirection::East));
    assert_eq!(bord((400.0, 1.0), t, 8.0), Some(ResizeDirection::North));
    assert_eq!(bord((400.0, 599.0), t, 8.0), Some(ResizeDirection::South));
    assert_eq!(bord((1.0, 1.0), t, 8.0), Some(ResizeDirection::NorthWest));
    assert_eq!(
        bord((799.0, 599.0), t, 8.0),
        Some(ResizeDirection::SouthEast)
    );
    assert_eq!(bord((400.0, 300.0), t, 8.0), None);
    assert_eq!(bord((8.0, 8.0), t, 8.0), None, "l'épaisseur est exclue");
}

/// **Le mode se retient d'une session à l'autre, et le même geste le défait** : une autre
/// application qui habite le même dossier le retrouve.
#[test]
fn test_le_mode_se_retient_et_se_defait() {
    let d = dossier("mode-reference");
    let mut app = application(&d);
    assert!(!app.mode_reference_retenu());
    app.basculer_le_mode_reference();
    assert!(app.ui.reference);
    assert_eq!(app.ui.header_height(), 0.0, "plus de bande");
    assert!(
        application(&d).mode_reference_retenu(),
        "retenu pour la relance"
    );
    app.basculer_le_mode_reference();
    assert!(!app.ui.reference);
    assert!(app.ui.header_height() > 0.0, "la bande revient");
    assert!(!application(&d).mode_reference_retenu(), "et oublié");
}

/// **`Ctrl+Maj+A` bascule le mode, et ne sélectionne rien** ; `Ctrl+A` sélectionne toujours.
#[test]
fn test_ctrl_maj_a_bascule_le_mode_sans_tout_selectionner() {
    let d = dossier("mode-reference-touche");
    let mut app = application(&d);
    app.store.clear_selection();
    app.modifiers = ModifiersState::CONTROL | ModifiersState::SHIFT;
    app.handle_shortcut_input(&Key::Character("A".into()), ElementState::Pressed);
    assert!(app.ui.reference);
    assert!(
        app.store.selected_annotation_ids.is_empty(),
        "rien de sélectionné"
    );
    app.modifiers = ModifiersState::CONTROL;
    app.handle_shortcut_input(&Key::Character("a".into()), ElementState::Pressed);
    assert!(
        !app.store.selected_annotation_ids.is_empty(),
        "Ctrl+A sélectionne"
    );
}

/// **Plus aucune interface ne se dessine**, et ses clics partent avec elle. À l'envers : hors du
/// mode, la même image porte la bande.
#[test]
fn test_plus_aucune_interface() {
    let dessin = |reference: bool| {
        let mut ui = crate::ui::UiState::new();
        ui.reference = reference;
        let store = glucose_core::store::Store::new("ref");
        let mut pixmap = tiny_skia::Pixmap::new(800, 600).expect("pixmap");
        crate::ui::render_ui(
            &mut pixmap.as_mut(),
            &store,
            &mut ui,
            &crate::typography::Typography::new(),
            &crate::theme::Theme::default(),
            crate::params::Pointer { x: 0.0, y: 0.0 },
        );
        let clic = crate::ui::handle_ui_click(
            100.0,
            10.0,
            800.0,
            600.0,
            &store,
            &mut ui,
            &crate::typography::Typography::new(),
        );
        (pixmap.data().iter().any(|&o| o != 0), clic.is_some())
    };
    assert_eq!(dessin(true), (false, false), "rien dessiné, rien pris");
    assert_eq!(
        dessin(false),
        (true, true),
        "à l'envers, la bande se dessine et se clique"
    );
}

/// **Les signets répondent en mode référence** : `Ctrl+1` pose, `1` y ramène — son essai du
/// 07/10 dit que non.
#[test]
fn test_les_signets_repondent_en_mode_reference() {
    let d = dossier("reference-signets");
    let mut app = application(&d);
    app.poser_le_mode_reference(true);
    let ici = app.store.viewport();
    app.modifiers = ModifiersState::CONTROL;
    app.handle_shortcut_input(&Key::Character("1".into()), ElementState::Pressed);
    app.modifiers = ModifiersState::empty();
    let board = app.store.project.active_board_id.clone();
    assert_eq!(
        app.store.bookmark(&board, "1"),
        Some(ici),
        "Ctrl+1 pose le signet"
    );
    app.store.set_viewport(
        &board,
        glucose_core::types::Viewport {
            x: ici.x + 5000.0,
            ..ici
        },
    );
    app.handle_shortcut_input(&Key::Character("1".into()), ElementState::Pressed);
    assert!(app.vol.en_cours(), "1 ramene au signet par un vol");
}

/// **REFERENCE-2 — `Alt` + pincer agrandit la fenêtre autour de son centre**, et le canevas ne
/// bouge pas. Une octave double, une octave en arrière divise par deux ; jamais sous le côté
/// minimal, jamais plus grand que l'écran, les proportions tenues.
#[test]
fn test_reference_2_le_pincement_et_le_cadre_de_la_fenetre() {
    let ((x, y), (l, h)) = cadre_apres_pincement((100.0, 100.0), (400.0, 300.0), 1.0, 160.0, None);
    assert_eq!(
        ((x, y), (l, h)),
        ((-100, -50), (800, 600)),
        "double, autour du centre"
    );
    let (_, (l, h)) = cadre_apres_pincement((0.0, 0.0), (400.0, 300.0), -4.0, 160.0, None);
    assert_eq!(
        (l, h),
        (213, 160),
        "jamais sous le cote minimal, proportions tenues"
    );
    let (_, (l, h)) = cadre_apres_pincement(
        (0.0, 0.0),
        (400.0, 300.0),
        4.0,
        160.0,
        Some((1600.0, 1000.0)),
    );
    assert_eq!((l, h), (1333, 1000), "jamais plus grand que l'ecran");
}

/// **En mode référence, `Alt` donne le geste à la fenêtre** : le pincement ne zoome plus la
/// vue, et `Alt` + glisser ne commence pas une sélection. Hors du mode, rien ne change.
#[test]
fn test_reference_2_alt_donne_le_geste_a_la_fenetre() {
    let d = dossier("reference-alt");
    let mut app = application(&d);
    app.poser_le_mode_reference(true);
    app.modifiers = ModifiersState::ALT;
    assert!(
        app.redimensionner_au_pincement(0.5),
        "le pincement va a la fenetre"
    );
    assert!(
        app.deplacer_la_fenetre_avec_alt(),
        "le glisser va a la fenetre"
    );
    app.modifiers = ModifiersState::empty();
    assert!(
        !app.redimensionner_au_pincement(0.5),
        "sans Alt, la vue zoome"
    );
    app.poser_le_mode_reference(false);
    app.modifiers = ModifiersState::ALT;
    assert!(
        !app.redimensionner_au_pincement(0.5),
        "hors du mode, Alt ne prend rien"
    );
    assert!(!app.deplacer_la_fenetre_avec_alt());
}

/// **Par les vraies entrées** : en mode référence, `Alt` + clic gauche sur le vide ne commence
/// pas de sélection, et `Alt` + un défilement de zoom ne zoome pas la vue. À l'envers, sans
/// `Alt`, les deux reviennent au canevas.
#[test]
fn test_reference_2_alt_par_le_clic_et_le_defilement() {
    use winit::event::{MouseButton, MouseScrollDelta};
    let d = dossier("reference-alt-entrees");
    let mut app = application(&d);
    app.store.clear_selection();
    app.poser_le_mode_reference(true);
    let jouer = |app: &mut crate::app::GlucoseApp, alt: bool| {
        app.selection_box = None;
        app.mouse_pos = (640.0, 360.0);
        app.modifiers = if alt {
            ModifiersState::ALT
        } else {
            ModifiersState::empty()
        };
        app.handle_mouse_down(MouseButton::Left, 1280.0, 720.0);
        let selection = app.selection_box.is_some();
        app.handle_mouse_up(MouseButton::Left);
        app.modifiers = if alt {
            ModifiersState::ALT | ModifiersState::CONTROL
        } else {
            ModifiersState::CONTROL
        };
        app.handle_mouse_wheel(MouseScrollDelta::LineDelta(0.0, 1.0));
        let zoom = app.elan.en_cours();
        app.appliquer_l_elan(1280, 720);
        app.modifiers = ModifiersState::empty();
        (selection, zoom)
    };
    assert_eq!(
        jouer(&mut app, true),
        (false, false),
        "Alt : tout va a la fenetre"
    );
    assert_eq!(
        jouer(&mut app, false),
        (true, true),
        "sans Alt : le canevas"
    );
}
