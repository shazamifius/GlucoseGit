//! **Les modèles, du plus capable au moins** : ce que chaque fournisseur annonce, lu et classé.
//! Pur : rien ici ne parle au réseau.

use crate::agents::json::Json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fournisseur {
    Google,
    GitHub,
}

/// Un modèle qu'un fournisseur ouvre à la clé donnée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modele {
    pub fournisseur: Fournisseur,
    /// Le nom qu'on envoie au fournisseur, sans le préfixe `models/`.
    pub id: String,
    /// Plus il est haut, plus le modèle est capable (version, puis gamme).
    pub rang: u32,
}

/// La gamme d'un modèle Gemini : pro au-dessus de flash, flash au-dessus de lite.
fn gamme(id: &str) -> u32 {
    if id.contains("pro") {
        3
    } else if id.contains("flash-lite") || id.contains("lite") {
        1
    } else if id.contains("flash") {
        2
    } else {
        0
    }
}

/// La version « 2.5 » du nom « gemini-2.5-pro » : (2, 5) ; un alias sans version vaut (0, 0).
fn version(id: &str) -> (u32, u32) {
    let apres = id.trim_start_matches("gemini-");
    let mut morceaux = apres
        .split(['-', '.'])
        .filter_map(|m| m.parse::<u32>().ok());
    let majeure = morceaux.next().unwrap_or(0);
    let mineure = morceaux.next().unwrap_or(0);
    (majeure, mineure)
}

/// Le rang d'un modèle : la version d'abord, la gamme ensuite.
pub fn rang_gemini(id: &str) -> u32 {
    let (majeure, mineure) = version(id);
    (majeure * 100 + mineure) * 10 + gamme(id)
}

/// Garde les modèles Gemini qui génèrent du texte : sans les images, la voix ni les plongements,
/// et sans les autres familles (recherche, antigravity…) dont les numéros fausseraient le rang.
fn genere_du_texte(id: &str) -> bool {
    id.starts_with("gemini-")
        && ![
            "image",
            "tts",
            "embedding",
            "customtools",
            "nano",
            "imagen",
            "veo",
        ]
        .iter()
        .any(|motif| id.contains(motif))
}

/// Les modèles Gemini qui génèrent du texte, du plus capable au moins capable.
pub fn gemini(reponse: &Json) -> Vec<Modele> {
    let Some(Json::Liste(liste)) = reponse.get("models") else {
        return Vec::new();
    };
    let mut modeles: Vec<Modele> = liste
        .iter()
        .filter(|m| sait_generer(m))
        .filter_map(|m| m.get("name").and_then(Json::as_str))
        .map(|nom| nom.trim_start_matches("models/").to_string())
        .filter(|id| genere_du_texte(id))
        .map(|id| Modele {
            fournisseur: Fournisseur::Google,
            rang: rang_gemini(&id),
            id,
        })
        .collect();
    modeles.sort_by(|a, b| b.rang.cmp(&a.rang).then_with(|| a.id.cmp(&b.id)));
    modeles
}

/// Le modèle sait-il `generateContent` ?
fn sait_generer(modele: &Json) -> bool {
    match modele.get("supportedGenerationMethods") {
        Some(Json::Liste(m)) => m.iter().any(|x| x.as_str() == Some("generateContent")),
        _ => false,
    }
}
