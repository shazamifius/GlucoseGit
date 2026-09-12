//! Application Glucose Desktop — Event Loop Winit 0.30 et Framebuffer Softbuffer 0.4.

use crate::dock::{apply_organize_layout, render_docks, DockManager, OrganizeState};
use crate::error::{DesktopError, DesktopResult};
use crate::interactions::resize::ResizeSession;
use crate::params::{Pointer, SceneOverlay, ScreenFrame};
use crate::renderer::card::text_card_fit_height;
use crate::renderer::{Renderer, TextEditSession};
use crate::ui::UiState;
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

/// Cadence minimale d'une animation d'interface (~60 Hz).
const ANIMATION_MIN_INTERVAL_MS: u64 = 16;
/// Cadence minimale de repli quand une frame est anormalement lente.
const ANIMATION_MAX_INTERVAL_MS: u64 = 250;

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
    /// Le redimensionnement en cours, s'il y en a un (RESIZE-1).
    pub resize_session: Option<ResizeSession>,
    pub active_guides: SnapGuides,
    pub selection_box: Option<(f64, f64, f64, f64)>,
    pub always_on_top: bool,

    // Session d'édition de texte in-place (double-clic)
    pub editing_session: Option<TextEditSession>,
    pub last_click: Option<LastClickInfo>,
    pub last_blink_phase: bool,
    /// Durée de la dernière frame présentée, en millisecondes.
    pub last_frame_ms: u64,

    /// Chemin du `.glucose` courant. `None` tant que le projet n'a jamais été enregistré :
    /// c'est ce qui fait que `Ctrl+S` ouvre un dialogue la première fois seulement.
    pub project_path: Option<std::path::PathBuf>,
    /// `store.version` au moment du dernier enregistrement ou de la dernière ouverture.
    ///
    /// INVARIANT SAVE-2 — « modifié » se lit `store.version != saved_version`. Aucun drapeau
    /// à lever dans chaque mutation, donc aucune mutation ne peut oublier de le lever : la
    /// pile d'undo fait déjà avancer la version, et elle seule (la navigation ne la touche pas).
    pub saved_version: u64,
    /// Dernier titre posé sur la fenêtre, pour ne pas repayer un appel système par frame.
    pub window_title_cache: String,
}

impl GlucoseApp {
    pub fn new() -> Self {
        let mut store = Store::new("Glucose Native");
        let active_bid = store.project.active_board_id.clone();
        let renderer = Renderer::new();

        // Carte d'accueil par défaut au look Glucose moderne. Sa hauteur est celle de son
        // texte à sa largeur (TEXT-FIT-1) : la boîte du document est celle de l'écran.
        let welcome_text = "# Bienvenue dans Glucose !\n- 100% Rust ultra-rapide\n- Teintes symbiotiques dynamiques\n- Double-cliquez pour éditer";
        let welcome_card = Annotation::Text {
            id: "welcome-card".into(),
            x: 0.0,
            y: 0.0,
            width: Some(260.0),
            height: Some(text_card_fit_height(&renderer.typography, welcome_text, 260.0)),
            text: welcome_text.into(),
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
        // La carte d'accueil n'est pas une modification de l'utilisateur : le document part
        // propre, sans marqueur dans le titre.
        let saved_version = store.version;

        Self {
            store,
            renderer,
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
            resize_session: None,
            active_guides: SnapGuides::default(),
            selection_box: None,
            always_on_top: false,
            editing_session: None,
            last_click: None,
            last_blink_phase: true,
            last_frame_ms: 0,
            project_path: None,
            saved_version,
            window_title_cache: String::new(),
        }
    }

    pub fn redraw(&mut self) {
        // Le marqueur « modifié » du titre suit l'état réel du document (INVARIANT SAVE-2).
        // Le poser ici plutôt que dans chaque mutation garantit qu'aucune ne l'oublie ;
        // `sync_window_title` ne touche la fenêtre que lorsque le titre change vraiment.
        self.sync_window_title();
        if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
            crate::perf::frame_begin();
            let frame_started = std::time::Instant::now();
            let size = window.inner_size();
            let width = size.width.max(1);
            let height = size.height.max(1);

            if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
                if let Err(e) = surface.resize(w, h) {
                    eprintln!("[GlucoseDesktop] surface.resize failed: {e}");
                }
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
                let pointer = Pointer { x: self.mouse_pos.0 as f32, y: self.mouse_pos.1 as f32 };
                self.renderer.render(
                    &mut pixmap_mut,
                    &self.store,
                    &mut self.ui,
                    SceneOverlay {
                        guides: &self.active_guides,
                        selection_box: self.selection_box,
                        editing: self.editing_session.as_ref(),
                    },
                    pointer,
                );

                // Rendu des panneaux déroulants & flottants (Top & Bottom Docks).
                // `scale` et les coordonnées de la souris sont désormais portés par
                // deux types distincts : les intervertir ne compile plus (R-44).
                render_docks(
                    &mut pixmap_mut,
                    &self.dock_manager,
                    &self.store,
                    &self.renderer.typography,
                    &self.renderer.theme,
                    ScreenFrame {
                        width: width as f32,
                        height: height as f32,
                        header_h: self.ui.header_height(),
                        scale: self.ui.scale_factor,
                    },
                    pointer,
                );
                crate::perf::stage("docks");

                if let Err(e) = blit_and_present(surface, pixmap) {
                    eprintln!("[GlucoseDesktop] présentation du framebuffer impossible : {e}");
                }
            }
            self.last_frame_ms = frame_started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            crate::perf::frame_end();
        }
    }

    /// Intervalle minimal entre deux frames animées.
    ///
    /// On ne demande jamais un rafraîchissement plus vite que la durée réelle de
    /// la dernière frame : sur une machine lente, une cadence fixe de 16 ms
    /// remplirait la file d'événements plus vite qu'elle ne se vide et priverait
    /// la pompe de messages de l'OS de temps de traitement.
    fn animation_interval_ms(&self) -> u64 {
        self.last_frame_ms
            .clamp(ANIMATION_MIN_INTERVAL_MS, ANIMATION_MAX_INTERVAL_MS)
    }

    /// Marque la vue comme sale et planifie un rafraîchissement asynchrone coalescé par Winit (Roadmap 1.11, R-15).
    pub fn mark_dirty(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// Réorganise automatiquement les éléments en grille ordonnée
    #[allow(dead_code)]
    pub fn organize_layout(&mut self) {
        let board_id = self.store.project.active_board_id.clone();
        let vide = self
            .store
            .active_board()
            .is_some_and(|b| b.images.is_empty() && b.annotations.is_empty());
        if vide {
            return;
        }
        // `push_undo` etait appele APRES la mise en page : le cliche capturait l'etat deja
        // modifie, et Ctrl+Z ne defaisait rien. La consigne se fait desormais autour du
        // geste, pas apres lui.
        self.store.mutate_board_layout(&board_id, |board| {
            glucose_core::layout::organize_board_grid(board, 40.0);
        });
        self.ui.show_toast("Canvas ordonné");
        self.mark_dirty();
    }

    /// Applique la réorganisation issue du panneau ORDONNER (Masonry, Grille, Même Hauteur, etc.)
    pub fn apply_dock_layout(&mut self, state: &OrganizeState) {
        let board_id = self.store.project.active_board_id.clone();
        if self.store.active_board().is_some_and(|b| b.images.is_empty()) {
            self.ui.show_toast("Aucune image sur le canvas");
            return;
        }
        // Meme correction que `organize_layout` : le cliche etait pris apres coup.
        self.store.mutate_board_layout(&board_id, |board| {
            for res in apply_organize_layout(&board.images, state) {
                if let Some(img) = board.images.iter_mut().find(|i| i.id == res.id) {
                    img.x = res.x;
                    img.y = res.y;
                    img.width = res.width;
                    img.height = res.height;
                }
            }
        });
        self.ui.show_toast(format!("Disposition {} appliquée", state.layout.title()));
        self.mark_dirty();
    }

    /// Crée la fenêtre et son framebuffer softbuffer ; toute erreur est propagée
    /// au lieu d'être avalée silencieusement (une fenêtre blanche sinon).
    fn init_window(&mut self, event_loop: &ActiveEventLoop) -> DesktopResult<()> {
        let title = self.window_title();
        let attrs = WindowAttributes::default()
            .with_title(&title)
            .with_inner_size(LogicalSize::new(1440.0, 900.0));

        let window = event_loop
            .create_window(attrs)
            .map(Arc::new)
            .map_err(|e| DesktopError::WindowError(format!("create_window : {e}")))?;

        let scale_factor = window.scale_factor();
        self.scale_factor = scale_factor;
        self.ui.scale_factor = scale_factor as f32;

        let context = softbuffer::Context::new(window.clone())
            .map_err(|e| DesktopError::WindowError(format!("softbuffer::Context : {e}")))?;
        let mut surface = softbuffer::Surface::new(&context, window.clone())
            .map_err(|e| DesktopError::WindowError(format!("softbuffer::Surface : {e}")))?;

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
            surface
                .resize(w, h)
                .map_err(|e| DesktopError::WindowError(format!("surface.resize : {e}")))?;
        }

        self.pixmap = Pixmap::new(width, height);
        window.set_cursor(winit::window::CursorIcon::Grab);
        self.window_title_cache = title;
        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        self.mark_dirty();
        Ok(())
    }
}

/// Recopie le pixmap tiny-skia (RGBA prémultiplié) dans le framebuffer
/// softbuffer (0RGB 32 bits) puis présente la frame.
fn blit_and_present(
    surface: &mut softbuffer::Surface<Arc<Window>, Arc<Window>>,
    pixmap: &Pixmap,
) -> DesktopResult<()> {
    let mut buffer = surface
        .buffer_mut()
        .map_err(|e| DesktopError::WindowError(format!("buffer_mut : {e}")))?;
    let (src, _) = pixmap.data().as_chunks::<4>();
    for (dst, chunk) in buffer.iter_mut().zip(src) {
        *dst = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
    }
    crate::perf::stage("blit");
    buffer
        .present()
        .map_err(|e| DesktopError::WindowError(format!("present : {e}")))?;
    crate::perf::stage("present");
    Ok(())
}

impl ApplicationHandler for GlucoseApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let started = std::time::Instant::now();
        if let Err(e) = self.init_window(event_loop) {
            eprintln!("[GlucoseDesktop] initialisation de la fenêtre impossible : {e}");
            event_loop.exit();
            return;
        }
        crate::perf::event("resumed", started);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                // R-48 — la croix ne jette plus le travail : un document modifié pose la
                // question, et un enregistrement raté annule la fermeture (SAVE-3).
                if self.request_close() {
                    event_loop.exit();
                } else {
                    self.mark_dirty();
                }
            }
            WindowEvent::Resized(size) => {
                let width = size.width.max(1);
                let height = size.height.max(1);
                if let Some(surface) = &mut self.surface {
                    if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
                        if let Err(e) = surface.resize(w, h) {
                            eprintln!("[GlucoseDesktop] surface.resize failed: {e}");
                        }
                    }
                }
                self.pixmap = Pixmap::new(width, height);
                self.mark_dirty();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.ui.scale_factor = scale_factor as f32;
                self.mark_dirty();
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
                // Trois preneurs, dans l'ordre : la saisie d'un nom de domaine, l'édition
                // d'une annotation, puis les raccourcis globaux. Chacun rend `false` quand la
                // touche ne le concerne pas ; aucun ne contient de logique (§ 1.7).
                if !self.handle_domain_rename_key(&event) && !self.handle_text_key(&event) {
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
        let mut has_timer = false;
        let mut min_timeout_ms = 1000u64;

        // 1. Clignotement du curseur d'édition de texte (période 500 ms)
        if let Some(session) = &self.editing_session {
            let elapsed = session.blink_timer.elapsed().as_millis();
            let phase = (elapsed / 500) % 2 == 0;
            if phase != self.last_blink_phase {
                self.last_blink_phase = phase;
                self.mark_dirty();
            }
            let remaining = 500 - (elapsed % 500);
            min_timeout_ms = min_timeout_ms.min(remaining.max(1) as u64);
            has_timer = true;
        } else {
            self.last_blink_phase = true;
        }

        // 2. Toasts actifs (décompte d'affichage et animation de fondu)
        if let Some(ref toast) = self.ui.current_toast {
            let elapsed = toast.created_at.elapsed().as_millis() as f32;
            let total = toast.duration.as_millis() as f32;

            if toast.is_expired() {
                self.ui.current_toast = None;
                self.mark_dirty();
            } else if elapsed < 150.0 {
                // Fondu entrant actif (150 ms) -> rafraîchissement doux
                self.mark_dirty();
                min_timeout_ms = min_timeout_ms.min(self.animation_interval_ms());
                has_timer = true;
            } else if elapsed < total - 400.0 {
                // Plateau statique (alpha = 1.0) : AUCUN rafraîchissement nécessaire !
                // On attend l'échéance du début de fondu sortant sans redessiner.
                let wait_ms = ((total - 400.0) - elapsed).ceil().max(1.0) as u64;
                min_timeout_ms = min_timeout_ms.min(wait_ms);
                has_timer = true;
            } else {
                // Fondu sortant actif (400 ms) -> rafraîchissement doux
                self.mark_dirty();
                min_timeout_ms = min_timeout_ms.min(self.animation_interval_ms());
                has_timer = true;
            }
        }

        // 3. Minuteur Pomodoro actif dans le dock
        if self.dock_manager.pomodoro.running {
            if self.dock_manager.tick_pomodoro() {
                self.mark_dirty();
            }
            let elapsed_ms = self.dock_manager.pomodoro.last_tick.elapsed().as_millis();
            let remaining_ms = 1000_u128.saturating_sub(elapsed_ms);
            min_timeout_ms = min_timeout_ms.min(remaining_ms.max(1) as u64);
            has_timer = true;
        }

        if has_timer {
            let next_deadline = std::time::Instant::now() + std::time::Duration::from_millis(min_timeout_ms);
            event_loop.set_control_flow(ControlFlow::WaitUntil(next_deadline));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}
