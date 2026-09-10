//! Moteur de rendu 2D Tiny-Skia pour Glucose Desktop (PureRef-style).

use crate::canvas::{screen_to_world, world_to_screen};
use crate::font::draw_text;
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use std::collections::HashMap;
use std::path::Path;
use tiny_skia::{
    Color, LineCap, Paint, PathBuilder, Pixmap, PixmapMut, PixmapPaint, Rect, Stroke,
    Transform,
};

pub struct Renderer {
    image_cache: HashMap<String, Pixmap>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            image_cache: HashMap::new(),
        }
    }

    /// Charge ou récupère une image décodée en Pixmap tiny-skia.
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
                    // Conversion RGBA standard en prémultiplié tiny-skia
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

    /// Rendu complet de la scène Glucose sur le buffer d'affichage.
    pub fn render(
        &mut self,
        pixmap: &mut PixmapMut,
        store: &Store,
        guides: &SnapGuides,
        selection_box: Option<(f64, f64, f64, f64)>,
        always_on_top: bool,
    ) {
        let width = pixmap.width();
        let height = pixmap.height();
        let vp = store
            .active_board()
            .map(|b| b.viewport)
            .unwrap_or(Viewport::default());

        // 1. Fond sombre sleek PureRef
        pixmap.fill(Color::from_rgba8(13, 14, 18, 255));

        // 2. Grille de points infinie
        self.draw_grid(pixmap, &vp, width, height);

        // 3. Membranes
        self.draw_membranes(pixmap, store, &vp);

        // 4. Images
        self.draw_images(pixmap, store, &vp);

        // 5. Annotations (stickies, textes, flèches)
        self.draw_annotations(pixmap, store, &vp);

        // 6. Guides d'alignement intelligents (Smart Align SNAP-1)
        self.draw_guides(pixmap, guides, &vp, width, height);

        // 7. Boîte de sélection élastique (Marquee)
        if let Some((x1, y1, x2, y2)) = selection_box {
            self.draw_selection_box(pixmap, x1, y1, x2, y2);
        }

        // 8. HUD PureRef (Overlay statut et raccourcis)
        self.draw_hud(pixmap, store, &vp, width, height, always_on_top);
    }

    fn draw_grid(&self, pixmap: &mut PixmapMut, vp: &Viewport, w: u32, h: u32) {
        let (min_wx, min_wy) = screen_to_world(0.0, 0.0, vp);
        let (max_wx, max_wy) = screen_to_world(w as f64, h as f64, vp);

        let grid_step = if vp.scale > 2.0 {
            20.0
        } else if vp.scale < 0.2 {
            100.0
        } else {
            40.0
        };

        let start_x = (min_wx / grid_step).floor() * grid_step;
        let end_x = (max_wx / grid_step).ceil() * grid_step;
        let start_y = (min_wy / grid_step).floor() * grid_step;
        let end_y = (max_wy / grid_step).ceil() * grid_step;

        let dot_color = Color::from_rgba8(255, 255, 255, 28);
        let mut paint = Paint::default();
        paint.set_color(dot_color);

        let mut gx = start_x;
        while gx <= end_x {
            let mut gy = start_y;
            while gy <= end_y {
                let (sx, sy) = world_to_screen(gx, gy, vp);
                if let Some(rect) = Rect::from_xywh(sx as f32, sy as f32, 1.5, 1.5) {
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
                gy += grid_step;
            }
            gx += grid_step;
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
                let sw = (*width * vp.scale).max(4.0);
                let sh = (*height * vp.scale).max(4.0);

                if let Some(rect) = Rect::from_xywh(sx as f32, sy as f32, sw as f32, sh as f32) {
                    // Fond translucide
                    let mut fill_paint = Paint::default();
                    fill_paint.set_color(Color::from_rgba8(96, 165, 250, 18));
                    pixmap.fill_rect(rect, &fill_paint, Transform::identity(), None);

                    // Bordure
                    let is_selected = store.selected_annotation_ids.contains(id);
                    let mut stroke_paint = Paint::default();
                    if is_selected {
                        stroke_paint.set_color(Color::from_rgba8(56, 189, 248, 255));
                    } else {
                        stroke_paint.set_color(Color::from_rgba8(96, 165, 250, 90));
                    }
                    let stroke = Stroke {
                        width: if is_selected { 2.5 } else { 1.5 },
                        ..Default::default()
                    };
                    let path = PathBuilder::from_rect(rect);
                    pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);

                    // Libellé de la membrane
                    if let Some(lbl) = text {
                        draw_text(
                            pixmap,
                            lbl,
                            (sx + 8.0) as i32,
                            (sy + 8.0) as i32,
                            1,
                            Color::from_rgba8(147, 197, 253, 200),
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

            // Tente de dessiner l'image chargée ou placeholder
            let mut drawn = false;
            if !src.is_empty() {
                if let Some(loaded_pixmap) = self.get_or_load_image(&src) {
                    let ts = Transform::from_translate(sx as f32, sy as f32)
                        .post_scale(sw / loaded_pixmap.width() as f32, sh / loaded_pixmap.height() as f32);
                    pixmap.draw_pixmap(0, 0, loaded_pixmap.as_ref(), &PixmapPaint::default(), ts, None);
                    drawn = true;
                }
            }

            if !drawn {
                // Placeholder géométrique propre pour image non chargée
                if let Some(rect) = Rect::from_xywh(sx as f32, sy as f32, sw, sh) {
                    let mut p = Paint::default();
                    p.set_color(Color::from_rgba8(30, 35, 45, 255));
                    pixmap.fill_rect(rect, &p, Transform::identity(), None);

                    let mut stroke_paint = Paint::default();
                    stroke_paint.set_color(Color::from_rgba8(60, 70, 85, 255));
                    let stroke = Stroke { width: 1.0, ..Default::default() };
                    let path = PathBuilder::from_rect(rect);
                    pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);

                    draw_text(
                        pixmap,
                        &format!("Image [{}]", id),
                        (sx + 10.0) as i32,
                        (sy + sh as f64 / 2.0 - 4.0) as i32,
                        1,
                        Color::from_rgba8(140, 150, 165, 200),
                    );
                }
            }

            // Outline de sélection cyan
            if is_selected {
                if let Some(rect) = Rect::from_xywh(sx as f32 - 2.0, sy as f32 - 2.0, sw + 4.0, sh + 4.0) {
                    let mut sel_paint = Paint::default();
                    sel_paint.set_color(Color::from_rgba8(56, 189, 248, 255));
                    let stroke = Stroke { width: 2.0, ..Default::default() };
                    let path = PathBuilder::from_rect(rect);
                    pixmap.stroke_path(&path, &sel_paint, &stroke, Transform::identity(), None);
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
                Annotation::Sticky { id, x, y, width, height, text, operator, .. } => {
                    let (sx, sy) = world_to_screen(*x, *y, vp);
                    let sw = (width.unwrap_or(160.0) * vp.scale).max(8.0) as f32;
                    let sh = (height.unwrap_or(120.0) * vp.scale).max(8.0) as f32;

                    if let Some(rect) = Rect::from_xywh(sx as f32, sy as f32, sw, sh) {
                        let mut p = Paint::default();
                        p.set_color(Color::from_rgba8(254, 240, 138, 230)); // Jaune sticky
                        pixmap.fill_rect(rect, &p, Transform::identity(), None);

                        let is_selected = store.selected_annotation_ids.contains(id);
                        let mut stroke_p = Paint::default();
                        stroke_p.set_color(if is_selected {
                            Color::from_rgba8(56, 189, 248, 255)
                        } else {
                            Color::from_rgba8(202, 138, 4, 180)
                        });
                        let stroke = Stroke {
                            width: if is_selected { 2.0 } else { 1.0 },
                            ..Default::default()
                        };
                        let path = PathBuilder::from_rect(rect);
                        pixmap.stroke_path(&path, &stroke_p, &stroke, Transform::identity(), None);

                        if let Some(op) = operator {
                            draw_text(
                                pixmap,
                                &format!("{:?}", op),
                                (sx + 8.0) as i32,
                                (sy + 6.0) as i32,
                                1,
                                Color::from_rgba8(161, 98, 7, 255),
                            );
                        }

                        draw_text(
                            pixmap,
                            text,
                            (sx + 8.0) as i32,
                            (sy + 22.0) as i32,
                            1,
                            Color::from_rgba8(28, 25, 23, 255),
                        );
                    }
                }
                Annotation::Text { id, x, y, text, .. } => {
                    let (sx, sy) = world_to_screen(*x, *y, vp);
                    let is_selected = store.selected_annotation_ids.contains(id);

                    draw_text(
                        pixmap,
                        text,
                        sx as i32,
                        sy as i32,
                        1,
                        if is_selected {
                            Color::from_rgba8(56, 189, 248, 255)
                        } else {
                            Color::from_rgba8(241, 245, 249, 255)
                        },
                    );
                }
                Annotation::Arrow { id, x, y, x2, y2, .. } => {
                    let (sx1, sy1) = world_to_screen(*x, *y, vp);
                    let (sx2, sy2) = world_to_screen(*x2, *y2, vp);

                    let mut pb = PathBuilder::new();
                    pb.move_to(sx1 as f32, sy1 as f32);
                    pb.line_to(sx2 as f32, sy2 as f32);

                    if let Some(path) = pb.finish() {
                        let is_selected = store.selected_annotation_ids.contains(id);
                        let mut stroke_paint = Paint::default();
                        stroke_paint.set_color(if is_selected {
                            Color::from_rgba8(56, 189, 248, 255)
                        } else {
                            Color::from_rgba8(148, 163, 184, 220)
                        });
                        let stroke = Stroke {
                            width: if is_selected { 3.0 } else { 2.0 },
                            line_cap: LineCap::Round,
                            ..Default::default()
                        };
                        pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
                    }
                }
                _ => {}
            }
        }
    }

    fn draw_guides(&self, pixmap: &mut PixmapMut, guides: &SnapGuides, vp: &Viewport, w: u32, h: u32) {
        let mut guide_paint = Paint::default();
        guide_paint.set_color(Color::from_rgba8(236, 72, 153, 200)); // Magenta SNAP-1
        let stroke = Stroke {
            width: 1.0,
            ..Default::default()
        };

        if let Some(ref xs) = guides.x {
            for &gx in xs {
                let (sx, _) = world_to_screen(gx, 0.0, vp);
                let mut pb = PathBuilder::new();
                pb.move_to(sx as f32, 0.0);
                pb.line_to(sx as f32, h as f32);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &guide_paint, &stroke, Transform::identity(), None);
                }
            }
        }

        if let Some(ref ys) = guides.y {
            for &gy in ys {
                let (_, sy) = world_to_screen(0.0, gy, vp);
                let mut pb = PathBuilder::new();
                pb.move_to(0.0, sy as f32);
                pb.line_to(w as f32, sy as f32);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &guide_paint, &stroke, Transform::identity(), None);
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
            let stroke = Stroke { width: 1.0, ..Default::default() };
            let path = PathBuilder::from_rect(rect);
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
        }
    }

    fn draw_hud(
        &self,
        pixmap: &mut PixmapMut,
        store: &Store,
        vp: &Viewport,
        w: u32,
        h: u32,
        always_on_top: bool,
    ) {
        let board_name = store.active_board().map(|b| b.name.as_str()).unwrap_or("Main");
        let img_count = store.active_board().map(|b| b.images.len()).unwrap_or(0);
        let ann_count = store.active_board().map(|b| b.annotations.len()).unwrap_or(0);

        // Status gauche
        let info_text = format!(
            "GLUCOSE | Board: {} | {} images | {} notes",
            board_name, img_count, ann_count
        );
        draw_text(
            pixmap,
            &info_text,
            12,
            h as i32 - 20,
            1,
            Color::from_rgba8(148, 163, 184, 180),
        );

        // Zoom & statut droite
        let zoom_pct = (vp.scale * 100.0).round() as i32;
        let top_indicator = if always_on_top { "[ON TOP]" } else { "" };
        let right_text = format!("{}  Zoom: {}%", top_indicator, zoom_pct);
        draw_text(
            pixmap,
            &right_text,
            w as i32 - (right_text.len() as i32 * 8) - 12,
            h as i32 - 20,
            1,
            Color::from_rgba8(148, 163, 184, 180),
        );

        // Aide raccourcis haut
        let help_text = "Clic-Glisser: Selection/Deplacement | Clic-Droit/Alt: Pan | Molette: Zoom | T: Toujours au-dessus | Del: Suppr";
        draw_text(
            pixmap,
            help_text,
            12,
            12,
            1,
            Color::from_rgba8(100, 116, 139, 150),
        );
    }
}
