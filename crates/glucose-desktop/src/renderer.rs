//! Moteur de rendu 2D haute fidélité pour Glucose Desktop (PureRef-style).
//! Utilise tiny-skia pour le rendu vectoriel anti-aliasé et fontdue pour la typographie.

use crate::canvas::{screen_to_world, world_to_screen};
use crate::typography::Typography;
use crate::ui::{render_ui, UiState, TOTAL_HEADER_HEIGHT};
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tiny_skia::{
    Color, FilterQuality, GradientStop, LineCap, Paint, PathBuilder, Pixmap, PixmapMut,
    PixmapPaint, Point, RadialGradient, Rect, SpreadMode, Stroke, Transform,
};

#[derive(Debug, Clone)]
pub struct TextEditSession {
    pub ann_id: String,
    pub buffer: String,
    pub cursor_idx: usize,
    pub blink_timer: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct CachedHue {
    pub x: f64,
    pub y: f64,
    pub hue: f64,
    pub rgb: (u8, u8, u8),
}

pub struct SymbioticHueCache {
    entries: HashMap<String, CachedHue>,
    last_positions: HashMap<String, (f64, f64)>,
}

impl SymbioticHueCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            last_positions: HashMap::new(),
        }
    }

    /// Invalidation par voisinage (1 200 px) : si une carte a bougé, seules les cartes à < 1 200 px sont invalidées
    pub fn update_positions_and_invalidate(&mut self, annotations: &[Annotation]) {
        let mut moved_points: Vec<(f64, f64)> = Vec::new();
        let mut current_ids = HashSet::new();

        for ann in annotations {
            if let Annotation::Text { id, x, y, .. } = ann {
                current_ids.insert(id.clone());
                match self.last_positions.get(id) {
                    Some(&(lx, ly)) => {
                        if (lx - x).abs() > 0.01 || (ly - y).abs() > 0.01 {
                            moved_points.push((*x, *y));
                            moved_points.push((lx, ly));
                        }
                    }
                    None => {
                        moved_points.push((*x, *y));
                    }
                }
            }
        }

        // Détecter les cartes supprimées
        for (old_id, &(lx, ly)) in &self.last_positions {
            if !current_ids.contains(old_id) {
                moved_points.push((lx, ly));
            }
        }

        // Si des cartes ont bougé / sont nées / sont mortes, invalider celles dans un rayon de 1 200 px
        if !moved_points.is_empty() {
            const INVALIDATION_RADIUS_SQ: f64 = 1200.0 * 1200.0;
            self.entries.retain(|id, entry| {
                if !current_ids.contains(id) {
                    return false;
                }
                for &(mx, my) in &moved_points {
                    let dx = entry.x - mx;
                    let dy = entry.y - my;
                    if dx * dx + dy * dy <= INVALIDATION_RADIUS_SQ {
                        return false;
                    }
                }
                true
            });
        }

        // Mettre à jour la table des positions
        self.last_positions.clear();
        for ann in annotations {
            if let Annotation::Text { id, x, y, .. } = ann {
                self.last_positions.insert(id.clone(), (*x, *y));
            }
        }
    }

    pub fn get_or_compute(&mut self, ann: &Annotation, all_annotations: &[Annotation]) -> (f64, (u8, u8, u8)) {
        let id = ann.id();
        let ax = ann.x();
        let ay = ann.y();

        if let Some(entry) = self.entries.get(id) {
            return (entry.hue, entry.rgb);
        }

        let hue = glucose_core::symbiotic_hue::get_symbiotic_hue(ann, all_annotations);
        let rgb = glucose_core::symbiotic_hue::hsl_to_rgb(hue, 0.75, 0.65);
        self.entries.insert(id.to_string(), CachedHue {
            x: ax,
            y: ay,
            hue,
            rgb,
        });
        (hue, rgb)
    }
}

/// Décode une couleur hexadécimale #RRGGBB ou #RGB
fn parse_hex_color(hex: &str, default_r: u8, default_g: u8, default_b: u8) -> (u8, u8, u8) {
    let s = hex.trim_start_matches('#');
    if s.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            return (r, g, b);
        }
    } else if s.len() == 3 {
        let r = u8::from_str_radix(&s[0..1], 16).map(|v| v * 17).unwrap_or(default_r);
        let g = u8::from_str_radix(&s[1..2], 16).map(|v| v * 17).unwrap_or(default_g);
        let b = u8::from_str_radix(&s[2..3], 16).map(|v| v * 17).unwrap_or(default_b);
        return (r, g, b);
    }
    (default_r, default_g, default_b)
}

/// Ajoute un rectangle à coins arrondis dans un PathBuilder
fn push_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.min(w / 2.0).min(h / 2.0);
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
}

pub struct Renderer {
    pub image_cache: HashMap<String, Pixmap>,
    pub typography: Typography,
    pub hue_cache: SymbioticHueCache,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            image_cache: HashMap::new(),
            typography: Typography::new(),
            hue_cache: SymbioticHueCache::new(),
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
        editing_session: Option<&TextEditSession>,
        mouse_x: f32,
        mouse_y: f32,
    ) {
        let width = pixmap.width();
        let height = pixmap.height();
        let vp = store
            .active_board()
            .map(|b| b.viewport)
            .unwrap_or(Viewport::default());

        if let Some(board) = store.active_board() {
            self.hue_cache.update_positions_and_invalidate(&board.annotations);
        }

        // 1. Fond sombre sleek PureRef #0D0E12
        pixmap.fill(Color::from_rgba8(13, 14, 18, 255));

        // 2. Grille de points infinie
        self.draw_grid(pixmap, &vp, width, height);

        // 3. Halos symbiotiques d'ambiance (Biome 2D + gradient vectoriel circulaire)
        self.draw_halos(pixmap, store, &vp);

        // 4. Membranes (large rayon rx=60, pointillés, titre protecteur en haut à gauche)
        self.draw_membranes(pixmap, store, &vp);

        // 5. Images
        self.draw_images(pixmap, store, &vp);

        // 6. Annotations (cartes de texte, stickies, flèches + édition live in-place)
        self.draw_annotations(pixmap, store, &vp, editing_session);

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

        // Pas dynamique adaptatif : ne descend jamais sous ~32px à l'écran pour éviter toute explosion CPU
        let mut effective_step = 60.0f64;
        while effective_step * vp.scale < 32.0 {
            effective_step *= 2.0;
        }

        let start_x = (min_wx / effective_step).floor() * effective_step;
        let end_x = (max_wx / effective_step).ceil() * effective_step;
        let start_y = (min_wy / effective_step).floor() * effective_step;
        let end_y = (max_wy / effective_step).ceil() * effective_step;

        let dot_paint = {
            let mut p = Paint::default();
            p.set_color(Color::from_rgba8(255, 255, 255, 22));
            p.anti_alias = true;
            p
        };

        let mut pb = PathBuilder::new();
        let mut gx = start_x;
        while gx <= end_x {
            let mut gy = start_y;
            while gy <= end_y {
                let (sx, sy) = world_to_screen(gx, gy, vp);
                if sy >= TOTAL_HEADER_HEIGHT as f64 && sx >= 0.0 && sx <= w as f64 && sy <= h as f64 {
                    pb.push_circle(sx as f32, sy as f32, 1.2);
                }
                gy += effective_step;
            }
            gx += effective_step;
        }
        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &dot_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }

    fn draw_halos(&mut self, pixmap: &mut PixmapMut, store: &Store, vp: &Viewport) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };

        let screen_w = pixmap.width() as f32;
        let screen_h = pixmap.height() as f32;

        for ann in &board.annotations {
            if let Annotation::Text { x, y, width, height, .. } = ann {
                let (sx, sy) = world_to_screen(*x, *y, vp);
                let w = width.unwrap_or(240.0) * vp.scale;
                let h = height.unwrap_or(48.0) * vp.scale;

                let cx = (sx + w / 2.0) as f32;
                let cy = (sy + h / 2.0) as f32;
                let radius = ((w.max(h) * 1.5) as f32 + 50.0 * vp.scale as f32).max(20.0 * vp.scale as f32);

                // Frustum culling : ignorer si complètement hors de l'écran visible ou trop microscopique
                if cx + radius < 0.0
                    || cx - radius > screen_w
                    || cy + radius < TOTAL_HEADER_HEIGHT
                    || cy - radius > screen_h
                    || radius < 4.0
                {
                    continue;
                }

                let (_hue, (r, g, b)) = self.hue_cache.get_or_compute(ann, &board.annotations);

                if let Some(shader) = RadialGradient::new(
                    Point::from_xy(cx, cy),
                    Point::from_xy(cx, cy),
                    radius,
                    vec![
                        GradientStop::new(0.0, Color::from_rgba8(r, g, b, 35)),
                        GradientStop::new(1.0, Color::from_rgba8(r, g, b, 0)),
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

    fn draw_membranes(&self, pixmap: &mut PixmapMut, store: &Store, vp: &Viewport) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };

        let screen_w = pixmap.width() as f32;
        let screen_h = pixmap.height() as f32;

        for ann in &board.annotations {
            if let Annotation::Membrane { id, x, y, width, height, text, color, .. } = ann {
                let (sx, sy) = world_to_screen(*x, *y, vp);
                let sw = (*width * vp.scale) as f32;
                let sh = (*height * vp.scale) as f32;

                // Frustum culling
                if sx as f32 + sw < 0.0
                    || sx as f32 > screen_w
                    || sy as f32 + sh < TOTAL_HEADER_HEIGHT
                    || sy as f32 > screen_h
                    || (sw < 2.0 && sh < 2.0)
                {
                    continue;
                }

                let (r, g, b) = color.as_deref()
                    .map(|c| parse_hex_color(c, 96, 165, 250))
                    .unwrap_or((96, 165, 250));

                let is_selected = store.selected_annotation_ids.contains(id);
                let rx = (60.0 * vp.scale as f32).clamp(4.0, 60.0).min(sw / 2.0).min(sh / 2.0);

                // 1. Glow ultra-discret multicouche
                if sw > 8.0 && sh > 8.0 {
                    for (pad, alpha) in [(20.0 * vp.scale as f32, 8u8), (10.0 * vp.scale as f32, 14u8)] {
                        let mut glow_pb = PathBuilder::new();
                        push_rounded_rect(
                            &mut glow_pb,
                            sx as f32 - pad,
                            sy as f32 - pad,
                            sw + pad * 2.0,
                            sh + pad * 2.0,
                            rx + pad * 0.5,
                        );
                        if let Some(path) = glow_pb.finish() {
                            let mut gp = Paint::default();
                            gp.set_color(Color::from_rgba8(r, g, b, alpha));
                            gp.anti_alias = true;
                            pixmap.fill_path(&path, &gp, tiny_skia::FillRule::Winding, Transform::identity(), None);
                        }
                    }
                }

                // 2. Fill translucide 2%
                let mut fill_pb = PathBuilder::new();
                push_rounded_rect(&mut fill_pb, sx as f32, sy as f32, sw, sh, rx);
                if let Some(fill_path) = fill_pb.finish() {
                    let mut fill_paint = Paint::default();
                    fill_paint.set_color(Color::from_rgba8(r, g, b, 8));
                    fill_paint.anti_alias = true;
                    pixmap.fill_path(&fill_path, &fill_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);

                    // 3. Bordure nette pointillée 10 10 (ou pleine si sélectionné)
                    let mut stroke_paint = Paint::default();
                    stroke_paint.set_color(if is_selected {
                        Color::from_rgba8(r, g, b, 235)
                    } else {
                        Color::from_rgba8(r, g, b, 115)
                    });
                    stroke_paint.anti_alias = true;

                    let stroke = Stroke {
                        width: if is_selected { 2.2 } else { 2.0 },
                        dash: if is_selected {
                            None
                        } else {
                            tiny_skia::StrokeDash::new(vec![10.0, 10.0], 0.0)
                        },
                        line_cap: LineCap::Round,
                        ..Default::default()
                    };
                    pixmap.stroke_path(&fill_path, &stroke_paint, &stroke, Transform::identity(), None);
                }

                // 4. Label de section (titre) si visible
                if sw > 24.0 && sh > 20.0 {
                    if let Some(lbl) = text {
                        if !lbl.is_empty() {
                            let lx = sx as f32 + (16.0 * vp.scale as f32).clamp(8.0, 20.0);
                            let ly = sy as f32 - (18.0 * vp.scale as f32).clamp(12.0, 24.0);
                            let font_size = (16.0 * vp.scale).clamp(12.0, 22.0) as f32;

                            if font_size >= 10.0 {
                                let shadow_offsets = [
                                    (-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0),
                                    (-1.5, -1.5), (1.5, -1.5), (-1.5, 1.5), (1.5, 1.5),
                                ];
                                let shadow_color = Color::from_rgba8(11, 11, 18, 255);
                                for (ox, oy) in shadow_offsets {
                                    self.typography.draw_text(
                                        pixmap,
                                        lbl,
                                        lx + ox,
                                        ly + oy,
                                        font_size,
                                        shadow_color,
                                        true,
                                    );
                                }

                                self.typography.draw_text(
                                    pixmap,
                                    lbl,
                                    lx,
                                    ly,
                                    font_size,
                                    Color::from_rgba8(r, g, b, 255),
                                    true,
                                );
                            }
                        }
                    }
                }

                // 5. Poignées de redimensionnement aux 4 coins si sélectionné
                if is_selected && sw > 16.0 && sh > 16.0 {
                    let handle_size = 7.0f32;
                    let corners = [
                        (sx as f32, sy as f32),
                        (sx as f32 + sw, sy as f32),
                        (sx as f32, sy as f32 + sh),
                        (sx as f32 + sw, sy as f32 + sh),
                    ];
                    let mut hp = Paint::default();
                    hp.set_color(Color::from_rgba8(26, 26, 26, 255));
                    let mut h_stroke_p = Paint::default();
                    h_stroke_p.set_color(Color::from_rgba8(r, g, b, 255));
                    let h_stroke = Stroke { width: 1.0, ..Default::default() };

                    for (cx, cy) in corners {
                        if let Some(hr) = Rect::from_xywh(cx - handle_size / 2.0, cy - handle_size / 2.0, handle_size, handle_size) {
                            pixmap.fill_rect(hr, &hp, Transform::identity(), None);
                            let h_path = PathBuilder::from_rect(hr);
                            pixmap.stroke_path(&h_path, &h_stroke_p, &h_stroke, Transform::identity(), None);
                        }
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

        let screen_w = pixmap.width() as f32;
        let screen_h = pixmap.height() as f32;

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
            let sw = (w * vp.scale) as f32;
            let sh = (h * vp.scale) as f32;

            // Frustum culling
            if sx as f32 + sw < 0.0
                || sx as f32 > screen_w
                || sy as f32 + sh < TOTAL_HEADER_HEIGHT
                || sy as f32 > screen_h
                || (sw < 1.0 && sh < 1.0)
            {
                continue;
            }

            let is_selected = store.selected_image_ids.contains(&id);

            let mut drawn = false;
            if !src.is_empty() {
                if let Some(loaded_pixmap) = self.get_or_load_image(&src) {
                    let scale_x = sw / loaded_pixmap.width() as f32;
                    let scale_y = sh / loaded_pixmap.height() as f32;
                    let ts = Transform::from_scale(scale_x, scale_y).post_translate(sx as f32, sy as f32);
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

    fn draw_annotations(
        &mut self,
        pixmap: &mut PixmapMut,
        store: &Store,
        vp: &Viewport,
        editing_session: Option<&TextEditSession>,
    ) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };
        let screen_w = pixmap.width() as f32;
        let screen_h = pixmap.height() as f32;

        for ann in &board.annotations {
            match ann {
                Annotation::Text { id, x, y, width, height, text, color, .. } => {
                    let (sx, sy) = world_to_screen(*x, *y, vp);
                    let sw = (width.unwrap_or(240.0) * vp.scale) as f32;
                    let sh = (height.unwrap_or(48.0) * vp.scale) as f32;

                    // Frustum culling
                    if sx as f32 + sw < 0.0
                        || sx as f32 > screen_w
                        || sy as f32 + sh < TOTAL_HEADER_HEIGHT
                        || sy as f32 > screen_h
                        || (sw < 3.0 && sh < 3.0)
                    {
                        continue;
                    }

                    let is_selected = store.selected_annotation_ids.contains(id);
                    let is_editing = editing_session.map(|s| s.ann_id == *id).unwrap_or(false);

                    let (_hue, (hr, hg, hb)) = self.hue_cache.get_or_compute(ann, &board.annotations);
                    let (r, g, b) = color.as_deref()
                        .map(|c| parse_hex_color(c, hr, hg, hb))
                        .unwrap_or((hr, hg, hb));

                    let content = if is_editing {
                        editing_session.unwrap().buffer.as_str()
                    } else {
                        text.as_str()
                    };

                    // Formatage du texte et calcul de la hauteur nécessaire
                    let font_size = (14.0 * vp.scale).clamp(8.0, 24.0) as f32;
                    let line_height = font_size * 1.35;
                    let pad_x = (18.0 * vp.scale as f32).clamp(4.0, 24.0);
                    let pad_y = (12.0 * vp.scale as f32).clamp(4.0, 16.0);

                    let lines: Vec<&str> = if content.is_empty() {
                        vec![""]
                    } else {
                        content.lines().collect()
                    };

                    let content_h = pad_y * 2.0 + (lines.len().max(1) as f32) * line_height;
                    let actual_sh = sh.max(content_h);

                    let rx = (24.0 * vp.scale as f32).clamp(4.0, 28.0).min(actual_sh / 2.0);

                    // 1. Aura douce d'ambiance (#18181B teinté avec 12% de la teinte symbiotique)
                    let mut pb = PathBuilder::new();
                    push_rounded_rect(&mut pb, sx as f32, sy as f32, sw, actual_sh, rx);

                    if let Some(path) = pb.finish() {
                        let mut fill = Paint::default();
                        let bg_r = ((r as u16 * 12 + 24 * 88) / 100) as u8;
                        let bg_g = ((g as u16 * 12 + 24 * 88) / 100) as u8;
                        let bg_b = ((b as u16 * 12 + 27 * 88) / 100) as u8;
                        fill.set_color(Color::from_rgba8(bg_r, bg_g, bg_b, 248));
                        fill.anti_alias = true;
                        pixmap.fill_path(&path, &fill, tiny_skia::FillRule::Winding, Transform::identity(), None);

                        // Bordure subtile ou brillante
                        let mut stroke_paint = Paint::default();
                        stroke_paint.anti_alias = true;
                        if is_editing {
                            stroke_paint.set_color(Color::from_rgba8(56, 189, 248, 255));
                        } else if is_selected {
                            stroke_paint.set_color(Color::from_rgba8(56, 189, 248, 220));
                        } else {
                            stroke_paint.set_color(Color::from_rgba8(r, g, b, 60));
                        }
                        let stroke = Stroke {
                            width: if is_editing || is_selected { 2.0 } else { 1.0 },
                            ..Default::default()
                        };
                        pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
                    }

                    // Ne rasteriser le texte que s'il est suffisamment grand pour être lisible
                    if font_size >= 9.0 && sw > 16.0 && actual_sh > 12.0 {
                        let mut cur_y = sy as f32 + pad_y;
                        let mut cursor_drawn = false;
                        let mut char_count_acc = 0;

                        let show_cursor = is_editing
                            && (editing_session.unwrap().blink_timer.elapsed().as_millis() / 500) % 2 == 0;
                        let cursor_idx = if is_editing {
                            editing_session.unwrap().cursor_idx
                        } else {
                            0
                        };

                        for (line_num, line) in lines.iter().enumerate() {
                            let line_len = line.len();
                            let line_start = char_count_acc;
                            let line_end = line_start + line_len;

                            let (display_text, is_bold, f_size, f_color, indent) = if line.starts_with("# ") {
                                (&line[2..], true, font_size * 1.25, Color::from_rgba8(255, 255, 255, 255), 0.0)
                            } else if line.starts_with("## ") {
                                (&line[3..], true, font_size * 1.1, Color::from_rgba8(240, 240, 245, 255), 0.0)
                            } else if line.starts_with("- ") || line.starts_with("* ") {
                                let bullet_x = sx as f32 + pad_x;
                                let bullet_y = cur_y + font_size * 0.45;
                                let mut b_paint = Paint::default();
                                b_paint.set_color(Color::from_rgba8(r, g, b, 200));
                                b_paint.anti_alias = true;
                                let mut b_pb = PathBuilder::new();
                                b_pb.push_circle(bullet_x + 3.0, bullet_y, 2.2);
                                if let Some(p) = b_pb.finish() {
                                    pixmap.fill_path(&p, &b_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                                }
                                (&line[2..], false, font_size, Color::from_rgba8(220, 225, 235, 255), 14.0)
                            } else {
                                (*line, false, font_size, Color::from_rgba8(220, 225, 235, 255), 0.0)
                            };

                            let start_x = sx as f32 + pad_x + indent;

                            self.typography.draw_text(
                                pixmap,
                                display_text,
                                start_x,
                                cur_y,
                                f_size,
                                f_color,
                                is_bold,
                            );

                            if show_cursor && !cursor_drawn && cursor_idx >= line_start && (cursor_idx <= line_end || line_num == lines.len() - 1) {
                                let prefix_len = cursor_idx.saturating_sub(line_start).min(line_len);
                                let prefix = &line[..prefix_len];
                                let (prefix_w, _) = self.typography.measure_text(prefix, f_size, is_bold);

                                let cx = start_x + prefix_w;
                                let cy = cur_y;
                                let ch = f_size * 1.2;

                                let mut c_paint = Paint::default();
                                c_paint.set_color(Color::from_rgba8(56, 189, 248, 255));
                                if let Some(cr) = Rect::from_xywh(cx, cy, 2.0, ch) {
                                    pixmap.fill_rect(cr, &c_paint, Transform::identity(), None);
                                }
                                cursor_drawn = true;
                            }

                            char_count_acc += line_len + 1;
                            cur_y += line_height;
                        }

                        if show_cursor && !cursor_drawn {
                            let mut c_paint = Paint::default();
                            c_paint.set_color(Color::from_rgba8(56, 189, 248, 255));
                            if let Some(cr) = Rect::from_xywh(sx as f32 + pad_x, sy as f32 + pad_y, 2.0, font_size * 1.2) {
                                pixmap.fill_rect(cr, &c_paint, Transform::identity(), None);
                            }
                        }
                    }
                }
                Annotation::Sticky { id, x, y, width, height, text, color, bg_color, operator, .. } => {
                    let (sx, sy) = world_to_screen(*x, *y, vp);
                    let sw = (width.unwrap_or(160.0) * vp.scale) as f32;
                    let sh = (height.unwrap_or(120.0) * vp.scale) as f32;

                    // Frustum culling
                    if sx as f32 + sw < 0.0
                        || sx as f32 > screen_w
                        || sy as f32 + sh < TOTAL_HEADER_HEIGHT
                        || sy as f32 > screen_h
                        || (sw < 3.0 && sh < 3.0)
                    {
                        continue;
                    }

                    let is_selected = store.selected_annotation_ids.contains(id);
                    let is_editing = editing_session.map(|s| s.ann_id == *id).unwrap_or(false);

                    let content = if is_editing {
                        editing_session.unwrap().buffer.as_str()
                    } else {
                        text.as_str()
                    };

                    let mut pb = PathBuilder::new();
                    push_rounded_rect(&mut pb, sx as f32, sy as f32, sw, sh, (6.0 * vp.scale as f32).clamp(2.0, 6.0));

                    if let Some(path) = pb.finish() {
                        let (bg_r, bg_g, bg_b) = bg_color.as_deref()
                            .map(|c| parse_hex_color(c, 254, 240, 138))
                            .unwrap_or((254, 240, 138));
                        let mut p = Paint::default();
                        p.set_color(Color::from_rgba8(bg_r, bg_g, bg_b, 245));
                        p.anti_alias = true;
                        pixmap.fill_path(&path, &p, tiny_skia::FillRule::Winding, Transform::identity(), None);

                        let mut sp = Paint::default();
                        sp.anti_alias = true;
                        sp.set_color(if is_editing || is_selected {
                            Color::from_rgba8(56, 189, 248, 255)
                        } else {
                            Color::from_rgba8(202, 138, 4, 180)
                        });
                        let stroke = Stroke {
                            width: if is_editing || is_selected { 2.0 } else { 1.0 },
                            ..Default::default()
                        };
                        pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);

                        let f_size = (12.0 * vp.scale).clamp(8.0, 20.0) as f32;
                        let line_h = f_size * 1.3;

                        if f_size >= 9.0 && sw > 16.0 && sh > 12.0 {
                            let mut cur_ty = sy as f32 + (10.0 * vp.scale as f32).clamp(4.0, 10.0);

                            if let Some(op) = operator {
                                let op_str = match op {
                                    glucose_core::types::StickyOperator::And => "ET",
                                    glucose_core::types::StickyOperator::Or => "OU",
                                    glucose_core::types::StickyOperator::But => "MAIS",
                                    glucose_core::types::StickyOperator::Because => "PARCE QUE",
                                };
                                self.typography.draw_text(
                                    pixmap,
                                    op_str,
                                    sx as f32 + 10.0,
                                    cur_ty,
                                    11.0,
                                    Color::from_rgba8(161, 98, 7, 255),
                                    true,
                                );
                                cur_ty += 16.0;
                            }

                            let (txt_r, txt_g, txt_b) = color.as_deref()
                                .map(|c| parse_hex_color(c, 28, 25, 23))
                                .unwrap_or((28, 25, 23));
                            let text_color = Color::from_rgba8(txt_r, txt_g, txt_b, 255);

                            for line in content.lines() {
                                self.typography.draw_text(
                                    pixmap,
                                    line,
                                    sx as f32 + 10.0,
                                    cur_ty,
                                    f_size,
                                    text_color,
                                    false,
                                );
                                cur_ty += line_h;
                            }

                            if is_editing && (editing_session.unwrap().blink_timer.elapsed().as_millis() / 500) % 2 == 0 {
                                let (cw, _) = self.typography.measure_text(content, f_size, false);
                                let cx = (sx as f32 + 10.0 + cw).min(sx as f32 + sw - 6.0);
                                let mut c_paint = Paint::default();
                                c_paint.set_color(Color::from_rgba8(28, 25, 23, 255));
                                if let Some(cr) = Rect::from_xywh(cx, cur_ty - line_h, 2.0, f_size * 1.2) {
                                    pixmap.fill_rect(cr, &c_paint, Transform::identity(), None);
                                }
                            }
                        }
                    }
                }
                Annotation::Arrow { id, x, y, x2, y2, .. } => {
                    let (sx1, sy1) = world_to_screen(*x, *y, vp);
                    let (sx2, sy2) = world_to_screen(*x2, *y2, vp);

                    let min_x = (sx1.min(sx2) as f32) - 16.0;
                    let max_x = (sx1.max(sx2) as f32) + 16.0;
                    let min_y = (sy1.min(sy2) as f32) - 16.0;
                    let max_y = (sy1.max(sy2) as f32) + 16.0;

                    if max_x < 0.0 || min_x > screen_w || max_y < TOTAL_HEADER_HEIGHT || min_y > screen_h {
                        continue;
                    }

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
                    let arrow_len = (12.0f32 * vp.scale as f32).clamp(6.0, 16.0);
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
