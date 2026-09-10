//! Application Glucose Desktop — Event Loop Winit 0.30 et Framebuffer Softbuffer 0.4.

use crate::dock::{apply_organize_layout, render_docks, DockManager, OrganizeState};
use crate::renderer::{Renderer, TextEditSession};
use crate::ui::{UiState, TOTAL_HEADER_HEIGHT};
use glucose_core::smart_align::{AlignRect, AlignTarget, SnapGuides};
use glucose_core::store::Store;
use glucose_core::types::Annotation;
use std::num::NonZeroU32;
use std::sync::Arc;
use tiny_skia::Pixmap;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowAttributes, WindowId};

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
    pub scale_factor: f64,

    // États d'interaction
    pub mouse_pos: (f64, f64),
    pub modifiers: ModifiersState,
    pub space_pressed: bool,
    pub right_or_middle_down: bool,
    pub is_panning: bool,
    pub is_dragging_item: bool,
    pub drag_start_world: (f64, f64),
    pub drag_selection_base: Option<AlignRect>,
    pub drag_snap_targets: Vec<AlignTarget>,
    pub drag_applied_delta: (f64, f64),
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
            scale_factor: 1.0,
            mouse_pos: (0.0, 0.0),
            modifiers: ModifiersState::empty(),
            space_pressed: false,
            right_or_middle_down: false,
            is_panning: false,
            is_dragging_item: false,
            drag_start_world: (0.0, 0.0),
            drag_selection_base: None,
            drag_snap_targets: Vec::new(),
            drag_applied_delta: (0.0, 0.0),
            active_guides: SnapGuides::default(),
            selection_box: None,
            always_on_top: false,
            editing_session: None,
            last_click: None,
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

    /// Réorganise automatiquement les éléments en grille ordonnée
    #[allow(dead_code)]
    pub fn organize_layout(&mut self) {
        if let Some(board) = self.store.active_board_mut() {
            if board.images.is_empty() && board.annotations.is_empty() {
                return;
            }
            glucose_core::layout::organize_board_grid(board, 40.0);
        }
        self.store.push_undo();
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
                let scale_factor = window.scale_factor();
                self.scale_factor = scale_factor;
                self.ui.scale_factor = scale_factor as f32;

                if let Ok(context) = softbuffer::Context::new(window.clone()) {
                    if let Ok(mut surface) = softbuffer::Surface::new(&context, window.clone()) {
                        let size = window.inner_size();
                        let width = size.width.max(1);
                        let height = size.height.max(1);
                        if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
                            let _ = surface.resize(w, h);
                        }
                        self.pixmap = Pixmap::new(width, height);
                        self.window = Some(window.clone());
                        window.set_cursor(winit::window::CursorIcon::Grab);
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
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.ui.scale_factor = scale_factor as f32;
                self.redraw();
            }
            WindowEvent::RedrawRequested => {
                self.redraw();
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_cursor_moved(position);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(delta);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                let (screen_w, screen_h) = if let Some(w) = &self.window {
                    let sz = w.inner_size();
                    (sz.width as f32, sz.height as f32)
                } else {
                    (1280.0, 720.0)
                };
                match state {
                    ElementState::Pressed => self.handle_mouse_down(button, screen_w, screen_h),
                    ElementState::Released => self.handle_mouse_up(button),
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !self.handle_text_key(&event) {
                    self.handle_keyboard_shortcut(&event);
                }
            }
            WindowEvent::DroppedFile(path_buf) => {
                self.import_image_files(&[path_buf]);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let mut need_anim = false;
        let mut min_timeout_ms = 1000u64;

        // 1. Clignotement du curseur d'édition de texte (période 500 ms)
        if self.editing_session.is_some() {
            need_anim = true;
            min_timeout_ms = min_timeout_ms.min(100);
        }

        // 2. Toasts actifs (décompte d'affichage et animation de fondu)
        if self.ui.current_toast.is_some() {
            need_anim = true;
            min_timeout_ms = min_timeout_ms.min(30);
        }

        // 3. Minuteur Pomodoro actif dans le dock
        if self.dock_manager.pomodoro.running {
            if self.dock_manager.tick_pomodoro() {
                self.redraw();
            }
            need_anim = true;
            min_timeout_ms = min_timeout_ms.min(200);
        }

        if need_anim {
            let next_deadline = std::time::Instant::now() + std::time::Duration::from_millis(min_timeout_ms);
            event_loop.set_control_flow(ControlFlow::WaitUntil(next_deadline));
            self.redraw();
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}
