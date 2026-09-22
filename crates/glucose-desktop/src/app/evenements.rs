//! La boucle d'evenements : ce que le systeme envoie, et a qui Glucose le confie.
//!
//! # Pourquoi ce fichier existe a part
//!
//! `window_event` est un aiguillage, et la fiche 05 § 1.7 est explicite : un gestionnaire
//! d'evenements ne contient **aucune logique**, il traduit un evenement en intention. Il
//! avait pourtant grossi jusqu'a melanger le redimensionnement d'une surface graphique, la
//! garde du clic de reveil et la lecture d'une taille de fenetre -- trois raisons de changer
//! dans un seul `match`.
//!
//! Les evenements sont donc ranges par famille, et chaque famille sait ce qu'elle fait : la
//! **fenetre** (naitre, grandir, se fermer), la **main** (souris, clavier, fichiers deposes),
//! et le **rythme** (redessiner, attendre).

use super::GlucoseApp;
use std::num::NonZeroU32;
use tiny_skia::Pixmap;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

impl GlucoseApp {
    /// **Ce que la main envoie** : la souris, le clavier, les fichiers deposes — et le focus,
    /// qui decide de la facon dont le premier clic sera lu.
    ///
    /// Rend `true` quand l'evenement lui appartenait, pour que l'aiguillage principal n'ait
    /// plus a le connaitre. La fiche 05 § 1.7 demande un gestionnaire qui traduit un evenement
    /// en intention et rien d'autre ; melanger la naissance d'une fenetre et le relachement
    /// d'un bouton dans un seul `match` est ce qui l'a fait passer les quatre-vingts lignes.
    fn evenement_de_la_main(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::ModifiersChanged(mods) => self.modifiers = mods.state(),
            // **Le focus n'etait ecoute nulle part** (REVEIL-1). Le clic qui ramene Glucose au
            // premier plan s'executait donc sur le canevas.
            WindowEvent::Focused(actif) => {
                self.clic_de_reveil = *actif;
                // Les modificateurs sont perimes des qu'on a quitte la fenetre : personne ne
                // nous a dit que Ctrl avait ete relache pendant qu'un autre logiciel l'avait.
                self.modifiers = winit::keyboard::ModifiersState::empty();
            }
            WindowEvent::CursorMoved { position, .. } => {
                // La souris a bouge : le prochain clic est voulu, pas une formalite.
                self.clic_de_reveil = false;
                self.handle_cursor_moved(*position);
            }
            WindowEvent::MouseWheel { delta, .. } => self.handle_mouse_wheel(*delta),
            WindowEvent::MouseInput { button, state, .. } => self.clic(*button, *state),
            WindowEvent::KeyboardInput { event, .. } => self.handle_key(event),
            // Un evenement par fichier : on accumule, et `about_to_wait` pose le lot.
            WindowEvent::DroppedFile(chemin) => self.dropped_files.push(chemin.clone()),
            _ => return false,
        }
        true
    }

    /// Un appui ou un relachement, une fois la garde du reveil passee (REVEIL-1).
    fn clic(&mut self, button: winit::event::MouseButton, state: ElementState) {
        if !self.ce_clic_agit(state) {
            return;
        }
        let (largeur, hauteur) = self.window.as_ref().map_or((1280.0, 720.0), |w| {
            let taille = w.inner_size();
            (taille.width as f32, taille.height as f32)
        });
        match state {
            ElementState::Pressed => self.handle_mouse_down(button, largeur, hauteur),
            ElementState::Released => self.handle_mouse_up(button),
        }
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
        if self.evenement_de_la_main(&event) {
            return;
        }
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

        // **Ce que le pont natif a recolte** (DEPOT-WEB-1). Il ecrit depuis la boucle de
        // messages de Windows, au milieu d'un geste ; on pose ici, ou le document n'est lu
        // par personne. Un lot par depot : glisser huit images d'une page est UN geste.
        for recolte in self
            .depots
            .as_ref()
            .map(|d| d.recolter())
            .unwrap_or_default()
        {
            self.poser_le_depot(&recolte);
        }

        // **La carte se rouvre ici, hors du rendu** (ARBITRE-1) : detruire la chaine pendant
        // qu'une image est detenue arrache le sol sous ses pieds, et c'est le plantage que la
        // fiche 17 § 2.3 raconte. Ici, aucune image ne l'est.
        self.rouvrir_la_carte_si_demande();

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
