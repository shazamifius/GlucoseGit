//! CHAMP-1 — les champs de nombre à la vraie souris : glisser, cliquer et taper, valider en
//! cliquant ailleurs ; et Ordonner qui range avec la valeur tapée.

use crate::app::GlucoseApp;
use crate::dock::champ::{Champ, Saisie};
use crate::dock::{cadre_du_panneau, organize, LayoutMode, TabId, WidgetRect};
use glucose_core::types::BoardImage;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};
use winit::window::CursorIcon;

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Une application vide, Ordonner ouvert.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    app.dock_manager.toggle_tab(TabId::Organize);
    app.ui.current_toast = None;
    app
}

/// Où un rectangle du panneau Ordonner est à l'écran.
fn a_l_ecran(
    app: &GlucoseApp,
    quoi: impl Fn(&organize::OrganizePanelLayout) -> WidgetRect,
) -> WidgetRect {
    let ecran = app.screen_frame(ECRAN.0, ECRAN.1);
    let cadre = cadre_du_panneau(&app.dock_manager, ecran, TabId::Organize).expect("ouvert");
    quoi(&organize::layout_organize_panel(
        cadre,
        &app.renderer.typography,
    ))
}

fn champ(app: &GlucoseApp, c: Champ) -> WidgetRect {
    let ecran = app.screen_frame(ECRAN.0, ECRAN.1);
    let cadre = cadre_du_panneau(&app.dock_manager, ecran, c.panneau()).expect("ouvert");
    c.rect(cadre, &app.renderer.typography, &app.dock_manager)
}

fn centre(r: WidgetRect) -> (f64, f64) {
    (f64::from(r.x + r.w / 2.0), f64::from(r.y + r.h / 2.0))
}

/// Appuie en `de`, glisse jusqu'à `a`, relâche.
fn glisser(app: &mut GlucoseApp, de: (f64, f64), a: (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(de.0, de.1));
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_cursor_moved(PhysicalPosition::new(a.0, a.1));
    app.handle_mouse_up(MouseButton::Left);
}

fn taper(app: &mut GlucoseApp, texte: &str) {
    for c in texte.chars() {
        assert!(app.prendre_la_touche(&Key::Character(c.to_string().into()), true, None));
    }
}

/// **Glisser sur la taille la règle**, et le curseur l'annonce.
#[test]
fn test_champ_1_glisser_regle_la_taille() {
    let mut app = app();
    let (x, y) = centre(champ(&app, Champ::OrdonnerTaille));
    app.handle_cursor_moved(PhysicalPosition::new(x, y));
    assert_eq!(app.current_cursor(), CursorIcon::EwResize);
    glisser(&mut app, (x, y), (x + 60.0, y));
    assert!(app.dock_manager.organize.size > 280.0, "elle a monté");
    assert_eq!(app.dock_manager.champ, None, "glissé, pas de frappe");
}

/// **Cliquer puis taper ; cliquer ailleurs valide.**
#[test]
fn test_champ_1_cliquer_taper_et_valider_ailleurs() {
    let mut app = app();
    let ecart = centre(champ(&app, Champ::OrdonnerEcart));
    glisser(&mut app, ecart, ecart);
    assert!(matches!(
        app.dock_manager.champ,
        Some(Saisie::Frappe { .. })
    ));
    taper(&mut app, "40");
    glisser(&mut app, (900.0, 500.0), (900.0, 500.0));
    assert_eq!(app.dock_manager.organize.gap, 40.0);
    assert_eq!(app.dock_manager.champ, None);
}

/// **Ordonner range avec la valeur tapée** : « Même hauteur » à 100, et chaque image a
/// 100 de haut — le champ n'est pas un décor.
#[test]
fn test_champ_1_ordonner_range_avec_la_valeur_tapee() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    for (k, id) in ["a", "b"].into_iter().enumerate() {
        let mut img = BoardImage::new(id, k as f64 * 400.0, 0.0, 300.0, 200.0);
        (img.original_width, img.original_height) = (300.0, 200.0);
        app.store.add_image(&board, img);
    }
    app.store.selected_image_ids.clear();
    app.dock_manager.organize.layout = LayoutMode::SameHeight;
    let taille = centre(champ(&app, Champ::OrdonnerTaille));
    glisser(&mut app, taille, taille);
    taper(&mut app, "100");
    assert!(app.prendre_la_touche(&Key::Named(NamedKey::Enter), true, None));
    let appliquer = centre(a_l_ecran(&app, |l| l.apply_rect));
    glisser(&mut app, appliquer, appliquer);
    let hauteurs: Vec<f64> = app
        .store
        .active_board()
        .expect("un tableau")
        .images
        .iter()
        .map(|i| i.height)
        .collect();
    assert_eq!(hauteurs, [100.0, 100.0]);
}

/// **Le champ qu'on tape se voit** : la vraie peinture du panneau change quand la frappe
/// s'ouvre — la valeur choisie, le contour.
#[test]
fn test_champ_1_la_frappe_se_voit() {
    let mut app = app();
    let r = champ(&app, Champ::OrdonnerEcart);
    let pixels = |app: &mut GlucoseApp| {
        app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
        let image = app.pixmap.as_ref().expect("une image");
        let (x0, y0) = (r.x as u32, r.y as u32);
        (x0..x0 + r.w as u32)
            .flat_map(|x| (y0..y0 + r.h as u32).map(move |y| (x, y)))
            .map(|(x, y)| image.pixel(x, y).expect("dans l'image"))
            .collect::<Vec<_>>()
    };
    let au_repos = pixels(&mut app);
    let c = centre(r);
    // Le pointeur s'en va : seul l'état du champ change, pas son survol.
    glisser(&mut app, c, c);
    app.handle_cursor_moved(PhysicalPosition::new(900.0, 500.0));
    assert_ne!(pixels(&mut app), au_repos);
}

/// **Les colonnes n'existent qu'en grille et en mosaïque** : en rangées compactes, cliquer où
/// elles seraient ne prend rien.
#[test]
fn test_champ_1_pas_de_colonnes_en_rangees_compactes() {
    let mut app = app();
    let c = centre(champ(&app, Champ::OrdonnerColonnes));
    glisser(&mut app, c, c);
    assert_eq!(app.dock_manager.champ, None);
    app.dock_manager.organize.layout = LayoutMode::Grid;
    glisser(&mut app, c, c);
    assert_eq!(
        app.dock_manager.champ.as_ref().map(Saisie::champ),
        Some(Champ::OrdonnerColonnes)
    );
}
