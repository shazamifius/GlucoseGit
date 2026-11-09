//! OPERATEUR-2 : une IA lit et pose les opérateurs des post-it, comme l'humain (fiche 98).

use super::{appeler, args, board};
use crate::app::GlucoseApp;
use glucose_core::types::{Annotation, StickyOperator, DEFAULT_OPERATOR_HEIGHT};

fn operateur(app: &GlucoseApp, id: &str) -> Option<StickyOperator> {
    match app.store.project.annotation(&board(app), id)? {
        Annotation::Sticky { operator, .. } => *operator,
        _ => None,
    }
}

/// **Un agent pose un « parce-que »** sans dire « post-it » — un opérateur en est un —, à la
/// taille de son mot ; il le relit tel que l'humain le voit ; « aucun » le rend papier, et une
/// mise à jour qui n'en parle pas le garde (AGENT-8).
#[test]
fn operateur_2_un_agent_pose_relit_et_retire_un_operateur() {
    let mut app = GlucoseApp::new();
    let r = appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"p","texte":"car il pleut","operateur":"parce-que"}"#,
    );
    assert_eq!(operateur(&app, "p"), Some(StickyOperator::Because));
    let hauteur = super::nombre(&r, &["boite", "hauteur"]);
    assert_eq!(
        hauteur, DEFAULT_OPERATOR_HEIGHT,
        "la boîte rendue est la pilule"
    );
    let lu = appeler(&mut app, "glucose_lire", r#"{"ids":["p"]}"#).ecrire();
    assert!(
        lu.contains(r#""operateur":"parce-que""#),
        "l'IA lit ce que l'humain voit : {lu}"
    );

    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"p","texte":"car il pleut fort","post_it":true}"#,
    );
    assert_eq!(
        operateur(&app, "p"),
        Some(StickyOperator::Because),
        "une mise à jour qui n'en parle pas le garde"
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"p","texte":"car il pleut fort","post_it":true,"operateur":"aucun"}"#,
    );
    assert_eq!(operateur(&app, "p"), None, "« aucun » : le papier revient");
}

/// **Un mot qui n'est pas un opérateur est refusé**, en disant les mots permis, et rien ne
/// s'écrit.
#[test]
fn operateur_2_un_mot_inconnu_est_refuse() {
    let mut app = GlucoseApp::new();
    let refus = app.executer_pour_un_agent(
        "glucose_poser_texte",
        &args(r#"{"id":"p","texte":"x","operateur":"donc"}"#),
    );
    assert!(refus.is_err_and(|e| e.contains("donc") && e.contains("parce-que")));
    assert!(app.store.project.annotation(&board(&app), "p").is_none());
}

/// **La pilule que l'humain pose au clavier se lit chez l'IA** : le pont dans l'autre sens.
#[test]
fn operateur_2_l_ia_lit_l_operateur_pose_par_l_humain() {
    let mut app = GlucoseApp::new();
    let b = board(&app);
    app.store
        .add_annotation(&b, Annotation::sticky("h", 0.0, 0.0, "et pourtant"));
    app.store
        .poser_l_operateur(&b, &["h".to_string()], Some(StickyOperator::And));
    let lu = appeler(&mut app, "glucose_lire", r#"{"ids":["h"]}"#).ecrire();
    assert!(lu.contains(r#""operateur":"et""#), "{lu}");
}
