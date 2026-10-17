//! **Glucose sans fenêtre, sa porte ouverte** (fiche 75), pour regarder ce qu'un agent y pose.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example porte_sans_fenetre -- <porte> <arrêt> <dossier>
//! ```
//!
//! Il sert la porte jusqu'à ce que le fichier `<arrêt>` existe, puis écrit dans `<dossier>` le
//! tableau entier et chacune de ses membranes en PNG (2560 × 1600, l'écran de son PC). Un agent
//! s'y branche par `glucose --mcp`, avec `GLUCOSE_PORTE=<porte>`. Rien ne s'ouvre à l'écran.

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(porte), Some(arret), Some(dossier)) = (args.next(), args.next(), args.next()) else {
        eprintln!("usage : porte_sans_fenetre <porte> <arrêt> <dossier>");
        std::process::exit(2);
    };
    let _ = std::fs::create_dir_all(&dossier);
    match glucose_desktop::bench::porte::servir_sans_fenetre(
        &porte,
        std::path::Path::new(&arret),
        std::path::Path::new(&dossier),
        (2560, 1600),
    ) {
        Ok(gestes) => println!("[porte] {gestes} gestes reçus"),
        Err(e) => {
            eprintln!("[porte] {e}");
            std::process::exit(1);
        }
    }
}
