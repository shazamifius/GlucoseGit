//! **Ce que les flèches coûtent à l'image** (FLECHE-1).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_fleches
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! Les flèches ont pris l'aspect de Glucose Tauri : un halo et un trait en dégradé, des
//! pastilles. Un dégradé n'est pas gratuit au processeur, et une flèche en diagonale touche
//! beaucoup de lignes de la couche du dessus pour peu de pixels. Ce banc dit ce que cela
//! coûte, sur la voie processeur, à la taille de sa fenêtre : le poste `annotations` avec les
//! flèches, moins le même poste sans elles — ce que les flèches seules demandent.

use glucose_core::store::Store;
use glucose_core::types::{Annotation, Point2D, Viewport};
use glucose_desktop::params::SceneOverlay;
use glucose_desktop::renderer::{Cadrage, Renderer};
use glucose_desktop::ui::UiState;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (2160, 1350);
const PASSES: usize = 30;
/// Des cartes sur une grille, et des flèches qui les relient.
const COLONNES: usize = 10;
const LIGNES: usize = 6;
const FLECHES: usize = 80;

fn main() {
    println!(
        "\n  {} cartes, {FLECHES} fleches, ecran {} x {} -- mediane de {PASSES} passes\n",
        COLONNES * LIGNES,
        ECRAN.0,
        ECRAN.1
    );
    println!("    vue                      sans fleches   avec fleches   les fleches seules");
    for (nom, echelle) in [("tout le graphe (0,8)", 0.8), ("de pres (2,5)", 2.5)] {
        let sans = poste_annotations(false, echelle);
        let avec = poste_annotations(true, echelle);
        println!(
            "    {nom:<24} {sans:>9.2} ms   {avec:>9.2} ms   {:>9.2} ms",
            avec - sans
        );
    }
    println!();
}

/// Un générateur déterministe, pour que deux passes du banc tracent les mêmes flèches.
fn suivant(etat: &mut u64) -> u64 {
    *etat = etat.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    *etat >> 33
}

fn document(avec_fleches: bool) -> Store {
    let mut store = Store::new("fleches");
    let board = store.project.active_board_id.clone();
    for i in 0..COLONNES * LIGNES {
        let (c, l) = ((i % COLONNES) as f64, (i / COLONNES) as f64);
        store.add_annotation(
            &board,
            Annotation::text(format!("c{i}"), c * 260.0, l * 260.0, "une idee"),
        );
    }
    if avec_fleches {
        let mut graine = 7_u64;
        for k in 0..FLECHES {
            let a = suivant(&mut graine) as usize % (COLONNES * LIGNES);
            let b = (a + 1 + suivant(&mut graine) as usize % (COLONNES * LIGNES - 1))
                % (COLONNES * LIGNES);
            let mut f = Annotation::arrow(format!("f{k}"), 0.0, 0.0, 0.0, 0.0);
            if let Annotation::Arrow {
                source_id,
                target_id,
                arrow_type,
                waypoints,
                ..
            } = &mut f
            {
                *source_id = Some(format!("c{a}"));
                *target_id = Some(format!("c{b}"));
                // Une sur quatre est courbe, par une étape à côté de la corde.
                if k % 4 == 0 {
                    *arrow_type = Some("curved".to_string());
                    let (ca, cb) = ((a % COLONNES) as f64, (b % COLONNES) as f64);
                    *waypoints = vec![Point2D {
                        x: (ca + cb) * 130.0 + 90.0,
                        y: ((a / COLONNES) as f64) * 260.0 - 60.0,
                    }];
                }
            }
            store.add_annotation(&board, f);
        }
    }
    store.clear_selection();
    store
}

fn mediane(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Le poste `annotations` de la voie processeur, la vue centrée sur le graphe.
fn poste_annotations(avec_fleches: bool, echelle: f64) -> f64 {
    let mut renderer = Renderer::new();
    let mut store = document(avec_fleches);
    let board = store.project.active_board_id.clone();
    let (largeur, hauteur) = (COLONNES as f64 * 260.0, LIGNES as f64 * 260.0);
    store.set_viewport(
        &board,
        Viewport {
            x: f64::from(ECRAN.0) / 2.0 - largeur / 2.0 * echelle,
            y: f64::from(ECRAN.1) / 2.0 - hauteur / 2.0 * echelle,
            scale: echelle,
        },
    );
    let mut p = Pixmap::new(ECRAN.0, ECRAN.1).expect("un pixmap");
    let ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let mesures = (0..PASSES)
        .map(|_| {
            glucose_desktop::perf::frame_begin();
            renderer.rendre_la_region(
                &mut p.as_mut(),
                &store,
                &ui,
                SceneOverlay {
                    guides: &guides,
                    selection_box: None,
                    editing: None,
                    arrivages: &[],
                    eclairages: &[],
                },
                ui.header_height(),
                Cadrage::plein(),
            );
            glucose_desktop::perf::postes()
                .into_iter()
                .find(|(n, _)| *n == "annotations")
                .map_or(0.0, |(_, ms)| ms)
        })
        .collect();
    mediane(mesures)
}
