//! **Les clés, sur ce Glucose seulement** : un fichier JSON à côté des autres données de
//! l'application, lisible par son seul utilisateur.

use crate::agents::json::Json;
use std::path::{Path, PathBuf};

/// Les clés qu'on a données. Absente ou vide : le fournisseur n'est pas branché.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cles {
    pub google: Option<String>,
}

/// Le fichier des clés, dans le dossier de Glucose.
pub fn chemin() -> PathBuf {
    crate::present::souvenir::dossier().join("ia-cles.json")
}

/// Lit les clés ; un fichier absent ou illisible donne « rien de branché ».
pub fn lire_depuis(chemin: &Path) -> Cles {
    let Ok(texte) = std::fs::read_to_string(chemin) else {
        return Cles::default();
    };
    let Ok(json) = Json::lire(&texte) else {
        return Cles::default();
    };
    let lire = |cle: &str| {
        json.get(cle)
            .and_then(Json::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
    };
    Cles {
        google: lire("google"),
    }
}

/// Écrit les clés, lisibles de leur seul utilisateur, d'un bloc (`persist::atomic`).
pub fn ecrire_dans(chemin: &Path, cles: &Cles) -> Result<(), String> {
    let mut paires = Vec::new();
    if let Some(k) = &cles.google {
        paires.push(("google", Json::texte(k.clone())));
    }
    ecrire_secret(chemin, Json::objet(paires).ecrire().as_bytes())
}

/// **Écrit un secret** dans le dossier de Glucose : le dossier créé s'il manque, le fichier
/// lisible de son seul utilisateur, posé d'un bloc. La clé des IA, le code d'un canevas partagé.
pub fn ecrire_secret(chemin: &Path, octets: &[u8]) -> Result<(), String> {
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier).map_err(|e| format!("dossier : {e}"))?;
    }
    crate::persist::atomic::poser_un_secret_d_un_bloc(chemin, octets)
        .map_err(|e| format!("écriture : {e}"))
}
