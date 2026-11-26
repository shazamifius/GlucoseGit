//! **MCP-AUTEUR — qui a écrit quoi, et quand** (fiche 181) : chaque nœud que lit une IA dit les
//! mains qui l'ont écrit et la dernière fois ; le document rouvert le redit, lu dans son histoire.

use super::appeler;
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use crate::epreuve::Dossier;
use glucose_core::store::journal::Main;

/// Un geste, sous cette main, écrit dans l'histoire.
fn sous(app: &mut GlucoseApp, main: Main, outil: &str, arguments: &str) {
    app.store.journal.main = main;
    appeler(app, outil, arguments);
    app.consigner();
}

/// Ce que la lecture dit de l'écriture de chaque nœud : (id, ses mains, sa date).
fn ecrits(app: &mut GlucoseApp) -> Vec<(String, Vec<String>, String)> {
    let j = appeler(app, "glucose_lire", "{}");
    let Some(Json::Liste(l)) = j.get("noeuds") else {
        panic!("pas de nœuds : {j:?}")
    };
    let mut rendus: Vec<_> = (l.iter())
        .filter_map(|n| {
            let id = n.get("id")?.as_str()?.to_string();
            let e = n.get("ecrit")?;
            let par = (e.get("par")?.as_liste()?.iter())
                .filter_map(|m| m.as_str().map(String::from))
                .collect();
            Some((id, par, e.get("le")?.as_str()?.to_string()))
        })
        .collect();
    rendus.sort();
    rendus
}

#[test]
fn mcp_auteur_chaque_noeud_dit_qui_l_a_ecrit_et_quand() {
    let dossier = Dossier::nouveau("mcp-auteur");
    let mut app = GlucoseApp::new();
    app.habiter(&dossier);
    let (ia, h) = (Main::Agent("proteo".into()), Main::Humain);
    sous(
        &mut app,
        ia.clone(),
        "glucose_poser_texte",
        r#"{"id":"a","texte":"Eau"}"#,
    );
    sous(
        &mut app,
        h.clone(),
        "glucose_poser_texte",
        r#"{"id":"b","texte":"Feu"}"#,
    );
    let pluie = r#"{"id":"a","texte":" et pluie","ajouter":true}"#;
    sous(&mut app, h.clone(), "glucose_ecrire", pluie);
    // Déplacer n'est pas écrire : b reste à l'humain seul.
    sous(
        &mut app,
        ia,
        "glucose_deplacer",
        r#"{"id":"b","x":900,"y":900}"#,
    );
    app.store.journal.main = h;

    let lus = ecrits(&mut app);
    let mains = |l: &[(String, Vec<String>, String)]| -> Vec<(String, Vec<String>)> {
        l.iter()
            .map(|(id, par, _)| (id.clone(), par.clone()))
            .collect()
    };
    let attendu = vec![
        (
            "a".to_string(),
            vec!["agent proteo".to_string(), "humain".to_string()],
        ),
        ("b".to_string(), vec!["humain".to_string()]),
    ];
    assert_eq!(mains(&lus), attendu);
    assert!(
        lus.iter().all(|(_, _, le)| le.contains('/')),
        "datés : {lus:?}"
    );

    // Rouvert, le document le redit, lu dans son histoire ; un seul scribe par fichier.
    let chemin = (app.disque.ecriture.as_ref())
        .expect("un brouillon")
        .chemin
        .clone();
    drop(app);
    let mut rouvert = GlucoseApp::new();
    rouvert.habiter(&dossier);
    rouvert.open_from(chemin);
    assert_eq!(
        ecrits(&mut rouvert),
        lus,
        "relu dans l'histoire, à la même date"
    );
}
