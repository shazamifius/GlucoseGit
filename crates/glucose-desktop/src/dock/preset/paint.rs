//! Le dessin du panneau PRESETS.

use super::{layout_presets_panel, PresetRowLayout, PresetSpec, CREATE_LABEL, PRESETS};
use crate::dock::paint::{Brush, ButtonLook};
use crate::params::ScaledRect;
use tiny_skia::{Color, PixmapMut};

/// L'opacité du fond d'une zone, sur 255 — son contour, lui, est à la couleur pleine.
const SLOT_FILL_ALPHA: u8 = 55;

pub fn render_presets_panel(pixmap: &mut PixmapMut, brush: &Brush, frame: ScaledRect) {
    let layout = layout_presets_panel(frame);
    brush.title(pixmap, frame, "PRESETS");
    brush.text(
        pixmap,
        "CHOISIR UN PRESET POUR \"BOARD PRINCIPAL\"",
        layout.caption_at,
        9.0,
        brush.theme.text_muted,
        true,
    );
    for row in &layout.rows {
        draw_preset(pixmap, brush, row, &PRESETS[row.index]);
    }
    let look = ButtonLook {
        active: false,
        disabled: true,
        quiet: false,
    };
    brush.button(pixmap, layout.create_button, CREATE_LABEL, 10.5, look);
}

/// Un gabarit : son titre, la rangée de ses zones, son résumé.
fn draw_preset(pixmap: &mut PixmapMut, brush: &Brush, row: &PresetRowLayout, preset: &PresetSpec) {
    let theme = brush.theme;
    brush.text(
        pixmap,
        preset.name,
        row.title_at,
        11.5,
        theme.text_primary,
        true,
    );
    for (rect, slot) in row.slots.iter().zip(preset.slots) {
        let color = slot.color();
        let c = color.to_color_u8();
        let fill = Color::from_rgba8(c.red(), c.green(), c.blue(), SLOT_FILL_ALPHA);
        brush.fill(pixmap, *rect, brush.px(2.0), fill);
        brush.stroke(pixmap, *rect, brush.px(2.0), color, brush.px(0.8));
        brush.text_centered(pixmap, *rect, slot.label, 7.5, color, false);
    }
    brush.text(
        pixmap,
        preset.summary,
        row.summary_at,
        9.0,
        theme.text_muted,
        false,
    );
}
