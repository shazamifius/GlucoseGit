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

/// Ce que le rendu des textures manquantes s'autorise par image, quand la cascade joue.
///
/// Ce n'est pas un choix du banc : c'est le plancher de la charte moins ce qu'une image
/// coute, et c'est ainsi que l'application le calcule (`Cadence::tranche_de_fond`). L'image
/// de reference coute ici une milliseconde, donc il en reste neuf.
const BUDGET: std::time::Duration = std::time::Duration::from_millis(9);

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
    /// Combien de textures de composants cette image a dû rendre parce que la carte ne les
    /// connaissait pas — le pic que la fiche 22 § 11.2 annonce à trente millisecondes.
    rendues: Vec<f64>,
    postes: BTreeMap<&'static str, Vec<f64>>,
}

/// Joue `IMAGES` images le long de `vue`, par la **voie graphique** — celle de l'utilisateur.
///
/// Le processeur y produit les deux couches et ce qu'il confie à la carte ; le banc ne
/// compose pas, parce que c'est le coût **processeur** qu'il mesure, et que la chronique
/// chiffre la composition à part.
fn jouer(
    renderer: &mut Renderer,
    store: &mut Store,
    vue: impl Fn(usize) -> Viewport,
    budget: std::time::Duration,
    connues: &mut std::collections::HashMap<String, String>,
) -> Geste {
    let board = store.project.active_board_id.clone();
    let mut dessous = Pixmap::new(ECRAN.0, ECRAN.1).expect("un pixmap");
    let mut dessus = Pixmap::new(ECRAN.0, ECRAN.1).expect("un pixmap");
    let mut ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let mut geste = Geste {
        durees: Vec::new(),
        rasterises: Vec::new(),
        rendues: Vec::new(),
        postes: BTreeMap::new(),
    };
    // La memoire de la carte graphique : ce qu'elle detient, et ce qu'elle oublie a la fin
    // d'une image ou cela n'a pas servi -- la meme loi que `SceneGpu`.
    for i in 0..IMAGES {
        store.set_viewport(&board, vue(i));
        let avant = renderer.typography.cached_glyph_count();
        glucose_desktop::perf::frame_begin();
        let debut = Instant::now();
        dessus.fill(tiny_skia::Color::TRANSPARENT);
        glucose_desktop::perf::stage("effacer");
        let confie = renderer.rendre_les_couches(
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
        // Ce que la carte ne connait pas se rend maintenant, comme la presentation le fait --
        // et dans le MEME ordre qu'elle : d'abord ce qui manque entierement, ensuite ce qui a
        // vieilli, et rien au-dela du budget (CASCADE-2).
        let debut_des_textures = Instant::now();
        let mut rendues = 0.0f64;
        let a_poser = confie.textures();
        for urgent in [true, false] {
            for t in &a_poser {
                if connues.get(&t.identite).is_some_and(|c| *c == t.cle) {
                    continue;
                }
                if connues.contains_key(&t.identite) == urgent {
                    continue;
                }
                if !urgent && debut_des_textures.elapsed() >= budget {
                    continue;
                }
                if let Some(c) = confie.composant(&t.cle) {
                    // Le rendu est ce qu'on mesure ; la texture elle-meme, la carte la garde.
                    if c.rendre(renderer.kit()).is_some() {
                        rendues += 1.0;
                        connues.insert(t.identite.clone(), t.cle.clone());
                    }
                }
            }
        }
        // La memoire oublie ce qui n'a pas servi, par IDENTITE : un composant garde son
        // ancien palier tant que le nouveau n'est pas pret.
        let servies: std::collections::HashSet<&String> =
            a_poser.iter().map(|t| &t.identite).collect();
        connues.retain(|id, _| servies.contains(id));
        glucose_desktop::perf::compteur("textures_rendues", rendues);
        glucose_desktop::perf::stage("textures");
        geste.rendues.push(rendues);
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
        "  {nom:<12} image  med {:6.2} ms  p90 {:6.2}  p99 {:6.2}  pire {:6.2}   |  rasterises  med {:5.0}  pire {:5.0}   |  textures rendues  med {:5.0}  pire {:5.0}",
        centile(&geste.durees, 0.5),
        centile(&geste.durees, 0.9),
        centile(&geste.durees, 0.99),
        centile(&geste.durees, 1.0),
        centile(&r, 0.5),
        centile(&r, 1.0),
        centile(&geste.rendues, 0.5),
        centile(&geste.rendues, 1.0),
    );
    // Les postes, du plus lourd au plus leger en mediane -- et jamais sommes (fiche 20 § 4.5).
    let mut postes: Vec<(&str, f64, f64, f64)> = geste
        .postes
        .iter()
        .map(|(n, v)| (*n, centile(v, 0.5), centile(v, 0.99), centile(v, 1.0)))
        // **Le filtre porte sur le pire, pas sur la médiane.** Un poste dont la médiane est
        // nulle et le pire vaut trente millisecondes est précisément celui qu'on cherche ici,
        // et filtrer sur la médiane le jetait — la faute de la fiche 20 § 4.5, à l'envers.
        .filter(|(_, med, _, pire)| *med >= 0.05 || *pire >= 0.5)
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

    // **Les deux regimes, alternes dans la MEME execution** (fiche 20 § 5.1) : sans budget,
    // toutes les textures manquantes se rendent d'un coup ; avec, ce qui ne rentre pas
    // attend l'image suivante en gardant son ancien palier pose.
    for (nom, budget) in [
        ("sans cascade", std::time::Duration::MAX),
        ("avec cascade", BUDGET),
    ] {
        println!("  {nom} :");
        // **La memoire de la carte survit d'un geste a l'autre**, comme dans l'application.
        // La reinitialiser ferait de chaque premiere image un pic legitime -- rien a garder,
        // donc rien a etaler -- et la cascade n'aurait rien a montrer.
        let mut connues: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        // Une image de chauffe a l'echelle 1 : le cache se remplit une fois.
        let g = jouer(
            &mut renderer,
            &mut store,
            |_| Viewport {
                x: 100.0,
                y: 100.0,
                scale: 1.0,
            },
            budget,
            &mut connues,
        );
        ligne("immobile", &g);

        // Le glissement : l'echelle ne bouge pas, la phase sous-pixel de chaque glyphe si.
        let g = jouer(
            &mut renderer,
            &mut store,
            |i| Viewport {
                x: 100.0 - i as f64 * 7.3,
                y: 100.0 - i as f64 * 2.1,
                scale: 1.0,
            },
            budget,
            &mut connues,
        );
        ligne("glissement", &g);

        // Le zoom : l'echelle avance d'un demi pour cent par image, donc la taille de
        // police change presque a chaque image.
        let g = jouer(
            &mut renderer,
            &mut store,
            |i| Viewport {
                x: 100.0,
                y: 100.0,
                scale: 1.0 + i as f64 * 0.005,
            },
            budget,
            &mut connues,
        );
        ligne("zoom", &g);

        // Le zoom par octaves : le pic le plus brutal, celui ou TOUTES les cartes changent
        // de texture sur une seule image.
        let g = jouer(
            &mut renderer,
            &mut store,
            |i| Viewport {
                x: 100.0,
                y: 100.0,
                scale: if i < 50 { 1.0 } else { 2.0 },
            },
            budget,
            &mut connues,
        );
        ligne("par paliers", &g);
        println!();
    }

    println!(
        "\n  cache de glyphes a la fin : {} variantes (plafond 4096)",
        renderer.typography.cached_glyph_count()
    );
}
