//! Le zoom proche : ce que coute une photo qui remplit l'ecran, et les autres sous elle.
//!
//! # Le cas mesure sur le terrain
//!
//! La chronique d'une session reelle a donne, en zoom :
//!
//! ```text
//!   35.0s   858.07ms zoomer   noeuds 112  photos 27  ecrans 1408.2x  cache 667 Mo
//!           dont images 844.71ms
//! ```
//!
//! Vingt-sept photos couvrant **mille quatre cents fois** la surface de l'ecran. En zoom
//! proche, une seule remplit la fenetre : les vingt-six autres sont dessinees sous elle, et
//! rien de ce qu'elles peignent n'atteint l'oeil.
//!
//! Ce banc reproduit exactement cela, et mesure ce que l'occlusion (OCCLUSION-1) en retire.

use glucose_core::store::Store;
use glucose_core::types::BoardImage;
use glucose_desktop::params::SceneOverlay;
use glucose_desktop::renderer::Renderer;
use glucose_desktop::ui::UiState;
use std::time::Instant;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (1920, 1080);

/// Une photo de test. `alpha` decide si elle peut cacher ce qu'il y a dessous : l'opacite est
/// l'une des trois conditions d'OCCLUSION-1, et elle se constate pixel par pixel au decodage.
fn photo(dossier: &std::path::Path, nom: &str, w: u32, h: u32, alpha: u8) -> String {
    let chemin = dossier.join(nom);
    if !chemin.exists() {
        let mut brute = image::RgbaImage::new(w, h);
        for (x, y, px) in brute.enumerate_pixels_mut() {
            *px = image::Rgba([(x % 256) as u8, (y % 256) as u8, 180, alpha]);
        }
        brute.save(&chemin).expect("ecriture de la photo");
    }
    chemin.to_string_lossy().to_string()
}

/// `combien` photos empilees au meme endroit, vues a l'echelle `zoom`.
///
/// Empilees et non cote a cote : c'est la situation du zoom proche, ou tout ce qui reste
/// visible se superpose.
fn document(src: &str, combien: usize, zoom: f64) -> Store {
    let mut store = Store::new("Occlusion");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport.scale = zoom;
        b.viewport.x = ECRAN.0 as f64 / 2.0;
        b.viewport.y = ECRAN.1 as f64 / 2.0;
    }
    for i in 0..combien {
        // Un leger decalage, comme des photos posees a la main les unes sur les autres.
        let (cx, cy) = (i as f64 * 3.0, i as f64 * 2.0);
        let mut img = BoardImage::new(format!("p{i}"), cx, cy, 1600.0, 1000.0);
        img.src = Some(src.to_string());
        store.add_image(&board, img);
    }
    store
}

/// Ce qu'une mesure rend : la duree, et ce que la scene a REELLEMENT pose.
///
/// Sans les quantites, une duree basse est ambigue -- elle peut vouloir dire "c'est rapide"
/// ou "rien n'a ete dessine". Un banc qui ne les rapporte pas se trompe en silence, et c'est
/// arrive ici meme : ce banc annoncait un gain nul parce qu'il ne posait aucune photo.
struct Mesure {
    ms: f64,
    posees: f64,
    ecrans: f64,
    cachees: f64,
}

fn chronometrer(store: &Store) -> Mesure {
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    ui.current_toast = None;
    let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("pixmap");
    let guides = glucose_core::smart_align::SnapGuides::default();
    let mut mesures = Vec::new();

    for i in 0..15 {
        glucose_desktop::perf::frame_begin();
        let t = Instant::now();
        renderer.rendre_la_scene(
            &mut pixmap.as_mut(),
            store,
            &ui,
            SceneOverlay {
                guides: &guides,
                selection_box: None,
                editing: None,
            },
            ui.header_height(),
        );
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        // Les photos arrivent par les fils de fond : on attend qu'elles soient la, puis on
        // mesure. Sans cela, on chronometrerait des cadres de remplacement.
        if renderer.magasin.en_travail() > 0 {
            renderer.magasin.attendre_le_chantier();
            continue;
        }
        if i >= 3 {
            mesures.push(ms);
        }
    }
    mesures.sort_by(f64::total_cmp);
    let lire = |nom: &str| glucose_desktop::perf::valeur_du_compteur(nom).unwrap_or(0.0);
    Mesure {
        ms: mesures[mesures.len() / 2],
        posees: lire("img_n"),
        ecrans: lire("img_ecrans"),
        cachees: lire("img_cachees"),
    }
}

fn main() {
    let dossier = std::env::temp_dir().join("glucose-bench-occlusion");
    std::fs::create_dir_all(&dossier).expect("dossier");
    let opaque = photo(&dossier, "empilee-opaque.png", 2400, 1500, 255);
    // Une photo translucide ne peut RIEN cacher : le fond doit transparaitre a travers elle.
    // Elle mesure donc exactement ce que coutait la scene avant OCCLUSION-1, sans qu'aucun
    // interrupteur ni aucun code mort n'ait a exister pour cela.
    let voile = photo(&dossier, "empilee-voile.png", 2400, 1500, 200);

    println!(
        "Le zoom proche — {} × {}, des photos empilees\n",
        ECRAN.0, ECRAN.1
    );
    println!(
        "  {:>7} {:>6} {:>13} {:>8} {:>13} {:>8} {:>9}",
        "photos", "zoom", "sans occlure", "posees", "avec occlure", "posees", "rapport"
    );

    for zoom in [0.25f64, 1.0, 4.0] {
        for combien in [1usize, 8, 27] {
            let sans = chronometrer(&document(&voile, combien, zoom));
            let avec = chronometrer(&document(&opaque, combien, zoom));
            println!(
                "  {combien:>7} {zoom:>6.2} {:>11.2}ms {:>8.0} {:>11.2}ms {:>8.0} {:>8.1}x",
                sans.ms,
                sans.posees,
                avec.ms,
                avec.posees,
                sans.ms / avec.ms.max(0.001)
            );
        }
        println!();
    }
    let temoin = chronometrer(&document(&voile, 27, 1.0));
    println!(
        "  Au zoom 1, vingt-sept photos translucides couvrent {:.1} fois l'ecran ; 
           l'occlusion en cache {:.0}.
",
        temoin.ecrans, temoin.cachees
    );

    println!(
        "  « sans occlure » utilise des photos translucides, qui ne peuvent rien cacher :\n  \
         c'est exactement ce que la scene coutait avant OCCLUSION-1. Le rapport est donc\n  \
         mesure entre deux chemins reels, et non estime."
    );
}
