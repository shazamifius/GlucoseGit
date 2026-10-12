//! PRESET-1 — poser un gabarit, de bout en bout : le clic dans le panneau, le fantôme sous le
//! curseur, le clic qui pose, l'annulation, et Échap.

use crate::app::GlucoseApp;
use crate::dock::preset::{layout_presets_panel, Contenu};
use crate::dock::{compute_panel_layouts, TabId};
use crate::params::{ScaledRect, ScreenFrame};
use winit::event::MouseButton;

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Une application au panneau des Presets ouvert.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    app.store.journal.clear();
    app.dock_manager.top_tabs.clear();
    app.dock_manager.bottom_tabs.clear();
    app.dock_manager.toggle_tab(TabId::Preset);
    app
}

/// Clique, à la vraie souris, la rangée de ce gabarit dans le panneau.
fn cliquer_le_gabarit(app: &mut GlucoseApp, rang: usize) {
    let ecran = ScreenFrame {
        width: ECRAN.0,
        height: ECRAN.1,
        header_h: app.ui.header_height(),
        scale: app.ui.scale_factor,
    };
    let s = crate::theme::clamp_ui_scale(ecran.scale);
    let panneau = compute_panel_layouts(&app.dock_manager, ECRAN.0, ECRAN.1, ecran.header_h, s)
        .into_iter()
        .find(|p| p.tab == TabId::Preset)
        .expect("le panneau est ouvert");
    let cadre = ScaledRect {
        x: panneau.x,
        y: panneau.y,
        w: panneau.width,
        h: panneau.height,
        scale: s,
    };
    let rect = layout_presets_panel(cadre, &Contenu::du(&app.store)).rows[rang].rect;
    app.mouse_pos = (
        f64::from(rect.x + rect.w / 2.0),
        f64::from(rect.y + rect.h / 2.0),
    );
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
}

/// Le point du monde sous ce point de l'écran.
fn monde(app: &GlucoseApp, (x, y): (f64, f64)) -> (f64, f64) {
    crate::canvas::screen_to_world(x, y, &app.store.viewport())
}

/// **Le geste entier** : un clic sur « Props & Items » arme le gabarit et ferme le panneau ; le
/// fantôme suit le curseur, cinq boîtes centrées sous lui ; le clic sur le canevas pose cinq
/// zones exactement là, et le gabarit cesse de suivre ; un seul `Ctrl+Z` les retire.
#[test]
fn test_preset_1_un_gabarit_se_pose_la_ou_on_l_a_vu() {
    let mut app = app();
    cliquer_le_gabarit(&mut app, 3);
    assert_eq!(app.ui.placement.gabarit.as_deref(), Some("props"));
    assert!(
        !app.dock_manager.top_tabs.contains(&TabId::Preset),
        "le panneau se ferme"
    );

    let ici = (700.0, 500.0);
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(ici.0, ici.1));
    let fantome = app.fantome_du_gabarit();
    assert_eq!(fantome.len(), 5);
    let centre = monde(&app, ici);
    let gauche = fantome[0].left;
    let droite = fantome[4].left + fantome[4].width;
    assert!(
        ((gauche + droite) / 2.0 - centre.0).abs() < 1e-6,
        "centré sous le curseur"
    );

    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
    let board = app.store.project.active_board_id.clone();
    let zones = app.store.zones(&board).to_vec();
    assert_eq!(zones.len(), 5);
    assert_eq!(
        (zones[0].x, zones[0].width),
        (fantome[0].left, fantome[0].width)
    );
    assert!(app.ui.placement.gabarit.is_none(), "il ne suit plus");
    assert!(app.fantome_du_gabarit().is_empty());

    assert!(app.store.undo());
    assert!(app.store.zones(&board).is_empty(), "un seul geste");
}

/// **Échap renonce** : le gabarit cesse de suivre, et un clic ensuite ne pose rien.
#[test]
fn test_preset_1_echap_renonce_au_gabarit() {
    let mut app = app();
    cliquer_le_gabarit(&mut app, 0);
    assert!(app.ui.placement.gabarit.is_some());
    assert!(app.renoncer_au_gabarit());
    assert!(app.ui.placement.gabarit.is_none());
    app.mouse_pos = (700.0, 500.0);
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
    let board = app.store.project.active_board_id.clone();
    assert!(app.store.zones(&board).is_empty());
}

/// La touche Échap elle-même y mène.
#[test]
fn test_preset_1_la_touche_echap() {
    let mut app = app();
    cliquer_le_gabarit(&mut app, 0);
    app.handle_shortcut_input(
        &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
        winit::event::ElementState::Pressed,
    );
    assert!(app.ui.placement.gabarit.is_none());
}

/// **Le fantôme suit le curseur** : un mouvement sans gabarit armé ne redemande rien, un
/// mouvement avec en redemande une image — sans quoi le fantôme resterait où il était.
#[test]
fn test_preset_1_le_fantome_se_redessine_quand_le_curseur_bouge() {
    let mut app = app();
    app.dock_manager.toggle_tab(TabId::Preset);
    let bouger = |app: &mut GlucoseApp, x: f64| {
        let _ = app.prendre_la_salissure();
        app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(x, 500.0));
        !app.prendre_la_salissure().est_propre()
    };
    assert!(!bouger(&mut app, 700.0), "sans gabarit, rien à redessiner");
    app.dock_manager.toggle_tab(TabId::Preset);
    cliquer_le_gabarit(&mut app, 3);
    assert!(bouger(&mut app, 720.0), "le fantôme suit");
}

/// **Le fantôme se voit** : une boîte par zone, au bord dessiné — la vraie peinture, de la
/// scène par-dessus au pixel.
#[test]
fn test_preset_1_le_fantome_se_peint() {
    let mut app = app();
    cliquer_le_gabarit(&mut app, 3);
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(700.0, 500.0));
    let zone = app.fantome_du_gabarit()[2];
    let vp = app.store.viewport();
    let (x, y) = crate::canvas::world_to_screen(zone.left, zone.top + zone.height / 2.0, &vp);
    let pixel = |app: &mut GlucoseApp| {
        app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
        let p = app.pixmap.as_ref().expect("une image");
        p.pixel(x.round() as u32, y.round() as u32)
            .expect("dans l'image")
    };
    let avec = pixel(&mut app);
    assert!(app.renoncer_au_gabarit());
    let sans = pixel(&mut app);
    assert_ne!(avec, sans, "le bord du fantôme");
}
