//! **Les commandes de Glucose au terminal** (fiche 90) — sa demande du 7/11/2026 : *« des
//! commandes Linux directement pour faire des trucs avec Glucose, plein plein de commandes, dont
//! l'ouvrir »*.
//!
//! ```text
//! glucose                               ouvre Glucose, sur ce qu'il reprend d'habitude
//! glucose recherche/eau.glucose         ouvre ce document
//! glucose aide [commande]               ce que fait chaque commande
//! glucose lire                          le sommaire du tableau qu'on regarde
//! glucose chercher eau chaude           ce que Ctrl+F trouve
//! glucose domaines domaine=Eau          les nœuds d'un domaine, dans tous les fichiers
//! cat notes.md | glucose poser-texte -  un texte de plus, sous ses yeux
//! ```
//!
//! # TERMINAL-1 — une seule table, deux portes
//!
//! Ce qu'une IA fait par MCP, un humain le fait au terminal, et l'inverse : **chaque commande est
//! un outil** de la porte des agents ([`super::outils`]) — son nom sans `glucose_`, ses arguments
//! lus par le schéma de l'outil ([`arguments`]), son aide tirée de sa description. Rien n'est écrit
//! deux fois : un outil neuf a sa commande le jour où il naît, et une commande ne peut rien qu'un
//! agent ne puisse.
//!
//! # TERMINAL-2 — la même porte, le même accord
//!
//! Une commande parle au Glucose ouvert par sa porte — une seule ligne du protocole moderne, sans
//! poignée de main — et l'ouvre s'il ne l'est pas, comme `glucose --mcp`. Elle se présente sous le
//! nom [`CLIENT`] et passe par l'accord de l'utilisateur (AGENTS-1) : un agent sait taper une
//! commande, et le terminal ne doit pas être une porte dérobée. Chaque geste s'annule par `Ctrl+Z`.
//!
//! # La sortie
//!
//! Ce que l'outil rend, en JSON — ce que lit une IA est ce que lit l'humain —, en retrait dans un
//! terminal, sur une ligne dans un tuyau (pour `jq`). Le code de sortie dit l'issue : 0 ; 1 si
//! Glucose a refusé ou n'a pas répondu, sa phrase sur la sortie d'erreur ; 2 si la commande est mal
//! écrite. (L'interface d'Obsidian, sortie en 2026, rend 0 même sur une erreur : on ne peut alors
//! rien enchaîner sans relire sa sortie.)

mod aide;
mod arguments;

pub use aide::aide;

use super::json::Json;
use super::{outils, porte, protocole, relais};
use std::io::{BufReader, IsTerminal, Read, Write};
use std::path::PathBuf;

/// Le nom sous lequel une commande se présente : celui que la question d'accord montre.
pub const CLIENT: &str = "le terminal";

/// Les mots qui demandent l'aide.
const AIDE: [&str; 4] = ["aide", "help", "--help", "-h"];

/// **Ce que la ligne de commande demande.**
#[derive(Debug, PartialEq)]
pub enum Demande {
    /// Ouvrir Glucose : sur ce qu'il reprend, ou sur ce document.
    Ouvrir(Option<PathBuf>),
    /// L'aide : de toutes les commandes, ou d'une.
    Aide(Option<String>),
    /// Un outil, et les mots qui suivent sa commande.
    Outil(&'static str, Vec<String>),
    /// Une option que `main` connaît (`--mcp`, `--version`…) : rien pour ce module.
    Ailleurs,
}

/// **La commande d'un outil** : son nom sans `glucose_`, les mots liés par des tirets.
pub fn commande_de(outil: &str) -> String {
    outil.trim_start_matches("glucose_").replace('_', "-")
}

/// L'outil d'une commande, s'il existe.
pub fn outil_de(commande: &str) -> Option<&'static str> {
    outils::noms().find(|o| commande_de(o) == commande)
}

/// **Lit la ligne de commande** — les mots qui suivent le programme. Un mot qui n'est ni une
/// commande ni un fichier est refusé : ouvrir Glucose sur une faute de frappe cacherait la faute.
pub fn lire(mots: &[String]) -> Result<Demande, String> {
    let Some(premier) = mots.first() else {
        return Ok(Demande::Ouvrir(None));
    };
    if AIDE.contains(&premier.as_str()) {
        return Ok(Demande::Aide(mots.get(1).cloned()));
    }
    if premier.starts_with("--") {
        return Ok(Demande::Ailleurs);
    }
    if let Some(outil) = outil_de(premier) {
        return Ok(Demande::Outil(outil, mots[1..].to_vec()));
    }
    if mots.len() > 1 {
        return Err(format!(
            "« {premier} » n'est pas une commande ; glucose aide les dit toutes"
        ));
    }
    let chemin = PathBuf::from(premier);
    if chemin.is_file() {
        return Ok(Demande::Ouvrir(Some(chemin)));
    }
    let un_chemin = chemin.extension().is_some() || chemin.components().count() > 1;
    Err(if un_chemin {
        format!("« {premier} » est introuvable")
    } else {
        format!("« {premier} » n'est ni une commande ni un fichier ; glucose aide les dit toutes")
    })
}

/// **Aiguille la ligne de commande** : `Ok` — Glucose s'ouvre, sur ce document s'il y en a un ;
/// `Err` — la commande est faite, et voici son code de sortie.
pub fn aiguiller(mots: &[String]) -> Result<Option<PathBuf>, i32> {
    let demande = lire(mots).map_err(|e| {
        crate::dire_erreur!("glucose : {e}");
        2
    })?;
    match demande {
        Demande::Ouvrir(chemin) => Ok(chemin),
        Demande::Ailleurs => Ok(None),
        Demande::Aide(commande) => Err(match aide(commande.as_deref()) {
            Ok(texte) => {
                crate::dire!("{texte}");
                0
            }
            Err(e) => {
                crate::dire_erreur!("glucose : {e}");
                2
            }
        }),
        Demande::Outil(outil, suite) => Err(executer(outil, &suite)),
    }
}

/// **Exécute la commande d'un outil** dans le Glucose ouvert ; rend le code de sortie.
fn executer(outil: &str, mots: &[String]) -> i32 {
    let commande = commande_de(outil);
    let schema = outils::schema(outil).unwrap_or(Json::Nul);
    let mut entree = || {
        let mut t = String::new();
        std::io::stdin()
            .read_to_string(&mut t)
            .map(|_| t)
            .map_err(|e| format!("l'entrée standard ne se lit pas : {e}"))
    };
    let arguments = match arguments::lire(&schema, mots, &mut entree) {
        Ok(a) => a,
        Err(e) => {
            crate::dire_erreur!("glucose {commande} : {e}");
            return 2;
        }
    };
    let Some((nom, ouvrir)) = relais::porte_a_joindre() else {
        crate::dire_erreur!("glucose {commande} : aucune porte pour ce compte");
        return 1;
    };
    match appeler(&nom, ouvrir, outil, arguments) {
        Ok(rendu) => {
            let ecrit = if std::io::stdout().is_terminal() {
                rendu.ecrire_en_retrait()
            } else {
                rendu.ecrire()
            };
            crate::dire!("{ecrit}");
            0
        }
        Err(e) => {
            crate::dire_erreur!("glucose {commande} : {e}");
            1
        }
    }
}

/// **Appelle un outil** par la porte `nom` — en ouvrant Glucose s'il le faut et si `ouvrir` —, et
/// rend ce qu'il dit, ou ce qui l'a empêché.
pub fn appeler(nom: &str, ouvrir: bool, outil: &str, arguments: Json) -> Result<Json, String> {
    let mut flux = relais::brancher(nom, ouvrir)?;
    let ligne = requete(outil, arguments).ecrire();
    writeln!(flux, "{ligne}")
        .and_then(|()| flux.flush())
        .map_err(|e| format!("la porte de Glucose s'est fermée : {e}"))?;
    let reponse = porte::lire_une_ligne(&mut BufReader::new(flux))
        .map_err(|e| format!("la réponse de Glucose ne se lit pas : {e}"))?
        .ok_or("Glucose a fermé la porte sans répondre")?;
    resultat_de(&reponse)
}

/// **La ligne d'un appel** : le protocole moderne (sans état), qui porte sa version et le nom du
/// client dans `_meta`.
pub fn requete(outil: &str, arguments: Json) -> Json {
    let meta = Json::objet([
        (
            protocole::CLE_VERSION,
            Json::from(protocole::VERSION_MODERNE),
        ),
        (
            protocole::CLE_CLIENT,
            Json::objet([
                ("name", Json::from(CLIENT)),
                ("version", Json::from(env!("CARGO_PKG_VERSION"))),
            ]),
        ),
    ]);
    Json::objet([
        ("jsonrpc", Json::from("2.0")),
        ("id", Json::from(1.0)),
        ("method", Json::from("tools/call")),
        (
            "params",
            Json::objet([
                ("name", Json::from(outil)),
                ("arguments", arguments),
                ("_meta", meta),
            ]),
        ),
    ])
}

/// **Ce que dit la réponse** : le JSON que l'outil a rendu, ou la phrase qui l'a empêché — un refus
/// de l'outil (`isError`) comme une erreur du protocole.
pub fn resultat_de(ligne: &str) -> Result<Json, String> {
    let reponse = Json::lire(ligne).map_err(|e| format!("une réponse illisible : {e}"))?;
    if let Some(erreur) = reponse.get("error") {
        let message = erreur.get("message").and_then(Json::as_str);
        return Err(message.unwrap_or("une erreur sans message").to_string());
    }
    let resultat = reponse.get("result").ok_or("une réponse sans résultat")?;
    let texte = resultat
        .get("content")
        .and_then(Json::as_liste)
        .and_then(<[Json]>::first)
        .and_then(|c| c.get("text"))
        .and_then(Json::as_str)
        .unwrap_or_default();
    if resultat.get("isError").and_then(Json::as_bool) == Some(true) {
        return Err(texte.to_string());
    }
    Ok(Json::lire(texte).unwrap_or_else(|_| Json::from(texte)))
}

#[cfg(test)]
mod tests;
