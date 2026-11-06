//! **Le banc des nœuds choisis** (fiche 86) : ce que coûte une image selon combien de textes sont
//! à l'écran, et combien sont choisis.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_noeuds_choisis
//! ```
//!
//! Son essai du 6/11, en texte seul : `annotations` à 378 ms et `lueurs` à 716 ms pour une image.
//! Trois colonnes : rien de choisi, tout choisi, et le document qui change à chaque image (ce qui
//! arrive quand on glisse) : si une colonne croît comme le carré du nombre de nœuds, elle se voit.

use glucose_core::store::Store;
use glucose_desktop::bench;
use glucose_desktop::interactions::tools::text_card;
use glucose_desktop::renderer::Renderer;
use glucose_desktop::ui::UiState;
use std::time::Instant;
use tiny_skia::Pixmap;

const IMAGES: usize = 12;
const ECRAN: (u32, u32) = (1920, 1080);

fn tableau(n: usize, ecart: (f64, f64)) -> Store {
    let renderer = Renderer::new();
    let mut store = Store::new("banc");
    let board = store.project.active_board_id.clone();
    let colonnes = (n as f64).sqrt().ceil() as usize;
    for i in 0..n {
        let carte = text_card(
            &renderer.typography,
            &renderer.math,
            format!("carte-{i}"),
            (i % colonnes) as f64 * ecart.0,
            (i / colonnes) as f64 * ecart.1,
            format!("Carte {i} — une note posée sur le mur"),
        );
        store.add_annotation(&board, carte);
    }
    store.clear_selection();
    store
}

/// La médiane d'une image quand le document change à chaque image.
fn en_changeant(store: &mut Store) -> f64 {
    let mut renderer = Renderer::new();
    // La tranche du vrai lancement (TEINTES-1).
    renderer.hue_cache.accorder_la_tranche(Some(
        glucose_desktop::cadence::BUDGET_TOTAL.saturating_sub(glucose_desktop::cadence::MARGE),
    ));
    let mut ui = UiState::new();
    let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("un tampon");
    bench::render_into(&mut renderer, &mut ui, store, &mut pixmap);
    let mut temps = Vec::with_capacity(IMAGES);
    for _ in 0..IMAGES {
        store.bump_version();
        let t = Instant::now();
        bench::render_into(&mut renderer, &mut ui, store, &mut pixmap);
        temps.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    temps.sort_by(f64::total_cmp);
    temps[temps.len() / 2]
}

fn main() {
    println!("Texte, 1920 × 1080, tout le tableau à l'écran ; médiane de {IMAGES} images\n");
    println!(
        "{:>8} {:>16} {:>14} {:>14} {:>18}",
        "nœuds", "serrés ou non", "rien choisi", "tout choisi", "document qui change"
    );
    for (nom, ecart) in [("écartés", (320.0, 180.0)), ("serrés", (60.0, 40.0))] {
        for n in [500usize, 1000, 2000, 4000] {
            let mut store = tableau(n, ecart);
            bench::frame_document(&mut store, 1.0, ECRAN.0, ECRAN.1);
            let rien = bench::measure(&store, ECRAN.0, ECRAN.1, IMAGES).median_ms;
            let ids = store
                .active_board()
                .map(|b| b.annotations.iter().map(|a| a.id().to_string()).collect())
                .unwrap_or_default();
            store.set_selected_annotation_ids(ids);
            let tout = bench::measure(&store, ECRAN.0, ECRAN.1, IMAGES).median_ms;
            store.clear_selection();
            let change = en_changeant(&mut store);
            println!("{n:>8} {nom:>16} {rien:>11.1} ms {tout:>11.1} ms {change:>15.1} ms");
        }
    }
}
