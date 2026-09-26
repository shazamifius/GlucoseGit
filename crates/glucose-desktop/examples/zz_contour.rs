//! TEMPORAIRE — regarder un détour de flèche (FLECHE-5). À supprimer avant le commit.
use glucose_core::store::Store;
use glucose_core::types::{Annotation, BoardImage};

fn carte(id: &str, x: f64, y: f64, w: f64, h: f64, texte: &str) -> Annotation {
    let mut a = Annotation::text(id, x, y, texte);
    if let Annotation::Text { width, height, .. } = &mut a {
        *width = Some(w);
        *height = Some(h);
    }
    a
}

fn fleche(id: &str, s: &str, t: &str, courbe: bool) -> Annotation {
    let mut f = Annotation::arrow(id, 0.0, 0.0, 0.0, 0.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        arrow_type,
        ..
    } = &mut f
    {
        *source_id = Some(s.into());
        *target_id = Some(t.into());
        if courbe {
            *arrow_type = Some("curved".into());
        }
    }
    f
}

fn main() {
    let dossier = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    for courbe in [false, true] {
        let mut store = Store::new("contour");
        let board = store.project.active_board_id.clone();
        if let Some(b) = store.active_board_mut() {
            b.annotations.clear();
            b.images.clear();
            b.folders.clear();
            b.viewport.x = 60.0;
            b.viewport.y = 170.0;
            b.viewport.scale = 0.8;
        }
        for a in [
            carte("s", 0.0, 0.0, 160.0, 80.0, "Départ"),
            carte("t", 900.0, 20.0, 160.0, 80.0, "Arrivée"),
            carte("o", 380.0, -60.0, 160.0, 200.0, "Un obstacle sur le trajet"),
            carte("s2", 0.0, 300.0, 160.0, 80.0, "Autre départ"),
            carte("t2", 900.0, 300.0, 160.0, 80.0, "Autre arrivée"),
        ] {
            store.add_annotation(&board, a);
        }
        if let Some(b) = store.active_board_mut() {
            let mut img = BoardImage::new("i1", 400.0, 340.0, 180.0, 120.0);
            img.rotation = 0.3;
            b.images.push(img);
            b.images.push(BoardImage::new("i2", 650.0, 330.0, 140.0, 200.0));
        }
        store.add_annotation(&board, fleche("f1", "s", "t", courbe));
        store.add_annotation(&board, fleche("f2", "s2", "t2", courbe));
        store.clear_selection();
        let png = glucose_desktop::bench::capture(&store, 1200, 600);
        let nom = format!("{dossier}/contour-{}.png", if courbe { "courbe" } else { "droite" });
        std::fs::write(&nom, png).expect("écrit");
        println!("{nom}");
    }
}
