//! Point d'entrée de Glucose Desktop Native (PureRef en Rust pur).
//!
//! Tout le programme vit dans la bibliothèque du même crate ([`glucose_desktop`]) ; il ne reste
//! ici que l'ouverture de la fenêtre. Voir l'en-tête de `lib.rs` pour la raison : un binaire pur
//! ne peut être ni mesuré, ni capturé, ni instrumenté de l'extérieur.
//!
//! **Une construction publiée est une application fenêtrée** (fiche 48) : sans cela, Windows
//! ouvre une console noire à côté de Glucose à chaque lancement depuis un raccourci. Ses
//! journaux ne se perdent pas pour autant : redirigés vers un fichier (`> sortie.txt`), ils s'y
//! écrivent. La construction de travail garde sa console.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use glucose_desktop::interactions::pincement;
use glucose_desktop::mise_a_jour::cycle;
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Avant toute fenêtre et tout dialogue : qui est Glucose pour la barre des tâches.
    glucose_desktop::plateforme::identite::declarer();
    let dossier = glucose_desktop::present::souvenir::dossier();
    // `--version`, `--mettre-a-jour` : ce que les épreuves de GitHub demandent, sans fenêtre.
    if let Some(code) = cycle::commande(
        std::env::args().nth(1).as_deref(),
        &dossier.join("mises-a-jour"),
    ) {
        std::process::exit(code);
    }
    let mut constructeur = EventLoop::builder();
    // Ce que le pavé tactile dit du pincement passe à côté de `winit` : le pont doit être en
    // place avant que la boucle existe, sinon le premier geste est déjà perdu.
    pincement::brancher(&mut constructeur);
    glucose_desktop::demarrage::lancer(constructeur.build()?, &dossier)
}
