//! Le banc de frame — travail A.5 du plan de marche, publié en chiffres.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_frame
//! cargo run --release -p glucose-desktop --example bench_frame -- 1080p
//! ```
//!
//! Mesure le coût d'une frame complète — scène et interface — sur des documents synthétiques
//! reproductibles, à trois définitions et trois zooms. Le budget est de 10 ms : cent images par
//! seconde, sur n'importe quelle machine.
//!
//! Les chiffres sortent en `debug` comme en `release`, mais seuls ceux de `release` valent
//! quelque chose : le profil de développement compile notre propre code en `opt-level = 1`.

use glucose_core::synth::{self, Shape};
use glucose_desktop::bench::{self, BUDGET_MS, DEFINITIONS};

/// Les tailles de document mesurées. Au-delà, ce n'est plus le rendu qu'on mesure mais la
/// construction du document — l'index spatial, lui, a son propre banc (`bench_arena`).
const TAILLES: &[usize] = &[1_000, 10_000, 100_000];

/// Les zooms mesurés. À 1, on lit un document ; à 0,01, on le survole en entier — et c'est le
/// cas difficile, puisque tout devient visible d'un coup.
const ZOOMS: &[f64] = &[1.0, 0.1, 0.01];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let definitions: Vec<_> = if args.is_empty() {
        DEFINITIONS.to_vec()
    } else {
        DEFINITIONS.iter().filter(|(nom, _, _)| args.iter().any(|a| a == nom)).copied().collect()
    };
    if definitions.is_empty() {
        eprintln!("definitions connues : {:?}", DEFINITIONS.iter().map(|d| d.0).collect::<Vec<_>>());
        return;
    }

    println!("Banc de frame — budget {BUDGET_MS} ms ({} fps)", (1000.0 / BUDGET_MS) as u32);
    println!(
        "{:>8}  {:>8}  {:>6}  {:>9}  {:>9}  {:>9}  {:>7}  budget",
        "def", "noeuds", "zoom", "min", "mediane", "max", "fps"
    );

    let mut hors_budget = 0usize;
    for (nom, w, h) in &definitions {
        for &n in TAILLES {
            // Le document est fabriqué une fois par taille : le regénérer à chaque zoom
            // mesurerait la génération autant que le rendu.
            let base = synth::document(n, span_pour(n), Shape::Clustered, 0x91ac05e);
            for &zoom in ZOOMS {
                let mut store = base.clone();
                bench::frame_document(&mut store, zoom, *w, *h);
                let s = bench::measure(&store, *w, *h, repetitions(n));
                if !s.within_budget() {
                    hors_budget += 1;
                }
                println!(
                    "{nom:>8}  {n:>8}  {zoom:>6}  {:>7.2}ms  {:>7.2}ms  {:>7.2}ms  {:>7.0}  {}",
                    s.min_ms,
                    s.median_ms,
                    s.max_ms,
                    s.fps(),
                    if s.within_budget() { "tenu" } else { "DEPASSE" }
                );
            }
        }
    }

    println!();
    if hors_budget == 0 {
        println!("Toutes les mesures tiennent le budget.");
    } else {
        println!("{hors_budget} mesure(s) hors budget.");
    }
}

/// L'étendue du document croît avec sa taille, pour garder une densité comparable : un
/// document dix fois plus peuplé sur la même surface ne mesurerait que l'entassement.
fn span_pour(n: usize) -> f64 {
    2_000.0 * (n as f64).sqrt()
}

/// Moins de répétitions sur les gros documents : la mesure y est plus stable, et une frame y
/// coûte assez cher pour que vingt tours soient longs sans rien apprendre de plus.
fn repetitions(n: usize) -> usize {
    if n >= 100_000 {
        10
    } else {
        30
    }
}
