//! **Le dessin d'un bouton** de la barre ou du rail : son fond, son icône, son libellé. Sorti de
//! [`super`] (fiche 55) quand le rail l'a fait passer six cents lignes — la place des boutons
//! d'un côté, leur dessin de l'autre.

use super::{ACTION_LABEL_FONT, ACTION_LABEL_X};
use crate::icons::{draw_icon_scaled, IconType};
use crate::params::{ButtonState, ScaledRect};
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Stroke, Transform};

fn push_ui_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.min(w / 2.0).min(h / 2.0);
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
}

pub fn draw_tool_button(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    rect: ScaledRect,
    icon: IconType,
    state: ButtonState,
) {
    let ScaledRect { x, y, w, h, scale } = rect;
    let ButtonState { active, hover } = state;
    let bg_color = if active {
        theme.bg_active
    } else if hover {
        theme.bg_hover
    } else {
        Color::TRANSPARENT
    };

    if bg_color != Color::TRANSPARENT {
        let mut p = Paint::default();
        p.set_color(bg_color);
        p.anti_alias = true;
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x, y, w, h, 4.0 * scale);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(
                &path,
                &p,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    if active {
        let mut sp = Paint::default();
        sp.set_color(theme.border_accent);
        sp.anti_alias = true;
        let stroke = Stroke {
            width: 1.0 * scale,
            ..Default::default()
        };
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(
            &mut pb,
            x + 0.5 * scale,
            y + 0.5 * scale,
            w - 1.0 * scale,
            h - 1.0 * scale,
            4.0 * scale,
        );
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);
        }
    }

    let icon_color = if active {
        theme.text_accent
    } else if hover {
        theme.text_primary
    } else {
        theme.text_muted
    };

    let icon_size = 14.0 * scale;
    let icon_x = x + (w - icon_size) / 2.0;
    let icon_y = y + (h - icon_size) / 2.0;
    draw_icon_scaled(
        pixmap,
        icon,
        icon_x,
        icon_y,
        icon_size,
        icon_color,
        1.4 * scale,
    );
}

/// Le fond arrondi d'un bouton, et son liseré quand il est actif.
///
/// Les deux vont ensemble : ce sont les deux couches sous le contenu, et elles partagent le
/// même rectangle arrondi.
fn fond_du_bouton(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    rect: ScaledRect,
    state: ButtonState,
    rayon: f32,
) {
    let ScaledRect { x, y, w, h, scale } = rect;
    let ButtonState { active, hover } = state;
    let bg_color = if active {
        theme.bg_active
    } else if hover {
        theme.bg_hover
    } else {
        Color::TRANSPARENT
    };
    if bg_color != Color::TRANSPARENT {
        let mut p = Paint {
            anti_alias: true,
            ..Default::default()
        };
        p.set_color(bg_color);
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x, y, w, h, rayon * scale);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(
                &path,
                &p,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }
    if !active {
        return;
    }
    let mut sp = Paint {
        anti_alias: true,
        ..Default::default()
    };
    sp.set_color(theme.border_accent);
    let stroke = Stroke {
        width: 1.0 * scale,
        ..Default::default()
    };
    // Le liseré se pose sur le demi-pixel : un trait d'un pixel centré sur la frontière
    // s'étalerait en deux demi-teintes.
    let mut pb = PathBuilder::new();
    push_ui_rounded_rect(
        &mut pb,
        x + 0.5 * scale,
        y + 0.5 * scale,
        w - 1.0 * scale,
        h - 1.0 * scale,
        rayon * scale,
    );
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);
    }
}

/// La couleur du contenu d'un bouton, selon ce qu'il vit.
fn teinte_du_contenu(theme: &Theme, state: ButtonState) -> Color {
    if state.active {
        theme.text_accent
    } else if state.hover {
        theme.text_primary
    } else {
        theme.text_secondary
    }
}

pub fn draw_action_button(
    pixmap: &mut PixmapMut,
    typo: &Typography,
    theme: &Theme,
    rect: ScaledRect,
    icon: IconType,
    label: &str,
    state: ButtonState,
) {
    let ScaledRect { x, y, w, h, scale } = rect;
    fond_du_bouton(pixmap, theme, rect, state, 4.0);
    let color = teinte_du_contenu(theme, state);
    let icon_size = 14.0 * scale;
    let icon_y = y + (h - icon_size) / 2.0;
    // Sans libellé, le bouton est carré et son icône est centrée ; avec, elle se range à
    // gauche et le texte suit à une abscisse que la mesure du bouton a déjà réservée.
    if label.is_empty() {
        let icon_x = x + (w - icon_size) / 2.0;
        draw_icon_scaled(pixmap, icon, icon_x, icon_y, icon_size, color, 1.3 * scale);
        return;
    }
    draw_icon_scaled(
        pixmap,
        icon,
        x + 8.0 * scale,
        icon_y,
        icon_size,
        color,
        1.3 * scale,
    );
    let font = ACTION_LABEL_FONT * scale;
    typo.draw_text(
        pixmap,
        label,
        x + ACTION_LABEL_X * scale,
        y + (h - font) / 2.0,
        TextStyle {
            size: font,
            color,
            face: if state.active {
                Face::Bold
            } else {
                Face::Regular
            },
        },
    );
}
