//! GROUPER-1 par les vraies entrées de l'application : `Ctrl+G`, le menu.

use crate::app::GlucoseApp;
use glucose_core::types::Annotation;
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState, SmolStr};

/// **`Ctrl+G` groupe les deux textes choisis** dans une membrane neuve, qui est choisie ensuite ;
/// un seul `Ctrl+Z` la défait.
#[test]
fn grouper_1_ctrl_g_groupe_la_selection() {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("a", 0.0, 0.0, "a"));
    app.store
        .add_annotation(&board, Annotation::text("b", 400.0, 300.0, "b"));
    app.store
        .set_selected_annotation_ids(vec!["a".into(), "b".into()]);
    let avant = app.store.project.clone();
    app.modifiers = ModifiersState::CONTROL;
    app.handle_shortcut_input(&Key::Character(SmolStr::new("g")), ElementState::Pressed);
    app.modifiers = ModifiersState::empty();
    let groupe = app
        .store
        .selected_annotation_ids
        .first()
        .cloned()
        .expect("choisie");
    let est_membre = |id: &str| {
        app.store
            .project
            .annotation(&board, id)
            .and_then(Annotation::membrane_id)
            == Some(groupe.as_str())
    };
    assert!(
        est_membre("a") && est_membre("b"),
        "les deux dans la membrane neuve"
    );
    assert!(matches!(
        app.store.project.annotation(&board, &groupe),
        Some(Annotation::Membrane { .. })
    ));
    assert!(app.store.undo(), "un geste");
    assert_eq!(app.store.project, avant, "Ctrl+Z rend tout");
}

/// **Sans rien qui ait une boîte, la commande ne s'applique pas.**
#[test]
fn grouper_1_sans_boite_la_commande_ne_s_applique_pas() {
    let mut app = GlucoseApp::new();
    app.store.clear_selection();
    assert!(!app.executer(crate::commandes::Commande::Grouper));
}
