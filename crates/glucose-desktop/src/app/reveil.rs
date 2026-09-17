//! Les raisons qu'a l'application de se réveiller, et dans combien de temps.
//!
//! # Une seule question, posée cinq fois
//!
//! Le curseur qui clignote, le toast qui s'efface, la caméra qui vole, les images qui se
//! décodent, le minuteur qui tourne : ces cinq-là n'ont rien en commun sauf l'essentiel — ils
//! veulent tous que l'application repasse **bientôt**, et chacun sait dire dans combien de
//! temps.
//!
//! Ils vivaient en cinq blocs accumulés dans `about_to_wait`, chacun mettant à jour à la main
//! un minimum courant et un drapeau « il y a un minuteur ». Le schéma se répétait, donc il
//! s'oubliait : ajouter une sixième raison demandait de se souvenir des deux gestes.
//!
//! Ici chaque raison rend simplement **le délai qu'elle demande**, ou rien si elle n'a rien à
//! demander. La plus pressée l'emporte, et c'est `min` qui le dit — pas un accumulateur.
//! Aucune borne arbitraire ne subsiste : l'ancien plafond d'une seconde n'était que la valeur
//! initiale de ce minimum, et il disparaît avec lui.

use super::GlucoseApp;
use crate::ui::ToastRepaint;

impl GlucoseApp {
    /// Le délai avant le prochain réveil, ou `None` si rien n'est attendu.
    ///
    /// Chaque raison salit la vue elle-même si elle a besoin d'être redessinée : demander un
    /// rafraîchissement et demander un réveil sont deux choses distinctes — un toast au
    /// plateau attend sans rien redessiner, un décodage redessine sans rien animer.
    pub(super) fn prochain_reveil(&mut self) -> Option<u64> {
        [
            self.attente_du_curseur(),
            self.attente_du_toast(),
            self.attente_de_l_animation(),
            self.attente_du_decodage(),
            self.attente_du_pomodoro(),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    /// Le curseur d'édition de texte clignote à la demi-seconde.
    ///
    /// Le réveil est calé sur **ce qui reste** de la demi-seconde en cours, et non sur une
    /// demi-seconde pleine : sans cela le clignotement dériverait à chaque image lente.
    fn attente_du_curseur(&mut self) -> Option<u64> {
        let Some(session) = &self.editing_session else {
            self.last_blink_phase = true;
            return None;
        };
        let elapsed = session.blink_timer.elapsed().as_millis();
        let phase = (elapsed / 500) % 2 == 0;
        if phase != self.last_blink_phase {
            self.last_blink_phase = phase;
            self.mark_dirty();
        }
        Some((500 - (elapsed % 500)).max(1) as u64)
    }

    /// Le toast sait lui-même s'il doit se redessiner, attendre, ou disparaître.
    fn attente_du_toast(&mut self) -> Option<u64> {
        match self.ui.current_toast.as_ref()?.repaint_need() {
            ToastRepaint::Gone => {
                self.ui.current_toast = None;
                self.mark_dirty();
                None
            }
            ToastRepaint::Redraw => {
                self.mark_dirty();
                Some(self.animation_interval_ms())
            }
            // Le plateau : l'opacité ne bouge plus, donc rien à redessiner — mais il faudra
            // revenir quand le fondu de sortie commencera.
            ToastRepaint::Sleep(wait_ms) => Some(wait_ms),
        }
    }

    /// Le vol de la caméra avance d'une image, et dit quand la suivante est due.
    fn attente_de_l_animation(&mut self) -> Option<u64> {
        let reste_ms = self.animator.tick(&mut self.store)?;
        self.mark_dirty();
        Some(reste_ms.max(1).min(self.animation_interval_ms()))
    }

    /// Des images se décodent sur les fils de fond (DECODE-1).
    ///
    /// Tant qu'il en reste, l'application a une raison de se réveiller même si personne ne
    /// touche à rien : c'est ce qui fait paraître une photo dès qu'elle est prête, sans que
    /// le rendu ait eu à l'attendre.
    fn attente_du_decodage(&mut self) -> Option<u64> {
        if self.renderer.magasin.en_travail() == 0 {
            return None;
        }
        self.mark_dirty();
        Some(self.animation_interval_ms())
    }

    /// Le minuteur Pomodoro du dock, qui compte en secondes pleines.
    fn attente_du_pomodoro(&mut self) -> Option<u64> {
        if !self.dock_manager.pomodoro.running {
            return None;
        }
        if self.dock_manager.tick_pomodoro() {
            self.mark_dirty();
        }
        let elapsed_ms = self.dock_manager.pomodoro.last_tick.elapsed().as_millis();
        Some(1000_u128.saturating_sub(elapsed_ms).max(1) as u64)
    }
}
