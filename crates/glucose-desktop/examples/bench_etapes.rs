//! Décompose le coût d'une frame par étape de rendu.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_etapes
//! ```
//!
//! Le banc global ([`bench_frame`](../bench_frame/index.html)) dit qu'une frame dépasse son
//! budget ; celui-ci dit **où**. C'est la différence entre savoir qu'il faut optimiser et savoir
//! quoi.
//!
//! La question qu'il existe pour trancher : à 4K, mille nœuds suffisent à dépasser les 10 ms.
//! Si le coût est dans le fond, la grille et le transfert, il est proportionnel à la **surface**
//! et seule une couche de présentation le règle. S'il est dans les halos ou les cartes, il est
//! proportionnel au **contenu** et se règle par l'algorithme.

use glucose_core::synth::{self, Shape};
use glucose_desktop::bench;
use glucose_desktop::perf;
use glucose_desktop::renderer::Renderer;
use glucose_desktop::ui::UiState;
use tiny_skia::Pixmap;

fn main() {
    if !perf::enabled() {
        eprintln!("Relancer avec GLUCOSE_PERF=1 pour obtenir le detail par etape.");
        eprintln!("  GLUCOSE_PERF=1 cargo run --release -p glucose-desktop --example bench_etapes");
        return;
    }

    for (nom, w, h) in bench::DEFINITIONS {
        for n in [1_000usize, 10_000] {
            let mut store = synth::document(n, 2_000.0 * (n as f64).sqrt(), Shape::Clustered, 0x91ac05e);
            bench::frame_document(&mut store, 1.0, *w, *h);

            let mut renderer = Renderer::new();
            let mut ui = UiState::new();
            let mut pixmap = Pixmap::new(*w, *h).expect("un pixmap");
            // Une frame de chauffe, hors mesure : elle remplit les caches.
            bench::render_into(&mut renderer, &mut ui, &store, &mut pixmap);

            eprintln!("=== {nom} {n} noeuds ===");
            for _ in 0..3 {
                perf::frame_begin();
                bench::render_into(&mut renderer, &mut ui, &store, &mut pixmap);
                perf::frame_end();
            }
        }
    }
}
