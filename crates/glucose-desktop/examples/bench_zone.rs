//! Rendre une **région** donne-t-il les mêmes pixels que rendre tout ? (préalable à A.1)
//!
//! # Pourquoi cette question avant toute autre
//!
//! Ne redessiner que ce qui a changé suppose qu'on sache redessiner *une partie* de l'écran
//! sans que le résultat diffère d'un rendu complet. Si ce n'est pas vrai, tout l'édifice de la
//! vague A s'écroule — et il vaut mieux l'apprendre sur vingt lignes de banc que sur trois
//! jours de réécriture.
//!
//! # Comment la région est rendue
//!
//! `world_to_screen` vaut `monde × échelle + vp`. Décaler la vue de `-x0` revient donc
//! exactement à déplacer l'origine de l'écran en `x0` : rendre la région `(x0, y0, w, h)` dans
//! une image de `w × h`, c'est rendre la scène entière avec `vp.x -= x0`.
//!
//! Aucune passe n'a besoin de le savoir, et c'est tout l'intérêt : la scène ignore qu'elle est
//! partielle.
//!
//! # Ce que ce banc rapporte
//!
//! Le nombre de pixels qui diffèrent, et **où**. Un écart massif dit qu'une passe dessine en
//! coordonnées absolues ; un écart sur une bordure dit que c'est l'anticrénelage aux limites.
//! Les deux se corrigent, mais pas de la même façon.

use glucose_core::store::Store;
use glucose_core::types::{Annotation, BoardImage};
use glucose_desktop::params::SceneOverlay;
use glucose_desktop::renderer::Renderer;
use glucose_desktop::ui::UiState;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (1280, 800);

/// Les bordures qu'on essaie d'ignorer, pour trouver à quelle distance l'écart disparaît.
///
/// Un écart qui s'éteint à quatre pixels est de l'anticrénelage ; un écart qui tient jusqu'à
/// quatre-vingt-seize est la portée d'un flou. Les deux appellent des réponses différentes, et
/// seule la mesure les distingue.
const MARGES: [u32; 6] = [0, 2, 4, 16, 48, 96];

/// Une scène qui a de tout : des images, des cartes, des membranes.
fn document() -> Store {
    let mut store = Store::new("Zone");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport.scale = 1.0;
        b.viewport.x = 100.0;
        b.viewport.y = 80.0;
    }
    for i in 0..12 {
        let (x, y) = ((i % 4) as f64 * 280.0, (i / 4) as f64 * 240.0);
        store.add_image(&board, BoardImage::new(format!("i{i}"), x, y, 220.0, 170.0));
        store.add_annotation(
            &board,
            Annotation::Text {
                id: format!("t{i}"),
                x: x + 30.0,
                y: y + 30.0,
                width: Some(180.0),
                height: Some(60.0),
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
    store
}

/// Rend la scène dans une image de `taille`, la vue décalée de `origine`.
fn rendre(store: &Store, taille: (u32, u32), origine: (f64, f64)) -> Pixmap {
    let mut decale = store.clone();
    if let Some(b) = decale.active_board_mut() {
        b.viewport.x -= origine.0;
        b.viewport.y -= origine.1;
    }
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    // Le mot d'accueil porte une horloge : il rendrait la comparaison non reproductible.
    ui.current_toast = None;
    let mut pixmap = Pixmap::new(taille.0, taille.1).expect("pixmap");
    let guides = glucose_core::smart_align::SnapGuides::default();
    renderer.sync_spatial_index(&decale);
    // La SCENE seule : la chrome se place sur la taille de la fenêtre et non sur la vue,
    // donc elle n'a rien à faire dans une comparaison de région. C'est précisément ce que la
    // première version de ce banc a montré — et c'est pourquoi les deux sont séparées.
    //
    // Le bandeau se trouve `origine.1` pixels plus haut dans la région, et peut être
    // entièrement au-dessus d'elle.
    renderer.rendre_la_scene(
        &mut pixmap.as_mut(),
        &decale,
        &ui,
        SceneOverlay::sans_rien(&guides),
        ui.header_height() - origine.1 as f32,
        glucose_desktop::renderer::Regard::immobile(),
    );
    pixmap
}

/// Compare la région de `entier` à `part`, en ignorant une bordure de `marge` pixels.
///
/// La marge existe parce que les premières mesures ont montré que les écarts restants se
/// tenaient **aux bords** : un trait coupé par la limite du pixmap ne s'anticrénèle pas comme
/// le même trait coupé au milieu d'une image plus grande. Comparer avec et sans marge dit donc
/// si le résidu est un effet de bord — auquel cas A.1 tient, à condition de redessiner un peu
/// plus large que la zone qu'on reporte — ou un vrai écart de fond.
fn comparer(
    entier: &Pixmap,
    part: &Pixmap,
    origine: (u32, u32),
    marge: u32,
) -> (usize, Option<u32>) {
    let ew = entier.width() as usize;
    let (ea, _) = entier.data().as_chunks::<4>();
    let (pa, _) = part.data().as_chunks::<4>();
    let mut differents = 0usize;
    let mut premiere = None;
    if part.width() <= 2 * marge || part.height() <= 2 * marge {
        return (0, None);
    }
    for y in marge..part.height() - marge {
        for x in marge..part.width() - marge {
            let ie = (y + origine.1) as usize * ew + (x + origine.0) as usize;
            let ip = y as usize * part.width() as usize + x as usize;
            if ea[ie] != pa[ip] {
                differents += 1;
                if premiere.is_none() {
                    premiere = Some(y);
                }
            }
        }
    }
    (differents, premiere)
}

/// Ce que coute une image, region par region -- la seconde moitie de la question.
///
/// L'exactitude ne sert a rien si le detour coute plus cher que ce qu'il evite. Un renderer
/// chauffe, puis la mediane de quinze images : la premiere porte l'ouverture des polices et
/// l'index, qui n'appartiennent a aucune region.
fn chronometrer(store: &Store, taille: (u32, u32), origine: (f64, f64)) -> f64 {
    let mut decale = store.clone();
    if let Some(b) = decale.active_board_mut() {
        b.viewport.x -= origine.0;
        b.viewport.y -= origine.1;
    }
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    ui.current_toast = None;
    let mut pixmap = Pixmap::new(taille.0, taille.1).expect("pixmap");
    let guides = glucose_core::smart_align::SnapGuides::default();
    let mut mesures = Vec::new();
    for i in 0..17 {
        let t = std::time::Instant::now();
        renderer.rendre_la_scene(
            &mut pixmap.as_mut(),
            &decale,
            &ui,
            SceneOverlay::sans_rien(&guides),
            ui.header_height() - origine.1 as f32,
            glucose_desktop::renderer::Regard::immobile(),
        );
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if i >= 2 {
            mesures.push(ms);
        }
    }
    mesures.sort_by(f64::total_cmp);
    mesures[mesures.len() / 2]
}

fn main() {
    let store = document();
    let entier = rendre(&store, ECRAN, (0.0, 0.0));

    println!(
        "Rendre une région vaut-il rendre tout ? — {} × {}\n",
        ECRAN.0, ECRAN.1
    );
    let mut entete = format!("  {:<22}", "région");
    for marge in MARGES {
        entete.push_str(&format!(" {:>11}", format!("bord {marge}")));
    }
    println!("{entete}");

    for (x0, y0, w, h) in [
        (0u32, 0u32, 1280u32, 800u32),
        (0, 0, 640, 400),
        (200, 150, 400, 300),
        (640, 400, 640, 400),
    ] {
        let part = rendre(&store, (w, h), (f64::from(x0), f64::from(y0)));
        let total = (w * h) as usize;
        // À quelle distance du bord l'écart disparaît-il ? Cette distance n'est pas une
        // curiosité : c'est exactement de combien il faudra redessiner plus large que la zone
        // qu'on reporte, et elle doit donc se **mesurer** plutôt que se choisir.
        let mut ligne = format!("  {:<22}", format!("({x0}, {y0}) {w}×{h}"));
        for marge in MARGES {
            let (differents, _) = comparer(&entier, &part, (x0, y0), marge);
            ligne.push_str(&format!(
                " {:>11}",
                if differents == 0 {
                    "0".to_string()
                } else {
                    format!("{:.3}%", 100.0 * differents as f64 / total as f64)
                }
            ));
        }
        println!("{ligne}");
    }

    println!(
        "\n  Chaque colonne ignore une bordure de N pixels. La colonne où l'écart tombe à zéro\n  \
         donne de combien il faudra redessiner plus large que la zone reportée — une valeur\n  \
         mesurée, et non choisie."
    );

    // La seconde moitié de la question : le détour coûte-t-il moins que ce qu'il évite ?
    let complet = chronometrer(&store, ECRAN, (0.0, 0.0));
    println!("\n  Ce qu'une image coûte, selon ce qu'on en redessine\n");
    println!(
        "  {:<26} {:>12} {:>12} {:>14}",
        "région", "part", "durée", "contre tout"
    );
    println!(
        "  {:<26} {:>11.1}% {:>10.2}ms {:>14}",
        "toute la fenêtre", 100.0, complet, "—"
    );
    for (x0, y0, w, h) in [(200u32, 150u32, 400u32, 300u32), (500, 300, 200, 150)] {
        let ms = chronometrer(&store, (w, h), (f64::from(x0), f64::from(y0)));
        let part = 100.0 * f64::from(w * h) / f64::from(ECRAN.0 * ECRAN.1);
        println!(
            "  {:<26} {:>11.1}% {:>10.2}ms {:>13.1}x",
            format!("({x0}, {y0}) {w}×{h}"),
            part,
            ms,
            complet / ms.max(0.001)
        );
    }
}
