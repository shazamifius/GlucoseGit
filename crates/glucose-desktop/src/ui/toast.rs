//! Le toast : le seul message que l'interface adresse à l'utilisateur.
//!
//! Il ne dit qu'une chose — **ce qui vient d'avoir lieu** — et le cliquet 2 en compte les
//! sites d'émission pour que cette règle ne se perde pas. Un bouton dont l'action n'existe
//! pas est absent ou grisé (fiche 05 § 5.4) ; il n'annonce jamais ce qu'il ne fait pas.

use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use std::time::{Duration, Instant};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Stroke, Transform};

pub struct Toast {
    pub message: String,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    /// Temps de vie à l'écran (fiche 07 § 1, `TOAST_DURATION`).
    pub const DURATION_MS: u64 = 2400;
    /// Apparition (fiche 07 § 1, `TOAST_ANIM_IN`).
    pub const FADE_IN_MS: f32 = 180.0;
    /// Disparition. La référence retire son toast d'un coup ; un fondu est une extension.
    pub const FADE_OUT_MS: f32 = 400.0;

    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            created_at: Instant::now(),
            duration: Duration::from_millis(Self::DURATION_MS),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    pub fn alpha(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_millis() as f32;
        let total = self.duration.as_millis() as f32;
        if elapsed > total - Self::FADE_OUT_MS {
            ((total - elapsed) / Self::FADE_OUT_MS).clamp(0.0, 1.0)
        } else {
            (elapsed / Self::FADE_IN_MS).clamp(0.0, 1.0)
        }
    }

    /// Ce que le toast demande à la boucle : `Redraw` pendant un fondu, `Sleep(ms)` sur le
    /// plateau jusqu'au début du fondu sortant, `Gone` une fois expiré. La boucle n'a ainsi
    /// aucun chiffre à connaître — les siens, dupliqués, avaient déjà divergé une fois.
    pub fn repaint_need(&self) -> ToastRepaint {
        if self.is_expired() {
            return ToastRepaint::Gone;
        }
        let elapsed = self.created_at.elapsed().as_millis() as f32;
        let total = self.duration.as_millis() as f32;
        let fade_out_at = total - Self::FADE_OUT_MS;
        if elapsed < Self::FADE_IN_MS || elapsed >= fade_out_at {
            ToastRepaint::Redraw
        } else {
            ToastRepaint::Sleep((fade_out_at - elapsed).ceil().max(1.0) as u64)
        }
    }
}

/// Voir [`Toast::repaint_need`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastRepaint {
    Redraw,
    Sleep(u64),
    Gone,
}

/// Une couleur du thème, fondue par l'opacité du toast.
///
/// Jamais tout à fait nulle tant que le toast est là : un octet d'alpha à zéro ne se
/// distingue pas d'un toast absent, et le fondu sauterait sur sa dernière image.
fn fondue(couleur: Color, alpha: f32) -> Color {
    Color::from_rgba8(
        (couleur.red() * 255.0) as u8,
        (couleur.green() * 255.0) as u8,
        (couleur.blue() * 255.0) as u8,
        ((couleur.alpha() * alpha * 255.0) as u8).max(1),
    )
}

/// La pilule du toast : un rectangle dont les bouts sont des demi-cercles.
fn pilule(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.finish()
}

pub fn render_toast(
    pixmap: &mut PixmapMut,
    toast: &Toast,
    typo: &Typography,
    theme: &Theme,
    w: f32,
    h: f32,
    scale: f32,
) {
    let alpha = toast.alpha();
    if alpha <= 0.01 {
        return;
    }
    let s = crate::theme::clamp_ui_scale(scale);
    let (tw, _) = typo.measure_text(&toast.message, 13.0 * s, Face::Regular);
    let (toast_w, toast_h) = (tw + 40.0 * s, 36.0 * s);
    let (toast_x, toast_y) = ((w - toast_w) / 2.0, h - 64.0 * s);

    if let Some(path) = pilule(toast_x, toast_y, toast_w, toast_h, 18.0 * s) {
        let mut bg_paint = Paint::default();
        bg_paint.set_color(fondue(theme.toast_bg, alpha));
        pixmap.fill_path(
            &path,
            &bg_paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );
        let mut border_paint = Paint::default();
        border_paint.set_color(fondue(theme.toast_border, alpha));
        let stroke = Stroke {
            width: 1.0 * s,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }

    typo.draw_text(
        pixmap,
        &toast.message,
        toast_x + 20.0 * s,
        toast_y + (toast_h - 13.0 * s) / 2.0,
        TextStyle {
            size: 13.0 * s,
            color: fondue(theme.toast_text, alpha),
            face: Face::Regular,
        },
    );
}
