//! Le banc des **vraies images** — celui qui manquait, et dont l'absence a tout faussé.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_photos
//! cargo run --release -p glucose-desktop --example bench_photos -- 36
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! Tous les autres bancs du projet posent des `BoardImage` **sans fichier source**. Le poste
//! `images` y affichait donc `0.00 ms` sur toutes les mesures, à toutes les échelles, et je
//! l'ai lu des dizaines de fois sans le voir : le chemin de décodage et de rééchantillonnage
//! n'a jamais été mesuré une seule fois, pendant que j'optimisais le parcours du modèle.
//!
//! L'usage réel dit l'inverse du banc : trente-six photos suffisent à rendre l'application
//! impraticable, là où le banc annonce deux millisecondes sur un million de nœuds.
//!
//! # Ce qu'il mesure
//!
//! Des photographies de tailles réalistes, écrites sur disque puis relues par le chemin
//! normal — `Renderer::load_image_impl`, `draw_pixmap`, la même chose qu'à l'écran. Trois
//! échelles, parce que le coût d'un rééchantillonnage dépend du rapport entre la taille
//! source et la taille affichée, et que c'est précisément ce rapport qui est en cause.

use glucose_core::store::Store;
use glucose_core::types::BoardImage;
use glucose_desktop::bench;
use glucose_desktop::renderer::Renderer;
use glucose_desktop::ui::UiState;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tiny_skia::Pixmap;

/// Définition de l'écran mesuré.
const ECRAN: (u32, u32) = (2560, 1600);

/// Les tailles de photo, telles qu'un appareil ou un téléphone les produit.
const FORMATS: &[(u32, u32, &str)] = &[
    (4032, 3024, "12 Mpx (photo de téléphone)"),
    (1920, 1080, "2 Mpx (capture d'écran)"),
    (800, 600, "0,5 Mpx (vignette)"),
];

/// Taille à laquelle une photo est posée sur le canevas, en unités monde.
const TAILLE_POSEE: (f64, f64) = (420.0, 315.0);

/// Écrit une photo synthétique et rend son chemin.
///
/// Le motif compte peu, mais il ne doit pas être uni : un PNG uniforme se compresse en
/// quelques octets et se décode trop vite, ce qui flatterait la mesure du chargement.
fn ecrire_photo(dossier: &Path, largeur: u32, hauteur: u32) -> PathBuf {
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

/// Un document portant `n` photos du format donné, disposées en grille.
fn document_de_photos(n: usize, chemin: &Path) -> Store {
    let mut store = Store::new("photos");
    let board = store.project.active_board_id.clone();
    store.begin_live_edit();

    let colonnes = (n as f64).sqrt().ceil() as usize;
    for i in 0..n {
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
    store.end_live_edit();
    store
}

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

    fn image(&mut self, store: &Store) -> f64 {
        glucose_desktop::perf::frame_begin();
        let t = Instant::now();
        bench::render_into(&mut self.renderer, &mut self.ui, store, &mut self.pixmap);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        glucose_desktop::perf::frame_end();
        ms
    }
}

/// La médiane de quelques images consécutives : une seule mesure attrape le bruit du système.
fn mediane(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(36);

    let dossier = std::env::temp_dir().join("glucose-bench-photos");
    std::fs::create_dir_all(&dossier).expect("un dossier de travail");

    println!("Banc des photos — {} × {}, {n} images\n", ECRAN.0, ECRAN.1);
    println!(
        "  {:<32} {:>10} {:>10} {:>10} {:>12}",
        "format source", "zoom 1", "zoom 0,5", "zoom 0,25", "1re image"
    );

    for &(largeur, hauteur, nom) in FORMATS {
        let chemin = ecrire_photo(&dossier, largeur, hauteur);
        let mut store = document_de_photos(n, &chemin);
        let mut atelier = Atelier::new();

        // La première image paie le décodage et la conversion en prémultiplié : c'est un coût
        // réel, mais il n'appartient pas aux images suivantes. On le mesure à part.
        bench::frame_document(&mut store, 1.0, ECRAN.0, ECRAN.1);
        let premiere = atelier.image(&store);

        let mut temps = Vec::new();
        for zoom in [1.0, 0.5, 0.25] {
            bench::frame_document(&mut store, zoom, ECRAN.0, ECRAN.1);
            // Deux images de chauffe : la première découvre la forme, la seconde construit les
            // vignettes. Ce qui est mesuré ensuite est le régime établi — celui où l'on
            // sélectionne, désélectionne, et regarde, c'est-à-dire la plupart du temps.
            atelier.image(&store);
            atelier.image(&store);
            temps.push(mediane((0..15).map(|_| atelier.image(&store)).collect()));
        }

        println!(
            "  {nom:<32} {:>8.1}ms {:>8.1}ms {:>8.1}ms {:>10.1}ms",
            temps[0], temps[1], temps[2], premiere
        );
    }

    // Une capture, pour être REGARDÉE : aucun test ne voit une image posée un pixel trop bas,
    // et le témoin visuel du projet ne porte aucune image munie d'un fichier.
    let chemin = ecrire_photo(&dossier, 1920, 1080);
    let mut store = document_de_photos(9, &chemin);
    bench::frame_document(&mut store, 1.0, ECRAN.0, ECRAN.1);
    let mut atelier = Atelier::new();
    atelier.image(&store);
    atelier.image(&store); // la seconde image est celle qui passe par les vignettes
    let capture = dossier.join("capture.png");
    atelier
        .pixmap
        .save_png(&capture)
        .expect("écrire la capture");
    println!(
        "
  capture : {}",
        capture.display()
    );

    println!(
        "\n  (photos écrites dans {}, réutilisées d'un lancement à l'autre)",
        dossier.display()
    );
}
