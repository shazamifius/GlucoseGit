//! Un écran composé de **tuiles** donne-t-il les mêmes pixels qu'un rendu direct ?
//!
//! # La question, et pourquoi elle vient avant le branchement
//!
//! TUILE-1 ne sert que si une tuile rendue seule, dans son propre repère, est identique au
//! morceau correspondant d'un rendu complet. Sinon la grille se voit — des coutures aux
//! frontières — et c'est le pire défaut possible : il apparaît **partout**, en permanence.
//!
//! Le soupçon est précis : le halo d'un nœud déborde de son rectangle. Une tuile qui ne
//! connaît que ce qui la traverse ne verra donc pas le halo d'un nœud voisin, resté juste
//! dehors. Ce banc mesure ce débord au lieu de le supposer — c'est exactement ce que
//! `bench_zone` a fait pour le rendu par région, et sa réponse (seize à quarante-huit pixels)
//! avait alors décidé de la marge.
//!
//! # Comment une tuile se rend
//!
//! Dans **son** repère : son coin à l'origine du pixmap, son échelle celle de son niveau. Elle
//! ne dépend donc d'aucune vue — c'est toute sa raison d'être, et `Cadrage::tuile` le dit.
//!
//! Avec un débord `d`, on rend un carré de `COTE + 2d` dont la tuile occupe le centre, puis on
//! ne garde que ce centre. Le banc essaie plusieurs débords et dit à partir duquel l'écart
//! s'éteint.

use glucose_core::store::Store;
use glucose_core::tuile::{Adresse, COTE};
use glucose_core::types::{Annotation, BoardImage, Viewport};
use glucose_desktop::params::SceneOverlay;
use glucose_desktop::renderer::{tuiles, Cadrage, Renderer};
use glucose_desktop::ui::UiState;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (1024, 768);

/// Les débords à essayer. Celui où l'écart s'éteint est la réponse.
const DEBORDS: [u32; 5] = [0, 4, 16, 48, 96];

/// Une scène qui a de tout — et surtout des nœuds proches des frontières de tuiles.
fn document() -> Store {
    let mut store = Store::new("Grille");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        };
    }
    for i in 0..16 {
        // Les positions tombent volontiers **à cheval** sur les frontières de tuiles : c'est
        // là que les coutures apparaîtraient.
        let (x, y) = ((i % 4) as f64 * 250.0 + 10.0, (i / 4) as f64 * 250.0 + 10.0);
        store.add_image(&board, BoardImage::new(format!("i{i}"), x, y, 200.0, 150.0));
        store.add_annotation(
            &board,
            Annotation::Text {
                id: format!("t{i}"),
                x: x + 20.0,
                y: y + 20.0,
                width: Some(160.0),
                height: Some(50.0),
                text: format!("carte {i}"),
                font_size: Some(14.0),
                color: None,
                cursor_pos: None,
                source_file: None,
                membrane_id: None,
                domains: Vec::new(),
                mirror_of: None,
                temporal_anchor: None,
            },
        );
    }
    // `add_image` selectionne ce qu'elle ajoute. Un cadre de selection n'appartient pas au
    // document : il se dessine par-dessus les tuiles, apres toutes les images, et non plus
    // entrelace avec elles. Le comparer a un rendu direct mesurerait cet ordre, pas la grille.
    store.clear_selection();
    store
}

fn interface() -> UiState {
    let mut ui = UiState::new();
    // Le mot d'accueil porte une horloge : il rendrait la comparaison non reproductible.
    ui.current_toast = None;
    ui
}

fn sans_reperes<'a>() -> SceneOverlay<'a> {
    // Une fuite volontaire et unique : les guides vivent le temps du banc, et leur donner une
    // durée de vie statique évite de les faire voyager dans chaque signature.
    let guides: &'static glucose_core::smart_align::SnapGuides =
        Box::leak(Box::new(glucose_core::smart_align::SnapGuides::default()));
    SceneOverlay {
        guides,
        selection_box: None,
        editing: None,
    }
}

/// L'écran rendu d'un bloc, comme aujourd'hui — la référence.
fn ecran_direct(store: &Store, vue: Viewport) -> Pixmap {
    let mut cadre = store.clone();
    if let Some(b) = cadre.active_board_mut() {
        b.viewport = vue;
    }
    let mut renderer = Renderer::new();
    let ui = interface();
    let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("l'ecran");
    renderer.sync_spatial_index(&cadre);
    renderer.rendre_la_scene(
        &mut pixmap.as_mut(),
        &cadre,
        &ui,
        sans_reperes(),
        0.0,
        glucose_desktop::renderer::Regard::immobile(),
    );
    pixmap
}

/// Une tuile, rendue seule dans son repère, avec `debord` pixels de marge tout autour.
fn rendre_une_tuile(
    renderer: &mut Renderer,
    store: &Store,
    ui: &UiState,
    adresse: Adresse,
    debord: u32,
) -> Pixmap {
    let cote = COTE + 2 * debord;
    let mut pixmap = Pixmap::new(cote, cote).expect("une tuile");
    let echelle = Adresse::echelle(adresse.niveau);
    let couverte = adresse.couvre();
    // Le coin de la tuile est à `debord` pixels du coin du pixmap.
    let origine = (
        couverte.left - f64::from(debord) / echelle,
        couverte.top - f64::from(debord) / echelle,
    );
    renderer.rendre_la_region(
        &mut pixmap.as_mut(),
        store,
        ui,
        sans_reperes(),
        0.0,
        Cadrage::tuile(adresse.niveau, origine),
    );
    pixmap
}

/// L'écran composé depuis les tuiles.
fn ecran_par_tuiles(store: &Store, vue: Viewport, debord: u32) -> (Pixmap, usize) {
    let mut renderer = Renderer::new();
    let ui = interface();
    renderer.sync_spatial_index(store);
    let mut ecran = Pixmap::new(ECRAN.0, ECRAN.1).expect("l'ecran");

    let (niveau, adresses) = tuiles::a_l_ecran(vue, (ECRAN.0 as f32, ECRAN.1 as f32));
    let echelle = Adresse::echelle(niveau);
    let mut posees = 0usize;
    for adresse in adresses {
        let rendue = rendre_une_tuile(&mut renderer, store, &ui, adresse, debord);
        // Le centre du carré rendu, c'est-à-dire la tuile elle-même.
        let mut centre = Pixmap::new(COTE, COTE).expect("le centre");
        for y in 0..COTE {
            let source = &rendue.data()[((y + debord) * rendue.width() + debord) as usize * 4..]
                [..COTE as usize * 4];
            centre.data_mut()[(y * COTE) as usize * 4..][..COTE as usize * 4]
                .copy_from_slice(source);
        }
        // Où cette tuile tombe à l'écran, sous cette vue.
        let couverte = adresse.couvre();
        let x = couverte.left * vue.scale + vue.x;
        let y = couverte.top * vue.scale + vue.y;
        glucose_desktop::composition::poser(
            &mut ecran.as_mut(),
            &centre,
            (x as f32, y as f32),
            glucose_core::report::Melange::Remplacer,
        );
        posees += 1;
    }
    let _ = echelle;
    (ecran, posees)
}

/// Combien de pixels diffèrent, et **où** : au bord de l'écran, ou sur une frontière ?
///
/// La distinction décide de la suite. Un écart sur les frontières de tuiles est une
/// **couture** — elle se verrait partout, en permanence, et interdirait l'architecture. Un
/// écart au bord de l'écran est un effet de pixmap tronqué, celui que `bench_zone` mesure
/// déjà et que le projet accepte.
fn comparer(a: &Pixmap, b: &Pixmap, vue: Viewport, niveau: i32) -> (usize, usize, usize, u8) {
    let (aa, _) = a.data().as_chunks::<4>();
    let (bb, _) = b.data().as_chunks::<4>();
    let largeur = a.width() as usize;
    let cote_ecran = Adresse::cote_monde(niveau) * vue.scale;
    let mut total = 0usize;
    let mut aux_frontieres = 0usize;
    let mut aux_bords = 0usize;
    // **L'amplitude, et pas seulement le compte.** Un pixel qui differe de un est un arrondi
    // a huit bits -- composer sur du transparent puis sur le fond n'arrondit pas exactement
    // comme composer sur le fond en une fois. Un pixel qui differe de cent est une couture.
    // Les compter ensemble a fait croire a un ecart la ou il n'y avait que de l'arrondi.
    let mut pire_ecart: u8 = 0;
    for (i, (x, y)) in aa.iter().zip(bb).enumerate() {
        if x == y {
            continue;
        }
        total += 1;
        let ecart = x
            .iter()
            .zip(y)
            .map(|(p, q)| p.abs_diff(*q))
            .max()
            .unwrap_or(0);
        pire_ecart = pire_ecart.max(ecart);
        let (px, py) = ((i % largeur) as f64, (i / largeur) as f64);
        let bord = px < 2.0
            || py < 2.0
            || px >= f64::from(ECRAN.0) - 2.0
            || py >= f64::from(ECRAN.1) - 2.0;
        // À moins de deux pixels d'une frontière de tuile ?
        let pres = |v: f64| {
            let dans = (v - vue.x).rem_euclid(cote_ecran);
            dans < 2.0 || dans > cote_ecran - 2.0
        };
        if bord {
            aux_bords += 1;
        } else if pres(px) || pres(py) {
            aux_frontieres += 1;
        }
    }
    (total, aux_frontieres, aux_bords, pire_ecart)
}

fn main() {
    println!("Un ecran compose de TUILES vaut-il un rendu direct ?\n");
    let store = document();
    // Une échelle exactement dyadique, et une position entière : c'est le régime de l'arrêt,
    // celui où la composition doit être identique AU BIT PRES.
    let vue = Viewport {
        x: -100.0,
        y: -60.0,
        scale: 1.0,
    };
    let direct = ecran_direct(&store, vue);
    let total = (ECRAN.0 * ECRAN.1) as f64;

    let niveau = Adresse::niveau_pour(vue.scale);
    println!(
        "  {:<10} {:>8} {:>11} {:>9} {:>12} {:>10} {:>12}",
        "debord", "tuiles", "differents", "part", "aux coutures", "aux bords", "pire ecart"
    );
    for debord in DEBORDS {
        let (compose, posees) = ecran_par_tuiles(&store, vue, debord);
        let (differents, coutures, bords, pire) = comparer(&direct, &compose, vue, niveau);
        println!(
            "  {:<10} {:>8} {:>11} {:>8.3}% {:>12} {:>10} {:>8}/255",
            format!("{debord} px"),
            posees,
            differents,
            100.0 * differents as f64 / total,
            coutures,
            bords,
            pire
        );
    }

    println!();
    println!("  Une couture se verrait PARTOUT et en permanence : c'est elle qui interdirait");
    println!("  l'architecture. Un ecart au bord de l'ecran est l'effet de pixmap tronque que");
    println!("  `bench_zone` mesure deja, et que le projet accepte.");
    println!();
    println!("  Le pire ecart dit ce que sont les pixels differents. A quatre sur 255, c'est");
    println!("  l'arrondi a huit bits de deux compositions -- sur du transparent puis sur le");
    println!("  fond, au lieu du fond en une fois -- et l'oeil ne le distingue pas. Une couture");
    println!("  se lirait a cent ou plus. Ce banc a annonce « zero ecart » tant qu'il ne");
    println!("  comptait que les pixels : il comptait alors mal les photos a cheval sur deux");
    println!("  tuiles, dont il prenait le centre pour le coin.");
    println!();

    // ── Ce que la grille fait gagner ────────────────────────────────────────
    println!(
        "  Ce que la grille fait gagner
"
    );

    // **Un renderer chaud des deux cotes.** La premiere version de ce banc appelait
    // `ecran_direct`, qui construit un `Renderer` neuf a chaque fois : elle comparait donc un
    // rendu aux caches froids -- glyphes, index spatial, magasin d'images -- a une grille
    // chaude, et annoncait un gain six fois trop grand. Un banc qui se trompe de reference est
    // pire qu'un banc absent.
    let mut cadre = store.clone();
    if let Some(b) = cadre.active_board_mut() {
        b.viewport = vue;
    }
    let mut direct_renderer = Renderer::new();
    direct_renderer.sync_spatial_index(&cadre);
    let ui_direct = interface();
    let mut direct_pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("l'ecran");
    let direct_ms = {
        let mut temps = Vec::new();
        for i in 0..9 {
            let t = std::time::Instant::now();
            direct_renderer.rendre_la_scene(
                &mut direct_pixmap.as_mut(),
                &cadre,
                &ui_direct,
                sans_reperes(),
                0.0,
                glucose_desktop::renderer::Regard::immobile(),
            );
            if i >= 2 {
                temps.push(t.elapsed().as_secs_f64() * 1000.0);
            }
        }
        temps.sort_by(|a, b| a.partial_cmp(b).unwrap());
        temps[temps.len() / 2]
    };

    // La grille, une fois chaude : c'est le régime normal, et celui qui compte.
    let mut renderer = Renderer::new();
    let ui = interface();
    renderer.sync_spatial_index(&store);
    let mut cache = tuiles::Tuiles::nouveau();
    let mut ecran = Pixmap::new(ECRAN.0, ECRAN.1).expect("l'ecran");
    let rangs: Vec<u32> = (0..store.active_board().map_or(0, |b| b.images.len() as u32)).collect();
    let mut une_image = || {
        cache.ouvrir();
        let (niveau, adresses) = tuiles::a_l_ecran(vue, (ECRAN.0 as f32, ECRAN.1 as f32));
        for adresse in adresses {
            let empreinte = tuiles::ce_que_porte(&store, adresse, &rangs);
            if cache.deja_peinte(empreinte).is_none() {
                let peinte = rendre_une_tuile(&mut renderer, &store, &ui, adresse, 0);
                cache.ranger(empreinte, peinte);
            }
            let Some((pixels, _)) = cache.deja_peinte(empreinte) else {
                continue;
            };
            let couverte = adresse.couvre();
            let _ = niveau;
            glucose_desktop::composition::poser(
                &mut ecran.as_mut(),
                pixels,
                (
                    (couverte.left * vue.scale + vue.x) as f32,
                    (couverte.top * vue.scale + vue.y) as f32,
                ),
                glucose_core::report::Melange::Remplacer,
            );
        }
        cache.fermer();
    };
    // La première image peint tout ; les suivantes ne doivent plus rien peindre.
    une_image();
    let mut temps = Vec::new();
    for _ in 0..7 {
        let t = std::time::Instant::now();
        une_image();
        temps.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    temps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let grille_ms = temps[temps.len() / 2];
    // La fermeture tient le cache emprunté ; on la laisse expirer avant de le relire.
    let _ = &une_image;
    let peintes = cache.peintes();
    let reprises = cache.reprises();

    println!("    rendu direct, a chaque image   {direct_ms:>8.3} ms");
    println!("    par la grille, une fois chaude {grille_ms:>8.3} ms");
    println!(
        "    soit                           {:>8.1}x moins cher",
        direct_ms / grille_ms
    );
    println!();
    println!("    tuiles peintes en tout, sur huit images : {peintes}");
    println!("    reprises : {reprises} — le travail que le cache a epargne");
}
