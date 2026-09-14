//! Les clics sur la chrome : le fil d'Ariane, la barre d'outils, les onglets, la minimap.
//!
//! La chrome dit ce qui a été cliqué — un [`UiAction`] — et ce module dit ce que ça fait.
//! La géométrie, elle, n'est calculée qu'une fois, par le même code qui dessine (loi L4) :
//! c'est [`handle_ui_click`] qui la lit.

use crate::animation::fly_out_to_depth;
use crate::app::GlucoseApp;
use crate::dock::TabId;
use crate::interactions::mouse::NOT_YET_EXPORT;
use crate::params::{Pointer, ScreenFrame};
use crate::ui::action_bar::ActionBarClick;
use crate::ui::context_menu::MenuAction;
use crate::ui::{handle_ui_click, UiAction};
use glucose_core::membrane_focus::ScreenSize;
use glucose_core::store::StackMove;
use glucose_core::types::Viewport;

impl GlucoseApp {
    /// Le fil d'Ariane occupe une bande sous les onglets, donc il passe avant la chrome.
    /// Remonter change le tableau : plus rien de ce clic ne vaut ensuite, même si le vol n'a
    /// pas pu partir.
    pub fn click_breadcrumb(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        let Some(depth) = crate::ui::breadcrumb::hit_breadcrumb(
            &self.store,
            &self.renderer.typography,
            screen.header_h,
            screen.scale,
            (pointer.x, pointer.y),
        ) else {
            return false;
        };
        fly_out_to_depth(
            &mut self.store,
            &mut self.animator,
            depth,
            screen_size(screen),
        );
        true
    }

    /// Un clic droit relâché **sur place** ouvre le menu contextuel ; un clic droit qui a
    /// déplacé le curseur était un pan, et n'ouvre rien.
    ///
    /// Le seuil est celui du double-clic (`DOUBLE_CLICK_SLOP_PX`) : la même tolérance de main
    /// tremblante, et une constante de moins.
    pub fn open_context_menu_if_still(&mut self) {
        let Some((dx, dy)) = self.right_down_at.take() else {
            return;
        };
        let bouge = (dx - self.mouse_pos.0).hypot(dy - self.mouse_pos.1);
        if bouge > super::pick::DOUBLE_CLICK_SLOP_PX {
            return;
        }
        self.ui.context_menu_at = Some((self.mouse_pos.0 as f32, self.mouse_pos.1 as f32));
        self.mark_dirty();
    }

    /// Le menu contextuel prend le clic gauche avant toute autre couche : tant qu'il est
    /// ouvert, c'est lui qui attend une décision.
    ///
    /// Il prend **tout** ce qui tombe sur lui, entrée ou pas, et se referme dans les deux cas
    /// — cliquer à côté ferme, comme dans n'importe quel menu.
    pub fn click_context_menu(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        let Some(at) = self.ui.context_menu_at else {
            return false;
        };
        let menu = crate::ui::context_menu::layout_context_menu(
            &self.store,
            &self.renderer.typography,
            at,
            (screen.width, screen.height),
            self.ui.scale_factor,
        );
        self.ui.context_menu_at = None;
        self.mark_dirty();
        let Some(menu) = menu else {
            return false;
        };
        if !crate::ui::context_menu::covers(&menu, pointer.x, pointer.y) {
            // Le clic n'était pas pour le menu : il le referme et continue sa descente.
            return false;
        }
        if let Some(action) = crate::ui::context_menu::hit_context_menu(&menu, pointer.x, pointer.y)
        {
            self.apply_menu_action(action);
        }
        true
    }

    fn apply_menu_action(&mut self, action: MenuAction) {
        let board = self.store.project.active_board_id.clone();
        match action {
            MenuAction::Duplicate => self.duplicate_selection(),
            MenuAction::ToggleLock => self.toggle_lock(),
            MenuAction::ToFront => {
                self.store.move_selection_in_stack(&board, StackMove::Front);
            }
            MenuAction::ToBack => {
                self.store.move_selection_in_stack(&board, StackMove::Back);
            }
            MenuAction::Delete => self.delete_selection(),
            MenuAction::Paste => self.paste_from_clipboard(),
            MenuAction::SelectAll => self.select_all(),
        }
        self.mark_dirty();
    }

    /// La barre d'action contextuelle, en bas de l'écran (fiche 10 § 3).
    ///
    /// Elle passe avant la chrome et avant le canevas, et elle prend **tout** ce qui tombe
    /// sur elle, boutons ou pas : un clic entre deux boutons qui filerait jusqu'au canevas
    /// désélectionnerait — donc ferait disparaître la barre sous le doigt.
    pub fn click_action_bar(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        let Some(bar) = crate::ui::action_bar::layout_action_bar(
            &self.store,
            &self.renderer.typography,
            (screen.width, screen.height),
            screen.scale,
        ) else {
            return false;
        };
        if !crate::ui::action_bar::covers(&bar, pointer.x, pointer.y) {
            return false;
        }
        match crate::ui::action_bar::hit_action_bar(&bar, pointer.x, pointer.y) {
            Some(ActionBarClick::ToggleLock) => self.toggle_lock(),
            Some(ActionBarClick::Delete) => self.delete_selection(),
            None => {}
        }
        true
    }

    /// La barre d'outils, les onglets et la minimap.
    pub fn click_chrome(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        let Some(action) = handle_ui_click(
            pointer.x,
            pointer.y,
            screen.width,
            screen.height,
            &self.store,
            &mut self.ui,
            &self.renderer.typography,
        ) else {
            return false;
        };
        self.apply_ui_action(action, screen);
        true
    }

    fn apply_ui_action(&mut self, action: UiAction, screen: ScreenFrame) {
        match action {
            UiAction::SelectTool(tool) => self.ui.active_tool = tool,
            UiAction::AddImages => self.pick_and_import_images(),
            UiAction::Organize => self.dock_manager.toggle_tab(TabId::Organize),
            UiAction::ToggleTimer => self.dock_manager.toggle_tab(TabId::Pomodoro),
            UiAction::ToggleStoryboard => self.dock_manager.toggle_tab(TabId::Storyboard),
            UiAction::TogglePlugins => self.dock_manager.toggle_tab(TabId::Plugins),
            UiAction::TogglePreset => self.dock_manager.toggle_tab(TabId::Preset),
            UiAction::ToggleDomains => self.dock_manager.toggle_tab(TabId::Domains),
            // L'aimant et la collaboration ont déjà basculé leur état dans `handle_ui_click`.
            UiAction::ToggleMagnet | UiAction::ToggleCollab => {}
            UiAction::ToggleTransDomain => self.ui.show_toast(if self.ui.trans_domain {
                "Trans-domaines activé"
            } else {
                "Trans-domaines désactivé"
            }),
            // Fiche 09 § 5 : aucun export n'est branché. Le bouton reste, parce que la barre
            // d'outils le prévoit (fiche 10) ; il dit la vérité plutôt que d'annoncer une
            // exportation qui n'a pas lieu.
            UiAction::ExportMenu => self.ui.show_toast(NOT_YET_EXPORT),
            UiAction::SelectBoard(id) => self.store.set_active_board_id(&id),
            UiAction::AddBoard => self.add_board(),
            UiAction::MinimapPan(wx, wy) => self.center_view_on(wx, wy, screen),
        }
    }

    fn add_board(&mut self) {
        let name = format!("Board {}", self.store.project.boards.len() + 1);
        let id = self.store.add_board(name);
        self.store.set_active_board_id(id);
        self.ui.show_toast("Nouveau board créé");
    }

    /// Recentre la caméra sur un point du monde, à l'échelle courante.
    fn center_view_on(&mut self, wx: f64, wy: f64, screen: ScreenFrame) {
        let Some(board) = self.store.active_board() else {
            return;
        };
        let (id, scale) = (board.id.clone(), board.viewport.scale);
        self.store.set_viewport(
            &id,
            Viewport {
                x: f64::from(screen.width) / 2.0 - wx * scale,
                y: f64::from(screen.height) / 2.0 - wy * scale,
                scale,
            },
        );
    }
}

/// La taille d'écran que les vols de caméra attendent.
pub(super) fn screen_size(screen: ScreenFrame) -> ScreenSize {
    ScreenSize {
        width: f64::from(screen.width),
        height: f64::from(screen.height),
    }
}
