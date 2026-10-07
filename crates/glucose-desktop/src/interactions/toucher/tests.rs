//! Le toucher (fiche 54), joué sur le vrai [`GlucoseApp`] par les mêmes événements que `winit`
//! livre sur un téléphone — un doigt qui se pose, bouge, se lève.

use super::*;
use crate::canvas::screen_to_world;
use glucose_core::types::BoardImage;
use winit::dpi::PhysicalPosition;
use winit::event::DeviceId;

fn doigt(app: &mut GlucoseApp, id: u64, phase: TouchPhase, (x, y): (f64, f64)) {
    app.toucher(&Touch {
        device_id: DeviceId::dummy(),
        phase,
        location: PhysicalPosition::new(x, y),
        force: None,
        id,
    });
}

/// Une application sans rien, vue centrée.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.viewport.x = 400.0;
        b.viewport.y = 300.0;
    }
    app
}

/// Joue des images, comme la boucle le fait — chacune présentée un balayage d'un écran à
/// 240 Hz après la précédente : c'est le temps qui passe qui fait glisser le doigt.
fn images(app: &mut GlucoseApp, n: usize) {
    let periode = std::time::Duration::from_micros(4_167);
    app.horloge.accorder(periode);
    let mut t = app
        .horloge
        .derniere_presentation()
        .unwrap_or_else(std::time::Instant::now);
    for _ in 0..n {
        t += periode;
        app.horloge.presentee(t, true);
        app.bouger_la_camera(1280, 720);
    }
}

/// **Deux doigts décrivent une similitude** : de (100, 100)–(200, 100) à (50, 100)–(250, 100),
/// l'écart double et le milieu (150, 100) reste — `r = 2`, `b = (150, 100) − 2 · (150, 100)`.
/// Deux doigts au même point n'en décrivent aucune.
#[test]
fn test_deux_doigts_decrivent_une_similitude() {
    let m = similitude(
        [(100.0, 100.0), (200.0, 100.0)],
        [(50.0, 100.0), (250.0, 100.0)],
    )
    .expect("une similitude");
    assert_eq!(m.echelle, 2.0);
    assert_eq!(m.decalage, (-150.0, -100.0));
    assert_eq!(
        similitude([(5.0, 5.0), (5.0, 5.0)], [(5.0, 5.0), (9.0, 9.0)]),
        None
    );
}

/// **Un doigt sur le vide déplace le canevas, et glisse** : aucun cadre de sélection, et la vue
/// continue après que le doigt s'est levé — l'élan du doigt, que la souris n'a pas.
#[test]
fn test_un_doigt_sur_le_vide_deplace_le_canevas_et_glisse() {
    let mut app = app();
    let avant = app.store.viewport();
    doigt(&mut app, 1, TouchPhase::Started, (300.0, 200.0));
    for k in 1..=8 {
        doigt(
            &mut app,
            1,
            TouchPhase::Moved,
            (300.0 + 10.0 * k as f64, 200.0),
        );
        images(&mut app, 1);
    }
    assert!(app.selection_box.is_none(), "aucun cadre de sélection");
    doigt(&mut app, 1, TouchPhase::Ended, (380.0, 200.0));
    let au_lever = app.store.viewport().x;
    images(&mut app, 20);
    let apres = app.store.viewport();
    assert!(
        au_lever > avant.x,
        "la vue suit le doigt : {} -> {au_lever}",
        avant.x
    );
    assert!(
        apres.x > au_lever,
        "elle glisse après lui — la souris, elle, s'arrête net : {au_lever} -> {}",
        apres.x
    );
    assert!(!app.au_doigt(), "le doigt levé, plus rien ne le suit");
}

/// **Un doigt sur un nœud le glisse**, comme la souris.
#[test]
fn test_un_doigt_sur_un_noeud_le_glisse() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_image(&board, BoardImage::new("a", 0.0, 0.0, 100.0, 100.0));
    let vp = app.store.viewport();
    let (sx, sy) = crate::canvas::world_to_screen(0.0, 0.0, &vp);
    let profondeur = app.store.undo_depth();
    doigt(&mut app, 1, TouchPhase::Started, (sx, sy));
    doigt(&mut app, 1, TouchPhase::Moved, (sx + 30.0, sy));
    doigt(&mut app, 1, TouchPhase::Moved, (sx + 60.0, sy));
    doigt(&mut app, 1, TouchPhase::Ended, (sx + 60.0, sy));
    let x = app.store.active_board().map(|b| b.images[0].x).expect("a");
    assert!((x - 60.0 / vp.scale).abs() < 1e-6, "le nœud a suivi : {x}");
    assert_eq!(app.store.undo_depth(), profondeur + 1, "un geste");
}

/// **Deux doigts pincent autour de leur milieu** : écartés du double, l'échelle double, et le
/// point du monde sous leur milieu y reste.
#[test]
fn test_deux_doigts_pincent_autour_de_leur_milieu() {
    let mut app = app();
    let avant = app.store.viewport();
    let sous_le_milieu = screen_to_world(500.0, 300.0, &avant);
    doigt(&mut app, 1, TouchPhase::Started, (450.0, 300.0));
    doigt(&mut app, 2, TouchPhase::Started, (550.0, 300.0));
    doigt(&mut app, 2, TouchPhase::Moved, (600.0, 300.0));
    doigt(&mut app, 1, TouchPhase::Moved, (400.0, 300.0));
    images(&mut app, 1);
    let apres = app.store.viewport();
    assert!((apres.scale / avant.scale - 2.0).abs() < 1e-9, "{apres:?}");
    let ici = screen_to_world(500.0, 300.0, &apres);
    assert!(
        (ici.0 - sous_le_milieu.0).abs() < 1e-6 && (ici.1 - sous_le_milieu.1).abs() < 1e-6,
        "le point sous le milieu reste : {ici:?} contre {sous_le_milieu:?}"
    );
    assert!(app.selection_box.is_none());
}

/// **Le deuxième doigt termine le glisser du premier** : le nœud se pose où il est, et les
/// doigts pincent ensuite sans plus le déplacer.
#[test]
fn test_le_deuxieme_doigt_termine_le_glisser_du_premier() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_image(&board, BoardImage::new("a", 0.0, 0.0, 100.0, 100.0));
    let vp = app.store.viewport();
    let (sx, sy) = crate::canvas::world_to_screen(0.0, 0.0, &vp);
    let profondeur = app.store.undo_depth();
    doigt(&mut app, 1, TouchPhase::Started, (sx, sy));
    doigt(&mut app, 1, TouchPhase::Moved, (sx + 30.0, sy));
    doigt(&mut app, 2, TouchPhase::Started, (sx + 200.0, sy));
    let pose = app.store.active_board().map(|b| b.images[0].x).expect("a");
    assert!(!app.is_dragging_item, "le glisser est terminé");
    doigt(&mut app, 1, TouchPhase::Moved, (sx - 100.0, sy));
    doigt(&mut app, 2, TouchPhase::Moved, (sx + 300.0, sy));
    let x = app.store.active_board().map(|b| b.images[0].x).expect("a");
    assert_eq!(x, pose, "les doigts ne déplacent plus le nœud");
    assert_eq!(
        app.store.undo_depth(),
        profondeur + 1,
        "le glisser est un geste"
    );
    doigt(&mut app, 1, TouchPhase::Ended, (0.0, 0.0));
    doigt(&mut app, 2, TouchPhase::Ended, (0.0, 0.0));
    assert!(!app.au_doigt());
}

/// **`a` puis `b`, composés, envoient chaque point où les deux l'envoient l'un après l'autre** :
/// c'est ce qui permet de n'appliquer qu'une similitude par image, quel que soit le nombre de
/// doigts qui ont bougé.
#[test]
fn test_composer_est_appliquer_l_une_puis_l_autre() {
    let a = Mouvement {
        echelle: 1.5,
        decalage: (-225.0, 10.0),
    };
    let b = Mouvement {
        echelle: 4.0 / 3.0,
        decalage: (-200.0, -3.0),
    };
    let ab = composer(a, b);
    let suit = |m: &Mouvement, (x, y): (f64, f64)| {
        (m.echelle * x + m.decalage.0, m.echelle * y + m.decalage.1)
    };
    for q in [(0.0, 0.0), (450.0, 300.0), (-80.0, 1200.0)] {
        let (u, v) = (suit(&ab, q), suit(&b, suit(&a, q)));
        assert!(
            (u.0 - v.0).abs() < 1e-9 && (u.1 - v.1).abs() < 1e-9,
            "{q:?}"
        );
    }
}
