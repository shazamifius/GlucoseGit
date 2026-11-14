//! **Ce qu'une demande de l'IA devient** : une requête HTTP que le relais a passée, une réponse
//! qu'il rend — le guichet d'OAuth ([`super::oauth`]), et la porte MCP selon la spécification du
//! 28/07/2026 (« Streamable HTTP » : un `POST` par message, une réponse JSON).
//!
//! Le transport est sans état : chaque requête porte sa clé, d'où vient le nom **vérifié** de
//! l'IA, et sa portée. Un client de l'époque ancienne (une poignée de main `initialize`) est servi
//! aussi, sans session : sa poignée de main se répond, puis chaque requête se suffit.

use super::accords::{self, Accord};
use super::oauth::{self, Guichet, Refus};
use super::Adresses;
use crate::agents::json::Json;
use crate::agents::outils::Portee;
use crate::agents::protocole::{self, Connexion, Suite};
use crate::agents::{Boite, Demande};
use std::path::Path;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

/// Le plus long qu'un appel d'outil attend la boucle de Glucose — sous l'attente du relais.
const ATTENTE_DE_LA_BOUCLE: Duration = Duration::from_secs(50);

/// **Une requête**, telle que le relais la passe.
#[derive(Debug, Clone, PartialEq)]
pub struct Requete {
    pub n: f64,
    pub methode: String,
    pub chemin: String,
    /// Les en-têtes, leurs noms en minuscules.
    pub entetes: Vec<(String, String)>,
    pub corps: String,
}

/// **Une réponse**, telle que le relais la rendra.
#[derive(Debug, Clone, PartialEq)]
pub struct Reponse {
    pub statut: u16,
    pub entetes: Vec<(&'static str, String)>,
    pub corps: String,
}

impl Requete {
    pub fn lire(texte: &str) -> Option<Requete> {
        let j = Json::lire(texte).ok()?;
        let entetes = match j.get("entetes") {
            Some(Json::Objet(paires)) => paires
                .iter()
                .filter_map(|(k, v)| Some((k.to_ascii_lowercase(), v.as_str()?.to_string())))
                .collect(),
            _ => Vec::new(),
        };
        Some(Requete {
            n: j.get("n")?.as_f64()?,
            methode: j.get("methode")?.as_str()?.to_string(),
            chemin: j.get("chemin")?.as_str()?.to_string(),
            entetes,
            corps: j
                .get("corps")
                .and_then(Json::as_str)
                .unwrap_or_default()
                .to_string(),
        })
    }

    fn entete(&self, nom: &str) -> Option<&str> {
        self.entetes
            .iter()
            .find(|(k, _)| k == nom)
            .map(|(_, v)| v.as_str())
    }
}

impl Reponse {
    fn json(statut: u16, corps: String) -> Reponse {
        Reponse {
            statut,
            entetes: vec![("content-type", "application/json".into())],
            corps,
        }
    }

    fn vide(statut: u16) -> Reponse {
        Reponse {
            statut,
            entetes: Vec::new(),
            corps: String::new(),
        }
    }

    /// Ce qui repart au relais, sous le numéro de la requête.
    pub fn ecrire(&self, n: f64) -> String {
        let entetes = self
            .entetes
            .iter()
            .map(|(k, v)| (*k, Json::from(v.as_str())));
        Json::objet([
            ("n", Json::Nombre(n)),
            ("statut", Json::Nombre(f64::from(self.statut))),
            ("entetes", Json::objet(entetes)),
            ("corps", Json::from(self.corps.as_str())),
        ])
        .ecrire()
    }
}

impl From<Refus> for Reponse {
    fn from(r: Refus) -> Reponse {
        let corps = Json::objet([
            ("error", Json::from(r.code)),
            ("error_description", Json::from(r.description)),
        ]);
        Reponse::json(r.statut, corps.ecrire())
    }
}

/// **Ce dont une requête a besoin** : les adresses de ce Glucose, son guichet, le fichier où il
/// garde ses accords, la boîte aux lettres de la boucle, et de quoi lire un document d'identité.
pub struct Contexte<'a> {
    pub adresses: &'a Adresses,
    pub guichet: &'a Mutex<Guichet>,
    pub fichier: &'a Path,
    pub boite: &'a Boite,
    pub chercher: &'a dyn Fn(&str) -> Result<String, String>,
    /// Les secondes depuis 1970 : la date d'un accord.
    pub maintenant: u64,
}

/// **Répond à une requête.**
pub fn traiter(r: &Requete, c: &Contexte<'_>) -> Reponse {
    let guichet = || c.guichet.lock().unwrap_or_else(PoisonError::into_inner);
    let corps_json = || Json::lire(&r.corps).unwrap_or(Json::objet([]));
    let issue = match (r.methode.as_str(), r.chemin.as_str()) {
        ("GET", "/.well-known/oauth-protected-resource") => {
            Ok((200, oauth::decouverte_de_la_ressource(c.adresses)))
        }
        ("GET", "/.well-known/oauth-authorization-server") => {
            Ok((200, oauth::decouverte_du_guichet(c.adresses)))
        }
        ("POST", "/register") => oauth::inscrire(&corps_json()).map(|j| (201, j)),
        ("POST", "/authorize/demande") => {
            let params = corps_json();
            // Le document d'identité se lit hors du guichet : le réseau n'y retient personne.
            oauth::client_de(&params, c.chercher).and_then(|client| {
                let j =
                    oauth::demander(&mut guichet(), &params, client, c.adresses, Instant::now());
                // La boucle pose la question dès son réveil.
                c.boite.reveiller();
                j.map(|j| (200, j))
            })
        }
        ("POST", "/authorize/attente") => {
            oauth::attendre(&mut guichet(), &corps_json(), c.adresses, Instant::now())
                .map(|j| (200, j))
        }
        ("POST", "/token") => echanger(r, c).map(|j| (200, j)),
        ("POST", "/mcp") => return mcp(r, c),
        (_, "/mcp") => {
            let mut rep = Reponse::vide(405);
            rep.entetes.push(("allow", "POST".into()));
            return rep;
        }
        _ => Err(Refus {
            statut: 404,
            code: "introuvable",
            description: format!("rien à {} {}", r.methode, r.chemin),
        }),
    };
    match issue {
        Ok((statut, j)) => Reponse::json(statut, j.ecrire()),
        Err(refus) => refus.into(),
    }
}

/// L'échange d'un code contre une clé — et l'accord neuf, enregistré tout de suite : une IA qui a
/// sa clé l'a pour de bon.
fn echanger(r: &Requete, c: &Contexte<'_>) -> Result<Json, Refus> {
    let form = formulaire(&r.corps);
    let mut g = c.guichet.lock().unwrap_or_else(PoisonError::into_inner);
    let (reponse, _) = oauth::echanger(&mut g, &form, c.adresses, Instant::now(), c.maintenant)?;
    accords::enregistrer(c.fichier, &g.etat).map_err(|e| Refus {
        statut: 500,
        code: "server_error",
        description: format!("l'accord ne s'enregistre pas : {e}"),
    })?;
    // La boucle redessine sa liste des IA qui ont la porte.
    c.boite.reveiller();
    Ok(reponse)
}

/// **La porte MCP** : la clé d'abord (sinon `401`, qui lance l'autorisation chez l'IA), puis le
/// message, puis l'outil — par la boîte aux lettres, comme pour un agent de cette machine.
fn mcp(r: &Requete, c: &Contexte<'_>) -> Reponse {
    let cle = r.entete("authorization").and_then(|v| {
        v.strip_prefix("Bearer ")
            .or_else(|| v.strip_prefix("bearer "))
    });
    let accord: Option<Accord> = cle.and_then(|k| {
        let g = c.guichet.lock().unwrap_or_else(PoisonError::into_inner);
        g.etat.accord_de(k).cloned()
    });
    let Some(accord) = accord else {
        let mut rep: Reponse = Refus {
            statut: 401,
            code: "invalid_token",
            description: "une clé de ce Glucose est requise".into(),
        }
        .into();
        let defi = format!("Bearer resource_metadata=\"{}\"", c.adresses.metadonnees());
        rep.entetes.push(("www-authenticate", defi));
        return rep;
    };
    if r.entete("origin")
        .is_some_and(|o| !o.starts_with("https://"))
    {
        return Reponse::vide(403);
    }
    let Ok(message) = Json::lire(&r.corps) else {
        let ligne = Connexion::default().traiter(&r.corps);
        return reponse_de(ligne);
    };
    if message.get("id").is_none() {
        return Reponse::vide(202);
    }
    if let Some(ecart) = ecart_d_entetes(r, &message) {
        return Reponse::json(400, protocole::erreur_d_entetes(message.get("id"), &ecart));
    }
    let portee = Portee::publique(accord.droits == accords::Droits::Ecrire);
    let mut connexion = Connexion::publique(&accord.signature(), portee);
    match connexion.traiter(&r.corps) {
        Suite::Executer(appel) => {
            let (envoi, attente) = std::sync::mpsc::channel();
            c.boite.deposer(Demande {
                appel,
                reponse: envoi,
            });
            match attente.recv_timeout(ATTENTE_DE_LA_BOUCLE) {
                Ok(ligne) => Reponse::json(200, ligne),
                Err(_) => Reponse::json(503, "{}".into()),
            }
        }
        autre => reponse_de(autre),
    }
}

fn reponse_de(suite: Suite) -> Reponse {
    match suite {
        Suite::Repondre(ligne) => Reponse::json(protocole::statut_http(&ligne), ligne),
        Suite::Rien | Suite::Executer(_) => Reponse::vide(202),
    }
}

/// **Les en-têtes contre le corps** (« Server Validation ») : une requête moderne doit dire dans
/// ses en-têtes sa version, sa méthode et, pour un outil, son nom — les mêmes que dans son corps.
/// Rend ce qui ne va pas.
fn ecart_d_entetes(r: &Requete, message: &Json) -> Option<String> {
    let version = message
        .get("params")
        .and_then(|p| p.get("_meta"))
        .and_then(|m| m.get(protocole::CLE_VERSION))
        .and_then(Json::as_str)?;
    let methode = message
        .get("method")
        .and_then(Json::as_str)
        .unwrap_or_default();
    let mut attendus = vec![("mcp-protocol-version", version), ("mcp-method", methode)];
    let nom = message
        .get("params")
        .and_then(|p| p.get("name").or_else(|| p.get("uri")))
        .and_then(Json::as_str);
    if let (true, Some(nom)) = (
        matches!(methode, "tools/call" | "resources/read" | "prompts/get"),
        nom,
    ) {
        attendus.push(("mcp-name", nom));
    }
    attendus.into_iter().find_map(|(entete, corps)| {
        let lu = r.entete(entete).map(decoder_l_entete);
        (lu.as_deref() != Some(corps)).then(|| {
            format!(
                "l'en-tête {entete} ({}) ne dit pas ce que dit le corps ({corps})",
                lu.unwrap_or_default()
            )
        })
    })
}

/// Une valeur d'en-tête, décodée si elle porte la sentinelle `=?base64?…?=`.
fn decoder_l_entete(v: &str) -> String {
    v.strip_prefix("=?base64?")
        .and_then(|r| r.strip_suffix("?="))
        .and_then(glucose_core::encodage::base64)
        .and_then(|o| String::from_utf8(o).ok())
        .unwrap_or_else(|| v.to_string())
}

/// **Un formulaire** `application/x-www-form-urlencoded` (RFC 6749 § 4.1.3).
pub fn formulaire(corps: &str) -> Vec<(String, String)> {
    corps
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (depourcent(k), depourcent(v))
        })
        .collect()
}

fn depourcent(s: &str) -> String {
    let o = s.as_bytes();
    let mut out = Vec::with_capacity(o.len());
    let mut i = 0;
    while i < o.len() {
        match o[i] {
            b'+' => out.push(b' '),
            b'%' => match o
                .get(i + 1..i + 3)
                .and_then(|h| std::str::from_utf8(h).ok())
                .and_then(|h| u8::from_str_radix(h, 16).ok())
            {
                Some(v) => {
                    out.push(v);
                    i += 2;
                }
                None => out.push(b'%'),
            },
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// **Écrit une valeur pour une adresse** (RFC 3986 § 2.3) : les signes non réservés passent, tout
/// le reste en `%XX`.
pub fn pourcent(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for o in s.bytes() {
        if o.is_ascii_alphanumeric() || matches!(o, b'-' | b'.' | b'_' | b'~') {
            out.push(o as char);
        } else {
            out.push_str(&format!("%{o:02X}"));
        }
    }
    out
}
