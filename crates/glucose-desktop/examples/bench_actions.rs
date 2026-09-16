//! Le banc des **actions** — ce que coûte ce qu'on fait, pas ce qu'on regarde.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_actions
//! cargo run --release -p glucose-desktop --example bench_actions -- 1000000
//! ```
//!
//! # L'angle mort que ce banc comble
//!
//! Tous les autres bancs du projet mesurent une **image**, et toujours la même : `bench_frame`
//! rend une scène immobile, `bench_geste` déplace la caméra, `bench_chrome` redessine des
//! panneaux qui ne changent pas. Aucun ne mesure ce qu'on **fait** — poser une image,
//! sélectionner, déplacer, enregistrer.
//!
//! Or c'est là que l'application est jugée. « Un million de blocs de texte ne la font pas
//! tomber » n'est pas la question ; la question est ce qui se passe quand on les sélectionne
//! tous et qu'on les déplace.
//!
//! # Une action, c'est deux temps
//!
//! Chaque ligne en donne deux, et les confondre serait perdre l'information utile :
//!
//! * **le modèle** — ce que la mutation coûte dans le document : parcourir, écrire,
//!   enregistrer l'annulation, reconstruire l'index ;
//! * **l'image** — ce que coûte la première image *après* la mutation, c'est-à-dire ce que
//!   l'utilisateur attend avant de voir son geste.
//!
//! Une action peut être instantanée dans le modèle et insupportable à l'écran, ou l'inverse.
//! Ce sont deux défauts différents, qui ne se corrigent pas au même endroit.

use glucose_core::store::Store;
use glucose_core::synth::{self, Shape};
use glucose_core::types::{Annotation, BoardImage};
use glucose_desktop::bench;
use glucose_desktop::renderer::Renderer;
use glucose_desktop::ui::UiState;
use std::time::Instant;
use tiny_skia::Pixmap;

/// Les tailles de document mesurées.
const TAILLES: &[usize] = &[1_000, 100_000, 1_000_000];

/// La définition de l'écran où le défaut a été constaté.
const ECRAN: (u32, u32) = (2560, 1600);

/// La même graine que les autres bancs.
const GRAINE: u64 = 0x91ac05e;

/// Le budget d'une action, en millisecondes.
///
/// Ce n'est pas celui d'une image. Une action franchit un seuil de perception bien plus haut :
/// en deçà de cent millisecondes elle paraît instantanée, au-delà d'une seconde l'attention
/// décroche. On vise le premier.
const BUDGET_ACTION_MS: f64 = 100.0;

fn span_pour(n: usize) -> f64 {
    2_000.0 * (n as f64).sqrt()
}

/// Ce qu'une action a coûté : la mutation, puis l'image qui la montre.
struct Cout {
    modele: f64,
    image: f64,
}

impl Cout {
    fn total(&self) -> f64 {
        self.modele + self.image
    }
}

/// De quoi rendre une image sans la refaire naître à chaque fois.
///
/// `bench::capture` ne convient pas ici : elle recrée le moteur — donc **recharge les
/// polices** — et encode un PNG de quatre mégapixels. Mesurée avec elle, une image de mille
/// nœuds semblait coûter 120 ms là où le banc de frame en annonce 2,19. Un banc qui mesure
/// son propre outillage ne mesure rien.
struct Atelier {
    renderer: Renderer,
    ui: UiState,
    pixmap: Pixmap,
}

impl Atelier {
    fn new() -> Self {
        Self {
            renderer: Renderer::new(),
            ui: UiState::new(),
            pixmap: Pixmap::new(ECRAN.0, ECRAN.1).expect("un tampon"),
        }
    }
}

/// Mesure une action : la mutation, puis la première image qui la rend visible.
fn mesurer(store: &mut Store, atelier: &mut Atelier, action: impl FnOnce(&mut Store)) -> Cout {
    let t = Instant::now();
    action(store);
    let modele = t.elapsed().as_secs_f64() * 1000.0;

    glucose_desktop::perf::frame_begin();
    let t = Instant::now();
    bench::render_into(
        &mut atelier.renderer,
        &mut atelier.ui,
        store,
        &mut atelier.pixmap,
    );
    let image = t.elapsed().as_secs_f64() * 1000.0;
    glucose_desktop::perf::frame_end();

    Cout { modele, image }
}

/// Tous les identifiants d'annotation du tableau courant.
fn tous_les_ids(store: &Store) -> Vec<String> {
    store
        .active_board()
        .map(|b| b.annotations.iter().map(|a| a.id().to_string()).collect())
        .unwrap_or_default()
}

fn ligne(nom: &str, c: &Cout) {
    let verdict = if c.total() > BUDGET_ACTION_MS {
        "  <-- HORS BUDGET"
    } else {
        ""
    };
    println!(
        "  {nom:<28} {:>9.2}ms {:>9.2}ms {:>9.2}ms{verdict}",
        c.modele,
        c.image,
        c.total()
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tailles: Vec<usize> = if args.is_empty() {
        TAILLES.to_vec()
    } else {
        args.iter().filter_map(|a| a.parse().ok()).collect()
    };

    println!(
        "Banc des actions — {} × {}, budget {BUDGET_ACTION_MS} ms par action\n",
        ECRAN.0, ECRAN.1
    );

    for taille in tailles {
        println!("── {taille} nœuds ──────────────────────────────────────────────");
        let t = Instant::now();
        let mut store = synth::document(taille, span_pour(taille), Shape::Clustered, GRAINE);
        bench::frame_document(&mut store, 1.0, ECRAN.0, ECRAN.1);
        println!(
            "  (document construit en {:.0} ms)",
            t.elapsed().as_secs_f64() * 1000.0
        );
        println!(
            "  {:<28} {:>11} {:>11} {:>11}",
            "action", "modèle", "image", "total"
        );

        let bid = store.project.active_board_id.clone();
        let mut atelier = Atelier::new();

        // Une image de chauffe, jetée : l'indexation spatiale initiale, le chargement des
        // polices et la première teinte se paient une fois, et n'appartiennent à aucune des
        // actions mesurées. Sans elle, la première ligne du tableau portait 1 192 ms
        // d'indexation qui n'avaient rien à voir avec elle — et j'ai cherché la cause dans le
        // décalage des rangs, qui n'y était pour rien.
        bench::render_into(
            &mut atelier.renderer,
            &mut atelier.ui,
            &store,
            &mut atelier.pixmap,
        );

        // ── Poser quelque chose ────────────────────────────────────────────
        let c = mesurer(&mut store, &mut atelier, |s| {
            let img = BoardImage::new("bench-img", 0.0, 0.0, 400.0, 300.0);
            s.add_image(&bid, img);
        });
        ligne("ajouter une image", &c);

        let bid2 = bid.clone();
        let c = mesurer(&mut store, &mut atelier, |s| {
            s.add_annotation(&bid2, Annotation::sticky("bench-note", 10.0, 10.0, "note"));
        });
        ligne("ajouter une note", &c);

        // ── Naviguer ───────────────────────────────────────────────────────
        let c = mesurer(&mut store, &mut atelier, |s| s.pan(120.0, 80.0));
        ligne("déplacer la vue", &c);

        let c = mesurer(&mut store, &mut atelier, |s| {
            s.zoom(1.1, 1280.0, 800.0, (0.02, 20.0));
        });
        ligne("zoomer d'un cran", &c);

        let c = mesurer(&mut store, &mut atelier, |s| {
            s.zoom(0.5, 1280.0, 800.0, (0.02, 20.0));
        });
        ligne("dézoomer de moitié", &c);

        // ── Sélectionner, puis agir sur la sélection ───────────────────────
        let ids = tous_les_ids(&store);
        let combien = ids.len();
        let c = mesurer(&mut store, &mut atelier, |s| {
            s.set_selected_annotation_ids(ids)
        });
        ligne(&format!("tout sélectionner ({combien})"), &c);

        let bid3 = bid.clone();
        let c = mesurer(&mut store, &mut atelier, |s| {
            s.move_selected(&bid3, 40.0, 25.0)
        });
        ligne("déplacer la sélection", &c);

        let c = mesurer(&mut store, &mut atelier, |s| {
            s.undo();
        });
        ligne("annuler ce déplacement", &c);

        // ── Enregistrer ────────────────────────────────────────────────────
        let t = Instant::now();
        let octets = glucose_core::persist::encode(&store.project, &store.assets, 0);
        let encodage = t.elapsed().as_secs_f64() * 1000.0;
        let verdict = if encodage > BUDGET_ACTION_MS {
            "  <-- HORS BUDGET"
        } else {
            ""
        };
        println!(
            "  {:<28} {encodage:>9.2}ms {:>9} {encodage:>9.2}ms{verdict}   ({} Mo)",
            "enregistrer (Ctrl+S)",
            "—",
            octets.len() / 1_000_000
        );

        // ── Supprimer ──────────────────────────────────────────────────────
        let bid4 = bid.clone();
        let c = mesurer(&mut store, &mut atelier, |s| s.delete_selected(&bid4));
        ligne("supprimer la sélection", &c);

        println!();
    }
}
