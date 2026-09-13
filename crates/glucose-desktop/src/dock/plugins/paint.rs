//! Le dessin du panneau PLUGINS.

use super::{
    layout_plugins_panel, PluginsPanelLayout, PluginsState, DENSITIES, DISPOSITIONS,
    DOWNLOAD_LABEL, ENGINE_DESCRIPTION, ENGINE_NAME, ENGINE_STATUS, MACHINE_STATUS, MODEL_ADVICE,
    OLLAMA_STATUS,
};
use crate::dock::paint::{Brush, ButtonLook};
use crate::dock::WidgetRect;
use crate::params::ScaledRect;
use tiny_skia::PixmapMut;

pub fn render_plugins_panel(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    frame: ScaledRect,
    state: &PluginsState,
) {
    let layout = layout_plugins_panel(frame);
    brush.title(pixmap, frame, "PLUGINS");
    let x = frame.x + brush.px(14.0);

    draw_local_ai(pixmap, brush, frame, &layout);
    brush.caption(pixmap, (x, layout.card_rect.y - brush.px(14.0)), "MOTEUR");
    draw_engine_card(pixmap, brush, layout.card_rect);

    let card_bottom = layout.card_rect.y + layout.card_rect.h;
    brush.caption(pixmap, (x, card_bottom + brush.px(16.0)), "RÉGLAGES");
    draw_choice(
        pixmap,
        brush,
        "Densité",
        &layout.density_options,
        &DENSITIES,
        state.density_idx,
    );
    draw_choice(
        pixmap,
        brush,
        "Disposition",
        &layout.disposition_options,
        &DISPOSITIONS,
        state.disposition_idx,
    );
}

/// L'état d'Ollama et de la machine — gris tant qu'aucune détection n'existe.
fn draw_local_ai(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    frame: ScaledRect,
    layout: &PluginsPanelLayout,
) {
    let theme = brush.theme;
    let x = frame.x + brush.px(14.0);
    brush.caption(pixmap, (x, frame.y + brush.px(38.0)), "IA LOCALE");
    brush.circle(
        pixmap,
        (x + brush.px(4.0), frame.y + brush.px(57.0)),
        brush.px(3.5),
        theme.text_muted,
    );
    brush.text(
        pixmap,
        OLLAMA_STATUS,
        (x + brush.px(12.0), frame.y + brush.px(52.0)),
        11.0,
        theme.text_primary,
        true,
    );
    brush.text(
        pixmap,
        MACHINE_STATUS,
        (x, frame.y + brush.px(68.0)),
        10.0,
        theme.text_muted,
        false,
    );
    brush.text(
        pixmap,
        MODEL_ADVICE,
        (x, frame.y + brush.px(82.0)),
        10.5,
        theme.text_secondary,
        false,
    );
    brush.button(
        pixmap,
        layout.download_button,
        DOWNLOAD_LABEL,
        11.0,
        ButtonLook::default(),
    );
}

/// La carte du moteur : son nom, ce qu'il fera, et ce qu'il en est.
fn draw_engine_card(pixmap: &mut PixmapMut, brush: &Brush, rect: WidgetRect) {
    let theme = brush.theme;
    brush.fill(pixmap, rect, brush.px(6.0), theme.bg_card);
    brush.stroke(
        pixmap,
        rect,
        brush.px(6.0),
        theme.border_subtle,
        brush.px(1.0),
    );
    let x = rect.x + brush.px(8.0);
    brush.text(
        pixmap,
        ENGINE_NAME,
        (x, rect.y + brush.px(8.0)),
        11.5,
        theme.text_primary,
        true,
    );
    brush.text(
        pixmap,
        ENGINE_DESCRIPTION,
        (x, rect.y + brush.px(23.0)),
        9.5,
        theme.text_secondary,
        false,
    );
    brush.text(
        pixmap,
        ENGINE_STATUS,
        (x, rect.y + brush.px(35.0)),
        9.5,
        theme.text_muted,
        false,
    );
}

/// Une liste de boutons radio sous un intitulé.
fn draw_choice(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    heading: &str,
    rows: &[WidgetRect],
    labels: &[&str],
    checked: usize,
) {
    let theme = brush.theme;
    let Some(first) = rows.first() else {
        return;
    };
    brush.text(
        pixmap,
        heading,
        (first.x, first.y - brush.px(14.0)),
        10.5,
        theme.text_secondary,
        true,
    );
    for (i, (row, label)) in rows.iter().zip(labels).enumerate() {
        let is_checked = i == checked;
        brush.radio(
            pixmap,
            (row.x + brush.px(6.0), row.y + brush.px(5.0)),
            is_checked,
        );
        let ink = if is_checked {
            theme.text_primary
        } else {
            theme.text_muted
        };
        brush.text(
            pixmap,
            label,
            (row.x + brush.px(16.0), row.y),
            10.0,
            ink,
            is_checked,
        );
    }
}
