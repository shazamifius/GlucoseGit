//! Le dessin du panneau ORDONNER. Il lit la géométrie de [`super::layout_organize_panel`]
//! et l'état ; il ne décide de rien.

use super::{layout_organize_panel, OrganizePanelLayout};
use crate::dock::champ::Champ;
use crate::dock::paint::{Brush, ButtonLook};
use crate::dock::DockManager;
use crate::params::ScaledRect;
use crate::typography::Face;
use glucose_core::store::Store;
use tiny_skia::PixmapMut;

pub fn render_organize_panel(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    frame: ScaledRect,
    dock: &DockManager,
    store: &Store,
) {
    let state = &dock.organize;
    let layout = layout_organize_panel(frame, brush.typo);
    brush.title(pixmap, frame, "ORDONNER");
    draw_target(pixmap, brush, &layout, store);

    let x = frame.x + brush.px(14.0);
    brush.caption(
        pixmap,
        (x, frame.y + brush.px(70.0)),
        "TRIER AVANT DISPOSITION",
    );
    for btn in &layout.sort_buttons {
        let look = ButtonLook {
            active: state.sort_by == btn.sort_type,
            disabled: !btn.sort_type.implemented(),
            quiet: false,
        };
        brush.button(pixmap, btn.rect, btn.sort_type.label(), 10.0, look);
    }

    if let Some(first) = layout.layout_modes.first() {
        brush.caption(pixmap, (x, first.rect.y - brush.px(14.0)), "DISPOSITION");
    }
    for item in &layout.layout_modes {
        draw_layout_mode(
            pixmap,
            brush,
            item.rect,
            item.mode,
            state.layout == item.mode,
        );
    }

    for (champ, rect) in [
        (Champ::OrdonnerTaille, layout.size_rect),
        (Champ::OrdonnerEcart, layout.gap_rect),
        (Champ::OrdonnerColonnes, layout.cols_rect),
    ] {
        if champ.present(dock) {
            crate::dock::champ::paint::peindre(pixmap, brush, dock, champ, rect);
        }
    }
    let look = ButtonLook::default();
    brush.button(pixmap, layout.apply_rect, "Appliquer", 12.0, look);
}

/// « n images sélectionnées » ou « Toutes les images (n) ».
fn draw_target(pixmap: &mut PixmapMut, brush: &Brush, layout: &OrganizePanelLayout, store: &Store) {
    let selected = store.selected_image_ids.len();
    let total = store.active_board().map(|b| b.images.len()).unwrap_or(0);
    let text = match selected {
        0 => format!("Toutes les images ({total})"),
        1 => "1 image sélectionnée".to_string(),
        n => format!("{n} images sélectionnées"),
    };
    let rect = layout.target_count_rect;
    brush.fill(pixmap, rect, brush.px(3.0), brush.theme.bg_card);
    let ink = if selected > 0 {
        brush.theme.text_accent
    } else {
        brush.theme.text_secondary
    };
    let at = (rect.x + brush.px(8.0), rect.y + brush.px(6.0));
    brush.text(pixmap, &text, at, 11.0, ink, Face::Regular);
}

/// Une disposition : son titre et sa description, sur fond quand elle est retenue ou survolée.
fn draw_layout_mode(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    rect: crate::dock::WidgetRect,
    mode: super::LayoutMode,
    active: bool,
) {
    let theme = brush.theme;
    let enabled = mode.implemented();
    let hovered = enabled && brush.hovered(rect);
    if active || hovered {
        let bg = if active {
            theme.bg_active
        } else {
            theme.bg_hover
        };
        brush.fill(pixmap, rect, brush.px(4.0), bg);
    }
    if active {
        brush.stroke(
            pixmap,
            rect,
            brush.px(4.0),
            theme.border_accent,
            brush.px(1.0),
        );
    }
    let ink = match (enabled, active) {
        (false, _) => theme.text_muted,
        (true, true) => theme.text_primary,
        (true, false) => theme.text_secondary,
    };
    let x = rect.x + brush.px(8.0);
    brush.text(
        pixmap,
        mode.title(),
        (x, rect.y + brush.px(4.0)),
        11.0,
        ink,
        if active { Face::Bold } else { Face::Regular },
    );
    brush.text(
        pixmap,
        mode.description(),
        (x, rect.y + brush.px(17.0)),
        9.0,
        theme.text_muted,
        Face::Regular,
    );
}
