//! Moteur de rendu 2D haute fidélité pour Glucose Desktop (PureRef-style).
//! Utilise tiny-skia pour le rendu vectoriel anti-aliasé et fontdue pour la typographie.

use crate::canvas::{screen_to_world, world_to_screen};
use crate::theme::Theme;
use crate::typography::Typography;
use crate::ui::{render_ui, UiState};
use glucose_core::quadtree::SpatialHash;
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
        let mut current_ids: HashSet<&str> = HashSet::with_capacity(annotations.len());

        for ann in annotations {
            if let Annotation::Text { id, x, y, .. } = ann {
                current_ids.insert(id.as_str());
                match self.last_positions.get(id.as_str()) {
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

        // Détecter les cartes supprimées et les retirer de last_positions sans réallocation globale
        self.last_positions.retain(|old_id, pos| {
            let (lx, ly) = *pos;
            if !current_ids.contains(old_id.as_str()) {
                moved_points.push((lx, ly));
                false
            } else {
                true
            }
        });

        // Si des cartes ont bougé / sont nées / sont mortes, invalider celles dans un rayon de 1 200 px
        if !moved_points.is_empty() {
            const INVALIDATION_RADIUS_SQ: f64 = 1200.0 * 1200.0;
            self.entries.retain(|id, entry| {
                if !current_ids.contains(id.as_str()) {
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

        // Mettre à jour les coordonnées en place sans réallouer de chaînes pour les cartes existantes (R-39)
        for ann in annotations {
            if let Annotation::Text { id, x, y, .. } = ann {
                if let Some(pos) = self.last_positions.get_mut(id.as_str()) {
                    *pos = (*x, *y);
                } else {
                    self.last_positions.insert(id.clone(), (*x, *y));
                }
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

/// Échelle de viewport en dessous de laquelle la grille n'est plus dessinable.
const MIN_GRID_SCALE: f64 = 1e-6;
/// Nombre maximal de doublements du pas de grille (borne la boucle d'adaptation).
const MAX_GRID_DOUBLINGS: u32 = 64;

pub struct Renderer {
    pub theme: Theme,
    pub image_cache: HashMap<String, Pixmap>,
    pub failed_images: HashSet<String>,
    pub typography: Typography,
    pub hue_cache: SymbioticHueCache,
    pub spatial_hash: SpatialHash,
    pub spatial_version: u64,
    pub active_board_id: String,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            theme: Theme::dark(),
            image_cache: HashMap::new(),
            failed_images: HashSet::new(),
            typography: Typography::new(),
            hue_cache: SymbioticHueCache::new(),
            spatial_hash: SpatialHash::new(1000.0),
            spatial_version: 0,
            active_board_id: String::new(),
        }
    }

    /// Charge ou récupère une image décodée en Pixmap tiny-skia (supporte WebP, PNG, JPG, GIF, BMP).
    /// Dispose d'un cache négatif pour ne jamais re-décoder un fichier inaccessible ou corrompu (R-29).
    #[allow(dead_code)]
    pub fn get_or_load_image(&mut self, src_or_path: &str) -> Option<&Pixmap> {
        Self::load_image_impl(&mut self.image_cache, &mut self.failed_images, src_or_path)
    }

    pub fn load_image_impl<'a>(
        image_cache: &'a mut HashMap<String, Pixmap>,
        failed_images: &mut HashSet<String>,
        src_or_path: &str,
    ) -> Option<&'a Pixmap> {
        if failed_images.contains(src_or_path) {
            return None;
        }
        if image_cache.contains_key(src_or_path) {
            return image_cache.get(src_or_path);
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
                    image_cache.insert(src_or_path.to_string(), pixmap);
                    return image_cache.get(src_or_path);
                }
            }
        }
        // Cache négatif (R-29) : ne pas retenter le décodage échoué chaque frame
        failed_images.insert(src_or_path.to_string());
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
            if self.spatial_version != store.version || self.active_board_id != board.id {
                self.spatial_hash.index_board(board);
                self.spatial_version = store.version;
                self.active_board_id = board.id.clone();
            }
        }

        let header_h = ui.header_height();
        let (min_wx, min_wy) = screen_to_world(0.0, header_h as f64, &vp);
        let (max_wx, max_wy) = screen_to_world(width as f64, height as f64, &vp);
        let visible_ids = self.spatial_hash.query_rect_refs(min_wx, min_wy, max_wx, max_wy, 200.0);
        crate::perf::stage("cull");

        // 1. Fond sombre sleek PureRef
        pixmap.fill(self.theme.bg_canvas);
        crate::perf::stage("clear");

        // 2. Grille de points infinie
        self.draw_grid(pixmap, &vp, width, height, header_h);
        crate::perf::stage("grid");

        // 3. Halos symbiotiques d'ambiance (Biome 2D + gradient vectoriel circulaire)
        Self::draw_halos(&mut self.hue_cache, pixmap, store, &vp, &visible_ids, header_h);
        crate::perf::stage("halos");

        // 4. Membranes (large rayon rx=60, pointillés, titre protecteur en haut à gauche)
        self.draw_membranes(pixmap, store, &vp, &visible_ids, header_h);
        crate::perf::stage("membranes");

        // 5. Images
        Self::draw_images(
            &mut self.image_cache,
            &mut self.failed_images,
            &self.typography,
            pixmap,
            store,
            &vp,
            &visible_ids,
            header_h,
        );
        crate::perf::stage("images");

        // 6. Annotations (cartes de texte, stickies, flèches + édition live in-place)
        Self::draw_annotations(
            &mut self.hue_cache,
            &self.typography,
            pixmap,
            store,
            &vp,
            editing_session,
            &visible_ids,
            header_h,
        );
        crate::perf::stage("annotations");

        // 7. Guides d'alignement intelligents (SNAP-1)
        if ui.smart_align {
            self.draw_guides(pixmap, guides, &vp, width, height, header_h);
        }

        // 8. Boîte de sélection élastique (Marquee)
        if let Some((x1, y1, x2, y2)) = selection_box {
            self.draw_selection_box(pixmap, x1, y1, x2, y2);
        }

        // 9. Interface utilisateur complète (TopBar, Tabs, Minimap, Toasts)
        render_ui(pixmap, store, ui, &self.typography, &self.theme, mouse_x, mouse_y);
        crate::perf::stage("ui");
    }

    fn draw_grid(&self, pixmap: &mut PixmapMut, vp: &Viewport, w: u32, h: u32, header_h: f32) {
        // Une échelle nulle, négative ou NaN rendait la boucle d'adaptation du pas
        // infinie : on la borne inconditionnellement.
        let safe_scale = if vp.scale.is_finite() && vp.scale > MIN_GRID_SCALE {
            vp.scale
        } else {
            return;
        };

        let (min_wx, min_wy) = screen_to_world(0.0, header_h as f64, vp);
        let (max_wx, max_wy) = screen_to_world(w as f64, h as f64, vp);
        if ![min_wx, min_wy, max_wx, max_wy].iter().all(|v| v.is_finite()) {
            return;
        }

        // Pas dynamique adaptatif : ne descend jamais sous ~32px à l'écran pour éviter toute explosion CPU
        let mut effective_step = 60.0f64;
        let mut doublings = 0u32;
        while effective_step * safe_scale < 32.0 && doublings < MAX_GRID_DOUBLINGS {
            effective_step *= 2.0;
            doublings += 1;
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
                if sy >= header_h as f64 && sx >= 0.0 && sx <= w as f64 && sy <= h as f64 {
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

    fn draw_halos(
        hue_cache: &mut SymbioticHueCache,
        pixmap: &mut PixmapMut,
        store: &Store,
        vp: &Viewport,
        visible_ids: &HashSet<&str>,
        header_h: f32,
    ) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };

        let screen_w = pixmap.width() as f32;
        let screen_h = pixmap.height() as f32;

        for ann in &board.annotations {
            if !visible_ids.contains(ann.id()) {
                continue;
            }
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
                    || cy + radius < header_h
                    || cy - radius > screen_h
                    || radius < 4.0
                {
                    continue;
                }

                let (_hue, (r, g, b)) = hue_cache.get_or_compute(ann, &board.annotations);

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

    fn draw_membranes(&self, pixmap: &mut PixmapMut, store: &Store, vp: &Viewport, visible_ids: &HashSet<&str>, header_h: f32) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };

        let screen_w = pixmap.width() as f32;
        let screen_h = pixmap.height() as f32;

        for ann in &board.annotations {
            if !visible_ids.contains(ann.id()) {
                continue;
            }
            if let Annotation::Membrane { id, x, y, width, height, text, color, .. } = ann {
                let (sx, sy) = world_to_screen(*x, *y, vp);
                let sw = (*width * vp.scale) as f32;
                let sh = (*height * vp.scale) as f32;

                // Frustum culling
                if sx as f32 + sw < 0.0
                    || sx as f32 > screen_w
                    || sy as f32 + sh < header_h
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
                                let shadow_color = Color::from_rgba8(11, 11, 18, 255);
                                self.typography.draw_text_with_outline(
                                    pixmap,
                                    lbl,
                                    lx,
                                    ly,
                                    font_size,
                                    Color::from_rgba8(r, g, b, 255),
                                    shadow_color,
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

    fn draw_images(
        image_cache: &mut HashMap<String, Pixmap>,
        failed_images: &mut HashSet<String>,
        typography: &Typography,
        pixmap: &mut PixmapMut,
        store: &Store,
        vp: &Viewport,
        visible_ids: &HashSet<&str>,
        header_h: f32,
    ) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };

        let screen_w = pixmap.width() as f32;
        let screen_h = pixmap.height() as f32;

        for img in &board.images {
            if !visible_ids.contains(img.id.as_str()) {
                continue;
            }
            let (sx, sy) = world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, vp);
            let sw = (img.width * vp.scale) as f32;
            let sh = (img.height * vp.scale) as f32;

            // Frustum culling
            if sx as f32 + sw < 0.0
                || sx as f32 > screen_w
                || sy as f32 + sh < header_h
                || sy as f32 > screen_h
                || (sw < 1.0 && sh < 1.0)
            {
                continue;
            }

            let is_selected = store.selected_image_ids.contains(&img.id);

            let mut drawn = false;
            if let Some(ref src) = img.src {
                if !src.is_empty() {
                    if let Some(loaded_pixmap) = Self::load_image_impl(image_cache, failed_images, src) {
                        let scale_x = sw / loaded_pixmap.width() as f32;
                        let scale_y = sh / loaded_pixmap.height() as f32;
                        let ts = Transform::from_scale(scale_x, scale_y).post_translate(sx as f32, sy as f32);
                        let mut pp = PixmapPaint::default();
                        pp.quality = FilterQuality::Bilinear;
                        pixmap.draw_pixmap(0, 0, loaded_pixmap.as_ref(), &pp, ts, None);
                        drawn = true;
                    }
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

                    let label = format!("Image [{}]", img.id);
                    typography.draw_text(
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
        hue_cache: &mut SymbioticHueCache,
        typography: &Typography,
        pixmap: &mut PixmapMut,
        store: &Store,
        vp: &Viewport,
        editing_session: Option<&TextEditSession>,
        visible_ids: &HashSet<&str>,
        header_h: f32,
    ) {
        let board = match store.active_board() {
            Some(b) => b,
            None => return,
        };
        let screen_w = pixmap.width() as f32;
        let screen_h = pixmap.height() as f32;

        for ann in &board.annotations {
            if !visible_ids.contains(ann.id()) {
                continue;
            }
            match ann {
                Annotation::Text { id, x, y, width, height, text, color, .. } => {
                    let (sx, sy) = world_to_screen(*x, *y, vp);
                    let sw = (width.unwrap_or(240.0) * vp.scale) as f32;
                    let sh = (height.unwrap_or(48.0) * vp.scale) as f32;

                    // Frustum culling
                    if sx as f32 + sw < 0.0
                        || sx as f32 > screen_w
                        || sy as f32 + sh < header_h
                        || sy as f32 > screen_h
                        || (sw < 3.0 && sh < 3.0)
                    {
                        continue;
                    }

                    let is_selected = store.selected_annotation_ids.contains(id);
                    let active_edit = editing_session.filter(|s| s.ann_id == *id);
                    let is_editing = active_edit.is_some();

                    let (_hue, (hr, hg, hb)) = hue_cache.get_or_compute(ann, &board.annotations);
                    let (r, g, b) = color.as_deref()
                        .map(|c| parse_hex_color(c, hr, hg, hb))
                        .unwrap_or((hr, hg, hb));

                    let content = if let Some(edit) = active_edit {
                        edit.buffer.as_str()
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

                        let show_cursor = active_edit
                            .map(|s| (s.blink_timer.elapsed().as_millis() / 500) % 2 == 0)
                            .unwrap_or(false);
                        let cursor_idx = active_edit.map(|s| s.cursor_idx).unwrap_or(0);

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

                            typography.draw_text(
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
                                let (prefix_w, _) = typography.measure_text(prefix, f_size, is_bold);

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
                        || sy as f32 + sh < header_h
                        || sy as f32 > screen_h
                        || (sw < 3.0 && sh < 3.0)
                    {
                        continue;
                    }

                    let is_selected = store.selected_annotation_ids.contains(id);
                    let active_edit = editing_session.filter(|s| s.ann_id == *id);
                    let is_editing = active_edit.is_some();

                    let content = if let Some(edit) = active_edit {
                        edit.buffer.as_str()
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
                                typography.draw_text(
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
                                typography.draw_text(
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

                            let show_cursor = active_edit
                                .map(|s| (s.blink_timer.elapsed().as_millis() / 500) % 2 == 0)
                                .unwrap_or(false);
                            if show_cursor {
                                let (cw, _) = typography.measure_text(content, f_size, false);
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

                    if max_x < 0.0 || min_x > screen_w || max_y < header_h || min_y > screen_h {
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

    fn draw_guides(&self, pixmap: &mut PixmapMut, guides: &SnapGuides, vp: &Viewport, w: u32, h: u32, header_h: f32) {
        let mut guide_paint = Paint::default();
        guide_paint.set_color(self.theme.snap_guide);
        let stroke = Stroke { width: 1.0, ..Default::default() };

        if let Some(ref xs) = guides.x {
            for &gx in xs {
                let (sx, _) = world_to_screen(gx, 0.0, vp);
                let mut pb = PathBuilder::new();
                pb.move_to(sx as f32, header_h);
                pb.line_to(sx as f32, h as f32);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &guide_paint, &stroke, Transform::identity(), None);
                }
            }
        }

        if let Some(ref ys) = guides.y {
            for &gy in ys {
                let (_, sy) = world_to_screen(0.0, gy, vp);
                if sy >= header_h as f64 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use glucose_core::types::Annotation;

    fn make_test_card(id: &str, x: f64, y: f64) -> Annotation {
        Annotation::Text {
            id: id.into(),
            x,
            y,
            width: Some(200.0),
            height: Some(50.0),
            text: format!("Card {}", id),
            font_size: Some(14.0),
            color: None,
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        }
    }

    /// Budget de temps d'une frame complète (scène + docks) en 1440x900, en debug.
    ///
    /// Garde-fou de démarrage : une frame qui dépasse ce budget affame la pompe
    /// de messages de l'OS et produit une fenêtre blanche « Ne répond pas ».
    const FULL_FRAME_BUDGET_MS: u128 = 2_000;

    fn render_one_frame(
        renderer: &mut Renderer,
        pixmap: &mut Pixmap,
        store: &Store,
        ui: &mut UiState,
        dock: &crate::dock::DockManager,
    ) {
        let guides = SnapGuides::default();
        let mut view = pixmap.as_mut();
        renderer.render(&mut view, store, &guides, None, ui, None, 0.0, 0.0);
        crate::dock::render_docks(
            &mut view,
            dock,
            store,
            &renderer.typography,
            &renderer.theme,
            1440.0,
            900.0,
            ui.header_height(),
            ui.scale_factor,
            0.0,
            0.0,
        );
    }

    #[test]
    fn test_full_frame_render_stays_within_time_budget() {
        let mut pixmap = Pixmap::new(1440, 900).expect("pixmap 1440x900");
        let mut store = Store::new("Budget");
        let board_id = store.project.active_board_id.clone();
        store.add_annotation(&board_id, make_test_card("budget-1", 0.0, 0.0));

        let mut renderer = Renderer::new();
        let mut ui = UiState::new();
        let dock = crate::dock::DockManager::new();

        // Frame de chauffe : remplit le cache de glyphes et l'index spatial.
        render_one_frame(&mut renderer, &mut pixmap, &store, &mut ui, &dock);

        let started = std::time::Instant::now();
        render_one_frame(&mut renderer, &mut pixmap, &store, &mut ui, &dock);
        let elapsed = started.elapsed().as_millis();

        assert!(
            elapsed < FULL_FRAME_BUDGET_MS,
            "frame complète : {elapsed} ms (budget {FULL_FRAME_BUDGET_MS} ms)"
        );
    }

    #[test]
    fn test_draw_grid_terminates_on_degenerate_scale() {
        let mut pixmap = Pixmap::new(320, 240).expect("pixmap 320x240");
        let renderer = Renderer::new();

        for bad_scale in [0.0_f64, -1.0, f64::NAN, f64::INFINITY, 1e-12] {
            let mut vp = Viewport::default();
            vp.scale = bad_scale;
            let started = std::time::Instant::now();
            let mut view = pixmap.as_mut();
            renderer.draw_grid(&mut view, &vp, 320, 240, 40.0);
            assert!(
                started.elapsed().as_millis() < 500,
                "draw_grid ne se termine pas pour scale={bad_scale}"
            );
        }
    }

    #[test]
    fn test_symbiotic_hue_cache_invalidation_radius() {
        let mut cache = SymbioticHueCache::new();

        // 3 cartes : T1 à (0,0), T2 à (500,0) (voisine), T3 à (3000,0) (lointaine)
        let t1 = make_test_card("T1", 0.0, 0.0);
        let t2 = make_test_card("T2", 500.0, 0.0);
        let t3 = make_test_card("T3", 3000.0, 0.0);
        let list = vec![t1.clone(), t2.clone(), t3.clone()];

        cache.update_positions_and_invalidate(&list);
        let _ = cache.get_or_compute(&t1, &list);
        let _ = cache.get_or_compute(&t2, &list);
        let _ = cache.get_or_compute(&t3, &list);

        assert_eq!(cache.entries.len(), 3);

        // Déplacer T1 de 50 px : T1 et T2 (< 1200 px) doivent être invalidées, T3 (> 1200 px) doit rester
        let t1_moved = make_test_card("T1", 50.0, 0.0);
        let list_moved = vec![t1_moved.clone(), t2.clone(), t3.clone()];

        cache.update_positions_and_invalidate(&list_moved);

        // T3 est restée en cache
        assert!(cache.entries.contains_key("T3"));
        // T1 et T2 ont été invalidées car dans le rayon de 1 200 px
        assert!(!cache.entries.contains_key("T1"));
        assert!(!cache.entries.contains_key("T2"));
    }

    #[test]
    fn test_symbiotic_hue_cache_stationary_preserves_all_entries() {
        let mut cache = SymbioticHueCache::new();
        let t1 = make_test_card("T1", 100.0, 100.0);
        let t2 = make_test_card("T2", 200.0, 200.0);
        let list = vec![t1.clone(), t2.clone()];

        cache.update_positions_and_invalidate(&list);
        let (h1, _) = cache.get_or_compute(&t1, &list);
        let (h2, _) = cache.get_or_compute(&t2, &list);

        // Deuxième frame stationnaire : aucune modification de position
        cache.update_positions_and_invalidate(&list);
        assert_eq!(cache.entries.len(), 2);
        assert_eq!(cache.entries.get("T1").unwrap().hue, h1);
        assert_eq!(cache.entries.get("T2").unwrap().hue, h2);
    }
}
