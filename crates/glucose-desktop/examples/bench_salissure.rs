//! Le banc de la **salissure** — où va le temps d'une image pendant un glissement, poste par
//! poste, chrome et téléversement compris ; et ce qui, dans cette image, était **identique** à
//! la précédente.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_salissure
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! La chronique du 20/09, sur 429 photos, lisait un rendu à dix millisecondes en médiane —
//! deux fois ce qu'un balayage double permet à 240 Hz — et « 98 % des images redessinent ».
//! Elle donnait les postes en parts, jamais en millisecondes, et elle ne pouvait pas dire ce
//! qui, dans ces dix millisecondes, était **du travail refait à l'identique** : la barre, les
//! onglets, la minimap, les panneaux, qui n'ont pas bougé entre deux images d'un glissement.
//!
//! Ce banc rejoue le glissement de `bench_freinage` sur une scène chargée, par le chemin de
//! l'application — la scène, l'interface, les panneaux, puis une copie plein écran qui tient
//! lieu du téléversement — et rend chaque poste en millisecondes. Puis il compare, image après
//! image, la bande de la chrome à celle de l'image précédente : chaque image où elle est
//! identique est une image où son coût a été payé pour rien.
//!
//! C'est le juge de l'étape 1 du plan 18 : le coût de base doit descendre sous la période du
//! balayage double, et la part refaite à l'identique doit tomber à zéro.

use glucose_core::store::Store;
use glucose_core::types::{BoardImage, Viewport};
use glucose_desktop::bench;
use glucose_desktop::dock::{render_docks, DockCache, DockManager, DockPass};
use glucose_desktop::interactions::tools::text_card;
use glucose_desktop::params::{Pointer, SceneOverlay, ScreenFrame};
use glucose_desktop::renderer::{Regard, Renderer};
use glucose_desktop::ui::UiState;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (2560, 1600);
/// Le nombre de photos et de cartes se passe en argument : `bench_salissure 429 0` rejoue le
/// document de l'utilisateur, `bench_salissure 120 20` une scène de cartes.
const PHOTOS_PAR_DEFAUT: usize = 120;
const CARTES_PAR_DEFAUT: usize = 20;
const TAILLE_POSEE: (f64, f64) = (420.0, 315.0);
const IMAGES: usize = 120;
const VITESSE: f64 = 1_200.0;
const TAU: f64 = 0.45;

/// La période d'un balayage double à 240 Hz : ce qu'une image doit coûter, présentation
/// comprise, pour que le tempo tienne deux balayages.
const DEUX_BALAYAGES_MS: f64 = 2.0 * 1000.0 / 240.0;

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

/// Un mur de photos, et des cartes de texte semées dessus : la scène d'un utilisateur.
fn document(chemin: &Path, renderer: &Renderer, photos: usize, cartes: usize) -> Store {
    let mut store = Store::new("salissure");
    let board = store.project.active_board_id.clone();
    let colonnes = (photos as f64).sqrt().ceil() as usize;
    for i in 0..photos {
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
    for i in 0..cartes {
        let carte = text_card(
            &renderer.typography,
            &renderer.math,
            format!("carte-{i}"),
            (i % 5) as f64 * 900.0 + 100.0,
            (i / 5) as f64 * 700.0 + 150.0,
            format!("Carte {i} — une note posée sur le mur, avec **du gras** et du texte."),
        );
        store.add_annotation(&board, carte);
    }
    store.clear_selection();
    store
}

/// La bande de la chrome en haut de l'écran : la barre et les onglets, pleine largeur.
fn bande_du_haut(pixmap: &Pixmap, hauteur: u32) -> &[u8] {
    &pixmap.data()[..(pixmap.width() * hauteur * 4) as usize]
}

fn centile(v: &[f64], p: f64) -> f64 {
    let mut t = v.to_vec();
    t.sort_by(f64::total_cmp);
    t[((t.len() as f64 * p) as usize).min(t.len() - 1)]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let photos: usize = args
        .next()
        .and_then(|a| a.parse().ok())
        .unwrap_or(PHOTOS_PAR_DEFAUT);
    let cartes: usize = args
        .next()
        .and_then(|a| a.parse().ok())
        .unwrap_or(CARTES_PAR_DEFAUT);
    let dossier = std::env::temp_dir().join("glucose-bench-photos");
    std::fs::create_dir_all(&dossier).expect("dossier temporaire");
    let chemin = ecrire_photo(&dossier);

    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    ui.current_toast = None;
    let mut store = document(&chemin, &renderer, photos, cartes);
    let dock_manager = DockManager::new();
    let dock_cache = DockCache::default();
    let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("l'ecran");
    let mut sortie = Pixmap::new(ECRAN.0, ECRAN.1).expect("la copie");
    let board = store.project.active_board_id.clone();
    let header_h = ui.header_height().ceil() as u32;

    let mut vue = Viewport {
        x: 200.0,
        y: 200.0,
        scale: 1.0,
    };
    store.set_viewport(&board, vue);
    // Les photos se décodent une fois, hors mesure ; puis deux images de chauffe, pour que
    // la première image d'un régime — celle qui peint toutes ses tuiles — ne soit pas comptée.
    bench::render_into(&mut renderer, &mut ui, &store, &mut pixmap);
    renderer.magasin.attendre_le_chantier();
    bench::render_into(&mut renderer, &mut ui, &store, &mut pixmap);
    let regard = Regard {
        degradation_permise: false,
        en_mouvement: true,
    };
    let guides = glucose_core::smart_align::SnapGuides::default();
    let pointer = Pointer { x: -1.0, y: -1.0 };
    let ecran = ScreenFrame {
        width: ECRAN.0 as f32,
        height: ECRAN.1 as f32,
        header_h: ui.header_height(),
        scale: ui.scale_factor,
    };

    let periode = 1.0 / 240.0;
    // Les postes de chaque image, image par image : un poste absent d'une image vaut ZÉRO
    // pour celle-ci, sinon sa médiane ne dirait que ce qu'il coûte quand il est là.
    let mut images: Vec<Vec<(&'static str, f64)>> = Vec::with_capacity(IMAGES);
    let mut totaux = Vec::with_capacity(IMAGES);
    let mut chrome_identique = 0usize;
    let mut comparees = 0usize;
    let mut precedente: Option<Vec<u8>> = None;
    for i in 0..IMAGES + 2 {
        let t = i as f64 * periode;
        let v = VITESSE * (-t / TAU).exp();
        vue.x -= v * periode;
        vue.y -= v * periode * 0.4;
        store.set_viewport(&board, vue);
        let overlay = SceneOverlay {
            guides: &guides,
            selection_box: None,
            editing: None,
        };

        glucose_desktop::perf::frame_begin();
        let t0 = std::time::Instant::now();
        renderer.render(
            &mut pixmap.as_mut(),
            &store,
            &mut ui,
            overlay,
            pointer,
            regard,
        );
        render_docks(
            &mut pixmap.as_mut(),
            &dock_manager,
            &store,
            &DockPass {
                typo: &renderer.typography,
                theme: &renderer.theme,
                screen: ecran,
                pointer,
                cache: Some(&dock_cache),
            },
        );
        glucose_desktop::perf::stage("docks");
        // Le téléversement, tel que `write_texture` le paie : une copie de l'image entière.
        sortie.data_mut().copy_from_slice(pixmap.data());
        glucose_desktop::perf::stage("blit");
        let total = t0.elapsed().as_secs_f64() * 1000.0;
        glucose_desktop::perf::frame_end();

        if i < 2 {
            continue;
        }
        totaux.push(total);
        images.push(glucose_desktop::perf::postes());
        let bande = bande_du_haut(&pixmap, header_h).to_vec();
        if let Some(avant) = &precedente {
            comparees += 1;
            if *avant == bande {
                chrome_identique += 1;
            }
        }
        precedente = Some(bande);
    }

    let mut postes: BTreeMap<&'static str, Vec<f64>> = BTreeMap::new();
    for image in &images {
        for (nom, _) in image {
            postes.entry(nom).or_insert_with(|| vec![0.0; images.len()]);
        }
    }
    for (i, image) in images.iter().enumerate() {
        for (nom, ms) in image {
            if let Some(v) = postes.get_mut(nom) {
                v[i] = *ms;
            }
        }
    }

    println!(
        "Banc de la salissure — {photos} photos et {cartes} cartes, {} x {}, {IMAGES} images \
         de glissade\n",
        ECRAN.0, ECRAN.1
    );
    println!(
        "  {:<14} {:>9} {:>9} {:>7}",
        "poste", "median", "p90", "part"
    );
    let total_median = centile(&totaux, 0.5);
    let mut lignes: Vec<(&str, f64, f64)> = postes
        .iter()
        .map(|(nom, v)| (*nom, centile(v, 0.5), centile(v, 0.9)))
        .filter(|(_, med, _)| *med >= 0.005)
        .collect();
    lignes.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (nom, med, p90) in &lignes {
        println!(
            "  {:<14} {:>7.2}ms {:>7.2}ms {:>5.0} %",
            nom,
            med,
            p90,
            100.0 * med / total_median
        );
    }
    println!(
        "  {:<14} {:>7.2}ms {:>7.2}ms",
        "TOTAL",
        total_median,
        centile(&totaux, 0.9)
    );
    println!(
        "\n  Deux balayages a 240 Hz valent {DEUX_BALAYAGES_MS:.2} ms : le total doit tenir \
         dessous, marge comprise, pour que le tempo s'y cale."
    );
    println!(
        "  La bande de la chrome (les {header_h} px du haut) est identique a l'image precedente \
         sur {chrome_identique}/{comparees} images : son cout est refait a l'identique."
    );
}
