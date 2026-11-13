//! **DATE-1 — un agent date des nœuds, et lit leur date** (fiches 113 et 114) : le geste de
//! `Maj+T` par `glucose_dater`, la pastille que l'humain voit rendue par `glucose_lire` — son mot
//! et ses années exactes.

use super::{appeler, args, board};
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::types::{Annotation, BoardImage, TemporalAnchor};

/// Un texte et une photo, sans date.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let b = board(&app);
    app.store
        .add_annotation(&b, Annotation::text("t", 0.0, 0.0, "La Révolution"));
    app.store
        .add_image(&b, BoardImage::new("p", 400.0, 0.0, 100.0, 100.0));
    app
}

/// La date de ce nœud, texte ou photo.
fn date(app: &GlucoseApp, id: &str) -> Option<TemporalAnchor> {
    let b = board(app);
    match app.store.project.annotation(&b, id) {
        Some(a) => a.temporal_anchor().cloned(),
        None => app.store.image(&b, id)?.temporal_anchor.clone(),
    }
}

/// Ce que `glucose_lire` dit de la date de ce nœud.
fn date_lue(app: &mut GlucoseApp, id: &str) -> Option<Json> {
    let lu = appeler(app, "glucose_lire", "{}");
    let noeud = lu
        .get("noeuds")?
        .as_liste()?
        .iter()
        .find(|n| n.get("id").and_then(Json::as_str) == Some(id))?;
    noeud.get("date").cloned()
}

fn annees(debut: i64, fin: i64) -> TemporalAnchor {
    TemporalAnchor {
        start: debut,
        end: fin,
        label: None,
    }
}

/// **Un texte et une photo datés d'un seul geste**, qu'un `Ctrl+Z` défait d'un coup ; puis
/// `glucose_lire` rend le mot que la pastille montre et les années exactes.
#[test]
fn date_1_un_agent_date_un_texte_et_une_photo_en_un_geste() {
    let mut app = application();
    let r = appeler(
        &mut app,
        "glucose_dater",
        r#"{"ids":["t","p"],"date":"1789-1799"}"#,
    );
    assert_eq!(r.get("dates").and_then(Json::as_f64), Some(2.0));
    assert_eq!(date(&app, "t"), Some(annees(1789, 1799)));
    assert_eq!(date(&app, "p"), Some(annees(1789, 1799)));
    let lue = date_lue(&mut app, "p").expect("la date de la photo, lue");
    assert_eq!(lue.get("texte").and_then(Json::as_str), Some("1789 – 1799"));
    assert_eq!(lue.get("debut").and_then(Json::as_f64), Some(1789.0));
    assert_eq!(lue.get("fin").and_then(Json::as_f64), Some(1799.0));
    assert!(app.store.undo(), "le geste se défait");
    assert_eq!(
        (date(&app, "t"), date(&app, "p")),
        (None, None),
        "d'un coup"
    );
    assert!(
        date_lue(&mut app, "t").is_none(),
        "sans date, glucose_lire n'en dit rien"
    );
}

/// **Le mot arrondit, les années non** : « 12.3 ka » pour -12 345 — l'agent lit les deux, et ce
/// qu'il renvoie en années entières retombe sur la même date.
#[test]
fn date_1_les_annees_lues_sont_exactes() {
    let mut app = application();
    appeler(
        &mut app,
        "glucose_dater",
        r#"{"ids":["t"],"date":"-12345"}"#,
    );
    let lue = date_lue(&mut app, "t").expect("la date lue");
    assert_eq!(lue.get("texte").and_then(Json::as_str), Some("12.3 ka"));
    assert_eq!(lue.get("debut").and_then(Json::as_f64), Some(-12_345.0));
    // Une ère garde son nom : le mot de la pastille, et sa période.
    appeler(
        &mut app,
        "glucose_dater",
        r#"{"ids":["t"],"date":"Renaissance"}"#,
    );
    let ere = date_lue(&mut app, "t").expect("l'ère lue");
    assert_eq!(ere.get("texte").and_then(Json::as_str), Some("Renaissance"));
    assert!(ere.get("fin").and_then(Json::as_f64) > ere.get("debut").and_then(Json::as_f64));
}

/// **Un nom qui n'est pas un nœud est refusé par son nom**, et rien ne change — pas même le nœud
/// qui existait ; **une date illisible aussi**, avec ce qu'on avait écrit.
#[test]
fn date_1_un_nom_inconnu_ou_une_date_illisible_est_refuse() {
    let mut app = application();
    let inconnu =
        app.executer_pour_un_agent("glucose_dater", &args(r#"{"ids":["t","x"],"date":"1789"}"#));
    let e = inconnu.expect_err("un nœud qui n'existe pas");
    assert!(e.contains("« x »"), "le nom dans le refus : {e}");
    assert_eq!(date(&app, "t"), None, "rien n'a changé");
    let illisible = app.executer_pour_un_agent(
        "glucose_dater",
        &args(r#"{"ids":["t"],"date":"l'an pomme"}"#),
    );
    let e = illisible.expect_err("une date que Glucose ne lit pas");
    assert!(e.contains("l'an pomme"), "ce qu'on avait écrit : {e}");
    assert_eq!(date(&app, "t"), None, "rien n'a changé");
}

/// **Sans date, la retire** — absente comme vide.
#[test]
fn date_1_sans_date_l_agent_la_retire() {
    let mut app = application();
    appeler(
        &mut app,
        "glucose_dater",
        r#"{"ids":["t","p"],"date":"1969"}"#,
    );
    appeler(&mut app, "glucose_dater", r#"{"ids":["t"]}"#);
    appeler(&mut app, "glucose_dater", r#"{"ids":["p"],"date":"  "}"#);
    assert_eq!((date(&app, "t"), date(&app, "p")), (None, None));
}
