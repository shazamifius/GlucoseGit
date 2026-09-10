//! Gestion des curseurs contextuels (Roadmap 1.18, R-18, 2.10).

use crate::app::GlucoseApp;
use crate::ui::ActiveTool;
use winit::window::CursorIcon;

impl GlucoseApp {
    /// Met à jour le curseur de la fenêtre winit en fonction de l'outil actif et de l'état d'interaction.
    pub fn update_cursor(&mut self) {
        let Some(window) = &self.window else { return };

        let cursor = if self.is_panning {
            CursorIcon::Grabbing
        } else if self.space_pressed || self.ui.active_tool == ActiveTool::Pan {
            CursorIcon::Grab
        } else if self.is_dragging_item {
            CursorIcon::Move
        } else if self.editing_session.is_some() || self.ui.active_tool == ActiveTool::Text {
            CursorIcon::Text
        } else if self.ui.active_tool == ActiveTool::Arrow {
            CursorIcon::Crosshair
        } else {
            CursorIcon::Default
        };

        window.set_cursor(cursor);
    }
}
