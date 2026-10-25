//! **Un grand tableau de photos** (fiche 82, P0) : `nombre` images JPEG fabriquées, posées par le
//! vrai dépôt, écrites par le vrai scribe dans un vrai document — le tableau de plus de 6 000
//! images que deux téléphones ont perdu le 15/10, à rejouer sur le téléphone virtuel.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example fabriquer_un_grand_tableau -- <sortie.glucose> <nombre> [largeur hauteur]
//! ```
//!
//! Les images naissent dans `<sortie>.images/`, par lots de cent ; chaque lot se pose à sa place
//! sur une grille, comme des dépôts successifs. Du bruit dans chaque image : sans lui, le JPEG
//! d'un dégradé pèserait dix fois moins qu'une photo, et la mémoire ne serait pas éprouvée.

use glucose_desktop::app::GlucoseApp;
use std::path::{Path, PathBuf};

/// Les images d'un lot : un dépôt.
const PAR_LOT: usize = 100;
/// Les lots d'une rangée de la grille.
const LOTS_PAR_RANGEE: usize = 8;

/// Une photo de `l` × `h`, sa teinte tirée de `rang`, du bruit déterministe par-dessus.
fn photo(chemin: &Path, rang: usize, (l, h): (u32, u32)) {
    let mut graine = (rang as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut bruit = move || {
        graine ^= graine << 13;
        graine ^= graine >> 7;
        graine ^= graine << 17;
        (graine & 0x3F) as u8
    };
    let teinte = (rang * 37 % 256) as u8;
    let image = image::RgbImage::from_fn(l, h, |x, y| {
        let n = bruit();
        image::Rgb([
            teinte.wrapping_add((x / 4) as u8).wrapping_add(n),
            ((y * 255) / h) as u8 ^ n,
            teinte.wrapping_sub((y / 3) as u8).wrapping_add(n / 2),
        ])
    });
    image.save(chemin).expect("la photo s'écrit");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage : fabriquer_un_grand_tableau <sortie.glucose> <nombre> [largeur hauteur]");
        std::process::exit(2);
    }
    let sortie = PathBuf::from(&args[1]);
    let nombre: usize = args[2].parse().expect("un nombre d'images");
    let taille = match (args.get(3), args.get(4)) {
        (Some(l), Some(h)) => (l.parse().expect("largeur"), h.parse().expect("hauteur")),
        _ => (600, 900),
    };
    let atelier = sortie.with_extension("images");
    std::fs::create_dir_all(&atelier).expect("le dossier des images");
    let maison = sortie.with_extension("maison");
    std::fs::create_dir_all(&maison).expect("le dossier de l'application");

    let mut app = GlucoseApp::new();
    app.habiter(&maison);
    app.project_path = Some(sortie.clone());
    app.save_project();
    let debut = std::time::Instant::now();
    for (lot, premier) in (0..nombre).step_by(PAR_LOT).enumerate() {
        let chemins: Vec<PathBuf> = (premier..(premier + PAR_LOT).min(nombre))
            .map(|rang| {
                let chemin = atelier.join(format!("photo-{rang:05}.jpg"));
                if !chemin.exists() {
                    photo(&chemin, rang, taille);
                }
                chemin
            })
            .collect();
        // Le lot se pose au centre de la vue : la vue va à sa place sur la grille.
        let (colonne, rangee) = (lot % LOTS_PAR_RANGEE, lot / LOTS_PAR_RANGEE);
        let tableau = app.store.project.active_board_id.clone();
        if let Some(b) = app
            .store
            .project
            .boards
            .iter_mut()
            .find(|b| b.id == tableau)
        {
            b.viewport.scale = 1.0;
            b.viewport.x = 720.0 - colonne as f64 * 7_000.0;
            b.viewport.y = 450.0 - rangee as f64 * 11_000.0;
        }
        app.drop_files(&chemins);
        app.consigner();
        println!(
            "lot {lot} : {} images, {:.0} s",
            premier + chemins.len(),
            debut.elapsed().as_secs_f64()
        );
    }
    assert!(
        app.fermer_le_document(),
        "le document s'écrit jusqu'au bout"
    );
    let poids = std::fs::metadata(&sortie).map_or(0, |m| m.len());
    println!(
        "{} : {} images, {:.0} Mo, en {:.0} s",
        sortie.display(),
        app.store.project.toutes_les_images().count(),
        poids as f64 / 1e6,
        debut.elapsed().as_secs_f64()
    );
}
