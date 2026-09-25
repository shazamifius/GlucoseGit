//! **Ce que coûtent les ornements d'une sélection de photos**, par la voie graphique.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_ornements
//! ```
//!
//! # La question
//!
//! La longue session du 23/09 au soir — 243 nœuds, dont 45 épingles rapatriées — montre ses
//! images les plus lentes dans le poste `ornements` : 15 à 28 ms, au repos comme en mouvement.
//! Et le tempo monte alors à cinq balayages par image, 48 images par seconde, sous le plancher.
//!
//! Les ornements d'une photo sélectionnée sont un cadre et huit poignées : dix-sept appels au
//! rastériseur. La supposition est que c'est leur **nombre** qui coûte, pas leur surface. Ce
//! banc la vérifie avant qu'on y touche : autant de photos, toutes sélectionnées, toutes à
//! l'écran, sur l'écran de l'utilisateur.

use glucose_core::store::Store;
use glucose_core::types::{BoardImage, Viewport};
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::renderer::{Regard, Renderer};
use glucose_desktop::ui::UiState;
use tiny_skia::Pixmap;

/// L'écran de l'utilisateur : 2160 × 1350, interface à 150 %.
const ECRAN: (u32, u32) = (2160, 1350);
const TOURS: usize = 30;

fn main() {
    println!(
        "\n  Les ornements d'une selection, ecran {} x {}\n",
        ECRAN.0, ECRAN.1
    );
    println!("    photos selectionnees    ornements (mediane)    par photo");
    for n in [0usize, 1, 50, 100, 243] {
        let ms = mesurer(n);
        let par = if n > 0 { ms * 1000.0 / n as f64 } else { 0.0 };
        println!("    {n:>20}    {ms:>13.2} ms    {par:>7.1} us");
    }
    println!();
}

/// `n` photos en grille, toutes sélectionnées et à l'écran ; la médiane du poste `ornements`.
fn mesurer(n: usize) -> f64 {
    let mut renderer = Renderer::new();
    let mut store = Store::new("Ornements");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
    }
    let colonnes = 18;
    for i in 0..n.max(1) {
        let (c, r) = ((i % colonnes) as f64, (i / colonnes) as f64);
        store.add_image(
            &board,
            BoardImage::new(
                format!("p{i}"),
                60.0 + c * 115.0,
                140.0 + r * 85.0,
                100.0,
                70.0,
            ),
        );
    }
    store.selected_image_ids = (0..n).map(|i| format!("p{i}")).collect();
    store.set_viewport(
        &board,
        Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        },
    );
    renderer.sync_spatial_index(&store);
    let mut dessous = Pixmap::new(ECRAN.0, ECRAN.1).expect("un pixmap");
    let mut dessus = Pixmap::new(ECRAN.0, ECRAN.1).expect("un pixmap");
    let mut ui = UiState::new();
    ui.scale_factor = 1.5;
    let guides = glucose_core::smart_align::SnapGuides::default();
    let mut mesures = Vec::with_capacity(TOURS);
    for _ in 0..TOURS {
        dessus.fill(tiny_skia::Color::TRANSPARENT);
        glucose_desktop::perf::frame_begin();
        renderer.rendre_les_couches(
            &mut dessous.as_mut(),
            &mut dessus.as_mut(),
            &store,
            (&mut ui, Pointer { x: 0.0, y: 0.0 }),
            SceneOverlay::sans_rien(&guides),
            Regard::immobile(),
        );
        let ornements = glucose_desktop::perf::postes()
            .into_iter()
            .find(|(nom, _)| *nom == "ornements")
            .map_or(0.0, |(_, ms)| ms);
        mesures.push(ornements);
    }
    mesures.sort_by(f64::total_cmp);
    mesures[TOURS / 2]
}
