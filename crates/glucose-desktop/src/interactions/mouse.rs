//! La souris : chaque événement est traduit en une question posée à une couche, du haut vers
//! le bas — jamais en logique.
//!
//! # Les preneurs
//!
//! Un clic gauche traverse les couches de l'écran dans l'ordre où elles se superposent : la
//! plongée en cours, le fil d'Ariane, la chrome (barre d'outils, onglets, minimap), les
//! panneaux du dock, et enfin le canevas. Chaque couche est un **preneur** — une fonction qui
//! rend `true` si le clic était pour elle — et le premier qui le prend arrête la descente.
//!
//! C'est la règle 1.7 de la fiche 05 : le gestionnaire d'événement ne contient aucune logique.
//! Il connaît l'ordre des couches, et c'est tout. Ce que chaque couche fait du clic est dans
//! son module : [`chrome`](super::chrome), [`panels`](super::panels), [`pick`](super::pick).
//!
//! Le gestionnaire précédent faisait 471 lignes sur sept niveaux : c'était `window_event`
//! (fiche 01, R-19) qui avait changé d'adresse, et chaque geste nouveau s'y entassait faute
//! d'un endroit où aller.

use crate::app::GlucoseApp;
use crate::params::{Pointer, ScreenFrame};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

/// Ce que disent les boutons dont la fonction n'existe pas encore. Un bouton qui annonce ce
/// qu'il n'a pas fait est un bouton qui ment ; celui-ci dit ce qu'il en est.
pub const NOT_YET_STORYBOARD: &str = "Storyboard : pas encore disponible";
pub const NOT_YET_AI: &str = "IA locale : pas encore disponible";

impl GlucoseApp {
    /// La position du curseur, en unités logiques d'écran.
    pub fn pointer(&self) -> Pointer {
        Pointer {
            x: self.mouse_pos.0 as f32,
            y: self.mouse_pos.1 as f32,
        }
    }

    /// Le cadre de la fenêtre tel que les couches le lisent.
    pub fn screen_frame(&self, screen_w: f32, screen_h: f32) -> ScreenFrame {
        ScreenFrame {
            width: screen_w,
            height: screen_h,
            header_h: self.ui.header_height(),
            scale: self.ui.scale_factor,
        }
    }

    /// Mouvement continu de la souris : il nourrit le geste en cours, s'il y en a un.
    pub fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        let prev_pos = self.mouse_pos;
        self.mouse_pos = (position.x, position.y);
        self.curseur_vu = true;
        let dx = position.x - prev_pos.0;
        let dy = position.y - prev_pos.1;

        // La minimap tenue passe avant tout le reste : tant qu'elle l'est, le curseur ne
        // designe rien d'autre qu'une destination.
        if self.minimap_tenue {
            self.suivre_la_minimap(position);
            return;
        }

        if self.dock_manager.drag.is_some() {
            self.dock_manager
                .update_drag(position.x as f32, position.y as f32);
            self.mark_dirty();
            return;
        }

        // Un glisser de sélection de texte passe avant tous les autres : tant qu'il dure, la
        // souris écrit dans une carte et ne déplace ni nœud ni caméra.
        if self.drag_text_to(self.mouse_pos) {
            self.update_cursor();
            return;
        }

        if self.is_panning {
            self.handle_pan_move(dx, dy);
        } else if self.draw_session.is_some() {
            // Un objet qui naît sous la main suit le curseur (DRAW-1). Il passe avant le
            // redimensionnement et le glisser : ces deux-là agissent sur ce qui existait
            // déjà, celui-ci sur ce qui vient d'apparaître sous le doigt.
            let vp = self.store.viewport();
            let (wx, wy) = crate::canvas::screen_to_world(position.x, position.y, &vp);
            self.update_draw(wx, wy);
        } else if self.resize_session.is_some() {
            self.handle_resize_move(position.x, position.y);
        } else if self.is_dragging_item {
            self.handle_item_drag_move(position.x, position.y);
        } else if self.selection_box.is_some() {
            self.update_selection_box(position.x, position.y);
            self.mark_dirty();
        } else if position.y < self.ui.header_height() as f64 {
            self.mark_dirty();
        }

        self.update_cursor();
    }

    /// Le curseur tient la minimap : la destination du vol le suit.
    fn suivre_la_minimap(&mut self, position: PhysicalPosition<f64>) {
        let Some(fenetre) = &self.window else {
            return;
        };
        let taille = fenetre.inner_size();
        let (w, h) = (taille.width as f32, taille.height as f32);
        let echelle = crate::theme::clamp_ui_scale(self.ui.scale());
        let Some(monde) = crate::ui::point_minimap(
            &self.store,
            position.x as f32,
            position.y as f32,
            w,
            h,
            echelle,
        ) else {
            // Sorti de la minimap en glissant : la derniere destination reste la bonne, et
            // le vol l'atteint. Relacher le bouton est ce qui termine le geste, pas le bord.
            return;
        };
        let ecran = glucose_core::membrane_focus::ScreenSize {
            width: f64::from(w),
            height: f64::from(h),
        };
        self.viser_par_la_minimap(monde, ecran, f64::from(self.ui.header_height()));
    }

    /// Enfoncement d'un bouton de la souris.
    pub fn handle_mouse_down(&mut self, button: MouseButton, screen_w: f32, screen_h: f32) {
        match button {
            MouseButton::Right | MouseButton::Middle => {
                self.right_or_middle_down = true;
                self.is_panning = true;
                // Un menu ouvert se referme au premier clic, où qu'il soit.
                self.ui.context_menu_at = None;
                if button == MouseButton::Right {
                    self.right_down_at = Some(self.mouse_pos);
                }
            }
            MouseButton::Left => self.handle_left_down(self.screen_frame(screen_w, screen_h)),
            _ => return,
        }
        self.update_cursor();
        self.mark_dirty();
    }

    /// Le clic gauche descend les couches ; la première qui le prend l'arrête.
    fn handle_left_down(&mut self, screen: ScreenFrame) {
        let pointer = self.pointer();
        let taken = self.click_context_menu(pointer, screen)
            || self.click_skips_flight()
            || self.click_breadcrumb(pointer, screen)
            || self.click_action_bar(pointer, screen)
            || self.click_chrome(pointer, screen)
            || self.click_dock(pointer, screen);
        if !taken {
            self.click_canvas(screen);
        }
    }

    /// Un clic pendant une plongée l'abrège : on arrive tout de suite. Le clic est consommé —
    /// le viser dans le tableau d'arrivée, à une position qui n'a rien à voir avec celle
    /// qu'on visait au départ, serait pire que de l'ignorer.
    fn click_skips_flight(&mut self) -> bool {
        self.animator.skip(&mut self.store)
    }

    /// Relâchement d'un bouton de souris : le geste en cours se termine.
    pub fn handle_mouse_up(&mut self, button: MouseButton) {
        match button {
            MouseButton::Right | MouseButton::Middle => {
                self.right_or_middle_down = false;
                self.is_panning = false;
                if button == MouseButton::Right {
                    self.open_context_menu_if_still();
                }
            }
            MouseButton::Left => {
                self.minimap_tenue = false;
                if let Some(dismissed) = self.dock_manager.finish_drag() {
                    self.ui
                        .show_toast(format!("Panneau {} fermé", dismissed.title()));
                    self.mark_dirty();
                    return;
                }
                if !self.right_or_middle_down {
                    self.is_panning = false;
                }
                // Le tracé se referme ici, et nulle part ailleurs : c'est le relâchement qui
                // dit si le geste était un clic ou un glisser (DRAW-1).
                self.finish_draw();
                self.end_text_drag();
                // Le cycle descend avant que le glisser ne s'efface : c'est ce qui a bougé,
                // pas la distance parcourue par le curseur, qui dit si on déplaçait un nœud
                // ou si on cherchait celui du dessous.
                if self.drag_applied_delta == (0.0, 0.0) {
                    self.advance_pick_cycle();
                    // SEL-MULTI-1 : rien n'a bouge, donc c'etait un clic -- la selection se
                    // ramene a l'element presse. Un glissement, lui, la garde entiere.
                    self.reduire_la_selection_si_demandee();
                } else {
                    self.reduire_a_la_relache = None;
                }
                self.finish_resize();
                self.finish_item_drag();
                self.finish_selection_box();
            }
            _ => return,
        }
        self.update_cursor();
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
