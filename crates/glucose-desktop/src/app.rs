//! Application Glucose Desktop — Event Loop Winit 0.30 et Framebuffer Softbuffer 0.4.

mod accueil;
mod fenetre;
mod mouvement;
mod peinture;
pub mod reveil;
mod terrain;

use crate::dock::{apply_organize_layout, DockCache, DockManager, OrganizeState};
use crate::interactions::resize::ResizeSession;
use crate::interactions::tools::text_card;
use crate::renderer::{Renderer, TextEditSession};
use crate::ui::UiState;
use glucose_core::hit_priority::CycleState;
use glucose_core::smart_align::{AlignRect, AlignTarget, SnapGuides};
use glucose_core::store::Store;
use std::num::NonZeroU32;
use std::sync::Arc;
use tiny_skia::Pixmap;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

/// Ce que dit la carte d'accueil d'un document neuf.
const WELCOME_TEXT: &str = "# Bienvenue dans Glucose !\n- 100% Rust ultra-rapide\n- Teintes symbiotiques dynamiques\n- Double-cliquez pour éditer";

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
    /// L'elan de la camera : ce que la main a demande et que l'image n'a pas encore montre,
    /// plus la vitesse qui lui survit quand la main lache (voir [`crate::interactions::elan`]).
    pub elan: crate::interactions::elan::Elan,
    /// Le vol de camera en cours : une destination decidee, rejointe en douceur plutot que
    /// par teleportation (voir [`crate::interactions::vol`]).
    pub vol: crate::interactions::vol::Vol,
    /// Le defilement en cours vient-il d'un pave tactile ?
    ///
    /// Observe, jamais suppose : un delta fractionnaire ou lateral est une chose qu'une
    /// molette ne peut pas produire. Le verdict vaut pour tout le geste -- une source ne
    /// change pas en son milieu -- et s'efface quand la vue s'immobilise, donc entre deux
    /// gestes. Aucune duree n'a eu a etre choisie.
    pub defilement_au_doigt: bool,
    /// Ce que l'oeil tolere de perdre, a la vitesse a laquelle la vue bouge en ce moment.
    ///
    /// Recalcule a chaque image depuis le deplacement REEL de la vue -- donc quelle qu'en
    /// soit la cause : un geste, l'elan qui s'eteint, un vol de camera.
    pub perception: crate::perception::Perception,
    /// La vue de l'image precedente, pour mesurer de combien elle a bouge.
    vue_precedente: Option<glucose_core::types::Viewport>,
    /// Ce que cette image montre du mouvement : le pas de temps sur lequel il a ete integre,
    /// et la vitesse apparente qui en resulte, en pixels par seconde.
    ///
    /// Les deux se lisent a la PRESENTATION, pas au calcul : c'est la seule facon de comparer
    /// le temps que le mouvement a parcouru au temps que l'ecran l'a montre (RYTHME-1).
    pas_et_vitesse: (std::time::Duration, f64),
    /// Ce que la derniere presentation a montre, tel que l'instantane le portera.
    rythme_de_l_image: crate::chronique::rythme::Mesure,
    /// Le tempo de soumission : combien de balayages par image, et quand soumettre pour que ce
    /// soit exactement cela (voir [`crate::tempo`]).
    pub tempo: crate::tempo::Tempo,
    /// La derniere image a-t-elle demande la suivante ?
    ///
    /// Vrai quand une raison de reveil etait active -- animation, elan, vol, decodage. Faux
    /// quand l'application s'est endormie faute de quoi que ce soit a faire : l'intervalle qui
    /// suit est alors du repos, pas un gel, et la trajectoire n'a rien a rattraper.
    image_attendue: bool,
    /// Le bouton gauche tient-il la minimap ?
    ///
    /// Tant qu'il tient, la destination du vol **suit le curseur** : c'est le voyage continu
    /// que Glucose Tauri permet, par opposition au saut par clic.
    pub minimap_tenue: bool,
    /// L'horloge de la trajectoire : le temps tel que l'ECRAN le montre (voir
    /// [`crate::horloge`]).
    ///
    /// # Elle remplace deux horloges, et c'est le fond du sujet
    ///
    /// Il y en avait deux -- celle de l'elan, celle de la boucle -- et toutes deux mesuraient
    /// entre deux DEBUTS de rendu. Le contenu avancait donc du temps de calcul, pendant qu'il
    /// etait montre pendant l'intervalle de presentation : deux durees qui different
    /// exactement de la variation du cout d'une image, soit jusqu'a soixante millisecondes.
    pub horloge: crate::horloge::Horloge,
    /// De combien la scene est rendue plus petite que la fenetre pendant un geste, et le
    /// tampon ou elle se rend alors (voir [`crate::resolution`]).
    pub resolution: crate::resolution::Resolution,
    pub tampon_reduit: Option<Pixmap>,
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
    /// Le curseur a-t-il ete pose au moins une fois dans cette fenetre ?
    ///
    /// `(0, 0)` est une position parfaitement valide, donc indiscernable de « on ne sait pas
    /// encore » -- et c'est la valeur de depart. Sans ce drapeau, une fenetre qu'on vient
    /// d'ouvrir zoome autour de son coin superieur gauche.
    pub curseur_vu: bool,
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
    /// La cadence de l'écran, lue et non supposée (CADENCE-1).
    ///
    /// Tout le projet a longtemps raisonné sur soixante hertz. Sur un écran à 240 Hz, un banc
    /// qui annonce « tenu » à 9 ms ment de plus du double, et fait optimiser dans la mauvaise
    /// direction. C'est elle qui dit ce qui reste pour le travail de fond après une image.
    pub cadence: crate::cadence::Cadence,

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
    /// Ce que la session a observe : ce qui coute, et ce qui a gele (CHRONIQUE-1).
    ///
    /// Enregistre en permanence, chez l'utilisateur, pendant l'usage reel. C'est la seule
    /// mesure qui dise ce qu'il vit -- un banc ne mesure que ce qu'on lui demande.
    pub chronique: crate::chronique::Chronique,
    /// L'echelle de la vue a l'image precedente, qui suffit a reconnaitre un zoom.
    ///
    /// Le zoom n'a pas de session : c'est un evenement de molette. Le deduire du document
    /// evite d'inventer une duree d'attente apres laquelle on cesserait de "zoomer".
    echelle_precedente: f64,
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
        let renderer = Renderer::new();
        let store = accueil::document_d_accueil(&renderer);
        let saved_version = store.version;

        Self {
            store,
            renderer,
            animator: crate::animation::Animator::new(),
            elan: crate::interactions::elan::Elan::default(),
            vol: crate::interactions::vol::Vol::default(),
            defilement_au_doigt: false,
            perception: crate::perception::Perception::nette(),
            vue_precedente: None,
            pas_et_vitesse: (std::time::Duration::ZERO, 0.0),
            rythme_de_l_image: crate::chronique::rythme::Mesure::default(),
            image_attendue: false,
            tempo: crate::tempo::Tempo::nouveau(),
            minimap_tenue: false,
            horloge: crate::horloge::Horloge::nouvelle(),
            resolution: crate::resolution::Resolution::nette(),
            tampon_reduit: None,
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
            curseur_vu: false,
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
            cadence: crate::cadence::Cadence::inconnue(),
            project_path: None,
            saved_version,
            window_title_cache: String::new(),
            chronique: crate::chronique::Chronique::nouvelle(),
            echelle_precedente: 1.0,
            // Tout, et non rien : la première image doit se dessiner entièrement.
            salissure: std::cell::Cell::new(crate::salissure::Salissure::Tout),
        }
    }

    pub fn redraw(&mut self) {
        // Le marqueur « modifié » du titre suit l'état réel du document (INVARIANT SAVE-2).
        // Le poser ici plutôt que dans chaque mutation garantit qu'aucune ne l'oublie ;
        // `sync_window_title` ne touche la fenêtre que lorsque le titre change vraiment.
        self.sync_window_title();
        if let Some(window) = self.window.clone() {
            crate::perf::frame_begin();
            let frame_started = std::time::Instant::now();
            let size = window.inner_size();
            let width = size.width.max(1);
            let height = size.height.max(1);

            if let (Some(w), Some(h), Some(presenter)) = (
                NonZeroU32::new(width),
                NonZeroU32::new(height),
                &mut self.presenter,
            ) {
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

            // La camera bouge ICI, une seule fois par image, et jamais dans l'evenement :
            // c'est ce qui fait qu'une diagonale est une diagonale et non un escalier.
            self.appliquer_l_elan(width, height);

            let tampon_neuf = need_new_pixmap | self.accorder_le_tampon_reduit(width, height);
            self.peindre_ce_qui_a_change((width, height), tampon_neuf);

            // TEMPO-1 : l'image ne part pas quand elle est prete, elle part quand c'est
            // l'heure -- un nombre entier et CONSTANT de balayages apres la precedente. Ce
            // qui reste d'ici la sert au travail de fond.
            self.attendre_l_heure_de_soumettre(frame_started);
            self.presenter_et_noter_le_rythme(frame_started);
            self.tempo.soumise(std::time::Instant::now());
            self.clore_l_image(frame_started, (width, height));
        }
    }

    /// Ce qui suit la présentation : la latence vécue, le travail de fond, et la trace.
    ///
    /// Séparé du rendu parce que rien ici ne retarde l'image — elle est déjà à l'écran. Ce
    /// bloc occupe le temps qu'on aurait passé à attendre la suivante.
    fn clore_l_image(&mut self, debut: std::time::Instant, (largeur, hauteur): (u32, u32)) {
        // NAV-3 : l'age du plus ancien geste que cette image montre enfin. C'est **la**
        // grandeur qui dit « fluide », et aucune duree d'image ne l'explique.
        if let Some(l) = self.chronique.navigation.image_presentee() {
            crate::perf::compteur("nav_latence_us", l.as_micros() as f64);
        }

        // CASCADE-1 : le travail de fond a deja pris l'attente du tempo. Il ne prend ici que
        // ce que le plancher de la charte laisse encore quand l'image etait en retard -- sans
        // quoi la machine reste enfermee dans son regime degrade : images cheres faute de
        // vignettes, vignettes jamais construites faute de temps.
        if crate::perf::valeur_du_compteur("tempo_attente_us").unwrap_or(0.0) <= 0.0 {
            let faites = self
                .renderer
                .magasin
                .avancer_les_vignettes(self.cadence.tranche_de_fond(debut.elapsed()));
            crate::perf::compteur(
                "vign_atelier",
                f64::from(u32::try_from(faites).unwrap_or(u32::MAX)),
            );
            crate::perf::stage("atelier");
        }
        crate::perf::compteur(
            "vign_attente",
            self.renderer.magasin.vignettes.en_chantier() as f64,
        );

        let ecoule = debut.elapsed();
        self.accorder_la_finesse(ecoule);
        crate::perf::compteur("img_reduction", f64::from(self.resolution.facteur()));
        crate::perf::frame_end();
        // La chronique lit les postes APRES `frame_end` : celui-ci ne les efface pas, il se
        // contente de les afficher quand la trace est demandee.
        self.enregistrer_l_image(
            ecoule.as_micros().min(u128::from(u32::MAX)) as u32,
            (largeur, hauteur),
        );
    }

    /// Applique a la camera ce que l'elan a retenu pour cette image.
    ///
    /// Un seul deplacement et un seul changement d'echelle, quel que soit le nombre
    /// d'evenements recus depuis la derniere image. La diagonale de la fenetre sert de mesure
    /// commune aux deux : elle dit ce qu'un reste de zoom deplacerait a l'ecran, donc quand il
    /// devient invisible.
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

    /// Winit appelle ceci quand la boucle se termine, quelle qu'en soit la raison.
    ///
    /// La croix n'est pas la seule facon de fermer une application : `exiting` couvre aussi
    /// l'arret demande par le systeme et toute sortie de boucle declenchee ailleurs. La
    /// chronique s'ecrit donc la, et non dans le seul gestionnaire de la croix -- c'est ce qui
    /// manquait, et une session entiere s'est perdue pour cette raison.
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.clore_la_chronique();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Le lot de fichiers deposes est complet : tous les `DroppedFile` d'un meme geste
        // sont pousses par le meme appel systeme, donc ils sont tous arrives.
        if !self.dropped_files.is_empty() {
            let lot = std::mem::take(&mut self.dropped_files);
            self.drop_files(&lot);
        }

        // Hors du rendu, et seulement quand il y a du neuf : une session qui finit mal garde
        // alors la trace de son pire moment (CHRONIQUE-1).
        self.sauver_la_chronique_si_besoin();

        // Chaque raison de se reveiller dit le delai qu'elle demande ; la plus pressee decide.
        // Aucune ne s'oublie, parce qu'aucune n'a de comptabilite a tenir (voir `reveil`).
        match self.prochain_reveil() {
            Some(ms) => {
                self.image_attendue = true;
                let echeance =
                    std::time::Instant::now() + std::time::Duration::from_millis(ms.max(1));
                event_loop.set_control_flow(ControlFlow::WaitUntil(echeance));
            }
            None => {
                self.image_attendue = false;
                // Un repos n'a pas de tempo : la grille repart de la prochaine soumission.
                self.tempo.oublier();
                event_loop.set_control_flow(ControlFlow::Wait);
            }
        }
    }
}
