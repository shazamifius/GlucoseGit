//! Les passes de scène qui ne sont pas des annotations : grille, membranes, images,
//! guides d'alignement et boîte de sélection élastique.
//!
//! Les membranes suivent la même règle que les cartes (SCALE-1) : leur rayon, leurs halos,
//! leur cadre, leur pointillé et leur titre viennent d'une seule mise à l'échelle. Avant
//! R-45, le rayon était `clamp(4, 60)`, le décalage du titre `clamp(8, 20)` et
//! `clamp(12, 24)`, et sa police `clamp(12, 22)` — quatre seuils pour un seul cadre, plus
//! un pointillé de 10 px qui ne suivait pas le zoom du tout.
//!
//! Les guides, les poignées et les points de la grille, eux, gardent une **taille écran
//! constante** : c'est l'exception de SCALE-1, celle que le TSX écrit `1 / scale`.

use super::card::{Clip, SELECTION_RING};
use super::domain::{draw_domain_gauge, gauge_width, DomainTints};
use super::scale::WorldScale;
use super::{parse_hex_color, push_rounded_rect};
use crate::canvas::{screen_to_world, world_to_screen};
use crate::params::ViewPass;
use crate::theme::Theme;
use crate::typography::{TextStyle, Typography};
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use std::collections::{HashMap, HashSet};
use tiny_skia::{
    Color, FilterQuality, LineCap, Paint, PathBuilder, Pixmap, PixmapMut, PixmapPaint, Rect,
    Stroke, Transform,
};

/// Échelle de viewport en dessous de laquelle la grille n'est plus dessinable.
const MIN_GRID_SCALE: f64 = 1e-6;
/// Nombre maximal de doublements du pas de grille (borne la boucle d'adaptation).
const MAX_GRID_DOUBLINGS: u32 = 64;
/// Pas nominal de la grille, en unités monde.
const GRID_STEP: f64 = 60.0;
/// Espacement minimal des points à l'écran, en pixels : en deçà, le pas double.
const GRID_MIN_SCREEN_STEP: f64 = 32.0;
/// Rayon d'un point de grille, en pixels écran (exception SCALE-1 : repère, pas contenu).
const GRID_DOT_RADIUS: f32 = 1.2;

/// Rayon des coins d'une membrane, en unités monde.
const MEMBRANE_RADIUS: f32 = 60.0;
/// Débords des deux couches de halo d'une membrane, en unités monde.
const MEMBRANE_GLOW: [(f32, u8); 2] = [(20.0, 8), (10.0, 14)];
/// Décalage horizontal du titre depuis le bord gauche, en unités monde.
const MEMBRANE_LABEL_DX: f32 = 16.0;
/// Élévation du titre au-dessus du bord haut, en unités monde.
const MEMBRANE_LABEL_DY: f32 = 18.0;
/// Corps du titre d'une membrane, en unités monde.
const MEMBRANE_LABEL_FONT: f32 = 16.0;
/// Épaisseur du cadre d'une membrane, en unités monde.
const MEMBRANE_BORDER: f32 = 2.0;
/// Longueur d'un tiret du pointillé, en unités monde.
const MEMBRANE_DASH: f32 = 10.0;
/// Côté d'une poignée de redimensionnement, en pixels écran (affordance).
const HANDLE_SIZE: f32 = 7.0;
/// Côté d'une poignée d'image, en pixels écran (affordance).
const IMAGE_HANDLE_SIZE: f32 = 8.0;

// ── Grille ──────────────────────────────────────────────────────────────────

pub(super) fn draw_grid(pixmap: &mut PixmapMut, vp: &Viewport, w: u32, h: u32, header_h: f32) {
    // Une échelle nulle, négative ou NaN rendait la boucle d'adaptation du pas
    // infinie : on la borne inconditionnellement.
    if !vp.scale.is_finite() || vp.scale <= MIN_GRID_SCALE {
        return;
    }

    let (min_wx, min_wy) = screen_to_world(0.0, header_h as f64, vp);
    let (max_wx, max_wy) = screen_to_world(w as f64, h as f64, vp);
    if ![min_wx, min_wy, max_wx, max_wy].iter().all(|v| v.is_finite()) {
        return;
    }

    // Pas dynamique adaptatif : ne descend jamais sous ~32 px à l'écran pour éviter toute
    // explosion CPU. C'est un choix de densité, pas une borne sur une longueur du monde.
    let mut step = GRID_STEP;
    let mut doublings = 0u32;
    while step * vp.scale < GRID_MIN_SCREEN_STEP && doublings < MAX_GRID_DOUBLINGS {
        step *= 2.0;
        doublings += 1;
    }

    let start_x = (min_wx / step).floor() * step;
    let end_x = (max_wx / step).ceil() * step;
    let start_y = (min_wy / step).floor() * step;
    let end_y = (max_wy / step).ceil() * step;

    let mut dot_paint = Paint { anti_alias: true, ..Default::default() };
    dot_paint.set_color(Color::from_rgba8(255, 255, 255, 22));

    let mut pb = PathBuilder::new();
    let mut gx = start_x;
    while gx <= end_x {
        let mut gy = start_y;
        while gy <= end_y {
            let (sx, sy) = world_to_screen(gx, gy, vp);
            if sy >= header_h as f64 && sx >= 0.0 && sx <= w as f64 && sy <= h as f64 {
                pb.push_circle(sx as f32, sy as f32, GRID_DOT_RADIUS);
            }
            gy += step;
        }
        gx += step;
    }
    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, &dot_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
}

// ── Membranes ───────────────────────────────────────────────────────────────

/// Mise en page d'une membrane. **En unités monde tant que `scaled` n'a pas été appelée.**
#[derive(Clone, Copy)]
struct MembraneLayout {
    width: f32,
    height: f32,
    radius: f32,
    label_dx: f32,
    label_dy: f32,
    label_font: f32,
    border: f32,
    dash: f32,
}

impl MembraneLayout {
    fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            radius: MEMBRANE_RADIUS,
            label_dx: MEMBRANE_LABEL_DX,
            label_dy: MEMBRANE_LABEL_DY,
            label_font: MEMBRANE_LABEL_FONT,
            border: MEMBRANE_BORDER,
            dash: MEMBRANE_DASH,
        }
    }

    /// L'UNIQUE transformation d'échelle de la membrane (SCALE-1).
    fn scaled(self, s: WorldScale) -> Self {
        Self {
            width: s.world(self.width),
            height: s.world(self.height),
            radius: s.world(self.radius),
            label_dx: s.world(self.label_dx),
            label_dy: s.world(self.label_dy),
            label_font: s.world(self.label_font),
            border: s.world(self.border),
            dash: s.world(self.dash),
        }
    }
}

pub(super) fn draw_membranes(
    typography: &Typography,
    tints: &DomainTints,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let scale = WorldScale::new(pass.vp.scale);
    let clip = Clip {
        width: pixmap.width() as f32,
        height: pixmap.height() as f32,
        top: pass.header_h,
    };

    for ann in &board.annotations {
        if !pass.visible_ids.contains(ann.id()) {
            continue;
        }
        let Annotation::Membrane { id, x, y, width, height, text, color, domains, .. } = ann else {
            continue;
        };
        let layout = MembraneLayout::new(*width as f32, *height as f32).scaled(scale);
        let (wx, wy) = world_to_screen(*x, *y, &pass.vp);
        let (sx, sy) = (wx as f32, wy as f32);
        if clip.rejects(sx, sy, layout.width, layout.height) {
            continue;
        }

        let tint = color
            .as_deref()
            .map(|c| parse_hex_color(c, 96, 165, 250))
            .unwrap_or((96, 165, 250));
        let selected = store.selected_annotation_ids.contains(id);
        draw_membrane_shape(pixmap, (sx, sy), &layout, tint, (selected, scale));

        if scale.draws_detail() {
            if let Some(label) = text.as_deref().filter(|l| !l.is_empty()) {
                typography.draw_text_with_outline(
                    pixmap,
                    label,
                    sx + layout.label_dx,
                    sy - layout.label_dy,
                    TextStyle { size: layout.label_font, color: Color::from_rgba8(tint.0, tint.1, tint.2, 255), bold: true },
                    Color::from_rgba8(11, 11, 18, 255),
                );
            }
        }
        if selected {
            draw_handles(pixmap, (sx, sy), (layout.width, layout.height), tint, HANDLE_SIZE);
        }
        // La réglette d'une membrane s'aligne à DROITE de son bord haut : le coin haut-gauche
        // est déjà occupé par le titre protecteur, et deux textes superposés ne se lisent ni
        // l'un ni l'autre.
        let gauge_x = sx + layout.width - gauge_width(scale, domains.len());
        draw_domain_gauge(typography, tints, pixmap, scale, (gauge_x, sy), domains);
    }
}

fn draw_membrane_shape(
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &MembraneLayout,
    tint: (u8, u8, u8),
    state: (bool, WorldScale),
) {
    let (selected, scale) = state;
    let (r, g, b) = tint;

    // 1. Halo ultra-discret multicouche — il appartient au cadre, donc il le suit.
    if scale.draws_detail() {
        for (pad_world, alpha) in MEMBRANE_GLOW {
            let pad = scale.world(pad_world);
            let mut pb = PathBuilder::new();
            push_rounded_rect(
                &mut pb,
                at.0 - pad,
                at.1 - pad,
                layout.width + pad * 2.0,
                layout.height + pad * 2.0,
                layout.radius + pad * 0.5,
            );
            if let Some(path) = pb.finish() {
                let mut paint = Paint { anti_alias: true, ..Default::default() };
                paint.set_color(Color::from_rgba8(r, g, b, alpha));
                pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
        }
    }

    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, at.0, at.1, layout.width, layout.height, layout.radius);
    let Some(path) = pb.finish() else {
        return;
    };

    // 2. Remplissage translucide.
    let mut fill = Paint { anti_alias: true, ..Default::default() };
    fill.set_color(Color::from_rgba8(r, g, b, 8));
    pixmap.fill_path(&path, &fill, tiny_skia::FillRule::Winding, Transform::identity(), None);

    // 3. Bordure pointillée, ou pleine si la membrane est sélectionnée.
    let mut border = Paint { anti_alias: true, ..Default::default() };
    border.set_color(if selected {
        Color::from_rgba8(r, g, b, 235)
    } else {
        Color::from_rgba8(r, g, b, 115)
    });
    let stroke = Stroke {
        width: if selected { scale.screen(SELECTION_RING) } else { layout.border },
        dash: if selected {
            None
        } else {
            tiny_skia::StrokeDash::new(vec![layout.dash, layout.dash], 0.0)
        },
        line_cap: LineCap::Round,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &border, &stroke, Transform::identity(), None);
}

/// Poignées de redimensionnement aux quatre coins — taille **écran** constante.
fn draw_handles(pixmap: &mut PixmapMut, at: (f32, f32), size: (f32, f32), tint: (u8, u8, u8), side: f32) {
    let mut fill = Paint::default();
    fill.set_color(Color::from_rgba8(26, 26, 26, 255));
    let mut border = Paint::default();
    border.set_color(Color::from_rgba8(tint.0, tint.1, tint.2, 255));
    let stroke = Stroke { width: 1.0, ..Default::default() };

    for (cx, cy) in [
        (at.0, at.1),
        (at.0 + size.0, at.1),
        (at.0, at.1 + size.1),
        (at.0 + size.0, at.1 + size.1),
    ] {
        if let Some(rect) = Rect::from_xywh(cx - side / 2.0, cy - side / 2.0, side, side) {
            pixmap.fill_rect(rect, &fill, Transform::identity(), None);
            pixmap.stroke_path(&PathBuilder::from_rect(rect), &border, &stroke, Transform::identity(), None);
        }
    }
}

// ── Images ──────────────────────────────────────────────────────────────────

pub(super) fn draw_images(
    image_cache: &mut HashMap<String, Pixmap>,
    failed_images: &mut HashSet<String>,
    typography: &Typography,
    tints: &DomainTints,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let scale = WorldScale::new(pass.vp.scale);
    let clip = Clip {
        width: pixmap.width() as f32,
        height: pixmap.height() as f32,
        top: pass.header_h,
    };

    for img in &board.images {
        if !pass.visible_ids.contains(img.id.as_str()) {
            continue;
        }
        let (wx, wy) = world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, &pass.vp);
        let (sx, sy) = (wx as f32, wy as f32);
        let sw = (img.width * pass.vp.scale) as f32;
        let sh = (img.height * pass.vp.scale) as f32;
        if clip.rejects(sx, sy, sw, sh) {
            continue;
        }

        let drawn = img.src.as_deref().filter(|s| !s.is_empty()).and_then(|src| {
            super::Renderer::load_image_impl(image_cache, failed_images, src).map(|loaded| {
                let ts = Transform::from_scale(sw / loaded.width() as f32, sh / loaded.height() as f32)
                    .post_translate(sx, sy);
                let paint = PixmapPaint { quality: FilterQuality::Bilinear, ..Default::default() };
                pixmap.draw_pixmap(0, 0, loaded.as_ref(), &paint, ts, None);
            })
        });
        if drawn.is_none() {
            draw_missing_image(typography, pixmap, (sx, sy), (sw, sh), &img.id);
        }
        if store.selected_image_ids.contains(&img.id) {
            draw_image_selection(pixmap, (sx, sy), (sw, sh));
        }
        draw_domain_gauge(typography, tints, pixmap, scale, (sx, sy), &img.domains);
    }
}

fn draw_missing_image(
    typography: &Typography,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    size: (f32, f32),
    id: &str,
) {
    let Some(rect) = Rect::from_xywh(at.0, at.1, size.0, size.1) else {
        return;
    };
    let mut fill = Paint::default();
    fill.set_color(Color::from_rgba8(30, 35, 45, 255));
    pixmap.fill_rect(rect, &fill, Transform::identity(), None);

    let mut border = Paint::default();
    border.set_color(Color::from_rgba8(60, 70, 85, 255));
    let stroke = Stroke { width: 1.0, ..Default::default() };
    pixmap.stroke_path(&PathBuilder::from_rect(rect), &border, &stroke, Transform::identity(), None);

    typography.draw_text(
        pixmap,
        &format!("Image [{id}]"),
        at.0 + 10.0,
        at.1 + size.1 / 2.0 - 6.0,
        TextStyle { size: 12.0, color: Color::from_rgba8(140, 150, 165, 200), bold: false },
    );
}

fn draw_image_selection(pixmap: &mut PixmapMut, at: (f32, f32), size: (f32, f32)) {
    let Some(rect) = Rect::from_xywh(at.0 - 1.0, at.1 - 1.0, size.0 + 2.0, size.1 + 2.0) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(56, 189, 248, 255));
    let stroke = Stroke { width: SELECTION_RING, ..Default::default() };
    pixmap.stroke_path(&PathBuilder::from_rect(rect), &paint, &stroke, Transform::identity(), None);

    let mut handle = Paint::default();
    handle.set_color(Color::from_rgba8(255, 255, 255, 255));
    let half = IMAGE_HANDLE_SIZE / 2.0;
    for (cx, cy) in [
        (at.0 - half, at.1 - half),
        (at.0 + size.0 - half, at.1 - half),
        (at.0 - half, at.1 + size.1 - half),
        (at.0 + size.0 - half, at.1 + size.1 - half),
    ] {
        if let Some(hr) = Rect::from_xywh(cx, cy, IMAGE_HANDLE_SIZE, IMAGE_HANDLE_SIZE) {
            pixmap.fill_rect(hr, &handle, Transform::identity(), None);
        }
    }
}

// ── Guides et boîte de sélection — taille écran constante ───────────────────

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
    let stroke = Stroke { width: 1.0, ..Default::default() };
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

pub(super) fn draw_selection_box(pixmap: &mut PixmapMut, a: (f64, f64), b: (f64, f64)) {
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
    fill.set_color(Color::from_rgba8(56, 189, 248, 30));
    pixmap.fill_rect(rect, &fill, Transform::identity(), None);

    let mut border = Paint::default();
    border.set_color(Color::from_rgba8(56, 189, 248, 180));
    let stroke = Stroke {
        width: 1.0,
        dash: tiny_skia::StrokeDash::new(vec![4.0, 3.0], 0.0),
        ..Default::default()
    };
    pixmap.stroke_path(&PathBuilder::from_rect(rect), &border, &stroke, Transform::identity(), None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_grid_terminates_on_degenerate_scale() {
        let mut pixmap = Pixmap::new(320, 240).expect("pixmap 320x240");

        for bad_scale in [0.0_f64, -1.0, f64::NAN, f64::INFINITY, 1e-12] {
            let vp = Viewport { scale: bad_scale, ..Default::default() };
            let started = std::time::Instant::now();
            let mut view = pixmap.as_mut();
            draw_grid(&mut view, &vp, 320, 240, 40.0);
            assert!(
                started.elapsed().as_millis() < 500,
                "draw_grid ne se termine pas pour scale={bad_scale}"
            );
        }
    }

    #[test]
    fn test_scale_1_a_membrane_is_self_similar_at_every_zoom() {
        let world = MembraneLayout::new(800.0, 600.0);
        for zoom in [0.25_f64, 0.5, 1.0, 2.0, 4.0] {
            let screen = world.scaled(WorldScale::new(zoom));
            for (on_screen, in_world) in [
                (screen.radius, world.radius),
                (screen.label_dx, world.label_dx),
                (screen.label_dy, world.label_dy),
                (screen.label_font, world.label_font),
                (screen.border, world.border),
                (screen.dash, world.dash),
            ] {
                let expected = in_world / world.width;
                let observed = on_screen / screen.width;
                assert!(
                    (observed - expected).abs() < 1e-6,
                    "zoom {zoom} : rapport {observed} au lieu de {expected}"
                );
            }
        }
    }
}
