//! Le banc du **freinage** — ce que coûte chaque image pendant qu'un glissement s'éteint, et
//! surtout de combien ce coût **varie**.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_freinage
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! L'utilisateur décrit, depuis plusieurs sessions et dans les mêmes termes : « quand on
//! freine progressivement, on voit tout en genre quatre images par seconde ». La chronique
//! du rythme a montré pourquoi aucune mesure de coût ne pouvait le voir : la vitesse
//! apparente vaut `v · pas / durée d'affichage`, et elle saute chaque fois que le coût d'une
//! image change. Ce qui fait le judder n'est pas le coût, c'est sa **variance**.
//!
//! Ce banc rejoue donc le geste — trente-six photos, un glissement franc, puis l'élan qui
//! s'éteint — et rend, pour chaque régime de rendu, le coût médian **et l'écart entre
//! l'image la moins chère et la plus chère**. C'est la deuxième colonne qui répond à la
//! question posée.
//!
//! # Ce qu'il compare
//!
//! * **direct** — la passe des images d'hier, chaque photo posée à chaque image ;
//! * **grille** — les tuiles, composées à leur place, ce que l'application fait désormais.
//!
//! Les photos sont celles de `bench_photos`, écrites une fois dans le dossier temporaire.

use glucose_core::store::Store;
use glucose_core::types::{BoardImage, Viewport};
use glucose_desktop::bench;
use glucose_desktop::renderer::{Regard, Renderer};
use glucose_desktop::ui::UiState;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (2560, 1600);
const PHOTOS: usize = 36;
const TAILLE_POSEE: (f64, f64) = (420.0, 315.0);

/// Combien d'images le freinage dure. Une glissade de pavé tactile s'éteint en une demi-
/// seconde environ, soit cent vingt images à 240 Hz.
const IMAGES: usize = 120;

/// La vitesse de départ, en pixels par seconde — un glissement franc.
const VITESSE: f64 = 1_200.0;

/// La constante de temps de l'amortissement, celle de l'élan (voir `interactions::elan`).
const TAU: f64 = 0.45;

fn ecrire_photo(dossier: &Path) -> PathBuf {
    let (largeur, hauteur) = (1920u32, 1080u32);
    let chemin = dossier.join(format!("photo-{largeur}x{hauteur}.png"));
    if chemin.exists() {
        return chemin;
    }
    let mut pixels = Vec::with_capacity((largeur * hauteur * 4) as usize);
    for y in 0..hauteur {
        for x in 0..largeur {
            let r = ((x * 7 + y * 3) % 256) as u8;
            let v = ((x ^ y) % 256) as u8;
            let b = ((x / 3 + y * 5) % 256) as u8;
            pixels.extend_from_slice(&[r, v, b, 255]);
        }
    }
    let tampon = image::RgbaImage::from_raw(largeur, hauteur, pixels).expect("une image");
    tampon.save(&chemin).expect("écrire la photo");
    chemin
}

fn document(chemin: &Path) -> Store {
    let mut store = Store::new("freinage");
    let board = store.project.active_board_id.clone();
    let colonnes = (PHOTOS as f64).sqrt().ceil() as usize;
    for i in 0..PHOTOS {
        let (ligne, colonne) = (i / colonnes, i % colonnes);
        let mut img = BoardImage::new(
            format!("photo-{i}"),
            colonne as f64 * (TAILLE_POSEE.0 + 40.0),
            ligne as f64 * (TAILLE_POSEE.1 + 40.0),
            TAILLE_POSEE.0,
            TAILLE_POSEE.1,
        );
        img.src = Some(chemin.to_string_lossy().into_owned());
        store.add_image(&board, img);
    }
    store.clear_selection();
    store
}

/// Ce qu'un freinage coûte, image par image.
struct Freinage {
    durees_ms: Vec<f64>,
}

impl Freinage {
    fn mediane(&self) -> f64 {
        let mut v = self.durees_ms.clone();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    }
    fn min(&self) -> f64 {
        self.durees_ms.iter().copied().fold(f64::INFINITY, f64::min)
    }
    fn max(&self) -> f64 {
        self.durees_ms.iter().copied().fold(0.0, f64::max)
    }
    /// Le plus grand écart entre deux images **consécutives** : c'est lui qui se voit.
    fn pire_saut(&self) -> f64 {
        self.durees_ms
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0, f64::max)
    }
}

/// Rejoue un freinage à l'échelle donnée, avec ou sans la grille.
fn rejouer(store: &mut Store, echelle: f64, regard: Regard) -> Freinage {
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    ui.current_toast = None;
    let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("l'ecran");
    let board = store.project.active_board_id.clone();

    // Le point de départ : au milieu des photos.
    let mut vue = Viewport {
        x: 200.0,
        y: 200.0,
        scale: echelle,
    };
    store.set_viewport(&board, vue);
    // Les photos se décodent une fois, hors mesure : c'est le régime chaud qui compte.
    bench::render_into(&mut renderer, &mut ui, store, &mut pixmap);
    renderer.magasin.attendre_le_chantier();
    bench::render_into(&mut renderer, &mut ui, store, &mut pixmap);

    let periode = 1.0 / 240.0;
    let mut durees_ms = Vec::with_capacity(IMAGES);
    for i in 0..IMAGES {
        // L'amortissement exponentiel de l'élan : la vitesse à cette image.
        let t = i as f64 * periode;
        let v = VITESSE * (-t / TAU).exp();
        vue.x -= v * periode;
        vue.y -= v * periode * 0.4;
        store.set_viewport(&board, vue);

        let guides = glucose_core::smart_align::SnapGuides::default();
        let overlay = glucose_desktop::params::SceneOverlay {
            guides: &guides,
            selection_box: None,
            editing: None,
        };
        // `GLUCOSE_PERF=1` decompose chaque image en postes, comme dans l'application.
        glucose_desktop::perf::frame_begin();
        let t0 = Instant::now();
        renderer.render(
            &mut pixmap.as_mut(),
            store,
            &mut ui,
            overlay,
            glucose_desktop::params::Pointer { x: -1.0, y: -1.0 },
            regard,
        );
        durees_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        glucose_desktop::perf::frame_end();
    }
    Freinage { durees_ms }
}

fn main() {
    let dossier = std::env::temp_dir().join("glucose-bench-photos");
    std::fs::create_dir_all(&dossier).expect("dossier temporaire");
    let chemin = ecrire_photo(&dossier);
    let mut store = document(&chemin);

    println!(
        "Banc du freinage — {PHOTOS} photos, {} x {}, {IMAGES} images de glissade\n",
        ECRAN.0, ECRAN.1
    );
    println!(
        "  {:<28} {:>9} {:>9} {:>9} {:>11}",
        "regime", "median", "min", "max", "pire saut"
    );

    // Le rendu direct : ce que l'application faisait hier. On le force en se plaçant entre
    // deux niveaux dyadiques et à l'arrêt, le seul cas où la grille cède la main.
    let direct = rejouer(&mut store, 1.3, Regard::immobile());
    println!(
        "  {:<28} {:>7.2}ms {:>7.2}ms {:>7.2}ms {:>9.2}ms",
        "direct (x1,3, hier)",
        direct.mediane(),
        direct.min(),
        direct.max(),
        direct.pire_saut()
    );

    // La grille, à l'échelle exacte : composition pixel pour pixel.
    let exact = rejouer(&mut store, 1.0, Regard::immobile());
    println!(
        "  {:<28} {:>7.2}ms {:>7.2}ms {:>7.2}ms {:>9.2}ms",
        "grille exacte (x1)",
        exact.mediane(),
        exact.min(),
        exact.max(),
        exact.pire_saut()
    );

    // La grille entre deux niveaux, en mouvement : agrandie au plus proche.
    let en_mouvement = Regard {
        degradation_permise: false,
        en_mouvement: true,
    };
    let proche = rejouer(&mut store, 1.3, en_mouvement);
    println!(
        "  {:<28} {:>7.2}ms {:>7.2}ms {:>7.2}ms {:>9.2}ms",
        "grille au plus proche (x1,3)",
        proche.mediane(),
        proche.min(),
        proche.max(),
        proche.pire_saut()
    );

    println!();
    println!("  Le pire saut est l'ecart entre deux images CONSECUTIVES : c'est lui que l'oeil");
    println!("  recoit comme un tressaut, puisque le contenu avance du pas de l'une et reste");
    println!("  affiche la duree de l'autre. Une mediane basse avec un pire saut eleve est");
    println!("  exactement le cas que l'utilisateur decrit.");
}
