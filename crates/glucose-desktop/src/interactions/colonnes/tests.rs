//! « Disposer en colonnes » joué sur le vrai `GlucoseApp` : la commande, le geste, le refus.

use crate::agents::json::Json;
use crate::app::GlucoseApp;
use crate::commandes::Commande;

/// Une membrane qui porte six textes empilés, posés comme par un agent.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let poser = |app: &mut GlucoseApp, outil: &str, a: &str| {
        let a = Json::lire(a).expect("JSON");
        app.executer_pour_un_agent(outil, &a).expect("posé");
    };
    poser(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","titre":"Eau","x":0,"y":0}"#,
    );
    for i in 0..6 {
        let a = format!(r#"{{"id":"p{i}","texte":"Le paragraphe {i}.","dans":"m"}}"#);
        poser(&mut app, "glucose_poser_texte", &a);
    }
    app.store.journal.clear();
    app
}

fn boite(app: &GlucoseApp, id: &str) -> glucose_core::geometry::Rect {
    let b = app.store.project.active_board_id.clone();
    app.store.project.annotation(&b, id).expect(id).bounds()
}

/// **La commande range la membrane choisie en un seul geste**, et `Ctrl+Z` la rend telle qu'elle était.
#[test]
fn colonnes_1_la_commande_range_et_se_defait() {
    let mut app = app();
    let avant = app.store.project.clone();
    let haute = boite(&app, "m");
    assert!(haute.height > haute.width, "empilée : {haute:?}");
    app.store.select_annotation("m".to_string(), false);
    assert!(app.executer(Commande::DisposerEnColonnes));
    let rangee = boite(&app, "m");
    assert!(
        rangee.width > rangee.height,
        "en colonnes, plus large que haute : {rangee:?}"
    );
    assert_eq!(app.store.undo_depth(), 1, "un seul geste");
    assert!(app.store.undo());
    assert_eq!(app.store.project, avant);
}

/// **Un nœud choisi range la membrane qui le porte** ; ce qui n'est dans aucune membrane ne range rien,
/// et n'écrit rien.
#[test]
fn colonnes_1_un_noeud_choisi_range_sa_membrane() {
    let mut app = app();
    app.store.select_annotation("p0".to_string(), false);
    assert_eq!(app.store.membranes_de_la_selection(), ["m"]);
    app.executer(Commande::DisposerEnColonnes);
    assert_eq!(app.store.undo_depth(), 1, "sa membrane est rangée");
    let b = app.store.project.active_board_id.clone();
    app.store.add_annotation(
        &b,
        glucose_core::types::Annotation::text("seul", 90000.0, 0.0, "Hors de tout."),
    );
    app.store.clear_selection();
    app.store.select_annotation("seul".to_string(), false);
    app.store.journal.clear();
    let avant = app.store.project.clone();
    app.executer(Commande::DisposerEnColonnes);
    assert_eq!(app.store.project, avant, "rien ne bouge");
    assert_eq!(app.store.undo_depth(), 0, "rien ne s'écrit");
}
