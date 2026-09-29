//! **La mise à jour, son cœur** : ce qui garantit qu'on ne perd jamais une version (fiche 44
//! § 4).
//!
//! # Le format et la clé de Tauri
//!
//! Glucose Tauri lit `latest.json` et vérifie une signature minisign par sa clé publique.
//! Glucose Rust lit le **même** fichier et vérifie par la **même** clé : la bascule est sans
//! couture, et il n'y a qu'une clé à garder — la sienne, que personne d'autre ne lit.
//!
//! Ce module est le cœur, sans réseau : lire le fichier, dire si une version est à proposer, et
//! vérifier ce qu'on a téléchargé. **Rien ne s'installe qui ne soit signé**, et une version ne
//! fait que monter : un fichier qui proposerait la même ou une plus ancienne ne propose rien.

pub mod version;

use glucose_core::persist::tauri::json;
use glucose_core::persist::tauri::provenance::base64;
use std::fmt;
use version::Version;

/// **La clé publique de Glucose** — celle de Tauri, telle que sa configuration la porte : le
/// base64 du fichier de clé minisign (« minisign public key: AF7A8A0B124C1ABD »). Publique :
/// elle ne signe rien, elle vérifie.
pub const CLE_PUBLIQUE: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEFGN0E4QTBCMTI0QzFBQkQKUldTOUdrd1NDNHA2ci96UEtEaHdadlgvdnZTSkJDcFA3YjlPS2g4RmllOGlNMVVUN1NKaEpVVGsK";

/// Ce que le fichier de mise à jour propose pour cette machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposition {
    pub version: Version,
    pub notes: String,
    /// D'où télécharger l'installeur.
    pub url: String,
    /// Sa signature : le base64 du fichier `.sig` entier, comme Tauri l'écrit.
    pub signature: String,
}

/// Pourquoi une mise à jour ne se propose pas, ou ne s'installe pas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refus {
    /// Le fichier ne se lit pas : ce qui manque.
    Illisible(&'static str),
    /// Le fichier ne propose rien pour cette plateforme.
    SansCettePlateforme(String),
    /// Ce qu'on a téléchargé n'est pas signé par la clé : pourquoi.
    NonSigne(&'static str),
}

impl fmt::Display for Refus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refus::Illisible(quoi) => write!(f, "fichier de mise à jour illisible : {quoi}"),
            Refus::SansCettePlateforme(p) => write!(f, "aucune mise à jour pour {p}"),
            Refus::NonSigne(pourquoi) => write!(f, "mise à jour refusée : {pourquoi}"),
        }
    }
}

/// **La clé de cette machine dans le fichier** : `{système}-{architecture}`, comme Tauri
/// l'écrit (`windows-x86_64`, `darwin-aarch64`, `linux-x86_64`…).
pub fn plateforme() -> String {
    plateforme_de(std::env::consts::OS, std::env::consts::ARCH)
}

/// La clé d'un système et d'une architecture, aux noms de Tauri.
pub fn plateforme_de(systeme: &str, architecture: &str) -> String {
    let systeme = match systeme {
        "macos" => "darwin",
        autre => autre,
    };
    let architecture = match architecture {
        "x86" => "i686",
        "arm" => "armv7",
        autre => autre,
    };
    format!("{systeme}-{architecture}")
}

/// **Ce que le fichier propose à la version `courante`** sur cette `plateforme` — rien si sa
/// version n'est pas strictement plus grande.
pub fn proposition(
    texte: &str,
    courante: &Version,
    plateforme: &str,
) -> Result<Option<Proposition>, Refus> {
    let v = json::lire(texte).map_err(|_| Refus::Illisible("ce n'est pas du JSON"))?;
    let version = v
        .texte("version")
        .and_then(Version::lire)
        .ok_or(Refus::Illisible("sa version"))?;
    if version <= *courante {
        return Ok(None);
    }
    let cible = v
        .champ("platforms")
        .and_then(|p| p.champ(plateforme))
        .ok_or_else(|| Refus::SansCettePlateforme(plateforme.to_string()))?;
    let url = cible.texte("url").ok_or(Refus::Illisible("l'adresse"))?;
    let signature = cible
        .texte("signature")
        .ok_or(Refus::Illisible("la signature"))?;
    Ok(Some(Proposition {
        version,
        notes: v.texte("notes").unwrap_or_default().to_string(),
        url: url.to_string(),
        signature: signature.to_string(),
    }))
}

/// **Vérifie ce qu'on a téléchargé** contre sa signature, par la clé — l'une et l'autre en base64
/// du fichier entier, comme Tauri les écrit. Les signatures préhachées (`ED`) et historiques
/// (`Ed`) passent, comme chez Tauri.
pub fn verifier(octets: &[u8], signature: &str, cle: &str) -> Result<(), Refus> {
    let texte = |b64: &str| base64(b64).and_then(|o| String::from_utf8(o).ok());
    let cle = texte(cle)
        .and_then(|t| minisign_verify::PublicKey::decode(&t).ok())
        .ok_or(Refus::NonSigne("la clé est illisible"))?;
    let signature = texte(signature)
        .and_then(|t| minisign_verify::Signature::decode(&t).ok())
        .ok_or(Refus::NonSigne("la signature est illisible"))?;
    cle.verify(octets, &signature, true)
        .map_err(|_| Refus::NonSigne("la signature ne correspond pas"))
}

#[cfg(test)]
mod tests;
