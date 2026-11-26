//! MCP-COURT (fiche 142) : une lecture ciblée ne répète pas le paysage — sa priorité du 24/11, les
//! jetons.

use super::appeler;
use crate::app::GlucoseApp;

/// **Par identifiant ou par phrase, la réponse ne redit que l'essentiel** : le tableau, où il est,
/// la sélection, les nœuds, les introuvables ; le document, la partie visible, les onglets et les
/// domaines restent au sommaire, qui oriente.
#[test]
fn mcp_court_une_lecture_ciblee_ne_repete_pas_le_paysage() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"t","texte":"Une phrase d'eau."}"#,
    );
    let paysage = ["document", "fichier", "visible", "tableaux", "domaines"];
    let sommaire = appeler(&mut app, "glucose_lire", "{}");
    for champ in paysage {
        assert!(
            sommaire.get(champ).is_some(),
            "le sommaire oriente : {champ}"
        );
    }
    for arguments in [r#"{"ids":["t","absent"]}"#, r#"{"cherche":"eau"}"#] {
        let lu = appeler(&mut app, "glucose_lire", arguments);
        for champ in paysage {
            assert!(lu.get(champ).is_none(), "{arguments} redit {champ}");
        }
        for champ in ["tableau", "ou", "selection", "noeuds"] {
            assert!(lu.get(champ).is_some(), "{arguments} sans {champ}");
        }
    }
    let lu = appeler(&mut app, "glucose_lire", r#"{"ids":["t","absent"]}"#);
    assert!(lu.get("introuvables").is_some(), "l'absent reste dit");
}

/// **MCP-FLECHES** (fiche 178) : « fleches » rend, d'une lecture, ce qu'un nœud désigne — en
/// entier — et ce qui pointe vers lui, en une phrase.
#[test]
fn mcp_fleches_une_lecture_rend_ce_qu_il_designe() {
    let mut app = GlucoseApp::new();
    for (id, texte) in [
        ("c", "Consigne. Lis ceci."),
        ("u", "URGENT. Les jetons."),
        ("r", "Réponse. Faite."),
    ] {
        let t = format!(r#"{{"id":"{id}","texte":"{texte}"}}"#);
        appeler(&mut app, "glucose_poser_texte", &t);
    }
    appeler(&mut app, "glucose_relier", r#"{"source":"c","cible":"u"}"#);
    appeler(&mut app, "glucose_relier", r#"{"source":"r","cible":"c"}"#);
    let lu = appeler(&mut app, "glucose_lire", r#"{"ids":["c"],"fleches":true}"#);
    let Some(crate::agents::json::Json::Liste(noeuds)) = lu.get("noeuds") else {
        panic!("{lu:?}")
    };
    let champ = |id: &str, cle: &str| {
        let n = noeuds
            .iter()
            .find(|n| n.get("id").and_then(|i| i.as_str()) == Some(id));
        n.and_then(|n| n.get(cle))
            .and_then(|v| v.as_str())
            .map(String::from)
    };
    assert_eq!(
        champ("u", "texte").as_deref(),
        Some("URGENT. Les jetons."),
        "{lu:?}"
    );
    assert_eq!(
        champ("r", "texte"),
        None,
        "ce qui pointe vers lui : une phrase"
    );
    assert_eq!(champ("r", "debut").as_deref(), Some("Réponse."), "{lu:?}");
    assert_eq!(
        noeuds.len(),
        5,
        "le nœud, ses deux flèches, leurs deux bouts : {lu:?}"
    );
    let seul = appeler(&mut app, "glucose_lire", r#"{"ids":["c"]}"#);
    assert!(
        matches!(seul.get("noeuds"), Some(crate::agents::json::Json::Liste(l)) if l.len() == 1)
    );
}
