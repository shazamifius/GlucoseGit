//! Types d'erreurs unifiés pour Glucose Desktop Native (Roadmap 1.23, R-21).

use glucose_core::error::CoreError;
use std::fmt;

#[derive(Debug)]
pub enum DesktopError {
    Core(CoreError),
    ImageDecodeFailed { path: String, reason: String },
    ImageDimensionsFailed { path: String, reason: String },
    ClipboardError(String),
    WindowError(String),
    Io(std::io::Error),
}

impl fmt::Display for DesktopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(err) => write!(f, "Noyau : {}", err),
            Self::ImageDecodeFailed { path, reason } => {
                write!(f, "Échec décodage image '{}' : {}", path, reason)
            }
            Self::ImageDimensionsFailed { path, reason } => {
                write!(f, "Format d'image non reconnu pour '{}' : {}", path, reason)
            }
            Self::ClipboardError(msg) => write!(f, "Presse-papiers : {}", msg),
            Self::WindowError(msg) => write!(f, "Fenêtre : {}", msg),
            Self::Io(err) => write!(f, "E/S : {}", err),
        }
    }
}

impl std::error::Error for DesktopError {}

impl From<CoreError> for DesktopError {
    fn from(err: CoreError) -> Self {
        Self::Core(err)
    }
}

impl From<std::io::Error> for DesktopError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

pub type DesktopResult<T> = Result<T, DesktopError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_error_display() {
        let err = DesktopError::ImageDimensionsFailed {
            path: "corrupted.jpg".into(),
            reason: "invalid header".into(),
        };
        assert!(err.to_string().contains("corrupted.jpg"));

        let core_err: DesktopError = CoreError::BoardNotFound("b1".into()).into();
        assert!(core_err.to_string().contains("Tableau introuvable"));
    }
}
