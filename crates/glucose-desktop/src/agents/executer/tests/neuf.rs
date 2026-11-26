//! **MCP-NEUF — quoi de neuf** (fiche 144) : une IA qui revient lit ce que les autres ont fait
//! depuis son dernier geste — chaque nœud une fois, posé, changé ou retiré, par qui, où — sans
//! relire le tableau.

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

fn proteo() -> Main {
    Main::Agent("proteo".into())
}

/// Les nœuds de la réponse : (id, fait, par), dans l'ordre rendu.
fn noeuds(j: &Json) -> Vec<(String, String, Vec<String>)> {
    let texte = |n: &Json, cle| {
        n.get(cle)
            .and_then(Json::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let Some(Json::Liste(l)) = j.get("noeuds") else {
        panic!("pas de nœuds : {j:?}")
    };
    (l.iter())
        .map(|n| {
            let par = match n.get("par") {
                Some(Json::Liste(p)) => p
                    .iter()
                    .filter_map(Json::as_str)
                    .map(String::from)
                    .collect(),
                _ => Vec::new(),
            };
            (texte(n, "id"), texte(n, "fait"), par)
        })
        .collect()
}

/// **Depuis son dernier geste** : ce que l'humain a changé, posé, retiré — le plus récent
/// d'abord ; ce qui est venu et reparti n'y est pas, ni ce que l'IA a fait elle-même.
#[test]
fn test_mcp_neuf_depuis_son_dernier_geste() {
    let dossier = Dossier::nouveau("mcp-neuf");
    let mut app = GlucoseApp::new();
    app.habiter(&dossier);
    let (moi, h) = (proteo(), Main::Humain);
    sous(
        &mut app,
        h.clone(),
        "glucose_poser_texte",
        r#"{"id":"a","texte":"Avant."}"#,
    );
    sous(
        &mut app,
        h.clone(),
        "glucose_poser_texte",
        r#"{"id":"r","texte":"À retirer."}"#,
    );
    sous(
        &mut app,
        moi.clone(),
        "glucose_poser_texte",
        r#"{"id":"p","texte":"De l'IA."}"#,
    );
    sous(
        &mut app,
        h.clone(),
        "glucose_poser_texte",
        r#"{"id":"a","texte":"Récrit. Suite."}"#,
    );
    sous(
        &mut app,
        h.clone(),
        "glucose_poser_texte",
        r#"{"id":"z","texte":"Passe."}"#,
    );
    sous(
        &mut app,
        h.clone(),
        "glucose_retirer",
        r#"{"ids":["z","r"]}"#,
    );
    sous(
        &mut app,
        h.clone(),
        "glucose_poser_texte",
        r#"{"id":"n","texte":"Neuf."}"#,
    );

    app.store.journal.main = moi;
    let j = appeler(&mut app, "glucose_quoi_de_neuf", "{}");
    let humain = vec!["humain".to_string()];
    assert_eq!(
        noeuds(&j),
        vec![
            ("n".into(), "pose".into(), humain.clone()),
            ("r".into(), "retire".into(), humain.clone()),
            ("a".into(), "change".into(), humain.clone()),
        ]
    );
    let Some(Json::Liste(l)) = j.get("noeuds") else {
        unreachable!()
    };
    assert_eq!(l[2].get("debut").and_then(Json::as_str), Some("Récrit."));
    assert_eq!(j.get("depuis"), Some(&Json::Nombre(3.0)), "après son geste");
    assert_eq!(j.get("jusqu_a"), Some(&Json::Nombre(7.0)));

    // Repartir de `jusqu_a` : plus rien de neuf.
    let j = appeler(&mut app, "glucose_quoi_de_neuf", r#"{"depuis":7}"#);
    assert!(noeuds(&j).is_empty(), "{j:?}");
    // Depuis le début : tout ce qui est là, chacun une fois, son geste à lui compris.
    let j = appeler(&mut app, "glucose_quoi_de_neuf", r#"{"depuis":0}"#);
    let tout: Vec<_> = noeuds(&j)
        .into_iter()
        .map(|(id, fait, _)| (id, fait))
        .collect();
    assert_eq!(
        tout,
        [("n", "pose"), ("a", "pose"), ("p", "pose")].map(|(i, f)| (i.into(), f.into()))
    );
}

/// **Où** : la membrane du nœud, telle qu'elle est maintenant.
#[test]
fn test_mcp_neuf_dit_la_membrane() {
    let dossier = Dossier::nouveau("mcp-neuf-membrane");
    let mut app = GlucoseApp::new();
    app.habiter(&dossier);
    let h = Main::Humain;
    sous(
        &mut app,
        h.clone(),
        "glucose_poser_membrane",
        r#"{"id":"m","titre":"Notes","x":0,"y":0,"largeur":800,"hauteur":600}"#,
    );
    sous(
        &mut app,
        h,
        "glucose_poser_texte",
        r#"{"id":"t","texte":"Ici.","dans":"m"}"#,
    );
    app.store.journal.main = proteo();
    let j = appeler(&mut app, "glucose_quoi_de_neuf", "{}");
    let Some(Json::Liste(l)) = j.get("noeuds") else {
        panic!("{j:?}")
    };
    let t = (l.iter())
        .find(|n| n.get("id").and_then(Json::as_str) == Some("t"))
        .expect("t");
    assert_eq!(t.get("membrane").and_then(Json::as_str), Some("m"));
}

/// **REGARD-1 — sur quoi il se concentre** (fiche 177) : sa dernière sélection, avec l'appareil et
/// l'heure ; rien choisi, rien de dit.
#[test]
fn test_regard_1_quoi_de_neuf_dit_sur_quoi_il_se_concentre() {
    let dossier = Dossier::nouveau("mcp-regard");
    let mut app = GlucoseApp::new();
    app.habiter(&dossier);
    sous(
        &mut app,
        Main::Humain,
        "glucose_poser_texte",
        r#"{"id":"a","texte":"Là."}"#,
    );
    app.suivre_le_regard();
    app.store.journal.main = proteo();
    let j = appeler(&mut app, "glucose_quoi_de_neuf", "{}");
    assert_eq!(j.get("regards"), None, "rien de choisi : {j:?}");
    app.store.clear_selection();
    if app
        .store
        .active_board()
        .is_some_and(|b| b.images.iter().any(|i| i.id == "a"))
    {
        app.store.select_image("a".into(), false);
    } else {
        app.store.select_annotation("a".into(), false);
    }
    app.suivre_le_regard();
    let j = appeler(&mut app, "glucose_quoi_de_neuf", "{}");
    let Some(Json::Liste(regards)) = j.get("regards") else {
        panic!("{j:?}")
    };
    let r = &regards[..];
    assert_eq!(r.len(), 1, "{j:?}");
    assert_eq!(r[0].get("appareil").and_then(Json::as_str), Some("ici"));
    assert_eq!(r[0].get("ids"), Some(&Json::Liste(vec![Json::from("a")])));
    let quand = r[0].get("quand").and_then(Json::as_str).unwrap_or_default();
    assert!(!quand.is_empty(), "l'heure est dite : {j:?}");
}

/// **Ce qui a changé dans un texte** (MCP-CHANGE, fiche 180) : sa réponse ajoutée sous une question
/// se lit seule, à sa ligne, en deux gestes ou un ; un texte récrit se relit entier ; un nœud
/// seulement déplacé ne dit rien de son texte.
#[test]
fn test_mcp_change_quoi_de_neuf_dit_ce_qui_a_change() {
    let dossier = Dossier::nouveau("mcp-change");
    let mut app = GlucoseApp::new();
    app.habiter(&dossier);
    let (moi, h) = (proteo(), Main::Humain);
    let q = r#"{"id":"q","texte":"**Une question**\n\nTon avis ?\n\nTa réponse :"}"#;
    sous(&mut app, moi.clone(), "glucose_poser_texte", q);
    let c = r#"{"id":"c","texte":"un\ndeux"}"#;
    sous(&mut app, moi.clone(), "glucose_poser_texte", c);
    sous(
        &mut app,
        moi.clone(),
        "glucose_poser_texte",
        r#"{"id":"d","texte":"Reste."}"#,
    );
    let oui = r#"{"id":"q","texte":"\nOui.","ajouter":true}"#;
    sous(&mut app, h.clone(), "glucose_ecrire", oui);
    let vraiment = r#"{"id":"q","texte":"\nVraiment.","ajouter":true}"#;
    sous(&mut app, h.clone(), "glucose_ecrire", vraiment);
    let recrit = r#"{"id":"c","texte":"trois\nquatre"}"#;
    sous(&mut app, h.clone(), "glucose_ecrire", recrit);
    sous(
        &mut app,
        h,
        "glucose_deplacer",
        r#"{"id":"d","x":900,"y":900}"#,
    );

    app.store.journal.main = moi;
    let j = appeler(&mut app, "glucose_quoi_de_neuf", "{}");
    let Some(Json::Liste(l)) = j.get("noeuds") else {
        panic!("pas de nœuds : {j:?}")
    };
    let changements = |id: &str| {
        let n = l
            .iter()
            .find(|n| n.get("id").and_then(Json::as_str) == Some(id));
        n.and_then(|n| n.get("changements").cloned())
    };
    let q = changements("q").expect("ce qui a changé dans q").ecrire();
    assert_eq!(q, r#"[{"ligne":6,"apres":"Oui.\nVraiment."}]"#);
    let c = changements("c").expect("ce qui a changé dans c");
    assert!(c.as_str().is_some_and(|t| t.starts_with("récrit")), "{c:?}");
    assert_eq!(changements("d"), None, "déplacé, son texte est le même");
}
