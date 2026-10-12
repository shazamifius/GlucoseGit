//! **Exporter un vrai document en image**, par le chemin même du bouton « Exporter »
//! (EXPORT-PNG-1) — pour regarder ce qu'il donne et le chronométrer.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example essai_export -- copie.glucose sortie.png
//! ```
//!
//! À jouer sur une **copie** de ses documents : l'application d'essai ouvre le document comme
//! Glucose l'ouvrirait.

use glucose_desktop::app::GlucoseApp;
use std::time::{Duration, Instant};

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(document), Some(sortie)) = (args.next(), args.next()) else {
        eprintln!("usage : essai_export <copie.glucose> <sortie.png>");
        return;
    };
    let mut app = GlucoseApp::new();
    app.open_from(document.into());
    // Le tableau qui a le plus d'images : c'est lui qu'on veut regarder.
    if let Some(plein) = app
        .store
        .project
        .boards
        .iter()
        .max_by_key(|b| b.images.len())
        .map(|b| b.id.clone())
    {
        app.store.project.active_board_id = plein;
    }
    let debut = Instant::now();
    app.exporter_vers(sortie.into());
    while app.exporte_encore() {
        app.suivre_l_export();
        std::thread::sleep(Duration::from_millis(20));
    }
    let message = app
        .ui
        .current_toast
        .as_ref()
        .map_or("(rien)", |t| t.message.as_str());
    println!("{message} — en {:.2} s", debut.elapsed().as_secs_f64());
}
