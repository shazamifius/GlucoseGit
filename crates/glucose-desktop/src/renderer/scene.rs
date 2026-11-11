//! Les passes de scène qui ne sont pas des annotations : grille, membranes, images,
//! guides d'alignement et boîte de sélection élastique.
//!
//! Les membranes ont leur module ([`membranes`]) : leur forme, la loi des régions (fiche 105),
//! leurs titres, leurs poignées.
//!
//! Les guides, les poignées et les points de la grille gardent une **taille écran constante** :
//! c'est l'exception de SCALE-1. Les poignées elles-mêmes sont dessinées par
//! [`super::handles`], aux positions que le test de clic utilise (RESIZE-1).

pub mod grid;
pub(super) mod image;
pub(super) mod membranes;

use crate::canvas::world_to_screen;
use crate::theme::Theme;
use glucose_core::smart_align::SnapGuides;
use glucose_core::types::Viewport;
use tiny_skia::{Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

pub(super) fn draw_guides(
    theme: &Theme,
    pixmap: &mut PixmapMut,
    guides: &SnapGuides,
    vp: &Viewport,
    size: (u32, u32),
    header_h: f32,
) {
    let mut paint = Paint::default();
    paint.set_color(theme.snap_guide);
    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    let (w, h) = size;

    for &gx in guides.x.iter().flatten() {
        let (sx, _) = world_to_screen(gx, 0.0, vp);
        let mut pb = PathBuilder::new();
        pb.move_to(sx as f32, header_h);
        pb.line_to(sx as f32, h as f32);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    for &gy in guides.y.iter().flatten() {
        let (_, sy) = world_to_screen(0.0, gy, vp);
        if sy < header_h as f64 {
            continue;
        }
        let mut pb = PathBuilder::new();
        pb.move_to(0.0, sy as f32);
        pb.line_to(w as f32, sy as f32);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}

/// **L'élément à naître** sous un outil de création armé (PLACEMENT-1) : sa boîte, là où un
/// clic le poserait, à l'habit de la sélection élastique — une affordance monochrome, comme
/// toute la chrome, et qui ne ressemble à rien de ce qu'un document contient.
pub(super) fn dessiner_le_fantome(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    fantome: glucose_core::geometry::Rect,
    vp: &Viewport,
) {
    let a = world_to_screen(fantome.left, fantome.top, vp);
    let b = world_to_screen(
        fantome.left + fantome.width,
        fantome.top + fantome.height,
        vp,
    );
    draw_selection_box(pixmap, theme, a, b);
}

/// Fiche 07 § 7.3 — la sélection élastique : contour blanc à 0,50 de 1 px, intérieur blanc
/// à 0,03. Monochrome, comme toute la chrome.
pub(super) fn draw_selection_box(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    a: (f64, f64),
    b: (f64, f64),
) {
    let rect = Rect::from_xywh(
        a.0.min(b.0) as f32,
        a.1.min(b.1) as f32,
        (a.0 - b.0).abs() as f32,
        (a.1 - b.1).abs() as f32,
    );
    let Some(rect) = rect else {
        return;
    };
    let mut fill = Paint::default();
    fill.set_color(theme.rubberband_fill);
    pixmap.fill_rect(rect, &fill, Transform::identity(), None);

    let mut border = Paint::default();
    border.set_color(theme.rubberband_stroke);
    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    pixmap.stroke_path(
        &PathBuilder::from_rect(rect),
        &border,
        &stroke,
        Transform::identity(),
        None,
    );
}
