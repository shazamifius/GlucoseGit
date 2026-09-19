//! Glucose Desktop — l'application native, exposée comme bibliothèque.
//!
//! # Pourquoi ce fichier existe
//!
//! Le crate n'était qu'un binaire. Un binaire ne se teste que de l'intérieur : aucun banc
//! d'essai, aucun exemple, aucun outil de mesure ne peut atteindre son rendu. C'est ce qui a
//! laissé le travail A.5 du plan de marche — « bancs d'essai, chiffres publiés, build en échec
//! au-delà de 10 % de régression » — sans moyen d'être fait.
//!
//! Tout vit donc ici, et [`main`](../main/index.html) se réduit à ouvrir une fenêtre. La
//! conséquence directe est que le rendu devient **mesurable et capturable sans écran** :
//! `Renderer::render` écrit dans un `PixmapMut`, qui n'a jamais eu besoin d'une fenêtre pour
//! exister.
//!
//! # Pourquoi les modules sont publics
//!
//! Un crate applicatif n'a pas d'API à protéger : personne d'autre ne le consomme. La barrière
//! `pub` n'y sépare donc rien d'utile — elle empêche seulement ses propres bancs et ses propres
//! preuves d'atteindre ce qu'ils mesurent.

pub mod animation;
pub mod app;
pub mod bench;
pub mod cadence;
pub mod canvas;
pub mod chronique;
pub mod composition;
pub mod dialogue;
pub mod dock;
pub mod error;
pub mod horloge;
pub mod icons;
pub mod interactions;
pub mod memoire;
pub mod params;
pub mod perception;
pub mod perf;
pub mod persist;
pub mod present;
pub mod renderer;
pub mod resolution;
pub mod salissure;
pub mod tempo;
pub mod theme;
pub mod typography;
pub mod ui;
