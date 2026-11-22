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

/// Écrit les clés. Le fichier est créé à 0600 sous Unix : son seul utilisateur le lit.
pub fn ecrire_dans(chemin: &Path, cles: &Cles) -> Result<(), String> {
    let mut paires = Vec::new();
    if let Some(k) = &cles.google {
        paires.push(("google", Json::texte(k.clone())));
    }
    let texte = Json::objet(paires).ecrire();
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier).map_err(|e| format!("dossier : {e}"))?;
    }
    ecrire_fichier(chemin, texte.as_bytes()).map_err(|e| format!("écriture : {e}"))
}

#[cfg(unix)]
fn ecrire_fichier(chemin: &Path, octets: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(chemin)?;
    f.write_all(octets)
}

#[cfg(not(unix))]
fn ecrire_fichier(chemin: &Path, octets: &[u8]) -> std::io::Result<()> {
    std::fs::write(chemin, octets)
}
