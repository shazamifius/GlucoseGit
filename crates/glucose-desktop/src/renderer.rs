//! Moteur de rendu 2D haute fidélité pour Glucose Desktop (PureRef-style).
//! Utilise tiny-skia pour le rendu vectoriel anti-aliasé et fontdue pour la typographie.

use crate::canvas::{screen_to_world, world_to_screen};
use crate::typography::Typography;
use crate::ui::{render_ui, UiState, TOTAL_HEADER_HEIGHT};
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use std::collections::HashMap;
use std::path::Path;
use tiny_skia::{
    Color, FilterQuality, GradientStop, LineCap, Paint, PathBuilder, Pixmap, PixmapMut,
    PixmapPaint, Point, RadialGradient, Rect, SpreadMode, Stroke, Transform,
};

pub struct Renderer {
    pub image_cache: HashMap<String, Pixmap>,
    pub typography: Typography,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            image_cache: HashMap::new(),
            typography: Typography::new(),
        }
    }

    /// Charge ou récupère une image décodée en Pixmap tiny-skia (supporte WebP, PNG, JPG, GIF, BMP).
    pub fn get_or_load_image(&mut self, src_or_path: &str) -> Option<&Pixmap> {
        if self.image_cache.contains_key(src_or_path) {
            return self.image_cache.get(src_or_path);
        }

        let path = Path::new(src_or_path);
        if path.exists() {
            if let Ok(dyn_img) = image::open(path) {
                let rgba = dyn_img.to_rgba8();
                let (w, h) = rgba.dimensions();
                if let Some(mut pixmap) = Pixmap::new(w, h) {
                    let src_bytes = rgba.into_raw();
                    let dst_bytes = pixmap.data_mut();

                    // Conversion RGBA en prémultiplié pour tiny-skia
                    for i in 0..(w as usize * h as usize) {
                        let r = src_bytes[i * 4] as f32 / 255.0;
                        let g = src_bytes[i * 4 + 1] as f32 / 255.0;
                        let b = src_bytes[i * 4 + 2] as f32 / 255.0;
                        let a = src_bytes[i * 4 + 3] as f32 / 255.0;

                        dst_bytes[i * 4] = ((r * a) * 255.0) as u8;
                        dst_bytes[i * 4 + 1] = ((g * a) * 255.0) as u8;
                        dst_bytes[i * 4 + 2] = ((b * a) * 255.0) as u8;
                        dst_bytes[i * 4 + 3] = (a * 255.0) as u8;
                    }
                    self.image_cache.insert(src_or_path.to_string(), pixmap);
                    return self.image_cache.get(src_or_path);
                }
            }
        }
        None
    }

    /// Rendu complet de la scène Glucose et de son interface
    pub fn render(
        &mut self,
        pixmap: &mut PixmapMut,
        store: &Store,
        guides: &SnapGuides,
        selection_box: Option<(f64, f64, f64, f64)>,
        ui: &mut UiState,
        mouse_x: f32,
        mouse_y: f32,
    ) {
        let width = pixmap.width();
        let height = pixmap.height();
        let vp = store
            .active_board()
            .map(|b| b.viewport)
            .unwrap_or(Viewport::default());

        // 1. Fond sombre sleek PureRef #0D0E12
        pixmap.fill(Color::from_rgba8(13, 14, 18, 255));

        // 2. Grille de points infinie
        self.draw_grid(pixmap, &vp, width, height);

        // 3. Halos symbiotiques d'ambiance
        self.draw_halos(pixmap, store, &vp);

        // 4. Membranes
        self.draw_membranes(pixmap, store, &vp);

        // 5. Images
        self.draw_images(pixmap, store, &vp);

        // 6. Annotations (cartes de texte, stickies, flèches)
        self.draw_annotations(pixmap, store, &vp);

        // 7. Guides d'alignement intelligents (SNAP-1)
        if ui.smart_align {
            self.draw_guides(pixmap, guides, &vp, width, height);
        }

        // 8. Boîte de sélection élastique (Marquee)
        if let Some((x1, y1, x2, y2)) = selection_box {
            self.draw_selection_box(pixmap, x1, y1, x2, y2);
        }

        // 9. Interface utilisateur complète (TopBar, Tabs, Minimap, Toasts)
        render_ui(pixmap, store, ui, &self.typography, mouse_x, mouse_y);
    }

    fn draw_grid(&self, pixmap: &mut PixmapMut, vp: &Viewport, w: u32, h: u32) {
        let (min_wx, min_wy) = screen_to_world(0.0, TOTAL_HEADER_HEIGHT as f64, vp);
        let (max_wx, max_wy) = screen_to_world(w as f64, h as f64, vp);

        let grid_step = 60.0;
        let start_x = (min_wx / grid_step).floor() * grid_step;
        let end_x = (max_wx / grid_step).ceil() * grid_step;
        let start_y = (min_wy / grid_step).floor() * grid_step;
        let end_y = (max_wy / grid_step).ceil() * grid_step;

        let dot_paint = {
            let mut p = Paint::default();
            p.set_color(Color::from_rgba8(255, 255, 255, 20));
            p.anti_alias = true;
            p
        };

        let mut gx = start_x;
        while gx <= end_x {
            let mut gy = start_y;
            while gy <= end_y {
                let (sx, sy) = world_to_screen(gx, gy, vp);
                if sy >= TOTAL_HEADER_HEIGHT as f64 {
                    let mut pb = PathBuilder::new();
                    pb.push_circle(sx as f32, sy as f32, 1.2);
                    if let Some(path) = pb.finish() {
                        pixmap.fill_path(&path, &dot_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                    }
                }
                gy += grid_step;
            }
            gx += grid_step;
        }
    }

    fn draw_halos(&self, pixmap: &mut PixmapMut, store: &Store, vp: &Viewport) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };

        for ann in &board.annotations {
            if let Annotation::Text { x, y, width, height, .. } = ann {
                let (sx, sy) = world_to_screen(*x, *y, vp);
                let w = width.unwrap_or(80.0) * vp.scale;
                let h = height.unwrap_or(48.0) * vp.scale;

                let cx = (sx + w / 2.0) as f32;
                let cy = (sy + h / 2.0) as f32;
                let radius = (w.max(h) * 1.6) as f32;

                if radius > 10.0 {
                    if let Some(shader) = RadialGradient::new(
                        Point::from_xy(cx, cy),
                        Point::from_xy(cx, cy),
                        radius,
                        vec![
                            GradientStop::new(0.0, Color::from_rgba8(160, 90, 50, 24)),
                            GradientStop::new(1.0, Color::from_rgba8(160, 90, 50, 0)),
                        ],
                        SpreadMode::Pad,
                        Transform::identity(),
                    ) {
                        let mut p = Paint::default();
                        p.shader = shader;
                        p.anti_alias = true;

                        let mut pb = PathBuilder::new();
                        pb.push_circle(cx, cy, radius);
                        if let Some(path) = pb.finish() {
                            pixmap.fill_path(&path, &p, tiny_skia::FillRule::Winding, Transform::identity(), None);
                        }
                    }
                }
            }
        }
    }

    fn draw_membranes(&self, pixmap: &mut PixmapMut, store: &Store, vp: &Viewport) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };

        for ann in &board.annotations {
            if let Annotation::Membrane { id, x, y, width, height, text, .. } = ann {
                let (sx, sy) = world_to_screen(*x, *y, vp);
                let sw = (*width * vp.scale).max(4.0) as f32;
                let sh = (*height * vp.scale).max(4.0) as f32;

                if let Some(rect) = Rect::from_xywh(sx as f32, sy as f32, sw, sh) {
                    // Fond translucide bleu
                    let mut fill_paint = Paint::default();
                    fill_paint.set_color(Color::from_rgba8(96, 165, 250, 18));
                    pixmap.fill_rect(rect, &fill_paint, Transform::identity(), None);

                    // Bordure
                    let is_selected = store.selected_annotation_ids.contains(id);
                    let mut stroke_paint = Paint::default();
                    stroke_paint.set_color(if is_selected {
                        Color::from_rgba8(56, 189, 248, 255)
                    } else {
                        Color::from_rgba8(96, 165, 250, 100)
                    });
                    let stroke = Stroke {
                        width: if is_selected { 2.0 } else { 1.2 },
                        dash: tiny_skia::StrokeDash::new(vec![5.0, 3.0], 0.0),
                        ..Default::default()
                    };
                    let path = PathBuilder::from_rect(rect);
                    pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);

                    if let Some(lbl) = text {
                        self.typography.draw_text(
                            pixmap,
                            lbl,
                            sx as f32 + 10.0,
                            sy as f32 + 10.0,
                            12.0,
                            Color::from_rgba8(147, 197, 253, 220),
                            true,
                        );
                    }
                }
            }
        }
    }

    fn draw_images(&mut self, pixmap: &mut PixmapMut, store: &Store, vp: &Viewport) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };

        let images_data: Vec<_> = board
            .images
            .iter()
            .map(|img| {
                (
                    img.id.clone(),
                    img.src.clone().unwrap_or_default(),
                    img.x,
                    img.y,
                    img.width,
                    img.height,
                )
            })
            .collect();

        for (id, src, x, y, w, h) in images_data {
            let (sx, sy) = world_to_screen(x - w / 2.0, y - h / 2.0, vp);
            let sw = (w * vp.scale).max(4.0) as f32;
            let sh = (h * vp.scale).max(4.0) as f32;

            let is_selected = store.selected_image_ids.contains(&id);

            let mut drawn = false;
            if !src.is_empty() {
                if let Some(loaded_pixmap) = self.get_or_load_image(&src) {
                    let ts = Transform::from_translate(sx as f32, sy as f32)
                        .post_scale(sw / loaded_pixmap.width() as f32, sh / loaded_pixmap.height() as f32);
                    let mut pp = PixmapPaint::default();
                    pp.quality = FilterQuality::Bilinear;
                    pixmap.draw_pixmap(0, 0, loaded_pixmap.as_ref(), &pp, ts, None);
                    drawn = true;
                }
            }

            if !drawn {
                if let Some(rect) = Rect::from_xywh(sx as f32, sy as f32, sw, sh) {
                    let mut p = Paint::default();
                    p.set_color(Color::from_rgba8(30, 35, 45, 255));
                    pixmap.fill_rect(rect, &p, Transform::identity(), None);

                    let mut sp = Paint::default();
                    sp.set_color(Color::from_rgba8(60, 70, 85, 255));
                    let stroke = Stroke { width: 1.0, ..Default::default() };
                    let path = PathBuilder::from_rect(rect);
                    pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);

                    let label = format!("Image [{}]", id);
                    self.typography.draw_text(
                        pixmap,
                        &label,
                        sx as f32 + 10.0,
                        sy as f32 + sh / 2.0 - 6.0,
                        12.0,
                        Color::from_rgba8(140, 150, 165, 200),
                        false,
                    );
                }
            }

            // Outline de sélection avec poignées
            if is_selected {
                if let Some(rect) = Rect::from_xywh(sx as f32 - 1.0, sy as f32 - 1.0, sw + 2.0, sh + 2.0) {
                    let mut sel_paint = Paint::default();
                    sel_paint.set_color(Color::from_rgba8(56, 189, 248, 255));
                    let stroke = Stroke { width: 2.0, ..Default::default() };
                    let path = PathBuilder::from_rect(rect);
                    pixmap.stroke_path(&path, &sel_paint, &stroke, Transform::identity(), None);

                    // Poignées de coins
                    let corners = [
                        (sx as f32 - 4.0, sy as f32 - 4.0),
                        (sx as f32 + sw - 4.0, sy as f32 - 4.0),
                        (sx as f32 - 4.0, sy as f32 + sh - 4.0),
                        (sx as f32 + sw - 4.0, sy as f32 + sh - 4.0),
                    ];
                    let mut handle_paint = Paint::default();
                    handle_paint.set_color(Color::from_rgba8(255, 255, 255, 255));
                    for (cx, cy) in corners {
                        if let Some(hr) = Rect::from_xywh(cx, cy, 8.0, 8.0) {
                            pixmap.fill_rect(hr, &handle_paint, Transform::identity(), None);
                        }
                    }
                }
            }
        }
    }

    fn draw_annotations(&self, pixmap: &mut PixmapMut, store: &Store, vp: &Viewport) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };

        for ann in &board.annotations {
            match ann {
                Annotation::Text { id, x, y, width, height, text, .. } => {
                    let (sx, sy) = world_to_screen(*x, *y, vp);
                    let sw = (width.unwrap_or(80.0) * vp.scale).max(50.0) as f32;
                    let sh = (height.unwrap_or(48.0) * vp.scale).max(36.0) as f32;

                    let is_selected = store.selected_annotation_ids.contains(id);

                    // Carte en pilule arrondie #18181B (comme dans la capture Glucose)
                    let mut pb = PathBuilder::new();
                    let r = 18.0f32.min(sh / 2.0);
                    pb.move_to(sx as f32 + r, sy as f32);
                    pb.line_to(sx as f32 + sw - r, sy as f32);
                    pb.quad_to(sx as f32 + sw, sy as f32, sx as f32 + sw, sy as f32 + r);
                    pb.line_to(sx as f32 + sw, sy as f32 + sh - r);
                    pb.quad_to(sx as f32 + sw, sy as f32 + sh, sx as f32 + sw - r, sy as f32 + sh);
                    pb.line_to(sx as f32 + r, sy as f32 + sh);
                    pb.quad_to(sx as f32, sy as f32 + sh, sx as f32, sy as f32 + sh - r);
                    pb.line_to(sx as f32, sy as f32 + r);
                    pb.quad_to(sx as f32, sy as f32, sx as f32 + r, sy as f32);
                    pb.close();

                    if let Some(path) = pb.finish() {
                        let mut fill = Paint::default();
                        fill.set_color(Color::from_rgba8(24, 24, 27, 245));
                        fill.anti_alias = true;
                        pixmap.fill_path(&path, &fill, tiny_skia::FillRule::Winding, Transform::identity(), None);

                        let mut stroke_paint = Paint::default();
                        stroke_paint.set_color(if is_selected {
                            Color::from_rgba8(56, 189, 248, 255)
                        } else {
                            Color::from_rgba8(45, 45, 52, 255)
                        });
                        let stroke = Stroke {
                            width: if is_selected { 2.0 } else { 1.0 },
                            ..Default::default()
                        };
                        pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
                    }

                    // Texte vectoriel anti-aliasé centré
                    let font_size = (14.0 * vp.scale).clamp(11.0, 24.0) as f32;
                    let (tw, th) = self.typography.measure_text(text, font_size, false);
                    let tx = (sx as f32 + (sw - tw) / 2.0).max(sx as f32 + 8.0);
                    let ty = (sy as f32 + (sh - th) / 2.0).max(sy as f32 + 4.0);

                    self.typography.draw_text(
                        pixmap,
                        text,
                        tx,
                        ty,
                        font_size,
                        Color::from_rgba8(255, 255, 255, 255),
                        false,
                    );
                }
                Annotation::Sticky { id, x, y, width, height, text, operator, .. } => {
                    let (sx, sy) = world_to_screen(*x, *y, vp);
                    let sw = (width.unwrap_or(160.0) * vp.scale).max(60.0) as f32;
                    let sh = (height.unwrap_or(120.0) * vp.scale).max(40.0) as f32;

                    let is_selected = store.selected_annotation_ids.contains(id);

                    if let Some(rect) = Rect::from_xywh(sx as f32, sy as f32, sw, sh) {
                        let mut p = Paint::default();
                        p.set_color(Color::from_rgba8(254, 240, 138, 240));
                        pixmap.fill_rect(rect, &p, Transform::identity(), None);

                        let mut sp = Paint::default();
                        sp.set_color(if is_selected {
                            Color::from_rgba8(56, 189, 248, 255)
                        } else {
                            Color::from_rgba8(202, 138, 4, 180)
                        });
                        let stroke = Stroke {
                            width: if is_selected { 2.0 } else { 1.0 },
                            ..Default::default()
                        };
                        let path = PathBuilder::from_rect(rect);
                        pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);

                        let mut cur_ty = sy as f32 + 8.0;

                        if let Some(op) = operator {
                            let op_str = format!("{:?}", op);
                            self.typography.draw_text(
                                pixmap,
                                &op_str,
                                sx as f32 + 8.0,
                                cur_ty,
                                11.0,
                                Color::from_rgba8(161, 98, 7, 255),
                                true,
                            );
                            cur_ty += 16.0;
                        }

                        self.typography.draw_text(
                            pixmap,
                            text,
                            sx as f32 + 8.0,
                            cur_ty,
                            12.0,
                            Color::from_rgba8(28, 25, 23, 255),
                            false,
                        );
                    }
                }
                Annotation::Arrow { id, x, y, x2, y2, .. } => {
                    let (sx1, sy1) = world_to_screen(*x, *y, vp);
                    let (sx2, sy2) = world_to_screen(*x2, *y2, vp);

                    let is_selected = store.selected_annotation_ids.contains(id);
                    let color = if is_selected {
                        Color::from_rgba8(56, 189, 248, 255)
                    } else {
                        Color::from_rgba8(148, 163, 184, 220)
                    };

                    let mut pb = PathBuilder::new();
                    pb.move_to(sx1 as f32, sy1 as f32);
                    pb.line_to(sx2 as f32, sy2 as f32);

                    let angle = ((sy2 - sy1) as f32).atan2((sx2 - sx1) as f32);
                    let arrow_len = 12.0f32;
                    let arrow_angle = 0.45f32;

                    let left_x = sx2 as f32 - arrow_len * (angle - arrow_angle).cos();
                    let left_y = sy2 as f32 - arrow_len * (angle - arrow_angle).sin();
                    let right_x = sx2 as f32 - arrow_len * (angle + arrow_angle).cos();
                    let right_y = sy2 as f32 - arrow_len * (angle + arrow_angle).sin();

                    pb.move_to(sx2 as f32, sy2 as f32);
                    pb.line_to(left_x, left_y);
                    pb.move_to(sx2 as f32, sy2 as f32);
                    pb.line_to(right_x, right_y);

                    if let Some(path) = pb.finish() {
                        let mut p = Paint::default();
                        p.set_color(color);
                        p.anti_alias = true;
                        let stroke = Stroke {
                            width: if is_selected { 2.5 } else { 1.8 },
                            line_cap: LineCap::Round,
                            ..Default::default()
                        };
                        pixmap.stroke_path(&path, &p, &stroke, Transform::identity(), None);
                    }
                }
                _ => {}
            }
        }
    }

    fn draw_guides(&self, pixmap: &mut PixmapMut, guides: &SnapGuides, vp: &Viewport, w: u32, h: u32) {
        let mut guide_paint = Paint::default();
        guide_paint.set_color(Color::from_rgba8(236, 72, 153, 200));
        let stroke = Stroke { width: 1.0, ..Default::default() };

        if let Some(ref xs) = guides.x {
            for &gx in xs {
                let (sx, _) = world_to_screen(gx, 0.0, vp);
                let mut pb = PathBuilder::new();
                pb.move_to(sx as f32, TOTAL_HEADER_HEIGHT);
                pb.line_to(sx as f32, h as f32);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &guide_paint, &stroke, Transform::identity(), None);
                }
            }
        }

        if let Some(ref ys) = guides.y {
            for &gy in ys {
                let (_, sy) = world_to_screen(0.0, gy, vp);
                if sy >= TOTAL_HEADER_HEIGHT as f64 {
                    let mut pb = PathBuilder::new();
                    pb.move_to(0.0, sy as f32);
                    pb.line_to(w as f32, sy as f32);
                    if let Some(path) = pb.finish() {
                        pixmap.stroke_path(&path, &guide_paint, &stroke, Transform::identity(), None);
                    }
                }
            }
        }
    }

    fn draw_selection_box(&self, pixmap: &mut PixmapMut, x1: f64, y1: f64, x2: f64, y2: f64) {
        let left = x1.min(x2) as f32;
        let top = y1.min(y2) as f32;
        let width = (x1 - x2).abs() as f32;
        let height = (y1 - y2).abs() as f32;

        if let Some(rect) = Rect::from_xywh(left, top, width, height) {
            let mut fill = Paint::default();
            fill.set_color(Color::from_rgba8(56, 189, 248, 30));
            pixmap.fill_rect(rect, &fill, Transform::identity(), None);

            let mut stroke_paint = Paint::default();
            stroke_paint.set_color(Color::from_rgba8(56, 189, 248, 180));
            let stroke = Stroke {
                width: 1.0,
                dash: tiny_skia::StrokeDash::new(vec![4.0, 3.0], 0.0),
                ..Default::default()
            };
            let path = PathBuilder::from_rect(rect);
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
        }
    }
}
