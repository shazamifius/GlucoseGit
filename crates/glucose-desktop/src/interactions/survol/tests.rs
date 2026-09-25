//! **Au survol d'une flèche ancrée au second « bonjours », seul le second brille** — le défaut de
//! Tauri qu'il a relevé, éprouvé par la vraie souris et la vraie image.

use crate::app::GlucoseApp;
use glucose_core::text_anchors::create_anchor;
use glucose_core::types::{Annotation, TextSelection};
use winit::dpi::PhysicalPosition;

const SA_CARTE: &str = "bonjours\ntest\ntest\nbonjours";
const TAILLE: (u32, u32) = (1200, 800);

/// Sa carte en `(100, 100)`, une flèche ancrée au second « bonjours » vers un point à droite,
/// à la vue 1:1.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("carte", 100.0, 100.0, SA_CARTE));
    let second = SA_CARTE.rfind("bonjours").expect("le second");
    let mut fleche = Annotation::arrow("f", 100.0, 100.0, 900.0, 600.0);
    if let Annotation::Arrow {
        source_id,
        source_text_sel,
        ..
    } = &mut fleche
    {
        *source_id = Some("carte".to_string());
        *source_text_sel = Some(TextSelection::Anchors(vec![create_anchor(
            SA_CARTE,
            second,
            second + 8,
        )
        .expect("une ancre")]));
    }
    app.store.add_annotation(&board, fleche);
    app.store.clear_selection();
    app.store.set_viewport(
        &board,
        glucose_core::types::Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        },
    );
    app.une_image_sans_fenetre(TAILLE);
    app
}

/// Un point du trait de la flèche, à mi-chemin, en pixels d'écran.
fn sur_la_fleche(app: &GlucoseApp) -> (f64, f64) {
    let board = app.store.active_board().expect("un tableau");
    let ann = board
        .annotations
        .iter()
        .find(|a| a.id() == "f")
        .expect("la flèche");
    let noeuds = crate::renderer::arrow::NoeudsDuRendu {
        board,
        index: Some(&app.renderer.spatial_hash),
        typographie: &app.renderer.typography,
        math: &app.renderer.math,
    };
    let chemin = glucose_core::arrow::path_with(ann, noeuds).expect("un chemin");
    let (a, b) = (chemin[0], chemin[chemin.len() - 1]);
    ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
}

/// **La souris sur la flèche : seul le second « bonjours » est à éclairer** ; hors d'elle,
/// rien.
#[test]
fn test_fleche_4_seul_le_second_bonjours_brille() {
    let mut app = application();
    let p = sur_la_fleche(&app);
    app.handle_cursor_moved(PhysicalPosition::new(p.0, p.1));
    assert_eq!(app.ui.fleche_survolee.as_deref(), Some("f"));
    let second = SA_CARTE.rfind("bonjours").expect("le second");
    let eclairages = app.eclairages();
    assert_eq!(eclairages.len(), 1);
    assert_eq!(eclairages[0].carte, "carte");
    assert_eq!(eclairages[0].plages, vec![(second, second + 8)]);

    app.handle_cursor_moved(PhysicalPosition::new(
        1150.0,
        50.0 + f64::from(app.ui.header_height()),
    ));
    assert!(app.ui.fleche_survolee.is_none(), "hors de la flèche, rien");
    assert!(app.eclairages().is_empty());
}

/// **Et à l'image, c'est bien lui qui brille** : sous la souris, le fond du second « bonjours »
/// prend la couleur de la carte ; celui du premier ne bouge pas.
#[test]
fn test_fleche_4_a_l_image_le_second_brille_et_le_premier_non() {
    let mut app = application();
    // Le fond de chaque mot, juste à gauche de sa première lettre, avant le survol.
    let ligne = crate::renderer::card::text_box(240.0).line_height;
    let (ox, oy) = crate::renderer::card::TEXT_ORIGIN;
    let fond = |app: &GlucoseApp, rang: f32| {
        let image = app.pixmap.as_ref().expect("une image");
        let (x, y) = (100.0 + ox - 1.5, 100.0 + oy + (rang + 0.5) * ligne);
        let p = image.pixel(x as u32, y as u32).expect("dedans");
        (p.red(), p.green(), p.blue())
    };
    let (premier, second) = (fond(&app, 0.0), fond(&app, 3.0));
    let p = sur_la_fleche(&app);
    app.handle_cursor_moved(PhysicalPosition::new(p.0, p.1));
    app.une_image_sans_fenetre(TAILLE);
    assert_eq!(fond(&app, 0.0), premier, "le premier ne brille pas");
    assert_ne!(fond(&app, 3.0), second, "le second brille");
}
