//! DEFILE-1 : la loi de la vitesse, puis le glisser joué sur le vrai [`GlucoseApp`], par le
//! chemin de la souris, image après image.

use super::*;
use glucose_core::types::BoardImage;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

const CADRE: Cadre = Cadre {
    gauche: 0.0,
    haut: 100.0,
    droite: 1000.0,
    bas: 700.0,
};

/// **Hors de la bande, rien ne défile** ; au bord, une étendue par seconde ; au milieu de la
/// bande, le quart — la vitesse suit le carré de l'enfoncement.
#[test]
fn test_la_vitesse_suit_le_carre_de_l_enfoncement() {
    assert_eq!(vitesse((500.0, 400.0), CADRE, 50.0), (0.0, 0.0));
    assert_eq!(
        vitesse((0.0, 400.0), CADRE, 50.0),
        (1000.0, 0.0),
        "au bord gauche"
    );
    assert_eq!(
        vitesse((1000.0, 400.0), CADRE, 50.0),
        (-1000.0, 0.0),
        "au bord droit"
    );
    assert_eq!(
        vitesse((25.0, 400.0), CADRE, 50.0),
        (250.0, 0.0),
        "à mi-bande"
    );
    assert_eq!(
        vitesse((500.0, 700.0), CADRE, 50.0),
        (0.0, -600.0),
        "en bas"
    );
    assert_eq!(
        vitesse((500.0, 40.0), CADRE, 50.0),
        (0.0, 600.0),
        "au-delà du haut : plein"
    );
    assert_eq!(
        vitesse((0.0, 100.0), CADRE, 50.0),
        (1000.0, 600.0),
        "dans un coin, les deux"
    );
}

/// **Un cadre plus étroit que deux bandes les partage** : son milieu ne défile pas, et rien
/// ne se contredit.
#[test]
fn test_un_cadre_etroit_partage_ses_bandes() {
    let etroit = Cadre {
        gauche: 0.0,
        haut: 0.0,
        droite: 60.0,
        bas: 60.0,
    };
    assert_eq!(vitesse((30.0, 30.0), etroit, 50.0), (0.0, 0.0));
    assert!(vitesse((1.0, 30.0), etroit, 50.0).0 > 0.0);
}

/// Une application vierge, l'origine du monde au milieu du canevas, une image en son centre.
fn app_avec_une_image() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.viewport.x = 400.0;
        b.viewport.y = 300.0;
    }
    app.store
        .add_image(&board, BoardImage::new("i", 0.0, 0.0, 200.0, 100.0));
    app.store.clear_selection();
    app.une_image_sans_fenetre(taille(&app));
    app
}

fn taille(app: &GlucoseApp) -> (u32, u32) {
    let (w, h) = app.taille_de_la_fenetre();
    (w as u32, h as u32)
}

/// Joue des images comme la boucle le fait, chacune présentée une période d'écran à 240 Hz
/// après la précédente : c'est le temps qui passe qui fait défiler.
fn images(app: &mut GlucoseApp, n: usize) {
    let periode = std::time::Duration::from_micros(4_167);
    app.horloge.accorder(periode);
    let mut t = app
        .horloge
        .derniere_presentation()
        .unwrap_or_else(std::time::Instant::now);
    let (w, h) = taille(app);
    for _ in 0..n {
        t += periode;
        app.horloge.presentee(t, true);
        app.bouger_la_camera(w, h);
    }
}

fn image(app: &GlucoseApp) -> (f64, f64) {
    let i = app
        .store
        .active_board()
        .and_then(|b| b.images.first())
        .expect("l'image");
    (i.x, i.y)
}

/// Prend l'image en son centre, et la mène au bord droit du canevas.
fn tenir_l_image_au_bord_droit(app: &mut GlucoseApp) {
    let vp = app.store.viewport();
    let (sx, sy) = crate::canvas::world_to_screen(0.0, 0.0, &vp);
    app.handle_cursor_moved(PhysicalPosition::new(sx, sy));
    let (w, h) = app.taille_de_la_fenetre();
    app.handle_mouse_down(MouseButton::Left, w, h);
    for x in [sx + 50.0, f64::from(w) - 20.0, f64::from(w) - 2.0] {
        app.handle_cursor_moved(PhysicalPosition::new(x, sy));
    }
    assert!(app.is_dragging_item, "le glisser a commencé");
}

/// **Tenue au bord, l'image avance dans le monde, et reste sous la main** : la vue défile
/// vers la droite à chaque image, main immobile, et le point de l'image que la main tient
/// est toujours sous elle. À l'envers : sans images jouées, rien ne bouge.
#[test]
fn test_defile_1_au_bord_la_vue_defile_et_l_image_suit() {
    let mut app = app_avec_une_image();
    tenir_l_image_au_bord_droit(&mut app);
    let (x_avant, y_avant) = image(&app);
    let vue_avant = app.store.viewport();
    assert!(
        app.attente_du_bord().is_some(),
        "la boucle se réveille pour défiler"
    );
    images(&mut app, 30);
    let (x_apres, y_apres) = image(&app);
    let vue_apres = app.store.viewport();
    assert!(vue_apres.x < vue_avant.x, "la vue défile vers la droite");
    assert!(
        x_apres > x_avant,
        "l'image avance dans le monde : {x_avant} → {x_apres}"
    );
    assert_eq!(y_apres, y_avant, "et pas en hauteur");
    let sous_la_main = crate::canvas::screen_to_world(app.mouse_pos.0, app.mouse_pos.1, &vue_apres);
    assert!(
        (sous_la_main.0 - x_apres).abs() < 1e-6,
        "le point tenu reste sous la main"
    );
}

/// **Sans geste, le bord ne fait rien** : la souris qui survole le bord ne fait pas défiler,
/// et la boucle ne se réveille pas pour lui.
#[test]
fn test_defile_1_sans_geste_le_bord_ne_fait_rien() {
    let mut app = app_avec_une_image();
    let (w, h) = app.taille_de_la_fenetre();
    app.handle_cursor_moved(PhysicalPosition::new(
        f64::from(w) - 2.0,
        f64::from(h) / 2.0,
    ));
    let vue = app.store.viewport();
    images(&mut app, 30);
    assert_eq!(app.store.viewport(), vue);
    assert_eq!(app.vitesse_au_bord((w, h)), (0.0, 0.0));
    assert_eq!(app.attente_du_bord(), None, "et ne réveille pas la boucle");
}

/// **Le rectangle de sélection garde son premier coin dans le monde** pendant que la vue
/// défile : il grandit vers ce qui entre, au lieu de glisser avec l'écran.
#[test]
fn test_defile_1_le_rectangle_garde_son_coin_dans_le_monde() {
    let mut app = app_avec_une_image();
    let (w, h) = app.taille_de_la_fenetre();
    // Dans le vide, au-dessus et à gauche de l'image (300–500 × 250–350 à l'écran).
    app.handle_cursor_moved(PhysicalPosition::new(100.0, 200.0));
    app.handle_mouse_down(MouseButton::Left, w, h);
    app.handle_cursor_moved(PhysicalPosition::new(
        f64::from(w) - 2.0,
        f64::from(h) / 2.0,
    ));
    let Some((x1, y1, _, _)) = app.selection_box else {
        panic!("un rectangle de sélection");
    };
    let coin = crate::canvas::screen_to_world(x1, y1, &app.store.viewport());
    images(&mut app, 30);
    let (nx, ny, _, _) = app.selection_box.expect("toujours tracé");
    let encore = crate::canvas::screen_to_world(nx, ny, &app.store.viewport());
    assert!(nx < x1, "le coin a glissé à l'écran avec le monde");
    assert!(
        (encore.0 - coin.0).abs() < 1e-6 && (encore.1 - coin.1).abs() < 1e-6,
        "il est resté au même point du monde"
    );
}
