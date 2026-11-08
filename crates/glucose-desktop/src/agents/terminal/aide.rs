//! **`glucose aide`** : ce que fait chaque commande, tiré de la description de son outil
//! (TERMINAL-1). L'aide que lit l'humain est ce qu'un agent lit à la découverte.

use super::{arguments, commande_de, outil_de};
use crate::agents::json::Json;
use crate::agents::outils;
use std::fmt::Write as _;

/// **L'aide** de toutes les commandes, ou d'une seule.
pub fn aide(commande: Option<&str>) -> Result<String, String> {
    match commande {
        None => Ok(sommaire()),
        Some(c) => outil_de(c)
            .map(detail)
            .ok_or_else(|| format!("« {c} » n'est pas une commande ; glucose aide les dit toutes")),
    }
}

/// Chaque commande, et la première phrase de ce qu'elle fait.
fn sommaire() -> String {
    let mut s = String::from(
        "Glucose — la toile infinie des idées et des images.\n\n\
         \x20 glucose                        ouvre Glucose\n\
         \x20 glucose <document.glucose>     ouvre ce document\n\
         \x20 glucose aide <commande>        ce que fait une commande, et ses arguments\n\n\
         Les commandes — chacune est un outil que les agents appellent aussi (glucose --mcp) :\n",
    );
    for outil in outils::noms() {
        let phrase = outils::description(outil)
            .and_then(glucose_core::recherche::premiere_phrase)
            .unwrap_or_default();
        let _ = write!(s, "\n  {}\n      {phrase}\n", commande_de(outil));
    }
    s.push_str(
        "\nUne commande parle au Glucose ouvert, et l'ouvre s'il ne l'est pas ; la première fois, \
         Glucose te demande ton accord. Chaque geste s'annule par Ctrl+Z.",
    );
    s
}

/// Ce que fait une commande, et chacun de ses arguments.
fn detail(outil: &str) -> String {
    let schema = outils::schema(outil).unwrap_or(Json::Nul);
    let requis = schema
        .get("required")
        .and_then(Json::as_liste)
        .unwrap_or_default();
    let mut s = format!(
        "glucose {} — {}\n\nArguments, en clé=valeur (une valeur « - » lit l'entrée standard) :\n",
        commande_de(outil),
        outils::description(outil).unwrap_or_default()
    );
    for cle in arguments::cles(&schema) {
        let propriete = schema.get("properties").and_then(|p| p.get(&cle));
        let mut ligne = format!("  {cle} : {}", nature(propriete));
        if requis.iter().any(|r| r.as_str() == Some(cle.as_str())) {
            ligne.push_str(", exigé");
            if requis.len() == 1 {
                ligne.push_str(" — s'écrit aussi sans « clé= »");
            }
        }
        if let Some(d) = propriete
            .and_then(|p| p.get("description"))
            .and_then(Json::as_str)
        {
            let _ = write!(ligne, " — {d}");
        }
        s.push_str(&ligne);
        s.push('\n');
    }
    s
}

/// Ce qu'une clé attend, en mots.
fn nature(propriete: Option<&Json>) -> String {
    let choix = propriete
        .and_then(|p| p.get("enum"))
        .and_then(Json::as_liste);
    if let Some(choix) = choix {
        let mots: Vec<&str> = choix.iter().filter_map(Json::as_str).collect();
        return mots.join(" ou ");
    }
    match propriete.and_then(|p| p.get("type")).and_then(Json::as_str) {
        Some("number") => "un nombre".into(),
        Some("boolean") => "true ou false".into(),
        Some("array") => "une liste (répéter la clé, ou du JSON)".into(),
        _ => "un texte".into(),
    }
}
