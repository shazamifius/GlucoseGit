//! GROUPER-1 pour un agent : `glucose_grouper`, le `Ctrl+G` de l'humain (fiche 98).

use super::{appeler, args, board};
use crate::app::GlucoseApp;
use glucose_core::types::Annotation;

/// **Un agent entoure deux textes d'une région nommée et teintée** ; la sélection de l'humain
/// n'a pas bougé ; un `Ctrl+Z` défait tout.
#[test]
fn grouper_1_un_agent_entoure_des_noeuds_d_une_region() {
    let mut app = GlucoseApp::new();
    for (id, x) in [("a", 0.0), ("b", 500.0)] {
        let t = format!(r#"{{"id":"{id}","texte":"{id}","x":{x},"y":0}}"#);
        appeler(&mut app, "glucose_poser_texte", &t);
    }
    app.store.set_selected_annotation_ids(vec!["a".into()]);
    let avant = app.store.project.clone();
    let r = appeler(
        &mut app,
        "glucose_grouper",
        r##"{"ids":["a","b"],"titre":"Eau","couleur":"#60a5fa"}"##,
    );
    let g = r
        .get("id")
        .and_then(|v| v.as_str())
        .expect("l'identifiant")
        .to_string();
    let b = board(&app);
    let Some(Annotation::Membrane { text, color, .. }) = app.store.project.annotation(&b, &g)
    else {
        panic!("une membrane")
    };
    assert_eq!(
        (text.as_deref(), color.as_deref()),
        (Some("Eau"), Some("#60a5fa"))
    );
    for id in ["a", "b"] {
        let m = app
            .store
            .project
            .annotation(&b, id)
            .and_then(Annotation::membrane_id);
        assert_eq!(m, Some(g.as_str()), "{id} dans la région");
    }
    assert_eq!(
        app.store.selected_annotation_ids.len(),
        1,
        "la sélection de l'humain reste"
    );
    assert!(app.store.undo(), "un geste");
    assert_eq!(app.store.project, avant);
}

/// **Un identifiant inconnu est refusé en le nommant**, et rien ne s'écrit.
#[test]
fn grouper_1_un_identifiant_inconnu_est_refuse() {
    let mut app = GlucoseApp::new();
    appeler(&mut app, "glucose_poser_texte", r#"{"id":"a","texte":"a"}"#);
    let avant = app.store.project.clone();
    let refus = app.executer_pour_un_agent("glucose_grouper", &args(r#"{"ids":["a","zut"]}"#));
    assert!(refus.is_err_and(|e| e.contains("zut")));
    assert_eq!(app.store.project, avant);
}
