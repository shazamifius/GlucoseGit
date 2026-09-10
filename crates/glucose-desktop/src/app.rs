//! Application Glucose Desktop — Event Loop Winit 0.30 et Framebuffer Softbuffer 0.4.

use crate::canvas::{screen_to_world, zoom_at};
use crate::dock::{
    apply_organize_layout, compute_panel_layouts, handle_dock_click, render_docks, DockManager,
    DragSession, OrganizeState, PanelClickResult, TabId,
};
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
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use crate::renderer::TextEditSession;
use winit::window::{Window, WindowAttributes, WindowId, WindowLevel};

pub struct LastClickInfo {
    pub time: std::time::Instant,
    pub pos: (f64, f64),
    pub id: String,
}

pub struct GlucoseApp {
    pub store: Store,
    pub renderer: Renderer,
    pub pixmap: Option<Pixmap>,
    pub ui: UiState,
    pub dock_manager: DockManager,
    pub window: Option<Arc<Window>>,
    pub context: Option<softbuffer::Context<Arc<Window>>>,
    pub surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,

    // États d'interaction
    pub mouse_pos: (f64, f64),
    pub modifiers: ModifiersState,
    pub space_pressed: bool,
    pub right_or_middle_down: bool,
    pub is_panning: bool,
    pub is_dragging_item: bool,
    pub drag_start_world: (f64, f64),
    pub active_guides: SnapGuides,
    pub selection_box: Option<(f64, f64, f64, f64)>,
    pub always_on_top: bool,

    // Session d'édition de texte in-place (double-clic)
    pub editing_session: Option<TextEditSession>,
    pub last_click: Option<LastClickInfo>,
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
            text: "# Bienvenue dans Glucose !\n- 100% Rust ultra-rapide\n- Teintes symbiotiques dynamiques\n- Double-cliquez pour éditer".into(),
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
            dock_manager: DockManager::new(),
            window: None,
            context: None,
            surface: None,
            mouse_pos: (0.0, 0.0),
            modifiers: ModifiersState::empty(),
            space_pressed: false,
            right_or_middle_down: false,
            is_panning: false,
            is_dragging_item: false,
            drag_start_world: (0.0, 0.0),
            active_guides: SnapGuides::default(),
            selection_box: None,
            always_on_top: false,
            editing_session: None,
            last_click: None,
        }
    }

    pub fn commit_editing(&mut self) {
        if let Some(session) = self.editing_session.take() {
            let is_empty = session.buffer.trim().is_empty();
            let mut should_delete = false;

            if let Some(b) = self.store.active_board_mut() {
                if let Some(ann) = b.annotations.iter_mut().find(|a| a.id() == session.ann_id) {
                    match ann {
                        Annotation::Text { text, .. } => {
                            if is_empty {
                                should_delete = true;
                            } else {
                                *text = session.buffer.clone();
                            }
                        }
                        Annotation::Sticky { text, .. } => {
                            *text = session.buffer.clone();
                        }
                        Annotation::Membrane { text, .. } => {
                            *text = if is_empty { None } else { Some(session.buffer.clone()) };
                        }
                        _ => {}
                    }
                }
            }

            if should_delete {
                if let Some(b) = self.store.active_board_mut() {
                    b.annotations.retain(|a| a.id() != session.ann_id);
                }
            }
            self.store.push_undo();
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
                    self.editing_session.as_ref(),
                    self.mouse_pos.0 as f32,
                    self.mouse_pos.1 as f32,
                );

                // Rendu des panneaux déroulants & flottants (Top & Bottom Docks)
                render_docks(
                    &mut pixmap_mut,
                    &self.dock_manager,
                    &self.store,
                    &self.renderer.typography,
                    width as f32,
                    height as f32,
                    TOTAL_HEADER_HEIGHT,
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
    #[allow(dead_code)]
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

    /// Applique la réorganisation issue du panneau ORDONNER (Masonry, Grille, Même Hauteur, etc.)
    pub fn apply_dock_layout(&mut self, state: &OrganizeState) {
        if let Some(board) = self.store.active_board_mut() {
            if board.images.is_empty() {
                self.ui.show_toast("⚠️ Aucune image sur le canvas");
                return;
            }

            let results = apply_organize_layout(&board.images, state);
            for res in results {
                if let Some(img) = board.images.iter_mut().find(|i| i.id == res.id) {
                    img.x = res.x;
                    img.y = res.y;
                    img.width = res.width;
                    img.height = res.height;
                }
            }
        }
        self.store.push_undo();
        self.ui.show_toast(format!("📐 Disposition {} appliquée", state.layout.title()));
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

                if self.dock_manager.drag.is_some() {
                    self.dock_manager.update_drag(position.x as f32, position.y as f32);
                    self.redraw();
                    return;
                }

                if self.is_panning {
                    // Protection contre les sauts anormaux du curseur OS
                    if dx.hypot(dy) < 300.0 {
                        self.store.pan(dx, dy);
                    }
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
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        if y.abs() > 0.001 {
                            // Zoom continu centré sur le curseur (comme PureRef)
                            let factor = (1.12f64).powf(y as f64);
                            if let Some(board) = self.store.active_board_mut() {
                                zoom_at(&mut board.viewport, factor, self.mouse_pos.0, self.mouse_pos.1);
                            }
                        }
                        if x.abs() > 0.001 {
                            self.store.pan(x as f64 * 30.0, 0.0);
                        }
                    }
                    MouseScrollDelta::PixelDelta(p) => {
                        if self.modifiers.control_key() {
                            // Pincement tactile / Ctrl + molette = zoom fin
                            let factor = (1.003f64).powf(p.y);
                            if let Some(board) = self.store.active_board_mut() {
                                zoom_at(&mut board.viewport, factor, self.mouse_pos.0, self.mouse_pos.1);
                            }
                        } else {
                            // Défilement 2 doigts pavé tactile = pan continu
                            self.store.pan(p.x, p.y);
                        }
                    }
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
                        self.right_or_middle_down = state == ElementState::Pressed;
                        self.is_panning = self.right_or_middle_down;
                        self.redraw();
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
                                            self.dock_manager.toggle_tab(TabId::Organize);
                                        }
                                        UiAction::ToggleTimer => {
                                            self.dock_manager.toggle_tab(TabId::Pomodoro);
                                        }
                                        UiAction::ToggleStoryboard => {
                                            self.dock_manager.toggle_tab(TabId::Storyboard);
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
                                            self.dock_manager.toggle_tab(TabId::Plugins);
                                        }
                                        UiAction::TogglePreset => {
                                            self.dock_manager.toggle_tab(TabId::Preset);
                                        }
                                        UiAction::ToggleDomains => {
                                            self.dock_manager.toggle_tab(TabId::Domains);
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

                            // Clic sur les panneaux déroulants & poignées (Dock)
                            let dock_layouts = compute_panel_layouts(
                                &self.dock_manager,
                                screen_w,
                                screen_h,
                                TOTAL_HEADER_HEIGHT,
                            );

                            // A. Détection du clic sur la poignée (⠿⠿) pour réorganisation / fermeture
                            let mut grip_hit = None;
                            for layout in dock_layouts.iter().rev() {
                                if layout.grip_contains_point(mx, my) {
                                    grip_hit = Some(layout.tab);
                                    break;
                                }
                            }
                            if let Some(tab) = grip_hit {
                                self.dock_manager.drag = Some(DragSession {
                                    tab,
                                    start_x: mx,
                                    start_y: my,
                                    current_x: mx,
                                    current_y: my,
                                });
                                self.redraw();
                                return;
                            }

                            // B. Clic à l'intérieur du corps d'un panneau
                            if let Some(action) = handle_dock_click(
                                &mut self.dock_manager,
                                &self.store,
                                mx,
                                my,
                                screen_w,
                                screen_h,
                                TOTAL_HEADER_HEIGHT,
                            ) {
                                match action {
                                    PanelClickResult::ApplyLayout(state) => {
                                        self.apply_dock_layout(&state);
                                    }
                                    PanelClickResult::AddDomain => {
                                        let did = format!("domain-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                                        let name = format!("Domaine {}", self.dock_manager.domains.domains.len() + 1);
                                        self.dock_manager.domains.domains.push(crate::dock::DomainItem {
                                            id: did,
                                            name,
                                            color: tiny_skia::Color::from_rgba8(168, 85, 247, 255),
                                        });
                                    }
                                    _ => {}
                                }
                                self.redraw();
                                return;
                            }

                            // Clic sur le canvas
                            if self.space_pressed || self.ui.active_tool == ActiveTool::Pan {
                                self.is_panning = true;
                                return;
                            }

                            let active_bid = self.store.project.active_board_id.clone();
                            let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
                            let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

                            // Outils interactifs
                            match self.ui.active_tool {
                                ActiveTool::Pan => unreachable!(),
                                ActiveTool::Text => {
                                    let aid = format!("text-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                                    let initial_str = "Nouveau texte".to_string();
                                    let ann = Annotation::Text {
                                        id: aid.clone(),
                                        x: wx,
                                        y: wy,
                                        width: Some(240.0),
                                        height: Some(48.0),
                                        text: initial_str.clone(),
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
                                    let cur_idx = initial_str.len();
                                    self.editing_session = Some(TextEditSession {
                                        ann_id: aid,
                                        buffer: initial_str,
                                        cursor_idx: cur_idx,
                                        blink_timer: std::time::Instant::now(),
                                    });
                                    self.ui.show_toast("📝 Édition du texte");
                                    self.ui.active_tool = ActiveTool::Select;
                                    self.redraw();
                                    return;
                                }
                                ActiveTool::Sticky => {
                                    let aid = format!("sticky-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                                    let initial_str = "Nouvelle note".to_string();
                                    let ann = Annotation::Sticky {
                                        id: aid.clone(),
                                        x: wx,
                                        y: wy,
                                        width: Some(180.0),
                                        height: Some(130.0),
                                        text: initial_str.clone(),
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
                                    let cur_idx = initial_str.len();
                                    self.editing_session = Some(TextEditSession {
                                        ann_id: aid,
                                        buffer: initial_str,
                                        cursor_idx: cur_idx,
                                        blink_timer: std::time::Instant::now(),
                                    });
                                    self.ui.show_toast("📌 Édition du sticky");
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
                                    self.ui.show_toast("🧊 Membrane créée (rx=60)");
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
                                    // Détection de double-clic (intervalle < 350ms et delta < 8.0px sur le même élément)
                                    let is_dbl_click = if let Some(ref lc) = self.last_click {
                                        lc.id == top.id
                                            && lc.time.elapsed().as_millis() < 350
                                            && (lc.pos.0 - self.mouse_pos.0).hypot(lc.pos.1 - self.mouse_pos.1) < 8.0
                                    } else {
                                        false
                                    };

                                    self.last_click = Some(LastClickInfo {
                                        time: std::time::Instant::now(),
                                        pos: self.mouse_pos,
                                        id: top.id.clone(),
                                    });

                                    if is_dbl_click {
                                        if let Some(ann) = b.annotations.iter().find(|a| a.id() == top.id) {
                                            let initial_text = match ann {
                                                Annotation::Text { text, .. } => text.clone(),
                                                Annotation::Sticky { text, .. } => text.clone(),
                                                Annotation::Membrane { text, .. } => text.clone().unwrap_or_default(),
                                                _ => String::new(),
                                            };
                                            let cur_idx = initial_text.len();
                                            self.editing_session = Some(TextEditSession {
                                                ann_id: top.id.clone(),
                                                buffer: initial_text,
                                                cursor_idx: cur_idx,
                                                blink_timer: std::time::Instant::now(),
                                            });
                                            self.redraw();
                                            return;
                                        }
                                    }

                                    // Si on clique sur un autre élément alors qu'on éditait, valider l'édition
                                    if let Some(ref session) = self.editing_session {
                                        if session.ann_id != top.id {
                                            self.commit_editing();
                                        }
                                    }

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
                                } else {
                                    // Clic dans le vide -> valider l'édition en cours
                                    self.commit_editing();
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
                            if let Some(dismissed) = self.dock_manager.finish_drag() {
                                self.ui.show_toast(format!("👋 Panneau {} fermé", dismissed.title()));
                                self.redraw();
                                return;
                            }
                            if !self.right_or_middle_down {
                                self.is_panning = false;
                            }
                            if self.is_dragging_item {
                                self.is_dragging_item = false;
                                self.store.end_live_edit();
                                self.active_guides = SnapGuides::default();
                            }
                            if let Some((x1, y1, x2, y2)) = self.selection_box.take() {
                                let sx_min = x1.min(x2);
                                let sx_max = x1.max(x2);
                                let sy_min = y1.min(y2);
                                let sy_max = y1.max(y2);

                                if (sx_max - sx_min) > 3.0 || (sy_max - sy_min) > 3.0 {
                                    let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
                                    let (wx1, wy1) = screen_to_world(sx_min, sy_min, &vp);
                                    let (wx2, wy2) = screen_to_world(sx_max, sy_max, &vp);
                                    let box_left = wx1.min(wx2);
                                    let box_right = wx1.max(wx2);
                                    let box_top = wy1.min(wy2);
                                    let box_bottom = wy1.max(wy2);

                                    let mut hits_imgs = Vec::new();
                                    let mut hits_anns = Vec::new();

                                    if let Some(b) = self.store.active_board() {
                                        for img in &b.images {
                                            let il = img.x - img.width / 2.0;
                                            let ir = img.x + img.width / 2.0;
                                            let it = img.y - img.height / 2.0;
                                            let ib = img.y + img.height / 2.0;
                                            if ir >= box_left && il <= box_right && ib >= box_top && it <= box_bottom {
                                                hits_imgs.push(img.id.clone());
                                            }
                                        }
                                        for ann in &b.annotations {
                                            let (al, ar, at, ab) = match ann {
                                                Annotation::Arrow { x, y, x2, y2, .. } => {
                                                    (x.min(*x2), x.max(*x2), y.min(*y2), y.max(*y2))
                                                }
                                                Annotation::Text { x, y, width, height, .. } => {
                                                    let w = width.unwrap_or(240.0);
                                                    let h = height.unwrap_or(48.0);
                                                    (*x, *x + w, *y, *y + h)
                                                }
                                                Annotation::Sticky { x, y, width, height, .. } => {
                                                    let w = width.unwrap_or(180.0);
                                                    let h = height.unwrap_or(130.0);
                                                    (*x, *x + w, *y, *y + h)
                                                }
                                                Annotation::Membrane { x, y, width, height, .. } => {
                                                    (*x, *x + *width, *y, *y + *height)
                                                }
                                            };
                                            if ar >= box_left && al <= box_right && ab >= box_top && at <= box_bottom {
                                                hits_anns.push(ann.id().to_string());
                                            }
                                        }
                                    }

                                    if !self.modifiers.shift_key() {
                                        self.store.clear_selection();
                                    }
                                    for id in hits_imgs {
                                        self.store.select_image(id, true);
                                    }
                                    for id in hits_anns {
                                        self.store.select_annotation(id, true);
                                    }
                                }
                            }
                            self.redraw();
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Interception prioritaire si une session d'édition de texte est active
                if let Some(ref mut session) = self.editing_session {
                    if event.state == ElementState::Pressed {
                        match event.logical_key {
                            Key::Named(NamedKey::Escape) => {
                                self.editing_session = None;
                                self.redraw();
                                return;
                            }
                            Key::Named(NamedKey::Enter) => {
                                if self.modifiers.control_key() {
                                    self.commit_editing();
                                    self.redraw();
                                    return;
                                } else {
                                    session.buffer.insert(session.cursor_idx, '\n');
                                    session.cursor_idx += 1;
                                    session.blink_timer = std::time::Instant::now();
                                    self.redraw();
                                    return;
                                }
                            }
                            Key::Named(NamedKey::Backspace) => {
                                if session.cursor_idx > 0 {
                                    let mut prev = session.cursor_idx - 1;
                                    while prev > 0 && !session.buffer.is_char_boundary(prev) {
                                        prev -= 1;
                                    }
                                    session.buffer.drain(prev..session.cursor_idx);
                                    session.cursor_idx = prev;
                                    session.blink_timer = std::time::Instant::now();
                                    self.redraw();
                                    return;
                                }
                            }
                            Key::Named(NamedKey::Delete) => {
                                if session.cursor_idx < session.buffer.len() {
                                    let mut next = session.cursor_idx + 1;
                                    while next < session.buffer.len() && !session.buffer.is_char_boundary(next) {
                                        next += 1;
                                    }
                                    session.buffer.drain(session.cursor_idx..next);
                                    session.blink_timer = std::time::Instant::now();
                                    self.redraw();
                                    return;
                                }
                            }
                            Key::Named(NamedKey::ArrowLeft) => {
                                if session.cursor_idx > 0 {
                                    let mut prev = session.cursor_idx - 1;
                                    while prev > 0 && !session.buffer.is_char_boundary(prev) {
                                        prev -= 1;
                                    }
                                    session.cursor_idx = prev;
                                    session.blink_timer = std::time::Instant::now();
                                    self.redraw();
                                    return;
                                }
                            }
                            Key::Named(NamedKey::ArrowRight) => {
                                if session.cursor_idx < session.buffer.len() {
                                    let mut next = session.cursor_idx + 1;
                                    while next < session.buffer.len() && !session.buffer.is_char_boundary(next) {
                                        next += 1;
                                    }
                                    session.cursor_idx = next;
                                    session.blink_timer = std::time::Instant::now();
                                    self.redraw();
                                    return;
                                }
                            }
                            Key::Named(NamedKey::Home) => {
                                session.cursor_idx = 0;
                                session.blink_timer = std::time::Instant::now();
                                self.redraw();
                                return;
                            }
                            Key::Named(NamedKey::End) => {
                                session.cursor_idx = session.buffer.len();
                                session.blink_timer = std::time::Instant::now();
                                self.redraw();
                                return;
                            }
                            Key::Character(ref c) => {
                                if !self.modifiers.control_key() && !self.modifiers.alt_key() {
                                    session.buffer.insert_str(session.cursor_idx, c.as_str());
                                    session.cursor_idx += c.len();
                                    session.blink_timer = std::time::Instant::now();
                                    self.redraw();
                                    return;
                                }
                            }
                            _ => {}
                        }
                    }
                    return;
                }

                if event.logical_key == Key::Named(NamedKey::Space) {
                    self.space_pressed = event.state == ElementState::Pressed;
                    if !self.space_pressed && !self.right_or_middle_down && self.ui.active_tool != ActiveTool::Pan {
                        self.is_panning = false;
                    }
                    self.redraw();
                    return;
                }

                if event.state == ElementState::Pressed {
                    let ctrl = self.modifiers.control_key();
                    let active_bid = self.store.project.active_board_id.clone();

                    match event.logical_key {
                        Key::Named(NamedKey::Delete) | Key::Named(NamedKey::Backspace) => {
                            self.store.delete_selected(&active_bid);
                            self.ui.show_toast("🗑 Supprimé");
                            self.redraw();
                        }
                        Key::Character(c) => match c.as_str() {
                            "h" | "H" => {
                                if !ctrl {
                                    self.ui.active_tool = ActiveTool::Pan;
                                    self.redraw();
                                }
                            }
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

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.dock_manager.pomodoro.running {
            if self.dock_manager.tick_pomodoro() {
                self.redraw();
            }
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(200),
            ));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}
