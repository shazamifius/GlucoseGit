//! Les documents de **Glucose Tauri**, lus par Glucose Rust.
//!
//! # Pourquoi ce module existe
//!
//! Un utilisateur de Glucose Tauri qui passe à Glucose Rust doit retrouver ses documents.
//! Jusqu'au 24/09/2026, aucun n'avait jamais été ouvert ici (fiche 14 § 5, fiche 36 § 1.3) :
//! le conteneur les refusait en nommant « l'importeur des projets v1 », qui n'existait pas.
//!
//! # Les trois formes d'un document Tauri
//!
//! | forme | reconnue par | lue par |
//! |---|---|---|
//! | binaire Automerge (depuis juillet 2026) | la signature `85 6f 4a 83` | [`automerge`] |
//! | JSON d'avant (« v1 ») | un `{` en tête | [`json`] |
//! | dossier portable (`project.glucose` + `objects/`) | le fichier Automerge qu'il contient | les deux ci-dessus |
//!
//! Les deux lecteurs rendent la même [`Valeur`], que [`projet`] traduit en [`Project`] — un
//! seul traducteur, donc une seule correspondance des champs à tenir juste.
//!
//! # Pourquoi ici, dans le noyau
//!
//! Tout ce module est **pur** : des octets entrent, un projet sort. Il vit donc à côté des
//! autres migrations du format, sans dépendance — même DEFLATE, que les colonnes d'Automerge
//! emploient, est écrit ici ([`inflate`]). Le bureau ne garde que ce qui touche au disque :
//! retrouver les octets des images dans le magasin de Tauri.
//!
//! [`Project`]: crate::types::Project

pub mod automerge;
pub mod inflate;
pub mod json;
pub mod projet;
pub mod provenance;
mod valeur;

pub use valeur::Valeur;

use crate::error::CoreError;

/// Une faute dans les octets d'un document Tauri. Le message dit ce qui manque, sans jargon
/// de format : c'est lui qui atteint l'utilisateur.
pub fn illisible(detail: &str) -> CoreError {
    CoreError::DeserializationError(format!("document de Glucose Tauri illisible : {detail}"))
}

/// Ce qu'on reconnaît en tête d'un fichier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forme {
    Automerge,
    Json,
}

/// La forme d'un document Tauri, ou rien si ces octets n'en sont pas un.
///
/// Un JSON se reconnaît à son `{` après d'éventuels blancs ; un conteneur de Glucose Rust
/// commence par `GLUCOSE`, et n'est donc jamais pris pour l'un ou l'autre.
pub fn reconnaitre(octets: &[u8]) -> Option<Forme> {
    if octets.starts_with(&automerge::MAGIE) {
        return Some(Forme::Automerge);
    }
    let premier = octets.iter().find(|o| !o.is_ascii_whitespace())?;
    (*premier == b'{').then_some(Forme::Json)
}

/// Lit un document Tauri, quelle que soit sa forme, jusqu'à la [`Valeur`] de son projet.
///
/// Rend aussi le nombre d'octets de fin ignorés — la trace d'un enregistrement interrompu
/// chez Tauri, qu'il faut dire à l'utilisateur plutôt que taire.
pub fn lire(octets: &[u8]) -> Result<(Valeur, usize), CoreError> {
    match reconnaitre(octets) {
        Some(Forme::Automerge) => automerge::lire(octets).map(|lu| (lu.valeur, lu.fin_ignoree)),
        Some(Forme::Json) => {
            let texte = std::str::from_utf8(octets)
                .map_err(|_| illisible("le JSON n'est pas de l'UTF-8"))?;
            json::lire(texte).map(|v| (v, 0))
        }
        None => Err(illisible(
            "ni un binaire Automerge ni un JSON — ce n'est pas un document de Glucose Tauri",
        )),
    }
}
