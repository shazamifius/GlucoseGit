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
