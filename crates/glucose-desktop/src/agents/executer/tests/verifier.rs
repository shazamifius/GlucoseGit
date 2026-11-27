//! **VERIFIER-1** (fiche 184) : le contrôleur du tableau, par le chemin d'une IA.

use super::appeler;
use crate::agents::json::Json;
use crate::app::GlucoseApp;

fn vide() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app
}

fn liste<'a>(v: &'a Json, champ: &str) -> &'a [Json] {
    match v.get(champ) {
        Some(Json::Liste(l)) => l,
        _ => panic!("{champ} manque dans {}", v.ecrire()),
    }
}

/// Propre, il le dit ; un texte posé sur un autre, il le nomme — et c'est un outil qui lit.
#[test]
fn verifier_1_dit_ce_qui_se_chevauche() {
    let mut app = vide();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"a","texte":"Un.","x":0,"y":0}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"b","texte":"Deux.","x":600,"y":0}"#,
    );
    let dit = appeler(&mut app, "glucose_verifier", "{}");
    assert_eq!(
        dit.get("propre"),
        Some(&Json::Bool(true)),
        "{}",
        dit.ecrire()
    );
    // Une IA qui pose sur un voisin glisse à la place libre la plus proche (PLACE-1, fiche 185).
    let pose = appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"c","texte":"Trois.","x":620,"y":10}"#,
    );
    let dit = appeler(&mut app, "glucose_verifier", "{}");
    assert_eq!(
        dit.get("propre"),
        Some(&Json::Bool(true)),
        "{}",
        pose.ecrire()
    );
    // Ce qui chevauche quand même — la main de l'humain —, le contrôleur le nomme.
    let board = app.store.project.active_board_id.clone();
    let d = glucose_core::types::Annotation::text("d", 620.0, 10.0, "Quatre.");
    app.store.add_annotation(&board, d);
    let dit = appeler(&mut app, "glucose_verifier", "{}");
    assert_eq!(dit.get("propre"), Some(&Json::Bool(false)));
    let paire = &liste(&dit, "chevauchements")[0];
    assert_eq!(paire.get("a"), Some(&Json::from("b")));
    assert_eq!(paire.get("b"), Some(&Json::from("d")));
    assert!(
        matches!(dit.get("qui"), Some(Json::Objet(_))),
        "{}",
        dit.ecrire()
    );
    assert!(crate::agents::outils::lit_seulement("glucose_verifier"));
}
