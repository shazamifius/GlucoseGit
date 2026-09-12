//! Interaction de navigation caméra : Pan & Zoom centré sur curseur (PureRef-style).

use crate::app::GlucoseApp;
use winit::event::MouseScrollDelta;

/// Bornes du zoom **au geste** — molette et pincement (fiche 07 § 7.1) : de ×50 dézoomé à
/// ×20 zoomé. Plus étroites que celles du modèle ([`glucose_core::types::Viewport::SCALE_RANGE`]),
/// qu'un signet ou un fichier peuvent atteindre sans que la main y arrive.
pub const WHEEL_SCALE_RANGE: (f64, f64) = (0.02, 20.0);

impl GlucoseApp {
    /// Gère les événements de molette et gestes tactiles pour le zoom continu et le pan.
    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let (cx, cy) = self.mouse_pos;
        match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                if y.abs() > 0.001 {
                    // Zoom continu centré sur le curseur
                    let factor = (1.12f64).powf(y as f64);
                    self.store.zoom(factor, cx, cy, WHEEL_SCALE_RANGE);
                }
                if x.abs() > 0.001 {
                    self.store.pan(x as f64 * 30.0, 0.0);
                }
            }
            MouseScrollDelta::PixelDelta(p) => {
                if self.modifiers.control_key() {
                    // Pincement tactile / Ctrl + molette = zoom fin
                    let factor = (1.003f64).powf(p.y);
                    self.store.zoom(factor, cx, cy, WHEEL_SCALE_RANGE);
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
