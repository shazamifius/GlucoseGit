//! Le dessin du panneau POMODORO : l'anneau, le temps, les boutons.

use super::{layout_pomodoro_panel, PomodoroPanelLayout, PomodoroState};
use crate::dock::paint::{Brush, ButtonLook};
use crate::params::ScaledRect;
use crate::typography::Face;
use tiny_skia::{LineCap, Paint, PathBuilder, PixmapMut, Stroke, Transform};

/// Épaisseur de l'anneau, en unités de la fiche.
const RING_WIDTH: f32 = 4.5;
/// Le nombre de segments qui approchent l'arc de progression.
const ARC_STEPS: usize = 60;

pub fn render_pomodoro_panel(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    frame: ScaledRect,
    state: &PomodoroState,
) {
    let layout = layout_pomodoro_panel(frame, brush.typo);
    let at = (frame.x + brush.px(14.0), frame.y + brush.px(18.0));
    brush.text(
        pixmap,
        "POMODORO",
        at,
        10.0,
        brush.theme.text_muted,
        Face::Bold,
    );

    draw_ring(pixmap, brush, &layout, state);
    draw_time(pixmap, brush, &layout, state);

    let look = ButtonLook::default();
    brush.button(pixmap, layout.start_button, state.start_label(), 11.0, look);
    brush.button(pixmap, layout.reset_button, "↺", 12.0, look);
    for preset in &layout.presets {
        let look = ButtonLook {
            active: state.total_seconds == preset.seconds,
            disabled: false,
            quiet: true,
        };
        brush.button(pixmap, preset.rect, preset.label, 9.5, look);
    }
}

/// L'anneau : la piste, puis l'arc parcouru.
///
/// La chrome est monochrome : l'arc est blanc. La référence le faisait bleu — la dette que
/// `style.md` avoue — et le port l'avait mis sur l'accent, qui n'est pas pour cela. Le vert
/// de fin est une couleur de contenu (fiche 10 § 1 : « validation Pomodoro »).
fn draw_ring(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    layout: &PomodoroPanelLayout,
    state: &PomodoroState,
) {
    let width = brush.px(RING_WIDTH);
    brush.ring(
        pixmap,
        layout.ring_center,
        layout.ring_radius,
        brush.theme.border_subtle,
        width,
    );

    let color = if state.left_seconds == 0 {
        brush.theme.success
    } else {
        brush.theme.text_accent
    };
    // À zéro, rien n'est parcouru : un segment nul à extrémité ronde poserait un point
    // blanc en haut de l'anneau, qui se lit comme une graduation.
    let steps = (state.progress() * ARC_STEPS as f32) as usize;
    if steps == 0 {
        return;
    }
    let (cx, cy) = layout.ring_center;
    let mut pb = PathBuilder::new();
    for i in 0..=steps {
        let angle =
            -std::f32::consts::FRAC_PI_2 + (i as f32 / ARC_STEPS as f32) * std::f32::consts::TAU;
        let point = (
            cx + layout.ring_radius * angle.cos(),
            cy + layout.ring_radius * angle.sin(),
        );
        if i == 0 {
            pb.move_to(point.0, point.1);
        } else {
            pb.line_to(point.0, point.1);
        }
    }
    let Some(arc) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(color);
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        ..Default::default()
    };
    pixmap.stroke_path(&arc, &paint, &stroke, Transform::identity(), None);
}

/// « 25:00 », centré dans l'anneau.
fn draw_time(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    layout: &PomodoroPanelLayout,
    state: &PomodoroState,
) {
    let text = format!(
        "{:02}:{:02}",
        state.left_seconds / 60,
        state.left_seconds % 60
    );
    let (w, _) = brush.typo.measure_text(&text, brush.px(16.0), Face::Bold);
    let (cx, cy) = layout.ring_center;
    let at = (cx - w / 2.0, cy - brush.px(8.0));
    brush.text(
        pixmap,
        &text,
        at,
        16.0,
        brush.theme.text_primary,
        Face::Bold,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::Pointer;
    use crate::theme::Theme;
    use crate::typography::Typography;
    use tiny_skia::Pixmap;

    const FRAME: ScaledRect = ScaledRect {
        x: 0.0,
        y: 0.0,
        w: 170.0,
        h: 210.0,
        scale: 1.0,
    };

    /// La couleur au sommet de l'anneau — l'endroit exact où l'arc commence. Compter l'encre
    /// du panneau entier ne dirait rien : l'arc se dessine **par-dessus** la piste, et le
    /// nombre de pixels encrés ne change donc pas.
    fn sommet(pixmap: &Pixmap) -> (u8, u8, u8) {
        let typo = Typography::new();
        let layout = layout_pomodoro_panel(FRAME, &typo);
        let (cx, cy) = layout.ring_center;
        let px = pixmap
            .pixel(cx as u32, (cy - layout.ring_radius) as u32)
            .expect("le sommet de l'anneau est dans le pixmap");
        (px.red(), px.green(), px.blue())
    }

    fn rgb(color: tiny_skia::Color) -> (u8, u8, u8) {
        let c = color.to_color_u8();
        (c.red(), c.green(), c.blue())
    }

    fn rendu(state: &PomodoroState) -> (Pixmap, Theme) {
        let theme = Theme::dark();
        let typo = Typography::new();
        let mut pixmap = Pixmap::new(200, 240).expect("pixmap");
        pixmap.fill(theme.bg_canvas);
        let brush = Brush {
            typo: &typo,
            theme: &theme,
            s: 1.0,
            pointer: Pointer { x: -1.0, y: -1.0 },
        };
        render_pomodoro_panel(&mut pixmap.as_mut(), &brush, FRAME, state);
        (pixmap, theme)
    }

    /// **L'anneau ne pose pas de point à zéro.** Un segment nul à extrémité ronde en posait
    /// un au sommet de l'anneau, qui se lisait comme une graduation — et il était là au
    /// démarrage, c'est-à-dire la plupart du temps.
    #[test]
    fn test_the_ring_draws_nothing_until_something_has_elapsed() {
        let (intact, theme) = rendu(&PomodoroState::default());
        assert_eq!(
            sommet(&intact),
            rgb(theme.border_subtle),
            "au repos, le sommet de l'anneau est la piste"
        );

        let entame = PomodoroState {
            left_seconds: PomodoroState::default().total_seconds / 2,
            ..PomodoroState::default()
        };
        let (parcouru, theme) = rendu(&entame);
        assert_eq!(
            sommet(&parcouru),
            rgb(theme.text_accent),
            "dès qu'une part est écoulée, l'arc part du sommet"
        );
    }
}
