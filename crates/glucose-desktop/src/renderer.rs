//! Moteur de rendu 2D logiciel pour Glucose Desktop (PureRef-style).
//! Entièrement basé sur notre propre Rasterizer pur Rust std (zéro dépendance vectorielle).

use crate::canvas::{screen_to_world, world_to_screen};
use crate::rasterizer::FrameBuffer;
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use std::collections::HashMap;
use std::path::Path;

pub struct DecodedImage {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>, // 0xAARRGGBB
}

pub struct Renderer {
    pub image_cache: HashMap<String, DecodedImage>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            image_cache: HashMap::new(),
        }
    }

    /// Charge ou récupère une image décodée en mémoire.
    pub fn get_or_load_image(&mut self, src_or_path: &str) -> Option<&DecodedImage> {
        if self.image_cache.contains_key(src_or_path) {
            return self.image_cache.get(src_or_path);
        }

        let path = Path::new(src_or_path);
        if path.exists() {
            if let Ok(dyn_img) = image::open(path) {
                let rgba = dyn_img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let raw = rgba.into_raw();
                let mut pixels = Vec::with_capacity((w * h) as usize);

                for chunk in raw.chunks_exact(4) {
                    let r = chunk[0] as u32;
                    let g = chunk[1] as u32;
                    let b = chunk[2] as u32;
                    let a = chunk[3] as u32;
                    pixels.push((a << 24) | (r << 16) | (g << 8) | b);
                }

                self.image_cache.insert(
                    src_or_path.to_string(),
                    DecodedImage {
                        width: w as usize,
                        height: h as usize,
                        pixels,
                    },
                );
                return self.image_cache.get(src_or_path);
            }
        }
        None
    }

    /// Rendu complet de la scène Glucose sur le framebuffer logiciel.
    pub fn render(
        &mut self,
        fb: &mut FrameBuffer,
        store: &Store,
        guides: &SnapGuides,
        selection_box: Option<(f64, f64, f64, f64)>,
        always_on_top: bool,
    ) {
        let vp = store
            .active_board()
            .map(|b| b.viewport)
            .unwrap_or(Viewport::default());

        // 1. Fond sombre PureRef (0x000D0E12)
        fb.clear(0x000D0E12);

        // 2. Grille de points infinie
        self.draw_grid(fb, &vp);

        // 3. Membranes
        self.draw_membranes(fb, store, &vp);

        // 4. Images
        self.draw_images(fb, store, &vp);

        // 5. Annotations (stickies, textes, flèches)
        self.draw_annotations(fb, store, &vp);

        // 6. Guides d'alignement SNAP-1
        self.draw_guides(fb, guides, &vp);

        // 7. Boîte de sélection élastique (Marquee)
        if let Some((x1, y1, x2, y2)) = selection_box {
            self.draw_selection_box(fb, x1, y1, x2, y2);
        }

        // 8. HUD PureRef (Overlay statut et raccourcis)
        self.draw_hud(fb, store, &vp, always_on_top);
    }

    fn draw_grid(&self, fb: &mut FrameBuffer, vp: &Viewport) {
        let (min_wx, min_wy) = screen_to_world(0.0, 0.0, vp);
        let (max_wx, max_wy) = screen_to_world(fb.width as f64, fb.height as f64, vp);

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

        let dot_color = 0x00262B35;

        let mut gx = start_x;
        while gx <= end_x {
            let mut gy = start_y;
            while gy <= end_y {
                let (sx, sy) = world_to_screen(gx, gy, vp);
                if sx >= 0.0 && (sx as usize) < fb.width && sy >= 0.0 && (sy as usize) < fb.height {
                    fb.draw_dot(sx as i32, sy as i32, 1, dot_color);
                }
                gy += grid_step;
            }
            gx += grid_step;
        }
    }

    fn draw_membranes(&self, fb: &mut FrameBuffer, store: &Store, vp: &Viewport) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };

        for ann in &board.annotations {
            if let Annotation::Membrane { id, x, y, width, height, text, .. } = ann {
                let (sx, sy) = world_to_screen(*x, *y, vp);
                let sw = (*width * vp.scale).max(4.0);
                let sh = (*height * vp.scale).max(4.0);

                let x0 = sx as i32;
                let y0 = sy as i32;
                let x1 = (sx + sw) as i32;
                let y1 = (sy + sh) as i32;

                // Fond translucide bleu
                fb.fill_rect(x0, y0, x1, y1, 0x0060A5FA, 22);

                // Bordure
                let is_selected = store.selected_annotation_ids.contains(id);
                let stroke_color = if is_selected { 0x0038BDF8 } else { 0x0060A5FA };
                let alpha = if is_selected { 255 } else { 120 };
                fb.stroke_dashed_rect(x0, y0, x1, y1, stroke_color, 6, alpha);

                // Libellé de la membrane
                if let Some(lbl) = text {
                    fb.draw_text(lbl, x0 + 8, y0 + 8, 1, 0x0093C5FD);
                }
            }
        }
    }

    fn draw_images(&mut self, fb: &mut FrameBuffer, store: &Store, vp: &Viewport) {
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

            let dst_x = sx as i32;
            let dst_y = sy as i32;
            let dst_w = sw as i32;
            let dst_h = sh as i32;

            let mut drawn = false;
            if !src.is_empty() {
                if let Some(loaded_img) = self.get_or_load_image(&src) {
                    fb.blit_image(
                        &loaded_img.pixels,
                        loaded_img.width,
                        loaded_img.height,
                        dst_x,
                        dst_y,
                        dst_w,
                        dst_h,
                    );
                    drawn = true;
                }
            }

            if !drawn {
                // Placeholder pour image non trouvée sur disque
                fb.fill_rect(dst_x, dst_y, dst_x + dst_w, dst_y + dst_h, 0x001E232D, 255);
                fb.stroke_rect(dst_x, dst_y, dst_x + dst_w, dst_y + dst_h, 0x003C4655, 1, 255);

                let label = format!("Image [{}]", id);
                fb.draw_text(&label, dst_x + 8, dst_y + (dst_h / 2) - 4, 1, 0x008C96A5);
            }

            // Outline de sélection cyan
            if is_selected {
                fb.stroke_rect(
                    dst_x - 2,
                    dst_y - 2,
                    dst_x + dst_w + 2,
                    dst_y + dst_h + 2,
                    0x0038BDF8,
                    2,
                    255,
                );
            }
        }
    }

    fn draw_annotations(&self, fb: &mut FrameBuffer, store: &Store, vp: &Viewport) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };

        for ann in &board.annotations {
            match ann {
                Annotation::Sticky { id, x, y, width, height, text, operator, .. } => {
                    let (sx, sy) = world_to_screen(*x, *y, vp);
                    let sw = (width.unwrap_or(160.0) * vp.scale).max(8.0) as i32;
                    let sh = (height.unwrap_or(120.0) * vp.scale).max(8.0) as i32;

                    let x0 = sx as i32;
                    let y0 = sy as i32;
                    let x1 = x0 + sw;
                    let y1 = y0 + sh;

                    // Fond jaune sticky
                    fb.fill_rect(x0, y0, x1, y1, 0x00FEF08A, 235);

                    let is_selected = store.selected_annotation_ids.contains(id);
                    let stroke_color = if is_selected { 0x0038BDF8 } else { 0x00CA8A04 };
                    let stroke_width = if is_selected { 2 } else { 1 };
                    fb.stroke_rect(x0, y0, x1, y1, stroke_color, stroke_width, 255);

                    if let Some(op) = operator {
                        let op_str = format!("{:?}", op);
                        fb.draw_text(&op_str, x0 + 8, y0 + 6, 1, 0x00A16207);
                    }

                    fb.draw_text(text, x0 + 8, y0 + 22, 1, 0x001C1917);
                }
                Annotation::Text { id, x, y, text, .. } => {
                    let (sx, sy) = world_to_screen(*x, *y, vp);
                    let is_selected = store.selected_annotation_ids.contains(id);
                    let color = if is_selected { 0x0038BDF8 } else { 0x00F1F5F9 };

                    fb.draw_text(text, sx as i32, sy as i32, 1, color);
                }
                Annotation::Arrow { id, x, y, x2, y2, .. } => {
                    let (sx1, sy1) = world_to_screen(*x, *y, vp);
                    let (sx2, sy2) = world_to_screen(*x2, *y2, vp);

                    let is_selected = store.selected_annotation_ids.contains(id);
                    let color = if is_selected { 0x0038BDF8 } else { 0x0094A3B8 };
                    let width = if is_selected { 3 } else { 2 };

                    fb.draw_arrow(sx1 as i32, sy1 as i32, sx2 as i32, sy2 as i32, color, width, 12.0);
                }
                _ => {}
            }
        }
    }

    fn draw_guides(&self, fb: &mut FrameBuffer, guides: &SnapGuides, vp: &Viewport) {
        let guide_color = 0x00EC4899; // Magenta SNAP-1

        if let Some(ref xs) = guides.x {
            for &gx in xs {
                let (sx, _) = world_to_screen(gx, 0.0, vp);
                fb.draw_line(sx as i32, 0, sx as i32, fb.height as i32, guide_color, 1);
            }
        }

        if let Some(ref ys) = guides.y {
            for &gy in ys {
                let (_, sy) = world_to_screen(0.0, gy, vp);
                fb.draw_line(0, sy as i32, fb.width as i32, sy as i32, guide_color, 1);
            }
        }
    }

    fn draw_selection_box(&self, fb: &mut FrameBuffer, x1: f64, y1: f64, x2: f64, y2: f64) {
        let left = x1.min(x2) as i32;
        let top = y1.min(y2) as i32;
        let right = x1.max(x2) as i32;
        let bottom = y1.max(y2) as i32;

        fb.fill_rect(left, top, right, bottom, 0x0038BDF8, 30);
        fb.stroke_dashed_rect(left, top, right, bottom, 0x0038BDF8, 4, 200);
    }

    fn draw_hud(&self, fb: &mut FrameBuffer, store: &Store, vp: &Viewport, always_on_top: bool) {
        let board_name = store.active_board().map(|b| b.name.as_str()).unwrap_or("Main");
        let img_count = store.active_board().map(|b| b.images.len()).unwrap_or(0);
        let ann_count = store.active_board().map(|b| b.annotations.len()).unwrap_or(0);

        // Status gauche
        let info_text = format!(
            "GLUCOSE | Board: {} | {} images | {} notes",
            board_name, img_count, ann_count
        );
        let h = fb.height as i32;
        let w = fb.width as i32;

        fb.draw_text(&info_text, 12, h - 20, 1, 0x0094A3B8);

        // Zoom & statut droite
        let zoom_pct = (vp.scale * 100.0).round() as i32;
        let top_indicator = if always_on_top { "[ON TOP]" } else { "" };
        let right_text = format!("{}  Zoom: {}%", top_indicator, zoom_pct);
        let right_x = w - (right_text.len() as i32 * 8) - 12;
        fb.draw_text(&right_text, right_x, h - 20, 1, 0x0094A3B8);

        // Aide raccourcis haut
        let help_text = "Clic-Glisser: Selection/Deplacement | Clic-Droit/Alt: Pan | Molette: Zoom | T: Always on Top | Del: Suppr";
        fb.draw_text(help_text, 12, 12, 1, 0x0064748B);
    }
}
