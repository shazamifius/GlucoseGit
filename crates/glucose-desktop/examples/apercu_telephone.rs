//! **Glucose sur un téléphone, hors écran** (fiche 55) : la taille de son Redmi 9 — 720 × 1600
//! pixels, interface à 200 % — debout et couché, le rail replié puis déplié ; et le menu que
//! deux touchers ouvrent sur le vide, à la taille du doigt (fiche 56). Écrit en PNG pour être
//! montré avant qu'il essaie.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example apercu_telephone -- <dossier>
//! ```

use glucose_core::store::Store;
use glucose_core::types::BoardImage;
use glucose_desktop::bench;

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
        ui.context_menu_at = Some((200.0, 700.0));
        ui.menu_au_doigt = true;
    });
    ecrire(
        &format!("{dossier}/telephone-debout-menu-au-doigt.png"),
        &png,
    );
}

fn ecrire(chemin: &str, png: &[u8]) {
    match std::fs::write(chemin, png) {
        Ok(()) => println!("{chemin}"),
        Err(e) => eprintln!("{chemin} : {e}"),
    }
}
