//! Le dessin du panneau STORYBOARD.

use super::{layout_storyboard_panel, StoryboardPanelLayout, StoryboardState};
use crate::dock::paint::{Brush, ButtonLook};
use crate::dock::WidgetRect;
use crate::params::ScaledRect;
use tiny_skia::PixmapMut;

pub fn render_storyboard_panel(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    frame: ScaledRect,
    state: &StoryboardState,
) {
    let layout = layout_storyboard_panel(frame);
    brush.title(pixmap, frame, "STORYBOARD");

    let x = frame.x + brush.px(14.0);
    brush.caption(
        pixmap,
        (x, layout.format_button.y - brush.px(14.0)),
        "FORMAT",
    );
    draw_format(pixmap, brush, layout.format_button, state);

    draw_fields(pixmap, brush, &layout, state);
    for (i, cell) in layout.cells.iter().enumerate() {
        draw_cell(pixmap, brush, *cell, i + 1);
    }

    let look = ButtonLook {
        active: state.active,
        disabled: false,
        quiet: false,
    };
    let label = if state.active {
        "Désactiver"
    } else {
        "Activer"
    };
    brush.button(pixmap, layout.activate_button, label, 12.0, look);
}

/// Le sélecteur de format : un champ avec un chevron.
fn draw_format(pixmap: &mut PixmapMut, brush: &Brush, rect: WidgetRect, state: &StoryboardState) {
    let theme = brush.theme;
    let bg = if brush.hovered(rect) {
        theme.bg_hover
    } else {
        theme.input_bg
    };
    brush.fill(pixmap, rect, brush.px(4.0), bg);
    brush.stroke(pixmap, rect, brush.px(4.0), theme.btn_border, brush.px(1.0));
    let at = (rect.x + brush.px(8.0), rect.y + brush.px(6.0));
    brush.text(
        pixmap,
        state.format_label(),
        at,
        11.0,
        theme.text_primary,
        false,
    );
    let chevron = (rect.x + rect.w - brush.px(16.0), rect.y + brush.px(4.0));
    brush.text(pixmap, "˅", chevron, 12.0, theme.text_muted, false);
}

fn draw_fields(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    layout: &StoryboardPanelLayout,
    state: &StoryboardState,
) {
    let fields = [
        (
            layout.width_input,
            "LARGEUR",
            format!("{}", state.panel_width as i32),
        ),
        (layout.cols_input, "COLONNES", format!("{}", state.cols)),
        (
            layout.gap_input,
            "ESPACEMENT",
            format!("{}", state.gap as i32),
        ),
    ];
    for (rect, caption, value) in fields {
        let above = (rect.x, rect.y - brush.px(12.0));
        brush.text(pixmap, caption, above, 9.0, brush.theme.text_muted, true);
        brush.field(pixmap, rect, &value);
    }
}

/// Une cellule numérotée de la grille.
fn draw_cell(pixmap: &mut PixmapMut, brush: &Brush, rect: WidgetRect, number: usize) {
    let theme = brush.theme;
    brush.fill(pixmap, rect, brush.px(2.0), theme.bg_card);
    brush.stroke(
        pixmap,
        rect,
        brush.px(2.0),
        theme.border_subtle,
        brush.px(0.8),
    );
    brush.text_centered(
        pixmap,
        rect,
        &number.to_string(),
        9.0,
        theme.text_muted,
        false,
    );
}
