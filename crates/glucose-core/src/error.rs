//! Types d'erreurs unifiés du noyau Glucose (100% Rust std, 0 dépendance).
//! Respecte la règle R-21 : aucune erreur n'est silencieuse.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    BoardNotFound(String),
    AnnotationNotFound(String),
    ImageNotFound(String),
    FolderNotFound(String),
    InvalidId(String),
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
            Self::InvalidId(id) => write!(f, "Identifiant invalide : '{}'", id),
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
    }
}
