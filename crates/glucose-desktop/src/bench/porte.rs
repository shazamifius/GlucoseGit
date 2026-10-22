//! **Glucose sans fenêtre, sa porte ouverte** (fiche 75) : pour regarder ce qu'un agent pose
//! dans un tableau, sans rien ouvrir sur l'écran de l'utilisateur — puis le rendre en images.
//!
//! L'application est la vraie (`GlucoseApp`), sa boucle aussi (`suivre_les_agents`) ; seule la
//! fenêtre manque. Personne n'est là pour répondre à la question d'accord : ce banc y répond
//! « Autoriser » par le chemin même de la main (`repondre_a_la_question`). Il n'existe que pour
//! les bancs et l'exemple `porte_sans_fenetre` ; le vrai Glucose demande toujours.

use crate::agents::porte::Porte;
use crate::agents::Boite;
use crate::app::agents::Agents;
use crate::app::GlucoseApp;
use crate::ui::question::{Reponse, Suite};
use glucose_core::geometry::Rect;
use glucose_core::store::agents::Sorte;
use glucose_core::types::Viewport;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// **Sert les agents** sur la porte `nom` jusqu'à ce que le fichier `arret` existe, puis rend le
/// tableau entier, et chaque membrane qui n'est dans aucune autre, en PNG dans `dossier`. Rend le
/// nombre de gestes reçus. Les gestes d'un utilisateur écrits dans `dossier/main.jsonl` sont
/// joués par la main d'épreuve.
pub fn servir_sans_fenetre(
    nom: &str,
    arret: &Path,
    dossier: &Path,
    taille: (u32, u32),
) -> Result<usize, String> {
    let mut app = GlucoseApp::new();
    let boite = Boite::nouvelle(Arc::new(|| {}));
    let _porte = Porte::ouvrir(nom, boite.clone()).ok_or("la porte ne s'ouvre pas")?;
    app.lancement.agents = Agents::avec_la_boite(boite);
    println!("[porte] ouverte : {nom}");
    // Les gestes d'un utilisateur, s'il y en a à jouer (fiche 80) : `<dossier>/main.jsonl`.
    let mut main = super::main_d_epreuve::MainDEpreuve::nouvelle(dossier.join("main.jsonl"));
    let mut gestes = 0;
    while !arret.exists() {
        main.jouer(&mut app);
        let avant = app.store.undo_depth();
        app.suivre_les_agents();
        if matches!(app.ui.question, Some((_, Suite::Agent(_)))) {
            app.repondre_a_la_question(Some(Reponse::Oui));
        }
        gestes += app.store.undo_depth().saturating_sub(avant);
        std::thread::sleep(Duration::from_millis(10));
    }
    let board = app.store.project.active_board_id.clone();
    let decrits = app
        .store
        .decrire_pour_un_agent(&board)
        .map_err(|r| r.to_string())?;
    let mut cadres: Vec<(String, Rect)> = decrits
        .iter()
        .filter(|d| d.sorte == Sorte::Membrane && d.membrane.is_none())
        .map(|d| (d.id.clone(), d.boite))
        .collect();
    if let Some(tout) = decrits.iter().map(|d| d.boite).reduce(|a, b| a.union(b)) {
        cadres.insert(0, ("tableau".to_string(), tout));
    }
    for (nom, cadre) in cadres {
        let chemin = dossier.join(format!("{nom}.png"));
        rendre(&mut app, &board, cadre, taille, &chemin)?;
        println!("[porte] {}", chemin.display());
    }
    // Une flèche ancrée, survolée : ce qu'elle relie, et ses deux passages qui brillent.
    let ancree = decrits.iter().find(|d| {
        d.sorte == Sorte::Fleche && d.source.is_some() && d.cible.is_some() && d.id.contains("lien")
    });
    if let Some(f) = ancree {
        let bout = |id: &Option<String>| {
            let id = id.as_ref()?;
            decrits.iter().find(|d| &d.id == id).map(|d| d.boite)
        };
        if let (Some(a), Some(b)) = (bout(&f.source), bout(&f.cible)) {
            app.ui.fleche_survolee = Some(f.id.clone());
            let chemin = dossier.join("survol.png");
            rendre(&mut app, &board, a.union(b), taille, &chemin)?;
            println!("[porte] {}", chemin.display());
        }
    }
    Ok(gestes)
}

/// Rend ce rectangle du monde, cadré dans l'image.
fn rendre(
    app: &mut GlucoseApp,
    board: &str,
    cadre: Rect,
    (w, h): (u32, u32),
    chemin: &Path,
) -> Result<(), String> {
    // La bande du haut et la marge : ce qui est cadré ne passe jamais dessous.
    let echelle = (f64::from(w) / cadre.width).min(f64::from(h) / cadre.height) * 0.85;
    let centre = cadre.center();
    app.store.set_viewport(
        board,
        Viewport {
            x: f64::from(w) / 2.0 - centre.x * echelle,
            y: f64::from(h) / 2.0 - centre.y * echelle,
            scale: echelle,
        },
    );
    let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or("une image vide")?;
    let eclairages = app.eclairages();
    super::render_eclaire(
        &mut app.renderer,
        &mut app.ui,
        &app.store,
        &mut pixmap,
        &eclairages,
    );
    let png = pixmap.encode_png().map_err(|e| e.to_string())?;
    std::fs::write(chemin, png).map_err(|e| e.to_string())
}
