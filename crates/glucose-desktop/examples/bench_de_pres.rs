//! **Ce qu'un texte vu de près coûte à rendre, carré par carré** (fiche 72).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_de_pres
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! Le 15/10, ses captures : de près, un texte reste flou « un bon moment », en gros pixels, un
//! carré net au milieu — illisible, « impossible de travailler dessus ». Les photos, elles,
//! restent nettes. Une carte vue de près se découpe en carrés (DE-PRES-1) que la cascade rend un
//! à un, dans le temps libre de chaque image (CASCADE-2) ; en attendant, l'ancienne texture est
//! posée, agrandie. Ce banc dit **combien coûte chaque carré**, et de quoi ce coût dépend :
//! l'échelle, le premier rendu (glyphes jamais vus) ou les suivants.
//!
//! Il vise l'écran de l'utilisateur (2 560 × 1 600) et une carte dont le texte couvre l'écran.

use glucose_core::store::Store;
use glucose_core::types::Viewport;
use glucose_desktop::interactions::tools::text_card;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::renderer::composants::Composant;
use glucose_desktop::renderer::{Regard, Renderer};
use glucose_desktop::ui::UiState;
use std::time::Instant;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (2560, 1600);

const TEXTE: &str = "Le pointillé de la membrane : elle s'entend automatiquement quand tu \
cliques en plein dans la membrane, et bien sûr un texte avec du **gras**, de l'*italique* et \
une formule $e^{i\\pi} + 1 = 0$ pour voir ce que tout cela coûte de près.";

/// Un document d'une seule carte, posée à l'origine.
fn une_carte(renderer: &Renderer) -> Store {
    let mut store = Store::new("de-pres");
    let board = store.project.active_board_id.clone();
    let carte = text_card(
        &renderer.typography,
        &renderer.math,
        "seule".to_string(),
        0.0,
        0.0,
        TEXTE.to_string(),
    );
    store.add_annotation(&board, carte);
    store.clear_selection();
    store
}

/// Les composants que l'écran montre à cette échelle — les carrés de la carte, de près —, la
/// vue posée sur le début du texte.
fn composants(renderer: &mut Renderer, store: &mut Store, echelle: f64) -> Vec<Composant> {
    let board = store.project.active_board_id.clone();
    // Le coin haut gauche de l'écran sur le monde (20, 40) : le début de la première ligne.
    store.set_viewport(
        &board,
        Viewport {
            scale: echelle,
            x: -20.0 * echelle,
            y: -40.0 * echelle,
        },
    );
    let mut dessous = Pixmap::new(ECRAN.0, ECRAN.1).expect("tampon");
    let mut dessus = Pixmap::new(ECRAN.0, ECRAN.1).expect("tampon");
    let mut ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let confie = renderer.rendre_les_couches(
        &mut dessous.as_mut(),
        &mut dessus.as_mut(),
        store,
        (&mut ui, Pointer { x: 0.0, y: 0.0 }),
        SceneOverlay::sans_rien(&guides),
        Regard {
            degradation_permise: false,
            en_mouvement: false,
        },
    );
    confie.composants
}

fn une_echelle(echelle: f64) {
    // Un renderer neuf : le cache de glyphes est froid, comme après un zoom d'une octave.
    let mut renderer = Renderer::new();
    let mut store = une_carte(&renderer);
    let carres = composants(&mut renderer, &mut store, echelle);
    let (mut froid, mut pire, mut kpx) = (0.0f64, 0.0f64, 0.0f64);
    for c in &carres {
        let debut = Instant::now();
        let rendu = c.rendre(renderer.kit());
        let ms = debut.elapsed().as_secs_f64() * 1000.0;
        froid += ms;
        pire = pire.max(ms);
        if let Some(p) = rendu {
            kpx += f64::from(p.width()) * f64::from(p.height()) / 1000.0;
        }
    }
    let debut = Instant::now();
    for c in &carres {
        let _ = c.rendre(renderer.kit());
    }
    let chaud = debut.elapsed().as_secs_f64() * 1000.0;
    println!(
        "  x{echelle:<6}  {:>4} carres  {kpx:>8.0} kpx   froid {froid:>9.1} ms  (pire carre {pire:>7.1} ms)   chaud {chaud:>8.1} ms",
        carres.len()
    );
}

fn main() {
    println!(
        "Une carte de texte qui couvre l'ecran ({} x {}), a l'arret :",
        ECRAN.0, ECRAN.1
    );
    for echelle in [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 50.0] {
        une_echelle(echelle);
    }
}
