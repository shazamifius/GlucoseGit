//! La cadence **pendant** un import — la seule mesure qui dise ce que l'utilisateur vit.
//!
//! # Pourquoi aucun banc existant ne pouvait voir cela
//!
//! Tous les autres bancs attendent que les images soient décodées avant de mesurer, parce
//! qu'ils mesurent le *dessin*. Ils ne peuvent donc rien dire du moment qui compte : celui où
//! trente-six photos arrivent d'un coup et où l'application doit rester vivante.
//!
//! C'est exactement ce que l'utilisateur décrivait — « importer une image fait tout laguer » —
//! et c'est ce qu'aucune de nos mesures ne regardait.
//!
//! # Ce qui est mesuré
//!
//! Le document reçoit d'un coup des photos qu'aucun cache ne connaît, puis on rend en boucle
//! **sans jamais attendre**, jusqu'à ce que tout soit là. On regarde la pire image de la
//! série : c'est elle qui décide si le geste a été vécu comme fluide ou comme un gel.
//!
//! Avant l'invariant DECODE-1, la réponse était arithmétique : le décodage ayant lieu dans la
//! boucle de rendu, la première image portait le décodage de **toutes** les photos visibles,
//! soit plusieurs secondes pendant lesquelles la fenêtre ne répondait pas.

use glucose_core::store::Store;
use glucose_core::types::BoardImage;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::renderer::Renderer;
use glucose_desktop::ui::UiState;
use std::time::Instant;
use tiny_skia::Pixmap;

/// La fenêtre sur laquelle on mesure — celle du banc des photos, pour que les chiffres se
/// comparent d'un banc à l'autre.
const ECRAN: (u32, u32) = (2560, 1600);

fn photo(dossier: &std::path::Path, nom: &str, w: u32, h: u32) -> String {
    let chemin = dossier.join(nom);
    if !chemin.exists() {
        let mut brute = image::RgbaImage::new(w, h);
        for (x, y, px) in brute.enumerate_pixels_mut() {
            *px = image::Rgba([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8, 255]);
        }
        brute.save(&chemin).expect("écriture de la photo");
    }
    chemin.to_string_lossy().to_string()
}

/// Un document où `combien` photos viennent d'être posées en grille.
fn document(sources: &[String], combien: usize) -> Store {
    let mut store = Store::new("Import");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport.scale = 1.0;
        b.viewport.x = 0.0;
        b.viewport.y = 0.0;
    }
    let cote = (combien as f64).sqrt().ceil() as usize;
    for i in 0..combien {
        let (cx, cy) = ((i % cote) as f64 * 420.0, (i / cote) as f64 * 320.0);
        let mut img = BoardImage::new(format!("img-{i}"), cx, cy, 400.0, 300.0);
        img.src = Some(sources[i % sources.len()].clone());
        store.add_image(&board, img);
    }
    store
}

/// Rend une image, sans jamais attendre.
fn une_image(renderer: &mut Renderer, ui: &mut UiState, pixmap: &mut Pixmap, store: &Store) -> f64 {
    let guides = glucose_core::smart_align::SnapGuides::default();
    // `frame_begin` vit dans l'application, pas dans le rendu : sans ces deux appels, aucun
    // banc n'émet la moindre trace, et j'ai cherché un pic les mains vides.
    glucose_desktop::perf::frame_begin();
    let t = Instant::now();
    renderer.render(
        &mut pixmap.as_mut(),
        store,
        ui,
        SceneOverlay {
            guides: &guides,
            selection_box: None,
            editing: None,
        },
        Pointer { x: -1.0, y: -1.0 },
        false,
    );
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    glucose_desktop::perf::frame_end();
    ms
}

/// Rend sans jamais attendre, jusqu'à ce que tout soit décodé. Rend (pire image, nombre
/// d'images rendues, durée totale) en millisecondes.
///
/// # L'image de chauffe, et pourquoi elle est indispensable
///
/// La toute première image d'un `Renderer` neuf porte l'ouverture des polices, le cache de
/// glyphes et l'index spatial — des dizaines de millisecondes qui n'appartiennent à aucun
/// import. Sans chauffe, ce banc les attribuait à la photo, et il l'a fait : il annonçait
/// 86 ms pour une seule photo alors que l'import n'y était pour rien.
///
/// La chauffe se fait sur un document **vide**, pour qu'aucune image n'y soit déjà demandée.
fn importer(store: &Store) -> (f64, usize, f64) {
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("pixmap");

    let vide = Store::new("chauffe");
    une_image(&mut renderer, &mut ui, &mut pixmap, &vide);

    let depart = Instant::now();
    let mut pire = 0.0f64;
    let mut images = 0usize;

    // On rend tant qu'il reste du travail, plus une passe pour poser la dernière photo.
    loop {
        let reste = renderer.magasin.en_travail() > 0 || images == 0;
        let ms = une_image(&mut renderer, &mut ui, &mut pixmap, store);
        pire = pire.max(ms);
        images += 1;
        if !reste {
            break;
        }
        if depart.elapsed().as_secs() > 120 {
            break;
        }
    }
    (pire, images, depart.elapsed().as_secs_f64() * 1000.0)
}

fn main() {
    let dossier = std::env::temp_dir().join("glucose-bench-import");
    std::fs::create_dir_all(&dossier).expect("dossier des photos");
    let sources = vec![
        photo(&dossier, "import-12mpx.png", 4032, 3024),
        photo(&dossier, "import-2mpx.png", 1920, 1080),
        photo(&dossier, "import-05mpx.png", 800, 600),
    ];

    println!("La cadence PENDANT un import — {} × {}\n", ECRAN.0, ECRAN.1);
    println!(
        "  {:>7}  {:>14}  {:>10}  {:>12}  {:>10}",
        "photos", "pire image", "images", "décodage", "cadence"
    );

    for combien in [1usize, 4, 12, 36] {
        let store = document(&sources, combien);
        let (pire, images, total) = importer(&store);
        println!(
            "  {combien:>7}  {pire:>12.1}ms  {images:>10}  {total:>10.0}ms  {:>8.0} fps",
            1000.0 / pire.max(0.001)
        );
    }

    println!(
        "\n  « pire image » est ce que l'utilisateur ressent : une image à 3 000 ms est un gel.\n  \
         « images » compte les images rendues pendant que les photos arrivaient — avant\n  \
         DECODE-1 il y en avait UNE, et elle portait tout le décodage."
    );
}
