//! **L'export SVG, hors écran** (P5, fiches 69 et 70) : la vitrine de l'export
//! ([`glucose_desktop::bench::vitrine`]), exportée en SVG et en PNG dans le même cadre, pour être
//! regardée côte à côte avant qu'il essaie.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example apercu_svg -- <dossier>
//! ```

use glucose_desktop::bench::vitrine::vitrine_de_l_export;
use std::path::PathBuf;

fn main() {
    let dossier = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".to_string()));
    let mut app = vitrine_de_l_export(&dossier);
    let svg = app.rendre_le_svg().expect("un tableau qui n'est pas vide");
    let chemin = dossier.join("vitrine.svg");
    std::fs::write(&chemin, svg).expect("le SVG écrit");
    println!("{}", chemin.display());
    app.exporter_en_image(dossier.join("vitrine.png"));
    while app.exporte_encore() {
        app.suivre_l_export();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    println!("{}", dossier.join("vitrine.png").display());
}
