//! Preuve visuelle de RESIZE-1 : la même scène avant et après le geste, en PNG.
//!
//! Comme `renderer/card/proof.rs`, on ne fait pas confiance aux nombres qui ont servi à
//! dessiner : on mesure **l'encre** posée sur la frame. Une image tirée par un coin doit
//! occuper à l'écran exactement la boîte que le document range — et rester un cercle, pas
//! une ellipse ; une carte de texte rétrécie en largeur doit porter un texte plus haut,
//! parce qu'il a reflué.

use super::tests::{drag_by, press_handle, release, render_frame, text_card};
use super::*;
use crate::canvas::world_to_screen;
use glucose_core::types::BoardImage;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Transform};

/// Couleur de l'image de la preuve, unique dans la scène : c'est elle que l'on compte.
const INK: (u8, u8, u8) = (250, 120, 30);

/// Dessine sur disque une image 200×100 : un fond uni et un disque. Le disque ne reste un
/// disque que si le rapport est conservé.
fn probe_image(dir: &std::path::Path) -> String {
    let mut pixmap = Pixmap::new(200, 100).expect("pixmap");
    pixmap.fill(Color::from_rgba8(INK.0, INK.1, INK.2, 255));
    let mut pb = PathBuilder::new();
    pb.push_circle(100.0, 50.0, 40.0);
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(20, 20, 40, 255));
    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
    let path = dir.join("probe.png");
    pixmap.save_png(&path).expect("écriture de l'image d'essai");
    path.to_string_lossy().to_string()
}

/// Boîte englobante des pixels de couleur `INK` (à une tolérance de mélange près).
fn ink_bbox(frame: &Pixmap) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let w = frame.width();
    let (pixels, _) = frame.data().as_chunks::<4>();
    for (i, p) in pixels.iter().enumerate() {
        let close = |a: u8, b: u8| a.abs_diff(b) <= 6;
        if close(p[0], INK.0) && close(p[1], INK.1) && close(p[2], INK.2) {
            let (x, y) = (i as u32 % w, i as u32 / w);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x + 1);
            y1 = y1.max(y + 1);
        }
    }
    (x0 != u32::MAX).then_some((x0, y0, x1, y1))
}

/// Boîte englobante des pixels qui diffèrent entre deux frames.
fn diff_bbox(a: &Pixmap, b: &Pixmap) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let w = a.width();
    let (left, _) = a.data().as_chunks::<4>();
    let (right, _) = b.data().as_chunks::<4>();
    for (i, (p, q)) in left.iter().zip(right).enumerate() {
        if p != q {
            let (x, y) = (i as u32 % w, i as u32 / w);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x + 1);
            y1 = y1.max(y + 1);
        }
    }
    (x0 != u32::MAX).then_some((x0, y0, x1, y1))
}

fn screen_box(app: &GlucoseApp, rect: AlignRect) -> (f64, f64, f64, f64) {
    let vp = app.store.active_board().map(|b| b.viewport).unwrap_or_default();
    let (x0, y0) = world_to_screen(rect.left, rect.top, &vp);
    let (x1, y1) = world_to_screen(rect.left + rect.width, rect.top + rect.height, &vp);
    (x0, y0, x1, y1)
}

fn assert_box_matches(observed: (u32, u32, u32, u32), expected: (f64, f64, f64, f64), what: &str) {
    let pairs = [
        (observed.0 as f64, expected.0),
        (observed.1 as f64, expected.1),
        (observed.2 as f64, expected.2),
        (observed.3 as f64, expected.3),
    ];
    for (o, e) in pairs {
        assert!((o - e).abs() <= 2.0, "{what} : bord à {o} px, le document dit {e} px ({observed:?} vs {expected:?})");
    }
}

#[test]
fn test_resize_proof_an_image_pulled_by_a_corner_keeps_its_ratio_on_screen() {
    let dir = std::path::Path::new("target/resize-proof");
    std::fs::create_dir_all(dir).expect("dossier de capture");
    let src = probe_image(dir);

    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.viewport.x = 500.0;
        b.viewport.y = 400.0;
    }
    let mut img = BoardImage::new("probe", 0.0, 0.0, 200.0, 100.0);
    img.src = Some(src);
    app.store.add_image(&board, img);

    let before = render_frame(&mut app);
    let start = rect_of_image(&app.store.active_board().expect("board").images[0]);
    let ink_before = ink_bbox(&before).expect("l'image d'essai est visible");
    assert_box_matches(ink_before, screen_box(&app, start), "avant");
    before.save_png(dir.join("image-avant.png")).expect("png");

    press_handle(&mut app, start, Handle::BottomRight);
    drag_by(&mut app, 150.0, 20.0, 8);
    release(&mut app);

    let after = render_frame(&mut app);
    let resized = rect_of_image(&app.store.active_board().expect("board").images[0]);
    assert!((resized.width / resized.height - 2.0).abs() < 1e-9, "rapport conservé : {resized:?}");
    assert!(resized.width > 300.0, "{resized:?}");
    assert!((resized.left - start.left).abs() < 1e-9 && (resized.top - start.top).abs() < 1e-9, "ancre");
    let ink_after = ink_bbox(&after).expect("l'image redimensionnée est visible");
    assert_box_matches(ink_after, screen_box(&app, resized), "après");
    let drawn_ratio = (ink_after.2 - ink_after.0) as f64 / (ink_after.3 - ink_after.1) as f64;
    assert!((drawn_ratio - 2.0).abs() < 0.05, "à l'écran aussi : {drawn_ratio}");
    after.save_png(dir.join("image-apres.png")).expect("png");
    println!("[proof] image : {ink_before:?} -> {ink_after:?} ({})", dir.display());
}

#[test]
fn test_resize_proof_a_text_card_narrowed_by_its_side_reflows_its_text() {
    let dir = std::path::Path::new("target/resize-proof");
    std::fs::create_dir_all(dir).expect("dossier de capture");
    let text = "Une carte de texte redimensionnée en largeur reflue son texte : la largeur change, le découpage en lignes change, et la hauteur suit.";

    let frame_with = |text: &str, width: f64| {
        let mut app = GlucoseApp::new();
        let board = app.store.project.active_board_id.clone();
        if let Some(b) = app.store.active_board_mut() {
            b.annotations.clear();
            b.viewport.x = 300.0;
            b.viewport.y = 300.0;
        }
        app.store.add_annotation(&board, text_card("carte", 0.0, 0.0, width, text));
        app.fit_text_card_height("carte");
        app
    };

    // Avant : la carte large, et son encre de texte (frame pleine moins frame vide).
    let mut app = frame_with(text, 640.0);
    let before = render_frame(&mut app);
    let mut blank = frame_with("", 640.0);
    let before_blank = render_frame(&mut blank);
    let ink_before = diff_bbox(&before, &before_blank).expect("la carte porte du texte");
    let start = rect_of_annotation(&app.store.active_board().expect("board").annotations[0]).expect("boîte");
    before.save_png(dir.join("carte-avant.png")).expect("png");

    // Le geste : la poignée droite, tirée vers la gauche.
    press_handle(&mut app, start, Handle::Right);
    drag_by(&mut app, -400.0, 0.0, 8);
    release(&mut app);
    let after = render_frame(&mut app);
    let narrowed = rect_of_annotation(&app.store.active_board().expect("board").annotations[0]).expect("boîte");
    assert!((narrowed.width - 240.0).abs() < 1e-9, "{narrowed:?}");
    assert!(narrowed.height > start.height, "la hauteur suit le texte : {narrowed:?} vs {start:?}");

    let mut blank = frame_with("", 240.0);
    let after_blank = render_frame(&mut blank);
    let ink_after = diff_bbox(&after, &after_blank).expect("la carte rétrécie porte du texte");
    after.save_png(dir.join("carte-apres.png")).expect("png");

    let (w_before, h_before) = (ink_before.2 - ink_before.0, ink_before.3 - ink_before.1);
    let (w_after, h_after) = (ink_after.2 - ink_after.0, ink_after.3 - ink_after.1);
    println!("[proof] carte : encre {w_before}x{h_before} -> {w_after}x{h_after} ({})", dir.display());
    assert!(w_after < w_before, "le texte est plus étroit");
    assert!(h_after > h_before * 2, "et bien plus haut : il a reflué sur plusieurs lignes");
    // L'encre reste dans la carte : rien ne déborde à droite.
    let (_, _, right_edge, _) = screen_box(&app, narrowed);
    assert!((ink_after.2 as f64) <= right_edge, "le texte déborde de la carte");
}
