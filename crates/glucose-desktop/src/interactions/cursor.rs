//! Gestion des curseurs contextuels (Roadmap 1.18, R-18, 2.10).
//!
//! Le curseur annonce le geste avant le clic : au-dessus d'une poignée, il prend la forme
//! du redimensionnement (`↔`, `↕`, `⤡`, `⤢`) que `hit_priority::handle_cursor` nomme.

use crate::app::GlucoseApp;
use crate::interactions::resize::cursor_for;
use crate::ui::ActiveTool;
use winit::window::CursorIcon;

impl GlucoseApp {
    /// Le curseur qui convient à l'outil actif et à l'état d'interaction.
    pub fn current_cursor(&self) -> CursorIcon {
        if self.is_panning {
            CursorIcon::Grabbing
        } else if self.space_pressed || self.ui.active_tool == ActiveTool::Pan {
            CursorIcon::Grab
        } else if let Some(session) = &self.resize_session {
            cursor_for(session.handle)
        } else if self.is_dragging_item {
            CursorIcon::Move
        } else if let Some(handle) = self.hovered_handle() {
            cursor_for(handle)
        } else if self.editing_session.is_some() || self.ui.active_tool == ActiveTool::Text {
            CursorIcon::Text
        } else if self.ui.active_tool == ActiveTool::Arrow {
            CursorIcon::Crosshair
        } else {
            CursorIcon::Default
        }
    }

    /// Met à jour le curseur de la fenêtre winit.
    pub fn update_cursor(&mut self) {
        let cursor = self.current_cursor();
        if let Some(window) = &self.window {
            window.set_cursor(cursor);
        }
    }
}
