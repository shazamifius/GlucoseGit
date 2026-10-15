//! **Ce qu'un texte vu de près coûte, maintenant que la carte le dessine depuis ses formes**
//! (fiches 72 et 73).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_de_pres
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! Le 15/10, ses captures : de près, un texte reste flou « un bon moment », en gros pixels. La
//! carte graphique posait un nœud de texte comme une image que le processeur avait peinte ; de
//! près, l'image se découpait en carrés repeints un à un (DE-PRES-1), l'image entière agrandie
//! en attendant. Ce banc chiffrait ces carrés (fiche 72, à froid, son écran, une carte qui le
//! couvre) :
//!
//! ```text
//!   x16    21,3 ms (pire carré 3,0)      — et le flou, le temps de les rendre tous
//!   x32    19,7 ms (pire carré 2,5)
//!   x50    18,7 ms (pire carré 3,8)
//! ```
//!
//! Depuis la fiche 73, un nœud plus grand que l'écran se dessine **en formes** : chaque lettre
//! par Slug, sur la carte, à chaque image — rien ne se repeint, rien n'est flou. Ce banc dit ce
//! que cela coûte : le **relevé** des formes au processeur (la mise en page, les primitives), et
//! le **dessin** par la carte, image après image, mesuré jusqu'à ce que la carte ait fini.
//!
//! Il vise l'écran de l'utilisateur (2 560 × 1 600) et une carte dont le texte couvre l'écran.

use glucose_core::store::Store;
use glucose_core::types::Viewport;
use glucose_desktop::interactions::tools::text_card;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::present::banc_gpu;
use glucose_desktop::present::formes_gpu::{Formes, FormesGpu};
use glucose_desktop::renderer::{Regard, Renderer};
use glucose_desktop::ui::UiState;
use std::time::Instant;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (2560, 1600);

/// Combien d'images la carte dessine pour une mesure.
const IMAGES: u32 = 60;

const TEXTE: &str = "Le pointillé de la membrane : elle s'entend automatiquement quand tu \
cliques en plein dans la membrane, et bien sûr un texte avec du **gras**, de l'*italique* et \
une formule $e^{i\\pi} + 1 = 0$ pour voir ce que tout cela coûte de près.";

/// Un document d'une seule carte de ce texte, posée à l'origine.
fn une_carte(renderer: &Renderer, texte: &str) -> Store {
    let mut store = Store::new("de-pres");
    let board = store.project.active_board_id.clone();
    let carte = text_card(
        &renderer.typography,
        &renderer.math,
        "seule".to_string(),
        0.0,
        0.0,
        texte.to_string(),
    );
    store.add_annotation(&board, carte);
    store.clear_selection();
    store
}

/// Les formes que l'écran montre à cette échelle, la vue posée sur le début du texte, et ce
/// que le relevé a coûté.
fn formes(renderer: &mut Renderer, store: &mut Store, echelle: f64) -> (Formes, f64) {
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
    let debut = Instant::now();
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
    let ms = debut.elapsed().as_secs_f64() * 1000.0;
    (confie.formes, ms)
}

/// **Ce que la carte met à dessiner ces formes**, en millisecondes par image : la première
/// (l'atlas se remplit), puis la médiane des suivantes.
fn dessiner(gpu: &(wgpu::Device, wgpu::Queue), formes: &Formes) -> (f64, f64) {
    let (peripherique, file) = gpu;
    let mut passe = FormesGpu::nouvelles(peripherique, banc_gpu::FORMAT);
    let cible = banc_gpu::cible(peripherique, ECRAN);
    let vue = cible.create_view(&Default::default());
    let ecran = (ECRAN.0 as f32, ECRAN.1 as f32);
    let mut une_image = || {
        let debut = Instant::now();
        passe.preparer((peripherique, file), ecran, formes);
        let mut encodeur = peripherique.create_command_encoder(&Default::default());
        banc_gpu::passe(&mut encodeur, &vue, wgpu::Color::BLACK, |p| {
            for s in 0..formes.suites.len() {
                passe.poser(p, s);
            }
        });
        file.submit(Some(encodeur.finish()));
        let _ = peripherique.poll(wgpu::PollType::wait_indefinitely());
        debut.elapsed().as_secs_f64() * 1000.0
    };
    let premiere = une_image();
    let mut suivantes: Vec<f64> = (0..IMAGES).map(|_| une_image()).collect();
    suivantes.sort_by(f64::total_cmp);
    (premiere, suivantes[suivantes.len() / 2])
}

fn main() {
    let Some(gpu) = banc_gpu::carte() else {
        println!("aucune carte graphique : rien a mesurer");
        return;
    };
    println!(
        "Une carte de texte qui couvre l'ecran ({} x {}), a l'arret :",
        ECRAN.0, ECRAN.1
    );
    for echelle in [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 50.0, 128.0, 256.0] {
        let mut renderer = Renderer::new();
        let mut store = une_carte(&renderer, TEXTE);
        let (formes, releve) = formes(&mut renderer, &mut store, echelle);
        if formes.primitives.is_empty() {
            println!("  x{echelle:<6}  une texture (la carte tient dans l'ecran)");
            continue;
        }
        let (premiere, mediane) = dessiner(&gpu, &formes);
        println!(
            "  x{echelle:<6}  {:>5} primitives   releve (toute la scene) {releve:>6.1} ms   \
             carte : premiere {premiere:>6.2} ms, puis {mediane:>6.2} ms par image",
            formes.primitives.len()
        );
    }
    immense(&gpu);
}

/// **Un nœud immense, de près** (fiche 73 § 7) : des milliers de mots, dont l'écran ne montre
/// que le début. Le relevé de chaque image relit-il toute sa mise en page ? La première image,
/// puis la médiane des suivantes, la vue immobile ; et ce que coûte une mise en page seule.
///
/// ```text
///   (fiche 73, avant correction)
///      44 mots : releve 0,46 ms par image, dont une mise en page 0,06
///   17 600 mots : releve 10,5 ms par image, dont une mise en page 3,3 — la carte, 1,04
/// ```
fn immense(gpu: &(wgpu::Device, wgpu::Queue)) {
    println!("Un noeud de plus en plus long, vu a x16 (l'ecran n'en montre que le debut) :");
    for paragraphes in [1usize, 10, 100, 400] {
        let texte = vec![TEXTE; paragraphes].join("\n\n");
        let mots = texte.split_whitespace().count();
        let mut renderer = Renderer::new();
        let mut store = une_carte(&renderer, &texte);
        let (formes_, premiere) = formes(&mut renderer, &mut store, 16.0);
        let mut suivantes: Vec<f64> = (0..20)
            .map(|_| formes(&mut renderer, &mut store, 16.0).1)
            .collect();
        suivantes.sort_by(f64::total_cmp);
        let (_, carte) = dessiner(gpu, &formes_);
        let debut = Instant::now();
        let _ = glucose_desktop::renderer::card::card_text_layout(
            &renderer.typography,
            &renderer.math,
            &texte,
            glucose_core::types::DEFAULT_TEXT_CARD_WIDTH as f32,
            glucose_desktop::renderer::richtext::TextMode::Rendered,
        );
        let mise_en_page = debut.elapsed().as_secs_f64() * 1000.0;
        println!(
            "  {mots:>6} mots : releve {premiere:>6.1} ms la premiere image, puis {:>6.2} ms, \
             dont une mise en page {mise_en_page:>5.2} ms ; carte {carte:>5.2} ms",
            suivantes[suivantes.len() / 2],
        );
    }
}
