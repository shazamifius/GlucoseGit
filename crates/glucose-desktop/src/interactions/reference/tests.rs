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
