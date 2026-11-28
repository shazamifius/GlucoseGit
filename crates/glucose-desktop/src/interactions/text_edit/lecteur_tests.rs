//! **LECTEUR-1** (fiche 193) : un lecteur ouvre la saisie d'un texte pour y choisir et copier,
//! jamais pour écrire ; celle d'un nœud refusé ne s'ouvre pas.

use crate::app::GlucoseApp;
use crate::interactions::text_edit::keys::Command;
use glucose_core::types::Annotation;

fn tampon(app: &GlucoseApp) -> Option<String> {
    app.editing_session.as_ref().map(|s| s.buffer.clone())
}

#[test]
fn lecteur_1_il_choisit_sans_ecrire() {
    let mut app = GlucoseApp::new();
    let tableau = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&tableau, Annotation::text("t", 0.0, 0.0, "abc"));
    assert!(app.store.rejoindre("c9", false, 0));

    app.start_text_edit("t".into(), "abc".into());
    assert_eq!(tampon(&app).as_deref(), Some("abc"), "la saisie s'ouvre");
    let refus = |app: &GlucoseApp| {
        app.store
            .partage
            .as_ref()
            .map_or(0, |p| p.defaits_au_lecteur)
    };
    assert_eq!(refus(&app), 0, "ouvrir n'est pas écrire");
    app.apply_text_command(Command::SelectAll, false);
    let choisi = app.editing_session.as_ref().map(|s| s.selection);
    assert!(choisi.is_some_and(|c| !c.is_empty()), "il choisit");
    assert_eq!(refus(&app), 0, "choisir n'est pas écrire");
    for c in [
        Command::Insert("x".into()),
        Command::Delete(
            glucose_core::text::selection::Motion::Char,
            glucose_core::text::selection::Direction::Backward,
        ),
        Command::Cut,
        Command::Paste,
    ] {
        app.apply_text_command(c, false);
    }
    assert_eq!(tampon(&app).as_deref(), Some("abc"), "rien ne s'écrit");
    assert_eq!(refus(&app), 4, "il le saura");
    app.apply_text_command(Command::Commit, false);
    assert!(app.editing_session.is_none());
    assert_eq!(refus(&app), 4, "sortir n'est pas écrire");

    app.start_text_edit("absent".into(), String::new());
    assert!(app.editing_session.is_none(), "un nœud refusé n'ouvre rien");
}
