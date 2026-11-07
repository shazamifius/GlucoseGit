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

/// **Écrire une ligne sur la console, sans jamais en mourir** (CONSOLE-1, fiche 88).
///
/// `println!` panique quand la console a disparu : un terminal fermé, un tuyau dont le lecteur est
/// parti. Le 5/11/2026, une session de Glucose sur son VivoBook est tombée ainsi après 21 minutes
/// (`std/src/io/stdio.rs`, ligne 1166, dans sa boîte noire). Ce que Glucose dit sur la console est
/// un diagnostic : si personne ne le lit, il se perd, et Glucose continue. Tout ce qu'il y écrit passe
/// par ici ou par [`dire_erreur!`] (cliquet 13).
#[macro_export]
macro_rules! dire {
    ($($t:tt)*) => {{
        use ::std::io::Write as _;
        let _ = ::std::writeln!(::std::io::stdout(), $($t)*);
    }};
}

/// [`dire!`], sur la sortie d'erreur.
#[macro_export]
macro_rules! dire_erreur {
    ($($t:tt)*) => {{
        use ::std::io::Write as _;
        let _ = ::std::writeln!(::std::io::stderr(), $($t)*);
    }};
}

pub mod agents;
pub mod animation;
pub mod app;
pub mod bench;
pub mod boite_noire;
pub mod cadence;
pub mod canvas;
pub mod chronique;
pub mod commandes;
pub mod composition;
pub mod demarrage;
pub mod dialogue;
pub mod dock;
#[cfg(test)]
mod epreuve;
pub mod error;
pub mod horloge;
pub mod icons;
pub mod interactions;
pub mod memoire;
pub mod mise_a_jour;
pub mod params;
pub mod perception;
pub mod perf;
pub mod persist;
pub mod plateforme;
pub mod present;
pub mod recensement;
pub mod renderer;
pub mod resolution;
pub mod salissure;
pub mod tauri;
pub mod telemetrie;
pub mod tempo;
pub mod theme;
pub mod typography;
pub mod ui;
