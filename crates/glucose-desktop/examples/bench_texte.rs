//! **Ce que le texte coûte, et pourquoi** : un mur de cartes, un glissement, puis un zoom.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_texte
//! cargo run --release -p glucose-desktop --example bench_texte -- 480
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! La chronique de terrain du 21/09, sur 482 nœuds, donnait `annotations` à 1 ms en médiane
//! — et à **19 ms au p99 pendant un glissement, 27 pendant un zoom, 81 au pire**. Un poste
//! dont le p99 vaut vingt fois la médiane n'est pas un poste lent : c'est un poste qui, une
//! image sur cent, refait tout. L'utilisateur l'a dit dans ses mots : « le problème c'est
//! clairement le texte, ça fait que lag ».
//!
//! Deux hypothèses, et ce banc les sépare parce qu'il **compte** au lieu de chronométrer :
//!
//! * pendant un **zoom**, la taille de police change à chaque image, et le cache de glyphes
//!   est indexé au dixième de point — chaque image rastérise donc tous les glyphes visibles à
//!   neuf ;
//! * pendant un **glissement**, la taille ne bouge pas, mais chaque glyphe existe en seize
//!   phases sous-pixel et le cache plafonne à 4 096 variantes. Assez de cartes, et il évince
//!   en boucle ce qu'il vient de rastériser.
//!
//! La colonne qui tranche est `rasterises` : combien de variantes de glyphes cette image a
//! dû construire. Zéro veut dire que le cache a servi ; des centaines veut dire qu'il a été
//! contourné, par la taille ou par l'éviction.

use glucose_core::store::Store;
use glucose_core::types::Viewport;
use glucose_desktop::interactions::tools::text_card;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::renderer::{Regard, Renderer};
use glucose_desktop::ui::UiState;
use std::collections::BTreeMap;
use std::time::Instant;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (2560, 1600);
const CARTES_PAR_DEFAUT: usize = 120;
const IMAGES: usize = 100;

fn mur(renderer: &Renderer, cartes: usize) -> Store {
    let mut store = Store::new("texte");
    let board = store.project.active_board_id.clone();
    let colonnes = (cartes as f64).sqrt().ceil().max(1.0) as usize;
    for i in 0..cartes {
        let carte = text_card(
            &renderer.typography,
            &renderer.math,
            format!("carte-{i}"),
            (i % colonnes) as f64 * 320.0,
            (i / colonnes) as f64 * 180.0,
            format!("Carte {i} — une note posée sur le mur, avec **du gras** et du texte qui revient à la ligne."),
        );
        store.add_annotation(&board, carte);
    }
    store.clear_selection();
    store
}

fn centile(v: &[f64], p: f64) -> f64 {
    let mut t = v.to_vec();
    t.sort_by(f64::total_cmp);
    t[((t.len() as f64 * p) as usize).min(t.len() - 1)]
}

/// Ce qu'un geste a donné : la durée de chaque image, les variantes de glyphes qu'elle a dû
/// construire, et ses postes en millisecondes.
struct Geste {
    durees: Vec<f64>,
    rasterises: Vec<usize>,
    postes: BTreeMap<&'static str, Vec<f64>>,
}

/// Joue `IMAGES` images le long de `vue`, par la **voie graphique** — celle de l'utilisateur.
///
/// Le processeur y produit les deux couches et ce qu'il confie à la carte ; le banc ne
/// compose pas, parce que c'est le coût **processeur** qu'il mesure, et que la chronique
/// chiffre la composition à part.
fn jouer(renderer: &mut Renderer, store: &mut Store, vue: impl Fn(usize) -> Viewport) -> Geste {
    let board = store.project.active_board_id.clone();
    let mut dessous = Pixmap::new(ECRAN.0, ECRAN.1).expect("un pixmap");
    let mut dessus = Pixmap::new(ECRAN.0, ECRAN.1).expect("un pixmap");
    let mut ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let mut geste = Geste {
        durees: Vec::new(),
        rasterises: Vec::new(),
        postes: BTreeMap::new(),
    };
    for i in 0..IMAGES {
        store.set_viewport(&board, vue(i));
        let avant = renderer.typography.cached_glyph_count();
        glucose_desktop::perf::frame_begin();
        let debut = Instant::now();
        dessus.fill(tiny_skia::Color::TRANSPARENT);
        glucose_desktop::perf::stage("effacer");
        renderer.rendre_les_couches(
            &mut dessous.as_mut(),
            &mut dessus.as_mut(),
            store,
            (&mut ui, Pointer { x: 0.0, y: 0.0 }),
            SceneOverlay {
                guides: &guides,
                selection_box: None,
                editing: None,
            },
            Regard {
                degradation_permise: false,
                en_mouvement: true,
            },
        );
        geste.durees.push(debut.elapsed().as_secs_f64() * 1000.0);
        glucose_desktop::perf::frame_end();
        for (nom, ms) in glucose_desktop::perf::postes() {
            let v = geste.postes.entry(nom).or_insert_with(|| vec![0.0; IMAGES]);
            v[i] = ms;
        }
        let apres = renderer.typography.cached_glyph_count();
        // Une eviction fait baisser le compte : ce qui a ete construit est au moins la
        // hausse nette, et c'est ce qu'on lit. Le cache ne dit pas plus.
        geste.rasterises.push(apres.saturating_sub(avant));
    }
    geste
}

fn ligne(nom: &str, geste: &Geste) {
    let r: Vec<f64> = geste.rasterises.iter().map(|&n| n as f64).collect();
    println!(
        "  {nom:<12} image  med {:6.2} ms  p90 {:6.2}  p99 {:6.2}  pire {:6.2}   |  rasterises  med {:5.0}  p99 {:5.0}  pire {:5.0}",
        centile(&geste.durees, 0.5),
        centile(&geste.durees, 0.9),
        centile(&geste.durees, 0.99),
        centile(&geste.durees, 1.0),
        centile(&r, 0.5),
        centile(&r, 0.99),
        centile(&r, 1.0),
    );
    // Les postes, du plus lourd au plus leger en mediane -- et jamais sommes (fiche 20 § 4.5).
    let mut postes: Vec<(&str, f64, f64, f64)> = geste
        .postes
        .iter()
        .map(|(n, v)| (*n, centile(v, 0.5), centile(v, 0.99), centile(v, 1.0)))
        .filter(|(_, med, _, _)| *med >= 0.05)
        .collect();
    postes.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (n, med, p99, pire) in postes.iter().take(8) {
        println!("      {n:<14} med {med:6.2}   p99 {p99:6.2}   pire {pire:6.2}");
    }
}

fn main() {
    let cartes: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(CARTES_PAR_DEFAUT);
    let mut renderer = Renderer::new();
    let mut store = mur(&renderer, cartes);
    println!(
        "Ce que le texte coute, mesure -- {cartes} cartes, {} x {}, {IMAGES} images par geste\n",
        ECRAN.0, ECRAN.1
    );
    println!(
        "  cache de glyphes : {} variantes au depart\n",
        renderer.typography.cached_glyph_count()
    );

    // Une image de chauffe a l'echelle 1 : le cache se remplit une fois.
    let chauffe = jouer(&mut renderer, &mut store, |_| Viewport {
        x: 100.0,
        y: 100.0,
        scale: 1.0,
    });
    ligne("immobile", &chauffe);
    println!(
        "  apres la chauffe : {} variantes\n",
        renderer.typography.cached_glyph_count()
    );

    // Le glissement : l'echelle ne bouge pas, la phase sous-pixel de chaque glyphe si.
    let g = jouer(&mut renderer, &mut store, |i| Viewport {
        x: 100.0 - i as f64 * 7.3,
        y: 100.0 - i as f64 * 2.1,
        scale: 1.0,
    });
    ligne("glissement", &g);

    // Le zoom : l'echelle avance d'un demi pour cent par image, donc la taille de police
    // change presque a chaque image.
    let g = jouer(&mut renderer, &mut store, |i| Viewport {
        x: 100.0,
        y: 100.0,
        scale: 1.0 + i as f64 * 0.005,
    });
    ligne("zoom", &g);

    // Le zoom par octaves : l'echelle double, mais par paliers dyadiques -- ce que
    // deviendrait un zoom si la taille de police se quantifiait comme les tuiles.
    let g = jouer(&mut renderer, &mut store, |i| Viewport {
        x: 100.0,
        y: 100.0,
        scale: if i < 50 { 1.0 } else { 2.0 },
    });
    ligne("par paliers", &g);

    println!(
        "\n  cache de glyphes a la fin : {} variantes (plafond 4096)",
        renderer.typography.cached_glyph_count()
    );
}
