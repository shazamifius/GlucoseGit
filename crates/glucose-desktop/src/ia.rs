//! **Les IA gratuites** (fiches 128 et 129) : le bouton « L'API publique » de Collaborer, et la
//! discussion.
//!
//! Glucose range ici la clé qu'on lui donne (Google AI Studio), lit les modèles qu'elle ouvre sans
//! payer ([`modeles`]), leur parle ([`gemini`], [`discussion`]) et passe de l'un à l'autre quand
//! l'un atteint sa limite ([`bascule`]). La clé reste sur ce Glucose, dans son dossier, à ses seuls
//! droits (0600) ; elle ne part que vers Google, et n'est jamais écrite dans un journal.
//!
//! GitHub Models, que la fiche 128 prévoyait, a fermé le 30/07/2026 (« GitHub Models has been
//! fully retired », docs.github.com) : son bouton est parti avec lui.

pub mod bascule;
pub mod cles;
pub mod discussion;
pub mod gemini;
pub mod modeles;

#[cfg(test)]
mod tests;
