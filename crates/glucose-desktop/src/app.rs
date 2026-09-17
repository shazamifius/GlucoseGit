//! Application Glucose Desktop — Event Loop Winit 0.30 et Framebuffer Softbuffer 0.4.

mod reveil;

use crate::dock::{
    apply_organize_layout, render_docks, DockCache, DockManager, DockPass, OrganizeState,
};
use crate::error::{DesktopError, DesktopResult};
use crate::interactions::resize::ResizeSession;
use crate::interactions::tools::text_card;
use crate::params::{Pointer, SceneOverlay, ScreenFrame};
use crate::renderer::{Renderer, TextEditSession};
use crate::ui::UiState;
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
    /// Les tampons des panneaux du dock (DOCK-CACHE-1).
    ///
    /// Mesuré avant qu'il existe : sur un plateau vide, les panneaux étaient le premier poste
    /// de l'application — plus cher que tout le contenu réuni — et rendaient cent fois de
    /// suite exactement les mêmes octets.
    pub dock_cache: DockCache,
    pub window: Option<Arc<Window>>,
    /// Ce qui met l'image à l'écran — la carte graphique, ou le processeur à défaut.
    ///
    /// Derrière un `dyn` parce que le choix se fait au démarrage, une fois, en essayant : un
    /// adaptateur graphique peut manquer, et sur une machine virtuelle ou un bureau distant
    /// il manque souvent. L'appel indirect est payé une fois par image, contre les quelques
    /// millisecondes que la présentation elle-même coûte.
    pub presenter: Option<Box<dyn crate::present::Presenter>>,
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
    /// L'objet en train de naître sous la main, s'il y en a un (DRAW-1).
    pub draw_session: Option<crate::interactions::tools::DrawSession>,
    /// Le coude de flèche tenu sous la main, s'il y en a un (ARROW-3).
    pub bend_session: Option<crate::interactions::arrow_edit::BendSession>,
    pub active_guides: SnapGuides,
    pub selection_box: Option<(f64, f64, f64, f64)>,
    pub always_on_top: bool,

    // Session d'édition de texte in-place (double-clic)
    pub editing_session: Option<TextEditSession>,
    /// Le glisser de sélection de texte en cours, s'il y en a un (MOUSE-1).
    pub text_drag: Option<crate::interactions::text_mouse::TextDrag>,
    pub last_click: Option<LastClickInfo>,
    /// Les fichiers déposés sur la fenêtre, en attente d'être posés **ensemble**.
    ///
    /// winit émet un `DroppedFile` **par fichier** : un lot de huit donne huit événements,
    /// tous poussés par le même appel système et donc tous présents avant le prochain
    /// `about_to_wait`. Les accumuler jusque-là reconstitue le lot — sans quoi chaque
    /// fichier se poserait comme s'il était seul, tous au même point, et le geste entier
    /// laisserait huit entrées d'annulation au lieu d'une.
    pub dropped_files: Vec<std::path::PathBuf>,
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
    /// Ce qui a changé depuis la dernière image, et doit donc être redessiné (A.1).
    ///
    /// Dans une `Cell` pour que [`GlucoseApp::mark_dirty`] reste en `&self` : soixante-deux
    /// appelants la prennent ainsi, et leur imposer `&mut` pour noter une salissure aurait
    /// remonté l'emprunt à travers tout l'arbre des gestes. `Salissure` est `Copy`, donc la
    /// cellule ne coûte rien.
    salissure: std::cell::Cell<crate::salissure::Salissure>,
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
            dock_cache: DockCache::new(),
            window: None,
            presenter: None,
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
            draw_session: None,
            bend_session: None,
            active_guides: SnapGuides::default(),
            selection_box: None,
            always_on_top: false,
            editing_session: None,
            text_drag: None,
            last_click: None,
            dropped_files: Vec::new(),
            pick_cycle: None,
            click_epoch: std::time::Instant::now(),
            last_blink_phase: true,
            last_frame_ms: 0,
            project_path: None,
            saved_version,
            window_title_cache: String::new(),
            // Tout, et non rien : la première image doit se dessiner entièrement.
            salissure: std::cell::Cell::new(crate::salissure::Salissure::Tout),
        }
    }

    pub fn redraw(&mut self) {
        // Le marqueur « modifié » du titre suit l'état réel du document (INVARIANT SAVE-2).
        // Le poser ici plutôt que dans chaque mutation garantit qu'aucune ne l'oublie ;
        // `sync_window_title` ne touche la fenêtre que lorsque le titre change vraiment.
        self.sync_window_title();
        if let (Some(window), Some(presenter)) = (&self.window, &mut self.presenter) {
            crate::perf::frame_begin();
            let frame_started = std::time::Instant::now();
            let size = window.inner_size();
            let width = size.width.max(1);
            let height = size.height.max(1);

            if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
                if let Err(e) = presenter.resize(w, h) {
                    eprintln!("[GlucoseDesktop] redimensionnement de la surface : {e}");
                }
            }

            let need_new_pixmap = match &self.pixmap {
                Some(p) => p.width() != width || p.height() != height,
                None => true,
            };
            if need_new_pixmap {
                self.pixmap = Pixmap::new(width, height);
            }

            // La salissure est **consommée** : ce qui est redessiné maintenant cesse d'être
            // sale, et une nouvelle demande arrivée pendant le rendu appartient à l'image
            // suivante. Une image neuve part de `Tout`, jamais de `Rien` (A.1).
            let sale = self
                .salissure
                .replace(crate::salissure::Salissure::Rien);
            let sale = if need_new_pixmap {
                crate::salissure::Salissure::Tout
            } else {
                sale
            };

            let echelle = self.ui.scale_factor;
            if let Some(pixmap) = &mut self.pixmap {
                if !sale.est_propre() {
                    Self::peindre(
                        pixmap,
                        &mut self.renderer,
                        &self.store,
                        &mut self.ui,
                        &self.dock_manager,
                        &self.dock_cache,
                        SceneOverlay {
                            guides: &self.active_guides,
                            selection_box: self.selection_box,
                            editing: self.editing_session.as_ref(),
                        },
                        Pointer {
                            x: self.mouse_pos.0 as f32,
                            y: self.mouse_pos.1 as f32,
                        },
                        echelle,
                    );
                }
                // Combien d'images se sont contentées de reparaitre. Un nombre qui monte vite
                // dit que l'application se fait réveiller pour rien -- et c'est une question
                // qu'on ne pouvait pas poser avant que la salissure existe.
                crate::perf::compteur("img_evitee", f64::from(u8::from(sale.est_propre())));
                // On présente même quand rien n'a été redessiné : la demande peut venir du
                // système -- une fenêtre recouverte puis dégagée -- et non de nous.
                if let Err(e) = presenter.present(pixmap) {
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

    /// Peint la scène et la chrome dans le tampon.
    ///
    /// Extraite de `redraw` parce que celle-ci a désormais une décision à prendre avant de
    /// peindre -- y a-t-il seulement quelque chose à redessiner -- et qu'un ordonnanceur qui
    /// peint aussi finit par ne plus laisser voir la décision.
    #[allow(clippy::too_many_arguments)]
    fn peindre(
        pixmap: &mut Pixmap,
        renderer: &mut Renderer,
        store: &Store,
        ui: &mut UiState,
        dock_manager: &DockManager,
        dock_cache: &DockCache,
        overlay: SceneOverlay<'_>,
        pointer: Pointer,
        scale: f32,
    ) {
        let (width, height) = (pixmap.width(), pixmap.height());
        let mut vue = pixmap.as_mut();
        renderer.render(&mut vue, store, ui, overlay, pointer);

        // Rendu des panneaux déroulants & flottants (Top & Bottom Docks).
        // `scale` et les coordonnées de la souris sont désormais portés par
        // deux types distincts : les intervertir ne compile plus (R-44).
        render_docks(
            &mut vue,
            dock_manager,
            store,
            &DockPass {
                typo: &renderer.typography,
                theme: &renderer.theme,
                screen: ScreenFrame {
                    width: width as f32,
                    height: height as f32,
                    header_h: ui.header_height(),
                    scale,
                },
                pointer,
                cache: Some(dock_cache),
            },
        );
        crate::perf::stage("docks");
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

    /// Marque **toute** la vue comme sale et planifie un rafraîchissement (R-15).
    ///
    /// C'est la déclaration de celui qui ne sait pas ce qu'il a changé, et elle reste juste :
    /// redessiner l'écran entier coûte ce qu'il coûtait hier. Un geste qui sait désigner sa
    /// zone appelle [`GlucoseApp::salir`] et paie beaucoup moins.
    pub fn mark_dirty(&self) {
        self.salissure.set(crate::salissure::Salissure::Tout);
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// Marque cette zone du **monde** comme sale, et elle seule (A.1).
    ///
    /// Une zone précise ne peut jamais réduire une salissure déjà posée : si `Tout` a été
    /// demandé par ailleurs pendant la même image, il l'emporte. C'est ce qui rend l'ordre des
    /// déclarations indifférent, et donc ce mécanisme sûr à adopter progressivement.
    pub fn salir(&self, zone: glucose_core::geometry::Rect) {
        self.salissure.set(self.salissure.get().avec(zone));
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

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let (w, h) = (
            NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN),
            NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN),
        );

        // La carte graphique d'abord, le processeur s'il n'y en a pas. Ce n'est pas un
        // secours honteux : une machine virtuelle, un bureau distant ou un pilote absent sont
        // des cas de tous les jours, et l'application doit s'ouvrir quand même.
        let mut presenter: Box<dyn crate::present::Presenter> =
            match crate::present::GpuPresenter::new(window.clone(), w, h) {
                Ok(gpu) => {
                    println!(
                        "[Glucose] présentation par la carte graphique : {}",
                        gpu.adaptateur()
                    );
                    Box::new(gpu)
                }
                Err(e) => {
                    eprintln!("[Glucose] pas de carte graphique disponible ({e}) — présentation par le processeur");
                    Box::new(crate::present::CpuPresenter::new(window.clone())?)
                }
            };
        presenter.resize(w, h)?;

        self.pixmap = Pixmap::new(width, height);
        window.set_cursor(winit::window::CursorIcon::Grab);
        self.window_title_cache = title;
        self.window = Some(window);
        self.presenter = Some(presenter);
        self.mark_dirty();
        Ok(())
    }
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
                if let Some(presenter) = &mut self.presenter {
                    if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
                        if let Err(e) = presenter.resize(w, h) {
                            eprintln!("[GlucoseDesktop] redimensionnement de la surface : {e}");
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
                // Un evenement par fichier : on accumule, et `about_to_wait` pose le lot.
                self.dropped_files.push(path_buf);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Le lot de fichiers deposes est complet : tous les `DroppedFile` d'un meme geste
        // sont pousses par le meme appel systeme, donc ils sont tous arrives.
        if !self.dropped_files.is_empty() {
            let lot = std::mem::take(&mut self.dropped_files);
            self.drop_files(&lot);
        }

        // Chaque raison de se reveiller dit le delai qu'elle demande ; la plus pressee decide.
        // Aucune ne s'oublie, parce qu'aucune n'a de comptabilite a tenir (voir `reveil`).
        match self.prochain_reveil() {
            Some(ms) => {
                let echeance =
                    std::time::Instant::now() + std::time::Duration::from_millis(ms.max(1));
                event_loop.set_control_flow(ControlFlow::WaitUntil(echeance));
            }
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }
}
