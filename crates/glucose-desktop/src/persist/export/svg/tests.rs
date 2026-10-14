//! L'export SVG (P5, fiche 69) — première épreuve : une photo y porte ses octets d'origine,
//! découpée à sa boîte ; un tableau vide n'en donne pas. Le reste (membranes, cartes,
//! flèches, regardé dans un lecteur, saboté) est à faire : voir la fiche 69.

use crate::app::GlucoseApp;
use crate::epreuve::Dossier;
use glucose_core::types::BoardImage;

/// **Une photo porte ses octets d'origine**, en `data:`, découpée à sa boîte.
#[test]
fn test_p5_une_photo_porte_ses_octets_d_origine() {
    let dossier = Dossier::nouveau("export-svg");
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    assert_eq!(
        app.rendre_le_svg(),
        None,
        "un tableau vide n'a rien à écrire"
    );
    let fichier = dossier.join("rouge.png");
    image::RgbaImage::from_pixel(10, 10, image::Rgba([220, 30, 30, 255]))
        .save(&fichier)
        .expect("une photo d'épreuve");
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new("rouge", 0.0, 0.0, 100.0, 100.0);
    (img.original_width, img.original_height) = (10.0, 10.0);
    img.src = Some(fichier.to_string_lossy().into_owned());
    app.store.add_image(&board, img);
    app.store.clear_selection();
    let svg = app.rendre_le_svg().expect("un SVG");
    let octets = std::fs::read(&fichier).expect("la photo");
    let attendu = format!(
        "data:image/png;base64,{}",
        crate::renderer::svg::base64(&octets)
    );
    assert!(svg.contains(&attendu), "les octets d'origine");
    assert!(svg.contains("<clipPath"), "découpée à sa boîte");
    assert!(svg.starts_with("<?xml") && svg.trim_end().ends_with("</svg>"));
}
