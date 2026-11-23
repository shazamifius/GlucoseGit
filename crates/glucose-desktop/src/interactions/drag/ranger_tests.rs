//! **RANGER-2 par le chemin de la souris** (fiche 134) : une photo glissée et lâchée sur un
//! dossier y entre, dans le même geste que le déplacement ; pendant le glisser, le dossier visé se
//! montre ; ailleurs, rien ne change.

use super::tests::{app, drag_by, press_on, release, render, with_image};
use crate::app::GlucoseApp;
use crate::canvas::world_to_screen;
use glucose_core::types::CanvasFolder;

/// Le dossier « d », à droite de la photo « I1 » posée en (0, 0) : 400 unités plus loin, le
/// pointeur est dans sa boîte.
fn un_dossier_a_droite(app: &mut GlucoseApp) -> String {
    let board = app.store.project.active_board_id.clone();
    let mut d = CanvasFolder::new("d", "archive", "");
    (d.x, d.y) = (300.0, -75.0);
    app.store.create_folder(&board, d);
    with_image(app, "I1", 0.0, 0.0);
    app.store.active_board().expect("un tableau").folders[0]
        .child_board_id
        .clone()
}

fn sur(app: &GlucoseApp, tableau: &str) -> bool {
    app.store
        .project
        .boards
        .iter()
        .find(|b| b.id == tableau)
        .is_some_and(|b| b.images.iter().any(|i| i.id == "I1"))
}

/// **Lâchée sur un dossier, la photo y entre** ; pendant le glisser, le dossier visé est connu et
/// sa boîte se pose par-dessus ; un seul `Ctrl+Z` la rend à sa place d'avant le glisser.
#[test]
fn test_ranger_2_lachee_sur_un_dossier_la_photo_y_entre_et_ctrl_z_la_rend() {
    let mut app = app();
    let ouvre = un_dossier_a_droite(&mut app);
    let ici = app.store.project.active_board_id.clone();
    press_on(&mut app, "I1");
    drag_by(&mut app, 400.0, 20);
    assert_eq!(app.glisser.depot.as_deref(), Some("d"), "le dossier visé");
    assert!(app.boite_du_depot().is_some(), "sa boîte se dessine");
    release(&mut app);
    assert!(sur(&app, &ouvre), "dans le dossier");
    assert!(!sur(&app, &ici), "plus sur le tableau");
    assert_eq!(app.glisser.depot, None);
    assert!(app.store.undo(), "un seul geste");
    assert!(sur(&app, &ici));
    render(&mut app);
    let x = app.store.active_board().expect("un tableau").images[0].x;
    assert_eq!(x, 0.0, "à sa place d'avant le glisser");
}

/// **Lâchée ailleurs, rien n'entre nulle part** : le glisser reste un déplacement.
#[test]
fn test_ranger_2_lachee_ailleurs_elle_se_deplace_seulement() {
    let mut app = app();
    let ouvre = un_dossier_a_droite(&mut app);
    press_on(&mut app, "I1");
    drag_by(&mut app, -400.0, 20);
    assert_eq!(app.glisser.depot, None);
    release(&mut app);
    assert!(!sur(&app, &ouvre));
    let x = app.store.active_board().expect("un tableau").images[0].x;
    assert!(x < -300.0, "elle a suivi la main : {x}");
}

/// **Un dossier qu'on glisse ne se vise pas lui-même** : sous le pointeur, c'est lui, et il n'est
/// pas un dépôt.
#[test]
fn test_ranger_2_un_dossier_glisse_ne_se_vise_pas() {
    let mut app = app();
    un_dossier_a_droite(&mut app);
    app.store.selected_folder_ids = vec!["d".to_string()].into();
    let vp = app.store.viewport();
    let (sx, sy) = world_to_screen(400.0, 0.0, &vp);
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(sx, sy));
    app.init_item_drag(400.0, 0.0);
    app.handle_item_drag_move(sx + 10.0, sy);
    assert_eq!(app.glisser.depot, None);
}

/// **`Échap` pendant le glisser** : plus de dossier visé, rien de rangé.
#[test]
fn test_ranger_2_echap_n_y_range_rien() {
    let mut app = app();
    let ouvre = un_dossier_a_droite(&mut app);
    press_on(&mut app, "I1");
    drag_by(&mut app, 400.0, 20);
    app.abandonner_le_glisser();
    assert_eq!(app.glisser.depot, None);
    release(&mut app);
    assert!(!sur(&app, &ouvre));
}

/// Les pixels qui diffèrent entre deux images, en coordonnées d'écran.
fn differences(p: &[u8], q: &[u8], largeur: usize) -> Vec<(f64, f64)> {
    (0..p.len() / 4)
        .filter(|&i| p[4 * i..4 * i + 4] != q[4 * i..4 * i + 4])
        .map(|i| ((i % largeur) as f64, (i / largeur) as f64))
        .collect()
}

/// La boîte du dossier « d » à l'écran, élargie de trois pixels (l'anticrénelage du trait).
fn sur_le_dossier(app: &GlucoseApp) -> impl Fn(&(f64, f64)) -> bool {
    let vp = app.store.viewport();
    let (x0, y0) = world_to_screen(300.0, -75.0, &vp);
    let (x1, y1) = world_to_screen(500.0, 75.0, &vp);
    move |&(x, y)| x >= x0 - 3.0 && x <= x1 + 3.0 && y >= y0 - 3.0 && y <= y1 + 3.0
}

/// **Le dossier visé se dessine par le vrai chemin de peinture** (celui de la fenêtre) : pendant
/// le glisser, son image change sur lui.
#[test]
fn test_ranger_2_le_dossier_vise_se_dessine_par_la_peinture() {
    let mut app = app();
    un_dossier_a_droite(&mut app);
    press_on(&mut app, "I1");
    drag_by(&mut app, 400.0, 20);
    let taille = (1440, 900);
    let image = |app: &mut GlucoseApp| {
        app.une_image_sans_fenetre(taille);
        app.pixmap.as_ref().expect("une image").data().to_vec()
    };
    let avec = image(&mut app);
    app.glisser.depot = None;
    let sans = image(&mut app);
    let sur = sur_le_dossier(&app);
    let dessus = differences(&avec, &sans, taille.0 as usize)
        .into_iter()
        .filter(|p| sur(p))
        .count();
    assert!(dessus > 0, "le dossier visé ne se voit pas");
}

/// **Et rien d'autre ne change** : la même scène rendue deux fois, avec et sans le dépôt dans la
/// couche du dessus — un rendu direct, sans l'interface qui s'anime d'une image à l'autre.
#[test]
fn test_ranger_2_la_boite_du_depot_et_rien_d_autre() {
    use crate::params::{Pointer, SceneOverlay};
    let mut app = app();
    un_dossier_a_droite(&mut app);
    let boite = glucose_core::geometry::Rect::new(300.0, -75.0, 200.0, 150.0);
    let image = |app: &mut GlucoseApp, depot| {
        let mut pixmap = tiny_skia::Pixmap::new(1440, 900).expect("une image");
        let guides = app.active_guides.clone();
        let overlay = SceneOverlay {
            depot,
            ..SceneOverlay::sans_rien(&guides)
        };
        let regard = crate::renderer::Regard::immobile();
        let pointeur = Pointer { x: 0.0, y: 0.0 };
        let (vue, store, ui) = (&mut pixmap.as_mut(), &app.store, &mut app.ui);
        app.renderer
            .render(vue, store, ui, overlay, pointeur, regard);
        pixmap.data().to_vec()
    };
    let (sans, avec) = (image(&mut app, None), image(&mut app, Some(boite)));
    let changes = differences(&avec, &sans, 1440);
    assert!(!changes.is_empty(), "la boîte ne se dessine pas");
    let sur = sur_le_dossier(&app);
    assert!(changes.iter().all(sur), "un pixel hors du dossier a changé");
}
