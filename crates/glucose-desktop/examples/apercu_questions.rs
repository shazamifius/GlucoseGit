//! **Les questions que Glucose dessine, au bureau, hors écran** (POPUP-1, fiche 58) : la
//! fenêtre de son portable — 1 920 × 1 200 pixels, interface à 150 % —, et chaque question qui
//! passait par une boîte du système : le travail non enregistré, le journal technique, la mise
//! à jour. Écrit en PNG pour être montré avant qu'il essaie.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example apercu_questions -- <dossier>
//! ```

use glucose_core::store::Store;
use glucose_core::types::BoardImage;
use glucose_desktop::bench;
use glucose_desktop::ui::question::{Question, Reponse, Suite};

const ECRAN: (u32, u32) = (1920, 1200);

fn document() -> Store {
    let mut store = Store::new("apercu");
    let board = store.project.active_board_id.clone();
    for i in 0..12 {
        let (col, rang) = ((i % 4) as f64, (i / 4) as f64);
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
    let questions = [
        ("travail", travail(), 0),
        ("travail-tab", travail(), 1),
        ("journal", journal(), 0),
        ("mise-a-jour", mise_a_jour(), 0),
    ];
    for (nom, mut question, focus) in questions {
        question.focus = focus;
        let mut store = document();
        bench::frame_document(&mut store, 0.8, ECRAN.0, ECRAN.1);
        let png = bench::capture_with(&store, ECRAN.0, ECRAN.1, |ui| {
            ui.scale_factor = 1.5;
            ui.question = Some((question, Suite::NouveauDocument));
        });
        let chemin = format!("{dossier}/bureau-question-{nom}.png");
        match std::fs::write(&chemin, png) {
            Ok(()) => println!("{chemin}"),
            Err(e) => eprintln!("{chemin} : {e}"),
        }
    }
}

/// La croix, sur un travail non enregistré.
fn travail() -> Question {
    Question {
        titre: "Modifications non enregistrées".into(),
        texte: "« Sans titre » contient des modifications non enregistrées.".into(),
        choix: vec![
            ("Enregistrer".into(), Reponse::Oui),
            ("Ne pas enregistrer".into(), Reponse::Non),
            ("Annuler".into(), Reponse::Annuler),
        ],
        ..Default::default()
    }
}

/// Le journal technique, au premier lancement.
fn journal() -> Question {
    Question {
        titre: "Journal technique".into(),
        texte: glucose_desktop::telemetrie::QUESTION.into(),
        choix: vec![("Oui".into(), Reponse::Oui), ("Non".into(), Reponse::Non)],
        ..Default::default()
    }
}

/// La mise à jour, ses notes comprises.
fn mise_a_jour() -> Question {
    Question {
        titre: "Mise à jour de Glucose".into(),
        texte: "Glucose 2.0.2-beta.1 est disponible.\n\nLes questions se dessinent dans \
                Glucose ; le téléphone écrit, et l'appui long fait le clic droit.\n\n\
                L'installer maintenant ? Ton travail s'enregistre, Glucose se ferme, \
                s'installe, puis se rouvre."
            .into(),
        choix: vec![
            ("Installer".into(), Reponse::Oui),
            ("Plus tard".into(), Reponse::Annuler),
        ],
        ..Default::default()
    }
}
