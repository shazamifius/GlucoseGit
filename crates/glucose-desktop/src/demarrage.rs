//! **Le démarrage**, commun au programme de bureau (`main.rs`) et au téléphone (la caisse
//! `glucose-android`, fiche 54) : chacun fabrique sa boucle et connaît son dossier ; tout le
//! reste est ici, une seule fois.

use crate::app::GlucoseApp;
use crate::mise_a_jour::cycle;
use crate::mise_a_jour::installation::Installation;
use std::path::Path;
use winit::event_loop::{ControlFlow, EventLoop};

/// **Lance Glucose** sur cette boucle, dans ce dossier, jusqu'à la fermeture.
pub fn lancer(event_loop: EventLoop<()>, dossier: &Path) -> Result<(), Box<dyn std::error::Error>> {
    event_loop.set_control_flow(ControlFlow::Wait);
    // Où se posent les installeurs d'une mise à jour, et ce qu'on y range (fiche 48).
    let installeurs = dossier.join("mises-a-jour");
    let mut app = GlucoseApp::new();
    // Le dossier de l'utilisateur, et ce qui l'y attend : le dernier document, ou ce qu'un
    // plantage a laissé. Ici et non dans `new` : les épreuves créent des centaines
    // d'applications, et aucune ne doit écrire chez l'utilisateur ni rouvrir son travail.
    app.habiter(dossier);
    // Ce qu'une page livre et qui n'est pas une image va où un navigateur l'aurait mis
    // (DEPOT-4). Pas dans `habiter`, que `new` ouvre sur un dossier temporaire : ce dossier-ci
    // ne dérive pas de celui de l'application, et seul le vrai lancement le connaît.
    app.depot.telechargements = crate::plateforme::telechargements::dossier();
    // Le presse-papiers du système, pour l'application seule : les épreuves ont le leur.
    crate::interactions::presse_papiers::prendre_celui_du_systeme();
    // Le clavier du système, s'il y en a un à demander — sous Android (CLAVIER-1).
    if let Some(clavier) = crate::plateforme::clavier::prendre() {
        app.lancement.clavier.brancher(clavier);
    }
    // La boîte noire (fiche 45) : ce qui se passe, écrit au fil de l'eau ; et comment la
    // session d'avant a fini.
    let precedente = app.chronique.ouvrir_la_boite_noire(dossier);
    if let Some(p) = &precedente {
        println!("[Glucose] la session précédente : {}", p.dire());
    }
    if let Some(boite) = app.chronique.boite_noire() {
        boite.temoigner_des_paniques();
    }
    // La boîte noire qui voyage (fiche 54) : avec son accord, les sessions closes partent.
    let session = app
        .chronique
        .boite_noire()
        .map(|b| b.chemin().to_path_buf());
    app.lancement.telemetrie =
        crate::telemetrie::Telemetrie::habiter(dossier.to_path_buf(), session);
    // Comment ce Glucose est installé : celui qui ne sait pas se remplacer (NixOS, `cargo run`,
    // un téléphone) ne cherche aucune mise à jour. Après une session qui a mal fini, elle se
    // cherche AVANT tout ce qui peut tomber — la carte graphique, le document (fiche 48).
    let installation = Installation::de_ce_programme();
    let a_mal_fini = precedente.is_some_and(|p| !p.fin.est_ordinaire());
    if let (true, Some(i)) = (a_mal_fini, &installation) {
        if app.mettre_a_jour_avant_tout(&installeurs, i) {
            app.chronique.clore_la_boite_noire();
            return Ok(());
        }
    }
    // Ce qui réveillera la boucle quand le système changera le budget de la carte, même si
    // Glucose dort (ETAGES-2) — quand la veille des mises à jour aura du neuf, et quand un
    // brouillon tenu au lancement se libérera (LIBERE-1) : avant la reprise, donc.
    app.lancement.reveil = Some(event_loop.create_proxy());
    // `SIGTERM`, `Ctrl+C`, le terminal qui se ferme : Glucose finit au lieu d'être tué — sous
    // Linux (FIN-SESSION-1).
    app.lancement.signaux = crate::plateforme::fin::ecouter_les_signaux(reveil_de(&event_loop));
    app.retrouver_le_travail();
    brancher_les_reveils(&mut app, &event_loop);
    if let Some(installation) = installation {
        let reveil = reveil_de(&event_loop);
        cycle::ranger_les_anciens(&installeurs, &cycle::courante());
        app.lancement.mise_a_jour =
            Some(cycle::Veille::commencer(installeurs, reveil, installation));
    }
    event_loop.run_app(&mut app)?;
    Ok(())
}

/// **Ce qui réveille la boucle depuis un autre fil** : chacun reçoit le sien.
fn brancher_les_reveils(app: &mut GlucoseApp, event_loop: &EventLoop<()>) {
    // Quand le système dit ses marges — la barre d'état, le clavier (BORD-1).
    crate::plateforme::marges::brancher(reveil_de(event_loop));
    // Quand un agent appelle un outil par sa porte (fiche 75).
    app.ouvrir_la_porte_des_agents(reveil_de(event_loop));
    // Quand un sélecteur de fichiers rend son choix (DIAL-2).
    crate::dialogue::brancher(reveil_de(event_loop));
    // Quand Android dit où en est la mise à jour qu'on lui a confiée (MAJ-ANDROID-1).
    crate::plateforme::installation::brancher(reveil_de(event_loop));
}

/// De quoi réveiller cette boucle depuis un autre fil.
fn reveil_de(event_loop: &EventLoop<()>) -> crate::plateforme::Reveil {
    let proxy = event_loop.create_proxy();
    std::sync::Arc::new(move || {
        let _ = proxy.send_event(());
    })
}
