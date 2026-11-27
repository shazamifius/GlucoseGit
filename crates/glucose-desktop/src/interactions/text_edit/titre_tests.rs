//! **Le titre d'une membrane s'annule** (sa note du 27/11, `text-n3rk-2584` : *« il est enregistré
//! dans la Time Machine mais ne possède pas de Ctrl+Z »*).

use crate::app::GlucoseApp;
use crate::interactions::text_edit::keys::Command;
use glucose_core::types::Annotation;

fn une_membrane(titre: &str) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let m = Annotation::membrane("m", 0.0, 0.0, 600.0, 400.0);
    app.store.add_annotation(&board, m);
    app.store.ecrire_le_texte(&board, "m", titre);
    app
}

fn titre(app: &GlucoseApp) -> String {
    app.editable_text_of("m").expect("une membrane")
}

/// Écrire le titre, valider, `Ctrl+Z` : le titre d'avant revient, et `Ctrl+Y` le rétablit.
#[test]
fn test_le_titre_d_une_membrane_s_annule() {
    let mut app = une_membrane("avant");
    let entrees = app.store.undo_depth();
    app.start_text_edit("m".into(), titre(&app));
    app.apply_text_command(Command::Insert(" et après".into()), false);
    app.commit_editing();
    assert_eq!(titre(&app), "avant et après");
    assert_eq!(app.store.undo_depth(), entrees + 1, "une saisie = un geste");
    assert!(app.store.undo(), "il y a un geste à défaire");
    assert_eq!(titre(&app), "avant");
    assert!(app.store.redo());
    assert_eq!(titre(&app), "avant et après");
}

/// Un titre vide, écrit pour la première fois : défait, la membrane redevient sans titre.
#[test]
fn test_un_premier_titre_s_annule() {
    let mut app = une_membrane("");
    app.start_text_edit("m".into(), String::new());
    app.apply_text_command(Command::Insert("neuf".into()), false);
    app.commit_editing();
    assert_eq!(titre(&app), "neuf");
    assert!(app.store.undo());
    assert_eq!(titre(&app), "");
}
