//! **Glucose sur un téléphone, hors écran** (fiche 55) : la taille de son Redmi 9 — 720 × 1600
//! pixels, interface à 200 % — debout et couché, le rail replié puis déplié ; et le menu que
//! deux touchers ouvrent sur le vide, à la taille du doigt (fiche 56). Écrit en PNG pour être
//! montré avant qu'il essaie.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example apercu_telephone -- <dossier>
//! ```

use glucose_core::store::Store;
use glucose_core::text::Selection;
use glucose_core::types::BoardImage;
use glucose_desktop::bench;
use glucose_desktop::plateforme::marges::Marges;

/// **Les marges d'un Redmi 9 en bord à bord** (BORD-1, fiche 57) : la barre d'état en haut,
/// la navigation en bas ; couché, la barre d'état à gauche de l'encoche n'existe pas.
fn marges(w: u32, clavier: f32) -> Marges {
    let debout = w < 1000;
    Marges {
        haut: if debout { 60.0 } else { 0.0 },
        gauche: if debout { 0.0 } else { 60.0 },
        bas: 96.0,
        clavier,
        ..Marges::default()
    }
}
use glucose_desktop::ui::question::{Champ, Question, Reponse, Suite};

fn document() -> Store {
    let mut store = Store::new("apercu");
    let board = store.project.active_board_id.clone();
    for i in 0..12 {
        let (col, rang) = ((i % 3) as f64, (i / 3) as f64);
        let (w, h) = [(200.0, 200.0), (260.0, 170.0), (160.0, 240.0)][i % 3];
        store.add_image(
            &board,
            BoardImage::new(format!("p{i}"), col * 290.0, rang * 290.0, w, h),
        );
    }
    store.clear_selection();
    bench::ouvert(store)
}

fn main() {
    let dossier = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    for (nom, (w, h)) in [("debout", (720, 1600)), ("couche", (1600, 720))] {
        for ouvert in [false, true] {
            let mut store = document();
            bench::frame_document(&mut store, 0.6, w, h);
            let png = bench::capture_with(&store, w, h, |ui| {
                ui.scale_factor = 2.0;
                ui.marges = marges(w, 0.0);
                ui.rail.ouvert = ouvert;
            });
            let etat = if ouvert { "deplie" } else { "replie" };
            let chemin = format!("{dossier}/telephone-{nom}-{etat}.png");
            ecrire(&chemin, &png);
        }
    }
    // Le menu du vide, ouvert par deux touchers au milieu de l'écran (fiche 56).
    let mut store = document();
    bench::frame_document(&mut store, 0.6, 720, 1600);
    let png = bench::capture_with(&store, 720, 1600, |ui| {
        ui.scale_factor = 2.0;
        ui.marges = marges(720, 0.0);
        ui.context_menu_at = Some((200.0, 700.0));
        ui.menu_au_doigt = true;
    });
    ecrire(
        &format!("{dossier}/telephone-debout-menu-au-doigt.png"),
        &png,
    );
    // La liste des documents, telle que le téléphone la dessine (fiche 56, DOCUMENTS-1).
    // Et ce qu'un appui long y demande (fiche 57, DOCUMENTS-2). « Renommer » se rend le clavier
    // sorti : en bord à bord, il recouvre le bas de la fenêtre (740 pixels de Gboard).
    let ecrans = [
        ("documents", liste(), 0.0),
        ("question", nouveau(), 0.0),
        ("gerer", gerer(), 0.0),
        ("renommer", renommer(), 740.0),
        ("supprimer", supprimer(), 0.0),
    ];
    for (nom, question, clavier) in ecrans {
        let mut store = document();
        bench::frame_document(&mut store, 0.6, 720, 1600);
        let png = bench::capture_with(&store, 720, 1600, |ui| {
            ui.scale_factor = 2.0;
            ui.marges = marges(720, clavier);
            ui.question = Some((question, Suite::Ouvrir(Vec::new())));
        });
        ecrire(&format!("{dossier}/telephone-debout-{nom}.png"), &png);
    }
}

/// La question de l'appui long sur un document.
fn gerer() -> Question {
    Question {
        titre: "« Canevas 2 »".into(),
        choix: vec![
            ("Renommer…".into(), Reponse::Choix(0)),
            ("Dupliquer".into(), Reponse::Choix(1)),
            ("Supprimer…".into(), Reponse::Choix(2)),
            ("Annuler".into(), Reponse::Annuler),
        ],
        ..Default::default()
    }
}

/// Renommer : le nom tout sélectionné, que la première lettre tapée remplace.
fn renommer() -> Question {
    Question {
        titre: "Renommer".into(),
        champ: Some(Champ {
            texte: "Canevas 2".into(),
            selection: Selection::all("Canevas 2"),
        }),
        choix: vec![
            ("Renommer".into(), Reponse::Oui),
            ("Annuler".into(), Reponse::Annuler),
        ],
        ..Default::default()
    }
}

/// Supprimer : définitif, et dit.
fn supprimer() -> Question {
    Question {
        titre: "Supprimer « Canevas 2 » ?".into(),
        texte: "C'est définitif : le document, son histoire et ses images disparaissent de ce \
                téléphone."
            .into(),
        choix: vec![
            ("Supprimer".into(), Reponse::Oui),
            ("Annuler".into(), Reponse::Annuler),
        ],
        ..Default::default()
    }
}

/// La liste de quatre documents rangés.
fn liste() -> Question {
    let mut choix: Vec<(String, Reponse)> =
        ["Canevas 3", "Planches Mary", "Canevas 2", "Canevas 1"]
            .iter()
            .enumerate()
            .map(|(i, n)| (n.to_string(), Reponse::Choix(i)))
            .collect();
    choix.push(("Annuler".into(), Reponse::Annuler));
    Question {
        titre: "Documents".into(),
        texte: "Un appui long sur l'un d'eux : le renommer, le dupliquer, le supprimer.".into(),
        choix,
        ..Default::default()
    }
}

/// La question de « Nouveau document ».
fn nouveau() -> Question {
    Question {
        titre: "Créer un nouveau document ?".into(),
        texte: "« Canevas 2 » est enregistré : il reste où il est, et se rouvre par Ouvrir.".into(),
        choix: vec![
            ("Créer".into(), Reponse::Oui),
            ("Annuler".into(), Reponse::Non),
        ],
        ..Default::default()
    }
}

fn ecrire(chemin: &str, png: &[u8]) {
    match std::fs::write(chemin, png) {
        Ok(()) => println!("{chemin}"),
        Err(e) => eprintln!("{chemin} : {e}"),
    }
}
