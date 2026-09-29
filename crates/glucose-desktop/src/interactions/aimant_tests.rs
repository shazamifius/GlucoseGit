//! L'aimant de l'application : ce que l'écran montre, et les voisines seulement (SNAP-4).
//!
//! Chaque épreuve joue la même scène deux fois, et un seul changement — la caméra, ce qu'on
//! emporte, le focus — sépare l'aimant qui tire de celui qui ne tire pas.

use crate::app::GlucoseApp;
use glucose_core::smart_align::{snap_move_sur, AlignRect};
use glucose_core::types::{Annotation, BoardImage, Viewport};
use std::collections::HashSet;

/// Un tableau vide, l'origine du monde au coin haut-gauche du canevas, à l'échelle `e`.
fn app(e: f64) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    regarder(&mut app, e);
    app
}

fn regarder(app: &mut GlucoseApp, e: f64) {
    let b = app.store.project.active_board_id.clone();
    let bandeau = f64::from(app.ui.header_height());
    app.store.set_viewport(
        &b,
        Viewport {
            x: 0.0,
            y: bandeau,
            scale: e,
        },
    );
}

/// Une photo de 100 × 100 dont le coin haut-gauche est en `(x, y)`.
fn photo(app: &mut GlucoseApp, id: &str, (x, y): (f64, f64)) {
    let b = app.store.project.active_board_id.clone();
    app.store
        .add_image(&b, BoardImage::new(id, x + 50.0, y + 50.0, 100.0, 100.0));
}

/// De combien l'aimant tire `rect`, sans ce que le geste emporte (`exclus`).
fn tire(app: &mut GlucoseApp, rect: AlignRect, exclus: &[&str]) -> (f64, f64) {
    let exclus: HashSet<String> = exclus.iter().map(|s| s.to_string()).collect();
    let lignes = app.lignes_d_aimant(rect, &exclus);
    let snap = snap_move_sur(rect, &lignes, app.options_d_aimant());
    (snap.dx, snap.dy)
}

/// **Ce que l'écran ne montre pas n'aimante pas** : une photo alignée à trois unités, loin sous
/// l'écran. Dézoomé, elle entre à l'écran — et tire.
#[test]
fn test_snap_4_ce_que_l_ecran_ne_montre_pas_n_aimante_pas() {
    let mut app = app(1.0);
    photo(&mut app, "loin", (0.0, 5000.0));
    let rect = AlignRect::new(3.0, 100.0, 100.0, 100.0);
    assert_eq!(tire(&mut app, rect, &[]), (0.0, 0.0), "hors de l'écran");
    regarder(&mut app, 0.1);
    assert_eq!(tire(&mut app, rect, &[]), (-3.0, 0.0), "à l'écran");
}

/// **Ce qui est derrière la voisine n'aimante pas** : sous `rect`, une voisine qui n'est pas
/// alignée, et derrière elle une photo qui l'est. Quand le geste emporte la voisine, celle de
/// derrière devient la voisine — et tire.
#[test]
fn test_snap_4_ce_qui_est_derriere_la_voisine_n_aimante_pas() {
    let mut app = app(1.0);
    photo(&mut app, "voisine", (40.0, 250.0));
    photo(&mut app, "derriere", (0.0, 450.0));
    let rect = AlignRect::new(3.0, 100.0, 100.0, 100.0);
    assert_eq!(
        tire(&mut app, rect, &[]),
        (0.0, 0.0),
        "cachée par la voisine"
    );
    assert_eq!(
        tire(&mut app, rect, &["voisine"]),
        (-3.0, 0.0),
        "la voisine emportée, celle de derrière tire"
    );
}

/// **Ce que le focus cache n'aimante pas** (MEMB-2) : dans la membrane `M`, `rect` a pour
/// voisine de droite une photo hors de `M`, alignée à trois unités. En focus sur `M`, elle ne
/// se voit plus — et ne tire plus.
#[test]
fn test_snap_4_ce_que_le_focus_cache_n_aimante_pas() {
    let mut app = app(1.0);
    let b = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&b, Annotation::membrane("M", 0.0, 0.0, 1000.0, 600.0));
    photo(&mut app, "dehors", (1100.0, 100.0));
    let rect = AlignRect::new(100.0, 103.0, 100.0, 100.0);
    assert_eq!(tire(&mut app, rect, &[]), (0.0, -3.0), "hors focus");
    app.renderer.regler_le_focus(&app.store, Some("M"));
    assert!(!app.renderer.focus.laisse_voir("dehors"));
    assert_eq!(tire(&mut app, rect, &[]), (0.0, 0.0), "en focus sur M");
}
