//! **Les modèles, du plus capable au moins** : ce que Google annonce, lu et classé. Pur : rien ici
//! ne parle au réseau.
//!
//! # Le classement n'est qu'un ordre de départ
//!
//! Le nom dit la version et la gamme (`gemini-3.8-flash`), rien de plus : il ne dit ni qu'un
//! modèle est gratuit, ni qu'il répond aujourd'hui. La liste du 22/11 annonce des modèles sans
//! quota gratuit (`gemini-3.1-pro-preview`, « limit: 0 »), d'autres retirés aux comptes neufs
//! (`gemini-2.5-flash`, 404). C'est la [bascule](super::bascule) qui l'apprend, à la première
//! question, et l'endort le temps que Google dit ; ce classement ne fait que dire par qui commencer.
//!
//! # Ce qui entre (fiche 129)
//!
//! Un nom `gemini-<version>-…` dont la version suit aussitôt le préfixe : les familles à part
//! (`gemini-omni-1.2-flash`, `gemini-robotics-er-2-preview`) prenaient leur numéro pour une
//! version — le robot passait devant les modèles de conversation. Sans les spécialités (l'image,
//! la voix, la transcription, l'usage d'un ordinateur). Puis **Gemma**, en dernier : elle sait
//! appeler les outils de Glucose (essayé le 22/11), avec des quotas plus larges, mais vingt-sept
//! secondes pour un seul geste.

use crate::agents::json::Json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fournisseur {
    Google,
}

/// Un modèle qu'un fournisseur ouvre à la clé donnée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modele {
    pub fournisseur: Fournisseur,
    /// Le nom qu'on envoie au fournisseur, sans le préfixe `models/`.
    pub id: String,
    /// Le nom qu'il se donne (« Gemini 3.8 Flash ») : celui que l'utilisateur lit, et qui signe ses
    /// gestes dans la Time Machine.
    pub nom: String,
    /// Plus il est haut, plus le modèle passe tôt (version, puis gamme).
    pub rang: u32,
}

/// Les segments d'un nom qui disent une spécialité, pas une conversation.
const SPECIALITES: [&str; 9] = [
    "image",
    "tts",
    "transcribe",
    "computer",
    "customtools",
    "embedding",
    "audio",
    "live",
    "native",
];

/// La gamme d'un modèle Gemini : pro au-dessus de flash, flash au-dessus de lite.
fn gamme(id: &str) -> u32 {
    let segments: Vec<&str> = id.split('-').collect();
    if segments.contains(&"pro") {
        3
    } else if segments.contains(&"lite") {
        1
    } else if segments.contains(&"flash") {
        2
    } else {
        0
    }
}

/// La version qui suit `gemini-` : « 3.8 » donne (3, 8), « 4 » donne (4, 0). Rien si le segment ne
/// commence pas par une version — une famille à part.
fn version(id: &str) -> Option<(u32, u32)> {
    let segment = id.strip_prefix("gemini-")?.split('-').next()?;
    let (majeure, mineure) = segment.split_once('.').unwrap_or((segment, "0"));
    Some((majeure.parse().ok()?, mineure.parse().ok()?))
}

/// Le rang d'un nom, ou rien s'il n'entre pas : la version d'abord, la gamme ensuite ; Gemma
/// sous tous les Gemini.
pub fn rang(id: &str) -> Option<u32> {
    if id.split('-').any(|s| SPECIALITES.contains(&s)) {
        return None;
    }
    if id.starts_with("gemma-") {
        return Some(0);
    }
    let (majeure, mineure) = version(id)?;
    Some(1 + (majeure * 100 + mineure) * 10 + gamme(id))
}

/// Les modèles de conversation qu'une liste annonce, du premier à essayer au dernier.
pub fn gemini(reponse: &Json) -> Vec<Modele> {
    let Some(liste) = reponse.get("models").and_then(Json::as_liste) else {
        return Vec::new();
    };
    let mut modeles: Vec<Modele> = liste
        .iter()
        .filter(|m| sait_generer(m))
        .filter_map(|m| {
            let id = m.get("name")?.as_str()?.trim_start_matches("models/");
            let nom = m.get("displayName").and_then(Json::as_str).unwrap_or(id);
            Some(Modele {
                fournisseur: Fournisseur::Google,
                rang: rang(id)?,
                id: id.to_string(),
                nom: nom.to_string(),
            })
        })
        .collect();
    modeles.sort_by(|a, b| b.rang.cmp(&a.rang).then_with(|| a.id.cmp(&b.id)));
    modeles
}

/// Le modèle sait-il `generateContent` ?
fn sait_generer(modele: &Json) -> bool {
    modele
        .get("supportedGenerationMethods")
        .and_then(Json::as_liste)
        .is_some_and(|m| m.iter().any(|x| x.as_str() == Some("generateContent")))
}
