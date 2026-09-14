//! Les interactions : ce que l'utilisateur fait, traduit en ce que le document et l'interface
//! font.
//!
//! Chaque module est une responsabilité d'interaction, courte et lisible — et, autant que
//! possible, testable sans fenêtre (fiche 05 § 7.1).

pub mod chrome;
pub mod clipboard;
pub mod cursor;
pub mod domains;
pub mod drag;
pub mod links;
pub mod mouse;
pub mod pan_zoom;
pub mod panels;
pub mod pick;
pub mod resize;
pub mod selection;
pub mod shortcuts;
pub mod text_edit;
pub mod text_entry;
pub mod text_mouse;
pub mod tools;
