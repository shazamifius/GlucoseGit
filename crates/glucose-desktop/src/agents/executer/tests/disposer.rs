//! `glucose_disposer` : la membrane rangée par un agent, en un geste ; un nom qui n'est pas une
//! membrane est refusé.

use super::{appeler, args};
use crate::agents::json::Json;
use crate::app::GlucoseApp;

#[test]
fn un_agent_range_une_membrane_en_colonnes() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","titre":"Eau","x":0,"y":0}"#,
    );
    for i in 0..8 {
        let a = format!(r#"{{"id":"p{i}","texte":"Le paragraphe {i}.","dans":"m"}}"#);
        appeler(&mut app, "glucose_poser_texte", &a);
    }
    let profondeur = app.store.undo_depth();
    let r = appeler(
        &mut app,
        "glucose_disposer",
        r#"{"dans":"m","proportion":2}"#,
    );
    assert_eq!(r.get("ranges").and_then(Json::as_f64), Some(8.0));
    let b = r.get("boite").expect("sa boîte");
    let (l, h) = (
        b.get("largeur").and_then(Json::as_f64).unwrap_or_default(),
        b.get("hauteur").and_then(Json::as_f64).unwrap_or_default(),
    );
    assert!(l > h, "plus large que haute : {l} × {h}");
    assert_eq!(app.store.undo_depth(), profondeur + 1, "un seul geste");
    let refus = app.executer_pour_un_agent("glucose_disposer", &args(r#"{"dans":"p0"}"#));
    assert!(refus.is_err_and(|e| e.contains("n'est pas une membrane")));
}
