//! **Le protocole MCP**, une ligne à la fois — des deux époques (spécification du 28/07/2026,
//! « Versioning and Compatibility ») :
//!
//! * **moderne** (`2026-07-28`) : sans état ; chaque requête porte sa version et l'identité de
//!   son client dans `_meta` ; `server/discover` dit ce que Glucose parle ;
//! * **ancienne** (`2025-11-25` et avant) : une poignée de main `initialize`, puis les requêtes.
//!
//! Glucose sert les deux sur la même connexion (« dual-era ») : un client moderne — celui de
//! Protéo — comme un client qui ne connaît que l'ancienne — la plupart des autres aujourd'hui.
//!
//! Ce module est **pur** : il lit une ligne et dit quoi faire. Répondre tout de suite (la
//! découverte, la liste des outils), ou confier un appel d'outil à la boucle de Glucose, qui
//! seule touche au document ([`super::Demande`]).

use super::json::Json;
use super::outils::{self, Portee};

pub const VERSION_MODERNE: &str = "2026-07-28";
/// Les versions anciennes que Glucose accepte de négocier, de la plus récente à la plus vieille.
/// Les outils sont les mêmes pour toutes : rien de ce qu'elles ont changé ne touche à
/// `tools/list` ni à `tools/call` tels que Glucose les emploie.
const VERSIONS_ANCIENNES: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
/// La version qu'une requête moderne demande à lire dans `_meta`.
pub(crate) const CLE_VERSION: &str = "io.modelcontextprotocol/protocolVersion";
pub(crate) const CLE_CLIENT: &str = "io.modelcontextprotocol/clientInfo";

/// Les codes d'erreur de JSON-RPC, et celui que la spécification moderne réserve à une version
/// inconnue.
const ERREUR_DE_LECTURE: f64 = -32700.0;
const METHODE_INCONNUE: f64 = -32601.0;
const ARGUMENTS_INVALIDES: f64 = -32602.0;
const VERSION_INCONNUE: f64 = -32022.0;
/// Les en-têtes d'une requête HTTP ne disent pas ce que dit son corps (« HeaderMismatch »).
const ENTETES_CONTRAIRES: f64 = -32020.0;

/// Ce qu'une ligne demande.
#[derive(Debug, Clone, PartialEq)]
pub enum Suite {
    /// Une ligne à rendre tout de suite.
    Repondre(String),
    /// Un outil à exécuter par la boucle ; la réponse s'écrit ensuite par [`reponse_d_outil`].
    Executer(Appel),
    /// Une notification, ou une réponse qu'un client n'a pas à envoyer : rien à rendre.
    Rien,
}

/// **Un appel d'outil**, tel qu'il part vers la boucle.
#[derive(Debug, Clone, PartialEq)]
pub struct Appel {
    pub id: Json,
    pub nom: String,
    pub arguments: Json,
    /// Le nom que le client se donne — déclaré, jamais vérifié : il sert à poser la question à
    /// l'utilisateur, pas à décider à sa place.
    pub client: String,
    pub moderne: bool,
    /// **Ce que l'appel peut atteindre** (fiche 116) : toute la machine pour un agent d'ici, le
    /// document ouvert pour une IA du nuage.
    pub portee: Portee,
}

/// **Une connexion** : ce qu'il faut retenir d'une ligne à l'autre — rien pour un client
/// moderne, la poignée de main pour un ancien.
#[derive(Debug, Default)]
pub struct Connexion {
    ancien_client: Option<String>,
    /// Le nom vient d'une clé que Glucose a donnée, pas de ce que le client déclare : il ne se
    /// remplace pas (la porte publique, fiche 116).
    verifie: bool,
    portee: Portee,
}

impl Connexion {
    /// **Une connexion de la porte publique** : son nom, vérifié par sa clé, vaut pour toutes ses
    /// requêtes — une poignée de main ancienne comprise —, et sa portée borne ses outils.
    pub fn publique(nom: &str, portee: Portee) -> Self {
        Self {
            ancien_client: Some(nom.to_string()),
            verifie: true,
            portee,
        }
    }

    /// **Ce qu'une ligne demande.**
    pub fn traiter(&mut self, ligne: &str) -> Suite {
        let message = match Json::lire(ligne) {
            Ok(m) => m,
            Err(e) => return Suite::Repondre(erreur(&Json::Nul, ERREUR_DE_LECTURE, &e, None)),
        };
        let (Some(id), Some(methode)) = (
            message.get("id"),
            message.get("method").and_then(Json::as_str),
        ) else {
            return Suite::Rien;
        };
        let params = message.get("params").cloned().unwrap_or(Json::objet([]));
        let meta = params.get("_meta");
        if let Some(version) = meta.and_then(|m| m.get(CLE_VERSION)).and_then(Json::as_str) {
            return self.moderne(id, methode, &params, version);
        }
        if methode == "initialize" {
            return Suite::Repondre(self.poignee_de_main(id, &params));
        }
        let Some(client) = self.ancien_client.clone() else {
            let message = format!(
                "cette requête ne porte pas de version (_meta.{CLE_VERSION}) ; Glucose parle \
                 {VERSION_MODERNE}, ou {} après initialize",
                VERSIONS_ANCIENNES.join(", ")
            );
            return Suite::Repondre(erreur(id, ARGUMENTS_INVALIDES, &message, None));
        };
        self.methode_commune(id, methode, &params, client, false)
    }

    fn moderne(&mut self, id: &Json, methode: &str, params: &Json, version: &str) -> Suite {
        if version != VERSION_MODERNE {
            let mut supportees = vec![Json::from(VERSION_MODERNE)];
            supportees.extend(VERSIONS_ANCIENNES.iter().map(|v| Json::from(*v)));
            let donnees = Json::objet([
                ("supported", Json::Liste(supportees)),
                ("requested", Json::from(version)),
            ]);
            let ligne = erreur(
                id,
                VERSION_INCONNUE,
                "Unsupported protocol version",
                Some(donnees),
            );
            return Suite::Repondre(ligne);
        }
        let declare = params
            .get("_meta")
            .and_then(|m| m.get(CLE_CLIENT))
            .and_then(|c| c.get("name"))
            .and_then(Json::as_str)
            .unwrap_or("un agent sans nom")
            .to_string();
        let client = match (&self.ancien_client, self.verifie) {
            (Some(verifie), true) => verifie.clone(),
            _ => declare,
        };
        if methode == "server/discover" {
            return Suite::Repondre(resultat(id, decouverte(), true));
        }
        self.methode_commune(id, methode, params, client, true)
    }

    fn poignee_de_main(&mut self, id: &Json, params: &Json) -> String {
        let demandee = params.get("protocolVersion").and_then(Json::as_str);
        let version = VERSIONS_ANCIENNES
            .iter()
            .find(|v| Some(**v) == demandee)
            .unwrap_or(&VERSIONS_ANCIENNES[0]);
        let client = params
            .get("clientInfo")
            .and_then(|c| c.get("name"))
            .and_then(Json::as_str)
            .unwrap_or("un agent sans nom");
        if !self.verifie {
            self.ancien_client = Some(client.to_string());
        }
        let r = Json::objet([
            ("protocolVersion", Json::from(*version)),
            ("capabilities", capacites()),
            ("serverInfo", identite()),
            ("instructions", Json::from(outils::INSTRUCTIONS)),
        ]);
        resultat(id, r, false)
    }
}

impl Connexion {
    fn methode_commune(
        &self,
        id: &Json,
        methode: &str,
        params: &Json,
        client: String,
        moderne: bool,
    ) -> Suite {
        let portee = self.portee;
        match methode {
            "ping" => Suite::Repondre(resultat(id, Json::objet([]), moderne)),
            "tools/list" => Suite::Repondre(resultat(id, outils::liste_pour(portee), moderne)),
            "tools/call" => {
                let nom = params
                    .get("name")
                    .and_then(Json::as_str)
                    .unwrap_or_default();
                if !outils::noms().any(|n| n == nom) {
                    let message = format!("Glucose n'a pas d'outil « {nom} »");
                    return Suite::Repondre(erreur(id, ARGUMENTS_INVALIDES, &message, None));
                }
                if !outils::permis(nom, portee) {
                    let message = format!(
                        "« {nom} » n'est pas ouvert à cette porte : {}",
                        portee.dit()
                    );
                    return Suite::Repondre(erreur(id, ARGUMENTS_INVALIDES, &message, None));
                }
                Suite::Executer(Appel {
                    id: id.clone(),
                    nom: nom.to_string(),
                    arguments: params.get("arguments").cloned().unwrap_or(Json::objet([])),
                    client,
                    moderne,
                    portee,
                })
            }
            _ => {
                let message = format!("méthode inconnue : {methode}");
                Suite::Repondre(erreur(id, METHODE_INCONNUE, &message, None))
            }
        }
    }
}

/// **Le statut HTTP d'une réponse** (Streamable HTTP, 28/07/2026) : `400` pour une version
/// inconnue, des en-têtes contraires ou une ligne illisible, `404` pour une méthode inconnue ;
/// `200` pour tout le reste — une erreur d'outil comprise, que le modèle lit.
pub fn statut_http(ligne: &str) -> u16 {
    let code = Json::lire(ligne)
        .ok()
        .and_then(|m| m.get("error")?.get("code")?.as_f64());
    match code {
        Some(c) if c == VERSION_INCONNUE || c == ENTETES_CONTRAIRES || c == ERREUR_DE_LECTURE => {
            400
        }
        Some(c) if c == METHODE_INCONNUE => 404,
        _ => 200,
    }
}

/// La réponse à une requête dont les en-têtes contredisent le corps.
pub fn erreur_d_entetes(id: Option<&Json>, ecart: &str) -> String {
    erreur(id.unwrap_or(&Json::Nul), ENTETES_CONTRAIRES, ecart, None)
}

/// **La réponse à un appel d'outil** : son résultat en texte JSON, ou ce qui l'a empêché — une
/// erreur *d'outil* (`isError`), que le modèle lit et peut corriger, et non une erreur du
/// protocole.
pub fn reponse_d_outil(appel: &Appel, issue: Result<Json, String>) -> String {
    let (texte, echec) = match issue {
        Ok(v) => (v.ecrire(), false),
        Err(e) => (e, true),
    };
    let contenu = Json::objet([("type", Json::from("text")), ("text", Json::from(texte))]);
    let r = Json::objet([
        ("content", Json::Liste(vec![contenu])),
        ("isError", Json::Bool(echec)),
    ]);
    resultat(&appel.id, r, appel.moderne)
}

fn identite() -> Json {
    Json::objet([
        ("name", Json::from("glucose")),
        ("version", Json::from(env!("CARGO_PKG_VERSION"))),
    ])
}

fn capacites() -> Json {
    Json::objet([("tools", Json::objet([]))])
}

fn decouverte() -> Json {
    let mut versions = vec![Json::from(VERSION_MODERNE)];
    versions.extend(VERSIONS_ANCIENNES.iter().map(|v| Json::from(*v)));
    Json::objet([
        ("supportedVersions", Json::Liste(versions)),
        ("capabilities", capacites()),
        (
            "_meta",
            Json::objet([("io.modelcontextprotocol/serverInfo", identite())]),
        ),
        ("instructions", Json::from(outils::INSTRUCTIONS)),
    ])
}

/// Un résultat ; une réponse moderne dit qu'il est complet (`resultType`), ce que la version
/// du 28/07/2026 demande à tout résultat qui n'attend rien d'autre du client.
fn resultat(id: &Json, mut r: Json, moderne: bool) -> String {
    if let (true, Json::Objet(o)) = (moderne, &mut r) {
        o.insert(0, ("resultType".to_string(), Json::from("complete")));
    }
    Json::objet([
        ("jsonrpc", Json::from("2.0")),
        ("id", id.clone()),
        ("result", r),
    ])
    .ecrire()
}

fn erreur(id: &Json, code: f64, message: &str, donnees: Option<Json>) -> String {
    let mut e = vec![
        ("code", Json::Nombre(code)),
        ("message", Json::from(message)),
    ];
    if let Some(d) = donnees {
        e.push(("data", d));
    }
    Json::objet([
        ("jsonrpc", Json::from("2.0")),
        ("id", id.clone()),
        ("error", Json::objet(e)),
    ])
    .ecrire()
}

#[cfg(test)]
mod tests;
