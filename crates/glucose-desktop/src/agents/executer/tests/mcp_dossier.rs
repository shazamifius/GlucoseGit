//! **MCP-DOSSIER** (fiche 133) : ce qu'un humain voit d'un dossier, une IA le lit — son nom, ce qu'il
//! ouvre, ce qu'il contient, où l'on est quand on y entre — et un dossier ou un onglet se renomme.

use super::{appeler, args, board};
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::types::{Annotation, CanvasFolder};

/// Un document au dossier « Dossier » (son nom de naissance), qui contient un texte ; rend le
/// dossier et le tableau qu'il ouvre.
fn un_dossier_plein() -> (GlucoseApp, String, String) {
    let mut app = GlucoseApp::new();
    let racine = board(&app);
    let mut f = CanvasFolder::new("archive-1", "Dossier", "");
    (f.x, f.y) = (4000.0, 0.0);
    app.store.create_folder(&racine, f);
    let ouvre = app.store.active_board().expect("un tableau").folders[0]
        .child_board_id
        .clone();
    let t = Annotation::text("dedans", 0.0, 0.0, "Une vieille passation.");
    app.store.add_annotation(&ouvre, t);
    (app, "archive-1".to_string(), ouvre)
}

fn noeud<'a>(r: &'a Json, id: &str) -> Option<&'a Json> {
    r.get("noeuds")?
        .as_liste()?
        .iter()
        .find(|n| n.get("id").and_then(Json::as_str) == Some(id))
}

fn nom_du_tableau(app: &GlucoseApp, id: &str) -> Option<String> {
    let b = app.store.project.boards.iter().find(|b| b.id == id)?;
    Some(b.name.clone())
}

fn texte<'a>(v: &'a Json, cle: &str) -> Option<&'a str> {
    v.get(cle).and_then(Json::as_str)
}

/// **MCP-DOSSIER-1 — un dossier se lit comme un nœud** : sa sorte, son nom, le tableau qu'il ouvre
/// et le chiffre de son badge ; au plan aussi, avec les membranes. Avant la fiche 133, il n'était
/// dans aucune lecture.
#[test]
fn test_mcp_dossier_1_un_dossier_se_lit_et_dit_ce_qu_il_ouvre() {
    let (mut app, dossier, ouvre) = un_dossier_plein();
    for niveau in ["sommaire", "plan"] {
        let r = appeler(
            &mut app,
            "glucose_lire",
            &format!(r#"{{"niveau":"{niveau}"}}"#),
        );
        let d = noeud(&r, &dossier).unwrap_or_else(|| panic!("le dossier, au {niveau}"));
        assert_eq!(texte(d, "sorte"), Some("dossier"));
        assert_eq!(texte(d, "debut"), Some("Dossier"), "son nom");
        assert_eq!(texte(d, "ouvre"), Some(ouvre.as_str()));
        assert_eq!(
            d.get("contient").and_then(Json::as_f64),
            Some(1.0),
            "son badge"
        );
    }
    let r = appeler(
        &mut app,
        "glucose_lire",
        &format!(r#"{{"ids":["{dossier}"]}}"#),
    );
    assert_eq!(
        noeud(&r, &dossier).and_then(|d| texte(d, "texte")),
        Some("Dossier")
    );
}

/// **MCP-DOSSIER-3 — on sait où l'on lit** : le tableau d'un dossier dit son chemin, l'onglet puis
/// le dossier ; la racine, son seul onglet. Une IA s'y croyait dans « Canvas Principal ».
#[test]
fn test_mcp_dossier_3_la_lecture_dit_ou_est_le_tableau() {
    let (mut app, _, ouvre) = un_dossier_plein();
    let onglet = app.store.active_board().expect("un tableau").name.clone();
    let r = appeler(&mut app, "glucose_lire", "{}");
    assert_eq!(texte(&r, "ou"), Some(onglet.as_str()));
    let r = appeler(
        &mut app,
        "glucose_lire",
        &format!(r#"{{"tableau":"{ouvre}"}}"#),
    );
    assert_eq!(
        texte(&r, "ou"),
        Some(format!("{onglet} › Dossier").as_str())
    );
    assert!(noeud(&r, "dedans").is_some(), "ce que le dossier contient");
}

/// **MCP-DOSSIER-4 — ce qu'on demande et qui n'est pas là est dit**, jamais tu : la liste qui se
/// taisait laissait croire qu'un nœud n'existait plus.
#[test]
fn test_mcp_dossier_4_un_identifiant_absent_est_dit_introuvable() {
    let (mut app, dossier, _) = un_dossier_plein();
    let r = appeler(
        &mut app,
        "glucose_lire",
        &format!(r#"{{"ids":["{dossier}","dedans","perdu"]}}"#),
    );
    let absents: Vec<&str> = r
        .get("introuvables")
        .and_then(Json::as_liste)
        .expect("les introuvables")
        .iter()
        .filter_map(Json::as_str)
        .collect();
    assert_eq!(
        absents,
        ["dedans", "perdu"],
        "« dedans » est dans le dossier, pas ici"
    );
    let r = appeler(&mut app, "glucose_lire", "{}");
    assert!(
        r.get("introuvables").is_none(),
        "rien demandé, rien d'absent"
    );
}

/// **MCP-DOSSIER-2 — un agent renomme un dossier** : le dossier et le tableau qu'il ouvre portent
/// le nouveau nom (RENOMMER-2), en un seul geste que `Ctrl+Z` défait ; par l'identifiant de son
/// tableau aussi. Un onglet se renomme ; un nom vide et un inconnu sont refusés.
#[test]
fn test_mcp_dossier_2_un_agent_renomme_un_dossier_et_son_tableau() {
    let (mut app, dossier, ouvre) = un_dossier_plein();
    let noms = |app: &GlucoseApp| {
        let f = app.store.active_board().expect("un tableau").folders[0]
            .name
            .clone();
        let b = nom_du_tableau(app, &ouvre).expect("son tableau");
        (f, b)
    };
    let r = appeler(
        &mut app,
        "glucose_renommer",
        &format!(r#"{{"id":"{dossier}","nom":"  archive "}}"#),
    );
    assert_eq!(texte(&r, "sorte"), Some("dossier"));
    assert_eq!(noms(&app), ("archive".into(), "archive".into()));
    assert!(app.store.undo(), "un seul geste");
    assert_eq!(noms(&app), ("Dossier".into(), "Dossier".into()));
    appeler(
        &mut app,
        "glucose_renommer",
        &format!(r#"{{"id":"{ouvre}","nom":"docs en ligne"}}"#),
    );
    assert_eq!(noms(&app), ("docs en ligne".into(), "docs en ligne".into()));
    let racine = board(&app);
    let r = appeler(
        &mut app,
        "glucose_renommer",
        &format!(r#"{{"id":"{racine}","nom":"Projet"}}"#),
    );
    assert_eq!(texte(&r, "sorte"), Some("tableau"));
    assert_eq!(nom_du_tableau(&app, &racine).as_deref(), Some("Projet"));
    for faux in [
        r#"{"id":"personne","nom":"x"}"#,
        r#"{"id":"archive-1","nom":"   "}"#,
    ] {
        assert!(app
            .executer_pour_un_agent("glucose_renommer", &args(faux))
            .is_err());
    }
}
