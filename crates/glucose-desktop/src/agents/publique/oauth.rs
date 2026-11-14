//! **Le guichet de la porte** : OAuth 2.1, tel que la spécification MCP le demande (fiche 116) —
//! chaque Glucose est **son propre serveur d'autorisation** ; le relais ne fait que passer.
//!
//! # Le chemin d'un accord
//!
//! 1. L'IA ouvre la page d'accord du relais ; la page envoie ses paramètres ici
//!    ([`demander`]) : qui demande — vérifié par son document d'identité (CIMD) ou par son
//!    inscription (DCR) —, et le défi PKCE. Le guichet tire un **code à trois chiffres**, que la
//!    page affiche et que Glucose montre dans sa question.
//! 2. Dans Glucose, l'utilisateur répond : *Lire et écrire*, *Lire seulement*, *Refuser*.
//! 3. La page, qui attend ([`attendre`]), reçoit l'adresse de retour : un code d'autorisation à
//!    usage unique, ou le refus.
//! 4. L'IA échange le code contre une clé d'accès ([`echanger`]), en prouvant qu'elle tient le
//!    secret dont le défi était l'empreinte (PKCE, S256). La clé ne se garde ici que par son
//!    empreinte, et vaut **jusqu'à ce que l'utilisateur la retire** (B1) : ni expiration, ni
//!    clé de rafraîchissement — un Glucose fermé ne pourrait pas en rafraîchir une.
//!
//! Ce module est pur : le temps et le document d'identité lui sont donnés.

use super::accords::{empreinte, Accord, Droits, Etat};
use super::Adresses;
use crate::agents::json::Json;
use glucose_core::encodage::{base64, base64_adresse};
use glucose_core::hash::sha256;
use std::time::{Duration, Instant};

/// La vie d'une demande d'accès et d'un code d'autorisation : dix minutes au plus (RFC 6749
/// § 4.1.2).
pub const DUREE: Duration = Duration::from_secs(600);
/// Les demandes qui attendent en même temps ; au-delà, la plus vieille cède sa place — une
/// page qu'on recharge sans fin ne remplit rien.
const ATTENTES_MAX: usize = 8;
/// Le nom qu'un client se donne, au plus : il s'affiche dans une question.
const NOM_MAX: usize = 60;

/// **Qui demande**, vérifié autant qu'on le peut.
#[derive(Debug, Clone, PartialEq)]
pub struct Client {
    pub id: String,
    pub nom: String,
    /// Le domaine d'où il vient : celui de son document d'identité, ou de son adresse de retour.
    pub hote: String,
    /// L'adresse de retour de cette demande.
    pub retour: String,
}

/// Ce que l'utilisateur a répondu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    Attente,
    Accordee(Droits),
    Refusee,
}

/// **Une demande d'accès** qui attend sa réponse dans Glucose.
#[derive(Debug, Clone)]
pub struct Autorisation {
    pub demande: String,
    pub code_affiche: String,
    pub client: Client,
    defi: String,
    etat_client: Option<String>,
    /// La question est posée dans Glucose (elle ne l'est qu'une fois).
    pub posee: bool,
    pub issue: Issue,
    nee: Instant,
}

/// Un code d'autorisation donné, que l'IA n'a pas encore échangé.
struct Code {
    empreinte: String,
    client: Client,
    droits: Droits,
    defi: String,
    ne: Instant,
}

/// **Le guichet** : l'état gardé, et ce qui est en route.
pub struct Guichet {
    pub etat: Etat,
    pub autorisations: Vec<Autorisation>,
    codes: Vec<Code>,
}

/// **Un refus**, à la forme d'OAuth (RFC 6749 § 5.2).
#[derive(Debug, Clone, PartialEq)]
pub struct Refus {
    pub statut: u16,
    pub code: &'static str,
    pub description: String,
}

fn refus(statut: u16, code: &'static str, description: impl Into<String>) -> Refus {
    Refus {
        statut,
        code,
        description: description.into(),
    }
}

impl Guichet {
    pub fn nouveau(etat: Etat) -> Self {
        Self {
            etat,
            autorisations: Vec::new(),
            codes: Vec::new(),
        }
    }

    fn oublier_le_perime(&mut self, maintenant: Instant) {
        self.autorisations
            .retain(|a| maintenant.duration_since(a.nee) < DUREE);
        self.codes
            .retain(|c| maintenant.duration_since(c.ne) < DUREE);
    }
}

/// Le domaine d'une adresse `http(s)://domaine[:port]/…`.
pub fn hote_de(adresse: &str) -> Option<String> {
    let reste = adresse
        .strip_prefix("https://")
        .or_else(|| adresse.strip_prefix("http://"))?;
    let autorite = reste.split(['/', '?', '#']).next()?;
    let hote = match autorite.strip_prefix('[') {
        Some(v6) => format!("[{}]", v6.split(']').next()?),
        None => autorite.split(':').next()?.to_string(),
    };
    (!hote.is_empty()).then(|| hote.to_ascii_lowercase())
}

/// Une adresse de retour sur cet ordinateur (RFC 8252 § 7.3) : `http`, un hôte de boucle.
fn boucle_locale(adresse: &str) -> bool {
    adresse.starts_with("http://")
        && matches!(
            hote_de(adresse).as_deref(),
            Some("localhost" | "127.0.0.1" | "[::1]")
        )
}

/// L'adresse sans son port : deux retours de boucle locale se comparent ainsi.
fn sans_port(adresse: &str) -> String {
    let Some(reste) = adresse.strip_prefix("http://") else {
        return adresse.to_string();
    };
    let (autorite, chemin) = reste.split_at(reste.find('/').unwrap_or(reste.len()));
    let hote = hote_de(adresse).unwrap_or_else(|| autorite.to_string());
    format!("http://{hote}{chemin}")
}

/// **Le retour demandé est-il celui que le client a déclaré ?** Exactement — ou, sur cet
/// ordinateur, au port près (Claude Code tire le sien à chaque session).
fn retour_permis(declare: &str, demande: &str) -> bool {
    declare == demande
        || (boucle_locale(declare)
            && boucle_locale(demande)
            && sans_port(declare) == sans_port(demande))
}

fn nom_borne(nom: &str) -> String {
    nom.chars()
        .filter(|c| !c.is_control())
        .take(NOM_MAX)
        .collect::<String>()
        .trim()
        .to_string()
}

/// **Qui demande** : son identité, vérifiée par son document (CIMD) ou par l'inscription que ce
/// guichet lui a donnée (DCR). `chercher` rend le document d'une adresse.
pub fn client_de(
    params: &Json,
    chercher: &dyn Fn(&str) -> Result<String, String>,
) -> Result<Client, Refus> {
    let texte = |c: &str| params.get(c).and_then(Json::as_str);
    let (Some(id), Some(retour)) = (texte("client_id"), texte("redirect_uri")) else {
        return Err(refus(
            400,
            "invalid_request",
            "client_id et redirect_uri sont requis",
        ));
    };
    let (nom, hote, retours) = if id.starts_with("https://") {
        // La raison ne repart pas : l'adresse vient du client, et ce que Glucose a vu en la
        // suivant ne le regarde pas.
        let document = chercher(id)
            .map_err(|_| refus(400, "invalid_client", "document d'identité introuvable"))?;
        let d = Json::lire(&document)
            .map_err(|_| refus(400, "invalid_client", "document d'identité illisible"))?;
        if d.get("client_id").and_then(Json::as_str) != Some(id) {
            return Err(refus(
                400,
                "invalid_client",
                "le document ne dit pas ce client_id",
            ));
        }
        let hote =
            hote_de(id).ok_or_else(|| refus(400, "invalid_client", "client_id sans domaine"))?;
        let nom = d
            .get("client_name")
            .and_then(Json::as_str)
            .unwrap_or(&hote)
            .to_string();
        (nom, hote, textes(d.get("redirect_uris")))
    } else if let Some(charge) = id.strip_prefix("dcr:") {
        let inscrit = base64(charge)
            .and_then(|o| String::from_utf8(o).ok())
            .and_then(|t| Json::lire(&t).ok())
            .ok_or_else(|| refus(400, "invalid_client", "inscription illisible"))?;
        let nom = inscrit
            .get("nom")
            .and_then(Json::as_str)
            .unwrap_or("une IA")
            .to_string();
        let hote = hote_de(retour).unwrap_or_default();
        (nom, hote, textes(inscrit.get("retours")))
    } else {
        return Err(refus(
            400,
            "invalid_client",
            "client inconnu : ni document d'identité, ni inscription",
        ));
    };
    if !retours.iter().any(|r| retour_permis(r, retour)) {
        return Err(refus(
            400,
            "invalid_request",
            "adresse de retour non déclarée par ce client",
        ));
    }
    // Un document d'identité ne renvoie que chez lui (ou sur la machine de l'utilisateur).
    if id.starts_with("https://")
        && !boucle_locale(retour)
        && hote_de(retour).as_deref() != Some(&hote)
    {
        return Err(refus(
            400,
            "invalid_request",
            "adresse de retour hors du domaine du client",
        ));
    }
    Ok(Client {
        id: id.to_string(),
        nom: nom_borne(&nom),
        hote,
        retour: retour.to_string(),
    })
}

fn textes(j: Option<&Json>) -> Vec<String> {
    j.and_then(Json::as_liste)
        .unwrap_or_default()
        .iter()
        .filter_map(|r| r.as_str().map(str::to_string))
        .collect()
}

/// **L'inscription d'un client** (RFC 7591) : rien ne se garde — son identifiant **est** son
/// inscription, son nom et ses adresses de retour, écrits dedans. Un client qui s'inscrit mille
/// fois ne remplit rien.
pub fn inscrire(corps: &Json) -> Result<Json, Refus> {
    let retours = textes(corps.get("redirect_uris"));
    let valables = retours
        .iter()
        .all(|r| (r.starts_with("https://") && hote_de(r).is_some()) || boucle_locale(r));
    if retours.is_empty() || !valables {
        return Err(refus(
            400,
            "invalid_redirect_uri",
            "des adresses de retour https://, ou de cet ordinateur",
        ));
    }
    let nom = nom_borne(
        corps
            .get("client_name")
            .and_then(Json::as_str)
            .unwrap_or("une IA"),
    );
    let liste = || Json::Liste(retours.iter().map(|r| Json::from(r.as_str())).collect());
    let inscrit = Json::objet([("nom", Json::from(nom.as_str())), ("retours", liste())]);
    let id = format!("dcr:{}", base64_adresse(inscrit.ecrire().as_bytes()));
    if id.len() > 4096 {
        return Err(refus(
            400,
            "invalid_client_metadata",
            "inscription trop longue",
        ));
    }
    Ok(Json::objet([
        ("client_id", Json::from(id)),
        ("client_name", Json::from(nom)),
        ("redirect_uris", liste()),
        ("token_endpoint_auth_method", Json::from("none")),
        (
            "grant_types",
            Json::Liste(vec![Json::from("authorization_code")]),
        ),
        ("response_types", Json::Liste(vec![Json::from("code")])),
    ]))
}

/// Un texte du hasard, sûr pour une adresse.
fn au_hasard<const N: usize>() -> Result<String, Refus> {
    crate::plateforme::hasard::octets::<N>()
        .map(|o| base64_adresse(&o))
        .map_err(|e| refus(500, "server_error", e))
}

/// **Une demande d'accès arrive** de la page : elle attendra la réponse de l'utilisateur.
pub fn demander(
    g: &mut Guichet,
    params: &Json,
    client: Client,
    adresses: &Adresses,
    maintenant: Instant,
) -> Result<Json, Refus> {
    let texte = |c: &str| params.get(c).and_then(Json::as_str);
    if texte("response_type") != Some("code") {
        return Err(refus(
            400,
            "unsupported_response_type",
            "seul response_type=code",
        ));
    }
    let defi = texte("code_challenge").unwrap_or_default();
    let defi_valable = (43..=128).contains(&defi.len())
        && defi
            .bytes()
            .all(|o| o.is_ascii_alphanumeric() || o == b'-' || o == b'_');
    if texte("code_challenge_method") != Some("S256") || !defi_valable {
        return Err(refus(
            400,
            "invalid_request",
            "un défi PKCE S256 est requis",
        ));
    }
    if texte("resource").is_some_and(|r| r != adresses.mcp()) {
        return Err(refus(
            400,
            "invalid_target",
            "cette ressource n'est pas ce Glucose",
        ));
    }
    g.oublier_le_perime(maintenant);
    if g.autorisations.len() >= ATTENTES_MAX {
        g.autorisations.remove(0);
    }
    let tirage =
        crate::plateforme::hasard::octets::<2>().map_err(|e| refus(500, "server_error", e))?;
    let code_affiche = format!("{:03}", u16::from_be_bytes(tirage) % 1000);
    let demande = au_hasard::<16>()?;
    let reponse = Json::objet([
        ("demande", Json::from(demande.as_str())),
        ("code", Json::from(code_affiche.as_str())),
        (
            "client",
            Json::objet([
                ("nom", Json::from(client.nom.as_str())),
                ("hote", Json::from(client.hote.as_str())),
            ]),
        ),
    ]);
    g.autorisations.push(Autorisation {
        demande,
        code_affiche,
        client,
        defi: defi.to_string(),
        etat_client: texte("state").map(str::to_string),
        posee: false,
        issue: Issue::Attente,
        nee: maintenant,
    });
    Ok(reponse)
}

/// L'adresse de retour, ses paramètres ajoutés (RFC 9207 : `iss` dit quel serveur répond).
fn retour_avec(
    client: &Client,
    etat: Option<&str>,
    paires: &[(&str, &str)],
    adresses: &Adresses,
) -> String {
    let mut adresse = client.retour.clone();
    let mut sep = if adresse.contains('?') { '&' } else { '?' };
    let iss = adresses.base();
    let tout = paires
        .iter()
        .copied()
        .chain(etat.map(|e| ("state", e)))
        .chain([("iss", iss.as_str())]);
    for (cle, valeur) in tout {
        adresse.push(sep);
        adresse.push_str(&format!("{cle}={}", super::http::pourcent(valeur)));
        sep = '&';
    }
    adresse
}

/// **La page demande où en est sa demande.** Accordée : un code d'autorisation, à usage unique.
pub fn attendre(
    g: &mut Guichet,
    params: &Json,
    adresses: &Adresses,
    maintenant: Instant,
) -> Result<Json, Refus> {
    g.oublier_le_perime(maintenant);
    let demande = params
        .get("demande")
        .and_then(Json::as_str)
        .unwrap_or_default();
    let Some(i) = g.autorisations.iter().position(|a| a.demande == demande) else {
        return Err(refus(
            404,
            "demande_inconnue",
            "cette demande n'existe plus : recommence depuis ton IA",
        ));
    };
    let droits = match g.autorisations[i].issue {
        Issue::Attente => return Ok(Json::objet([("etat", Json::from("attente"))])),
        Issue::Refusee => None,
        Issue::Accordee(d) => Some(d),
    };
    let a = g.autorisations.remove(i);
    let etat_client = a.etat_client.as_deref();
    let Some(droits) = droits else {
        let r = retour_avec(
            &a.client,
            etat_client,
            &[("error", "access_denied")],
            adresses,
        );
        return Ok(Json::objet([
            ("etat", Json::from("refuse")),
            ("redirection", Json::from(r)),
        ]));
    };
    let code = au_hasard::<32>()?;
    let r = retour_avec(&a.client, etat_client, &[("code", &code)], adresses);
    g.codes.push(Code {
        empreinte: empreinte(&code),
        client: a.client,
        droits,
        defi: a.defi,
        ne: maintenant,
    });
    Ok(Json::objet([
        ("etat", Json::from("accorde")),
        ("redirection", Json::from(r)),
    ]))
}

/// **L'IA échange son code contre une clé d'accès** (RFC 6749 § 4.1.3, RFC 7636 § 4.6). Rend la
/// réponse et l'accord neuf, déjà inscrit dans l'état — à enregistrer.
pub fn echanger(
    g: &mut Guichet,
    form: &[(String, String)],
    adresses: &Adresses,
    maintenant: Instant,
    depuis: u64,
) -> Result<(Json, Accord), Refus> {
    let champ = |c: &str| form.iter().find(|(k, _)| k == c).map(|(_, v)| v.as_str());
    if champ("grant_type") != Some("authorization_code") {
        return Err(refus(
            400,
            "unsupported_grant_type",
            "seul authorization_code",
        ));
    }
    g.oublier_le_perime(maintenant);
    let voulu = empreinte(champ("code").unwrap_or_default());
    // À usage unique : retiré avant toute vérification, il ne se rejoue pas.
    let Some(i) = g.codes.iter().position(|c| c.empreinte == voulu) else {
        return Err(refus(
            400,
            "invalid_grant",
            "code inconnu, déjà servi ou périmé",
        ));
    };
    let c = g.codes.remove(i);
    let preuve = champ("code_verifier").map(|v| base64_adresse(&sha256(v.as_bytes())));
    let bon = champ("client_id") == Some(c.client.id.as_str())
        && champ("redirect_uri") == Some(c.client.retour.as_str())
        && preuve.as_deref() == Some(c.defi.as_str())
        && champ("resource").is_none_or(|r| r == adresses.mcp());
    if !bon {
        return Err(refus(
            400,
            "invalid_grant",
            "le code ne va pas avec ce client, ce retour ou cette preuve",
        ));
    }
    let cle = au_hasard::<32>()?;
    let accord = Accord {
        client_id: c.client.id,
        nom: c.client.nom,
        hote: c.client.hote,
        droits: c.droits,
        empreinte: empreinte(&cle),
        depuis,
    };
    g.etat.accorder(accord.clone());
    let reponse = Json::objet([
        ("access_token", Json::from(cle)),
        ("token_type", Json::from("Bearer")),
    ]);
    Ok((reponse, accord))
}

/// **Ce que la ressource dit d'elle** (RFC 9728) : l'adresse MCP de ce Glucose, et son guichet.
pub fn decouverte_de_la_ressource(adresses: &Adresses) -> Json {
    Json::objet([
        ("resource", Json::from(adresses.mcp())),
        (
            "authorization_servers",
            Json::Liste(vec![Json::from(adresses.base())]),
        ),
        (
            "bearer_methods_supported",
            Json::Liste(vec![Json::from("header")]),
        ),
        ("resource_name", Json::from("Glucose")),
    ])
}

/// **Ce que le guichet dit de lui** (RFC 8414) : ce qu'il faut à Claude pour choisir CIMD — le
/// document d'identité, et `none` à l'échange (un client public, qui prouve par PKCE).
pub fn decouverte_du_guichet(adresses: &Adresses) -> Json {
    let base = adresses.base();
    let liste = |v: &[&str]| Json::Liste(v.iter().map(|s| Json::from(*s)).collect());
    Json::objet([
        ("issuer", Json::from(base.as_str())),
        (
            "authorization_endpoint",
            Json::from(format!("{base}/authorize")),
        ),
        ("token_endpoint", Json::from(format!("{base}/token"))),
        (
            "registration_endpoint",
            Json::from(format!("{base}/register")),
        ),
        ("response_types_supported", liste(&["code"])),
        ("grant_types_supported", liste(&["authorization_code"])),
        ("code_challenge_methods_supported", liste(&["S256"])),
        ("token_endpoint_auth_methods_supported", liste(&["none"])),
        ("client_id_metadata_document_supported", Json::Bool(true)),
        (
            "authorization_response_iss_parameter_supported",
            Json::Bool(true),
        ),
    ])
}
