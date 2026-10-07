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
pub(crate) fn pilule(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
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

/// **Le toast, placé** : sa pilule, et chaque ligne avec le haut de son texte.
///
/// Une ligne tant qu'elle tient — au bureau, rien ne change. Plus large que l'écran moins ses
/// marges, le message se **coupe en lignes** (WRAP-1) au lieu de sortir de l'écran : sur un
/// téléphone de 360 points, « Glucose reçoit les images et les liens : ce partage n'en porte
/// aucun » en fait 450 (fiche 56).
pub(crate) fn placer_le_toast(
    message: &str,
    typo: &Typography,
    (w, h): (f32, f32),
    scale: f32,
) -> (crate::ui::question::Rangee, Vec<(String, f32)>) {
    let s = crate::theme::clamp_ui_scale(scale);
    let (corps, pad_x, une_ligne) = (13.0 * s, 20.0 * s, 36.0 * s);
    let largeur_max = (w - 2.0 * (16.0 * s + pad_x)).max(corps);
    let avance = |_: usize, c: char| typo.advance(c, corps, Face::Regular);
    let lignes: Vec<String> = crate::renderer::wrap::wrap_paragraph(message, largeur_max, avance)
        .into_iter()
        .map(|(debut, fin)| message[debut..fin].to_string())
        .collect();
    let pas = corps * 1.4;
    let large = lignes
        .iter()
        .map(|l| typo.measure_text(l, corps, Face::Regular).0)
        .fold(0.0f32, f32::max);
    let (toast_w, toast_h) = (
        large + 2.0 * pad_x,
        une_ligne + (lignes.len().max(1) - 1) as f32 * pas,
    );
    let (toast_x, toast_y) = ((w - toast_w) / 2.0, h - 28.0 * s - toast_h);
    let haut = toast_y + (une_ligne - corps) / 2.0;
    let lignes = lignes
        .into_iter()
        .enumerate()
        .map(|(i, l)| (l, haut + i as f32 * pas))
        .collect();
    ((toast_x, toast_y, toast_w, toast_h), lignes)
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
    let ((toast_x, toast_y, toast_w, toast_h), lignes) =
        placer_le_toast(&toast.message, typo, (w, h), scale);

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

    for (ligne, haut) in &lignes {
        typo.draw_text(
            pixmap,
            ligne,
            toast_x + 20.0 * s,
            *haut,
            TextStyle {
                size: 13.0 * s,
                color: fondue(theme.toast_text, alpha),
                face: Face::Regular,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Au bureau, une ligne, comme toujours** ; **au téléphone, le message se coupe** et
    /// chaque ligne tient dans l'écran, marges comprises (fiche 56).
    #[test]
    fn test_le_toast_se_coupe_plutot_que_de_sortir_de_l_ecran() {
        let typo = Typography::new();
        let message = crate::plateforme::partage::RIEN;
        let (pilule, lignes) = placer_le_toast(message, &typo, (1440.0, 900.0), 1.0);
        assert_eq!(lignes.len(), 1, "au bureau, une ligne");
        assert_eq!(
            (pilule.1, pilule.3),
            (900.0 - 64.0, 36.0),
            "à sa place de toujours"
        );

        let ecran = (720.0, 1600.0);
        let ((x, _, w, _), lignes) = placer_le_toast(message, &typo, ecran, 2.0);
        assert!(lignes.len() >= 2, "au téléphone, le message se coupe");
        assert!(
            x >= 32.0 && x + w <= ecran.0 - 32.0,
            "la pilule tient dans l'écran"
        );
        for (ligne, _) in &lignes {
            let (lw, _) = typo.measure_text(ligne, 26.0, Face::Regular);
            assert!(
                x + 40.0 + lw <= x + w + 0.5,
                "« {ligne} » déborde de la pilule"
            );
        }
    }
}
