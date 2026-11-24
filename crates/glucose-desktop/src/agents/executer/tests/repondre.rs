//! **MCP-REPONDRE** (fiche 143 § 4) : une IA répond à une note en un seul geste — son texte à côté,
//! dans la membrane de la note, sans rien chevaucher, et la flèche « réponse » ; un `Ctrl+Z`
//! défait les deux.

use super::{appeler, board, texte_de};
use crate::app::GlucoseApp;
use glucose_core::types::Annotation;

/// Une membrane « m » qui porte la note « n » et, à sa droite, un voisin « v ».
fn une_note() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","titre":"Mes notes","x":0,"y":0,"largeur":2000,"hauteur":1200}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"n","texte":"Ma note.","dans":"m","x":100,"y":100}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"v","texte":"Un voisin.","dans":"m","x":380,"y":100}"#,
    );
    app
}

fn boite(app: &GlucoseApp, id: &str) -> glucose_core::geometry::Rect {
    (app.store.project.annotation(&board(app), id))
        .map(Annotation::bounds)
        .unwrap_or_else(|| panic!("{id} manque"))
}

/// **La réponse se pose à côté, dans la membrane, sans chevaucher ; la flèche dit « réponse »** ;
/// un seul `Ctrl+Z` les défait.
#[test]
fn test_mcp_repondre_a_cote_dans_la_membrane_et_un_seul_geste() {
    let mut app = une_note();
    let r = appeler(
        &mut app,
        "glucose_repondre",
        r#"{"a":"n","id":"r","texte":"Ma réponse."}"#,
    );
    assert_eq!(r.get("fleche").and_then(|f| f.as_str()), Some("r-fleche"));
    let (rep, n, v) = (boite(&app, "r"), boite(&app, "n"), boite(&app, "v"));
    assert!(!rep.overlaps(n) && !rep.overlaps(v), "{rep:?} chevauche");
    let decrits = app
        .store
        .decrire_pour_un_agent(&board(&app))
        .expect("décrit");
    let de = |id: &str| decrits.iter().find(|d| d.id == id).expect("décrit");
    assert_eq!(
        de("r").membrane.as_deref(),
        Some("m"),
        "hors de la membrane"
    );
    let f = de("r-fleche");
    assert_eq!(
        (f.source.as_deref(), f.cible.as_deref()),
        (Some("r"), Some("n"))
    );
    assert_eq!(f.texte.as_deref(), Some("réponse"));
    assert!(app.store.undo(), "un geste");
    assert!(texte_de(&app, "r").is_none() && texte_de(&app, "r-fleche").is_none());
}

/// **Le même id récrit la réponse là où elle est**, sans doubler la flèche.
#[test]
fn test_mcp_repondre_le_meme_id_recrit_sur_place() {
    let mut app = une_note();
    let tenir = r#"{"a":"n","id":"r","texte":"Première."}"#;
    appeler(&mut app, "glucose_repondre", tenir);
    let avant = boite(&app, "r");
    let recrire = r#"{"a":"n","id":"r","texte":"Seconde."}"#;
    appeler(&mut app, "glucose_repondre", recrire);
    assert_eq!(
        (boite(&app, "r").left, boite(&app, "r").top),
        (avant.left, avant.top)
    );
    assert_eq!(texte_de(&app, "r").as_deref(), Some("Seconde."));
    let fleches = (app
        .store
        .active_board()
        .expect("tableau")
        .annotations
        .iter())
    .filter(|a| matches!(a, Annotation::Arrow { .. }))
    .count();
    assert_eq!(fleches, 1);
}

/// **Une membrane serrée grandit pour la réponse** : la place libre sort de sa boîte, mais la
/// réponse y entre quand même — la membrane grandit, la note et sa réponse restent ensemble.
#[test]
fn test_mcp_repondre_une_membrane_serree_grandit_pour_elle() {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","titre":"Serrée","x":0,"y":0,"largeur":400,"hauteur":300}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"n","texte":"Ma note.","dans":"m","x":100,"y":100}"#,
    );
    appeler(
        &mut app,
        "glucose_repondre",
        r#"{"a":"n","id":"r","texte":"Ma réponse."}"#,
    );
    let decrits = (app.store)
        .decrire_pour_un_agent(&board(&app))
        .expect("décrit");
    let de = |id: &str| decrits.iter().find(|d| d.id == id).expect("décrit");
    assert_eq!(
        de("r").membrane.as_deref(),
        Some("m"),
        "hors de la membrane"
    );
    assert!(
        boite(&app, "m").contains_rect(boite(&app, "r")),
        "la membrane n'a pas grandi"
    );
}

/// **La membrane de la note, pas celle qui la recouvre** (MEMB-8) : une membrane plus petite
/// recouvre la note sans la contenir ; la réponse, posée dans son aire, appartient quand même à la
/// membrane de la note — ce qu'on a nommé l'emporte sur la géométrie.
#[test]
fn test_mcp_repondre_suit_la_membrane_nommee_pas_la_geometrie() {
    let mut app = une_note();
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"x","titre":"Recouvre","x":50,"y":50,"largeur":1900,"hauteur":800}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"n","texte":"Ma note.","dans":"m","x":100,"y":100}"#,
    );
    appeler(
        &mut app,
        "glucose_repondre",
        r#"{"a":"n","id":"r","texte":"Ma réponse."}"#,
    );
    let decrits = (app.store)
        .decrire_pour_un_agent(&board(&app))
        .expect("décrit");
    let de = |id: &str| decrits.iter().find(|d| d.id == id).expect("décrit");
    assert_eq!(de("n").membrane.as_deref(), Some("m"), "la note");
    assert_eq!(de("r").membrane.as_deref(), Some("m"), "la réponse");
}
