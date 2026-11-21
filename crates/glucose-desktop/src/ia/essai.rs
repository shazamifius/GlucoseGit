//! **Essayer une clé** : ce qu'elle ouvre, et si un modèle répond vraiment. Le réseau ne passe
//! que par ici ; le reste de [`super::modeles`] se lit sans lui.

use super::modeles::{self, Modele};
use crate::agents::json::Json;
use std::time::Duration;

const GEMINI: &str = "https://generativelanguage.googleapis.com/v1beta";
const DELAI: Duration = Duration::from_secs(20);
/// Combien de modèles on essaie vraiment, du meilleur au moins bon. Google liste des modèles que les
/// nouveaux comptes ne peuvent plus utiliser, et d'autres répondent 429 ou 503 : la liste ne suffit pas.
pub const ESSAIS_MAX: usize = 8;

/// Le résultat d'un essai de clé : ce qu'elle ouvre, et ce qui a répondu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bilan {
    pub modeles: Vec<Modele>,
    /// Les identifiants qui ont vraiment généré une réponse, du meilleur au moins bon.
    pub qui_repond: Vec<String>,
}

fn agent() -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(DELAI))
        .user_agent("Glucose")
        .http_status_as_error(false)
        .build();
    ureq::Agent::new_with_config(config)
}

/// Lit une réponse : son code, et son texte si c'est un 200.
fn lire(mut rep: ureq::http::Response<ureq::Body>) -> Result<String, String> {
    let statut = rep.status().as_u16();
    let texte = rep
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("lecture : {e}"))?;
    if statut == 200 {
        Ok(texte)
    } else {
        Err(format!("le fournisseur répond {statut}"))
    }
}

/// Les modèles Gemini que cette clé ouvre, du plus capable au moins capable.
pub fn lister_gemini(cle: &str) -> Result<Vec<Modele>, String> {
    let rep = agent()
        .get(&format!("{GEMINI}/models?pageSize=200"))
        .header("x-goog-api-key", cle)
        .call()
        .map_err(|e| format!("requête : {e}"))?;
    let texte = lire(rep)?;
    Ok(modeles::gemini(&Json::lire(&texte)?))
}

/// Un modèle Gemini génère-t-il une réponse à cette clé ? Une seule question, courte.
pub fn generer_gemini(cle: &str, id: &str) -> Result<(), String> {
    let corps = r#"{"contents":[{"parts":[{"text":"Réponds par un seul mot : bonjour."}]}]}"#;
    let rep = agent()
        .post(&format!("{GEMINI}/models/{id}:generateContent"))
        .header("x-goog-api-key", cle)
        .header("Content-Type", "application/json")
        .send(corps)
        .map_err(|e| format!("requête : {e}"))?;
    lire(rep).map(|_| ())
}

/// Tout l'essai d'une clé Google : ce qu'elle ouvre, puis les meilleurs modèles, un à un.
pub fn essayer_google(cle: &str) -> Result<Bilan, String> {
    let modeles = lister_gemini(cle)?;
    let mut qui_repond = Vec::new();
    for modele in modeles.iter().take(ESSAIS_MAX) {
        if generer_gemini(cle, &modele.id).is_ok() {
            qui_repond.push(modele.id.clone());
        }
    }
    Ok(Bilan {
        modeles,
        qui_repond,
    })
}
