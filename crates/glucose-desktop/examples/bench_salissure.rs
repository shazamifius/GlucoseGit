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
/// De combien le mur commence avant l'origine, pour que la vue reste à l'intérieur.
const DEBORD: f64 = 2_000.0;
const IMAGES: usize = 120;
const VITESSE: f64 = 1_200.0;
const TAU: f64 = 0.45;
/// La vitesse d'un pincement, en octaves par seconde : deux octaves en une demi-seconde, ce
/// qui franchit deux niveaux de tuiles -- le cas qui fait peindre le plus.
const OCTAVES_PAR_SECONDE: f64 = 4.0;

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
/// `ecart` est l'espace entre deux photos : à zéro, elles **pavent** l'écran, et le fond
/// n'a plus aucune chance d'être vu — c'est le seul régime où la grille peut le sauter.
fn document(chemin: &Path, renderer: &Renderer, photos: usize, cartes: usize, ecart: f64) -> Store {
    let mut store = Store::new("salissure");
    let board = store.project.active_board_id.clone();
    let colonnes = (photos as f64).sqrt().ceil() as usize;
    for i in 0..photos {
        let (ligne, colonne) = (i / colonnes, i % colonnes);
        // Le mur déborde largement de ce que la glissade parcourt : sinon l'écran voit son
        // bord, et une tuile à cheval sur le vide n'est pas couverte -- ce qui est exact, et
        // ne mesure pas le cas qu'on veut mesurer.
        let mut img = BoardImage::new(
            format!("photo-{i}"),
            colonne as f64 * (TAILLE_POSEE.0 + ecart) - DEBORD,
            ligne as f64 * (TAILLE_POSEE.1 + ecart) - DEBORD,
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

/// Ce qu'une passe du banc a besoin d'emprunter — regroupé parce que ces champs voyagent
/// toujours ensemble, et qu'une fonction qui les recevrait un par un aurait dix arguments
/// dont l'ordre serait la seule protection.
struct Scene<'a> {
    renderer: &'a mut Renderer,
    ui: &'a mut UiState,
    store: &'a mut Store,
    pixmap: &'a mut Pixmap,
    sortie: &'a mut Pixmap,
    dock_manager: &'a DockManager,
    dock_cache: &'a DockCache,
    ecran: ScreenFrame,
    board: &'a str,
    depart: Viewport,
    /// Le geste joué : un zoom qui s'éteint, ou un glissement qui s'éteint.
    zoom: bool,
}

/// Ce qu'un régime a donné : ses images, leurs postes, et ce que la chrome y a répété.
#[derive(Default)]
struct Regime {
    totaux: Vec<f64>,
    /// Combien d'images n'ont pas eu à peindre leur fond.
    fond_saute: usize,
    /// Combien de tuiles chaque image a dû peindre : c'est le pic qui fait rater le tempo.
    tuiles: Vec<f64>,
    images: Vec<Vec<(&'static str, f64)>>,
    chrome_identique: usize,
    comparees: usize,
}

/// Rejoue le glissement une fois, et rend ce qu'il a coûté.
///
/// `en_cache` dit si la bande du haut a le droit de se souvenir. Sinon son cache est vidé
/// avant chaque image : c'est le régime d'hier, celui qui redessine la chrome à chaque fois.
fn jouer(scene: &mut Scene<'_>, regard: Regard, en_cache: bool) -> Regime {
    let periode = 1.0 / 240.0;
    let guides = glucose_core::smart_align::SnapGuides::default();
    let pointer = Pointer { x: -1.0, y: -1.0 };
    let mut regime = Regime::default();
    let mut precedente: Option<Vec<u8>> = None;
    let header_h = scene.ui.header_height().ceil() as u32;
    let mut vue = scene.depart;
    scene.store.set_viewport(scene.board, vue);
    for i in 0..IMAGES + 2 {
        let t = i as f64 * periode;
        let v = VITESSE * (-t / TAU).exp();
        if scene.zoom {
            // Un pincement qui s'éteint : l'échelle avance en OCTAVES, parce que c'est en
            // octaves que les niveaux de tuiles changent -- et c'est le franchissement d'une
            // octave qui fait peindre une colonne entière d'un coup.
            let octaves = OCTAVES_PAR_SECONDE * (-t / TAU).exp() * periode;
            let avant = vue.scale;
            vue.scale *= octaves.exp2();
            // Le zoom se fait au centre de l'écran : ce que la main fait d'un pincement.
            let centre = (f64::from(ECRAN.0) / 2.0, f64::from(ECRAN.1) / 2.0);
            let facteur = vue.scale / avant;
            vue.x = centre.0 - (centre.0 - vue.x) * facteur;
            vue.y = centre.1 - (centre.1 - vue.y) * facteur;
        } else {
            vue.x -= v * periode;
            vue.y -= v * periode * 0.4;
        }
        scene.store.set_viewport(scene.board, vue);
        let overlay = SceneOverlay {
            guides: &guides,
            selection_box: None,
            editing: None,
        };
        if !en_cache {
            scene.ui.bande_cache = None;
        }

        glucose_desktop::perf::frame_begin();
        let t0 = std::time::Instant::now();
        scene.renderer.render(
            &mut scene.pixmap.as_mut(),
            scene.store,
            scene.ui,
            overlay,
            pointer,
            regard,
        );
        render_docks(
            &mut scene.pixmap.as_mut(),
            scene.dock_manager,
            scene.store,
            &DockPass {
                typo: &scene.renderer.typography,
                theme: &scene.renderer.theme,
                screen: scene.ecran,
                pointer,
                cache: Some(scene.dock_cache),
            },
        );
        glucose_desktop::perf::stage("docks");
        // Le téléversement, tel que `write_texture` le paie : une copie de l'image entière.
        scene.sortie.data_mut().copy_from_slice(scene.pixmap.data());
        glucose_desktop::perf::stage("blit");
        let total = t0.elapsed().as_secs_f64() * 1000.0;
        glucose_desktop::perf::frame_end();

        if i < 2 {
            continue;
        }
        regime.totaux.push(total);
        regime.images.push(glucose_desktop::perf::postes());
        regime
            .tuiles
            .push(glucose_desktop::perf::valeur_du_compteur("tuiles_peintes").unwrap_or(0.0));
        if glucose_desktop::perf::valeur_du_compteur("fond_saute").unwrap_or(0.0) > 0.0 {
            regime.fond_saute += 1;
        }
        let bande = bande_du_haut(scene.pixmap, header_h).to_vec();
        if let Some(avant) = &precedente {
            regime.comparees += 1;
            if *avant == bande {
                regime.chrome_identique += 1;
            }
        }
        precedente = Some(bande);
    }
    regime
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
    // L'écart entre deux photos : `bench_salissure 429 0 0` les fait paver l'écran.
    let ecart: f64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(40.0);
    // Le geste : `zoom` joue un pincement qui s'éteint, sinon un glissement.
    let zoom = args.next().is_some_and(|a| a == "zoom");
    let dossier = std::env::temp_dir().join("glucose-bench-photos");
    std::fs::create_dir_all(&dossier).expect("dossier temporaire");
    let chemin = ecrire_photo(&dossier);

    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    ui.current_toast = None;
    let mut store = document(&chemin, &renderer, photos, cartes, ecart);
    let dock_manager = DockManager::new();
    let dock_cache = DockCache::default();
    let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("l'ecran");
    let mut sortie = Pixmap::new(ECRAN.0, ECRAN.1).expect("la copie");
    let board = store.project.active_board_id.clone();
    let header_h = ui.header_height().ceil() as u32;

    let depart = Viewport {
        x: 200.0,
        y: 200.0,
        scale: 1.0,
    };
    store.set_viewport(&board, depart);
    // Les photos se décodent une fois, hors mesure ; puis deux images de chauffe, pour que
    // la première image d'un régime — celle qui peint toutes ses tuiles — ne soit pas comptée.
    bench::render_into(&mut renderer, &mut ui, &store, &mut pixmap);
    renderer.magasin.attendre_le_chantier();
    bench::render_into(&mut renderer, &mut ui, &store, &mut pixmap);
    let regard = Regard {
        degradation_permise: false,
        en_mouvement: true,
    };
    let _guides = glucose_core::smart_align::SnapGuides::default();
    let _pointer = Pointer { x: -1.0, y: -1.0 };
    let ecran = ScreenFrame {
        width: ECRAN.0 as f32,
        height: ECRAN.1 as f32,
        header_h: ui.header_height(),
        scale: ui.scale_factor,
    };

    // **Les deux régimes se mesurent dans la MÊME exécution, et alternés.** Trois lancements
    // du même banc ont donné 2,3, 6,2 et 5,7 ms de médiane : la fréquence de la machine varie
    // plus que ce qu'on mesure. Comparer deux exécutions, c'est comparer le bruit — et c'est
    // la faute que ce dépôt a déjà payée deux fois (fiche 19 § 4).
    let mut mesures: BTreeMap<bool, Regime> = BTreeMap::new();
    for tour in 0..2 {
        // L'ordre s'inverse d'un tour à l'autre : ce qui passe en premier paie le
        // réchauffement des caches, et l'alternance le partage équitablement.
        for en_cache in [tour == 0, tour != 0] {
            let regime = jouer(
                &mut Scene {
                    renderer: &mut renderer,
                    ui: &mut ui,
                    store: &mut store,
                    pixmap: &mut pixmap,
                    sortie: &mut sortie,
                    dock_manager: &dock_manager,
                    dock_cache: &dock_cache,
                    ecran,
                    board: &board,
                    depart,
                    zoom,
                },
                regard,
                en_cache,
            );
            let entree = mesures.entry(en_cache).or_default();
            entree.totaux.extend(regime.totaux);
            entree.images.extend(regime.images);
            entree.chrome_identique += regime.chrome_identique;
            entree.fond_saute += regime.fond_saute;
            entree.tuiles.extend(regime.tuiles);
            entree.comparees += regime.comparees;
        }
    }
    let avec = mesures.remove(&true).unwrap_or_default();
    let sans = mesures.remove(&false).unwrap_or_default();
    let images = avec.images;
    let totaux = avec.totaux;
    let chrome_identique = avec.chrome_identique;
    let comparees = avec.comparees;
    let saute = if totaux.is_empty() {
        0
    } else {
        100 * avec.fond_saute / totaux.len()
    };

    // Un poste absent d'une image y vaut ZÉRO, et non « absent » : sinon sa médiane ne dirait
    // que ce qu'il coûte quand il est là.
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

    let geste = if zoom { "pincement" } else { "glissade" };
    println!(
        "Banc de la salissure — {photos} photos et {cartes} cartes, {} x {}, {IMAGES} images \
         de {geste}, {ecart:.0} px entre les photos
",
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
    let sans_median = centile(&sans.totaux, 0.5);
    println!(
        "\n  Chrome REDESSINEE a chaque image : {:>7.2}ms median, {:>7.2}ms p90 -- soit \
         {:+.2}ms par image",
        sans_median,
        centile(&sans.totaux, 0.9),
        sans_median - total_median
    );
    println!(
        "\n  Deux balayages a 240 Hz valent {DEUX_BALAYAGES_MS:.2} ms : le total doit tenir \
         dessous, marge comprise, pour que le tempo s'y cale."
    );
    println!(
        "  La bande de la chrome (les {header_h} px du haut) est identique a l'image precedente \
         sur {chrome_identique}/{comparees} images : son cout est refait a l'identique."
    );
    let tuiles = avec.tuiles;
    // L'image la plus chère, et ce qu'elle peignait : c'est la seule façon de savoir si le
    // pic du temps EST le pic des tuiles, au lieu de le supposer.
    if let Some((i, pire)) = totaux.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)) {
        let sans_pic: Vec<f64> = totaux
            .iter()
            .zip(&tuiles)
            .filter(|(_, t)| **t == 0.0)
            .map(|(d, _)| *d)
            .collect();
        println!(
            "
  L'image la plus chere : {:.2}ms, et elle peignait {:.0} tuiles. Les images              qui n'en peignent AUCUNE : {:.2}ms median, {:.2}ms p99.",
            pire,
            tuiles.get(i).copied().unwrap_or(0.0),
            centile(&sans_pic, 0.5),
            centile(&sans_pic, 0.99)
        );
    }
    println!(
        "  Tuiles peintes par image : median {:.0}, p90 {:.0}, p99 {:.0}, pire {:.0} -- c'est          le PIC qui fait rater le tempo, jamais la mediane.",
        centile(&tuiles, 0.5),
        centile(&tuiles, 0.9),
        centile(&tuiles, 0.99),
        centile(&tuiles, 1.0)
    );
    println!(
        "  Le fond n'a pas ete peint sur {saute} % des images : les tuiles le recouvraient \
         entierement, et `clear` comme `grid` disparaissent alors du profil."
    );
}
