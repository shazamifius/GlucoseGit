//! **Ce qu'une membrane coûte à l'image**, avant et après MEMB-FORME-1.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_membranes
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! La chronique du 25/09 : sur son document de cent trente-neuf nœuds, dès qu'une membrane
//! remplissait l'écran, le poste `membranes` coûtait **23 ms en médiane, 48 au pire**, et la
//! couche du dessous partait entière — `effacer` 2,4 ms, `blit` 9,7 ms.
//!
//! Il mesure, à la taille de sa fenêtre (2160 × 1350) :
//!
//! * **l'ancien tracé** — trois remplissages anticrénelés et un pointillé par `tiny-skia`,
//!   recopié ici pour la mesure seulement : il n'existe plus dans l'application ;
//! * **la voie processeur** d'aujourd'hui : le poste `membranes` de la scène, lu comme la
//!   chronique le lit — ce que la loi coûte au processeur, par le code qui tourne ;
//! * **la voie graphique** : ce que la couche du dessous porte encore — les lignes que son
//!   relevé trouve, donc ce qui s'efface et part sur le bus.

use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use glucose_desktop::params::SceneOverlay;
use glucose_desktop::renderer::{Cadrage, Renderer};
use glucose_desktop::ui::UiState;
use std::time::Instant;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, StrokeDash, Transform};

const ECRAN: (u32, u32) = (2160, 1350);
const PASSES: usize = 20;

/// Une membrane de 2 000 × 1 300 unités, avec un titre : ce qu'il a posé.
const MEMBRANE: (f64, f64, f64, f64) = (0.0, 0.0, 2000.0, 1300.0);

fn main() {
    println!(
        "\n  Une membrane, sur un ecran de {} x {} -- mediane de {PASSES} passes\n",
        ECRAN.0, ECRAN.1
    );
    println!(
        "    vue                          ancien trace   voie processeur   dessous (graphique)"
    );
    for (nom, vue) in vues() {
        let avant = ancien_trace(vue);
        let processeur = voie_processeur(vue);
        let (lignes, ms) = voie_graphique(vue);
        println!(
            "    {nom:<28} {avant:>9.2} ms      {processeur:>7.2} ms      {lignes:>4} lignes, {ms:.2} ms"
        );
    }
    println!(
        "\n  « voie processeur » : le poste `membranes` de la scene, forme et ornements.\n  \
         « dessous » : les lignes que la couche du dessous porte encore sur la voie graphique\n  \
         (titre, poignees), et ce que la rendre et la relever coute ; avant, elle partait\n  \
         entiere des qu'une membrane etait visible : {} lignes.\n",
        ECRAN.1
    );
}

/// Les vues du banc : entière, zoomée jusqu'à remplir l'écran, et de très près sur un coin.
fn vues() -> [(&'static str, Viewport); 3] {
    let (w, h) = (f64::from(ECRAN.0), f64::from(ECRAN.1));
    let centre = |echelle: f64| Viewport {
        x: w / 2.0 - (MEMBRANE.0 + MEMBRANE.2 / 2.0) * echelle,
        y: h / 2.0 - (MEMBRANE.1 + MEMBRANE.3 / 2.0) * echelle,
        scale: echelle,
    };
    [
        ("entiere (echelle 0,9)", centre(0.9)),
        ("remplit l'ecran (echelle 4)", centre(4.0)),
        (
            "son coin, de pres (echelle 40)",
            Viewport {
                x: 300.0,
                y: 300.0,
                scale: 40.0,
            },
        ),
    ]
}

fn document(avec_membrane: bool) -> Store {
    let mut store = Store::new("membranes");
    let board = store.project.active_board_id.clone();
    if avec_membrane {
        let (x, y, l, h) = MEMBRANE;
        let mut m = Annotation::membrane("m", x, y, l, h);
        if let Annotation::Membrane { text, .. } = &mut m {
            *text = Some("Recherche".to_string());
        }
        store.add_annotation(&board, m);
    }
    store.clear_selection();
    store
}

fn mediane(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Le poste `membranes` de la scène sur la voie processeur, lu comme la chronique le lit.
fn voie_processeur(vue: Viewport) -> f64 {
    let mut renderer = Renderer::new();
    let mut store = document(true);
    let board = store.project.active_board_id.clone();
    store.set_viewport(&board, vue);
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
                rien(&guides),
                ui.header_height(),
                Cadrage::plein(),
            );
            poste("membranes")
        })
        .collect();
    mediane(mesures)
}

/// La durée d'un poste de l'image en cours.
fn poste(nom: &str) -> f64 {
    glucose_desktop::perf::postes()
        .into_iter()
        .find(|(n, _)| *n == nom)
        .map_or(0.0, |(_, ms)| ms)
}

/// Ce que la couche du dessous porte sur la voie graphique, et ce que la rendre coûte.
fn voie_graphique(vue: Viewport) -> (u32, f64) {
    let mut renderer = Renderer::new();
    let mut store = document(true);
    let board = store.project.active_board_id.clone();
    store.set_viewport(&board, vue);
    let mut dessous = Pixmap::new(ECRAN.0, ECRAN.1).expect("un pixmap");
    let ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let mut lignes = 0;
    let mesures = (0..PASSES)
        .map(|_| {
            dessous.fill(Color::TRANSPARENT);
            let debut = Instant::now();
            renderer.rendre_la_region(
                &mut dessous.as_mut(),
                &store,
                &ui,
                rien(&guides),
                ui.header_height(),
                Cadrage {
                    couche: glucose_desktop::renderer::Couche::Dessous,
                    ..Cadrage::plein()
                },
            );
            let bandes = glucose_desktop::present::bandes::Bandes::relever(&dessous);
            lignes = bandes.lignes();
            debut.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    (lignes, mediane(mesures))
}

fn rien(guides: &glucose_core::smart_align::SnapGuides) -> SceneOverlay<'_> {
    SceneOverlay::sans_rien(guides)
}

/// **L'ancien tracé**, recopié pour la mesure : deux halos et le fond en rectangles aux coins
/// paraboliques, puis le pointillé — quatre passes de `tiny-skia` sur la surface de la
/// membrane.
fn ancien_trace(vue: Viewport) -> f64 {
    let e = vue.scale as f32;
    let (x, y) = (
        (MEMBRANE.0 * vue.scale + vue.x) as f32,
        (MEMBRANE.1 * vue.scale + vue.y) as f32,
    );
    let (l, h) = (MEMBRANE.2 as f32 * e, MEMBRANE.3 as f32 * e);
    let r = 60.0 * e;
    let mut p = Pixmap::new(ECRAN.0, ECRAN.1).expect("un pixmap");
    let mesures = (0..PASSES)
        .map(|_| {
            p.fill(Color::BLACK);
            let debut = Instant::now();
            for (pad, alpha) in [(20.0 * e, 8), (10.0 * e, 14)] {
                remplir(
                    &mut p,
                    (x - pad, y - pad, l + 2.0 * pad, h + 2.0 * pad),
                    r + pad / 2.0,
                    alpha,
                );
            }
            remplir(&mut p, (x, y, l, h), r, 8);
            let mut pb = PathBuilder::new();
            arrondi(&mut pb, (x, y, l, h), r);
            if let Some(chemin) = pb.finish() {
                let mut paint = Paint::default();
                paint.set_color(Color::from_rgba8(96, 165, 250, 115));
                let trait_ = Stroke {
                    width: 2.0 * e,
                    dash: StrokeDash::new(vec![10.0 * e, 10.0 * e], 0.0),
                    ..Default::default()
                };
                p.stroke_path(&chemin, &paint, &trait_, Transform::identity(), None);
            }
            debut.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    mediane(mesures)
}

fn remplir(p: &mut Pixmap, boite: (f32, f32, f32, f32), r: f32, alpha: u8) {
    let mut pb = PathBuilder::new();
    arrondi(&mut pb, boite, r);
    if let Some(chemin) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(96, 165, 250, alpha));
        p.fill_path(
            &chemin,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

/// Le rectangle arrondi de l'ancien traceur : des coins en Bézier quadratiques.
fn arrondi(pb: &mut PathBuilder, (x, y, w, h): (f32, f32, f32, f32), r: f32) {
    let r = r.min(w / 2.0).min(h / 2.0);
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
}
