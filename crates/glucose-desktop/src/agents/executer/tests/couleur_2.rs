//! COULEUR-2 : une IA lit et pose les couleurs avec le sens que l'humain leur donne (fiche 98).

use super::{appeler, board};
use crate::app::GlucoseApp;
use glucose_core::types::Annotation;

/// **La couleur qu'un agent donne à un post-it est son papier** — ce que la barre de couleur de
/// l'humain change —, pas son encre ; et `glucose_lire` la dit.
#[test]
fn couleur_2_un_agent_colore_le_papier_d_un_post_it_et_le_relit() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r##"{"id":"p","texte":"il pleut","post_it":true,"couleur":"#f472b6"}"##,
    );
    let Some(Annotation::Sticky {
        bg_color, color, ..
    }) = app.store.project.annotation(&board(&app), "p")
    else {
        panic!("un post-it")
    };
    assert_eq!(bg_color.as_deref(), Some("#f472b6"), "son papier");
    assert_eq!(*color, None, "son encre ne change pas");
    let lu = appeler(&mut app, "glucose_lire", r#"{"ids":["p"]}"#).ecrire();
    assert!(lu.contains(r##""couleur":"#f472b6""##), "{lu}");
}

/// **La teinte que l'humain donne à une région se lit chez l'IA** : D — les régions reconnues par
/// leur couleur — ne vaut que si l'IA les reconnaît aussi.
#[test]
fn couleur_2_l_ia_lit_la_couleur_posee_par_l_humain() {
    let mut app = GlucoseApp::new();
    let b = board(&app);
    app.store
        .add_annotation(&b, Annotation::membrane("m", 0.0, 0.0, 300.0, 200.0));
    app.store.colorer(&b, &["m".to_string()], Some("#34d399"));
    let lu = appeler(&mut app, "glucose_lire", r#"{"ids":["m"]}"#).ecrire();
    assert!(lu.contains(r##""couleur":"#34d399""##), "{lu}");
}
