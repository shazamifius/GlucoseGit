//! AGENT-7 — l'agent cherche comme l'humain : la même loi, le même ordre, la même page.

use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::types::Annotation;

fn chercher(app: &mut GlucoseApp, arguments: &str) -> Json {
    let a = Json::lire(arguments).expect("JSON");
    app.executer_pour_un_agent("glucose_chercher", &a)
        .unwrap_or_else(|e| panic!("glucose_chercher a échoué : {e}"))
}

fn ids(r: &Json) -> Vec<String> {
    r.get("resultats")
        .and_then(Json::as_liste)
        .expect("des résultats")
        .iter()
        .filter_map(|t| t.get("id").and_then(Json::as_str).map(str::to_string))
        .collect()
}

/// **L'agent trouve ce que trouve `Ctrl+F`**, dans le même ordre (les débuts de mots d'abord), par
/// la même page de dix rangées, et `depuis` donne la suite — sans rien écrire au journal.
#[test]
fn agent_7_l_agent_trouve_ce_que_trouve_ctrl_f() {
    let mut app = GlucoseApp::new();
    let un = app.store.project.active_board_id.clone();
    let deux = app.store.add_board("Chimie");
    for i in 0..12 {
        // Les pairs ne disent que « bureau » : ils passent après ceux qui disent « l'eau ».
        let texte = if i % 2 == 0 {
            "Le bureau."
        } else {
            "Le goût de l'eau."
        };
        let tableau = if i < 6 { &un } else { &deux };
        let n = Annotation::text(format!("n{i}"), 0.0, f64::from(i) * 100.0, texte);
        app.store.add_annotation(tableau, n);
    }
    app.store
        .add_annotation(&deux, Annotation::text("h2o", 0.0, 0.0, "Eau = H2O."));
    app.store.journal.clear();
    let palette: Vec<String> = app
        .chercher_dans_le_document("EAU")
        .into_iter()
        .filter_map(|t| t.id)
        .collect();
    assert_eq!(palette.len(), 13);
    assert_eq!(palette[0], "n1", "un début de mot d'abord");

    let page = chercher(&mut app, r#"{"cherche":"EAU"}"#);
    assert_eq!(page.get("total").and_then(Json::as_f64), Some(13.0));
    assert_eq!(ids(&page), palette[..10], "la page de la palette");
    let suite = chercher(&mut app, r#"{"cherche":"EAU","depuis":10}"#);
    assert_eq!(ids(&suite), palette[10..], "puis la suite");

    let h2o = page
        .get("resultats")
        .and_then(Json::as_liste)
        .and_then(|l| {
            l.iter()
                .find(|t| t.get("id").and_then(Json::as_str) == Some("h2o"))
        })
        .expect("h2o, parmi les dix");
    assert_eq!(h2o.get("onglet").and_then(Json::as_str), Some("Chimie"));
    assert_eq!(h2o.get("sorte").and_then(Json::as_str), Some("texte"));
    assert_eq!(h2o.get("phrase").and_then(Json::as_str), Some("Eau = H2O."));
    assert_eq!(app.store.undo_depth(), 0, "chercher n'écrit rien");
}
