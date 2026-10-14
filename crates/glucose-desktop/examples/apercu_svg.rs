//! **L'export SVG, hors écran** (P5, fiche 69) : la vitrine, plus deux photos — l'une
//! recadrée, l'autre tournée —, exportée en SVG et en PNG dans le même cadre, pour être
//! regardée côte à côte avant qu'il essaie.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example apercu_svg -- <dossier>
//! ```

use glucose_core::synth;
use glucose_core::types::{BoardImage, Recadrage};
use glucose_desktop::app::GlucoseApp;

fn main() {
    let dossier = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    app.store = glucose_desktop::bench::ouvert(synth::showcase());
    let board = app.store.project.active_board_id.clone();
    let bornes = app.store.content_bounds(&board).expect("une vitrine");
    // Deux photos à droite de la vitrine : un dégradé recadré, un damier tourné.
    let photos = [
        (
            "degrade",
            Recadrage::depuis_les_marges(0.2, 0.1, 0.1, 0.3),
            0.0,
        ),
        (
            "damier",
            Recadrage::depuis_les_marges(0.0, 0.0, 0.0, 0.0),
            0.35,
        ),
    ];
    for (k, (nom, recadrage, rotation)) in photos.into_iter().enumerate() {
        let chemin = format!("{dossier}/{nom}.png");
        image::RgbImage::from_fn(240, 160, |x, y| match nom {
            "degrade" => image::Rgb([x as u8, y as u8, 200]),
            _ => {
                let case = (x / 20 + y / 20) % 2 == 0;
                if case {
                    image::Rgb([230, 230, 230])
                } else {
                    image::Rgb([40, 40, 40])
                }
            }
        })
        .save(&chemin)
        .expect("une photo d'aperçu");
        let x = bornes.left + bornes.width + 200.0;
        let y = bornes.top + 150.0 + k as f64 * 260.0;
        let mut img = BoardImage::new(nom, x, y, 240.0, 160.0);
        (img.original_width, img.original_height) = (240.0, 160.0);
        img.src = Some(chemin);
        img.crop = recadrage;
        img.rotation = rotation;
        app.store.add_image(&board, img);
    }
    app.store.clear_selection();
    let svg = app.rendre_le_svg().expect("un tableau qui n'est pas vide");
    let chemin = format!("{dossier}/vitrine.svg");
    std::fs::write(&chemin, svg).expect("le SVG écrit");
    println!("{chemin}");
    app.exporter_en_image(format!("{dossier}/vitrine.png").into());
    while app.exporte_encore() {
        app.suivre_l_export();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    println!("{dossier}/vitrine.png");
}
