use super::*;

fn moderne(id: u32, methode: &str, params: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"{methode}","params":{{{params}"_meta":{{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{{"name":"proteo","version":"1"}}}}}}}}"#
    )
}

fn reponse(suite: Suite) -> Json {
    let Suite::Repondre(ligne) = suite else {
        panic!("une réponse était attendue : {suite:?}");
    };
    assert!(!ligne.contains('\n'), "un message par ligne");
    Json::lire(&ligne).expect("une réponse en JSON")
}

#[test]
fn un_client_moderne_decouvre_glucose_sans_poignee_de_main() {
    let mut c = Connexion::default();
    let r = reponse(c.traiter(&moderne(1, "server/discover", "")));
    let res = r.get("result").expect("un résultat");
    assert_eq!(
        res.get("resultType").and_then(Json::as_str),
        Some("complete")
    );
    let versions = res
        .get("supportedVersions")
        .and_then(Json::as_liste)
        .expect("versions");
    assert_eq!(versions[0].as_str(), Some(VERSION_MODERNE));
    assert!(res
        .get("capabilities")
        .and_then(|c| c.get("tools"))
        .is_some());
    let nom = res
        .get("_meta")
        .and_then(|m| m.get("io.modelcontextprotocol/serverInfo"))
        .and_then(|s| s.get("name"))
        .and_then(Json::as_str);
    assert_eq!(nom, Some("glucose"));
}

#[test]
fn un_appel_d_outil_moderne_part_vers_la_boucle_avec_le_nom_du_client() {
    let mut c = Connexion::default();
    let ligne = moderne(
        2,
        "tools/call",
        r#""name":"glucose_poser_texte","arguments":{"texte":"bonjour"},"#,
    );
    let Suite::Executer(appel) = c.traiter(&ligne) else {
        panic!("un appel était attendu");
    };
    assert_eq!(appel.nom, "glucose_poser_texte");
    assert_eq!(appel.client, "proteo");
    assert!(appel.moderne);
    assert_eq!(appel.id, Json::Nombre(2.0));
    assert_eq!(
        appel.arguments.get("texte").and_then(Json::as_str),
        Some("bonjour")
    );
    let ligne = reponse_d_outil(&appel, Ok(Json::objet([("id", Json::from("t1"))])));
    let r = Json::lire(&ligne).expect("lu");
    let res = r.get("result").expect("un résultat");
    assert_eq!(res.get("isError").and_then(Json::as_bool), Some(false));
    assert_eq!(
        res.get("resultType").and_then(Json::as_str),
        Some("complete")
    );
}

#[test]
fn une_version_moderne_inconnue_rend_les_versions_parlees() {
    let mut c = Connexion::default();
    let ligne = moderne(3, "tools/list", "").replace("2026-07-28", "2099-01-01");
    let r = reponse(c.traiter(&ligne));
    let e = r.get("error").expect("une erreur");
    assert_eq!(e.get("code").and_then(Json::as_f64), Some(-32022.0));
    let supportees = e
        .get("data")
        .and_then(|d| d.get("supported"))
        .and_then(Json::as_liste)
        .expect("les versions");
    assert!(supportees
        .iter()
        .any(|v| v.as_str() == Some(VERSION_MODERNE)));
}

#[test]
fn un_client_ancien_fait_sa_poignee_de_main_puis_appelle() {
    let mut c = Connexion::default();
    let avant = reponse(c.traiter(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#));
    assert!(
        avant.get("error").is_some(),
        "rien sans version ni poignée de main"
    );
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"claude-code","version":"2"}}}"#;
    let r = reponse(c.traiter(init));
    let res = r.get("result").expect("un résultat");
    assert_eq!(
        res.get("protocolVersion").and_then(Json::as_str),
        Some("2025-06-18")
    );
    assert!(
        res.get("resultType").is_none(),
        "l'ancienne époque ne le connaît pas"
    );
    assert_eq!(
        c.traiter(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#),
        Suite::Rien
    );
    let liste = reponse(c.traiter(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#));
    let outils = liste
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(Json::as_liste)
        .expect("les outils");
    assert_eq!(outils.len(), outils::noms().count());
    let appel =
        r#"{"jsonrpc":"2.0","id":"x","method":"tools/call","params":{"name":"glucose_lire"}}"#;
    let Suite::Executer(a) = c.traiter(appel) else {
        panic!("un appel était attendu");
    };
    assert_eq!((a.client.as_str(), a.moderne), ("claude-code", false));
}

#[test]
fn une_version_ancienne_inconnue_negocie_la_plus_recente() {
    let mut c = Connexion::default();
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2023-01-01"}}"#;
    let r = reponse(c.traiter(init));
    let v = r
        .get("result")
        .and_then(|r| r.get("protocolVersion"))
        .and_then(Json::as_str);
    assert_eq!(v, Some(VERSIONS_ANCIENNES[0]));
}

#[test]
fn un_outil_inconnu_une_methode_inconnue_et_une_ligne_illisible_sont_dites() {
    let mut c = Connexion::default();
    for (ligne, code) in [
        (
            moderne(4, "tools/call", r#""name":"effacer_le_disque","#),
            -32602.0,
        ),
        (moderne(5, "resources/list", ""), -32601.0),
        ("{pas du json".to_string(), -32700.0),
    ] {
        let r = reponse(c.traiter(&ligne));
        assert_eq!(
            r.get("error")
                .and_then(|e| e.get("code"))
                .and_then(Json::as_f64),
            Some(code),
            "{ligne}"
        );
    }
}

#[test]
fn chaque_schema_d_outil_se_lit() {
    let liste = outils::liste();
    for outil in liste.get("tools").and_then(Json::as_liste).expect("outils") {
        let schema = outil.get("inputSchema").expect("un schéma");
        assert_eq!(
            schema.get("type").and_then(Json::as_str),
            Some("object"),
            "{}",
            outil.ecrire()
        );
    }
}
