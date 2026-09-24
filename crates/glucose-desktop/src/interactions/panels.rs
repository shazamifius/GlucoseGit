//! Les clics sur les panneaux du dock : leur poignée, qui commence un glisser, et leur
//! contenu, qui produit un [`PanelClickResult`].

use crate::app::GlucoseApp;
use crate::dock::{compute_panel_layouts, handle_dock_click, DragSession, PanelClickResult};
use crate::interactions::mouse::{NOT_YET_AI, NOT_YET_STORYBOARD};
use crate::params::{Pointer, ScreenFrame};

impl GlucoseApp {
    /// La poignée d'un panneau, puis son contenu. Le panneau le plus haut dans la pile — le
    /// dernier dessiné — est regardé en premier.
    pub fn click_dock(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        self.click_dock_grip(pointer, screen) || self.click_dock_content(pointer, screen)
    }

    fn click_dock_grip(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        let layouts = compute_panel_layouts(
            &self.dock_manager,
            screen.width,
            screen.height,
            screen.header_h,
            screen.scale,
        );
        let Some(tab) = layouts
            .iter()
            .rev()
            .find(|l| l.grip_contains_point(pointer.x, pointer.y))
            .map(|l| l.tab)
        else {
            return false;
        };
        self.dock_manager.drag = Some(DragSession {
            tab,
            start_x: pointer.x,
            start_y: pointer.y,
            current_x: pointer.x,
            current_y: pointer.y,
        });
        true
    }

    fn click_dock_content(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        let Some(result) = handle_dock_click(
            &mut self.dock_manager,
            &self.store,
            &self.renderer.typography,
            screen,
            pointer,
        ) else {
            return false;
        };
        self.apply_panel_click(result);
        true
    }

    fn apply_panel_click(&mut self, result: PanelClickResult) {
        match result {
            PanelClickResult::ApplyLayout(state) => self.apply_dock_layout(&state),
            PanelClickResult::Domain(intent) => self.apply_domain_intent(intent),
            PanelClickResult::Temps(intent) => self.agir_dans_le_temps(intent),
            // Fiche 09 § 8 : le storyboard n'a pas d'effet sur le canevas. Le panneau ne doit
            // pas laisser croire le contraire.
            PanelClickResult::StoryboardNotReady => self.ui.show_toast(NOT_YET_STORYBOARD),
            // Fiche 09 § 10.3 : pas de moteur, pas de téléchargement.
            PanelClickResult::DownloadModel => self.ui.show_toast(NOT_YET_AI),
            // Le panneau a déjà changé son propre état — un tri, une densité, le minuteur ;
            // rien à faire au-dessus.
            PanelClickResult::Handled => {}
        }
    }
}
