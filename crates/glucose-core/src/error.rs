//! Types d'erreurs unifiés du noyau Glucose (100% Rust std, 0 dépendance).
//! Respecte la règle R-21 : aucune erreur n'est silencieuse.

use std::fmt;

/// `Eq` n'est pas dérivable : [`CoreError::InvalidWeight`] porte le poids refusé, et un `f64`
/// n'a pas d'égalité totale. Montrer la valeur rejetée vaut mieux que la taire (standard 6.5) :
/// « Pondération invalide : NaN » nomme la faute, « poids invalide » la laisse deviner.
#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    BoardNotFound(String),
    AnnotationNotFound(String),
    ImageNotFound(String),
    FolderNotFound(String),
    /// Nœud introuvable : ni annotation, ni image du tableau visé.
    NodeNotFound(String),
    /// Domaine absent du catalogue du projet.
    DomainNotFound(String),
    /// Un domaine porte déjà cet identifiant (R-47 : deux domaines homonymes rendaient
    /// l'assignation ambiguë et la suppression destructrice).
    DuplicateDomainId(String),
    /// Pondération hors de `0.0..=1.0`, infinie ou `NaN`.
    InvalidWeight(f64),
    /// Le nœud ne porte pas ce domaine : il n'y a rien à retirer.
    DomainNotAssigned { node_id: String, domain_id: String },
    InvalidId(String),
    /// Opération refusée par un invariant du modèle (dernier tableau, cycle, etc.).
    /// Le message dit **quoi faire**, pas seulement ce qui a échoué (standard 6.5).
    InvalidOperation(String),
    CycleDetected(String),
    IoError(String),
    SerializationError(String),
    DeserializationError(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BoardNotFound(id) => write!(f, "Tableau introuvable : '{}'", id),
            Self::AnnotationNotFound(id) => write!(f, "Annotation introuvable : '{}'", id),
            Self::ImageNotFound(id) => write!(f, "Image introuvable : '{}'", id),
            Self::FolderNotFound(id) => write!(f, "Dossier introuvable : '{}'", id),
            Self::NodeNotFound(id) => write!(
                f,
                "Nœud introuvable : '{}' — ce tableau ne porte ni annotation ni image de cet identifiant",
                id
            ),
            Self::DomainNotFound(id) => write!(
                f,
                "Domaine introuvable : '{}' — crée-le dans le panneau DOMAINES avant de l'assigner",
                id
            ),
            Self::DuplicateDomainId(id) => write!(
                f,
                "Domaine en double : '{}' — cet identifiant est déjà pris, demandes-en un neuf au générateur du store",
                id
            ),
            Self::InvalidWeight(weight) => write!(
                f,
                "Pondération invalide : {} — un poids est un nombre fini de 0,0 à 1,0",
                weight
            ),
            Self::DomainNotAssigned { node_id, domain_id } => write!(
                f,
                "Le nœud '{}' ne porte pas le domaine '{}' — il n'y a rien à retirer",
                node_id, domain_id
            ),
            Self::InvalidId(id) => write!(f, "Identifiant invalide : '{}'", id),
            Self::InvalidOperation(msg) => write!(f, "Opération impossible : {}", msg),
            Self::CycleDetected(msg) => write!(f, "Cycle interdit : {}", msg),
            Self::IoError(msg) => write!(f, "Erreur E/S noyau : {}", msg),
            Self::SerializationError(msg) => write!(f, "Erreur de sérialisation : {}", msg),
            Self::DeserializationError(msg) => write!(f, "Erreur de désérialisation : {}", msg),
        }
    }
}

impl std::error::Error for CoreError {}

pub type CoreResult<T> = Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_error_display() {
        let err = CoreError::BoardNotFound("board-42".into());
        assert_eq!(err.to_string(), "Tableau introuvable : 'board-42'");

        let cycle = CoreError::CycleDetected("A -> B -> A".into());
        assert_eq!(cycle.to_string(), "Cycle interdit : A -> B -> A");

        let refus = CoreError::InvalidOperation("c'est le dernier tableau".into());
        assert_eq!(refus.to_string(), "Opération impossible : c'est le dernier tableau");
    }

    /// Standard 6.5 — chacune des erreurs de domaine nomme la faute **et** dit quoi faire.
    #[test]
    fn test_domain_errors_name_the_fault_and_say_what_to_do() {
        let unknown = CoreError::DomainNotFound("domain-7".into()).to_string();
        assert!(unknown.contains("domain-7"), "{unknown}");
        assert!(unknown.contains("panneau DOMAINES"), "{unknown}");

        let dup = CoreError::DuplicateDomainId("domain-7".into()).to_string();
        assert!(dup.contains("déjà pris"), "{dup}");

        let weight = CoreError::InvalidWeight(1.7).to_string();
        assert!(weight.contains("1.7"), "{weight}");
        assert!(weight.contains("0,0 à 1,0"), "{weight}");
        assert!(CoreError::InvalidWeight(f64::NAN).to_string().contains("NaN"));

        let node = CoreError::NodeNotFound("n-1".into()).to_string();
        assert!(node.contains("ni annotation ni image"), "{node}");

        let loose = CoreError::DomainNotAssigned {
            node_id: "n-1".into(),
            domain_id: "domain-7".into(),
        }
        .to_string();
        assert!(loose.contains("rien à retirer"), "{loose}");
    }
}
