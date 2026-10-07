//! **Les poignées d'une sélection, du plus près au plus loin** (POIGNEE-1) : une grille de
//! photos toutes sélectionnées, peinte hors écran à quatre zooms, écrite en PNG pour être
//! regardée. Au dézoom, huit carrés de 9 px entouraient des vignettes plus petites qu'eux
//! (son retour du 07/10).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example apercu_poignees -- <dossier>
//! ```

use glucose_core::store::Store;
use glucose_core::types::BoardImage;
use glucose_desktop::bench;

const ECRAN: (u32, u32) = (900, 700);

fn document() -> Store {
    let mut store = Store::new("apercu");
    let board = store.project.active_board_id.clone();
    for i in 0..300 {
        let (col, rang) = ((i % 20) as f64, (i / 20) as f64);
        // Des formats variés, comme sur un vrai tableau : carrés, paysages, portraits.
        let (w, h) = [(200.0, 200.0), (260.0, 170.0), (160.0, 240.0)][i % 3];
        let id = format!("p{i}");
        let mut photo = BoardImage::new(id.clone(), col * 280.0, rang * 280.0, w, h);
        photo.id = id.clone();
        store.add_image(&board, photo);
    }
    // Après la boucle : poser une image la sélectionne seule.
    store.set_selected_image_ids((0..300).map(|i| format!("p{i}")).collect());
    bench::ouvert(store)
}

fn main() {
    let dossier = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let (w, h) = ECRAN;
    for zoom in [0.08, 0.12, 0.2, 0.5] {
        let mut store = document();
        bench::frame_document(&mut store, zoom, w, h);
        let png = bench::capture(&store, w, h);
        let chemin = format!("{dossier}/poignees-{zoom}.png");
        match std::fs::write(&chemin, &png) {
            Ok(()) => println!("{chemin} -- zoom {zoom}"),
            Err(e) => eprintln!("{chemin} : {e}"),
        }
    }
}
