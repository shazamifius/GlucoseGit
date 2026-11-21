//! **Les IA publiques gratuites** (fiche 128) : le bouton « API publique » de Collaborer.
//!
//! Glucose range ici les clés qu'on lui donne (Google AI Studio, GitHub), cherche les modèles
//! qu'elles ouvrent sans payer, et garde de quoi basculer de l'un à l'autre quand l'un atteint sa
//! limite. Les clés restent sur ce Glucose, dans son dossier, à leurs seuls droits (0600) ; elles
//! ne partent que vers le fournisseur qui les a données, et ne sont jamais écrites dans un journal.

pub mod cles;
pub mod modeles;

#[cfg(not(windows))]
pub mod essai;

#[cfg(test)]
mod tests;
