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

use glucose_desktop::app::GlucoseApp;
use glucose_desktop::boite_noire::bilan::Fin;
use glucose_desktop::interactions::pincement;
use glucose_desktop::mise_a_jour::cycle;
use glucose_desktop::mise_a_jour::installation::Installation;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Avant toute fenêtre et tout dialogue : qui est Glucose pour la barre des tâches.
    glucose_desktop::plateforme::identite::declarer();
    let dossier = glucose_desktop::present::souvenir::dossier();
    // Où se posent les installeurs d'une mise à jour, et ce qu'on y range (fiche 48).
    let installeurs = dossier.join("mises-a-jour");
    // `--version`, `--mettre-a-jour` : ce que les épreuves de GitHub demandent, sans fenêtre.
    if let Some(code) = cycle::commande(std::env::args().nth(1).as_deref(), &installeurs) {
        std::process::exit(code);
    }

    let mut constructeur = EventLoop::builder();
    // Ce que le pavé tactile dit du pincement passe à côté de `winit` : le pont doit être en
    // place avant que la boucle existe, sinon le premier geste est déjà perdu.
    pincement::brancher(&mut constructeur);
    let event_loop = constructeur.build()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = GlucoseApp::new();
    // Le dossier de l'utilisateur, et ce qui l'y attend : le dernier document, ou ce qu'un
    // plantage a laissé. Ici et non dans `new` : les épreuves créent des centaines
    // d'applications, et aucune ne doit écrire chez l'utilisateur ni rouvrir son travail.
    app.habiter(&dossier);
    // Ce qu'une page livre et qui n'est pas une image va où un navigateur l'aurait mis
    // (DEPOT-4). Pas dans `habiter`, que `new` ouvre sur un dossier temporaire : ce dossier-ci
    // ne dérive pas de celui de l'application, et seul le vrai lancement le connaît.
    app.depot.telechargements = glucose_desktop::plateforme::telechargements::dossier();
    // Le presse-papiers du système, pour l'application seule : les épreuves ont le leur.
    glucose_desktop::interactions::presse_papiers::prendre_celui_du_systeme();
    // La boîte noire (fiche 45) : ce qui se passe, écrit au fil de l'eau ; et comment la
    // session d'avant a fini.
    let precedente = app.chronique.ouvrir_la_boite_noire(&dossier);
    if let Some(p) = &precedente {
        println!("[Glucose] la session précédente : {}", p.dire());
    }
    if let Some(boite) = app.chronique.boite_noire() {
        boite.temoigner_des_paniques();
    }
    // Comment ce Glucose est installé : celui qui ne sait pas se remplacer (NixOS, `cargo run`)
    // ne cherche aucune mise à jour. Après une session qui a mal fini, elle se cherche AVANT
    // tout ce qui peut tomber — la carte graphique, le document (fiche 48).
    let installation = Installation::de_ce_programme();
    let a_mal_fini = precedente.is_some_and(|p| p.fin != Fin::Propre);
    if let (true, Some(i)) = (a_mal_fini, &installation) {
        if app.mettre_a_jour_avant_tout(&installeurs, i) {
            app.chronique.clore_la_boite_noire();
            return Ok(());
        }
    }
    app.retrouver_le_travail();
    // Ce qui réveillera la boucle quand le système changera le budget de la carte, même si
    // Glucose dort (ETAGES-2) — et quand la veille des mises à jour aura du neuf.
    app.lancement.reveil = Some(event_loop.create_proxy());
    if let Some(installation) = installation {
        let proxy = event_loop.create_proxy();
        let reveil: glucose_desktop::plateforme::Reveil = std::sync::Arc::new(move || {
            let _ = proxy.send_event(());
        });
        cycle::ranger_les_anciens(&installeurs, &cycle::courante());
        app.lancement.mise_a_jour =
            Some(cycle::Veille::commencer(installeurs, reveil, installation));
    }
    event_loop.run_app(&mut app)?;

    Ok(())
}
