//! Les interactions : ce que l'utilisateur fait, traduit en ce que le document et l'interface
//! font.
//!
//! Chaque module est une responsabilité d'interaction, courte et lisible — et, autant que
//! possible, testable sans fenêtre (fiche 05 § 7.1).

pub mod ancrage;
pub mod arrow_edit;
pub mod chrome;
pub mod clipboard;
pub mod cursor;
pub mod depot_web;
pub mod domains;
pub mod drag;
pub mod drop;
pub mod elan;
pub mod links;
pub mod mouse;
pub mod onglets;
pub mod pan_zoom;
pub mod panels;
pub mod pick;
pub mod pincement;
pub mod recadrage;
pub mod resize;
pub mod selection;
pub mod shortcuts;
pub mod survol;
pub mod temps;
pub mod text_edit;
pub mod text_entry;
pub mod text_mouse;
pub mod tools;
pub mod vol;
