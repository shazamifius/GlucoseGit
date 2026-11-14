//! Les épreuves de la porte publique : tout le chemin d'un accord, rejoué sans réseau ni fenêtre —
//! la découverte, la demande de Claude, le code, la réponse de l'utilisateur, l'échange de la clé,
//! puis les outils, dans leur portée.

use super::accords::{Accord, Droits, Etat};
use super::http::{traiter, Contexte, Requete};
use super::oauth::{Guichet, Issue};
use super::Adresses;
use crate::agents::json::Json;
use crate::agents::{Boite, Demande};
use glucose_core::encodage::base64_adresse;
use glucose_core::hash::sha256;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const RACINE: &str = "https://relais.exemple";
const CLAUDE: &str = "https://claude.ai/oauth/mcp-oauth-client-metadata";
const RETOUR: &str = "https://claude.ai/api/mcp/auth_callback";
/// Le document d'identité que claude.ai publie, tel qu'il était le 14/11/2026.
const DOCUMENT_DE_CLAUDE: &str = r#"{"client_id":"https://claude.ai/oauth/mcp-oauth-client-metadata","client_name":"Claude","client_uri":"https://claude.ai","redirect_uris":["https://claude.ai/api/mcp/auth_callback"],"grant_types":["authorization_code","refresh_token","urn:ietf:params:oauth:grant-type:jwt-bearer"],"response_types":["code"],"token_endpoint_auth_method":"none"}"#;
const DOCUMENT_DE_CLAUDE_CODE: &str = r#"{"client_id":"https://claude.ai/oauth/claude-code-client-metadata","client_name":"Claude Code","client_uri":"https://claude.ai","redirect_uris":["http://localhost/callback","http://127.0.0.1/callback"],"grant_types":["authorization_code","refresh_token"],"response_types":["code"],"token_endpoint_auth_method":"none"}"#;
const VERIFICATEUR: &str = "le-secret-de-claude-qui-fait-bien-plus-de-quarante-trois-signes";

fn chercher(adresse: &str) -> Result<String, String> {
    match adresse {
        CLAUDE => Ok(DOCUMENT_DE_CLAUDE.into()),
        "https://claude.ai/oauth/claude-code-client-metadata" => Ok(DOCUMENT_DE_CLAUDE_CODE.into()),
        "https://faussaire.exemple/id" => Ok(DOCUMENT_DE_CLAUDE.into()),
        // Il renvoie bien chez lui, mais prétend être le document de claude.ai.
        "https://imposteur.exemple/id" => Ok(r#"{"client_id":"https://claude.ai/oauth/mcp-oauth-client-metadata","client_name":"Claude","redirect_uris":["https://imposteur.exemple/retour"]}"#.into()),
        // Il se dit bien lui-même, mais renvoie ailleurs que chez lui.
        "https://etrange.exemple/id" => Ok(r#"{"client_id":"https://etrange.exemple/id","client_name":"Étrange","redirect_uris":["https://ailleurs.exemple/retour"]}"#.into()),
        _ => Err("introuvable".into()),
    }
}

/// Un Glucose et sa porte, sans fenêtre : son guichet, un fichier à lui, une boîte que personne
/// ne relève — sauf l'épreuve qui joue la boucle.
struct Banc {
    adresses: Adresses,
    guichet: Mutex<Guichet>,
    fichier: PathBuf,
    boite: Boite,
}

impl Banc {
    fn nouveau(nom: &str) -> Banc {
        let etat = Etat {
            secret: [7; 32],
            ouverte: true,
            accords: Vec::new(),
        };
        let dossier = std::env::temp_dir().join(format!(
            "glucose-porte-publique-{nom}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dossier).expect("un dossier");
        Banc {
            adresses: Adresses::nouvelles(RACINE, &etat.identifiant()),
            guichet: Mutex::new(Guichet::nouveau(etat)),
            fichier: dossier.join(super::accords::FICHIER),
            boite: Boite::nouvelle(Arc::new(|| {})),
        }
    }

    fn requete(
        &self,
        methode: &str,
        chemin: &str,
        entetes: &[(&str, &str)],
        corps: &str,
    ) -> (u16, Json, Vec<(&'static str, String)>) {
        let r = Requete {
            n: 1.0,
            methode: methode.into(),
            chemin: chemin.into(),
            entetes: entetes
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            corps: corps.into(),
        };
        let c = Contexte {
            adresses: &self.adresses,
            guichet: &self.guichet,
            fichier: &self.fichier,
            boite: &self.boite,
            chercher: &chercher,
            maintenant: 1_800_000_000,
        };
        let rep = traiter(&r, &c);
        let json = Json::lire(&rep.corps).unwrap_or(Json::Nul);
        (rep.statut, json, rep.entetes)
    }

    /// La page d'accord envoie ses paramètres ; rend la demande et son code.
    fn demander(&self, client_id: &str, retour: &str) -> (u16, Json) {
        let defi = base64_adresse(&sha256(VERIFICATEUR.as_bytes()));
        let params = Json::objet([
            ("response_type", Json::from("code")),
            ("client_id", Json::from(client_id)),
            ("redirect_uri", Json::from(retour)),
            ("code_challenge", Json::from(defi)),
            ("code_challenge_method", Json::from("S256")),
            ("state", Json::from("xyz")),
            ("resource", Json::from(self.adresses.mcp())),
        ]);
        let (s, j, _) = self.requete("POST", "/authorize/demande", &[], &params.ecrire());
        (s, j)
    }

    /// L'utilisateur répond dans Glucose.
    fn repondre(&self, issue: Issue) {
        let mut g = self.guichet.lock().expect("guichet");
        for a in &mut g.autorisations {
            a.issue = issue;
        }
    }

    fn attendre(&self, demande: &str) -> Json {
        let corps = Json::objet([("demande", Json::from(demande))]).ecrire();
        self.requete("POST", "/authorize/attente", &[], &corps).1
    }

    /// Tout le chemin, jusqu'à la clé.
    fn obtenir_une_cle(&self, droits: Droits) -> String {
        let (_, d) = self.demander(CLAUDE, RETOUR);
        self.repondre(Issue::Accordee(droits));
        let e = self.attendre(d.get("demande").and_then(Json::as_str).expect("demande"));
        let code = code_de(
            e.get("redirection")
                .and_then(Json::as_str)
                .expect("redirection"),
        );
        let (s, j, _) = self.requete("POST", "/token", &[], &formulaire(&code, VERIFICATEUR));
        assert_eq!(s, 200, "{j:?}");
        j.get("access_token")
            .and_then(Json::as_str)
            .expect("une clé")
            .to_string()
    }

    fn mcp(&self, cle: &str, corps: &str) -> (u16, Json) {
        let porteur = format!("Bearer {cle}");
        let (s, j, _) = self.requete("POST", "/mcp", &[("authorization", &porteur)], corps);
        (s, j)
    }
}

fn code_de(redirection: &str) -> String {
    let requete = redirection.split_once('?').expect("des paramètres").1;
    super::http::formulaire(requete)
        .into_iter()
        .find(|(k, _)| k == "code")
        .expect("un code")
        .1
}

fn formulaire(code: &str, verificateur: &str) -> String {
    let p = super::http::pourcent;
    format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}",
        p(code),
        p(RETOUR),
        p(CLAUDE),
        p(verificateur)
    )
}

fn ancien(id: u32, methode: &str, params: &str) -> String {
    format!(r#"{{"jsonrpc":"2.0","id":{id},"method":"{methode}","params":{{{params}}}}}"#)
}

fn noms_des_outils(j: &Json) -> Vec<String> {
    j.get("result")
        .and_then(|r| r.get("tools"))
        .and_then(Json::as_liste)
        .expect("des outils")
        .iter()
        .filter_map(|t| t.get("name").and_then(Json::as_str).map(str::to_string))
        .collect()
}

#[test]
fn la_decouverte_dit_ce_qu_il_faut_a_claude_pour_choisir_son_document_d_identite() {
    let b = Banc::nouveau("decouverte");
    let (s, ressource, _) = b.requete("GET", "/.well-known/oauth-protected-resource", &[], "");
    assert_eq!(s, 200);
    assert_eq!(
        ressource.get("resource").and_then(Json::as_str),
        Some(b.adresses.mcp().as_str())
    );
    let (_, guichet, _) = b.requete("GET", "/.well-known/oauth-authorization-server", &[], "");
    assert_eq!(
        guichet.get("issuer").and_then(Json::as_str),
        Some(b.adresses.base().as_str())
    );
    assert_eq!(
        guichet.get("client_id_metadata_document_supported"),
        Some(&Json::Bool(true))
    );
    let methodes = guichet
        .get("token_endpoint_auth_methods_supported")
        .expect("méthodes");
    assert_eq!(methodes, &Json::Liste(vec![Json::from("none")]));
    let defis = guichet
        .get("code_challenge_methods_supported")
        .expect("défis");
    assert_eq!(defis, &Json::Liste(vec![Json::from("S256")]));
}

/// Sans clé, `401` et l'adresse de la découverte : c'est ce qui lance l'autorisation chez Claude
/// (un `200` ne la lance jamais).
#[test]
fn sans_cle_la_porte_rend_401_et_dit_ou_lire_la_decouverte() {
    let b = Banc::nouveau("sans-cle");
    let (s, _, entetes) = b.requete("POST", "/mcp", &[], &ancien(1, "tools/list", ""));
    assert_eq!(s, 401);
    let defi = entetes
        .iter()
        .find(|(k, _)| *k == "www-authenticate")
        .expect("un défi");
    assert_eq!(
        defi.1,
        format!("Bearer resource_metadata=\"{}\"", b.adresses.metadonnees())
    );
    let (s, _) = b.mcp("une-cle-inventee", &ancien(1, "tools/list", ""));
    assert_eq!(s, 401);
}

/// **Tout le chemin** : Claude demande, le code s'affiche, l'utilisateur accorde, la clé se donne,
/// s'enregistre — et ses gestes sont signés « Claude (claude.ai) ».
#[test]
fn claude_obtient_une_cle_par_le_clic_de_l_utilisateur_et_signe_de_son_nom() {
    let b = Banc::nouveau("chemin");
    let (s, d) = b.demander(CLAUDE, RETOUR);
    assert_eq!(s, 200, "{d:?}");
    let code_affiche = d.get("code").and_then(Json::as_str).expect("un code");
    assert!(code_affiche.len() == 3 && code_affiche.bytes().all(|o| o.is_ascii_digit()));
    let client = d.get("client").expect("le client");
    assert_eq!(client.get("hote").and_then(Json::as_str), Some("claude.ai"));
    let demande = d
        .get("demande")
        .and_then(Json::as_str)
        .expect("demande")
        .to_string();
    assert_eq!(
        b.attendre(&demande).get("etat").and_then(Json::as_str),
        Some("attente")
    );
    b.repondre(Issue::Accordee(Droits::Ecrire));
    let e = b.attendre(&demande);
    assert_eq!(e.get("etat").and_then(Json::as_str), Some("accorde"));
    let redirection = e
        .get("redirection")
        .and_then(Json::as_str)
        .expect("redirection");
    assert!(
        redirection.starts_with(&format!("{RETOUR}?code=")),
        "{redirection}"
    );
    assert!(redirection.contains("state=xyz"));
    assert!(redirection.contains(&format!(
        "iss={}",
        super::http::pourcent(&b.adresses.base())
    )));
    let code = code_de(redirection);
    let (s, j, _) = b.requete("POST", "/token", &[], &formulaire(&code, VERIFICATEUR));
    assert_eq!(s, 200, "{j:?}");
    assert_eq!(j.get("token_type").and_then(Json::as_str), Some("Bearer"));
    assert!(j.get("expires_in").is_none() && j.get("refresh_token").is_none());
    let cle = j
        .get("access_token")
        .and_then(Json::as_str)
        .expect("clé")
        .to_string();
    // L'accord est sur le disque, par l'empreinte de la clé seulement.
    let disque = std::fs::read_to_string(&b.fichier).expect("enregistré");
    assert!(!disque.contains(&cle));
    let relu = Etat::lire(&disque).expect("relisible");
    assert_eq!(
        relu.accord_de(&cle).map(Accord::signature).as_deref(),
        Some("Claude (claude.ai)")
    );
    // Le code ne sert qu'une fois.
    let (s, j, _) = b.requete("POST", "/token", &[], &formulaire(&code, VERIFICATEUR));
    assert_eq!(
        (s, j.get("error").and_then(Json::as_str)),
        (400, Some("invalid_grant"))
    );
    // Un appel d'outil part vers la boucle, signé du nom vérifié — pas de celui qu'il déclare.
    let boucle = b.boite.clone();
    let fil = std::thread::spawn(move || loop {
        if let Some(Demande { appel, reponse }) = boucle.relever().pop() {
            let _ = reponse.send(crate::agents::protocole::reponse_d_outil(
                &appel,
                Ok(Json::from("posé")),
            ));
            return appel;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    });
    let appel = ancien(
        3,
        "tools/call",
        r#""name":"glucose_poser_texte","arguments":{"texte":"bonjour"}"#,
    );
    let (s, j) = b.mcp(&cle, &appel);
    assert_eq!(s, 200, "{j:?}");
    let parti = fil.join().expect("la boucle a reçu l'appel");
    assert_eq!(parti.client, "Claude (claude.ai)");
    assert!(!parti.portee.machine && parti.portee.ecrire);
}

/// La preuve PKCE : un code volé en route ne sert à rien sans le secret dont le défi était
/// l'empreinte — et le code est brûlé.
#[test]
fn un_code_sans_la_bonne_preuve_ne_donne_rien_et_se_brule() {
    let b = Banc::nouveau("pkce");
    let (_, d) = b.demander(CLAUDE, RETOUR);
    b.repondre(Issue::Accordee(Droits::Ecrire));
    let e = b.attendre(d.get("demande").and_then(Json::as_str).expect("demande"));
    let code = code_de(
        e.get("redirection")
            .and_then(Json::as_str)
            .expect("redirection"),
    );
    let (s, j, _) = b.requete(
        "POST",
        "/token",
        &[],
        &formulaire(&code, "un-autre-secret-un-autre-secret-un-autre-secret"),
    );
    assert_eq!(
        (s, j.get("error").and_then(Json::as_str)),
        (400, Some("invalid_grant"))
    );
    let (s, _, _) = b.requete("POST", "/token", &[], &formulaire(&code, VERIFICATEUR));
    assert_eq!(s, 400, "le code a été brûlé au premier essai");
    assert!(b.guichet.lock().expect("guichet").etat.accords.is_empty());
}

#[test]
fn un_refus_dans_glucose_revient_a_l_ia_comme_access_denied() {
    let b = Banc::nouveau("refus");
    let (_, d) = b.demander(CLAUDE, RETOUR);
    b.repondre(Issue::Refusee);
    let e = b.attendre(d.get("demande").and_then(Json::as_str).expect("demande"));
    assert_eq!(e.get("etat").and_then(Json::as_str), Some("refuse"));
    let r = e
        .get("redirection")
        .and_then(Json::as_str)
        .expect("redirection");
    assert!(
        r.starts_with(&format!("{RETOUR}?error=access_denied&state=xyz")),
        "{r}"
    );
}

/// Une identité se vérifie : un document qui ne dit pas son propre client_id, un retour qu'il ne
/// déclare pas, un retour hors de son domaine — refusés avant toute question.
#[test]
fn une_identite_fausse_ou_un_retour_etranger_sont_refuses_avant_toute_question() {
    let b = Banc::nouveau("identite");
    for (client, retour) in [
        ("https://faussaire.exemple/id", RETOUR),
        (
            "https://imposteur.exemple/id",
            "https://imposteur.exemple/retour",
        ),
        (CLAUDE, "https://claude.ai/ailleurs"),
        (CLAUDE, "https://pirate.exemple/api/mcp/auth_callback"),
        ("https://introuvable.exemple/id", RETOUR),
        (
            "https://etrange.exemple/id",
            "https://ailleurs.exemple/retour",
        ),
        ("pas-une-adresse", RETOUR),
    ] {
        let (s, j) = b.demander(client, retour);
        assert_eq!(s, 400, "{client} {retour} : {j:?}");
    }
    assert!(b.guichet.lock().expect("guichet").autorisations.is_empty());
    // Une demande pour une autre ressource que ce Glucose (RFC 8707).
    let defi = base64_adresse(&sha256(VERIFICATEUR.as_bytes()));
    let params = Json::objet([
        ("response_type", Json::from("code")),
        ("client_id", Json::from(CLAUDE)),
        ("redirect_uri", Json::from(RETOUR)),
        ("code_challenge", Json::from(defi)),
        ("code_challenge_method", Json::from("S256")),
        ("resource", Json::from("https://relais.exemple/g/autre/mcp")),
    ]);
    let (s, _, _) = b.requete("POST", "/authorize/demande", &[], &params.ecrire());
    assert_eq!(s, 400, "une ressource étrangère");
}

/// Claude Code tire un port à chaque session : son retour de boucle locale se reconnaît au port
/// près (RFC 8252 § 7.3).
#[test]
fn le_retour_de_claude_code_se_reconnait_au_port_pres() {
    let b = Banc::nouveau("boucle");
    let id = "https://claude.ai/oauth/claude-code-client-metadata";
    let (s, d) = b.demander(id, "http://localhost:51234/callback");
    assert_eq!(s, 200, "{d:?}");
    assert_eq!(
        d.get("client")
            .and_then(|c| c.get("hote"))
            .and_then(Json::as_str),
        Some("claude.ai")
    );
    let (s, _) = b.demander(id, "http://localhost:51234/autre");
    assert_eq!(s, 400);
}

/// L'inscription (DCR) ne garde rien : l'identifiant porte le nom et les retours ; l'hôte montré
/// est celui du retour, que l'inscription ne peut pas truquer.
#[test]
fn une_ia_qui_s_inscrit_est_nommee_par_son_retour() {
    let b = Banc::nouveau("dcr");
    let corps = r#"{"client_name":"ChatGPT","redirect_uris":["https://chatgpt.com/connector_platform_oauth_redirect"]}"#;
    let (s, j, _) = b.requete("POST", "/register", &[], corps);
    assert_eq!(s, 201, "{j:?}");
    let id = j
        .get("client_id")
        .and_then(Json::as_str)
        .expect("un identifiant");
    let (s, d) = b.demander(id, "https://chatgpt.com/connector_platform_oauth_redirect");
    assert_eq!(s, 200, "{d:?}");
    let client = d.get("client").expect("client");
    assert_eq!(client.get("nom").and_then(Json::as_str), Some("ChatGPT"));
    assert_eq!(
        client.get("hote").and_then(Json::as_str),
        Some("chatgpt.com")
    );
    let (s, _) = b.demander(id, "https://ailleurs.exemple/retour");
    assert_eq!(s, 400);
    let (s, _, _) = b.requete(
        "POST",
        "/register",
        &[],
        r#"{"redirect_uris":["http://pirate.exemple/"]}"#,
    );
    assert_eq!(s, 400, "un retour en clair hors de cette machine");
}

/// **La portée** : une IA du nuage ne voit jamais l'outil qui lit les autres fichiers ; « Lire
/// seulement » ne donne que les outils qui lisent, et refuse un outil qui écrit.
#[test]
fn la_portee_borne_les_outils_d_une_ia_du_nuage() {
    let b = Banc::nouveau("portee");
    let ecrire = b.obtenir_une_cle(Droits::Ecrire);
    let (_, j) = b.mcp(&ecrire, &ancien(1, "tools/list", ""));
    let outils = noms_des_outils(&j);
    assert!(outils.contains(&"glucose_poser_texte".to_string()));
    assert!(!outils.contains(&"glucose_domaines".to_string()));
    let b = Banc::nouveau("portee-lire");
    let lire = b.obtenir_une_cle(Droits::Lire);
    let (_, j) = b.mcp(&lire, &ancien(1, "tools/list", ""));
    assert_eq!(
        noms_des_outils(&j),
        vec!["glucose_lire", "glucose_chercher", "glucose_histoire"]
    );
    let (s, j) = b.mcp(
        &lire,
        &ancien(
            2,
            "tools/call",
            r#""name":"glucose_poser_texte","arguments":{"texte":"a"}"#,
        ),
    );
    assert_eq!(s, 200);
    let message = j
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(message.contains("lecture"), "{message}");
}

/// Une requête moderne dont les en-têtes contredisent le corps est refusée (« HeaderMismatch ») ;
/// la même, aux bons en-têtes, passe — sous le nom vérifié, pas sous celui qu'elle déclare.
#[test]
fn une_requete_moderne_doit_dire_dans_ses_entetes_ce_que_dit_son_corps() {
    let b = Banc::nouveau("entetes");
    let cle = b.obtenir_une_cle(Droits::Ecrire);
    let corps = r#"{"jsonrpc":"2.0","id":4,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"un-faux-nom","version":"1"}}}}"#;
    let porteur = format!("Bearer {cle}");
    let (s, j, _) = b.requete(
        "POST",
        "/mcp",
        &[
            ("authorization", &porteur),
            ("mcp-protocol-version", "2026-07-28"),
            ("mcp-method", "tools/call"),
        ],
        corps,
    );
    assert_eq!(s, 400);
    assert_eq!(
        j.get("error")
            .and_then(|e| e.get("code"))
            .and_then(Json::as_f64),
        Some(-32020.0)
    );
    let (s, j, _) = b.requete(
        "POST",
        "/mcp",
        &[
            ("authorization", &porteur),
            ("mcp-protocol-version", "2026-07-28"),
            ("mcp-method", "tools/list"),
        ],
        corps,
    );
    assert_eq!(s, 200, "{j:?}");
}

/// Un client de l'époque ancienne : sa poignée de main se répond sans session, sa notification
/// rend `202`, et un `GET` rend `405`.
#[test]
fn un_client_ancien_est_servi_sans_session() {
    let b = Banc::nouveau("ancien");
    let cle = b.obtenir_une_cle(Droits::Ecrire);
    let init = ancien(
        1,
        "initialize",
        r#""protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"claude-ai","version":"0.1"}"#,
    );
    let (s, j) = b.mcp(&cle, &init);
    assert_eq!(s, 200, "{j:?}");
    assert_eq!(
        j.get("result")
            .and_then(|r| r.get("protocolVersion"))
            .and_then(Json::as_str),
        Some("2025-06-18")
    );
    let (s, _) = b.mcp(
        &cle,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    );
    assert_eq!(s, 202);
    let porteur = format!("Bearer {cle}");
    let (s, _, entetes) = b.requete("GET", "/mcp", &[("authorization", &porteur)], "");
    assert_eq!(s, 405);
    assert!(entetes.contains(&("allow", "POST".to_string())));
}

/// **Retirer une IA** : sa clé ne vaut plus rien, à la requête suivante.
#[test]
fn une_ia_retiree_ne_passe_plus() {
    let b = Banc::nouveau("retrait");
    let cle = b.obtenir_une_cle(Droits::Ecrire);
    assert_eq!(b.mcp(&cle, &ancien(1, "tools/list", "")).0, 200);
    assert_eq!(b.mcp("une-autre-cle", &ancien(1, "tools/list", "")).0, 401);
    b.guichet.lock().expect("guichet").etat.accords.clear();
    assert_eq!(b.mcp(&cle, &ancien(1, "tools/list", "")).0, 401);
}

#[test]
fn l_etat_se_relit_tel_qu_il_s_ecrit_et_une_ia_revenue_reprend_sa_place() {
    let mut etat = Etat::neuf().expect("un secret");
    let accord = |empreinte: &str| Accord {
        client_id: CLAUDE.into(),
        nom: "Claude".into(),
        hote: "claude.ai".into(),
        droits: Droits::Lire,
        empreinte: empreinte.into(),
        depuis: 3,
    };
    etat.accorder(accord("a"));
    etat.accorder(accord("b"));
    assert_eq!(etat.accords.len(), 1, "le même client reprend sa place");
    assert_eq!(Etat::lire(&etat.ecrire()), Some(etat.clone()));
    assert_eq!(etat.identifiant().len(), 32);
    assert_eq!(Etat::lire("{}"), None);
}

#[test]
fn les_adresses_d_un_glucose_se_tirent_du_relais_et_de_son_identifiant() {
    let a = Adresses::nouvelles("https://relais.exemple/", "0123");
    assert_eq!(a.mcp(), "https://relais.exemple/g/0123/mcp");
    assert_eq!(a.websocket(), "wss://relais.exemple/g/0123/glucose");
    assert_eq!(
        a.metadonnees(),
        "https://relais.exemple/.well-known/oauth-protected-resource/g/0123/mcp"
    );
    assert_eq!(
        Adresses::nouvelles("http://127.0.0.1:8787", "0").websocket(),
        "ws://127.0.0.1:8787/g/0/glucose"
    );
}

/// **Le nom qui signe est celui de la clé**, jamais celui que la requête déclare : ni dans le
/// `_meta` d'une requête moderne, ni dans la poignée de main d'un client ancien.
#[test]
fn le_nom_verifie_l_emporte_sur_le_nom_declare() {
    use crate::agents::outils::Portee;
    use crate::agents::protocole::{Connexion, Suite};
    let moderne = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"glucose_lire","arguments":{},"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"un-intrus","version":"1"}}}}"#;
    let mut c = Connexion::publique("Claude (claude.ai)", Portee::publique(true));
    let Suite::Executer(appel) = c.traiter(moderne) else {
        panic!("un appel");
    };
    assert_eq!(appel.client, "Claude (claude.ai)");
    let init = ancien(
        1,
        "initialize",
        r#""protocolVersion":"2025-06-18","clientInfo":{"name":"un-intrus","version":"1"}"#,
    );
    let _ = c.traiter(&init);
    let Suite::Executer(appel) = c.traiter(&ancien(
        2,
        "tools/call",
        r#""name":"glucose_lire","arguments":{}"#,
    )) else {
        panic!("un appel");
    };
    assert_eq!(appel.client, "Claude (claude.ai)");
}
