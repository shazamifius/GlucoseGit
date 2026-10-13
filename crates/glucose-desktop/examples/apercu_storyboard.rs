//! **Le Storyboard, hors écran** (STORYBOARD-1, fiche 64) : des cases décrites, des images
//! ajustées dans deux d'entre elles, une case choisie, le panneau ouvert ; puis la grille armée
//! qui suit le curseur. Écrit en PNG pour être regardé avant qu'il essaie.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example apercu_storyboard -- <dossier>
//! ```

use glucose_core::store::storyboard::{Format, Grille};
use glucose_core::types::{BoardImage, Viewport};
use glucose_desktop::app::GlucoseApp;
use glucose_desktop::dock::TabId;

const ECRAN: (u32, u32) = (1600, 1000);

fn grille() -> Grille {
    Grille {
        format: Format::Cinema,
        largeur: 280.0,
        colonnes: 4,
        ecart: 24.0,
    }
}

/// Une application au tableau vide, vue à l'échelle 1 depuis l'origine.
fn application() -> (GlucoseApp, String) {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    app.store.clear_selection();
    let board = app.store.project.active_board_id.clone();
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    app.store.set_viewport(&board, vue);
    (app, board)
}

fn main() {
    let dossier = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let (mut app, board) = application();
    app.store
        .poser_le_storyboard(&board, grille(), (980.0, 420.0));
    let cases: Vec<_> = app.store.cases(&board).into_iter().cloned().collect();
    let textes = [
        "Plan large, la ville au loin",
        "Elle ouvre la porte",
        "Gros plan : la clé",
    ];
    for (case, texte) in cases.iter().zip(textes) {
        app.store.update_panel(&board, &case.id, texte);
    }
    for (i, (w, h)) in [(400.0, 300.0), (300.0, 420.0)].into_iter().enumerate() {
        let c = &cases[i];
        let id = format!("photo-{i}");
        let image = BoardImage::new(id.clone(), c.x + 20.0, c.y + 20.0, w, h);
        app.store.add_image(&board, image);
        app.store.ajuster_dans_les_cases(&board, &[id]);
    }
    app.store.clear_selection();
    app.ui.storyboard.choisie = Some(cases[2].id.clone());
    app.dock_manager.toggle_tab(TabId::Storyboard);
    ecrire(&mut app, &format!("{dossier}/storyboard.png"));

    // La grille armée, qui suit le curseur sur un tableau sans cases.
    let (mut app, _) = application();
    app.ui.placement.storyboard = Some(grille());
    app.mouse_pos = (800.0, 500.0);
    ecrire(&mut app, &format!("{dossier}/storyboard-fantome.png"));
}

fn ecrire(app: &mut GlucoseApp, chemin: &str) {
    app.une_image_sans_fenetre(ECRAN);
    let Some(png) = app.pixmap.as_ref().and_then(|p| p.encode_png().ok()) else {
        eprintln!("aucune image");
        return;
    };
    match std::fs::write(chemin, png) {
        Ok(()) => println!("{chemin}"),
        Err(e) => eprintln!("{chemin} : {e}"),
    }
}
