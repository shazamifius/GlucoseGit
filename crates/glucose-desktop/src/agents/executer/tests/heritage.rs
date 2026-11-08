//! DOM-6 pour un agent : un domaine posé sur une membrane se lit sur ce qu'elle contient.

use super::{appeler, nombre};
use crate::agents::json::Json;
use crate::app::GlucoseApp;

/// **Posé une fois sur la membrane, lu sur tout ce qu'elle porte** : `glucose_lire domaine=` rend la
/// membrane et ses textes ; `glucose_domaines` les liste, chacun disant de quelle membrane il le
/// tient ; le compte des endroits ne dit que la membrane ; le sommaire ne le répète pas.
#[test]
fn dom_6_un_agent_lit_l_heritage() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","titre":"Eau","x":0,"y":0}"#,
    );
    for i in 0..3 {
        let a = format!(r#"{{"id":"p{i}","texte":"Le paragraphe {i}.","dans":"m"}}"#);
        appeler(&mut app, "glucose_poser_texte", &a);
    }
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"dehors","texte":"Le feu.","x":9000,"y":0}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"Eau","ids":["m"]}"#,
    );
    let lus = appeler(&mut app, "glucose_lire", r#"{"domaine":"Eau"}"#);
    let ids: Vec<&str> = lus
        .get("noeuds")
        .and_then(Json::as_liste)
        .expect("des nœuds")
        .iter()
        .filter_map(|n| n.get("id").and_then(Json::as_str))
        .collect();
    assert_eq!(ids.len(), 4, "la membrane et ses trois textes : {ids:?}");
    assert!(!ids.contains(&"dehors"));
    let sommaire = appeler(&mut app, "glucose_lire", "{}");
    let p0 = sommaire
        .get("noeuds")
        .and_then(Json::as_liste)
        .and_then(|l| {
            l.iter()
                .find(|n| n.get("id").and_then(Json::as_str) == Some("p0"))
        })
        .expect("p0");
    assert!(
        p0.get("domaines").is_none(),
        "le sommaire ne répète pas l'héritage"
    );

    let r = appeler(&mut app, "glucose_domaines", r#"{"domaine":"Eau"}"#);
    let noeuds = r.get("noeuds").and_then(Json::as_liste).expect("des nœuds");
    assert_eq!(noeuds.len(), 4);
    let herites = noeuds
        .iter()
        .filter(|n| n.get("herite_de").and_then(Json::as_str) == Some("m"))
        .count();
    assert_eq!(herites, 3, "chacun dit de quelle membrane il le tient");
    assert_eq!(nombre(&r, &["domaine", "noeuds"]), 1.0, "un seul endroit");
}
