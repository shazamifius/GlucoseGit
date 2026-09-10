//! Interaction de navigation caméra : Pan & Zoom centré sur curseur (PureRef-style).

use crate::app::GlucoseApp;
use crate::canvas::zoom_at;
use winit::event::MouseScrollDelta;

impl GlucoseApp {
    /// Gère les événements de molette et gestes tactiles pour le zoom continu et le pan.
    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                if y.abs() > 0.001 {
                    // Zoom continu centré sur le curseur
                    let factor = (1.12f64).powf(y as f64);
                    if let Some(board) = self.store.active_board_mut() {
                        zoom_at(&mut board.viewport, factor, self.mouse_pos.0, self.mouse_pos.1);
                    }
                }
                if x.abs() > 0.001 {
                    self.store.pan(x as f64 * 30.0, 0.0);
                }
            }
            MouseScrollDelta::PixelDelta(p) => {
                if self.modifiers.control_key() {
                    // Pincement tactile / Ctrl + molette = zoom fin
                    let factor = (1.003f64).powf(p.y);
                    if let Some(board) = self.store.active_board_mut() {
                        zoom_at(&mut board.viewport, factor, self.mouse_pos.0, self.mouse_pos.1);
                    }
                } else {
                    // Défilement 2 doigts pavé tactile = pan continu
                    self.store.pan(p.x, p.y);
                }
            }
        }
        self.mark_dirty();
    }

    /// Déplacement relatif de la caméra lors d'un pan souris (bouton milieu, droit, ou Espace+gauche).
    pub fn handle_pan_move(&mut self, dx: f64, dy: f64) {
        // Protection contre les sauts anormaux du curseur OS
        if dx.hypot(dy) < 300.0 {
            self.store.pan(dx, dy);
        }
        self.mark_dirty();
    }
}
