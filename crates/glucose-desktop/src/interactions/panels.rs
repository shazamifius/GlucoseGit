//! Les clics sur les panneaux du dock : leur poignée, qui commence un glisser, et leur
//! contenu, qui produit un [`PanelClickResult`].

use crate::app::GlucoseApp;
use crate::dock::{compute_panel_layouts, handle_dock_click, DragSession, PanelClickResult};
use crate::interactions::mouse::NOT_YET_AI;
use crate::params::{Pointer, ScreenFrame};

impl GlucoseApp {
    /// La poignée d'un panneau, puis son contenu. Le panneau le plus haut dans la pile — le
    /// dernier dessiné — est regardé en premier.
    pub fn click_dock(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        self.click_dock_grip(pointer, screen)
            || self.prendre_un_champ(pointer, screen)
            || self.click_dock_content(pointer, screen)
    }

    fn click_dock_grip(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        let layouts = compute_panel_layouts(
            &self.dock_manager,
            &self.store,
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

    /// **Le tiroir tenu suit la main**, et se range où il est vu (TIROIR-4) — sur la géométrie que
    /// le dessin lit. Rend `true` si un tiroir était tenu.
    pub(crate) fn glisser_le_tiroir(&mut self, souris: (f32, f32)) -> bool {
        if self.dock_manager.drag.is_none() {
            return false;
        }
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        let ecran = self.screen_frame(largeur, hauteur);
        let store = &self.store;
        self.dock_manager
            .update_drag(souris, |d| crate::dock::places(d, store, ecran));
        self.mark_dirty();
        true
    }

    /// **Le tiroir lâché** : rangé, ou fermé s'il est sorti de l'écran de son côté. Rend celui
    /// qui s'est fermé.
    pub(crate) fn lacher_le_tiroir(&mut self) -> Option<crate::dock::TabId> {
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        let ecran = self.screen_frame(largeur, hauteur);
        let store = &self.store;
        self.dock_manager
            .finish_drag(largeur, |d| crate::dock::places(d, store, ecran))
    }

    /// **TIROIR-6 — la molette fait défiler le tiroir qui cache une part de son contenu** : d'une
    /// ligne de canevas par cran (`PAN_LIGNE_PX`, à l'échelle de l'interface), ou de ce que le pavé
    /// parcourt ; bornée à ce qu'il cache. Arrivée au bout, elle reste au tiroir — le canevas
    /// derrière ne zoome pas par surprise. Un tiroir montré entier la laisse passer, comme avant.
    /// Rend vrai s'il l'a prise.
    pub(crate) fn molette_sur_un_tiroir(&mut self, delta: winit::event::MouseScrollDelta) -> bool {
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        let ecran = self.screen_frame(largeur, hauteur);
        let (x, y) = (self.mouse_pos.0 as f32, self.mouse_pos.1 as f32);
        let places = crate::dock::places(&self.dock_manager, &self.store, ecran);
        let Some(b) = places.iter().rev().find(|b| b.contains_point(x, y)) else {
            return false;
        };
        let cache = b.naturelle - b.height;
        if cache <= 0.0 {
            return false;
        }
        let (_, dy, ligne) = crate::interactions::pan_zoom::deltas(delta);
        let s = f64::from(crate::theme::clamp_ui_scale(ecran.scale));
        let points = if ligne {
            -dy * crate::interactions::pan_zoom::PAN_LIGNE_PX * s
        } else {
            -dy
        };
        let neuf = (f64::from(b.defile) + points).clamp(0.0, f64::from(cache)) as f32;
        self.dock_manager.defilement.insert(b.tab, neuf);
        self.mark_dirty();
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
            PanelClickResult::Gabarit(intent) => self.suivre_le_panneau_des_presets(intent),
            PanelClickResult::Storyboard(intent) => self.suivre_le_panneau_du_storyboard(intent),
            // Fiche 09 § 10.3 : pas de moteur, pas de téléchargement.
            PanelClickResult::DownloadModel => self.ui.show_toast(NOT_YET_AI),
            // Le panneau a déjà changé son propre état — un tri, une densité, le minuteur ;
            // rien à faire au-dessus.
            PanelClickResult::Handled => {}
        }
    }
}
