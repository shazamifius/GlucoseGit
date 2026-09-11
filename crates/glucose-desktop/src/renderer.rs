//! Moteur de rendu 2D haute fidélité pour Glucose Desktop (PureRef-style).
//! Utilise tiny-skia pour le rendu vectoriel anti-aliasé et fontdue pour la typographie.
//!
//! Ce fichier n'est plus que l'**ordonnanceur** d'une frame : il tient les caches, fait la
//! requête spatiale (loi L1) et appelle les passes dans l'ordre. Chaque passe vit dans son
//! module, avec ses mesures et ses tests :
//!
//! | Module | Passe |
//! |---|---|
//! | [`scale`] | l'unique mise à l'échelle monde → écran (standard § 4.4) |
//! | [`hue`] | les teintes symbiotiques et leur invalidation |
//! | [`scene`] | grille, membranes, images, guides, boîte de sélection |
//! | [`halo`] | les halos d'ambiance |
//! | [`card`] | les cartes de texte |
//! | [`note`] | les pense-bêtes et les flèches |

pub mod card;
pub mod halo;
pub mod hue;
pub mod note;
pub mod scale;
pub mod scene;

use crate::canvas::screen_to_world;
use crate::params::{Pointer, SceneOverlay, ViewPass};
use crate::theme::Theme;
use crate::typography::Typography;
use crate::ui::{render_ui, UiState};
use glucose_core::quadtree::SpatialHash;
use glucose_core::store::Store;
use hue::SymbioticHueCache;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tiny_skia::{PathBuilder, Pixmap, PixmapMut};

#[derive(Debug, Clone)]
pub struct TextEditSession {
    pub ann_id: String,
    pub buffer: String,
    pub cursor_idx: usize,
    pub blink_timer: std::time::Instant,
}

/// Décode une couleur hexadécimale #RRGGBB ou #RGB
pub(crate) fn parse_hex_color(hex: &str, default_r: u8, default_g: u8, default_b: u8) -> (u8, u8, u8) {
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

/// Ajoute un rectangle à coins arrondis dans un PathBuilder.
///
/// Le rayon est ramené à la moitié du plus petit côté : c'est une contrainte **géométrique**
/// — un coin ne peut pas être plus rond que la forme — et non une borne sur une valeur
/// dérivée du zoom, puisque rayon et côtés subissent la même mise à l'échelle.
pub(crate) fn push_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
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
        ui: &mut UiState,
        overlay: SceneOverlay<'_>,
        pointer: Pointer,
    ) {
        let width = pixmap.width();
        let height = pixmap.height();
        let vp = store.active_board().map(|b| b.viewport).unwrap_or_default();

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
        let pass = ViewPass { vp, visible_ids: &visible_ids, header_h };

        // 1. Fond sombre sleek PureRef
        pixmap.fill(self.theme.bg_canvas);
        crate::perf::stage("clear");

        // 2. Grille de points infinie
        scene::draw_grid(pixmap, &vp, width, height, header_h);
        crate::perf::stage("grid");

        // 3. Halos symbiotiques d'ambiance (Biome 2D + composition par anneaux)
        halo::draw_halos(&mut self.hue_cache, pixmap, store, pass);
        crate::perf::stage("halos");

        // 4. Membranes (pointillés, titre protecteur en haut à gauche)
        scene::draw_membranes(&self.typography, pixmap, store, pass);
        crate::perf::stage("membranes");

        // 5. Images
        scene::draw_images(
            &mut self.image_cache,
            &mut self.failed_images,
            &self.typography,
            pixmap,
            store,
            pass,
        );
        crate::perf::stage("images");

        // 6. Annotations (cartes de texte, pense-bêtes, flèches + édition live in-place)
        card::draw_annotations(
            &mut self.hue_cache,
            &self.typography,
            pixmap,
            store,
            overlay.editing,
            pass,
        );
        crate::perf::stage("annotations");

        // 7. Guides d'alignement intelligents (SNAP-1)
        if ui.smart_align {
            scene::draw_guides(&self.theme, pixmap, overlay.guides, &vp, (width, height), header_h);
        }

        // 8. Boîte de sélection élastique (Marquee)
        if let Some((x1, y1, x2, y2)) = overlay.selection_box {
            scene::draw_selection_box(pixmap, (x1, y1), (x2, y2));
        }

        // 9. Interface utilisateur complète (TopBar, Tabs, Minimap, Toasts)
        render_ui(pixmap, store, ui, &self.typography, &self.theme, pointer);
        crate::perf::stage("ui");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ScreenFrame;
    use glucose_core::smart_align::SnapGuides;

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
        let overlay = SceneOverlay { guides: &guides, selection_box: None, editing: None };
        let origin = Pointer { x: 0.0, y: 0.0 };
        renderer.render(&mut view, store, ui, overlay, origin);
        crate::dock::render_docks(
            &mut view,
            dock,
            store,
            &renderer.typography,
            &renderer.theme,
            ScreenFrame {
                width: 1440.0,
                height: 900.0,
                header_h: ui.header_height(),
                scale: ui.scale_factor,
            },
            origin,
        );
    }

    #[test]
    fn test_full_frame_render_stays_within_time_budget() {
        let mut pixmap = Pixmap::new(1440, 900).expect("pixmap 1440x900");
        let mut store = Store::new("Budget");
        let board_id = store.project.active_board_id.clone();
        store.add_annotation(&board_id, card::tests::probe_card("budget-1", 0.0, 0.0));

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
        // Ce que la frame coûte au cache de glyphes, phases sous-pixel comprises (GLYPH-1).
        println!(
            "[perf] frame complète : {elapsed} ms, {} variantes de glyphes en cache",
            renderer.typography.cached_glyph_count()
        );
    }
}
