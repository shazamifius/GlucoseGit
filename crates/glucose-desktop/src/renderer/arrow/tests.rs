//! **Le dessin d'une flèche tient la loi de Tauri** (FLECHE-1) : le dégradé va de la teinte de
//! départ à celle d'arrivée, le trait garde son épaisseur à l'écran, la pastille est au bout,
//! et un coude n'est pas couvert deux fois.

use super::super::pass::Clip;
use super::super::scale::WorldScale;
use super::*;
use crate::renderer::math::MathRenderer;
use crate::renderer::DomainTints;
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::arrow::trace::morceaux;
use glucose_core::types::Viewport;
use tiny_skia::Pixmap;

const TAILLE: (u32, u32) = (400, 300);

/// Deux teintes franches : un rouge au départ, un bleu à l'arrivée.
fn teintes() -> Teintes {
    aspect::teintes(0.0, 240.0)
}

/// Rend une flèche passant par ces points du monde, sur un fond noir, au zoom donné (la vue
/// est centrée sur l'origine du monde).
fn rendu(points: &[(f64, f64)], zoom: f64, selectionnee: bool, double_sens: bool) -> Pixmap {
    let mut pixmap = Pixmap::new(TAILLE.0, TAILLE.1).expect("pixmap");
    pixmap.fill(tiny_skia::Color::BLACK);
    let (typography, math, tints, theme) = (
        Typography::new(),
        MathRenderer::new(),
        DomainTints::default(),
        Theme::dark(),
    );
    let vp = Viewport {
        x: f64::from(TAILLE.0) / 2.0,
        y: f64::from(TAILLE.1) / 2.0,
        scale: zoom,
    };
    let ctx = Pass {
        typography: &typography,
        math: &math,
        tints: &tints,
        theme: &theme,
        vp,
        scale: WorldScale::new(zoom, 1.0),
        clip: Clip {
            width: TAILLE.0 as f32,
            height: TAILLE.1 as f32,
            top: 0.0,
        },
    };
    let trace = morceaux(points, false);
    draw_arrow(
        &ctx,
        &mut pixmap.as_mut(),
        &Fleche {
            morceaux: trace,
            teintes: teintes(),
            epaisseur: aspect::EPAISSEUR,
            selectionnee,
            double_sens,
        },
    );
    pixmap
}

fn rgb(p: &Pixmap, x: u32, y: u32) -> (u8, u8, u8) {
    let c = p.pixel(x, y).expect("dedans");
    (c.red(), c.green(), c.blue())
}

/// **Le dégradé va du départ à l'arrivée** : près du départ, le trait tire vers le rouge ;
/// près de l'arrivée, vers le bleu.
#[test]
fn test_fleche_1_le_degrade_va_de_la_source_a_la_cible() {
    let p = rendu(&[(-150.0, 0.0), (150.0, 0.0)], 1.0, false, false);
    let (r0, _, b0) = rgb(&p, 60, 150);
    let (r1, _, b1) = rgb(&p, 280, 150);
    assert!(r0 > b0, "près du départ, rouge : ({r0}, {b0})");
    assert!(b1 > r1, "près de l'arrivée, bleu : ({r1}, {b1})");
}

/// L'épaisseur du trait, en pixels : la colonne `x` compte ce qui est à plus de 60 % de la
/// luminosité du trait.
fn epaisseur(p: &Pixmap, x: u32) -> usize {
    let lum = |y: u32| {
        let (r, g, b) = rgb(p, x, y);
        u32::from(r) + u32::from(g) + u32::from(b)
    };
    let max = (0..TAILLE.1).map(lum).max().unwrap_or(0);
    (0..TAILLE.1).filter(|&y| lum(y) * 10 > max * 6).count()
}

/// **Le trait garde son épaisseur à l'écran** quand on zoome — le `non-scaling-stroke` de
/// Tauri : de loin, le graphe reste lisible ; de près, la flèche reste un fil.
#[test]
fn test_fleche_1_le_trait_garde_son_epaisseur_a_l_ecran() {
    let loin = rendu(&[(-600.0, 0.0), (600.0, 0.0)], 0.25, false, false);
    let pres = rendu(&[(-40.0, 0.0), (40.0, 0.0)], 4.0, false, false);
    let (e_loin, e_pres) = (epaisseur(&loin, 200), epaisseur(&pres, 200));
    assert!(
        e_loin > 0 && e_loin == e_pres,
        "{e_loin} px de loin, {e_pres} de près"
    );
}

/// **La pastille est au bout** — sombre au centre, cerclée — et au départ seulement si la
/// flèche va dans les deux sens.
#[test]
fn test_fleche_1_la_pastille_marque_l_arrivee() {
    // Au zoom 4, la pastille fait 4 × 1,5 × 2 = 12 pixels de rayon.
    let aller = rendu(&[(-30.0, 0.0), (30.0, 0.0)], 4.0, false, false);
    let (cx_fin, cx_debut, cy) = (200 + 120, 200 - 120, 150);
    assert_eq!(
        rgb(&aller, cx_fin, cy),
        (0x11, 0x11, 0x11),
        "le fond de la pastille"
    );
    assert_ne!(
        rgb(&aller, cx_debut, cy),
        (0x11, 0x11, 0x11),
        "pas de pastille au départ"
    );
    let retour = rendu(&[(-30.0, 0.0), (30.0, 0.0)], 4.0, false, true);
    assert_eq!(
        rgb(&retour, cx_debut, cy),
        (0x11, 0x11, 0x11),
        "double sens : au départ aussi"
    );
}

/// **Sélectionnée, la flèche est blanche**, et ses bouts sont des disques pleins de sa teinte.
#[test]
fn test_fleche_1_selectionnee_elle_est_blanche() {
    let p = rendu(&[(-150.0, 0.0), (150.0, 0.0)], 1.0, true, false);
    let (r, g, b) = rgb(&p, 200, 150);
    assert!(r > 220 && g > 220 && b > 220, "blanche : ({r}, {g}, {b})");
    let fin = rgb(&p, 350, 150);
    let (tr, tg, tb) = teintes().arrivee;
    assert!(
        fin.0.abs_diff(tr) < 8 && fin.1.abs_diff(tg) < 8 && fin.2.abs_diff(tb) < 8,
        "le bout, plein de la teinte d'arrivée : {fin:?}"
    );
}

/// **Un coude n'est pas couvert deux fois** : le halo y est aussi doux que le long du
/// trait. Deux tronçons dessinés l'un après l'autre y auraient superposé leurs halos à 18 %,
/// soit 33 %.
#[test]
fn test_fleche_1_un_coude_n_est_pas_couvert_deux_fois() {
    let coude = rendu(
        &[(-150.0, 60.0), (0.0, -60.0), (150.0, 60.0)],
        1.0,
        false,
        false,
    );
    let droit = rendu(&[(-150.0, 0.0), (150.0, 0.0)], 1.0, false, false);
    // Le halo seul : les pixels à 2,5 px du trait (le trait fait 1 px de demi-épaisseur,
    // le halo 3), **loin des bouts** — le contour d'une pastille y brille, et il masquerait
    // tout. Le plus lumineux du coude ne doit pas dépasser le plus lumineux du droit.
    let halo = |p: &Pixmap, points: &[(f64, f64)]| -> u32 {
        let mut max = 0;
        for y in 0..TAILLE.1 {
            for x in 0..TAILLE.0 {
                let w = (f64::from(x) - 200.0 + 0.5, f64::from(y) - 150.0 + 0.5);
                let d = points
                    .windows(2)
                    .map(|s| glucose_core::geometry::distance_to_segment(w, s[0], s[1]))
                    .fold(f64::INFINITY, f64::min);
                if (2.2..=2.8).contains(&d) && w.0.abs() < 100.0 {
                    let (r, g, b) = rgb(p, x, y);
                    max = max.max(u32::from(r).max(u32::from(g)).max(u32::from(b)));
                }
            }
        }
        max
    };
    let h_coude = halo(&coude, &[(-150.0, 60.0), (0.0, -60.0), (150.0, 60.0)]);
    let h_droit = halo(&droit, &[(-150.0, 0.0), (150.0, 0.0)]);
    assert!(
        h_coude <= h_droit + 3,
        "le coude brille plus : {h_coude} contre {h_droit}"
    );
}

/// **DPI-1 — à 150 %, le trait d'une flèche est une fois et demie plus épais à l'écran** : ses
/// deux pixels sont des pixels logiques, comme chez Tauri. L'épreuve passe par tout le chemin —
/// l'échelle de l'interface, le moteur, la passe, le champ —, pas par une échelle posée à la
/// main.
#[test]
fn test_dpi_1_le_trait_suit_la_densite_de_l_ecran() {
    use glucose_core::store::Store;
    let epaisseur = |densite: f32| {
        let mut store = Store::new("dpi");
        let board = store.project.active_board_id.clone();
        store.add_annotation(&board, Annotation::arrow("f", -150.0, 0.0, 150.0, 0.0));
        store.clear_selection();
        store.set_viewport(
            &board,
            Viewport {
                x: 400.0,
                y: 400.0,
                scale: 1.0,
            },
        );
        let mut renderer = crate::renderer::Renderer::new();
        let mut ui = crate::ui::UiState::new();
        ui.scale_factor = densite;
        let image = crate::bench::render_frame(&mut renderer, &mut ui, &store, 800, 800);
        // L'encre de la colonne du milieu, halo et âme ensemble, fond retranché : sa somme
        // est proportionnelle à la largeur du trait, quelle que soit la façon dont ses bords
        // tombent sur la grille de pixels.
        let teinte = |x: u32, y: u32| {
            let c = image.pixel(x, y).expect("dans l'image");
            f64::from(c.red()) + f64::from(c.green()) + f64::from(c.blue())
        };
        // Le fond du canevas, lu loin du trait, entre deux points de la grille.
        let fond = teinte(401, 250);
        (360..440).map(|y| teinte(401, y) - fond).sum::<f64>()
    };
    let rapport = epaisseur(1.5) / epaisseur(1.0);
    assert!(
        (rapport - 1.5).abs() < 0.03,
        "à 150 %, le trait porte une fois et demie l'encre : {rapport}"
    );
}

/// **DPI-1 — les poignées d'une flèche se dessinent en pixels logiques** : à 150 %, le losange
/// du milieu couvre deux fois et quart la surface qu'il couvre à 100 %.
#[test]
fn test_dpi_1_les_poignees_d_une_fleche_suivent_la_densite() {
    let surface = |densite: f32| {
        let mut pixmap = Pixmap::new(100, 100).expect("pixmap");
        let poignee = glucose_core::arrow::ArrowHandle {
            at: (0.0, 0.0),
            kind: glucose_core::arrow::HandleKind::Midpoint(0),
        };
        let vp = Viewport {
            x: 50.0,
            y: 50.0,
            scale: 1.0,
        };
        crate::renderer::handles::draw_arrow_handles(
            &mut pixmap.as_mut(),
            &Theme::dark(),
            &[poignee],
            (&vp, WorldScale::new(1.0, densite)),
        );
        pixmap
            .data()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| f64::from(p[3]) / 255.0)
            .sum::<f64>()
    };
    let rapport = surface(1.5) / surface(1.0);
    assert!(
        (rapport - 2.25).abs() < 0.15,
        "rapport des surfaces : {rapport}"
    );
}

/// **DPI-1, sur la voie graphique** — celle de sa machine : les flèches que la carte reçoit
/// portent déjà leur trait en pixels logiques. À 150 %, l'âme d'un trait de deux pixels a une
/// demi-largeur d'un pixel et demi, et son halo de six, quatre et demi.
#[test]
fn test_dpi_1_la_carte_recoit_des_fleches_a_la_densite_de_l_ecran() {
    use glucose_core::quadtree::{tous_les_rangs, SpatialHash};
    use glucose_core::store::Store;
    let mut store = Store::new("dpi");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::arrow("f", -100.0, 0.0, 100.0, 0.0));
    store.clear_selection();
    let (typographie, math) = (Typography::new(), MathRenderer::new());
    let mut index = SpatialHash::new(1000.0);
    index.index_board(store.active_board().expect("un tableau"));
    let rangs = tous_les_rangs(store.active_board().expect("un tableau"));
    let pass = crate::params::ViewPass {
        vp: Viewport {
            x: 200.0,
            y: 150.0,
            scale: 1.0,
        },
        visibles: &rangs,
        index: &index,
        header_h: 0.0,
        densite: 1.5,
    };
    let champs = fleches_a_poser(
        &mut SymbioticHueCache::new(),
        (&store, pass),
        (&typographie, &math),
        (400.0, 300.0),
    );
    let champ = champs.first().expect("une flèche");
    assert_eq!(champ.ame[0], 1.5, "demi-âme");
    assert_eq!(champ.halo[0], 4.5, "demi-halo");
}
