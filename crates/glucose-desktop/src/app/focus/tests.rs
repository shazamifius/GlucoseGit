//! **Le mode Focus, par la vraie application** : la vue qui se remplit d'une membrane y entre,
//! le rendu et le clic ne voient plus qu'elle, et dézoomer en sort.

use crate::app::GlucoseApp;
use glucose_core::types::{Annotation, BoardImage, Viewport};

const ECRAN: (u32, u32) = (1440, 900);

/// Une membrane `M` qui porte `dedans`, une image `dehors` loin d'elle, et une seconde
/// membrane `voisine` à côté.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let b = app.store.project.active_board_id.clone();
    if let Some(board) = app.store.active_board_mut() {
        board.annotations.clear();
        board.images.clear();
    }
    app.store
        .add_annotation(&b, Annotation::membrane("M", 0.0, 0.0, 1000.0, 600.0));
    app.store
        .add_image(&b, BoardImage::new("dedans", 500.0, 300.0, 100.0, 80.0));
    app.store
        .add_image(&b, BoardImage::new("dehors", 5000.0, 300.0, 100.0, 80.0));
    app.store.add_annotation(
        &b,
        Annotation::membrane("voisine", 1100.0, 0.0, 300.0, 300.0),
    );
    app.renderer.sync_spatial_index(&app.store);
    app
}

/// La vue centrée sur `M`, à l'échelle `e` : à 3, `M` remplit tout le canevas.
fn regarder_m(app: &mut GlucoseApp, e: f64) {
    let b = app.store.project.active_board_id.clone();
    let bandeau = f64::from(app.ui.header_height());
    let centre_y = bandeau + (f64::from(ECRAN.1) - bandeau) / 2.0;
    app.store.set_viewport(
        &b,
        Viewport {
            x: f64::from(ECRAN.0) / 2.0 - 500.0 * e,
            y: centre_y - 300.0 * e,
            scale: e,
        },
    );
}

#[test]
fn test_remplir_l_ecran_d_une_membrane_y_entre_et_ne_montre_qu_elle() {
    let mut app = app();
    let fond = app.renderer.theme.bg_canvas;
    regarder_m(&mut app, 0.5);
    app.suivre_le_focus(ECRAN);
    assert_eq!(app.focus.membrane(), None, "de loin, pas de focus");

    regarder_m(&mut app, 3.0);
    app.suivre_le_focus(ECRAN);
    assert_eq!(app.focus.membrane(), Some("M"));
    assert!(app.vol.en_cours(), "la caméra se cale sur la membrane");
    let f = &app.renderer.focus;
    assert!(f.laisse_voir("M") && f.laisse_voir("dedans"));
    assert!(!f.laisse_voir("dehors") && !f.laisse_voir("voisine"));
    assert_ne!(
        app.renderer.theme.bg_canvas, fond,
        "le fond prend sa teinte"
    );

    // Ce qu'on ne voit pas ne s'attrape pas.
    assert!(app.pick_candidates_at(5000.0, 300.0).is_empty());
    assert!(!app.pick_candidates_at(500.0, 300.0).is_empty());
}

#[test]
fn test_dezoomer_sort_du_focus_et_rend_tout() {
    let mut app = app();
    let fond = app.renderer.theme.bg_canvas;
    regarder_m(&mut app, 3.0);
    app.suivre_le_focus(ECRAN);
    assert_eq!(app.focus.membrane(), Some("M"));

    // Le temps mort du noyau est passé : on peut sortir.
    app.focus.etat.t -= 10_000;
    regarder_m(&mut app, 0.5);
    app.suivre_le_focus(ECRAN);
    assert_eq!(app.focus.membrane(), None);
    assert!(app.renderer.focus.laisse_voir("dehors"));
    assert_eq!(app.renderer.theme.bg_canvas, fond, "le fond revient");
    assert!(!app.pick_candidates_at(5000.0, 300.0).is_empty());
}

/// Le rectangle de sélection, en focus, ne ramasse que ce qui se voit.
#[test]
fn test_le_rectangle_ne_ramasse_que_ce_qui_se_voit() {
    let mut app = app();
    regarder_m(&mut app, 3.0);
    app.suivre_le_focus(ECRAN);
    let vp = app.store.viewport();
    let (x1, y1) = crate::canvas::world_to_screen(-100.0, -100.0, &vp);
    let (x2, y2) = crate::canvas::world_to_screen(6000.0, 700.0, &vp);
    app.selection_box = Some((x1, y1, x2, y2));
    app.finish_selection_box();
    assert!(app.store.selected_image_ids.contains(&"dedans".to_string()));
    assert!(!app.store.selected_image_ids.contains(&"dehors".to_string()));
    assert!(!app
        .store
        .selected_annotation_ids
        .contains(&"voisine".to_string()));
}

/// Le couleur d'un pixel de l'image rendue par la voie processeur, là où le monde vaut
/// `(wx, wy)`.
fn pixel_du_rendu(app: &mut GlucoseApp, (wx, wy): (f64, f64)) -> [u8; 4] {
    let mut p = tiny_skia::Pixmap::new(ECRAN.0, ECRAN.1).expect("un tampon");
    let guides = glucose_core::smart_align::SnapGuides::default();
    app.renderer.render(
        &mut p.as_mut(),
        &app.store,
        &mut app.ui,
        crate::params::SceneOverlay::sans_rien(&guides),
        crate::params::Pointer { x: -1.0, y: -1.0 },
        crate::renderer::Regard::immobile(),
    );
    let (sx, sy) = crate::canvas::world_to_screen(wx, wy, &app.store.viewport());
    let i = (sy as u32 * ECRAN.0 + sx as u32) as usize * 4;
    let d = p.data();
    [d[i], d[i + 1], d[i + 2], d[i + 3]]
}

/// **Ce que le focus cache ne se dessine pas** : une image posée juste hors de la membrane,
/// dans le champ, se voit hors focus ; en focus, à sa place, il n'y a que le fond teinté.
#[test]
fn test_ce_que_le_focus_cache_ne_se_dessine_pas() {
    let mut app = app();
    let b = app.store.project.active_board_id.clone();
    app.store
        .add_image(&b, BoardImage::new("au-bord", 1040.0, 300.0, 40.0, 40.0));
    app.renderer.sync_spatial_index(&app.store);
    regarder_m(&mut app, 1.2);
    let fond = app.renderer.theme.bg_canvas.to_color_u8();
    let hors_focus = pixel_du_rendu(&mut app, (1040.0, 300.0));
    assert_ne!(
        &hors_focus[..3],
        &[fond.red(), fond.green(), fond.blue()][..],
        "hors focus, l'image se voit"
    );

    regarder_m(&mut app, 3.0);
    app.suivre_le_focus(ECRAN);
    assert_eq!(app.focus.membrane(), Some("M"));
    regarder_m(&mut app, 1.2);
    let teinte = app.renderer.theme.bg_canvas.to_color_u8();
    let en_focus = pixel_du_rendu(&mut app, (1040.0, 300.0));
    assert_eq!(
        &en_focus[..3],
        &[teinte.red(), teinte.green(), teinte.blue()][..],
        "en focus, le fond teinté"
    );
}

/// La décision se prend **à chaque image où la vue a bougé**, sans qu'aucun geste n'ait à la
/// demander : c'est l'étape du mouvement de l'image qui la porte.
#[test]
fn test_l_image_decide_du_focus_d_elle_meme() {
    let mut app = app();
    regarder_m(&mut app, 3.0);
    app.appliquer_l_elan(ECRAN.0, ECRAN.1);
    assert_eq!(app.focus.membrane(), Some("M"));
}
