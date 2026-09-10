//! Application Glucose Desktop — Event Loop Winit 0.30 et Framebuffer Softbuffer 0.4.

use crate::canvas::{screen_to_world, zoom_at};
use crate::renderer::Renderer;
use crate::ui::{handle_ui_click, ActiveTool, UiAction, UiState, TOTAL_HEADER_HEIGHT};
use arboard::Clipboard;
use glucose_core::hit_priority::{collect_candidates, PickInput, PickOwner};
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, BoardImage};
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;
use tiny_skia::Pixmap;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId, WindowLevel};

pub struct GlucoseApp {
    pub store: Store,
    pub renderer: Renderer,
    pub pixmap: Option<Pixmap>,
    pub ui: UiState,
    pub window: Option<Arc<Window>>,
    pub context: Option<softbuffer::Context<Arc<Window>>>,
    pub surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,

    // États d'interaction
    pub mouse_pos: (f64, f64),
    pub modifiers: ModifiersState,
    pub is_panning: bool,
    pub is_dragging_item: bool,
    pub drag_start_world: (f64, f64),
    pub active_guides: SnapGuides,
    pub selection_box: Option<(f64, f64, f64, f64)>,
    pub always_on_top: bool,
}

impl GlucoseApp {
    pub fn new() -> Self {
        let mut store = Store::new("Glucose Native");
        let active_bid = store.project.active_board_id.clone();

        // Carte d'accueil par défaut au look Glucose moderne
        let welcome_card = Annotation::Text {
            id: "welcome-card".into(),
            x: 0.0,
            y: 0.0,
            width: Some(260.0),
            height: Some(48.0),
            text: "Bienvenue dans Glucose !".into(),
            font_size: Some(14.0),
            color: None,
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        };
        store.add_annotation(&active_bid, welcome_card);

        Self {
            store,
            renderer: Renderer::new(),
            pixmap: None,
            ui: UiState::new(),
            window: None,
            context: None,
            surface: None,
            mouse_pos: (0.0, 0.0),
            modifiers: ModifiersState::empty(),
            is_panning: false,
            is_dragging_item: false,
            drag_start_world: (0.0, 0.0),
            active_guides: SnapGuides::default(),
            selection_box: None,
            always_on_top: false,
        }
    }

    pub fn redraw(&mut self) {
        if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
            let size = window.inner_size();
            let width = size.width.max(1);
            let height = size.height.max(1);

            if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
                let _ = surface.resize(w, h);
            }

            let need_new_pixmap = match &self.pixmap {
                Some(p) => p.width() != width || p.height() != height,
                None => true,
            };
            if need_new_pixmap {
                self.pixmap = Pixmap::new(width, height);
            }

            if let Some(pixmap) = &mut self.pixmap {
                let mut pixmap_mut = pixmap.as_mut();
                self.renderer.render(
                    &mut pixmap_mut,
                    &self.store,
                    &self.active_guides,
                    self.selection_box,
                    &mut self.ui,
                    self.mouse_pos.0 as f32,
                    self.mouse_pos.1 as f32,
                );

                if let Ok(mut buffer) = surface.buffer_mut() {
                    let src = pixmap.data();
                    for (dst, chunk) in buffer.iter_mut().zip(src.chunks_exact(4)) {
                        *dst = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
                    }
                    let _ = buffer.present();
                }
            }
        }
    }

    /// Importe une liste de chemins de fichiers image dans le board actif
    pub fn import_image_files(&mut self, paths: &[std::path::PathBuf]) {
        let active_bid = self.store.project.active_board_id.clone();
        let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
        let (mut cur_wx, mut cur_wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

        if self.mouse_pos.1 < TOTAL_HEADER_HEIGHT as f64 {
            cur_wx = 0.0;
            cur_wy = 0.0;
        }

        let mut count = 0;
        for path_buf in paths {
            if let Some(path_str) = path_buf.to_str() {
                let (w, h) = if let Ok(dyn_img) = image::open(path_buf) {
                    (dyn_img.width() as f64, dyn_img.height() as f64)
                } else {
                    (300.0, 200.0)
                };

                let max_dim = 600.0f64;
                let scale = if w > max_dim || h > max_dim {
                    (max_dim / w).min(max_dim / h)
                } else {
                    1.0
                };
                let final_w = (w * scale).max(50.0);
                let final_h = (h * scale).max(50.0);

                let id = format!("img-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
                let mut img = BoardImage::new(id, cur_wx, cur_wy, final_w, final_h);
                img.src = Some(path_str.to_string());
                img.original_width = w;
                img.original_height = h;

                self.store.add_image(&active_bid, img);
                cur_wx += final_w + 30.0;
                count += 1;
            }
        }

        if count > 0 {
            self.ui.show_toast(format!("📥 {} image(s) ajoutée(s)", count));
            self.redraw();
        }
    }

    /// Coller depuis le presse-papiers (Image ou Texte)
    pub fn paste_from_clipboard(&mut self) {
        let active_bid = self.store.project.active_board_id.clone();
        let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
        let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

        if let Ok(mut clipboard) = Clipboard::new() {
            // 1. Tenter de coller une image bitmap (Pinterest, navigateur, capture d'écran)
            if let Ok(img_data) = clipboard.get_image() {
                let w = img_data.width as usize;
                let h = img_data.height as usize;
                let temp_dir = std::env::temp_dir().join("glucose_pasted");
                let _ = std::fs::create_dir_all(&temp_dir);
                let filename = format!("paste_{}.png", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
                let file_path = temp_dir.join(filename);

                if image::save_buffer(
                    &file_path,
                    &img_data.bytes,
                    w as u32,
                    h as u32,
                    image::ExtendedColorType::Rgba8,
                ).is_ok() {
                    let mut img = BoardImage::new(
                        format!("paste-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
                        wx,
                        wy,
                        (w as f64).min(600.0),
                        (h as f64) * ((w as f64).min(600.0) / (w as f64).max(1.0)),
                    );
                    img.src = Some(file_path.to_string_lossy().to_string());
                    img.original_width = w as f64;
                    img.original_height = h as f64;

                    self.store.add_image(&active_bid, img);
                    self.ui.show_toast("📥 Image collée");
                    self.redraw();
                    return;
                }
            }

            // 2. Tenter de coller du texte ou un chemin de fichier
            if let Ok(text) = clipboard.get_text() {
                let trimmed = text.trim();
                let path = Path::new(trimmed);
                if path.exists() && path.is_file() {
                    self.import_image_files(&[path.to_path_buf()]);
                    return;
                }

                // Coller en tant que carte texte
                let aid = format!("text-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                let ann = Annotation::Text {
                    id: aid,
                    x: wx,
                    y: wy,
                    width: Some(220.0),
                    height: Some(44.0),
                    text: trimmed.to_string(),
                    font_size: Some(13.0),
                    color: None,
                    cursor_pos: None,
                    source_file: None,
                    membrane_id: None,
                    domains: Vec::new(),
                    mirror_of: None,
                    temporal_anchor: None,
                };
                self.store.add_annotation(&active_bid, ann);
                self.ui.show_toast("📝 Texte collé");
                self.redraw();
            }
        }
    }

    /// Réorganise automatiquement les éléments en grille ordonnée
    pub fn organize_layout(&mut self) {
        if let Some(board) = self.store.active_board_mut() {
            if board.images.is_empty() && board.annotations.is_empty() {
                return;
            }

            let total_count = board.images.len() + board.annotations.len();
            let cols = (total_count as f64).sqrt().ceil() as usize;
            let cols = cols.max(1);
            let padding = 40.0;
            let mut cur_x = 0.0;
            let mut cur_y = 0.0;
            let mut row_max_h = 0.0f64;

            let mut idx = 0;
            for img in &mut board.images {
                img.x = cur_x + img.width / 2.0;
                img.y = cur_y + img.height / 2.0;
                row_max_h = row_max_h.max(img.height);
                cur_x += img.width + padding;
                idx += 1;

                if idx % cols == 0 {
                    cur_x = 0.0;
                    cur_y += row_max_h + padding;
                    row_max_h = 0.0;
                }
            }

            for ann in &mut board.annotations {
                match ann {
                    Annotation::Text { x, y, width, height, .. } => {
                        let w = width.unwrap_or(120.0);
                        let h = height.unwrap_or(40.0);
                        *x = cur_x;
                        *y = cur_y;
                        row_max_h = row_max_h.max(h);
                        cur_x += w + padding;
                        idx += 1;
                        if idx % cols == 0 {
                            cur_x = 0.0;
                            cur_y += row_max_h + padding;
                            row_max_h = 0.0;
                        }
                    }
                    Annotation::Sticky { x, y, width, height, .. } => {
                        let w = width.unwrap_or(160.0);
                        let h = height.unwrap_or(120.0);
                        *x = cur_x;
                        *y = cur_y;
                        row_max_h = row_max_h.max(h);
                        cur_x += w + padding;
                        idx += 1;
                        if idx % cols == 0 {
                            cur_x = 0.0;
                            cur_y += row_max_h + padding;
                            row_max_h = 0.0;
                        }
                    }
                    _ => {}
                }
            }
        }
        self.ui.show_toast("📐 Canvas ordonné");
        self.redraw();
    }
}

impl ApplicationHandler for GlucoseApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = WindowAttributes::default()
                .with_title("GLUCOSE — PureRef Native Rust")
                .with_inner_size(LogicalSize::new(1440.0, 900.0));

            if let Ok(w) = event_loop.create_window(attrs) {
                let window = Arc::new(w);
                if let Ok(context) = softbuffer::Context::new(window.clone()) {
                    if let Ok(mut surface) = softbuffer::Surface::new(&context, window.clone()) {
                        let size = window.inner_size();
                        let width = size.width.max(1);
                        let height = size.height.max(1);
                        if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
                            let _ = surface.resize(w, h);
                        }
                        self.pixmap = Pixmap::new(width, height);
                        self.window = Some(window);
                        self.context = Some(context);
                        self.surface = Some(surface);
                        self.redraw();
                    }
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                let width = size.width.max(1);
                let height = size.height.max(1);
                if let Some(surface) = &mut self.surface {
                    if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
                        let _ = surface.resize(w, h);
                    }
                }
                self.pixmap = Pixmap::new(width, height);
                self.redraw();
            }
            WindowEvent::RedrawRequested => {
                self.redraw();
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let prev_pos = self.mouse_pos;
                self.mouse_pos = (position.x, position.y);
                let dx = position.x - prev_pos.0;
                let dy = position.y - prev_pos.1;

                if self.is_panning {
                    self.store.pan(dx, dy);
                    self.redraw();
                } else if self.is_dragging_item {
                    let active_bid = self.store.project.active_board_id.clone();
                    let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
                    let (wx, wy) = screen_to_world(position.x, position.y, &vp);
                    let w_dx = wx - self.drag_start_world.0;
                    let w_dy = wy - self.drag_start_world.1;

                    self.store.move_selected(&active_bid, w_dx, w_dy);
                    self.drag_start_world = (wx, wy);
                    self.redraw();
                } else if let Some((bx1, by1, _, _)) = self.selection_box {
                    self.selection_box = Some((bx1, by1, position.x, position.y));
                    self.redraw();
                } else if position.y < TOTAL_HEADER_HEIGHT as f64 {
                    // Hover sur les boutons d'en-tête
                    self.redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let factor = match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        if y > 0.0 { 1.15 } else { 0.85 }
                    }
                    MouseScrollDelta::PixelDelta(p) => {
                        if p.y > 0.0 { 1.10 } else { 0.90 }
                    }
                };
                if let Some(board) = self.store.active_board_mut() {
                    zoom_at(&mut board.viewport, factor, self.mouse_pos.0, self.mouse_pos.1);
                }
                self.redraw();
            }
            WindowEvent::MouseInput { button, state, .. } => {
                let (screen_w, screen_h) = if let Some(w) = &self.window {
                    let sz = w.inner_size();
                    (sz.width as f32, sz.height as f32)
                } else {
                    (1280.0, 720.0)
                };

                match button {
                    MouseButton::Right | MouseButton::Middle => {
                        self.is_panning = state == ElementState::Pressed;
                    }
                    MouseButton::Left => {
                        if state == ElementState::Pressed {
                            let mx = self.mouse_pos.0 as f32;
                            let my = self.mouse_pos.1 as f32;

                            // Clic sur l'interface (Header / TopBar / Tabs)
                            if my < TOTAL_HEADER_HEIGHT {
                                if let Some(action) = handle_ui_click(
                                    mx,
                                    my,
                                    screen_w,
                                    screen_h,
                                    &self.store,
                                    &mut self.ui,
                                    &self.renderer.typography,
                                ) {
                                    match action {
                                        UiAction::SelectTool(tool) => {
                                            self.ui.active_tool = tool;
                                        }
                                        UiAction::AddImages => {
                                            if let Some(files) = rfd::FileDialog::new()
                                                .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
                                                .pick_files()
                                            {
                                                self.import_image_files(&files);
                                            }
                                        }
                                        UiAction::Organize => {
                                            self.organize_layout();
                                        }
                                        UiAction::ToggleTimer => {
                                            self.ui.show_toast("⏱ Timer démarré (25m)");
                                        }
                                        UiAction::ToggleStoryboard => {
                                            self.ui.show_toast("🎬 Mode Storyboard");
                                        }
                                        UiAction::ToggleMagnet => {
                                            // Déjà basculé dans handle_ui_click
                                        }
                                        UiAction::ToggleTransDomain => {
                                            self.ui.show_toast(if self.ui.trans_domain {
                                                "🌌 Trans-domaines activé"
                                            } else {
                                                "Trans-domaines désactivé"
                                            });
                                        }
                                        UiAction::ToggleCollab => {
                                            // Déjà basculé dans handle_ui_click
                                        }
                                        UiAction::ExportMenu => {
                                            self.ui.show_toast("💾 Exportation du canvas");
                                        }
                                        UiAction::TogglePlugins => {
                                            self.ui.show_toast("🧩 Extensions & Plugins");
                                        }
                                        UiAction::TogglePreset => {
                                            self.ui.show_toast("🎨 Préréglage PureRef appliqué");
                                        }
                                        UiAction::ToggleDomains => {
                                            self.ui.show_toast("🏷 Domaines thématiques");
                                        }
                                        UiAction::SelectBoard(id) => {
                                            self.store.set_active_board_id(&id);
                                        }
                                        UiAction::AddBoard => {
                                            let new_name = format!("Board {}", self.store.project.boards.len() + 1);
                                            let new_id = self.store.add_board(new_name);
                                            self.store.set_active_board_id(new_id);
                                            self.ui.show_toast("📋 Nouveau board créé");
                                        }
                                        UiAction::MinimapPan(wx, wy) => {
                                            if let Some(b) = self.store.active_board_mut() {
                                                b.viewport.x = screen_w as f64 / 2.0 - wx * b.viewport.scale;
                                                b.viewport.y = screen_h as f64 / 2.0 - wy * b.viewport.scale;
                                            }
                                        }
                                    }
                                }
                                self.redraw();
                                return;
                            }

                            // Clic sur le canvas
                            let active_bid = self.store.project.active_board_id.clone();
                            let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
                            let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

                            // Outils interactifs
                            match self.ui.active_tool {
                                ActiveTool::Pan => {
                                    self.is_panning = true;
                                    return;
                                }
                                ActiveTool::Text => {
                                    let aid = format!("text-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                                    let ann = Annotation::Text {
                                        id: aid,
                                        x: wx,
                                        y: wy,
                                        width: Some(180.0),
                                        height: Some(44.0),
                                        text: "Nouveau texte".into(),
                                        font_size: Some(14.0),
                                        color: None,
                                        cursor_pos: None,
                                        source_file: None,
                                        membrane_id: None,
                                        domains: Vec::new(),
                                        mirror_of: None,
                                        temporal_anchor: None,
                                    };
                                    self.store.add_annotation(&active_bid, ann);
                                    self.ui.show_toast("📝 Carte texte ajoutée");
                                    self.ui.active_tool = ActiveTool::Select;
                                    self.redraw();
                                    return;
                                }
                                ActiveTool::Sticky => {
                                    let aid = format!("sticky-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                                    let ann = Annotation::Sticky {
                                        id: aid,
                                        x: wx,
                                        y: wy,
                                        width: Some(180.0),
                                        height: Some(130.0),
                                        text: "Nouvelle note".into(),
                                        font_size: Some(12.0),
                                        color: Some("#1c1917".into()),
                                        bg_color: Some("#fef08a".into()),
                                        cursor_pos: None,
                                        operator: None,
                                        source_file: None,
                                        membrane_id: None,
                                        domains: Vec::new(),
                                        mirror_of: None,
                                        temporal_anchor: None,
                                    };
                                    self.store.add_annotation(&active_bid, ann);
                                    self.ui.show_toast("📌 Sticky note ajoutée");
                                    self.ui.active_tool = ActiveTool::Select;
                                    self.redraw();
                                    return;
                                }
                                ActiveTool::Arrow => {
                                    let aid = format!("arrow-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                                    let ann = Annotation::Arrow {
                                        id: aid,
                                        x: wx,
                                        y: wy,
                                        x2: wx + 120.0,
                                        y2: wy + 80.0,
                                        text: None,
                                        font_size: None,
                                        color: Some("#94a3b8".into()),
                                        arrow_type: None,
                                        arrow_bidirectional: false,
                                        predicate: None,
                                        stroke_width: Some(2.0),
                                        waypoints: Vec::new(),
                                        source_id: None,
                                        target_id: None,
                                        source_block_id: None,
                                        target_block_id: None,
                                        source_text_sel: None,
                                        target_text_sel: None,
                                        long_text: None,
                                        target_board_id: None,
                                        membrane_id: None,
                                        domains: Vec::new(),
                                        mirror_of: None,
                                        temporal_anchor: None,
                                    };
                                    self.store.add_annotation(&active_bid, ann);
                                    self.ui.show_toast("↗️ Flèche ajoutée");
                                    self.ui.active_tool = ActiveTool::Select;
                                    self.redraw();
                                    return;
                                }
                                ActiveTool::Membrane => {
                                    let aid = format!("membrane-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                                    let ann = Annotation::Membrane {
                                        id: aid,
                                        x: wx,
                                        y: wy,
                                        width: 320.0,
                                        height: 240.0,
                                        color: Some("#60a5fa".into()),
                                        text: Some("Groupe".into()),
                                        mode: glucose_core::types::MembraneMode::Classic,
                                        curtains: Vec::new(),
                                        membrane_id: None,
                                        domains: Vec::new(),
                                        mirror_of: None,
                                        temporal_anchor: None,
                                    };
                                    self.store.add_annotation(&active_bid, ann);
                                    self.ui.show_toast("🧊 Membrane créée");
                                    self.ui.active_tool = ActiveTool::Select;
                                    self.redraw();
                                    return;
                                }
                                ActiveTool::Folder => {
                                    self.ui.show_toast("📁 Dossier");
                                    self.ui.active_tool = ActiveTool::Select;
                                    return;
                                }
                                ActiveTool::Select => {}
                            }

                            // Sélection par clic (PICK-1)
                            let mut selected = false;
                            if let Some(b) = self.store.active_board() {
                                let input = PickInput {
                                    wx,
                                    wy,
                                    scale: vp.scale,
                                    images: &b.images,
                                    annotations: &b.annotations,
                                    folders: &b.folders,
                                    selected_image_ids: &self.store.selected_image_ids,
                                    selected_annotation_ids: &self.store.selected_annotation_ids,
                                    selected_folder_id: self.store.selected_folder_id.as_deref(),
                                    arrow_id: None,
                                    dom_hint: None,
                                };
                                let candidates = collect_candidates(&input);
                                if let Some(top) = candidates.first() {
                                    match top.owner {
                                        PickOwner::Image => {
                                            self.store.select_image(top.id.clone(), self.modifiers.shift_key());
                                            selected = true;
                                        }
                                        PickOwner::Annotation | PickOwner::Membrane | PickOwner::Arrow => {
                                            self.store.select_annotation(top.id.clone(), self.modifiers.shift_key());
                                            selected = true;
                                        }
                                        PickOwner::Folder => {
                                            self.store.select_folder(top.id.clone());
                                            selected = true;
                                        }
                                    }
                                }
                            }

                            if selected {
                                self.is_dragging_item = true;
                                self.drag_start_world = (wx, wy);
                                self.store.begin_live_edit();
                            } else {
                                self.store.clear_selection();
                                self.selection_box = Some((self.mouse_pos.0, self.mouse_pos.1, self.mouse_pos.0, self.mouse_pos.1));
                            }
                            self.redraw();
                        } else {
                            if self.is_dragging_item {
                                self.is_dragging_item = false;
                                self.store.end_live_edit();
                                self.active_guides = SnapGuides::default();
                            }
                            if self.selection_box.is_some() {
                                self.selection_box = None;
                            }
                            self.redraw();
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    let ctrl = self.modifiers.control_key();
                    let active_bid = self.store.project.active_board_id.clone();

                    match event.logical_key {
                        Key::Named(NamedKey::Delete) | Key::Named(NamedKey::Backspace) => {
                            self.store.delete_selected(&active_bid);
                            self.ui.show_toast("🗑 Supprimé");
                            self.redraw();
                        }
                        Key::Named(NamedKey::Space) => {
                            self.ui.active_tool = if self.ui.active_tool == ActiveTool::Pan {
                                ActiveTool::Select
                            } else {
                                ActiveTool::Pan
                            };
                            self.redraw();
                        }
                        Key::Character(c) => match c.as_str() {
                            "v" | "V" => {
                                if ctrl {
                                    self.paste_from_clipboard();
                                } else {
                                    self.ui.active_tool = ActiveTool::Select;
                                    self.redraw();
                                }
                            }
                            "z" | "Z" => {
                                if ctrl {
                                    if self.modifiers.shift_key() {
                                        if self.store.redo() {
                                            self.ui.show_toast("🔁 Rétablir");
                                        }
                                    } else if self.store.undo() {
                                        self.ui.show_toast("↩️ Annuler");
                                    }
                                    self.redraw();
                                }
                            }
                            "y" | "Y" => {
                                if ctrl {
                                    if self.store.redo() {
                                        self.ui.show_toast("🔁 Rétablir");
                                    }
                                    self.redraw();
                                }
                            }
                            "d" | "D" => {
                                if ctrl {
                                    self.store.duplicate_selected(&active_bid);
                                    self.ui.show_toast("📑 Dupliqué");
                                    self.redraw();
                                }
                            }
                            "a" | "A" => {
                                if ctrl {
                                    // Sélectionner tout sur le board actif
                                    if let Some(b) = self.store.active_board() {
                                        let img_ids = b.images.iter().map(|i| i.id.clone()).collect();
                                        let ann_ids = b.annotations.iter().map(|a| a.id().to_string()).collect();
                                        self.store.set_selected_image_ids(img_ids);
                                        self.store.set_selected_annotation_ids(ann_ids);
                                    }
                                    self.redraw();
                                } else {
                                    self.ui.active_tool = ActiveTool::Arrow;
                                    self.redraw();
                                }
                            }
                            "t" | "T" => {
                                if self.modifiers.alt_key() {
                                    self.always_on_top = !self.always_on_top;
                                    if let Some(w) = &self.window {
                                        w.set_window_level(if self.always_on_top {
                                            WindowLevel::AlwaysOnTop
                                        } else {
                                            WindowLevel::Normal
                                        });
                                    }
                                    self.ui.show_toast(if self.always_on_top {
                                        "📌 Toujours au premier plan"
                                    } else {
                                        "Fenêtre normale"
                                    });
                                    self.redraw();
                                } else {
                                    self.ui.active_tool = ActiveTool::Text;
                                    self.redraw();
                                }
                            }
                            "n" | "N" => {
                                self.ui.active_tool = ActiveTool::Sticky;
                                self.redraw();
                            }
                            "m" | "M" => {
                                self.ui.active_tool = ActiveTool::Membrane;
                                self.redraw();
                            }
                            "f" | "F" => {
                                // Fit view / recentrer la caméra PureRef
                                if let Some(b) = self.store.active_board_mut() {
                                    b.viewport.x = 0.0;
                                    b.viewport.y = 0.0;
                                    b.viewport.scale = 1.0;
                                }
                                self.ui.show_toast("🎯 Vue recentrée");
                                self.redraw();
                            }
                            "o" | "O" => {
                                if ctrl {
                                    if let Some(files) = rfd::FileDialog::new()
                                        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
                                        .pick_files()
                                    {
                                        self.import_image_files(&files);
                                    }
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            WindowEvent::DroppedFile(path_buf) => {
                // Drag & Drop universel d'image OS (WebP, PNG, JPEG, GIF, BMP)
                self.import_image_files(&[path_buf]);
            }
            _ => {}
        }
    }
}
