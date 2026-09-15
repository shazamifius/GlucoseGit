//! Application Glucose Desktop — Event Loop Winit 0.30 et Framebuffer Softbuffer 0.4.

use crate::dock::{apply_organize_layout, render_docks, DockManager, OrganizeState};
use crate::error::{DesktopError, DesktopResult};
use crate::interactions::resize::ResizeSession;
use crate::interactions::tools::text_card;
use crate::params::{Pointer, SceneOverlay, ScreenFrame};
use crate::renderer::{Renderer, TextEditSession};
use crate::ui::{ToastRepaint, UiState};
use glucose_core::hit_priority::CycleState;
use glucose_core::smart_align::{AlignRect, AlignTarget, SnapGuides};
use glucose_core::store::Store;
use std::num::NonZeroU32;
use std::sync::Arc;
use tiny_skia::Pixmap;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowAttributes, WindowId};

/// Ce que dit la carte d'accueil d'un document neuf.
const WELCOME_TEXT: &str = "# Bienvenue dans Glucose !\n- 100% Rust ultra-rapide\n- Teintes symbiotiques dynamiques\n- Double-cliquez pour éditer";

/// Cadence minimale d'une animation d'interface (~60 Hz).
const ANIMATION_MIN_INTERVAL_MS: u64 = 16;
/// Cadence minimale de repli quand une frame est anormalement lente.
const ANIMATION_MAX_INTERVAL_MS: u64 = 250;

pub struct LastClickInfo {
    /// L'instant du clic, en millisecondes depuis [`GlucoseApp::click_epoch`].
    ///
    /// Une date entière et non un `Instant` : le compte des clics rapprochés et le cycle de
    /// profondeur posent la **même** question — « ces deux clics se suivent-ils ? » — et la
    /// posaient à deux horloges différentes, l'une réelle, l'autre en millisecondes. Deux
    /// horloges pour une question, c'est une divergence qui attend son bug.
    pub at_ms: i64,
    pub pos: (f64, f64),
    pub id: String,
    /// Le nombre de clics rapprochés sur ce même nœud : 1, puis 2, puis 3. C'est lui qui
    /// décide de la maille d'une sélection de texte (MOUSE-1), et le compter ici plutôt que
    /// dans le texte évite qu'un double-clic sur une carte et un double-clic sur un dossier
    /// aient deux définitions de « rapproché ».
    pub count: u32,
}

pub struct GlucoseApp {
    pub store: Store,
    pub renderer: Renderer,
    /// Les animations en cours — pour l'instant, le vol de la caméra.
    pub animator: crate::animation::Animator,
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
    pub right_or_middle_down: bool,
    /// Où le clic droit s'est enfoncé, tant qu'il l'est.
    ///
    /// C'est ce qui permet de savoir, **au relâchement**, si le geste était un déplacement de
    /// la vue ou une demande de menu contextuel : la seule différence est que le curseur a
    /// bougé, ou non.
    pub right_down_at: Option<(f64, f64)>,
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
    /// Le glisser de sélection de texte en cours, s'il y en a un (MOUSE-1).
    pub text_drag: Option<crate::interactions::text_mouse::TextDrag>,
    pub last_click: Option<LastClickInfo>,
    /// Où en est le cycle de profondeur (PICK-1) : la pile visée au dernier clic, et le rang
    /// qu'on y a atteint. `None` quand le dernier clic n'a désigné aucun nœud, ou qu'il a
    /// fait autre chose que sélectionner — ouvrir, éditer, glisser.
    pub pick_cycle: Option<CycleState>,
    /// L'origine du temps des clics — celle que [`GlucoseApp::now_ms`] mesure.
    ///
    /// Un `Instant` ne se soustrait pas à un entier, et l'arbitre de clic raisonne en
    /// millisecondes. C'est aussi ce qui rend les gestes testables **sans dormir** : un test
    /// qui veut jouer un re-clic « une seconde plus tard » recule cette origine, au lieu
    /// d'attendre vraiment.
    pub click_epoch: std::time::Instant,
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

/// `new` ne prend aucun argument : `Default` est donc exactement le même constructeur.
/// Le déclarer évite qu'un appelant générique ait à connaître le nom `new`.
impl Default for GlucoseApp {
    fn default() -> Self {
        Self::new()
    }
}

impl GlucoseApp {
    pub fn new() -> Self {
        let mut store = Store::new("Glucose Native");
        let active_bid = store.project.active_board_id.clone();
        let renderer = Renderer::new();

        // La carte d'accueil naît par la même fabrique qu'une carte posée d'un clic : même
        // largeur de naissance, même hauteur suivie (TEXT-FIT-1).
        let welcome = text_card(
            &renderer.typography,
            &renderer.math,
            "welcome-card",
            0.0,
            0.0,
            WELCOME_TEXT,
        );
        store.add_annotation(&active_bid, welcome);
        // La carte d'accueil n'est pas une modification de l'utilisateur : le document part
        // propre, sans marqueur dans le titre — et sans rien à défaire. Tant que seul le
        // marqueur était traité, Ctrl+Z était actif dès le lancement et retirait une carte
        // que personne n'avait posée.
        store.journal.clear();
        let saved_version = store.version;

        Self {
            store,
            renderer,
            animator: crate::animation::Animator::new(),
            pixmap: None,
            // Le mot d'accueil est posé ici, au démarrage, et non dans `UiState::new` : un
            // constructeur d'état ne déclenche pas de notification, et un toast porte une
            // horloge qui rendait tout rendu non reproductible.
            ui: {
                let mut ui = UiState::new();
                ui.show_toast(crate::ui::WELCOME_TOAST);
                ui
            },
            dock_manager: DockManager::new(),
            window: None,
            context: None,
            surface: None,
            scale_factor: 1.0,
            mouse_pos: (0.0, 0.0),
            modifiers: ModifiersState::empty(),
            right_or_middle_down: false,
            right_down_at: None,
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
            text_drag: None,
            last_click: None,
            pick_cycle: None,
            click_epoch: std::time::Instant::now(),
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
                let pointer = Pointer {
                    x: self.mouse_pos.0 as f32,
                    y: self.mouse_pos.1 as f32,
                };
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
            self.last_frame_ms = frame_started
                .elapsed()
                .as_millis()
                .min(u128::from(u64::MAX)) as u64;
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
        if self
            .store
            .active_board()
            .is_some_and(|b| b.images.is_empty())
        {
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
        self.ui
            .show_toast(format!("Disposition {} appliquée", state.layout.title()));
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
/// Un pixel de `tiny-skia` dans le format que la fenêtre attend.
///
/// # Une opération par pixel plutôt que six
///
/// La version précédente lisait trois octets et les recomposait à coups de décalages et de
/// `ou`. Celle-ci dit la même chose en une fois : un pixel vaut `r,g,b,a` en mémoire, donc
/// `a<<24 | b<<16 | g<<8 | r` lu comme un mot ; l'échanger bout à bout donne
/// `r<<24 | g<<16 | b<<8 | a`, et un décalage de huit bits laisse exactement `r<<16 | g<<8 | b`.
/// Le processeur a une instruction pour l'échange d'octets, et le compilateur peut la
/// vectoriser — ce qu'une recomposition octet par octet lui interdit.
///
/// Mesuré : **7,83 → 5,47 ms en 4K**, 1,58 → 1,06 ms en 1080p. Ce coût est payé à **chaque**
/// image, quoi qu'il y ait à l'écran : c'est un coût de surface, le seul que ni le culling ni
/// aucun cache ne réduira (fiche 13, vague B).
fn pixel_fenetre(px: [u8; 4]) -> u32 {
    u32::from_le_bytes(px).swap_bytes() >> 8
}

fn blit_and_present(
    surface: &mut softbuffer::Surface<Arc<Window>, Arc<Window>>,
    pixmap: &Pixmap,
) -> DesktopResult<()> {
    let mut buffer = surface
        .buffer_mut()
        .map_err(|e| DesktopError::WindowError(format!("buffer_mut : {e}")))?;
    let (src, _) = pixmap.data().as_chunks::<4>();
    for (dst, chunk) in buffer.iter_mut().zip(src) {
        *dst = pixel_fenetre(*chunk);
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

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
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
                self.handle_key(&event);
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

        // 2. Toast actif : c'est lui qui sait s'il faut redessiner (fondu), attendre
        // (plateau, alpha = 1, aucun rafraîchissement) ou disparaître.
        if let Some(ref toast) = self.ui.current_toast {
            match toast.repaint_need() {
                ToastRepaint::Gone => {
                    self.ui.current_toast = None;
                    self.mark_dirty();
                }
                ToastRepaint::Redraw => {
                    self.mark_dirty();
                    min_timeout_ms = min_timeout_ms.min(self.animation_interval_ms());
                    has_timer = true;
                }
                ToastRepaint::Sleep(wait_ms) => {
                    min_timeout_ms = min_timeout_ms.min(wait_ms);
                    has_timer = true;
                }
            }
        }

        // 2 bis. Vol de la caméra : chaque image avance le viewport, et l'animation dit
        // elle-même dans combien de temps la suivante est due.
        if let Some(reste_ms) = self.animator.tick(&mut self.store) {
            self.mark_dirty();
            min_timeout_ms = min_timeout_ms.min(reste_ms.max(1).min(self.animation_interval_ms()));
            has_timer = true;
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
            let next_deadline =
                std::time::Instant::now() + std::time::Duration::from_millis(min_timeout_ms);
            event_loop.set_control_flow(ControlFlow::WaitUntil(next_deadline));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pixel_fenetre;

    /// La conversion rapide rend **exactement** ce que la recomposition octet par octet rendait.
    ///
    /// Une optimisation qui change la couleur d'un pixel n'est pas une optimisation : c'est un
    /// défaut plus rapide. Le test parcourt chaque valeur possible sur chaque canal, l'alpha
    /// compris — il n'échantillonne pas, il démontre.
    #[test]
    fn test_the_fast_conversion_is_the_same_pixel() {
        for v in 0..=255u8 {
            for (i, canal) in [
                [v, 0, 0, 255],
                [0, v, 0, 255],
                [0, 0, v, 255],
                [7, 9, 11, v],
            ]
            .into_iter()
            .enumerate()
            {
                let attendu =
                    (u32::from(canal[0]) << 16) | (u32::from(canal[1]) << 8) | u32::from(canal[2]);
                assert_eq!(
                    pixel_fenetre(canal),
                    attendu,
                    "canal {i}, valeur {v} : {canal:?}"
                );
            }
        }
    }

    /// L'alpha ne doit **jamais** atteindre la fenêtre : elle attend `0RGB`, et un octet de
    /// poids fort non nul y serait lu comme une couleur.
    #[test]
    fn test_the_alpha_never_reaches_the_window() {
        for a in 0..=255u8 {
            assert_eq!(pixel_fenetre([1, 2, 3, a]) >> 24, 0, "alpha {a} a fuité");
        }
    }
}
