//! Application Glucose Desktop — Event Loop Winit 0.30 et Framebuffer Softbuffer 0.4.

use crate::canvas::{screen_to_world, zoom_at};
use crate::rasterizer::FrameBuffer;
use crate::renderer::Renderer;
use glucose_core::hit_priority::{collect_candidates, PickInput, PickOwner};
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::BoardImage;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId, WindowLevel};

pub struct GlucoseApp {
    pub store: Store,
    pub renderer: Renderer,
    pub framebuffer: FrameBuffer,
    pub window: Option<Arc<Window>>,
    pub context: Option<softbuffer::Context<Arc<Window>>>,
    pub surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,

    // États d'interaction
    pub mouse_pos: (f64, f64),
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
        // Board d'accueil par défaut
        let welcome_text = glucose_core::types::Annotation::Text {
            id: "welcome-txt".into(),
            x: 0.0,
            y: 0.0,
            width: Some(380.0),
            height: Some(160.0),
            text: "# Glucose Desktop (Rust PureRef)\n\nGlissez-deposez vos images ici !\nMolette: Zoom | Clic-droit/Milieu: Pan\nDel: Suppr | T: Toujours au-dessus".into(),
            font_size: Some(15.0),
            color: Some("#38bdf8".into()),
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        };
        store.add_annotation("main", welcome_text);

        Self {
            store,
            renderer: Renderer::new(),
            framebuffer: FrameBuffer::new(1280, 720),
            window: None,
            context: None,
            surface: None,
            mouse_pos: (0.0, 0.0),
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
            self.framebuffer.resize(width as usize, height as usize);

            self.renderer.render(
                &mut self.framebuffer,
                &self.store,
                &self.active_guides,
                self.selection_box,
                self.always_on_top,
            );

            if let Ok(mut buffer) = surface.buffer_mut() {
                buffer.copy_from_slice(&self.framebuffer.pixels);
                let _ = buffer.present();
            }
        }
    }
}

impl ApplicationHandler for GlucoseApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = WindowAttributes::default()
                .with_title("Glucose — PureRef Native Rust")
                .with_inner_size(LogicalSize::new(1280.0, 720.0));

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
                        self.framebuffer.resize(width as usize, height as usize);
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
                self.framebuffer.resize(width as usize, height as usize);
                self.redraw();
            }
            WindowEvent::RedrawRequested => {
                self.redraw();
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
                    let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
                    let (wx, wy) = screen_to_world(position.x, position.y, &vp);
                    let w_dx = wx - self.drag_start_world.0;
                    let w_dy = wy - self.drag_start_world.1;

                    self.store.move_selected("main", w_dx, w_dy);
                    self.drag_start_world = (wx, wy);
                    self.redraw();
                } else if let Some((bx1, by1, _, _)) = self.selection_box {
                    self.selection_box = Some((bx1, by1, position.x, position.y));
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
                match button {
                    MouseButton::Right | MouseButton::Middle => {
                        self.is_panning = state == ElementState::Pressed;
                    }
                    MouseButton::Left => {
                        if state == ElementState::Pressed {
                            let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
                            let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

                            // Test de sélection via hit_priority (PICK-1)
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
                                            self.store.select_image(top.id.clone(), false);
                                            selected = true;
                                        }
                                        PickOwner::Annotation | PickOwner::Membrane | PickOwner::Arrow => {
                                            self.store.select_annotation(top.id.clone(), false);
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
                                // Clic sur le fond : démarre la boîte de sélection élastique
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
                    match event.logical_key {
                        Key::Named(NamedKey::Delete) | Key::Named(NamedKey::Backspace) => {
                            self.store.delete_selected("main");
                            self.redraw();
                        }
                        Key::Character(c) => match c.as_str() {
                            "t" | "T" => {
                                // Toggle Always On Top (PureRef standard)
                                self.always_on_top = !self.always_on_top;
                                if let Some(w) = &self.window {
                                    w.set_window_level(if self.always_on_top {
                                        WindowLevel::AlwaysOnTop
                                    } else {
                                        WindowLevel::Normal
                                    });
                                }
                                self.redraw();
                            }
                            "f" | "F" | " " => {
                                // Fit view / recentrer
                                if let Some(b) = self.store.active_board_mut() {
                                    b.viewport.x = 0.0;
                                    b.viewport.y = 0.0;
                                    b.viewport.scale = 1.0;
                                }
                                self.redraw();
                            }
                            "z" | "Z" => {
                                self.store.undo();
                                self.redraw();
                            }
                            "y" | "Y" => {
                                self.store.redo();
                                self.redraw();
                            }
                            "d" | "D" => {
                                self.store.duplicate_selected("main");
                                self.redraw();
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            WindowEvent::DroppedFile(path_buf) => {
                // Drop d'image OS sur le canvas (PureRef drop handler)
                if let Some(path_str) = path_buf.to_str() {
                    let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
                    let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

                    let (w, h) = if let Ok(dyn_img) = image::open(&path_buf) {
                        (dyn_img.width() as f64, dyn_img.height() as f64)
                    } else {
                        (200.0, 150.0)
                    };

                    let mut img = BoardImage::new(
                        format!("img-{}", self.store.active_board().map(|b| b.images.len()).unwrap_or(0)),
                        wx,
                        wy,
                        w.min(600.0),
                        h * (w.min(600.0) / w.max(1.0)),
                    );
                    img.src = Some(path_str.to_string());
                    self.store.add_image("main", img);
                    self.redraw();
                }
            }
            _ => {}
        }
    }
}
