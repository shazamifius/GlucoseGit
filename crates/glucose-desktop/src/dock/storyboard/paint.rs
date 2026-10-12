//! Le dessin du panneau STORYBOARD.

use super::{layout_storyboard_panel, StoryboardState};
use crate::dock::champ::Champ;
use crate::dock::paint::{Brush, ButtonLook};
use crate::dock::DockManager;
use crate::dock::WidgetRect;
use crate::params::ScaledRect;
use crate::typography::Face;
use tiny_skia::PixmapMut;

pub fn render_storyboard_panel(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    frame: ScaledRect,
    dock: &DockManager,
) {
    let state = &dock.storyboard;
    let layout = layout_storyboard_panel(frame);
    brush.title(pixmap, frame, "STORYBOARD");

    let x = frame.x + brush.px(14.0);
    brush.caption(
        pixmap,
        (x, layout.format_button.y - brush.px(14.0)),
        "FORMAT",
    );
    draw_format(pixmap, brush, layout.format_button, state);

    for (champ, rect) in [
        (Champ::StoryLargeur, layout.width_input),
        (Champ::StoryColonnes, layout.cols_input),
        (Champ::StoryEcart, layout.gap_input),
    ] {
        crate::dock::champ::paint::peindre(pixmap, brush, dock, champ, rect);
    }
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
        Face::Regular,
    );
    let chevron = (rect.x + rect.w - brush.px(16.0), rect.y + brush.px(4.0));
    brush.text(pixmap, "˅", chevron, 12.0, theme.text_muted, Face::Regular);
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
        Face::Regular,
    );
}
