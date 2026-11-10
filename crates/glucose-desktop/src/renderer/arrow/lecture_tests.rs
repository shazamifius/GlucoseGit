//! **LIEN-1 — de loin, un lien rétrécit avec le monde ; ceux qu'on regarde restent pleins.**
//!
//! Sa parole du 9/11 : *« de base, les liens sont très transparents ; dès qu'on s'approche d'un
//! bloc de texte, ils apparaissent »* — la seule exception de niveau de détail qu'il ait donnée.
//! Éprouvé dans la loi (le champ, en nombres exacts), dans l'encre que le processeur peint, par
//! la carte graphique (ce qu'elle reçoit), dans l'export SVG, et par la vraie souris.

use super::*;
use crate::app::GlucoseApp;
use crate::renderer::scale::WorldScale;
use crate::salissure::Salissure;
use glucose_core::arrow::trace::morceaux;
use glucose_core::types::Viewport;
use winit::dpi::PhysicalPosition;

/// Le zoom de la taille réelle, à la densité 1 : une unité du monde par pixel.
const LECTURE: f64 = 1.0;
const TAILLE: (u32, u32) = (1200, 800);

/// Une flèche de l'origine à `(1 000, 0)`, telle qu'elle se montre.
fn fleche(en_lumiere: bool) -> Fleche {
    Fleche {
        morceaux: morceaux(&[(0.0, 0.0), (1000.0, 0.0)], false),
        teintes: aspect::teintes(0.0, 240.0),
        epaisseur: aspect::EPAISSEUR,
        selectionnee: false,
        en_lumiere,
        double_sens: false,
    }
}

/// Son champ à l'écran, au zoom `z` (la vue sur l'origine, à gauche de l'écran).
fn champ(f: &Fleche, z: f64) -> Champ {
    let vp = Viewport {
        x: 10.0,
        y: 400.0,
        scale: z,
    };
    f.champ(&vp, WorldScale::new(z, 1.0), (1200.0, 800.0))
        .expect("un champ")
}

/// **À la distance de lecture et plus près, rien ne change** — le `non-scaling-stroke` de Tauri
/// (FLECHE-1) ; **plus loin, chaque longueur d'écran suit le zoom** : le trait, son halo, le
/// contour de ses disques. La preuve à l'envers : en lumière, la loi de Tauri vaut partout.
#[test]
fn test_lien_1_de_loin_un_lien_retrecit_avec_le_monde() {
    let mesures = |c: &Champ| {
        let disque = c.disques[1].expect("la pastille");
        [c.ame[0], c.halo[0], disque.demi_contour]
    };
    let tauri = [1.0, 3.0, 1.0];
    for z in [LECTURE, 2.0 * LECTURE, 8.0] {
        assert_eq!(mesures(&champ(&fleche(false), z)), tauri, "au zoom {z}");
    }
    for part in [0.5_f32, 0.25, 0.1] {
        let loin = mesures(&champ(&fleche(false), LECTURE * f64::from(part)));
        for (m, t) in loin.iter().zip(tauri) {
            assert!((m - t * part).abs() < 1e-6, "à {part} : {loin:?}");
        }
        let regardee = mesures(&champ(&fleche(true), LECTURE * f64::from(part)));
        assert_eq!(regardee, tauri, "en lumière, pleine à {part}");
    }
}

/// **La taille réelle se compte en pixels logiques** (DPI-1) : sur un écran à 200 %, le monde est à
/// sa taille quand une unité y fait deux pixels physiques.
#[test]
fn test_lien_1_la_taille_reelle_est_en_pixels_logiques() {
    assert_eq!(WorldScale::new(2.0, 2.0).lecture(), 1.0);
    assert_eq!(WorldScale::new(1.0, 2.0).lecture(), 0.5);
    assert_eq!(WorldScale::new(4.0, 2.0).lecture(), 1.0);
}

/// **L'encre peinte suit la loi** : plus fin qu'un pixel, le trait se peint à sa couverture
/// exacte. Au-delà de la lecture, un lien est une chose du monde : quand le zoom est divisé par
/// deux, son encre l'est par quatre (sa longueur et sa largeur) ; en lumière, par deux, comme chez
/// Tauri (sa longueur seule).
#[test]
fn test_lien_1_l_encre_peinte_suit_le_monde() {
    let encre = |f: &Fleche, z: f64| {
        let (l, h) = (1200_u32, 800_u32);
        let mut pixels = vec![[0_u8; 4]; (l * h) as usize];
        assert!(champ(f, z).peindre(&mut pixels, l, h), "de l'encre");
        pixels.iter().map(|p| f64::from(p[3])).sum::<f64>()
    };
    let (moitie, quart) = (LECTURE / 2.0, LECTURE / 4.0);
    let du_monde = encre(&fleche(false), moitie) / encre(&fleche(false), quart);
    assert!((du_monde - 4.0).abs() < 0.2, "rapport {du_monde}");
    let de_tauri = encre(&fleche(true), moitie) / encre(&fleche(true), quart);
    assert!((de_tauri - 2.0).abs() < 0.2, "rapport {de_tauri}");
}

/// Deux textes de 800 × 200 reliés, à 1 600 unités l'un de l'autre, un troisième seul, la vue au
/// quart de la taille réelle : chaque texte y fait 200 × 50 pixels.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    for (id, x, y) in [("a", 0.0, 0.0), ("b", 2400.0, 0.0), ("seul", 0.0, 1200.0)] {
        let mut texte = Annotation::text(id, x, y, "un bloc de texte");
        if let Annotation::Text { width, height, .. } = &mut texte {
            (*width, *height) = (Some(800.0), Some(200.0));
        }
        app.store.add_annotation(&board, texte);
    }
    let mut lien = Annotation::arrow("f", 800.0, 100.0, 2400.0, 100.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut lien
    {
        *source_id = Some("a".to_string());
        *target_id = Some("b".to_string());
    }
    app.store.add_annotation(&board, lien);
    app.store.clear_selection();
    app.store.set_viewport(
        &board,
        Viewport {
            x: 100.0,
            y: 100.0,
            scale: LECTURE / 4.0,
        },
    );
    app.une_image_sans_fenetre(TAILLE);
    app
}

/// Ce que la carte graphique reçoit de l'unique flèche : la demi-largeur de son trait.
fn ame_pour_la_carte(app: &GlucoseApp, survole: Option<&str>) -> f32 {
    let board = app.store.active_board().expect("un tableau");
    let rangs = glucose_core::quadtree::tous_les_rangs(board);
    let pass = crate::params::ViewPass {
        vp: board.viewport,
        visibles: &rangs,
        index: &app.renderer.spatial_hash,
        contournement: None,
        header_h: 0.0,
        densite: 1.0,
    };
    let champs = fleches_a_poser(
        &mut SymbioticHueCache::new(),
        (&app.store, pass, survole),
        (&app.renderer.typography, &app.renderer.math),
        (1200.0, 800.0),
    );
    champs.first().expect("la flèche").ame[0]
}

/// **Ceux qu'on regarde restent pleins, sur la voie de la carte** : un bout choisi, un bout
/// survolé, la flèche choisie ; un nœud sans lien qu'on choisit n'y change rien.
#[test]
fn test_lien_1_les_liens_qu_on_regarde_restent_pleins() {
    let mut app = application();
    assert_eq!(ame_pour_la_carte(&app, None), 0.25, "de loin");
    assert_eq!(ame_pour_la_carte(&app, Some("seul")), 0.25, "un autre nœud");
    assert_eq!(ame_pour_la_carte(&app, Some("b")), 1.0, "sa cible survolée");
    assert_eq!(
        ame_pour_la_carte(&app, Some("f")),
        1.0,
        "elle-même survolée"
    );
    for choisi in ["a", "b", "f"] {
        app.store.clear_selection();
        app.store.select_annotation(choisi.to_string(), false);
        assert_eq!(ame_pour_la_carte(&app, None), 1.0, "{choisi} choisi");
    }
    app.store.clear_selection();
    app.store.select_annotation("seul".to_string(), false);
    assert_eq!(
        ame_pour_la_carte(&app, None),
        0.25,
        "un nœud sans lien choisi"
    );
}

/// L'encre d'une bande verticale au milieu du lien, loin des nœuds, dans la dernière image.
fn encre_au_milieu(app: &GlucoseApp) -> f64 {
    let pixmap = app.pixmap.as_ref().expect("une image");
    let (x, y) = crate::canvas::world_to_screen(1600.0, 100.0, &app.store.viewport());
    (x as u32 - 2..x as u32 + 2)
        .flat_map(|x| (y as u32 - 12..y as u32 + 12).map(move |y| (x, y)))
        .map(|(x, y)| {
            let p = pixmap.pixel(x, y).expect("dedans");
            f64::from(p.red()) + f64::from(p.green()) + f64::from(p.blue())
        })
        .sum()
}

/// **Par la vraie souris et la vraie image** : survoler un bout du lien le montre plein dans
/// l'image du processeur, et demande une image ; survoler un nœud sans lien ne redessine rien.
/// À la distance de lecture, où tout lien est déjà plein, le survol ne redessine rien non plus.
#[test]
fn test_lien_1_le_survol_d_un_bout_allume_son_lien() {
    let mut app = application();
    let fond = {
        let mut sans = application();
        let board = sans.store.project.active_board_id.clone();
        sans.store.remove_annotations(&board, &["f"]);
        sans.une_image_sans_fenetre(TAILLE);
        encre_au_milieu(&sans)
    };
    let vers = |app: &mut GlucoseApp, monde: (f64, f64)| {
        let ecran = crate::canvas::world_to_screen(monde.0, monde.1, &app.store.viewport());
        app.prendre_la_salissure();
        app.handle_cursor_moved(PhysicalPosition::new(ecran.0, ecran.1));
        let sale = app.prendre_la_salissure() != Salissure::Rien;
        app.une_image_sans_fenetre(TAILLE);
        (sale, encre_au_milieu(app) - fond)
    };
    let (sale, de_loin) = vers(&mut app, (100.0, 1300.0));
    assert_eq!(app.ui.noeud_survole.as_deref(), Some("seul"));
    assert!(!sale, "un nœud sans lien ne redessine rien");
    assert!(de_loin > 0.0, "de loin, le lien reste là");
    let (sale, allume) = vers(&mut app, (100.0, 100.0));
    assert_eq!(app.ui.noeud_survole.as_deref(), Some("a"));
    assert!(sale, "son bout survolé demande une image");
    assert!(allume > 3.0 * de_loin, "plein : {allume} contre {de_loin}");
    let board = app.store.project.active_board_id.clone();
    app.store.set_viewport(
        &board,
        Viewport {
            scale: LECTURE,
            ..app.store.viewport()
        },
    );
    // La souris quitte un bout du lien : de loin, cela redessinerait ; ici, rien ne change.
    let (sale, _) = vers(&mut app, (100.0, 1300.0));
    assert_eq!(app.ui.noeud_survole.as_deref(), Some("seul"));
    assert!(
        !sale,
        "à la distance de lecture, le survol ne change aucun lien"
    );
}

/// **Par l'entrée de la voie graphique** (`Renderer::rendre_les_couches`), celle de sa machine :
/// la flèche qu'elle confie à la carte est pleine quand on survole un bout, réduite sinon. Une
/// épreuve qui appellerait `fleches_a_poser` directement ne verrait pas le survol oublié en chemin.
#[test]
fn test_lien_1_la_voie_graphique_recoit_le_survol() {
    let mut app = application();
    let confiee = |app: &mut GlucoseApp, survole: Option<&str>| {
        let overlay = crate::params::SceneOverlay {
            survole,
            ..crate::params::SceneOverlay::sans_rien(&app.active_guides)
        };
        let pointeur = crate::params::Pointer { x: 0.0, y: 0.0 };
        let (mut dessous, mut dessus) = (
            tiny_skia::Pixmap::new(TAILLE.0, TAILLE.1).expect("pixmap"),
            tiny_skia::Pixmap::new(TAILLE.0, TAILLE.1).expect("pixmap"),
        );
        let confie = app.renderer.rendre_les_couches(
            &mut dessous.as_mut(),
            &mut dessus.as_mut(),
            &app.store,
            (&mut app.ui, pointeur),
            overlay,
            crate::renderer::Regard::immobile(),
        );
        confie.fleches.first().expect("la flèche").ame[0]
    };
    assert_eq!(confiee(&mut app, None), 0.25);
    assert_eq!(confiee(&mut app, Some("a")), 1.0);
}

/// **L'export SVG montre ce que le canevas montre** : de loin, le trait et son halo ont la
/// largeur de la loi.
#[test]
fn test_lien_1_l_export_svg_suit_la_loi() {
    let mut app = application();
    let vue = app.store.viewport();
    let svg = app
        .renderer
        .rendre_en_svg((&app.store, &mut app.ui), vue, TAILLE, &mut |_| None);
    assert!(
        svg.contains(r#"stroke-width="0.5" stroke-opacity="0.92""#),
        "{svg}"
    );
    assert!(
        svg.contains(r#"stroke-width="1.5" stroke-opacity="0.18""#),
        "{svg}"
    );
    assert!(
        svg.contains(r#"stroke-width="0.5"/>"#),
        "le contour de la pastille : {svg}"
    );
}
