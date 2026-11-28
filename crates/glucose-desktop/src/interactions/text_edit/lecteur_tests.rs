//! **LECTEUR-1** (fiche 193) : un lecteur n'ouvre pas de saisie — chaque frappe serait refusée sous
//! ses yeux ; il le sait par le message du canevas.

use crate::app::GlucoseApp;

#[test]
fn lecteur_1_la_saisie_ne_s_ouvre_pas() {
    let mut app = GlucoseApp::new();
    app.start_text_edit("t".into(), "abc".into());
    assert!(app.editing_session.is_some(), "seul, on écrit");
    app.editing_session = None;

    assert!(app.store.rejoindre("c9", false, 0));
    app.start_text_edit("t".into(), "abc".into());
    assert!(app.editing_session.is_none(), "un lecteur n'écrit pas");
    let refus = app.store.partage.as_ref().map(|p| p.defaits_au_lecteur);
    assert_eq!(refus, Some(1), "il le saura");
}
