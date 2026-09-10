//! Modules d'interaction utilisateur de Glucose Desktop.
//!
//! Éclatement architectural d'app.rs (Roadmap 1.26, R-19) :
//! Chaque sous-module encapsule une responsabilité d'interaction autonome
//! et hautement lisible.

pub mod clipboard;
pub mod cursor;
pub mod drag;
pub mod mouse;
pub mod pan_zoom;
pub mod selection;
pub mod shortcuts;
pub mod text_edit;
